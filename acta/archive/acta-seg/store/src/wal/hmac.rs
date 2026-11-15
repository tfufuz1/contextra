//! # HMAC-Integritaetskette fuer WAL (Ring 1)
//!
//! Bietet kryptografische Verkettung und Manipulationssicherheit der WAL-Frames.

// ZWECK: HMAC-Kette der Journal-Rahmen.
// TODO(acta-altlast): `load_or_create_integrity_key*` legt eine Schluesseldatei im Datenverzeichnis an und widerspricht Z10/5.1; Schluessel nur als Parameter (HKDF `acta/wal/v1`).

use crate::error::{StoreError, StoreResult as Result};
use std::path::{Path, PathBuf};

use super::{PreparedBatch, Wal, WalEntry, WalOp};

// Altlast entfernt: statischer Legacy-Integritaetsschluessel, Legacy-Migration und
// `LegacyKeyStatus`. Es gibt keine Standard- oder Ersatzschluessel (Z10); das Journal
// erhaelt seinen Schluessel nur als Parameter (5.1).

impl Wal {
    /// Prepares a batch of WAL operations with sequential sequence numbers and HMAC hash-chaining.
    pub async fn prepare_batch(&self, ops: Vec<(WalOp, u64)>) -> Result<(PreparedBatch, [u8; 32])> {
        let mut last_hmac = self.last_hmac.lock().await;
        if self.is_sealed() {
            return Err(StoreError::Storage(format!(
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

    /// Rolls back the in-memory HMAC state to `prev_hmac` asynchronously.
    pub async fn rollback_hmac(&self, prev_hmac: [u8; 32]) {
        let mut last_hmac = self.last_hmac.lock().await;
        *last_hmac = prev_hmac;
    }

    /// Synchronously rolls back the in-memory HMAC state to `prev_hmac` using non-blocking `try_lock`.
    /// Suitable for use within `Drop` implementations.
    pub fn rollback_hmac_sync(&self, prev_hmac: [u8; 32]) -> bool {
        if let Ok(mut last_hmac) = self.last_hmac.try_lock() {
            *last_hmac = prev_hmac;
            true
        } else {
            false
        }
    }

    /// Internal helper to retrieve or derive the 256-bit integrity key for HMAC chaining.
    pub(crate) fn get_integrity_key(&self) -> Result<[u8; 32]> {
        if let Some(km) = &self.key_manager {
            km.integrity_key().map_err(Into::into)
        } else if let Some(key) = self.fallback_integrity_key {
            Ok(key)
        } else {
            Err(StoreError::Storage(
                "Integrity key missing from WAL state".into(),
            ))
        }
    }

    /// Exposes the HMAC integrity key for testing.
    
    pub fn integrity_key_for_test(&self) -> Result<[u8; 32]> {
        self.get_integrity_key()
    }

    /// Exposes internal `last_hmac` lock for concurrency testing.
    
    pub fn last_hmac_lock_for_test(&self) -> std::sync::Arc<tokio::sync::Mutex<[u8; 32]>> {
        std::sync::Arc::clone(&self.last_hmac)
    }

    /// Loads or creates the unencrypted file-local integrity key (`.wal_integrity_key`).
    ///
    /// **SECURITY NOTE**: Storing `.wal_integrity_key` adjacent to the WAL file protects
    /// against accidental bitrot and storage device corruption. It does NOT protect against
    /// malicious filesystem tampering, as an attacker with write access to the directory
    /// can also alter or replace `.wal_integrity_key`. For cryptographic tamper resistance,
    /// an active `KeyManager` (passphrase-derived keying) must be supplied.
    pub(crate) async fn load_or_create_integrity_key(wal_path: &Path) -> Result<[u8; 32]> {
        let default_source = crate::key_source::OsKeyMaterialSource;
        Self::load_or_create_integrity_key_with_source(wal_path, &default_source).await
    }

    pub(crate) async fn load_or_create_integrity_key_with_source(
        wal_path: &Path,
        key_source: &dyn crate::key_source::KeyMaterialSource,
    ) -> Result<[u8; 32]> {
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
                StoreError::Storage(format!("Failed to read WAL integrity key: {}", e))
            })?;
            if bytes.is_empty() {
                return Err(StoreError::Storage(
                    "WAL integrity key file is empty — possible crash during creation. Delete and restart.".into(),
                ));
            }
            if bytes.len() != 32 {
                return Err(StoreError::Storage(format!(
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
            use tokio::io::AsyncWriteExt;

            let mut key = [0u8; 32];
            key_source.fill_key_bytes(&mut key)?;

            let tmp_path = dir_path.join(format!(
                ".wal_integrity_key.tmp.{}.{}",
                std::process::id(),
                crate::lsm::recovery::next_temp_file_counter()
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
                    return Err(StoreError::Storage(format!(
                        "Failed to create temporary WAL integrity key file at {}: {}",
                        tmp_path.display(),
                        e
                    )));
                }
            };

            if let Err(e) = file.write_all(&key).await {
                let _ = super::fs::remove_file(&tmp_path).await;
                return Err(StoreError::Storage(format!(
                    "Failed to write WAL integrity key: {}",
                    e
                )));
            }
            if let Err(e) = file.sync_all().await {
                let _ = super::fs::remove_file(&tmp_path).await;
                return Err(StoreError::Storage(format!(
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
                    crate::fsync::sync_parent_dir(&key_path).await?;
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
        hasher.update(b"acta-wal-retirement-v1\n");
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

        use tokio::io::AsyncWriteExt;

        let tmp_path = parent_dir.join(format!(
            "{}.tmp.{}.{}",
            marker_path
                .file_name()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default(),
            std::process::id(),
            crate::lsm::recovery::next_temp_file_counter()
        ));

        let mut options = super::fs::OpenOptions::new();
        options.write(true).create_new(true);

        let mut file = match options.open(&tmp_path).await {
            Ok(f) => f,
            Err(e) => {
                return Err(StoreError::Storage(format!(
                    "Failed to create temporary WAL migration marker file at {}: {}",
                    tmp_path.display(),
                    e
                )));
            }
        };

        let marker_content = Self::calculate_retirement_marker_hash(wal_path, new_key);
        if let Err(e) = file.write_all(marker_content.as_bytes()).await {
            let _ = super::fs::remove_file(&tmp_path).await;
            return Err(StoreError::Storage(format!(
                "Failed to write WAL migration marker: {}",
                e
            )));
        }

        if let Err(e) = file.sync_all().await {
            let _ = super::fs::remove_file(&tmp_path).await;
            return Err(StoreError::Storage(format!(
                "Failed to sync WAL migration marker file: {}",
                e
            )));
        }
        drop(file);

        if let Err(e) = super::fs::rename(&tmp_path, &marker_path).await {
            let _ = super::fs::remove_file(&tmp_path).await;
            return Err(StoreError::Storage(format!(
                "Failed to rename WAL migration marker from {} to {}: {}",
                tmp_path.display(),
                marker_path.display(),
                e
            )));
        }

        crate::fsync::sync_parent_dir(&marker_path).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) async fn load_or_create_wal_uuid(wal_path: &Path) -> Result<[u8; 16]> {
        Self::load_or_create_wal_uuid_with_gen(wal_path, None).await
    }

    pub(crate) async fn load_or_create_wal_uuid_with_gen(
        wal_path: &Path,
        id_gen: Option<&dyn acta_base::IdGen>,
    ) -> Result<[u8; 16]> {
        let uuid_path = {
            let mut p = wal_path.as_os_str().to_os_string();
            p.push(".uuid");
            PathBuf::from(p)
        };

        async fn read_uuid_file(path: &Path) -> Result<[u8; 16]> {
            let bytes = super::fs::read(path).await.map_err(|e| {
                StoreError::Storage(format!("Failed to read WAL UUID sidecar: {}", e))
            })?;
            if bytes.len() != 16 {
                return Err(StoreError::Storage(format!(
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
            use tokio::io::AsyncWriteExt;

            let bytes = if let Some(gen) = id_gen {
                let id1 = gen.next_id();
                let id2 = gen.next_id();
                let mut b = [0u8; 16];
                if let Some(chunk1) = b.get_mut(..8) {
                    chunk1.copy_from_slice(&id1.to_le_bytes());
                }
                if let Some(chunk2) = b.get_mut(8..) {
                    chunk2.copy_from_slice(&id2.to_le_bytes());
                }
                b
            } else {
                let counter = crate::lsm::recovery::next_temp_file_counter();
                let pid = std::process::id() as u64;
                let mut b = [0u8; 16];
                if let Some(chunk1) = b.get_mut(..8) {
                    chunk1.copy_from_slice(&pid.to_le_bytes());
                }
                if let Some(chunk2) = b.get_mut(8..) {
                    chunk2.copy_from_slice(&counter.to_le_bytes());
                }
                b
            };

            let uuid_filename = uuid_path
                .file_name()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default();
            let tmp_filename = format!(
                "{}.tmp.{}.{}",
                uuid_filename,
                std::process::id(),
                crate::lsm::recovery::next_temp_file_counter()
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
                    return Err(StoreError::Storage(format!(
                        "Failed to create temporary WAL UUID sidecar at {}: {}",
                        tmp_path.display(),
                        e
                    )));
                }
            };

            if let Err(e) = file.write_all(&bytes).await {
                let _ = super::fs::remove_file(&tmp_path).await;
                return Err(StoreError::Storage(format!(
                    "Failed to write WAL UUID sidecar: {}",
                    e
                )));
            }

            if let Err(e) = file.sync_all().await {
                let _ = super::fs::remove_file(&tmp_path).await;
                return Err(StoreError::Storage(format!(
                    "Failed to sync WAL UUID sidecar file: {}",
                    e
                )));
            }
            drop(file);

            let link_res = super::fs::hard_link(&tmp_path, &uuid_path).await;
            let _ = super::fs::remove_file(&tmp_path).await;

            match link_res {
                Ok(()) => {
                    crate::fsync::sync_parent_dir(&uuid_path).await?;
                    Ok(bytes)
                }
                Err(_) => read_uuid_file(&uuid_path).await,
            }
        }
    }
}
