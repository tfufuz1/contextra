//! Write-Ahead Log (WAL) for durability and crash recovery with HMAC chaining.

pub mod encode;
pub mod flusher;
pub mod hmac;
pub mod io;
pub mod replay;

#[cfg(test)]
mod replay_tests;

pub use encode::*;
pub use flusher::*;
pub(crate) use hmac::*;
pub(crate) use io::*;
pub(crate) use replay::*;

use memfuse_core::{MemFuseError, Result};
use memfuse_crypto::crypto::KeyManager;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Maximum WAL size before triggering a flush (128MB).
pub const MAX_WAL_SIZE: u64 = 128 * 1024 * 1024;

/// Maximum size for a single WAL entry payload (64MB).
pub const MAX_WAL_ENTRY_SIZE: u32 = 64 * 1024 * 1024;

pub static FAIL_APPEND_FOR_TX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static DELAY_APPEND_FOR_TX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static DELAY_APPEND_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct WalConfig {
    pub key_manager: Option<Arc<KeyManager>>,
    pub allow_legacy_integrity_key_fallback: bool,
    pub min_wal_version: WalVersion,
    pub flusher_config: WalFlusherConfig,
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            key_manager: None,
            allow_legacy_integrity_key_fallback: false,
            min_wal_version: WalVersion::V3,
            flusher_config: WalFlusherConfig::default(),
        }
    }
}

pub struct Wal {
    pub(crate) path: PathBuf,
    pub(crate) file: Arc<tokio::sync::Mutex<tokio::fs::File>>,
    pub(crate) size: Arc<std::sync::atomic::AtomicU64>,
    pub(crate) header_written: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) key_manager: Option<Arc<KeyManager>>,
    pub(crate) fallback_integrity_key: Option<[u8; 32]>,
    pub(crate) allow_legacy_integrity_key_fallback: bool,
    pub(crate) last_hmac: Arc<tokio::sync::Mutex<[u8; 32]>>,
    pub(crate) flusher_tx:
        std::sync::RwLock<Option<tokio::sync::mpsc::UnboundedSender<FlusherMessage>>>,
    pub(crate) sealed: Arc<std::sync::atomic::AtomicBool>,
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
            let uuid_bytes = Self::load_or_create_wal_uuid(&path).await?;
            (Some(Arc::new(km.derive_file_key(&uuid_bytes)?)), None)
        } else {
            let key = Self::load_or_create_integrity_key(&path).await?;
            (None, Some(key))
        };

        let (file, is_new) = match tokio::fs::OpenOptions::new()
            .create_new(true)
            .append(true)
            .read(true)
            .open(&path)
            .await
        {
            Ok(file) => (file, true),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let file = tokio::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .read(true)
                    .open(&path)
                    .await
                    .map_err(|e| MemFuseError::Storage(format!("Failed to open WAL: {}", e)))?;
                (file, false)
            }
            Err(e) => {
                return Err(MemFuseError::Storage(format!(
                    "Failed to create WAL: {}",
                    e
                )));
            }
        };

        // 🛡️ SICHERUNG: Directory FSync (FIND-STO-004 / Task G)
        if is_new {
            file.sync_all().await.map_err(|e| {
                MemFuseError::Storage(format!(
                    "WAL file fsync failed for {}: {}",
                    path.display(),
                    e
                ))
            })?;
            crate::util::fsync_parent_dir(&path).await?;
        }

        let metadata = file
            .metadata()
            .await
            .map_err(|e| MemFuseError::Storage(e.to_string()))?;

        let wal = Self {
            path: path.clone(),
            size: Arc::new(std::sync::atomic::AtomicU64::new(metadata.len())),
            header_written: Arc::new(std::sync::atomic::AtomicBool::new(metadata.len() > 0)),
            file: Arc::new(tokio::sync::Mutex::new(file)),
            key_manager: derived_key_manager,
            fallback_integrity_key,
            allow_legacy_integrity_key_fallback: config.allow_legacy_integrity_key_fallback,
            last_hmac: Arc::new(tokio::sync::Mutex::new([0u8; 32])),
            flusher_tx: std::sync::RwLock::new(None),
            sealed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };

        wal.enable_flusher_with_config(config.flusher_config);

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
                let copy_res = tokio::fs::copy(&wal.path, &bak_path).await;
                if copy_res.is_ok() {
                    // Backup-Datei fsyncen: Recovery-Sicherheit VOR der Truncation der Original-WAL.
                    match tokio::fs::OpenOptions::new()
                        .write(true)
                        .open(&bak_path)
                        .await
                    {
                        Ok(bak_file) => {
                            if let Err(e) = bak_file.sync_all().await {
                                tracing::warn!("WAL backup fsync failed before rewrite: {e}");
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Could not reopen WAL backup for fsync: {e}");
                        }
                    }
                }
                let rewrite_res = wal.rewrite_as_v3(&entries).await;

                if copy_res.is_err() || rewrite_res.is_err() {
                    if config.min_wal_version > WalVersion::V1 || version < config.min_wal_version {
                        let err_msg = match (copy_res, rewrite_res) {
                            (Err(e), _) => {
                                format!("Failed to create backup copy {:?}: {}", bak_path, e)
                            }
                            (_, Err(e)) => format!("Failed to rewrite WAL as V3: {}", e),
                            (Ok(_), Ok(_)) => unreachable!(),
                        };
                        return Err(MemFuseError::invalid_input(format!(
                            "Configuration error: WAL version {:?} is below min_wal_version {:?} and migration failed: {}",
                            version, config.min_wal_version, err_msg
                        )));
                    } else {
                        rewrite_res?;
                    }
                }
            } else if let Some((_, last_entry, _)) = entries.last() {
                let mut guard = wal.last_hmac.lock().await;
                *guard = last_entry.checksum;
            }
        }

        wal.enable_flusher_with_config(config.flusher_config);

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
}
