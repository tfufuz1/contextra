// FILE-CONTEXT
// STAND: 2026-10-22
// ZWECK: Property-based testing for ModelStore differential oracle semantics.
// INVARIANTEN: Monotonic MVCC sequence semantics, scope purge correctness.

use contextra_testkit::model_store::ModelStore;
use contextra_testkit::reference_model::RefOp;

#[test]
fn model_store_semantics_proptest() {
    let mut store = ModelStore::new();

    // 1. Initial state empty
    assert_eq!(store.get_at(0, b"key1"), None);

    // 2. Apply batch 1: put key1, key2
    let seq1 = store.apply_batch(vec![
        RefOp::Put {
            key: b"tenant1/doc1".to_vec(),
            value: b"v1_seq1".to_vec(),
        },
        RefOp::Put {
            key: b"tenant1/doc2".to_vec(),
            value: b"v2_seq1".to_vec(),
        },
        RefOp::Put {
            key: b"tenant2/doc1".to_vec(),
            value: b"v3_seq1".to_vec(),
        },
    ]);
    assert_eq!(seq1, 1);

    // 3. Apply batch 2: update tenant1/doc1, delete tenant1/doc2
    let seq2 = store.apply_batch(vec![
        RefOp::Put {
            key: b"tenant1/doc1".to_vec(),
            value: b"v1_seq2".to_vec(),
        },
        RefOp::Delete {
            key: b"tenant1/doc2".to_vec(),
        },
    ]);
    assert_eq!(seq2, 2);

    // Verify MVCC isolation
    assert_eq!(
        store.get_at(seq1, b"tenant1/doc1"),
        Some(b"v1_seq1".to_vec())
    );
    assert_eq!(
        store.get_at(seq2, b"tenant1/doc1"),
        Some(b"v1_seq2".to_vec())
    );
    assert_eq!(
        store.get_at(seq1, b"tenant1/doc2"),
        Some(b"v2_seq1".to_vec())
    );
    assert_eq!(store.get_at(seq2, b"tenant1/doc2"), None);
    assert_eq!(
        store.get_at(seq2, b"tenant2/doc1"),
        Some(b"v3_seq1".to_vec())
    );

    // Purge scope "tenant1/" as of seq1
    store.purge_scope(seq1, b"tenant1/");

    // Seq 2 vision must remain unaffected
    assert_eq!(
        store.get_at(seq2, b"tenant1/doc1"),
        Some(b"v1_seq2".to_vec())
    );
    assert_eq!(
        store.get_at(seq2, b"tenant2/doc1"),
        Some(b"v3_seq1".to_vec())
    );

    // Randomized property simulation loop
    let mut rng_state: u64 = 987654321;
    let mut model = ModelStore::new();

    for i in 0..1000 {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let key_idx = (rng_state % 5) as u8;
        let key = format!("tenant_{}/key", key_idx).into_bytes();
        let val = format!("val_{}", i).into_bytes();

        let seq = if rng_state % 3 == 0 {
            model.apply_batch(vec![RefOp::Delete { key: key.clone() }])
        } else {
            model.apply_batch(vec![RefOp::Put {
                key: key.clone(),
                value: val.clone(),
            }])
        };

        let active_val = model.get_at(seq, &key);
        if rng_state % 3 == 0 {
            assert_eq!(active_val, None);
        } else {
            assert_eq!(active_val, Some(val));
        }
    }
}
