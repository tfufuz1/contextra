//! Kern-Struktur und Basismethoden für das Write-Ahead-Log.
//!
//! Invarianten:
//! - INV-STORE-1: Durabilität durch Vorwegschreiben ins Log.
//! - INV-FSYNC-1: Fail-closed Verhalten bei Fehlern.

use super::config::WalConfig;
use super::flusher::{WalCommand, WalFlusherConfig};
use super::poison::PoisonCause;
use super::replay::recover_from_bak_if_present;
use super::encode::*;
use super::fs;
use super::hmac::*;
use crate::error::{StoreError, StoreResult as Result};
use contextra_crypto::crypto::KeyManager;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const MAX_WAL_SIZE: u64 = 128 * 1024 * 1024;

/// Maximum size for a single WAL entry payload (64MB).
pub const MAX_WAL_ENTRY_SIZE: u32 = 64 * 1024 * 1024;

/// Default capacity for the bounded WAL flusher command channel.
pub const DEFAULT_WAL_QUEUE_CAPACITY: usize = 1_024;

pub static FAIL_APPEND_FOR_TX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static DELAY_APPEND_FOR_TX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static DELAY_APPEND_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg(feature = "fault-injection")]
pub static FAIL_TRUNCATE_ONCE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(feature = "fault-injection")]
pub static FAIL_APPEND_AFTER_PARTIAL_BYTES: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

#[cfg(feature = "fault-injection")]
pub static FAIL_APPEND_PARTIAL_ONCE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

// ANCHOR[contextra-store:WAL-CORE-002]
/// Hauptstruktur fuer das Write-Ahead-Log mit Append- und Replay-Operationen.
pub struct Wal {
    pub(crate) path: PathBuf,
    pub(crate) size: Arc<std::sync::atomic::AtomicU64>,
    pub(crate) header_written: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) key_manager: Option<Arc<KeyManager>>,
    pub(crate) fallback_integrity_key: Option<[u8; 32]>,
    pub(crate) allow_legacy_integrity_key_fallback: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) legacy_key_used: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) was_legacy_rekeyed: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) last_hmac: Arc<tokio::sync::Mutex<[u8; 32]>>,
    pub(crate) flusher_tx: std::sync::RwLock<Option<tokio::sync::mpsc::Sender<WalCommand>>>,
    pub(crate) flusher_task: std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
    pub(crate) sealed: Arc<std::sync::atomic::AtomicBool>,
    pub truncate_lock: Arc<tokio::sync::Mutex<()>>,
    pub(crate) poison_state: Arc<std::sync::atomic::AtomicU8>,
}

impl Drop for Wal {
    fn drop(&mut self) {
        // No-op: Flusher task loop terminates naturally when all flusher_tx senders are dropped (rx.recv() returns None).
    }
}

impl std::fmt::Debug for Wal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wal")
            .field("path", &self.path)
            .field("size", &self.size())
            .finish()
    }
}

impl Wal {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_config(path, WalConfig::default()).await
    }

    /// Explicitly migrates a legacy WAL segment to the V3 format signed with a secure integrity key.
    ///
    /// **INV-WAL-LEGACY-KEY-1**: Legacy migration requires explicit operator invocation.
    /// Fallback to the legacy static integrity key is NEVER enabled implicitly during standard database open.
    ///
    /// # Prerequisites for Future Permanent Removal of `LEGACY_INTEGRITY_KEY_OBFUSCATED`:
    /// 1. Operator migration complete across all active deployments (zero unmigrated V1/V2 WAL files).
    /// 2. Every migrated WAL segment has a `.rekeyed` retirement marker on disk and V3 payload format.
    /// 3. Audit verification (`has_pending_legacy_wal_migration_path`) returns `false` across all paths.
    /// 4. Feature gate `legacy-wal-key` deprecation window elapsed.
    ///
    /// Returns `Ok(true)` if legacy entries were detected and successfully rekeyed to V3 format,
    /// or `Ok(false)` if the segment was already migrated or contained no legacy entries.
    pub async fn migrate_legacy_wal(
        path: impl AsRef<Path>,
        key_manager: Option<Arc<KeyManager>>,
    ) -> Result<bool> {
        let path_ref = path.as_ref();
        if Self::has_migration_marker(path_ref).await {
            tracing::info!(
                wal_path = %path_ref.display(),
                "WAL segment already has migration marker; no legacy migration needed."
            );
            return Ok(false);
        }

        let config = WalConfig {
            allow_legacy_integrity_key_fallback: true,
            key_manager,
            min_wal_version: WalVersion::V1,
            ..Default::default()
        };
        let wal = Self::open_with_config(path_ref, config).await?;
        let was_rekeyed = wal.rekey_from_legacy().await?;
        wal.was_legacy_rekeyed
            .store(was_rekeyed, std::sync::atomic::Ordering::SeqCst);
        Ok(was_rekeyed)
    }

    /// Explicitly opens a legacy WAL file requiring migration using the legacy integrity key fallback.
    ///
    /// **INV-WAL-LEGACY-KEY-1**: This is the ONLY entry point permitted to enable
    /// `allow_legacy_integrity_key_fallback = true` for legacy WAL migration.
    ///
    /// # Forced Migration Workflow
    /// Calling this function automatically triggers a one-time forced migration (rekeying):
    /// - **(a) Automated Rekeying**: Replays all entries using the static XOR legacy key fallback and
    ///   rewrites the WAL log as V3 entries signed with a freshly generated, secure, file-local
    ///   integrity key (persisted in `.wal_integrity_key`).
    /// - **(b) Permanent Opt-Out**: Upon successful migration, a `.rekeyed` migration marker is
    ///   atomically written to disk, permanently disabling legacy fallback for this `Wal` handle and
    ///   for future process restarts on this file path.
    /// - **(c) Cryptographic Isolation**: The static XOR-obfuscated legacy key (`wal/hmac.rs`)
    ///   remains intentionally weak solely for one-time transitional reading of legacy files; it is
    ///   never used for new writes after migration.
    /// - **(d) Error Boundary**: If rekeying fails (e.g., I/O write error), an `Err(...)` is returned,
    ///   preventing silent ongoing operation in legacy mode.
    pub async fn open_for_legacy_migration(
        path: impl AsRef<Path>,
        key_manager: Option<Arc<KeyManager>>,
    ) -> Result<Self> {
        let path_ref = path.as_ref();
        if Self::has_migration_marker(path_ref).await {
            tracing::info!(
                "WAL {:?} already has migration marker; opening via standard secure path.",
                path_ref
            );
            return Self::open_with_key_manager(path_ref, key_manager).await;
        }

        let config = WalConfig {
            allow_legacy_integrity_key_fallback: true,
            key_manager,
            min_wal_version: WalVersion::V1,
            ..Default::default()
        };
        let wal = Self::open_with_config(path_ref, config).await?;
        let was_rekeyed = wal.rekey_from_legacy().await?;
        wal.was_legacy_rekeyed
            .store(was_rekeyed, std::sync::atomic::Ordering::SeqCst);
        Ok(wal)
    }

    /// Performs forced rekeying of a legacy WAL segment to a secure integrity key.
    /// Returns `true` if legacy key entries were detected and rekeyed.
    pub(crate) async fn rekey_from_legacy(&self) -> Result<bool> {
        if Self::has_migration_marker(&self.path).await {
            self.allow_legacy_integrity_key_fallback
                .store(false, std::sync::atomic::Ordering::SeqCst);
            self.legacy_key_used
                .store(false, std::sync::atomic::Ordering::SeqCst);
            return Ok(false);
        }

        if !self
            .legacy_key_used
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            Self::write_migration_marker_atomically(&self.path).await?;
            self.allow_legacy_integrity_key_fallback
                .store(false, std::sync::atomic::Ordering::SeqCst);
            return Ok(false);
        }

        // 1. Replay all entries using current legacy-enabled handle
        let entries = self.replay().await?;

        // 2. Ensure secure integrity key exists on disk
        let _secure_key = Self::load_or_create_integrity_key(&self.path).await?;

        // 3. Rewrite all entries using the secure key
        self.rewrite_as_v3(&entries).await?;

        // 4. Atomically write the migration marker file
        Self::write_migration_marker_atomically(&self.path).await?;

        // 5. Permanently disable legacy fallback and clear legacy_key_used flag
        self.allow_legacy_integrity_key_fallback
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.legacy_key_used
            .store(false, std::sync::atomic::Ordering::SeqCst);

        tracing::info!(
            "WAL segment {:?} successfully rekeyed from legacy static key to secure key.",
            self.path
        );

        Ok(true)
    }

    /// Opens an existing WAL file solely for read-only replay, without spawning
    /// a background write flusher actor or keeping write handles open.
    pub async fn open_read_only(
        path: impl AsRef<Path>,
        key_manager: Option<Arc<KeyManager>>,
    ) -> Result<Self> {
        Self::open_read_only_with_config(
            path,
            WalConfig {
                key_manager,
                ..Default::default()
            },
        )
        .await
    }

    /// Opens an existing WAL file solely for read-only replay with explicit configuration.
    pub async fn open_read_only_with_config(
        path: impl AsRef<Path>,
        config: WalConfig,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let has_marker = Self::has_migration_marker(&path).await;
        let allow_legacy_fallback = if has_marker {
            false
        } else {
            config.allow_legacy_integrity_key_fallback
        };

        let (derived_key_manager, fallback_integrity_key) = if let Some(km) = config.key_manager {
            let uuid_bytes =
                Self::load_or_create_wal_uuid_with_gen(&path, config.id_gen.as_deref()).await?;
            (Some(Arc::new(km.derive_file_key(&uuid_bytes)?)), None)
        } else {
            let key = Self::load_or_create_integrity_key(&path).await?;
            (None, Some(key))
        };

        let file_len = fs::metadata(&path)
            .await
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(Self {
            path,
            size: Arc::new(std::sync::atomic::AtomicU64::new(file_len)),
            header_written: Arc::new(std::sync::atomic::AtomicBool::new(file_len > 0)),
            key_manager: derived_key_manager,
            fallback_integrity_key,
            allow_legacy_integrity_key_fallback: Arc::new(std::sync::atomic::AtomicBool::new(
                allow_legacy_fallback,
            )),
            legacy_key_used: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            was_legacy_rekeyed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            last_hmac: Arc::new(tokio::sync::Mutex::new([0u8; 32])),
            flusher_tx: std::sync::RwLock::new(None),
            flusher_task: std::sync::Mutex::new(None),
            sealed: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            truncate_lock: Arc::new(tokio::sync::Mutex::new(())),
            poison_state: Arc::new(std::sync::atomic::AtomicU8::new(0)),
        })
    }

    /// Opens or creates a WAL file with an optional KeyManager.
    pub async fn open_with_key_manager(
        path: impl AsRef<Path>,
        key_manager: Option<Arc<KeyManager>>,
    ) -> Result<Self> {
        Self::open_with_config(
            path,
            WalConfig {
                key_manager,
                ..Default::default()
            },
        )
        .await
    }

    /// Opens or creates a WAL file with explicit configuration options.
    pub async fn open_with_config(path: impl AsRef<Path>, config: WalConfig) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Vor dem eigentlichen Öffnen der WAL-Datei: prüfen, ob ein Crash-Recovery aus
        // einem .bak-Backup nötig ist (z. B. Crash zwischen set_len(0) und V3-Rewrite).
        let _ = recover_from_bak_if_present(&path).await?;

        // SD-09-CRYPTO-002: Use a persisted UUID v4 as file_id instead of the
        // filename.  This makes the WAL's cryptographic sub-key independent of
        // the filesystem path — renaming or moving the file cannot cause nonce-
        // reuse between two WAL instances sharing the same master key.
        let (derived_key_manager, fallback_integrity_key) = if let Some(km) = config.key_manager {
            let uuid_bytes =
                Self::load_or_create_wal_uuid_with_gen(&path, config.id_gen.as_deref()).await?;
            (Some(Arc::new(km.derive_file_key(&uuid_bytes)?)), None)
        } else {
            let key = Self::load_or_create_integrity_key(&path).await?;
            (None, Some(key))
        };

        let (file, is_new) = match fs::OpenOptions::new()
            .create_new(true)
            .append(true)
            .read(true)
            .open(&path)
            .await
        {
            Ok(file) => (file, true),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .read(true)
                    .open(&path)
                    .await
                    .map_err(|e| StoreError::Storage(format!("Failed to open WAL: {}", e)))?;
                (file, false)
            }
            Err(e) => {
                return Err(StoreError::Storage(format!(
                    "Failed to create WAL: {}",
                    e
                )));
            }
        };

        // 🛡️ SICHERUNG: Directory FSync (FIND-STO-004 / Task G)
        if is_new {
            file.sync_all().await.map_err(|e| {
                StoreError::Storage(format!(
                    "WAL file fsync failed for {}: {}",
                    path.display(),
                    e
                ))
            })?;
            crate::fsync::sync_parent_dir(&path).await?;
        }

        let metadata = file
            .metadata()
            .await
            .map_err(|e| StoreError::Storage(e.to_string()))?;

        let has_marker = Self::has_migration_marker(&path).await;
        let allow_legacy_fallback = if has_marker {
            false
        } else {
            config.allow_legacy_integrity_key_fallback
        };

        let wal = Self {
            path: path.clone(),
            size: Arc::new(std::sync::atomic::AtomicU64::new(metadata.len())),
            header_written: Arc::new(std::sync::atomic::AtomicBool::new(metadata.len() > 0)),
            key_manager: derived_key_manager,
            fallback_integrity_key,
            allow_legacy_integrity_key_fallback: Arc::new(std::sync::atomic::AtomicBool::new(
                allow_legacy_fallback,
            )),
            legacy_key_used: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            was_legacy_rekeyed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            last_hmac: Arc::new(tokio::sync::Mutex::new([0u8; 32])),
            flusher_tx: std::sync::RwLock::new(None),
            flusher_task: std::sync::Mutex::new(None),
            sealed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            truncate_lock: Arc::new(tokio::sync::Mutex::new(())),
            poison_state: Arc::new(std::sync::atomic::AtomicU8::new(0)),
        };

        wal.enable_flusher_with_config(file, config.flusher_config)?;

        // If file is not empty, find the last valid HMAC to continue the chain
        if metadata.len() > 0 {
            let (entries, version) = wal.replay_with_size_and_version(metadata.len()).await?;
            if version < config.min_wal_version || version != WalVersion::V3 {
                tracing::info!(
                    "WAL {:?} format detected at {:?}. Will be rewritten as V3 after successful replay.",
                    version,
                    wal.path
                );
                let bak_suffix = match version {
                    WalVersion::V1 => "v1.bak",
                    WalVersion::V2 => "v2.bak",
                    WalVersion::V3 => "v3.bak",
                };
                let bak_path = PathBuf::from(format!("{}.{}", wal.path.display(), bak_suffix));
                let copy_res = fs::copy(&wal.path, &bak_path).await;
                if copy_res.is_ok() {
                    // Backup-Datei fsyncen: Recovery-Sicherheit VOR der Truncation der Original-WAL.
                    let bak_file = fs::OpenOptions::new()
                        .write(true)
                        .open(&bak_path)
                        .await
                        .map_err(|e| {
                            StoreError::Storage(format!(
                                "Could not reopen WAL backup for fsync: {e}"
                            ))
                        })?;
                    bak_file.sync_all().await.map_err(|e| {
                        StoreError::Storage(format!(
                            "WAL backup fsync failed before rewrite: {e}"
                        ))
                    })?;
                    crate::fsync::sync_parent_dir(&bak_path).await?;
                }
                let rewrite_res = wal.rewrite_as_v3(&entries).await;

                if copy_res.is_err() || rewrite_res.is_err() {
                    if config.min_wal_version > WalVersion::V1 || version < config.min_wal_version {
                        let err_msg = match (copy_res, rewrite_res) {
                            (Err(e), _) => {
                                format!("Failed to create backup copy {:?}: {}", bak_path, e)
                            }
                            (_, Err(e)) => format!("Failed to rewrite WAL as V3: {}", e),
                            (Ok(_), Ok(_)) => {
                                "Unexpected state during backup and rewrite".to_string()
                            }
                        };
                        return Err(StoreError::invalid_input(format!(
                            "Configuration error: WAL version {:?} is below min_wal_version {:?} and migration failed: {}",
                            version, config.min_wal_version, err_msg
                        )));
                    } else {
                        rewrite_res?;
                    }
                }
            } else {
                let (verified_end, last_hmac) =
                    if let Some((_, last_entry, end_pos)) = entries.last() {
                        (*end_pos, last_entry.checksum)
                    } else if metadata.len() >= 4 {
                        (4u64, [0u8; 32])
                    } else {
                        (0u64, [0u8; 32])
                    };

                if verified_end < metadata.len() {
                    let discarded_bytes = metadata.len() - verified_end;
                    tracing::warn!(
                        wal_path = %wal.path.display(),
                        discarded_bytes,
                        verified_end,
                        file_len = metadata.len(),
                        "Truncating torn WAL write tail on open"
                    );
                    wal.truncate(verified_end, last_hmac).await?;
                } else {
                    let mut guard = wal.last_hmac.lock().await;
                    *guard = last_hmac;
                }
            }
        }

        Ok(wal)
    }

    /// Helper to expose integrity key for tests
    pub fn size(&self) -> u64 {
        self.size.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_sealed(&self) -> bool {
        self.sealed.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Gibt an, ob das Handle vergiftet ist.
    pub fn is_poisoned(&self) -> bool {
        self.poison_state.load(std::sync::atomic::Ordering::SeqCst) != 0
    }

    /// Gibt die typisierte Ursache des Poisoning zurück, sofern vorhanden.
    pub fn poison_cause(&self) -> Option<PoisonCause> {
        let code = self.poison_state.load(std::sync::atomic::Ordering::SeqCst);
        PoisonCause::from_u8(code)
    }

    /// Vergiftet das Handle mit der angegebenen typisierten Ursache.
    ///
    /// Das erste Setzen gewinnt (`compare_exchange`), um die primäre Ursache deterministisch zu bewahren.
    pub fn poison_with_cause(&self, cause: PoisonCause) {
        let _ = self.poison_state.compare_exchange(
            0,
            cause.to_u8(),
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        );
    }


    pub fn poison_for_test(&self) {
        self.poison_with_cause(PoisonCause::WriteFailed);
    }


    pub fn allow_legacy_fallback_for_test(&self) -> bool {
        self.allow_legacy_integrity_key_fallback
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn legacy_key_used_for_test(&self) -> bool {
        self.legacy_key_used
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn was_legacy_rekeyed(&self) -> bool {
        self.was_legacy_rekeyed
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Truncates any uncommitted entries appearing after the last committed `TxEnd` marker in the WAL.
    ///
    /// Returns the number of bytes truncated.
    ///
    /// **Policy Note**: `truncate_uncommitted_tail` is NOT invoked during `Wal::open`.
    /// Transaction commitment policy belongs in the higher LSM/Store layer; low-level
    /// WAL unit tests and single-operation Put loggers write entries without explicit `TxEnd` markers.
    /// In-flight transactions (Puts without `TxEnd`) left behind after a crash are pruned on demand by LSM recovery.
    pub async fn truncate_uncommitted_tail(&self) -> Result<u64> {
        let entries = self.replay().await?;
        let current_len = self.size();

        let mut last_tx_end: Option<(WalEntry, u64)> = None;
        for (_, entry, pos) in &entries {
            if matches!(entry.op, WalOp::TxEnd { .. }) {
                last_tx_end = Some((entry.clone(), *pos));
            }
        }

        if let Some((last_entry, last_pos)) = last_tx_end {
            if last_pos < current_len {
                let truncated_bytes = current_len - last_pos;
                self.truncate(last_pos, last_entry.checksum).await?;
                return Ok(truncated_bytes);
            }
        }

        Ok(0)
    }

    /// Stellt ein mit `PoisonCause::TornWrite` vergiftetes `Wal`-Handle nach einem partiellen Schreibabbruch wieder her.
    ///
    /// Re-verifiziert den physischen Dateizustand per Replay gegen die letzte gültige HMAC-Kette,
    /// aktualisiert `size` und `last_hmac` auf den verifizierten Endzustand, schneidet verwaiste / partielle
    /// Bytes am Dateiende physisch ab und setzt den Poison-Zustand auf `0` (nicht vergiftet) zurück.
    ///
    /// **Dies ist der einzige erlaubte Mechanismus, um ein vergiftetes `Wal`-Handle wieder in Betrieb zu nehmen.**
    /// Gemäß SPEC §12.1 und ADR-002 sind alle anderen Ursachen (`FsyncFailed`, `WriteFailed`, `TruncateFailed`,
    /// `HmacChainDiverged`) pro Handle-Instanz **irreversibel**. Ein Wiederherstellungsversuch für diese Ursachen
    /// wird deterministisch mit einem Fehler abgewiesen.
    ///
    /// # Errors
    /// Gibt einen Fehler zurück, wenn:
    /// - Das Handle mit einer irreversiblen Ursache vergiftet ist (`FsyncFailed`, `WriteFailed`, `TruncateFailed`, `HmacChainDiverged`).
    /// - Das Replay fehlschlägt oder die HMAC-Kette vor dem verifizierten Ende inkonsistent ist.
    /// - Das Dateisystem-Truncate fehlschlägt.
    ///
    /// # Crash
    /// Bricht die Wiederherstellung während des Dateisystem-Truncates ab, bleibt das Handle vergiftet.
    /// Ein erneuter Start liest beim nächsten Öffnen das WAL bis zum letzten gültigen HMAC-Präfix ein.
    pub async fn recover_from_poison(&self) -> Result<()> {
        let cause = match self.poison_cause() {
            Some(c) => c,
            None => return Ok(()),
        };

        if cause != PoisonCause::TornWrite {
            return Err(StoreError::Storage(format!(
                "Irreversible WAL poison state ({cause:?}); handle cannot be recovered into service (SPEC §12.1, INV-FSYNC-1)"
            )));
        }

        // Replay all entries up to the last valid HMAC-verified entry
        let entries = self.replay().await?;

        let metadata = fs::metadata(&self.path).await.map_err(|e| {
            StoreError::Storage(format!("Failed to stat WAL for poison recovery: {e}"))
        })?;
        let file_len = metadata.len();

        let (verified_offset, verified_hmac) =
            if let Some((_, last_entry, end_pos)) = entries.last() {
                (*end_pos, last_entry.checksum)
            } else if file_len >= 4
                && self
                    .header_written
                    .load(std::sync::atomic::Ordering::Acquire)
            {
                (4u64, [0u8; 32])
            } else {
                (0u64, [0u8; 32])
            };

        if verified_offset > file_len {
            return Err(StoreError::wal_corruption(
                verified_offset,
                "Verified end offset exceeds physical file length during poison recovery",
            ));
        }

        // Reset poison state temporarily to allow truncation command to be executed by flusher
        self.poison_state
            .store(0, std::sync::atomic::Ordering::SeqCst);

        if let Err(e) = self.truncate(verified_offset, verified_hmac).await {
            self.poison_state.store(
                PoisonCause::TornWrite.to_u8(),
                std::sync::atomic::Ordering::SeqCst,
            );
            return Err(e);
        }

        self.size
            .store(verified_offset, std::sync::atomic::Ordering::SeqCst);
        let mut hmac_guard = self.last_hmac.lock().await;
        *hmac_guard = verified_hmac;

        Ok(())
    }
}

#[cfg(loom)]
impl Wal {
    pub fn open_loom() -> Self {
        let wal = Self {
            path: PathBuf::from("loom.wal"),
            size: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            header_written: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            key_manager: None,
            fallback_integrity_key: Some([1u8; 32]),
            allow_legacy_integrity_key_fallback: Arc::new(std::sync::atomic::AtomicBool::new(
                false,
            )),
            legacy_key_used: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            was_legacy_rekeyed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            last_hmac: Arc::new(tokio::sync::Mutex::new([0u8; 32])),
            flusher_tx: std::sync::RwLock::new(None),
            flusher_task: std::sync::Mutex::new(None),
            sealed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            truncate_lock: Arc::new(tokio::sync::Mutex::new(())),
            poison_state: Arc::new(std::sync::atomic::AtomicU8::new(0)),
        };
        if let Err(e) = wal.enable_flusher_with_config(
            fs::LoomFile::new(),
            WalFlusherConfig {
                batch_window_micros: 0,
                queue_capacity: DEFAULT_WAL_QUEUE_CAPACITY,
            },
        ) {
            tracing::error!("Failed to enable flusher in loom: {e}");
        }
        wal
    }
}
