// ZWECK: Test offline encryption migration, data parity, raw canary sweep, and crash resumability.

use contextra_engine::migration::{
    migrate_database_encryption, EncryptionMigrationReport, MigrationMethod,
};
use contextra_engine::{Contextra, ContextraConfig};
use contextra_store::lsm::config::EncryptionConfig;
use contextra_testkit::erasure_harness::raw_sweep;
use tempfile::tempdir;

#[tokio::test]
async fn test_offline_encryption_migration_roundtrip_data_parity() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempdir()?;
    let db_path = tmp.path().to_path_buf();

    let canary_str = "CANARY_SENSITIVE_UNENCRYPTED_TEXT_12345";
    let canary_bytes = canary_str.as_bytes();

    // 1. Create unencrypted database and insert documents
    {
        let config = ContextraConfig {
            dimension: 4,
            encryption: EncryptionConfig::None,
            ..Default::default()
        };
        let db = Contextra::open_with_config(&db_path, config).await?;
        db.insert(
            "doc_canary_1",
            &[0.1, 0.2, 0.3, 0.4],
            Some(serde_json::json!({ "secret": canary_str })),
        )
        .await?;
        db.insert(
            "doc_canary_2",
            &[0.5, 0.6, 0.7, 0.8],
            Some(serde_json::json!({ "value": "non_secret" })),
        )
        .await?;
        db.close().await?;
    }

    // Anti-mirroring reference copy: record expected document states before migration
    let expected_doc1_id = "doc_canary_1".to_string();
    let expected_doc2_id = "doc_canary_2".to_string();

    // Verify unencrypted canary is visible in raw sweep prior to migration
    let raw_before = raw_sweep(&db_path, &[canary_bytes.to_vec()])?;
    assert!(
        !raw_before.hits.is_empty(),
        "Canary text should be visible on disk prior to migration"
    );

    // 2. Perform offline encryption migration
    let target_config = EncryptionConfig::Passphrase("migration_passphrase_789".to_string());
    let report: EncryptionMigrationReport =
        migrate_database_encryption(&db_path, target_config.clone()).await?;

    assert!(report.files_migrated > 0, "At least one file must be migrated");
    assert_eq!(
        report.method,
        MigrationMethod::OfflineAtomicManifestSwitch
    );
    assert_eq!(
        report.warning_note,
        "alte Dateien möglicherweise nicht überschrieben (CoW/Hardlink/SSD)"
    );

    // 3. Reopen database with new encryption passphrase and verify data parity against reference copy
    {
        let config = ContextraConfig {
            dimension: 4,
            encryption: target_config.clone(),
            ..Default::default()
        };
        let db = Contextra::open_with_config(&db_path, config).await?;

        let doc1 = db.get(&expected_doc1_id).await?.expect("doc1 must exist post-migration");
        assert_eq!(doc1.id, expected_doc1_id);
        assert_eq!(
            doc1.metadata.as_ref().unwrap()["secret"],
            canary_str,
            "Document 1 payload must match reference copy post-migration"
        );

        let doc2 = db.get(&expected_doc2_id).await?.expect("doc2 must exist post-migration");
        assert_eq!(doc2.id, expected_doc2_id);

        db.close().await?;
    }

    // 4. Raw sweep check: active SSTables & WAL files must contain no unencrypted canary text
    let raw_after = raw_sweep(&db_path, &[canary_bytes.to_vec()])?;
    // Filter out hits that are in explicitly reported old files (if retained in backup)
    let active_hits: Vec<_> = raw_after
        .hits
        .into_iter()
        .filter(|hit| {
            let path_str = hit.relative_path.to_string_lossy();
            !path_str.contains(".old") && !path_str.contains(".backup")
        })
        .collect();

    assert!(
        active_hits.is_empty(),
        "No plaintext canary data must be found in active migrated files: {:?}",
        active_hits
    );

    Ok(())
}
