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
    let entry2 = ManifestEntry::Remove {
        path: PathBuf::from("sst-1.sst"),
    };
    let entry3 = ManifestEntry::Add {
        path: PathBuf::from("sst-2.sst"),
        max_tx: 20,
    };

    manifest.append(&entry1).await.expect("append 1");
    manifest.append(&entry2).await.expect("append 2");
    manifest.append(&entry3).await.expect("append 3");
    drop(manifest);

    let mut file_bytes = tokio::fs::read(&manifest_path).await.expect("read file");

    // Corrupt entry 2 payload byte (flip bit in middle of file)
    let corrupt_offset = 35;
    assert!(
        corrupt_offset < file_bytes.len() - 15,
        "offset must be in the middle of file (file len={})",
        file_bytes.len()
    );
    file_bytes[corrupt_offset] ^= 0xFF;

    tokio::fs::write(&manifest_path, file_bytes)
        .await
        .expect("write corrupted file");

    let loaded_res = Manifest::load(&manifest_path).await;
    assert!(
        loaded_res.is_err(),
        "Corruption in the middle of MANIFEST must return Err, got: {:?}",
        loaded_res
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
    let entry2 = ManifestEntry::Remove {
        path: PathBuf::from("sst-1.sst"),
    };
    let entry3 = ManifestEntry::Add {
        path: PathBuf::from("sst-2.sst"),
        max_tx: 20,
    };

    manifest.append(&entry1).await.expect("append 1");
    manifest.append(&entry2).await.expect("append 2");
    manifest.append(&entry3).await.expect("append 3");
    drop(manifest);

    let mut file_bytes = tokio::fs::read(&manifest_path).await.expect("read file");
    // Truncate last entry (entry 3) partially
    file_bytes.truncate(file_bytes.len() - 10);

    tokio::fs::write(&manifest_path, file_bytes)
        .await
        .expect("write truncated file");

    let loaded = Manifest::load(&manifest_path)
        .await
        .expect("tail truncation must be recoverable with Ok");
    assert_eq!(
        loaded.len(),
        2,
        "Should recover valid prefix entries (entry1 and entry2)"
    );
    assert_eq!(loaded[0], entry1);
    assert_eq!(loaded[1], entry2);
}

#[tokio::test]
async fn proof_no_sstable_resurrection_after_corruption() {
    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

    let entry1 = ManifestEntry::Add {
        path: PathBuf::from("sst-1.sst"),
        max_tx: 10,
    };
    let entry2 = ManifestEntry::Remove {
        path: PathBuf::from("sst-1.sst"),
    };
    let entry3 = ManifestEntry::Add {
        path: PathBuf::from("sst-2.sst"),
        max_tx: 20,
    };

    manifest.append(&entry1).await.expect("append 1");
    manifest.append(&entry2).await.expect("append 2");
    manifest.append(&entry3).await.expect("append 3");
    drop(manifest);

    // Verify baseline SSTable state before corruption
    let clean_entries = Manifest::load(&manifest_path).await.expect("clean load");
    let valid_ssts_clean = Manifest::reconstruct_valid_sstables(&clean_entries);
    assert_eq!(
        valid_ssts_clean,
        vec![(PathBuf::from("sst-2.sst"), 1)],
        "sst-1.sst must be removed in clean state"
    );

    let mut file_bytes = tokio::fs::read(&manifest_path).await.expect("read file");

    // Corrupt entry 2 (Remove entry) in the middle of the manifest file
    let corrupt_offset = 36;
    file_bytes[corrupt_offset] ^= 0xAA;

    tokio::fs::write(&manifest_path, file_bytes)
        .await
        .expect("write corrupted file");

    let loaded_res = Manifest::load(&manifest_path).await;

    // Manifest::load MUST return Err on mid-file corruption.
    // If it incorrectly returned Ok(partial_entries) containing only entry 1,
    // reconstruct_valid_sstables would resurrect sst-1.sst!
    assert!(
        loaded_res.is_err(),
        "Manifest::load must return Err on mid-file corruption to prevent SSTable resurrection"
    );
}
