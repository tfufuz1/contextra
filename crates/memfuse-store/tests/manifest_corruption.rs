use memfuse_core::{StorageEngine, TxId};
use memfuse_store::lsm::{LsmConfig, LsmStorage};
use memfuse_store::manifest::{Manifest, ManifestEntry};
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn proof_manifest_mid_corruption_returns_err() {
    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

    let entry1 = ManifestEntry::Add {
        path: PathBuf::from("sst-1.sst"),
        max_tx: 10,
    };
    let entry2 = ManifestEntry::Add {
        path: PathBuf::from("sst-2.sst"),
        max_tx: 20,
    };
    let entry3 = ManifestEntry::Add {
        path: PathBuf::from("sst-3.sst"),
        max_tx: 30,
    };

    manifest.append(&entry1).await.expect("append 1");
    manifest.append(&entry2).await.expect("append 2");
    manifest.append(&entry3).await.expect("append 3");
    drop(manifest);

    // Corrupt entry 2 (in the middle of the file before entry 3)
    let mut file_bytes = tokio::fs::read(&manifest_path).await.expect("read file");
    let offset1 = entry1.to_bytes().unwrap().len();
    file_bytes[offset1 + 10] ^= 0xFF;

    tokio::fs::write(&manifest_path, file_bytes)
        .await
        .expect("write corrupted file");

    let res = Manifest::load(&manifest_path).await;
    assert!(
        res.is_err(),
        "Manifest::load must return Err on mid-file CRC/data corruption"
    );
}

#[tokio::test]
async fn proof_manifest_tail_truncation_is_recoverable() {
    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

    let entry1 = ManifestEntry::Add {
        path: PathBuf::from("sst-1.sst"),
        max_tx: 10,
    };
    let entry2 = ManifestEntry::Add {
        path: PathBuf::from("sst-2.sst"),
        max_tx: 20,
    };

    manifest.append(&entry1).await.expect("append 1");
    manifest.append(&entry2).await.expect("append 2");
    drop(manifest);

    // Truncate the file at the tail (cut off middle of entry 2)
    let mut file_bytes = tokio::fs::read(&manifest_path).await.expect("read file");
    file_bytes.truncate(file_bytes.len() - 8);
    tokio::fs::write(&manifest_path, file_bytes)
        .await
        .expect("write truncated file");

    let loaded = Manifest::load(&manifest_path)
        .await
        .expect("load manifest with tail truncation should succeed");
    assert_eq!(
        loaded.len(),
        1,
        "Tail truncated entry should be ignored, returning entries up to truncation"
    );
    assert_eq!(loaded[0], entry1);
}

#[tokio::test]
async fn proof_no_sstable_resurrection_after_corruption() {
    let dir = tempdir().expect("tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };

    {
        let storage = LsmStorage::new(config.clone())
            .await
            .expect("create storage");

        let tx1 = TxId::new(1);
        storage.put(tx1, b"key1", b"val1").await.unwrap();
        storage.commit(tx1).await.unwrap();
        storage.force_flush().await.unwrap();

        let tx2 = TxId::new(2);
        storage.put(tx2, b"key2", b"val2").await.unwrap();
        storage.commit(tx2).await.unwrap();
        storage.force_flush().await.unwrap();
    }

    let manifest_path = dir.path().join("MANIFEST");
    let mut file_bytes = tokio::fs::read(&manifest_path).await.expect("read file");
    let file_len = file_bytes.len();
    assert!(file_len > 10);
    // Corrupt entry in MANIFEST
    file_bytes[file_len - 5] ^= 0xFF;
    tokio::fs::write(&manifest_path, file_bytes)
        .await
        .expect("write corrupted file");

    // Reopen LSM storage. It MUST return Err, failing to start rather than resurrecting deleted/unmanifested SSTables.
    let res = LsmStorage::new(config).await;
    assert!(
        res.is_err(),
        "LsmStorage::new must return Err when MANIFEST is corrupted"
    );
}
