//! FILE-CONTEXT: STAND/ZWECK/INVARIANTEN
//! Stand: 2026-10-06
//! Zweck: Write-Ahead Log (WAL) core structure, lifecycle management, and recovery.
//! Invarianten:
//! - I-3 (Crash-Konsistenz): Restores Log up to last HMAC-verified frame.
//! - I-5 (Poison-State Isolation): Flusher panics in close() are evaluated and propagated as ContextraError::Internal.

pub mod encode;
pub mod flusher;
pub mod hmac;
pub mod io;
pub mod replay;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod open_heal_tests;

pub use encode::*;
pub use flusher::*;
#[cfg(feature = "wal-integrity")]
pub(crate) use hmac::*;
pub use replay::*;

#[cfg(not(loom))]
pub(crate) mod fs {
    pub use tokio::fs::*;
}

#[cfg(loom)]
#[allow(dead_code)]
pub(crate) mod fs {
    use std::path::Path;

    // INVARIANT-KONFORM: Exklusiv in loom (Single-Thread / Loom-Simulation) für Mock-File-I/O verwendet.
    pub async fn read<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
        std::fs::read(path)
    }
    // INVARIANT-KONFORM: Exklusiv in loom (Single-Thread / Loom-Simulation) für Mock-File-I/O verwendet.
    #[allow(dead_code)]
    pub async fn write<P: AsRef<Path>, C: AsRef<[u8]>>(
        path: P,
        contents: C,
    ) -> std::io::Result<()> {
        std::fs::write(path, contents)
    }
    pub async fn remove_file<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        let _ = path;
        Ok(())
    }
    pub async fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> std::io::Result<()> {
        let _ = (from, to);
        Ok(())
    }
    pub async fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> std::io::Result<u64> {
        let _ = (from, to);
        Ok(0)
    }
    pub async fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> std::io::Result<()> {
        let _ = (from, to);
        Ok(())
    }
    pub async fn try_exists<P: AsRef<Path>>(path: P) -> std::io::Result<bool> {
        let _ = path;
        Ok(false)
    }
    // INVARIANT-KONFORM: Perm-Signatur in Loom-Mock-Typen.
    pub async fn set_permissions<P: AsRef<Path>>(
        path: P,
        perm: std::fs::Permissions,
    ) -> std::io::Result<()> {
        let _ = (path, perm);
        Ok(())
    }
    pub async fn metadata<P: AsRef<Path>>(path: P) -> std::io::Result<LoomMetadata> {
        let _ = path;
        Ok(LoomMetadata)
    }

    #[derive(Debug, Clone)]
    pub struct LoomMetadata;
    impl LoomMetadata {
        pub fn len(&self) -> u64 {
            0
        }
        // INVARIANT-KONFORM: Perm-Signatur und Mode-Konstruktion in Loom-Mock-Metadata.
        pub fn permissions(&self) -> std::fs::Permissions {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                // INVARIANT-KONFORM: Constructing Permissions from unix mode mask in Loom mock metadata.
                std::fs::Permissions::from_mode(0o644)
            }
            #[cfg(not(unix))]
            {
                // INVARIANT-KONFORM: Non-unix fallback in Loom mock metadata.
                let path = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
                match std::fs::metadata(&path) {
                    Ok(m) => m.permissions(),
                    Err(_) => {
                        match std::fs::metadata(std::env::temp_dir()) {
                            Ok(m) => m.permissions(),
                            Err(e) => {
                                tracing::error!("Failed to query Loom mock permissions: {e}");
                                std::fs::metadata(&path).map(|m| m.permissions()).unwrap_or_else(|_| {
                                tracing::error!("Retrying Loom mock permissions query failed: {e}");
                                match std::fs::File::open(&path).and_then(|f| f.metadata()) {
                                    Ok(m) => m.permissions(),
                                    Err(err) => {
                                        tracing::error!("Loom mock permissions fallback error: {err}");
                                        std::fs::metadata(&path).map(|m| m.permissions()).unwrap_or_else(|_| {
                                            std::fs::metadata(&path).expect("Loom mock permissions fallback failure")
                                        })
                                    }
                                }
                            })
                            }
                        }
                    }
                }
            }
        }
    }

    pub struct OpenOptions;
    impl OpenOptions {
        pub fn new() -> Self {
            Self
        }
        pub fn read(&mut self, _read: bool) -> &mut Self {
            self
        }
        pub fn write(&mut self, _write: bool) -> &mut Self {
            self
        }
        pub fn create(&mut self, _create: bool) -> &mut Self {
            self
        }
        pub fn create_new(&mut self, _create_new: bool) -> &mut Self {
            self
        }
        pub fn append(&mut self, _append: bool) -> &mut Self {
            self
        }
        pub fn mode(&mut self, _mode: u32) -> &mut Self {
            self
        }
        pub async fn open<P: AsRef<Path>>(&self, _path: P) -> std::io::Result<LoomFile> {
            Ok(LoomFile::new())
        }
    }

    #[derive(Default)]
    pub struct LoomFile {
        pub pos: usize,
        pub buf: Vec<u8>,
    }

    impl LoomFile {
        pub fn new() -> Self {
            Self {
                pos: 0,
                buf: Vec::new(),
            }
        }
        pub async fn open<P: AsRef<Path>>(_path: P) -> std::io::Result<Self> {
            Ok(Self::new())
        }
        pub async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
            self.buf.extend_from_slice(buf);
            Ok(())
        }
        pub async fn flush(&self) -> std::io::Result<()> {
            Ok(())
        }
        pub async fn sync_all(&self) -> std::io::Result<()> {
            Ok(())
        }
        pub async fn set_len(&mut self, len: u64) -> std::io::Result<()> {
            self.buf.truncate(len as usize);
            Ok(())
        }
        pub async fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
            match pos {
                std::io::SeekFrom::Start(offset) => self.pos = offset as usize,
                std::io::SeekFrom::End(offset) => {
                    self.pos = (self.buf.len() as i64 + offset).max(0) as usize;
                }
                std::io::SeekFrom::Current(offset) => {
                    self.pos = (self.pos as i64 + offset).max(0) as usize;
                }
            }
            Ok(self.pos as u64)
        }
        pub async fn metadata(&self) -> std::io::Result<LoomMetadata> {
            Ok(LoomMetadata)
        }
    }

    impl tokio::io::AsyncRead for LoomFile {
        fn poll_read(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            let pos = self.pos;
            if pos < self.buf.len() {
                let rem = &self.buf[pos..];
                let amt = rem.len().min(buf.remaining());
                buf.put_slice(&rem[..amt]);
                self.pos += amt;
            }
            std::task::Poll::Ready(Ok(()))
        }
    }

    impl tokio::io::AsyncWrite for LoomFile {
        fn poll_write(
            mut self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            self.buf.extend_from_slice(buf);
            std::task::Poll::Ready(Ok(buf.len()))
        }
        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            std::task::Poll::Ready(Ok(()))
        }
        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            std::task::Poll::Ready(Ok(()))
        }
    }

    impl tokio::io::AsyncSeek for LoomFile {
        fn start_seek(
            mut self: std::pin::Pin<&mut Self>,
            pos: std::io::SeekFrom,
        ) -> std::io::Result<()> {
            match pos {
                std::io::SeekFrom::Start(offset) => self.pos = offset as usize,
                std::io::SeekFrom::End(offset) => {
                    self.pos = (self.buf.len() as i64 + offset).max(0) as usize;
                }
                std::io::SeekFrom::Current(offset) => {
                    self.pos = (self.pos as i64 + offset).max(0) as usize;
                }
            }
            Ok(())
        }
        fn poll_complete(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<u64>> {
            std::task::Poll::Ready(Ok(self.pos as u64))
        }
    }

    pub type File = LoomFile;
}

use contextra_core::{ContextraError, Result};
#[cfg(feature = "wal-integrity")]
pub use contextra_crypto::crypto::KeyManager;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(not(feature = "wal-integrity"))]
#[derive(Debug, Clone)]
pub struct KeyManager;

#[cfg(not(feature = "wal-integrity"))]
impl KeyManager {
    pub fn try_new(_passphrase: &str, _salt: &[u8]) -> Result<Self> {
        Ok(Self)
    }
    pub fn derive_file_key(&self, _file_id: &[u8]) -> Result<Self> {
        Ok(Self)
    }
    pub fn encrypt_auto_nonce(&self, _plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        Ok((_plaintext.to_vec(), [0u8; 12]))
    }
    pub fn decrypt_auto_nonce(&self, _ciphertext: &[u8], _nonce: &[u8; 12]) -> Result<Vec<u8>> {
        Ok(_ciphertext.to_vec())
    }
    pub fn integrity_key(&self) -> Result<[u8; 32]> {
        Ok([0u8; 32])
    }
}

/// Maximum WAL size before triggering a flush (128MB).
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

#[derive(Debug, Clone)]
pub struct WalConfig {
    pub key_manager: Option<Arc<KeyManager>>,
    pub(crate) allow_legacy_integrity_key_fallback: bool,
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

impl WalConfig {
    pub fn with_legacy_fallback(mut self, allow: bool) -> Self {
        self.allow_legacy_integrity_key_fallback = allow;
        self
    }
}

pub struct Wal {
    pub(crate) path: PathBuf,
    pub(crate) size: Arc<std::sync::atomic::AtomicU64>,
    pub(crate) header_written: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) key_manager: Option<Arc<KeyManager>>,
    pub(crate) fallback_integrity_key: Option<[u8; 32]>,
    pub(crate) allow_legacy_integrity_key_fallback: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) min_wal_version: WalVersion,
    pub(crate) legacy_key_used: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) was_legacy_rekeyed: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) last_hmac: Arc<tokio::sync::Mutex<[u8; 32]>>,
    pub(crate) flusher_tx: std::sync::RwLock<Option<tokio::sync::mpsc::Sender<WalCommand>>>,
    pub(crate) flusher_task: std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
    pub(crate) sealed: Arc<std::sync::atomic::AtomicBool>,
    pub truncate_lock: Arc<tokio::sync::Mutex<()>>,
    pub(crate) poisoned: Arc<std::sync::atomic::AtomicBool>,
}

impl Drop for Wal {
    fn drop(&mut self) {
        if let Ok(mut tx_guard) = self.flusher_tx.write() {
            tx_guard.take();
        }
        if let Ok(mut task_guard) = self.flusher_task.lock() {
            if let Some(task) = task_guard.take() {
                task.abort();
            }
        }
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

        if self
            .was_legacy_rekeyed
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            Self::write_migration_marker_atomically(&self.path).await?;
            self.allow_legacy_integrity_key_fallback
                .store(false, std::sync::atomic::Ordering::SeqCst);
            self.legacy_key_used
                .store(false, std::sync::atomic::Ordering::SeqCst);
            return Ok(true);
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
        #[cfg(all(not(feature = "wal-integrity"), not(feature = "memory-only-storage")))]
        {
            let _ = (path, config);
            return Err(ContextraError::Storage(
                "WAL integrity verification is disabled at compile time ('wal-integrity' feature missing). Refusing to open persistent WAL without integrity checks.".into(),
            ));
        }

        #[cfg(any(feature = "wal-integrity", feature = "memory-only-storage"))]
        {
            let path = path.as_ref().to_path_buf();
            let has_marker = Self::has_migration_marker(&path).await;
            let allow_legacy_fallback = if has_marker {
                false
            } else {
                config.allow_legacy_integrity_key_fallback
            };

            #[cfg(feature = "wal-integrity")]
            let (derived_key_manager, fallback_integrity_key) = if let Some(km) = config.key_manager
            {
                let uuid_bytes = Self::load_or_create_wal_uuid(&path).await?;
                (Some(Arc::new(km.derive_file_key(&uuid_bytes)?)), None)
            } else {
                let key = Self::load_or_create_integrity_key(&path).await?;
                (None, Some(key))
            };

            #[cfg(not(feature = "wal-integrity"))]
            let (derived_key_manager, fallback_integrity_key) = {
                let key = Self::load_or_create_integrity_key(&path).await?;
                (None, Some(key))
            };

            let file_len = self::fs::metadata(&path)
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
                min_wal_version: config.min_wal_version,
                legacy_key_used: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                was_legacy_rekeyed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                last_hmac: Arc::new(tokio::sync::Mutex::new([0u8; 32])),
                flusher_tx: std::sync::RwLock::new(None),
                flusher_task: std::sync::Mutex::new(None),
                sealed: Arc::new(std::sync::atomic::AtomicBool::new(true)),
                truncate_lock: Arc::new(tokio::sync::Mutex::new(())),
                poisoned: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            })
        }
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
        #[cfg(all(not(feature = "wal-integrity"), not(feature = "memory-only-storage")))]
        {
            let _ = (path, config);
            return Err(ContextraError::Storage(
                "WAL integrity verification is disabled at compile time ('wal-integrity' feature missing). Refusing to open persistent WAL without integrity checks.".into(),
            ));
        }

        #[cfg(any(feature = "wal-integrity", feature = "memory-only-storage"))]
        {
            let path = path.as_ref().to_path_buf();

            // Vor dem eigentlichen Öffnen der WAL-Datei: prüfen, ob ein Crash-Recovery aus
            // einem .bak-Backup nötig ist (z. B. Crash zwischen set_len(0) und V3-Rewrite).
            let _ = recover_from_bak_if_present(&path).await?;

            // SD-09-CRYPTO-002: Use a persisted UUID v4 as file_id instead of the
            // filename.  This makes the WAL's cryptographic sub-key independent of
            // the filesystem path — renaming or moving the file cannot cause nonce-
            // reuse between two WAL instances sharing the same master key.
            #[cfg(feature = "wal-integrity")]
            let (derived_key_manager, fallback_integrity_key) = if let Some(km) = config.key_manager
            {
                let uuid_bytes = Self::load_or_create_wal_uuid(&path).await?;
                (Some(Arc::new(km.derive_file_key(&uuid_bytes)?)), None)
            } else {
                let key = Self::load_or_create_integrity_key(&path).await?;
                (None, Some(key))
            };

            #[cfg(not(feature = "wal-integrity"))]
            let (derived_key_manager, fallback_integrity_key) = {
                let key = Self::load_or_create_integrity_key(&path).await?;
                (None, Some(key))
            };

            let (file, is_new) = match self::fs::OpenOptions::new()
                .create_new(true)
                .append(true)
                .read(true)
                .open(&path)
                .await
            {
                Ok(file) => (file, true),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    let file = self::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .read(true)
                        .open(&path)
                        .await
                        .map_err(|e| {
                            ContextraError::Storage(format!("Failed to open WAL: {}", e))
                        })?;
                    (file, false)
                }
                Err(e) => {
                    return Err(ContextraError::Storage(format!(
                        "Failed to create WAL: {}",
                        e
                    )));
                }
            };

            // 🛡️ SICHERUNG: Directory FSync (FIND-STO-004 / Task G)
            if is_new {
                file.sync_all().await.map_err(|e| {
                    ContextraError::Storage(format!(
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
                .map_err(|e| ContextraError::Storage(e.to_string()))?;

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
                min_wal_version: config.min_wal_version,
                legacy_key_used: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                was_legacy_rekeyed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                last_hmac: Arc::new(tokio::sync::Mutex::new([0u8; 32])),
                flusher_tx: std::sync::RwLock::new(None),
                flusher_task: std::sync::Mutex::new(None),
                sealed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                truncate_lock: Arc::new(tokio::sync::Mutex::new(())),
                poisoned: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            };

            wal.enable_flusher_with_config(file, config.flusher_config)?;

            // If file is not empty, find the last valid HMAC to continue the chain
            if metadata.len() > 0 {
                let (entries, version) = wal.replay_with_size_and_version(metadata.len()).await?;
                if version < config.min_wal_version
                    || (version != WalVersion::V3 && !allow_legacy_fallback)
                {
                    return Err(ContextraError::invalid_input(format!(
                        "WAL format version {:?} is disallowed by configuration (min_wal_version: {:?}, allow_legacy_integrity_key_fallback: {}). Explicit migration via open_for_legacy_migration / migrate_legacy_wal required.",
                        version, config.min_wal_version, allow_legacy_fallback
                    )));
                }

                if version != WalVersion::V3 {
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
                    let copy_res = self::fs::copy(&wal.path, &bak_path).await;
                    if copy_res.is_ok() {
                        // Backup-Datei fsyncen: Recovery-Sicherheit VOR der Truncation der Original-WAL.
                        let bak_file = self::fs::OpenOptions::new()
                            .write(true)
                            .open(&bak_path)
                            .await
                            .map_err(|e| {
                                ContextraError::Storage(format!(
                                    "Could not reopen WAL backup for fsync: {e}"
                                ))
                            })?;
                        bak_file.sync_all().await.map_err(|e| {
                            ContextraError::Storage(format!(
                                "WAL backup fsync failed before rewrite: {e}"
                            ))
                        })?;
                        crate::util::fsync_parent_dir(&bak_path).await?;
                    }
                    let rewrite_res = wal.rewrite_as_v3(&entries).await;

                    if copy_res.is_err() || rewrite_res.is_err() {
                        let err_msg = match (copy_res, rewrite_res) {
                            (Err(e), _) => {
                                format!("Failed to create backup copy {:?}: {}", bak_path, e)
                            }
                            (_, Err(e)) => format!("Failed to rewrite WAL as V3: {}", e),
                            (Ok(_), Ok(_)) => {
                                "Unexpected state during backup and rewrite".to_string()
                            }
                        };
                        return Err(ContextraError::invalid_input(format!(
                            "Configuration error: WAL version {:?} is below min_wal_version {:?} and migration failed: {}",
                            version, config.min_wal_version, err_msg
                        )));
                    } else {
                        wal.was_legacy_rekeyed
                            .store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                } else {
                    // TODO(Implementer): [P01 / F-02 / HIGH / JULES-P01-02]
                    // Unvollständige Truncation partieller Frames beim Replay auf Open (Invariante I-3):
                    // Sicherstellen, dass `verified_end` exakt der physischen Endposition des letzten vollständig
                    // verifizierten HMAC-Frames entspricht (auch bei V3-AEAD-Verschlüsselung mit Nonce/Tag).
                    // Müll- oder Nulldaten nach einem Absturz müssen deterministisch abgeschnitten werden.
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
    }

    /// Gracefully closes the WAL, stopping the background flusher task and waiting for it to exit.
    pub async fn close(&self) -> Result<()> {
        self.sealed.store(true, std::sync::atomic::Ordering::SeqCst);
        let flusher_tx = {
            let mut guard = self
                .flusher_tx
                .write()
                .map_err(|_| ContextraError::Storage("flusher_tx RwLock poisoned".into()))?;
            guard.take()
        };
        drop(flusher_tx);

        let flusher_task = {
            let mut guard = self
                .flusher_task
                .lock()
                .map_err(|_| ContextraError::Storage("flusher_task Mutex poisoned".into()))?;
            guard.take()
        };

        if let Some(task) = flusher_task {
            if let Err(join_err) = task.await {
                return Err(ContextraError::Internal(format!(
                    "WAL flusher task panicked: {join_err}"
                )));
            }
        }

        Ok(())
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

    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(std::sync::atomic::Ordering::SeqCst)
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

    pub fn abort_flusher_for_test(&self) {
        if let Ok(task_guard) = self.flusher_task.lock() {
            if let Some(task) = task_guard.as_ref() {
                task.abort();
            }
        }
    }

    /// Recovers a poisoned `Wal` handle after a suspected torn write event.
    ///
    /// Re-verifies physical file state via offline replay against the last valid HMAC chain,
    /// updates `size` and `last_hmac` to the verified end state, physically truncates any
    /// orphan/partial bytes at the tail, and resets the `poisoned` flag to `false`.
    ///
    /// **This is the only supported mechanism to recover a poisoned `Wal` handle back into service.**
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

    pub async fn recover_from_poison(&self) -> Result<()> {
        if !self.is_poisoned() {
            return Ok(());
        }

        // Replay all entries up to the last valid HMAC-verified entry
        let entries = self.replay().await?;

        let metadata = self::fs::metadata(&self.path).await.map_err(|e| {
            ContextraError::Storage(format!("Failed to stat WAL for poison recovery: {e}"))
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
            return Err(ContextraError::wal_corruption(
                verified_offset,
                "Verified end offset exceeds physical file length during poison recovery",
            ));
        }

        // Reset poisoned flag temporarily to allow truncation command to be executed by flusher
        self.poisoned
            .store(false, std::sync::atomic::Ordering::SeqCst);

        if let Err(e) = self.truncate(verified_offset, verified_hmac).await {
            self.poisoned
                .store(true, std::sync::atomic::Ordering::SeqCst);
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
            min_wal_version: WalVersion::V3,
            legacy_key_used: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            was_legacy_rekeyed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            last_hmac: Arc::new(tokio::sync::Mutex::new([0u8; 32])),
            flusher_tx: std::sync::RwLock::new(None),
            flusher_task: std::sync::Mutex::new(None),
            sealed: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            truncate_lock: Arc::new(tokio::sync::Mutex::new(())),
            poisoned: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };
        if let Err(e) = wal.enable_flusher_with_config(
            self::fs::LoomFile::new(),
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
