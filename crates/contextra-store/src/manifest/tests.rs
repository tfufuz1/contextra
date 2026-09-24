use super::*;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn test_manifest_entry_roundtrip() {
    let entries = vec![
        ManifestEntry::Add {
            path: PathBuf::from("sst-00000000000000000001-000000.sst"),
            max_tx: 42,
        },
        ManifestEntry::Remove {
            path: PathBuf::from("sst-00000000000000000001-000000.sst"),
        },
        ManifestEntry::RollbackComplete { target_tx: 100 },
        ManifestEntry::Replace {
            removed: vec![
                PathBuf::from("sst-00000000000000000001-000000.sst"),
                PathBuf::from("sst-00000000000000000002-000000.sst"),
            ],
            added: PathBuf::from("sst-compact-00000000000000000003-0000.sst"),
            added_max_tx: 50,
            rank: 1,
        },
    ];

    for entry in entries {
        let bytes = entry.to_bytes().expect("serialization should succeed");
        let payload_from_bytes = &bytes[4..]; // Skip total_payload_size prefix
        let decoded = ManifestEntry::from_bytes(payload_from_bytes)
            .expect("deserialization should succeed");
        assert_eq!(entry, decoded);
    }
}

#[tokio::test]
async fn test_manifest_crc_corruption_detection() {
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

    // Corrupt entry 2 (flip bytes near the end of the file)
    let mut file_bytes = tokio::fs::read(&manifest_path).await.expect("read file");
    let len = file_bytes.len();
    file_bytes[len - 2] ^= 0xFF;
    tokio::fs::write(&manifest_path, file_bytes)
        .await
        .expect("write corrupted file");

    let loaded_res = Manifest::load(&manifest_path).await;
    assert!(
        loaded_res.is_err(),
        "Internal corruption in MANIFEST must return Err instead of silent degradation"
    );
}

#[tokio::test]
async fn test_manifest_truncated_tail_recovery() {
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

    // Truncate the file in the middle of entry 2
    let mut file_bytes = tokio::fs::read(&manifest_path).await.expect("read file");
    file_bytes.truncate(file_bytes.len() - 8);
    tokio::fs::write(&manifest_path, file_bytes)
        .await
        .expect("write truncated file");

    let loaded = Manifest::load(&manifest_path).await.expect("load manifest");
    assert_eq!(
        loaded.len(),
        1,
        "Truncated tail entry should be safely ignored"
    );
    assert_eq!(loaded[0], entry1);
}

#[tokio::test]
async fn test_manifest_replace_tail_truncation_discards_incomplete_replace() {
    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

    let add1 = ManifestEntry::Add {
        path: PathBuf::from("sst-1.sst"),
        max_tx: 10,
    };
    let add2 = ManifestEntry::Add {
        path: PathBuf::from("sst-2.sst"),
        max_tx: 20,
    };
    let replace_entry = ManifestEntry::Replace {
        removed: vec![PathBuf::from("sst-1.sst"), PathBuf::from("sst-2.sst")],
        added: PathBuf::from("sst-compact-1.sst"),
        added_max_tx: 20,
        rank: 0,
    };

    manifest.append(&add1).await.expect("append add1");
    manifest.append(&add2).await.expect("append add2");
    manifest
        .append(&replace_entry)
        .await
        .expect("append replace");
    drop(manifest);

    // Truncate file in the middle of replace_entry (e.g., cut off last 10 bytes)
    let mut file_bytes = tokio::fs::read(&manifest_path).await.expect("read file");
    file_bytes.truncate(file_bytes.len() - 10);
    tokio::fs::write(&manifest_path, file_bytes)
        .await
        .expect("write truncated file");

    let loaded = Manifest::load(&manifest_path)
        .await
        .expect("load manifest with truncated replace entry");
    assert_eq!(
        loaded.len(),
        2,
        "Incomplete Replace tail entry must be discarded"
    );
    assert_eq!(loaded[0], add1);
    assert_eq!(loaded[1], add2);

    let valid = Manifest::reconstruct_valid_sstables(&loaded);
    assert_eq!(
        valid.len(),
        2,
        "Original input SSTables must remain valid when Replace entry was truncated during crash"
    );
    assert!(valid.iter().any(|(p, _)| p == Path::new("sst-1.sst")));
    assert!(valid.iter().any(|(p, _)| p == Path::new("sst-2.sst")));
}

#[test]
fn test_reconstruct_valid_sstables() {
    let entries = vec![
        ManifestEntry::Add {
            path: PathBuf::from("/data/sst-1.sst"),
            max_tx: 10,
        },
        ManifestEntry::Add {
            path: PathBuf::from("sst-2.sst"),
            max_tx: 20,
        },
        ManifestEntry::Remove {
            path: PathBuf::from("/data/sst-1.sst"),
        },
        ManifestEntry::Add {
            path: PathBuf::from("sst-3.sst"),
            max_tx: 30,
        },
        ManifestEntry::Replace {
            removed: vec![PathBuf::from("sst-2.sst"), PathBuf::from("sst-3.sst")],
            added: PathBuf::from("sst-compact-1.sst"),
            added_max_tx: 30,
            rank: 1,
        },
    ];

    let valid = Manifest::reconstruct_valid_sstables(&entries);
    assert_eq!(valid.len(), 1);
    assert_eq!(valid[0], (PathBuf::from("sst-compact-1.sst"), 1));

    let dead = Manifest::reconstruct_dead_sstables(&entries);
    assert_eq!(dead.len(), 3);
    assert!(dead.contains(Path::new("sst-1.sst")));
    assert!(dead.contains(Path::new("sst-2.sst")));
    assert!(dead.contains(Path::new("sst-3.sst")));
    assert!(!dead.contains(Path::new("sst-compact-1.sst")));
}

#[tokio::test]
async fn test_manifest_rollover_reduces_size_and_preserves_live_sstables() {
    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

    // Add 50 SSTables, then Remove 45 of them to generate a long history with few live SSTables
    for i in 1..=50 {
        let entry = ManifestEntry::Add {
            path: PathBuf::from(format!("sst-{:04}.sst", i)),
            max_tx: i as u64,
        };
        manifest.append(&entry).await.expect("append add");
    }
    for i in 1..=45 {
        let entry = ManifestEntry::Remove {
            path: PathBuf::from(format!("sst-{:04}.sst", i)),
        };
        manifest.append(&entry).await.expect("append remove");
    }

    let pre_entries = Manifest::load(&manifest_path)
        .await
        .expect("load pre-rollover entries");
    let pre_live_set = Manifest::reconstruct_valid_sstables(&pre_entries);
    assert_eq!(pre_live_set.len(), 5);

    let pre_size = tokio::fs::metadata(&manifest_path)
        .await
        .expect("pre metadata")
        .len();

    let live_entries_snapshot: Vec<ManifestEntry> = (46..=50)
        .map(|i| ManifestEntry::Add {
            path: PathBuf::from(format!("sst-{:04}.sst", i)),
            max_tx: i as u64,
        })
        .collect();

    // Perform rollover with threshold lower than pre_size
    let rolled_over = manifest
        .maybe_rollover(&live_entries_snapshot, 100)
        .await
        .expect("maybe_rollover should succeed");
    assert!(rolled_over, "Rollover should have been triggered");

    let post_size = tokio::fs::metadata(&manifest_path)
        .await
        .expect("post metadata")
        .len();
    assert!(
        post_size < pre_size,
        "Post-rollover size ({}) must be strictly smaller than pre-rollover size ({})",
        post_size,
        pre_size
    );

    let post_entries = Manifest::load(&manifest_path)
        .await
        .expect("load post-rollover entries");
    let post_live_set = Manifest::reconstruct_valid_sstables(&post_entries);
    let pre_paths: Vec<&Path> = pre_live_set.iter().map(|(p, _)| p.as_path()).collect();
    let post_paths: Vec<&Path> = post_live_set.iter().map(|(p, _)| p.as_path()).collect();
    assert_eq!(
        pre_paths, post_paths,
        "Live SSTable set after rollover must be identical to pre-rollover live set"
    );

    // Verify reopened manifest handle can still append new entries
    let new_entry = ManifestEntry::Add {
        path: PathBuf::from("sst-0051.sst"),
        max_tx: 51,
    };
    manifest
        .append(&new_entry)
        .await
        .expect("append post-rollover");

    let final_entries = Manifest::load(&manifest_path)
        .await
        .expect("load final entries");
    let final_live_set = Manifest::reconstruct_valid_sstables(&final_entries);
    assert_eq!(final_live_set.len(), 6);
    assert!(final_live_set
        .iter()
        .any(|(p, _)| p == Path::new("sst-0051.sst")));
}

#[tokio::test]
async fn test_manifest_rollover_crash_injection_preserves_old_manifest() {
    use crate::lsm::{LsmConfig, LsmStorage};

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
    manifest.append(&entry1).await.expect("append entry 1");
    manifest.append(&entry2).await.expect("append entry 2");
    drop(manifest);

    // Simulate crash before rename: leftover temp file MANIFEST.new.123.456 exists
    let leftover_tmp = dir.path().join("MANIFEST.new.123.456");
    tokio::fs::write(&leftover_tmp, b"partial manifest content from crash")
        .await
        .expect("write leftover temp file");

    // Verify loading original MANIFEST is unaffected by leftover temp file
    let loaded = Manifest::load(&manifest_path)
        .await
        .expect("load manifest during crash recovery");
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0], entry1);
    assert_eq!(loaded[1], entry2);

    // Verify startup recovery removes leftover MANIFEST.new.* file
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let _storage = LsmStorage::new(config)
        .await
        .expect("LsmStorage startup with leftover temp manifest file");

    assert!(
        !leftover_tmp.exists(),
        "Leftover MANIFEST.new.* temp file must be cleaned up during startup recovery"
    );
}

#[tokio::test]
async fn test_manifest_rollover_restart_recovery() {
    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let live_sstables = vec![
        ManifestEntry::Add {
            path: PathBuf::from("sst-10.sst"),
            max_tx: 10,
        },
        ManifestEntry::Add {
            path: PathBuf::from("sst-20.sst"),
            max_tx: 20,
        },
    ];

    {
        let manifest = Manifest::open(&manifest_path).await.expect("open manifest");
        for entry in &live_sstables {
            manifest.append(entry).await.expect("append entry");
        }

        let rolled = manifest
            .maybe_rollover(&live_sstables, 1)
            .await
            .expect("rollover");
        assert!(rolled);
    }

    // Process restart simulation: reload manifest from disk
    let reloaded_entries = Manifest::load(&manifest_path)
        .await
        .expect("load manifest after restart");
    let valid_sstables = Manifest::reconstruct_valid_sstables(&reloaded_entries);

    assert_eq!(valid_sstables.len(), 2);
    assert!(valid_sstables
        .iter()
        .any(|(p, _)| p == Path::new("sst-10.sst")));
    assert!(valid_sstables
        .iter()
        .any(|(p, _)| p == Path::new("sst-20.sst")));
}

#[tokio::test]
async fn test_manifest_explicit_rollover() {
    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

    let entry1 = ManifestEntry::Add {
        path: PathBuf::from("sst-1.sst"),
        max_tx: 10,
    };
    manifest.append(&entry1).await.expect("append 1");

    let live = vec![entry1.clone()];
    manifest.rollover(&live).await.expect("explicit rollover");

    let loaded = Manifest::load(&manifest_path).await.expect("load manifest");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0], entry1);
}
