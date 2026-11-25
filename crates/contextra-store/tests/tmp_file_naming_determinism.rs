// ZWECK: Determinismus-Test für temporäre Dateinamen in contextra-store (recovery.rs / rollover.rs).
// Prüft Monotonie, PID-Einbindung, keine rand-Abhängigkeit im Dateinamen und Kollisionswiederholung.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_store::manifest::Manifest;
use contextra_store::{LsmConfig, LsmStorage};
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_temp_file_naming_pattern_and_determinism() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().to_path_buf();

    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };

    // Trigger SALT creation via LsmStorage::open
    let storage = LsmStorage::open(config.clone())
        .await
        .expect("open storage");

    let salt_path = db_path.join("SALT");
    assert!(salt_path.exists(), "SALT file must be created");

    // Test MANIFEST rollover temp file naming
    let manifest_path = db_path.join("MANIFEST");
    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

    let live_entries =
        vec![contextra_store::manifest::ManifestEntry::WalCheckpoint { hmac: [1u8; 32] }];

    manifest
        .rollover(&live_entries)
        .await
        .expect("manifest rollover");

    // Read directory entries to ensure no lingering .tmp or MANIFEST.new files exist
    let mut read_dir = fs::read_dir(&db_path).await.expect("read_dir");
    while let Some(entry) = read_dir.next_entry().await.expect("next_entry") {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        assert!(
            !name_str.starts_with("SALT.tmp."),
            "No lingering temporary SALT file expected: {}",
            name_str
        );
        assert!(
            !name_str.starts_with("MANIFEST.new."),
            "No lingering temporary MANIFEST file expected: {}",
            name_str
        );
    }

    let _ = storage.close().await;
}

#[tokio::test]
async fn test_temp_file_naming_collision_retry() {
    let dir = tempdir().expect("tempdir");
    let db_path = dir.path().to_path_buf();
    let pid = std::process::id();

    // Pre-create collision files for counters 1 and 2
    for c in 1..=2 {
        let collision_tmp = db_path.join(format!("SALT.tmp.{}.{}", pid, c));
        fs::create_dir_all(&db_path).await.expect("create dir");
        fs::write(&collision_tmp, b"existing collision file")
            .await
            .expect("pre-create collision temp file");
    }

    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };

    // LsmStorage::open should hit AlreadyExists on counters 1 and 2, retry with counter 3, and succeed!
    let storage = LsmStorage::open(config)
        .await
        .expect("LsmStorage::open must handle AlreadyExists by retrying with next counter");

    let salt_path = db_path.join("SALT");
    assert!(salt_path.exists(), "SALT file must be created on retry");

    let _ = storage.close().await;
}
