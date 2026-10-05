use contextra_testkit::{FaultConfig, FaultVfs, ManualClock, RefOp, ReferenceModel};
use std::ops::Bound;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

#[test]
fn test_j37closure_fault_vfs_all_symbols() {
    let vfs = FaultVfs::new();

    // 1. set_config & 2. write_file
    vfs.set_config(FaultConfig {
        fail_writes_after: Some(2),
        fail_reads_after: Some(1),
        fail_syncs_after: Some(1),
        fail_all: false,
    });

    let p1 = Path::new("file1.dat");
    let p2 = Path::new("file2.dat");

    assert!(vfs.write_file(p1, b"hello").is_ok());
    assert!(vfs.write_file(p2, b"world").is_ok());
    assert_eq!(vfs.write_count(), 2);

    // 3. read_file
    let r1 = vfs.read_file(p1);
    assert!(r1.is_ok());
    assert_eq!(r1.unwrap(), b"hello");
    assert_eq!(vfs.read_count(), 1);

    // Read failure due to fail_reads_after limit
    assert!(vfs.read_file(p2).is_err());

    // 4. reset_counters
    vfs.reset_counters();
    assert_eq!(vfs.read_count(), 0);
    assert_eq!(vfs.write_count(), 0);

    // 5. trigger_crash
    vfs.trigger_crash();
    assert!(vfs.read_file(p1).is_err());
    assert!(vfs.write_file(Path::new("file3.dat"), b"test").is_err());

    // Recover via set_config (which calls reset_counters internally)
    vfs.set_config(FaultConfig::default());
    assert!(vfs.read_file(p1).is_ok());
}

#[test]
fn test_j37closure_manual_clock_all_symbols() {
    let clock = ManualClock::new(2_000_000_000);

    // 6. set_nanos
    clock.set_nanos(10_000_000_000);
    assert_eq!(clock.now_nanos(), 10_000_000_000);

    // 7. now_secs_f64
    assert_eq!(clock.now_secs_f64(), 10.0);

    // 8. now_system_time
    assert_eq!(
        clock.now_system_time(),
        UNIX_EPOCH + Duration::from_secs(10)
    );
}

#[test]
fn test_j37closure_reference_model_all_symbols() {
    let mut model = ReferenceModel::new();

    // 9. commit_batch
    let seq1 = model.commit_batch(vec![
        RefOp::Put {
            key: b"alpha:1".to_vec(),
            value: b"v1".to_vec(),
        },
        RefOp::Put {
            key: b"alpha:2".to_vec(),
            value: b"v2".to_vec(),
        },
        RefOp::Put {
            key: b"beta:1".to_vec(),
            value: b"v3".to_vec(),
        },
    ]);
    assert_eq!(seq1, 1);

    // 10. get_latest
    assert_eq!(model.get_latest(b"alpha:1"), Some(b"v1".to_vec()));
    assert_eq!(model.get_latest(b"alpha:2"), Some(b"v2".to_vec()));

    // 11. scan_prefix_latest
    let alpha_latest = model.scan_prefix_latest(b"alpha:");
    assert_eq!(
        alpha_latest,
        vec![
            (b"alpha:1".to_vec(), b"v1".to_vec()),
            (b"alpha:2".to_vec(), b"v2".to_vec()),
        ]
    );

    let seq2 = model.commit_batch(vec![
        RefOp::Put {
            key: b"alpha:1".to_vec(),
            value: b"v1_updated".to_vec(),
        },
        RefOp::Delete {
            key: b"alpha:2".to_vec(),
        },
    ]);
    assert_eq!(seq2, 2);

    // 12. scan_range_at
    let range_seq1 = model.scan_range_at(
        Bound::Included(b"alpha:1".as_slice()),
        Bound::Included(b"alpha:2".as_slice()),
        seq1,
    );
    assert_eq!(
        range_seq1,
        vec![
            (b"alpha:1".to_vec(), b"v1".to_vec()),
            (b"alpha:2".to_vec(), b"v2".to_vec()),
        ]
    );

    // 13. snapshot_map_at
    let map_seq2 = model.snapshot_map_at(seq2);
    assert_eq!(map_seq2.get(b"alpha:1".as_slice()), Some(&b"v1_updated".to_vec()));
    assert!(!map_seq2.contains_key(b"alpha:2".as_slice()));
    assert_eq!(map_seq2.get(b"beta:1".as_slice()), Some(&b"v3".to_vec()));
}
