use contextra_testkit::{RefOp, ReferenceModel};
use std::ops::Bound;

#[test]
fn test_j37_reference_model_methods_integration() {
    let mut model = ReferenceModel::new();

    // Stage put and delete using commit_batch
    let seq1 = model.commit_batch(vec![
        RefOp::Put {
            key: b"app:setting1".to_vec(),
            value: b"value1".to_vec(),
        },
        RefOp::Put {
            key: b"app:setting2".to_vec(),
            value: b"value2".to_vec(),
        },
        RefOp::Put {
            key: b"user:100".to_vec(),
            value: b"alice".to_vec(),
        },
    ]);
    assert_eq!(seq1, 1);

    // Verify get_latest
    assert_eq!(model.get_latest(b"app:setting1"), Some(b"value1".to_vec()));
    assert_eq!(model.get_latest(b"user:100"), Some(b"alice".to_vec()));

    // Verify scan_prefix_latest
    let app_entries = model.scan_prefix_latest(b"app:");
    assert_eq!(
        app_entries,
        vec![
            (b"app:setting1".to_vec(), b"value1".to_vec()),
            (b"app:setting2".to_vec(), b"value2".to_vec()),
        ]
    );

    // Stage second batch
    let seq2 = model.commit_batch(vec![
        RefOp::Put {
            key: b"app:setting1".to_vec(),
            value: b"value1_updated".to_vec(),
        },
        RefOp::Delete {
            key: b"app:setting2".to_vec(),
        },
    ]);
    assert_eq!(seq2, 2);

    // Verify range query at seq1
    let range_seq1 = model.scan_range_at(
        Bound::Included(b"app:setting1".as_slice()),
        Bound::Included(b"app:setting2".as_slice()),
        seq1,
    );
    assert_eq!(
        range_seq1,
        vec![
            (b"app:setting1".to_vec(), b"value1".to_vec()),
            (b"app:setting2".to_vec(), b"value2".to_vec()),
        ]
    );

    // Verify snapshot_map_at at seq2 (which delegates to scan_range_at)
    let map_seq2 = model.snapshot_map_at(seq2);
    assert_eq!(
        map_seq2.get(b"app:setting1".as_slice()),
        Some(&b"value1_updated".to_vec())
    );
    assert!(!map_seq2.contains_key(b"app:setting2".as_slice()));
    assert_eq!(
        map_seq2.get(b"user:100".as_slice()),
        Some(&b"alice".to_vec())
    );
}
