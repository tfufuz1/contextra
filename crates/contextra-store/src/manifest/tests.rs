//! # Manifest-Tests (Ring 1)
//!
//! Validiert Rollover, Recovery und atomare Manifest-Commits.

use super::*;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[test]
fn manifest_v2_batch_and_snapshot_frames_roundtrip() {
    let batch = vec![
        ManifestEntry::Add {
            path: PathBuf::from("live.sst"),
            max_tx: 22,
        },
        ManifestEntry::Replace {
            removed: vec![PathBuf::from("old.sst")],
            added: PathBuf::from("new.sst"),
            added_max_tx: 23,
        },
        ManifestEntry::WalCheckpoint { hmac: [0x37; 32] },
    ];
    let encoded = super::entry::encode_batch_frame(&batch).expect("encode batch");
    let frame = encoded.get(4..).expect("frame payload");
    let decoded = super::entry::decode_v2_frame(frame).expect("decode batch");
    assert_eq!(decoded[0], batch[0]);
    assert_eq!(
        decoded[1],
        ManifestEntry::Replace {
            removed: vec![PathBuf::from("old.sst")],
            added: PathBuf::from("new.sst"),
            added_max_tx: 23,
        }
    );
    assert_eq!(decoded[2], batch[2]);

    let state = super::state::ManifestState::from_entries(&[
        ManifestEntry::Add {
            path: PathBuf::from("live.sst"),
            max_tx: 22,
        },
        ManifestEntry::Add {
            path: PathBuf::from("dead.sst"),
            max_tx: 21,
        },
        ManifestEntry::Remove {
            path: PathBuf::from("dead.sst"),
        },
        ManifestEntry::RollbackComplete { target_tx: 9 },
        ManifestEntry::WalCheckpoint { hmac: [0x37; 32] },
    ])
    .expect("reduce state");
    let snapshot = state.snapshot_frame().expect("encode snapshot");
    let decoded = super::entry::decode_v2_frame(snapshot.get(4..).expect("snapshot payload"))
        .expect("decode snapshot");
    let restored =
        super::state::ManifestState::from_entries(&decoded).expect("restore snapshot state");
    assert_eq!(restored.live(), state.live());
    assert_eq!(restored.dead(), state.dead());
    assert_eq!(restored.wal_hwm(), state.wal_hwm());
}

#[tokio::test]
async fn manifest_append_batch_is_one_frame_and_validation_is_atomic() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("MANIFEST");
    let manifest = Manifest::open(&path).await.expect("open manifest");
    let batch = [
        ManifestEntry::Add {
            path: PathBuf::from("batch-a.sst"),
            max_tx: 1,
        },
        ManifestEntry::WalCheckpoint { hmac: [0x42; 32] },
    ];
    manifest.append_batch(&batch).await.expect("append batch");

    let bytes = tokio::fs::read(&path).await.expect("read manifest");
    let snapshot_len = usize::try_from(u32::from_le_bytes(
        bytes
            .get(5..9)
            .expect("snapshot length")
            .try_into()
            .expect("u32"),
    ))
    .expect("snapshot size");
    let batch_offset = 5usize + 4 + snapshot_len;
    let batch_len = usize::try_from(u32::from_le_bytes(
        bytes
            .get(batch_offset..batch_offset + 4)
            .expect("batch length")
            .try_into()
            .expect("u32"),
    ))
    .expect("batch size");
    let batch_frame = bytes
        .get(batch_offset + 4..batch_offset + 4 + batch_len)
        .expect("batch frame");
    assert_eq!(
        batch_frame.get(4),
        Some(&1),
        "one Batch frame stores the entries"
    );
    assert_eq!(
        super::entry::decode_v2_frame(batch_frame)
            .expect("decode batch frame")
            .len(),
        2
    );

    let before = bytes.len();
    let invalid_batch = [
        ManifestEntry::Add {
            path: PathBuf::from("batch-b.sst"),
            max_tx: 2,
        },
        ManifestEntry::Remove {
            path: PathBuf::from("missing.sst"),
        },
    ];
    assert!(manifest.append_batch(&invalid_batch).await.is_err());
    let after = tokio::fs::metadata(&path)
        .await
        .expect("manifest metadata")
        .len();
    assert_eq!(after, u64::try_from(before).expect("file size"));
    assert!(!manifest
        .state_snapshot()
        .await
        .live()
        .contains_key(Path::new("batch-b.sst")));
}

#[tokio::test]
async fn manifest_rollover_keeps_wal_hwm_and_dead_sstables() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("MANIFEST");
    let hwm = [0x5au8; 32];
    let live = PathBuf::from("live.sst");
    let dead = PathBuf::from("dead.sst");
    let manifest = Manifest::open(&path).await.expect("open manifest");
    for entry in [
        ManifestEntry::Add {
            path: live.clone(),
            max_tx: 7,
        },
        ManifestEntry::Add {
            path: dead.clone(),
            max_tx: 5,
        },
        ManifestEntry::Remove { path: dead.clone() },
        ManifestEntry::WalCheckpoint { hmac: hwm },
    ] {
        manifest
            .append(&entry)
            .await
            .expect("append manifest entry");
    }
    manifest.rollover().await.expect("rollover");

    let entries = Manifest::load(&path).await.expect("load rolled manifest");
    assert_eq!(Manifest::extract_high_water_mark(&entries), Some(hwm));
    assert!(Manifest::reconstruct_dead_sstables(&entries).contains(&dead));
    assert!(Manifest::reconstruct_valid_sstables(&entries)
        .iter()
        .any(|(name, _)| name == &live));
}

#[tokio::test]
async fn manifest_rollover_racing_append_keeps_both_catalog_entries() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("MANIFEST");
    let manifest = std::sync::Arc::new(Manifest::open(&path).await.expect("open manifest"));
    manifest
        .append(&ManifestEntry::Add {
            path: PathBuf::from("before.sst"),
            max_tx: 1,
        })
        .await
        .expect("seed manifest");

    let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(3));
    let rollover_manifest = std::sync::Arc::clone(&manifest);
    let rollover_barrier = std::sync::Arc::clone(&barrier);
    let rollover = tokio::spawn(async move {
        rollover_barrier.wait().await;
        rollover_manifest.rollover().await
    });
    let append_manifest = std::sync::Arc::clone(&manifest);
    let append_barrier = std::sync::Arc::clone(&barrier);
    let append = tokio::spawn(async move {
        append_barrier.wait().await;
        append_manifest
            .append(&ManifestEntry::Add {
                path: PathBuf::from("racing.sst"),
                max_tx: 2,
            })
            .await
    });
    barrier.wait().await;
    rollover.await.expect("rollover task").expect("rollover");
    append.await.expect("append task").expect("append");

    let entries = Manifest::load(&path).await.expect("load raced manifest");
    let state = super::state::ManifestState::from_entries(&entries).expect("reduce state");
    assert_eq!(state.live().get(Path::new("before.sst")), Some(&1));
    assert_eq!(state.live().get(Path::new("racing.sst")), Some(&2));
}

#[tokio::test]
async fn manifest_load_propagates_vfs_read_errors() {
    let path = PathBuf::from("MANIFEST");
    let vfs = contextra_testkit::FaultVfs::new();
    vfs.write_file(&path, b"MFMN\x01").expect("seed manifest");
    vfs.set_config(contextra_testkit::FaultConfig {
        fail_reads_after: Some(0),
        ..Default::default()
    });
    let io = crate::io_seam::StoreIo::new(std::sync::Arc::new(vfs));
    assert!(Manifest::load_with_io(&path, io).await.is_err());
}

#[tokio::test]
async fn manifest_open_truncates_torn_tail_before_next_append() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("MANIFEST");
    let first = ManifestEntry::Add {
        path: PathBuf::from("first.sst"),
        max_tx: 1,
    };
    {
        let manifest = Manifest::open(&path).await.expect("open manifest");
        manifest.append(&first).await.expect("append first entry");
    }

    let mut torn_tail = 64u32.to_le_bytes().to_vec();
    torn_tail.extend_from_slice(&[0x11, 0x22, 0x33]);
    let mut file = tokio::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .await
        .expect("open manifest to simulate power loss");
    tokio::io::AsyncWriteExt::write_all(&mut file, &torn_tail)
        .await
        .expect("write torn tail");
    file.sync_all().await.expect("sync torn tail");
    drop(file);

    let manifest = Manifest::open(&path).await.expect("heal torn tail");
    let second = ManifestEntry::Add {
        path: PathBuf::from("second.sst"),
        max_tx: 2,
    };
    manifest
        .append(&second)
        .await
        .expect("append after healing");
    drop(manifest);
    let entries = Manifest::load(&path).await.expect("load healed manifest");
    let state = super::state::ManifestState::from_entries(&entries).expect("reduce healed entries");
    assert_eq!(state.live().get(Path::new("first.sst")), Some(&1));
    assert_eq!(state.live().get(Path::new("second.sst")), Some(&2));
}

#[tokio::test]
async fn manifest_rollover_post_rename_fault_poisons_handle() {
    let vfs = contextra_testkit::FaultVfs::new();
    let io = crate::io_seam::StoreIo::new(std::sync::Arc::new(vfs.clone()));
    let path = PathBuf::from("data/MANIFEST");
    let manifest = Manifest::open_with_io(&path, None, io.clone())
        .await
        .expect("open manifest on fault VFS");
    manifest
        .append(&ManifestEntry::Add {
            path: PathBuf::from("kept.sst"),
            max_tx: 11,
        })
        .await
        .expect("append live SSTable");

    vfs.reset_counters();
    vfs.set_config(contextra_testkit::FaultConfig {
        // Rollover syncs the temp append, the temp inode, then the renamed directory entry.
        fail_syncs_after: Some(2),
        ..Default::default()
    });
    assert!(manifest.rollover().await.is_err());
    assert!(manifest.is_poisoned());
    assert!(manifest
        .append(&ManifestEntry::WalCheckpoint { hmac: [9; 32] })
        .await
        .is_err());

    drop(manifest);
    vfs.reset_counters();
    vfs.set_config(contextra_testkit::FaultConfig::default());
    let recovered = Manifest::open_with_io(&path, None, io).await;
    assert!(
        recovered.is_ok(),
        "renamed snapshot must be readable after restart"
    );
    let entries = Manifest::load_with_io(
        &path,
        crate::io_seam::StoreIo::new(std::sync::Arc::new(vfs)),
    )
    .await
    .expect("load renamed snapshot");
    let state = super::state::ManifestState::from_entries(&entries).expect("reduce snapshot");
    assert_eq!(state.live().get(Path::new("kept.sst")), Some(&11));
}

#[tokio::test]
async fn manifest_rollover_crash_vfs_preserves_previously_committed_state() {
    use contextra_testkit::crash_vfs::{CrashPolicy, CrashSeed, CrashVfs};

    for crash_offset in 1..=6 {
        let vfs = CrashVfs::new(CrashSeed(0xC0FFEE));
        let io = crate::io_seam::StoreIo::new(std::sync::Arc::new(vfs.clone()));
        let dir = PathBuf::from("store");
        io.create_dir_all(&dir)
            .await
            .expect("create virtual store dir");
        io.sync_dir(Path::new(""))
            .await
            .expect("make store dir durable");
        let path = dir.join("MANIFEST");
        let manifest = Manifest::open_with_io(&path, None, io)
            .await
            .expect("open manifest on CrashVfs");
        manifest
            .append(&ManifestEntry::Add {
                path: PathBuf::from("committed.sst"),
                max_tx: 19,
            })
            .await
            .expect("commit initial manifest state");

        let crash_at = vfs.op_count() + crash_offset;
        vfs.crash_after(crash_at);
        let _ = manifest.rollover().await;
        assert!(vfs.is_crashed(), "CrashVfs missed mutation {crash_at}");
        drop(manifest);

        let restarted = vfs.restart(CrashPolicy::DropUnsynced);
        let restarted_io = crate::io_seam::StoreIo::new(std::sync::Arc::new(restarted));
        let recovered = Manifest::open_with_io(&path, None, restarted_io)
            .await
            .expect("recover manifest after power cut");
        let state = recovered.state_snapshot().await;
        assert_eq!(state.live().get(Path::new("committed.sst")), Some(&19));
    }
}

/// F5/INV-M4: ENOSPC während eines Appends darf weder den Zustand verändern noch einen halben
/// Frame hinterlassen, der das Manifest nach dem Neustart unlesbar macht.
#[tokio::test]
async fn manifest_append_enospc_midframe() {
    use contextra_testkit::crash_vfs::{CrashPolicy, CrashSeed, CrashVfs, ErrKind};

    let vfs = CrashVfs::new(CrashSeed(0xE05C));
    let io = crate::io_seam::StoreIo::new(std::sync::Arc::new(vfs.clone()));
    let dir = PathBuf::from("store");
    io.create_dir_all(&dir).await.expect("create store dir");
    io.sync_dir(Path::new(""))
        .await
        .expect("make store dir durable");
    let path = dir.join("MANIFEST");
    let manifest = Manifest::open_with_io(&path, None, io)
        .await
        .expect("open manifest");
    manifest
        .append(&ManifestEntry::Add {
            path: PathBuf::from("committed.sst"),
            max_tx: 7,
        })
        .await
        .expect("commit first entry");

    vfs.fail_write(&path, 0, ErrKind::NoSpace);
    let failed = manifest
        .append(&ManifestEntry::Add {
            path: PathBuf::from("lost.sst"),
            max_tx: 8,
        })
        .await;
    assert!(failed.is_err(), "append under ENOSPC must report failure");

    let live = manifest.state_snapshot().await;
    assert_eq!(live.live().get(Path::new("committed.sst")), Some(&7));
    assert!(
        !live.live().contains_key(Path::new("lost.sst")),
        "failed append must not change the in-memory state"
    );
    drop(manifest);

    let restarted = vfs.restart(CrashPolicy::DropUnsynced);
    let restarted_io = crate::io_seam::StoreIo::new(std::sync::Arc::new(restarted));
    let recovered = Manifest::open_with_io(&path, None, restarted_io)
        .await
        .expect("manifest must stay openable after ENOSPC");
    let state = recovered.state_snapshot().await;
    assert_eq!(state.live().get(Path::new("committed.sst")), Some(&7));
    assert!(!state.live().contains_key(Path::new("lost.sst")));
}

#[tokio::test]
async fn manifest_v1_is_migrated_with_backup_to_v2_snapshot() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("MANIFEST");
    let entry = ManifestEntry::Add {
        path: PathBuf::from("legacy.sst"),
        max_tx: 42,
    };
    let mut legacy = b"MFMN\x01".to_vec();
    legacy.extend_from_slice(&entry.to_bytes().expect("encode v1 entry"));
    tokio::fs::write(&path, &legacy)
        .await
        .expect("write v1 manifest");

    let manifest = Manifest::open(&path).await.expect("migrate v1 manifest");
    let bytes = tokio::fs::read(&path)
        .await
        .expect("read migrated manifest");
    assert_eq!(bytes.get(..5), Some(b"CMAN\x02".as_slice()));
    assert!(tokio::fs::try_exists(dir.path().join("MANIFEST.v1.bak"))
        .await
        .expect("check backup"));
    let entries = Manifest::load(&path).await.expect("load migrated entries");
    let state =
        super::state::ManifestState::from_entries(&entries).expect("reduce migrated entries");
    assert_eq!(state.live().get(Path::new("legacy.sst")), Some(&42));
    drop(manifest);
}

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

    // Corrupt the first batch while a later frame remains; mid-file CRC errors fail closed.
    let mut file_bytes = tokio::fs::read(&manifest_path).await.expect("read file");
    let snapshot_len =
        u32::from_le_bytes(file_bytes[5..9].try_into().expect("snapshot length")) as usize;
    let first_batch_offset = 5 + 4 + snapshot_len;
    file_bytes[first_batch_offset + 9] ^= 0xFF;
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
async fn manifest_rejects_complete_malformed_final_frame() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("MANIFEST");
    let manifest = Manifest::open(&path).await.expect("open manifest");
    manifest
        .append(&ManifestEntry::Add {
            path: PathBuf::from("valid.sst"),
            max_tx: 1,
        })
        .await
        .expect("append entry");
    drop(manifest);

    let mut bytes = tokio::fs::read(&path).await.expect("read manifest");
    let snapshot_len = usize::try_from(u32::from_le_bytes(
        bytes
            .get(5..9)
            .expect("snapshot length")
            .try_into()
            .expect("u32"),
    ))
    .expect("snapshot size");
    let frame_start = 5usize + 4 + snapshot_len;
    let frame_len = usize::try_from(u32::from_le_bytes(
        bytes
            .get(frame_start..frame_start + 4)
            .expect("frame length")
            .try_into()
            .expect("u32"),
    ))
    .expect("frame size");
    let payload_start = frame_start + 8;
    let frame_end = frame_start + 4 + frame_len;
    *bytes.get_mut(payload_start).expect("batch tag") = 99;
    let crc = crc32fast::hash(bytes.get(payload_start..frame_end).expect("frame payload"));
    bytes
        .get_mut(frame_start + 4..payload_start)
        .expect("frame CRC")
        .copy_from_slice(&crc.to_le_bytes());
    tokio::fs::write(&path, bytes)
        .await
        .expect("write malformed frame");

    assert!(
        Manifest::load(&path).await.is_err(),
        "a complete malformed frame with a valid CRC is corruption even at EOF"
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

    // Perform rollover with threshold lower than pre_size; state comes from Manifest itself.
    let rolled_over = manifest
        .maybe_rollover(100)
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

        let rolled = manifest.maybe_rollover(1).await.expect("rollover");
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

    manifest.rollover().await.expect("explicit rollover");

    let loaded = Manifest::load(&manifest_path).await.expect("load manifest");
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0], entry1);
}

#[tokio::test]
async fn test_manifest_poisoning_on_fsync_failure() {
    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");
    assert!(!manifest.is_poisoned());

    // Manually trigger poison state to simulate fsync failure (INV-FSYNC-1)
    manifest
        .is_poisoned
        .store(true, std::sync::atomic::Ordering::SeqCst);
    assert!(manifest.is_poisoned());

    let entry = ManifestEntry::Add {
        path: PathBuf::from("sst-fail.sst"),
        max_tx: 99,
    };
    let err = manifest.append(&entry).await;
    assert!(
        err.is_err(),
        "Poisoned manifest must reject append operations"
    );
    let err_msg = err.unwrap_err().to_string();
    assert!(
        err_msg.contains("poisoned") && err_msg.contains("INV-FSYNC-1"),
        "Error message should mention poisoned and INV-FSYNC-1"
    );
}
