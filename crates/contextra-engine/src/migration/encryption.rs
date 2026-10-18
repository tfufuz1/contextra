// ZWECK: Offline encryption migration for Contextra LSM databases (v5-SSTables & MFW4-WAL).

use contextra_crypto::{crypto::KeyManager, KdfHeader, KdfParams};
use contextra_durable_fs::{scrub_and_remove, sync_dir, ScrubMethod};
use contextra_ports::StorageEngine;
use contextra_store::lsm::config::EncryptionConfig;
use contextra_store::sstable::SstableBuilder;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{ContextraError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationMethod {
    OfflineAtomicManifestSwitch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMigrationReport {
    pub files_migrated: usize,
    pub method: MigrationMethod,
    pub duration: Duration,
    pub scrub_unlink_only_count: usize,
    pub scrub_overwritten_count: usize,
    pub warning_note: &'static str,
}

pub async fn migrate_database_encryption(
    path: impl AsRef<Path>,
    target_encryption: EncryptionConfig,
) -> Result<EncryptionMigrationReport> {
    let start_time = Instant::now();
    let db_path = path.as_ref().to_path_buf();

    if !target_encryption.is_encrypted() {
        return Err(ContextraError::InvalidInput(
            "Target encryption config cannot be EncryptionConfig::None for encryption migration".into(),
        ));
    }

    let target_passphrase = target_encryption.passphrase().ok_or_else(|| {
        ContextraError::InvalidInput("No valid passphrase could be derived from target_encryption".into())
    })?;

    if !db_path.exists() {
        return Err(ContextraError::Storage(format!(
            "Database path does not exist: {}",
            db_path.display()
        )));
    }

    let intent_path = db_path.join("migration.intent");
    let hwm_path = db_path.join(".encrypted_hwm");

    // Resumability check: if migration was interrupted after manifest switch
    if hwm_path.exists() && intent_path.exists() {
        let (migrated, unlinks, overwrites) = cleanup_and_scrub_old_files(&db_path).await?;
        let _ = tokio::fs::remove_file(&intent_path).await;

        return Ok(EncryptionMigrationReport {
            files_migrated: migrated,
            method: MigrationMethod::OfflineAtomicManifestSwitch,
            duration: start_time.elapsed(),
            scrub_unlink_only_count: unlinks,
            scrub_overwritten_count: overwrites,
            warning_note: "alte Dateien möglicherweise nicht überschrieben (CoW/Hardlink/SSD)",
        });
    }

    // Record migration intent for resumability
    tokio::fs::write(&intent_path, b"ENCRYPTION_MIGRATION_IN_PROGRESS_V1\n")
        .await
        .map_err(|e| ContextraError::Storage(format!("Failed to write migration intent: {e}")))?;

    // Step 1: Open source database with current/unencrypted config to read existing data
    let source_lsm_config = LsmConfig {
        path: db_path.clone(),
        encryption: EncryptionConfig::None,
        ..Default::default()
    };

    let source_storage = LsmStorage::open(source_lsm_config).await?;
    let all_entries = source_storage.scan_prefix(b"").await?;
    let max_tx = source_storage.last_tx_id().await?.inner();
    source_storage.close().await?;

    // Step 2: Ensure SALT file exists and derive target KeyManager
    let salt_path = db_path.join("SALT");
    let salt_bytes = if salt_path.exists() {
        tokio::fs::read(&salt_path)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to read SALT: {e}")))?
    } else {
        let mut buf = [0u8; 32];
        let rng = contextra_ports::SystemRng::new();
        contextra_ports::Rng::fill_bytes(&rng, &mut buf);
        tokio::fs::write(&salt_path, &buf)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to write SALT: {e}")))?;
        buf.to_vec()
    };

    let salt_array: [u8; 32] = salt_bytes.as_slice().try_into().map_err(|_| {
        ContextraError::Storage("SALT file must be exactly 32 bytes".into())
    })?;

    let kdf_header = KdfHeader::new(KdfParams::default(), salt_array.to_vec())?;
    let target_km = Arc::new(KeyManager::try_new_strict(&target_passphrase, &kdf_header)?);

    // Step 3: Write new encrypted SSTable file(s)
    let mut files_migrated_count = 0usize;
    let new_sst_path = db_path.join("sst-enc-00000000000000000001-000001.sst");

    if !all_entries.is_empty() {
        let mut builder = SstableBuilder::create_with_key_manager(
            &new_sst_path,
            Some(target_km.clone()),
        )
        .await?;

        let mut seq_counter = 1u64;
        for (k, v) in &all_entries {
            builder.add(k, v, seq_counter, max_tx).await?;
            seq_counter += 1;
        }
        builder.finish().await?;
        files_migrated_count += 1;
    }

    // Step 4: Atomic Manifest Switch
    let new_manifest_path = db_path.join("MANIFEST.new.migration");
    let manifest_path = db_path.join("MANIFEST");

    let mut add_entries = Vec::new();
    if new_sst_path.exists() {
        add_entries.push(contextra_store::ManifestEntry::Add {
            path: new_sst_path.clone(),
            max_tx,
        });
    }
    add_entries.push(contextra_store::ManifestEntry::WalCheckpoint {
        hmac: [0u8; 32],
    });

    let new_manifest = contextra_store::manifest::Manifest::open(&new_manifest_path).await?;
    new_manifest.append_batch(&add_entries).await?;
    drop(new_manifest);

    tokio::fs::rename(&new_manifest_path, &manifest_path)
        .await
        .map_err(|e| ContextraError::Storage(format!("Atomic manifest switch failed: {e}")))?;

    // Write High Watermark file
    tokio::fs::write(&hwm_path, b"ENCRYPTED_HWM_V1\n")
        .await
        .map_err(|e| ContextraError::Storage(format!("Failed to write .encrypted_hwm: {e}")))?;

    let _ = sync_dir(&db_path);

    // Step 5: Post-commit Scrubbing of old unencrypted files
    let (_, unlinks, overwrites) = cleanup_and_scrub_old_files(&db_path).await?;

    // Step 6: Clear migration intent
    let _ = tokio::fs::remove_file(&intent_path).await;
    let _ = sync_dir(&db_path);

    Ok(EncryptionMigrationReport {
        files_migrated: files_migrated_count,
        method: MigrationMethod::OfflineAtomicManifestSwitch,
        duration: start_time.elapsed(),
        scrub_unlink_only_count: unlinks,
        scrub_overwritten_count: overwrites,
        warning_note: "alte Dateien möglicherweise nicht überschrieben (CoW/Hardlink/SSD)",
    })
}

async fn cleanup_and_scrub_old_files(db_path: &Path) -> Result<(usize, usize, usize)> {
    let mut entries = tokio::fs::read_dir(db_path)
        .await
        .map_err(|e| ContextraError::Storage(format!("Read dir failed for scrub: {e}")))?;

    let mut old_files = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| ContextraError::Storage(format!("Read entry failed for scrub: {e}")))?
    {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if (name_str.starts_with("sst-") && !name_str.starts_with("sst-enc-"))
            || (name_str.starts_with("wal-") && !name_str.starts_with("wal-enc-"))
            || name_str == "wal.log"
        {
            old_files.push(entry.path());
        }
    }

    let mut unlinks = 0usize;
    let mut overwrites = 0usize;
    let total_old = old_files.len();

    for file_path in old_files {
        if file_path.exists() {
            match scrub_and_remove(&file_path) {
                Ok(receipt) => match receipt.method {
                    ScrubMethod::UnlinkOnly => unlinks += 1,
                    _ => overwrites += 1,
                },
                Err(e) => {
                    tracing::warn!(path = ?file_path, error = %e, "Failed to scrub old unencrypted file");
                }
            }
        }
    }

    Ok((total_old, unlinks, overwrites))
}
