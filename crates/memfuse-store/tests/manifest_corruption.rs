use memfuse_store::manifest::{Manifest, ManifestEntry};
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn scenario1_mid_file_corruption_returns_err() {
    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

    let entries = vec![
        ManifestEntry::Add {
            path: PathBuf::from("sst-1.sst"),
            max_tx: 10,
        },
        ManifestEntry::Add {
            path: PathBuf::from("sst-2.sst"),
            max_tx: 20,
        },
        ManifestEntry::Remove {
            path: PathBuf::from("sst-1.sst"),
        },
        ManifestEntry::Add {
            path: PathBuf::from("sst-3.sst"),
            max_tx: 30,
        },
        ManifestEntry::Remove {
            path: PathBuf::from("sst-2.sst"),
        },
    ];

    for entry in &entries {
        manifest.append(entry).await.expect("append entry");
    }
    drop(manifest);

    let mut raw = tokio::fs::read(&manifest_path).await.expect("read file");
    // Corrupt bytes in middle range of file (around Remove(sst-1.sst) entry)
    let offset = raw.len() / 3;
    raw[offset] ^= 0xFF;

    tokio::fs::write(&manifest_path, raw)
        .await
        .expect("write corrupted file");

    let res = Manifest::load(&manifest_path).await;
    assert!(
        res.is_err(),
        "B-4 FIX REGRESSION: load() gibt Ok bei Mid-Corruption zurück — Add sst-1 würde SST resurrekten"
    );
}

#[tokio::test]
async fn scenario2_tail_truncation_is_recoverable() {
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

    // Truncate the file at the tail by 3 bytes (incomplete entry at EOF)
    let mut raw = tokio::fs::read(&manifest_path).await.expect("read file");
    raw.truncate(raw.len() - 3);
    tokio::fs::write(&manifest_path, raw)
        .await
        .expect("write truncated file");

    let loaded_entries = Manifest::load(&manifest_path)
        .await
        .expect("load manifest with tail truncation should succeed");
    assert!(
        loaded_entries.len() >= 1,
        "Tail truncated entry should return valid entries read up to truncation"
    );

    let valid = Manifest::reconstruct_valid_sstables(&loaded_entries);
    assert!(
        valid.iter().any(|(p, _)| p == &PathBuf::from("sst-1.sst")),
        "reconstruct_valid_sstables must contain sst-1.sst after tail truncation of sst-2"
    );
}

#[tokio::test]
async fn scenario3_prevent_sstable_resurrection() {
    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

    let entry1 = ManifestEntry::Add {
        path: PathBuf::from("sst-dead.sst"),
        max_tx: 10,
    };
    let entry2 = ManifestEntry::Remove {
        path: PathBuf::from("sst-dead.sst"),
    };
    let entry3 = ManifestEntry::Add {
        path: PathBuf::from("sst-live.sst"),
        max_tx: 20,
    };

    manifest.append(&entry1).await.expect("append dead add");
    manifest.append(&entry2).await.expect("append dead remove");
    manifest.append(&entry3).await.expect("append live add");
    drop(manifest);

    let mut raw = tokio::fs::read(&manifest_path).await.expect("read file");
    // Corrupt bytes in the Remove(sst-dead.sst) entry (middle range of file)
    let offset = raw.len() / 2;
    raw[offset] ^= 0xFF;

    tokio::fs::write(&manifest_path, raw)
        .await
        .expect("write corrupted file");

    if let Ok(entries) = Manifest::load(&manifest_path).await {
        let valid = Manifest::reconstruct_valid_sstables(&entries);
        assert!(
            !valid.iter().any(|(p, _)| p == &PathBuf::from("sst-dead.sst")),
            "B-4 SSTable-Resurrection: sst-dead.sst ist in valid_set nach Remove-Korruption"
        );
    }
}

#[tokio::test]
async fn scenario4_crc_header_corruption_returns_err() {
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

    let mut raw = tokio::fs::read(&manifest_path).await.expect("read file");
    // Corrupt the CRC header field (first 4 bytes of second entry, located at ~50% of file)
    let offset = raw.len() / 2;
    raw[offset] ^= 0xFF;

    tokio::fs::write(&manifest_path, raw)
        .await
        .expect("write corrupted file");

    let res = Manifest::load(&manifest_path).await;
    assert!(
        res.is_err(),
        "Manifest::load must return Err on CRC header corruption"
    );
}
