//! # Manifest-Tests (Ring 1)
//!
//! Validiert Rollover, Recovery und atomare Manifest-Commits.

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
            rank: 0,
        },
    ];

    for entry in entries {
        let bytes = entry.to_bytes().expect("serialization should succeed");
        let payload_from_bytes = &bytes[4..]; // Skip total_payload_size prefix
        let decoded =
            ManifestEntry::from_bytes(payload_from_bytes).expect("deserialization should succeed");
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
            rank: 0,
        },
    ];

    let valid = Manifest::reconstruct_valid_sstables(&entries);
    assert_eq!(valid.len(), 1);
    assert_eq!(valid[0], (PathBuf::from("sst-compact-1.sst"), 3));

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

    // Write SALT so directory is valid for LsmStorage recovery
    tokio::fs::write(dir.path().join("SALT"), &[0u8; 32])
        .await
        .expect("write salt");

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

#[tokio::test]
async fn test_sl1_two_separate_entries_tear_in_torn_write_vs_sst_swap() {
    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");
    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");
    let old_sst = PathBuf::from("sst-old.sst");
    let new_sst = PathBuf::from("sst-new.sst");
    manifest
        .append(&ManifestEntry::Add {
            path: old_sst.clone(),
            max_tx: 10,
        })
        .await
        .expect("append initial");
    manifest
        .append(&ManifestEntry::Add {
            path: new_sst.clone(),
            max_tx: 20,
        })
        .await
        .expect("append add new");
    let loaded = Manifest::load(&manifest_path).await.expect("load");
    let valid = Manifest::reconstruct_valid_sstables(&loaded);
    assert_eq!(
        valid.len(),
        2,
        "Two separate entries tear under torn-write: both old and new SSTables remain active!"
    );

    let dir_b = tempdir().expect("tempdir");
    let manifest_b_path = dir_b.path().join("MANIFEST");
    let manifest_b = Manifest::open(&manifest_b_path)
        .await
        .expect("open manifest_b");
    manifest_b
        .append(&ManifestEntry::Add {
            path: old_sst.clone(),
            max_tx: 10,
        })
        .await
        .expect("append initial b");
    let sst_swap = ManifestEntry::SstSwap {
        add: vec![new_sst.clone()],
        remove: vec![old_sst.clone()],
    };
    manifest_b.append(&sst_swap).await.expect("append sst_swap");
    drop(manifest_b);

    // If SstSwap is torn (truncated at tail), the whole swap is discarded and old SSTable remains valid
    let mut file_bytes = tokio::fs::read(&manifest_b_path).await.expect("read file");
    file_bytes.truncate(file_bytes.len() - 5);
    tokio::fs::write(&manifest_b_path, file_bytes)
        .await
        .expect("truncate");

    let loaded_b = Manifest::load(&manifest_b_path).await.expect("load b");
    let valid_b = Manifest::reconstruct_valid_sstables(&loaded_b);
    assert_eq!(
        valid_b.len(),
        1,
        "Atomic SstSwap tail-torn write must discard the incomplete swap as a whole"
    );
    assert_eq!(
        valid_b[0].0, old_sst,
        "Old SSTable must remain active if SstSwap was torn"
    );
}

#[tokio::test]
async fn test_wal_floor_manifest_entry_roundtrip_and_current_floor() {
    use contextra_ports::keys::FileId;

    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

    let floor1 = ManifestEntry::WalFloor {
        seq: 100,
        tail_mac: [0x42; 32],
        removed_file_ids: vec![FileId::new([1u8; 16]), FileId::new([2u8; 16])],
    };
    let floor2 = ManifestEntry::WalFloor {
        seq: 200,
        tail_mac: [0x77; 32],
        removed_file_ids: vec![FileId::new([3u8; 16])],
    };

    manifest.append(&floor1).await.expect("append floor1");
    manifest.append(&floor2).await.expect("append floor2");

    let loaded = Manifest::load(&manifest_path).await.expect("load manifest");
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0], floor1);
    assert_eq!(loaded[1], floor2);

    let cur = Manifest::current_wal_floor(&loaded);
    assert!(cur.is_some());
    let cur_floor = cur.expect("current floor");
    assert_eq!(cur_floor.seq, 200);
    assert_eq!(cur_floor.tail_mac, [0x77; 32]);
    assert_eq!(cur_floor.removed_file_ids, vec![FileId::new([3u8; 16])]);

    // Direct API: append_wal_floor
    manifest
        .append_wal_floor(300, [0x88; 32], vec![FileId::new([4u8; 16])])
        .await
        .expect("append_wal_floor");

    let reloaded = Manifest::load(&manifest_path).await.expect("reloaded");
    let cur2 = Manifest::current_wal_floor(&reloaded).expect("floor 3");
    assert_eq!(cur2.seq, 300);
    assert_eq!(cur2.tail_mac, [0x88; 32]);
    assert_eq!(cur2.removed_file_ids, vec![FileId::new([4u8; 16])]);
}

#[tokio::test]
async fn test_text_commit_manifest_entry_roundtrip_and_query() {
    use contextra_ports::keys::FileId;

    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

    let tc1 = ManifestEntry::TextCommit {
        add: Some(FileId::new([10u8; 16])),
        remove: vec![FileId::new([1u8; 16]), FileId::new([2u8; 16])],
        floor_seq: 150,
    };
    manifest.append(&tc1).await.expect("append tc1");

    manifest
        .append_text_commit(
            Some(FileId::new([20u8; 16])),
            vec![FileId::new([10u8; 16])],
            250,
        )
        .await
        .expect("append_text_commit");

    let loaded = Manifest::load(&manifest_path)
        .await
        .expect("load text commit");
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0], tc1);

    let cur_tc = Manifest::current_text_commit(&loaded).expect("current tc");
    assert_eq!(cur_tc.add, Some(FileId::new([20u8; 16])));
    assert_eq!(cur_tc.remove, vec![FileId::new([10u8; 16])]);
    assert_eq!(cur_tc.floor_seq, 250);

    let all_tc = Manifest::extract_text_commits(&loaded);
    assert_eq!(all_tc.len(), 2);
    assert_eq!(all_tc[0].floor_seq, 150);
    assert_eq!(all_tc[1].floor_seq, 250);
}

#[test]
fn test_candidates_intersecting_interval_query() {
    use super::core::ManifestIntervalIndex;

    let mut idx = ManifestIntervalIndex::new();
    idx.insert("sst-01.sst", [10u8; 65], [30u8; 65]);
    idx.insert("sst-02.sst", [40u8; 65], [60u8; 65]);
    idx.insert("sst-03.sst", [70u8; 65], [90u8; 65]);

    // Query intersecting [25, 45]: should return sst-01 and sst-02
    let res = idx.candidates_intersecting([25u8; 65], [45u8; 65]);
    assert_eq!(res.len(), 2);
    assert!(res.contains(&PathBuf::from("sst-01.sst")));
    assert!(res.contains(&PathBuf::from("sst-02.sst")));
    assert!(!res.contains(&PathBuf::from("sst-03.sst")));

    // Query non-intersecting
    let empty_res = idx.candidates_intersecting([61u8; 65], [69u8; 65]);
    assert!(empty_res.is_empty());
}

#[test]
fn test_t_fmt_1_golden_and_fuzz_safety() {
    use contextra_ports::keys::FileId;

    let entries = vec![
        ManifestEntry::WalFloor {
            seq: 42,
            tail_mac: [0xaa; 32],
            removed_file_ids: vec![FileId::new([1u8; 16]), FileId::new([2u8; 16])],
        },
        ManifestEntry::TextCommit {
            add: Some(FileId::new([99u8; 16])),
            remove: vec![FileId::new([5u8; 16]), FileId::new([6u8; 16])],
            floor_seq: 400,
        },
        ManifestEntry::SstSwap {
            add: vec![PathBuf::from("a1.sst"), PathBuf::from("a2.sst")],
            remove: vec![PathBuf::from("r1.sst")],
        },
    ];

    for entry in entries {
        let bytes = entry.to_bytes().expect("to_bytes");
        // Frame structure: [len: 4] [crc: 4] [payload...]
        assert!(bytes.len() >= 8);
        let decoded = ManifestEntry::from_bytes(&bytes[4..]).expect("from_bytes");
        assert_eq!(entry, decoded);

        // Bit-flip test: ensure corrupt crc is rejected without panic
        let mut corrupted = bytes.clone();
        corrupted[4] ^= 0xff;
        assert!(
            ManifestEntry::from_bytes(&corrupted[4..]).is_err(),
            "Bit-flip in CRC must be rejected fail-closed"
        );

        // Payload corruption
        let mut corrupted_payload = bytes.clone();
        if corrupted_payload.len() > 9 {
            corrupted_payload[9] ^= 0x55;
            assert!(
                ManifestEntry::from_bytes(&corrupted_payload[4..]).is_err(),
                "Corrupted payload must fail CRC check"
            );
        }

        // Truncated payload
        assert!(
            ManifestEntry::from_bytes(&bytes[4..bytes.len() - 1]).is_err(),
            "Truncated payload must be rejected"
        );
    }
}

#[tokio::test]
async fn test_recovery_crash_before_and_after_wal_floor_fsync() {
    use contextra_ports::keys::FileId;

    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");

    manifest
        .append(&ManifestEntry::Add {
            path: PathBuf::from("sst-1.sst"),
            max_tx: 50,
        })
        .await
        .expect("append add");

    // Case 1: Crash before WalFloor fsync (interrupted during write)
    let incomplete_floor = ManifestEntry::WalFloor {
        seq: 50,
        tail_mac: [0x55; 32],
        removed_file_ids: vec![FileId::new([10u8; 16]), FileId::new([11u8; 16])],
    };
    let mut raw_bytes = incomplete_floor.to_bytes().expect("to_bytes");
    // Simulate crash before fsync / torn write: write only partial bytes
    raw_bytes.truncate(raw_bytes.len() - 12);
    let mut file = tokio::fs::OpenOptions::new()
        .append(true)
        .open(&manifest_path)
        .await
        .expect("open file");
    tokio::io::AsyncWriteExt::write_all(&mut file, &raw_bytes)
        .await
        .expect("write partial");
    drop(file);
    drop(manifest);

    // On reopen: torn WalFloor at EOF must be discarded safely
    let loaded = Manifest::load(&manifest_path)
        .await
        .expect("load after crash before fsync");
    assert_eq!(loaded.len(), 1, "Incomplete WalFloor must be discarded");
    assert!(
        Manifest::current_wal_floor(&loaded).is_none(),
        "Floor must not be visible if crash happened before fsync"
    );

    // Case 2: Crash after fsync: WalFloor is durable and valid
    let dir2 = tempdir().expect("tempdir");
    let manifest2_path = dir2.path().join("MANIFEST");
    let reopened_manifest = Manifest::open(&manifest2_path).await.expect("reopen");
    reopened_manifest
        .append_wal_floor(
            60,
            [0x99; 32],
            vec![FileId::new([10u8; 16]), FileId::new([11u8; 16])],
        )
        .await
        .expect("append_wal_floor with durable fsync");
    drop(reopened_manifest);

    let loaded2 = Manifest::load(&manifest2_path)
        .await
        .expect("load after crash after fsync");
    assert_eq!(loaded2.len(), 1);
    let cur_floor = Manifest::current_wal_floor(&loaded2).expect("floor exists");
    assert_eq!(cur_floor.seq, 60);
    assert_eq!(cur_floor.tail_mac, [0x99; 32]);
    assert_eq!(
        cur_floor.removed_file_ids,
        vec![FileId::new([10u8; 16]), FileId::new([11u8; 16])]
    );
}
