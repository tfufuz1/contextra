//! # Legacy WAL Key Migration (Ring 1)
//!
//! ZWECK: Pruefung und Migration von Alt-WAL-Dateien vor der vollstaendigen
//! Abloesung von Legacy-HMAC-Schluesseln.

use crate::error::StoreResult as Result;
use crate::lsm::config::LsmConfig;
use crate::lsm::engine::LsmStorage;
use crate::wal::Wal;
use std::sync::Arc;

impl LsmStorage {
    /// Checks if a database directory contains any legacy WAL files requiring explicit migration.
    pub async fn has_pending_legacy_wal_migration_path(path: &std::path::Path) -> Result<bool> {
        let mut entries = match tokio::fs::read_dir(path).await {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(e) => {
                return Err(crate::error::StoreError::Storage(format!(
                    "Failed to read directory for legacy WAL check: {e}"
                )))
            }
        };

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            crate::error::StoreError::Storage(format!(
                "Failed to read directory entry for legacy WAL check: {e}"
            ))
        })? {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            let is_wal_file = (name_str.starts_with("wal-") && name_str.ends_with(".log"))
                || name_str == "wal.log"
                || name_str.ends_with(".wal");

            if is_wal_file {
                let file_path = entry.path();
                if !Wal::has_migration_marker(&file_path).await {
                    let test_wal = Wal::open_read_only_with_config(
                        &file_path,
                        crate::wal::WalConfig {
                            allow_legacy_integrity_key_fallback: true,
                            min_wal_version: crate::wal::WalVersion::V1,
                            ..Default::default()
                        },
                    )
                    .await?;

                    let _ = test_wal.replay().await?;
                    if test_wal.legacy_key_used_for_test() {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Checks if the current storage instance's data directory contains any pending legacy WAL files.
    pub async fn has_pending_legacy_wal_migration(&self) -> Result<bool> {
        Self::has_pending_legacy_wal_migration_path(&self.config.path).await
    }

    /// Explicitly migrates all legacy WAL files in the specified directory using the provided clock port.
    pub async fn migrate_legacy_wal_keys(
        config: &LsmConfig,
        clock: Arc<dyn contextra_env::Clock>,
    ) -> Result<usize> {
        let mut entries = match tokio::fs::read_dir(&config.path).await {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(e) => {
                return Err(crate::error::StoreError::Storage(format!(
                    "Failed to read directory for legacy WAL migration: {e}"
                )))
            }
        };

        let salt_path = config.path.join("SALT");
        if !salt_path.exists() {
            let buf = crate::lsm::recovery::generate_crypto_salt()?;
            crate::lsm::recovery::write_salt_atomically(&salt_path, &buf).await?;
        }

        let salt = tokio::fs::read(&salt_path).await.map_err(|e| {
            crate::error::StoreError::Storage(format!(
                "SALT file read failed during migration: {e}"
            ))
        })?;

        let key_manager = if let Some(passphrase) = &config.encryption_passphrase {
            Some(Arc::new(crate::wal::KeyManager::try_new(
                passphrase, &salt,
            )?))
        } else {
            None
        };

        let mut migrated_count = 0usize;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            crate::error::StoreError::Storage(format!(
                "Failed to read directory entry for legacy WAL migration: {e}"
            ))
        })? {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            let is_wal_file = (name_str.starts_with("wal-") && name_str.ends_with(".log"))
                || name_str == "wal.log"
                || name_str.ends_with(".wal");

            if is_wal_file {
                let file_path = entry.path();
                if !Wal::has_migration_marker(&file_path).await {
                    let wal =
                        Wal::open_for_legacy_migration(&file_path, key_manager.clone()).await?;

                    if wal.was_legacy_rekeyed() {
                        let timestamp_nanos = clock.now_unix_nanos();
                        tracing::info!(
                            wal_path = %file_path.display(),
                            timestamp_nanos = timestamp_nanos,
                            "Legacy WAL key migration executed for segment"
                        );
                        migrated_count += 1;
                    }
                }
            }
        }

        Ok(migrated_count)
    }
}
