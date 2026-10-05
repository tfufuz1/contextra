// ZWECK: Legacy HMAC key migration path, deobfuscation and integrity key status.

use contextra_core::{ContextraError, Result};
use std::path::{Path, PathBuf};

use super::{PreparedBatch, Wal, WalEntry, WalOp};

#[cfg(feature = "legacy-wal-key")]
const LEGACY_KEY_OBFUSCATION_MASK: u8 = 0x5A;

/// Obfuscated legacy static HMAC integrity key used strictly for backward-compatibility fallback
/// during WAL replay of legacy databases (INV-WAL-LEGACY-KEY-1).
///
/// # PREREQUISITES FOR FUTURE PERMANENT REMOVAL OF `LEGACY_INTEGRITY_KEY_OBFUSCATED`:
/// To safely remove this constant, the static key fallback, and `deobfuscate_legacy_integrity_key()`
/// in a future separate step without breaking existing operational databases, the following
/// conditions MUST be verified and fulfilled:
///
/// 1. **Complete Operator Migration**: All legacy WAL files (V1/V2) across all active deployments
///    must have been explicitly migrated using `Wal::migrate_legacy_wal` or `open_for_legacy_migration`.
/// 2. **Retirement Marker Verification**: Every stored WAL segment in active deployment data
///    directories must possess a corresponding `.rekeyed` retirement marker file containing a valid
///    BLAKE3 key binding and be rewritten in V3 format.
/// 3. **Zero Pending Migrations**: Audit checks via `LsmStorage::has_pending_legacy_wal_migration_path`
///    return `false` across all operational environments, proven by an operator migration report audit artifact.
/// 4. **Feature Gate Deprecation**: Cargo feature `legacy-wal-key` is formally deprecated and removed
///    from crate feature definitions, enforcing fail-closed compilation errors on legacy key usage.
#[cfg(feature = "legacy-wal-key")]
const LEGACY_INTEGRITY_KEY_OBFUSCATED: [u8; 32] = *b"954.?\".(;w34.?=(3.#w1?#w,kZZZZZZ";

#[cfg(feature = "legacy-wal-key")]
static LEGACY_KEY_WARN_ONCE: std::sync::Once = std::sync::Once::new();

/// Emits a process-wide one-time warning log when the legacy HMAC integrity key path is used.
///
/// **SECURITY**: This message intentionally contains NO key material or secret data.
#[cfg(feature = "legacy-wal-key")]
pub(crate) fn warn_legacy_key_use_once(context_msg: &str) {
    LEGACY_KEY_WARN_ONCE.call_once(|| {
        tracing::warn!(
            target: "contextra_store::wal::legacy_key",
            event = "legacy_integrity_key_accessed",
            context = %context_msg,
            "Legacy-WAL-Integritätsschlüssel im Einsatz ({context_msg}) — dieses Format bietet keine echte Manipulationssicherheit, da der Schlüssel öffentlich ist. Bitte migriere die WAL-Datei auf ein neues Schlüsselformat."
        );
    });
}

/// Status indicating whether a WAL segment or key configuration uses the obfuscated legacy key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LegacyKeyStatus {
    /// A standard, secure integrity key (e.g. from `.wal_integrity_key` or `KeyManager`) is in use.
    Standard,
    /// The legacy obfuscated key is active and requires migration.
    LegacyActive,
}

impl LegacyKeyStatus {
    /// Returns `true` if the status is [`LegacyKeyStatus::LegacyActive`].
    pub const fn is_legacy(self) -> bool {
        !self.is_standard()
    }

    /// Returns `true` if the status is [`LegacyKeyStatus::Standard`].
    pub const fn is_standard(self) -> bool {
        matches!(self, Self::Standard)
    }
}

/// Internal const helper for deobfuscating the legacy static integrity key bytes.
#[cfg(feature = "legacy-wal-key")]
pub(crate) const fn deobfuscate_legacy_integrity_key() -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = LEGACY_INTEGRITY_KEY_OBFUSCATED[i] ^ LEGACY_KEY_OBFUSCATION_MASK;
        i += 1;
    }
    out
}

/// Obfuscated legacy static HMAC integrity key used strictly for backward-compatibility fallback during WAL replay of legacy databases.
#[cfg(feature = "legacy-wal-key")]
pub(crate) fn legacy_integrity_key() -> Result<[u8; 32]> {
    warn_legacy_key_use_once("legacy_integrity_key");
    Ok(deobfuscate_legacy_integrity_key())
}

#[cfg(not(feature = "legacy-wal-key"))]
pub(crate) fn legacy_integrity_key() -> Result<[u8; 32]> {
    Err(ContextraError::Storage(
        "Legacy WAL key support is disabled at compile time. Enable Cargo feature 'legacy-wal-key' to allow legacy key fallback.".into()
    ))
}

/// Migrates a legacy integrity key to a new 32-byte key material state.
///
/// # Chain Compatibility
/// When migrating a legacy WAL segment to a new integrity key format, HMAC chain validation
/// during replay must verify historical entries using the legacy key before signing new entries
/// or rewriting segments using `new_key_material`.
///
/// # Security & Zeroize Note
/// `contextra-store` does not use a `zeroize` dependency. To avoid non-volatile manual memory
/// wipes, key material in memory is handled via standard Rust slice operations without manual zeroing.
///
/// # Errors
/// Returns [`ContextraError::Storage`] if `legacy` does not match the expected legacy integrity key,
/// if `new_key_material` is not 32 bytes long, or if `new_key_material` is identical to `legacy`.
#[cfg(feature = "legacy-wal-key")]
pub fn migrate_legacy_key(legacy: &[u8], new_key_material: &[u8]) -> Result<[u8; 32]> {
    warn_legacy_key_use_once("migrate_legacy_key");

    let expected_legacy = deobfuscate_legacy_integrity_key();
    if legacy != expected_legacy {
        return Err(ContextraError::Storage(
            "Provided legacy key material does not match expected legacy integrity key".into(),
        ));
    }

    if new_key_material.len() != 32 {
        return Err(ContextraError::Storage(format!(
            "New key material must be exactly 32 bytes, got {}",
            new_key_material.len()
        )));
    }

    if new_key_material == expected_legacy {
        return Err(ContextraError::Storage(
            "New key material cannot be identical to the legacy integrity key".into(),
        ));
    }

    let mut derived = [0u8; 32];
    derived.copy_from_slice(new_key_material);
    Ok(derived)
}

#[cfg(not(feature = "legacy-wal-key"))]
pub fn migrate_legacy_key(_legacy: &[u8], _new_key_material: &[u8]) -> Result<[u8; 32]> {
    Err(ContextraError::Storage(
        "Legacy WAL key support is disabled at compile time. Recompile with Cargo feature 'legacy-wal-key' to perform legacy key migration.".into()
    ))
}

impl Wal {
    /// Prepares a batch of WAL operations with sequential sequence numbers and HMAC hash-chaining.
    pub async fn prepare_batch(&self, ops: Vec<(WalOp, u64)>) -> Result<(PreparedBatch, [u8; 32])> {
        let mut last_hmac = self.last_hmac.lock().await;
        if self.is_sealed() {
            return Err(ContextraError::Storage(format!(
                "Cannot prepare batch for sealed WAL segment {}",
                self.path.display()
            )));
        }
        let prev_hmac = *last_hmac;
        let integrity_key = self.get_integrity_key()?;

        let mut entries = Vec::with_capacity(ops.len());
        let mut current_chain = prev_hmac;

        for (op, seq_no) in ops {
            let entry = WalEntry::try_new(op, seq_no, &integrity_key, current_chain)?;
            current_chain = entry.checksum;
            entries.push(entry);
        }

        *last_hmac = current_chain;

        Ok((PreparedBatch(entries), prev_hmac))
    }

    /// Returns the legacy integrity key status for this WAL segment.
    pub fn legacy_key_status(&self) -> LegacyKeyStatus {
        if self.legacy_key_used.load(std::sync::atomic::Ordering::SeqCst) {
            LegacyKeyStatus::LegacyActive
        } else {
            LegacyKeyStatus::Standard
        }
    }

    /// Internal helper to retrieve or derive the 256-bit integrity key for HMAC chaining.
    pub(crate) fn get_integrity_key(&self) -> Result<[u8; 32]> {
        if self.legacy_key_status().is_standard() {
            tracing::trace!("Using standard WAL integrity key");
        } else {
            tracing::trace!("Using legacy WAL integrity key");
        }
        if let Some(km) = &self.key_manager {
            km.integrity_key().map_err(Into::into)
        } else if let Some(key) = self.fallback_integrity_key {
            Ok(key)
        } else {
            Err(ContextraError::Storage(
                "Integrity key missing from WAL state".into(),
            ))
        }
    }

    /// Exposes the HMAC integrity key for testing.
    pub fn integrity_key_for_test(&self) -> Result<[u8; 32]> {
        self.get_integrity_key()
    }

    #[doc(hidden)]
    #[allow(clippy::expect_used)]
    pub fn legacy_integrity_key_for_test() -> [u8; 32] {
        legacy_integrity_key().expect("Legacy WAL key support is disabled at compile time.")
    }

    /// Loads or creates the unencrypted file-local integrity key (`.wal_integrity_key`).
    ///
    /// **SECURITY NOTE**: Storing `.wal_integrity_key` adjacent to the WAL file protects
    /// against accidental bitrot and storage device corruption. It does NOT protect against
    /// malicious filesystem tampering, as an attacker with write access to the directory
    /// can also alter or replace `.wal_integrity_key`. For cryptographic tamper resistance,
    /// an active `KeyManager` (passphrase-derived keying) must be supplied.
    pub(crate) async fn load_or_create_integrity_key(wal_path: &Path) -> Result<[u8; 32]> {
        let parent = wal_path.parent().unwrap_or_else(|| Path::new(""));
        let dir_path = if parent.as_os_str().is_empty() {
            Path::new(".")
        } else {
            parent
        };
        let key_path = if parent.as_os_str().is_empty() {
            PathBuf::from(".wal_integrity_key")
        } else {
            parent.join(".wal_integrity_key")
        };

        async fn read_key_file(path: &Path) -> Result<[u8; 32]> {
            let bytes = super::fs::read(path).await.map_err(|e| {
                ContextraError::Storage(format!("Failed to read WAL integrity key: {}", e))
            })?;
            if bytes.is_empty() {
                return Err(ContextraError::Storage(
                    "WAL integrity key file is empty — possible crash during creation. Delete and restart.".into(),
                ));
            }
            if bytes.len() != 32 {
                return Err(ContextraError::Storage(format!(
                    "WAL integrity key has unexpected length: {} (expected 32)",
                    bytes.len()
                )));
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            Ok(arr)
        }

        if key_path.exists() {
            read_key_file(&key_path).await
        } else {
            use rand::RngCore;
            use tokio::io::AsyncWriteExt;

            let mut key = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);

            let tmp_path = dir_path.join(format!(
                ".wal_integrity_key.tmp.{}.{}",
                std::process::id(),
                rand::thread_rng().next_u64()
            ));

            let mut options = super::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                #[allow(unused_imports)]
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }

            let file_res = options.open(&tmp_path).await;
            let mut file = match file_res {
                Ok(f) => f,
                Err(e) => {
                    return Err(ContextraError::Storage(format!(
                        "Failed to create temporary WAL integrity key file at {}: {}",
                        tmp_path.display(),
                        e
                    )));
                }
            };

            if let Err(e) = file.write_all(&key).await {
                let _ = super::fs::remove_file(&tmp_path).await;
                return Err(ContextraError::Storage(format!(
                    "Failed to write WAL integrity key: {}",
                    e
                )));
            }
            if let Err(e) = file.sync_all().await {
                let _ = super::fs::remove_file(&tmp_path).await;
                return Err(ContextraError::Storage(format!(
                    "Failed to sync WAL integrity key file: {}",
                    e
                )));
            }
            drop(file);

            #[cfg(windows)]
            if let Err(e) = super::io::set_restrictive_file_acl(&tmp_path) {
                let _ = super::fs::remove_file(&tmp_path).await;
                return Err(e.into());
            }

            let link_res = super::fs::hard_link(&tmp_path, &key_path).await;
            let _ = super::fs::remove_file(&tmp_path).await;

            match link_res {
                Ok(()) => {
                    crate::util::fsync_parent_dir(&key_path).await?;
                    Ok(key)
                }
                Err(_) => read_key_file(&key_path).await,
            }
        }
    }

    pub(crate) fn migration_marker_path(wal_path: &Path) -> PathBuf {
        PathBuf::from(format!("{}.rekeyed", wal_path.display()))
    }

    pub(crate) async fn has_migration_marker(wal_path: &Path) -> bool {
        let marker = Self::migration_marker_path(wal_path);
        super::fs::try_exists(&marker).await.unwrap_or(false)
    }

    pub(crate) fn calculate_retirement_marker_hash(wal_path: &Path, new_key: &[u8; 32]) -> String {
        let parent = wal_path.parent().unwrap_or_else(|| Path::new("."));
        let dir_path = if parent.as_os_str().is_empty() {
            Path::new(".")
        } else {
            parent
        };
        let dir_str = dir_path
            .canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| dir_path.to_string_lossy().to_string());

        let mut hasher = blake3::Hasher::new();
        hasher.update(b"contextra-wal-retirement-v1\n");
        hasher.update(dir_str.as_bytes());
        hasher.update(b"\n");
        hasher.update(new_key);
        let hash = hasher.finalize();
        format!("rekeyed=v3:blake3:{}\n", hash.to_hex())
    }

    pub(crate) async fn write_migration_marker_atomically(wal_path: &Path) -> Result<()> {
        let new_key = Self::load_or_create_integrity_key(wal_path).await?;
        Self::write_migration_marker_with_key_atomically(wal_path, &new_key).await
    }

    pub(crate) async fn write_migration_marker_with_key_atomically(
        wal_path: &Path,
        new_key: &[u8; 32],
    ) -> Result<()> {
        let marker_path = Self::migration_marker_path(wal_path);
        let parent = marker_path.parent().unwrap_or_else(|| Path::new("."));
        let parent_dir = if parent.as_os_str().is_empty() {
            Path::new(".")
        } else {
            parent
        };

        use rand::RngCore;
        use tokio::io::AsyncWriteExt;

        let tmp_path = parent_dir.join(format!(
            "{}.tmp.{}.{}",
            marker_path
                .file_name()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default(),
            std::process::id(),
            rand::thread_rng().next_u64()
        ));

        let mut options = super::fs::OpenOptions::new();
        options.write(true).create_new(true);

        let mut file = match options.open(&tmp_path).await {
            Ok(f) => f,
            Err(e) => {
                return Err(ContextraError::Storage(format!(
                    "Failed to create temporary WAL migration marker file at {}: {}",
                    tmp_path.display(),
                    e
                )));
            }
        };

        let marker_content = Self::calculate_retirement_marker_hash(wal_path, new_key);
        if let Err(e) = file.write_all(marker_content.as_bytes()).await {
            let _ = super::fs::remove_file(&tmp_path).await;
            return Err(ContextraError::Storage(format!(
                "Failed to write WAL migration marker: {}",
                e
            )));
        }

        if let Err(e) = file.sync_all().await {
            let _ = super::fs::remove_file(&tmp_path).await;
            return Err(ContextraError::Storage(format!(
                "Failed to sync WAL migration marker file: {}",
                e
            )));
        }
        drop(file);

        if let Err(e) = super::fs::rename(&tmp_path, &marker_path).await {
            let _ = super::fs::remove_file(&tmp_path).await;
            return Err(ContextraError::Storage(format!(
                "Failed to rename WAL migration marker from {} to {}: {}",
                tmp_path.display(),
                marker_path.display(),
                e
            )));
        }

        crate::util::fsync_parent_dir(&marker_path).await?;
        Ok(())
    }

    #[cfg(feature = "wal-integrity")]
    pub(crate) async fn load_or_create_wal_uuid(wal_path: &Path) -> Result<[u8; 16]> {
        let uuid_path = {
            let mut p = wal_path.as_os_str().to_os_string();
            p.push(".uuid");
            PathBuf::from(p)
        };

        async fn read_uuid_file(path: &Path) -> Result<[u8; 16]> {
            let bytes = super::fs::read(path).await.map_err(|e| {
                ContextraError::Storage(format!("Failed to read WAL UUID sidecar: {}", e))
            })?;
            if bytes.len() != 16 {
                return Err(ContextraError::Storage(format!(
                    "WAL UUID sidecar has unexpected length: {} (expected 16)",
                    bytes.len()
                )));
            }
            let mut arr = [0u8; 16];
            arr.copy_from_slice(&bytes);
            Ok(arr)
        }

        if uuid_path.exists() {
            read_uuid_file(&uuid_path).await
        } else {
            use rand::RngCore;
            use tokio::io::AsyncWriteExt;

            let uuid = uuid::Uuid::new_v4();
            let bytes = *uuid.as_bytes();

            let uuid_filename = uuid_path
                .file_name()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default();
            let tmp_filename = format!(
                "{}.tmp.{}.{}",
                uuid_filename,
                std::process::id(),
                rand::thread_rng().next_u64()
            );

            let parent = uuid_path.parent().unwrap_or_else(|| Path::new(""));
            let tmp_path = if parent.as_os_str().is_empty() {
                PathBuf::from(tmp_filename)
            } else {
                parent.join(tmp_filename)
            };

            let mut options = super::fs::OpenOptions::new();
            options.write(true).create_new(true);

            let mut file = match options.open(&tmp_path).await {
                Ok(f) => f,
                Err(e) => {
                    if uuid_path.exists() {
                        return read_uuid_file(&uuid_path).await;
                    }
                    return Err(ContextraError::Storage(format!(
                        "Failed to create temporary WAL UUID sidecar at {}: {}",
                        tmp_path.display(),
                        e
                    )));
                }
            };

            if let Err(e) = file.write_all(&bytes).await {
                let _ = super::fs::remove_file(&tmp_path).await;
                return Err(ContextraError::Storage(format!(
                    "Failed to write WAL UUID sidecar: {}",
                    e
                )));
            }

            if let Err(e) = file.sync_all().await {
                let _ = super::fs::remove_file(&tmp_path).await;
                return Err(ContextraError::Storage(format!(
                    "Failed to sync WAL UUID sidecar file: {}",
                    e
                )));
            }
            drop(file);

            let link_res = super::fs::hard_link(&tmp_path, &uuid_path).await;
            let _ = super::fs::remove_file(&tmp_path).await;

            match link_res {
                Ok(()) => {
                    crate::util::fsync_parent_dir(&uuid_path).await?;
                    Ok(bytes)
                }
                Err(_) => read_uuid_file(&uuid_path).await,
            }
        }
    }
}
