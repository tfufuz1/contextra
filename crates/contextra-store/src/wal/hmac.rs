use contextra_core::{ContextraError, Result};
use std::path::{Path, PathBuf};

use super::{PreparedBatch, Wal, WalEntry, WalOp};

const LEGACY_KEY_OBFUSCATION_MASK: u8 = 0x5A;
const LEGACY_INTEGRITY_KEY_OBFUSCATED: [u8; 32] = *b"7?7</)?w34.?=(3.#w1?#w,kZZZZZZZZ";

/// Obfuscated legacy static HMAC integrity key used strictly for backward-compatibility fallback during WAL replay of legacy databases.
pub(crate) const fn legacy_integrity_key() -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = LEGACY_INTEGRITY_KEY_OBFUSCATED[i] ^ LEGACY_KEY_OBFUSCATION_MASK;
        i += 1;
    }
    out
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

    /// Restores `last_hmac` to a previous state after an append failure.
    pub async fn restore_last_hmac(&self, hmac: [u8; 32]) -> Result<()> {
        let mut hmac_guard = self.last_hmac.lock().await;
        *hmac_guard = hmac;
        Ok(())
    }

    /// Internal helper to retrieve or derive the 256-bit integrity key for HMAC chaining.
    pub(crate) fn get_integrity_key(&self) -> Result<[u8; 32]> {
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

            if let Err(e) = super::fs::rename(&tmp_path, &uuid_path).await {
                let _ = super::fs::remove_file(&tmp_path).await;
                if uuid_path.exists() {
                    return read_uuid_file(&uuid_path).await;
                }
                return Err(ContextraError::Storage(format!(
                    "Failed to rename WAL UUID sidecar from {} to {}: {}",
                    tmp_path.display(),
                    uuid_path.display(),
                    e
                )));
            }

            crate::util::fsync_parent_dir(&uuid_path).await?;

            Ok(bytes)
        }
    }
}
