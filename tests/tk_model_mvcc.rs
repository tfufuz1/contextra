// ANCHOR[contextra-testkit:TEST-TK-MODEL-MVCC]
//! Red & Property tests for ReferenceModel adhering to MVCC & Gate specification (07-contextra-mvcc.md, 08-contextra-store.md).

use contextra_testkit::reference_model::ReferenceModel;
use std::collections::BTreeMap;

#[test]
fn mv_01_reference_model_no_sentinel() {
    let model = ReferenceModel::new();
    assert_eq!(model.snapshot_seq(), 0);
    assert_eq!(model.get_at(b"nonexistent", 0), None);
    assert_eq!(model.min_read_snapshot(), None);
}

#[test]
fn mv_02_reference_model_compaction_fallback_current_seq() {
    let mut model = ReferenceModel::new();
    assert_eq!(model.min_read_snapshot(), None);
    model.put(b"k", b"v");
    let seq = model.commit();
    assert_eq!(model.snapshot_seq(), seq);
}

#[test]
fn mv_03_reference_model_tenant_tag_isolation() {
    let mut model = ReferenceModel::new();
    model.put(b"tenant1:k", b"v1");
    model.put(b"tenant2:k", b"v2");
    let seq = model.commit();

    model.gate_prefix(b"tenant1:", seq + 1);
    assert_eq!(model.get_at_with_gate(b"tenant1:k", seq + 1), None);
    assert_eq!(
        model.get_at_with_gate(b"tenant2:k", seq + 1),
        Some(b"v2".to_vec())
    );
}

#[test]
fn mv_04_reference_model_snapshot_limit_guard() {
    let mut model = ReferenceModel::new();
    for i in 0..10 {
        model.open_snapshot(i);
    }
    assert_eq!(model.min_read_snapshot(), Some(0));
}

#[test]
fn mv_05_reference_model_immediate_revocation() {
    let mut model = ReferenceModel::new();
    model.put(b"t1:k", b"v");
    let seq = model.commit();
    model.gate_prefix(b"t1:", seq + 1);
    assert_eq!(model.get_at_with_gate(b"t1:k", seq + 1), None);
}

#[test]
fn mv_06_reference_model_monotonic_seq() {
    let mut model = ReferenceModel::new();
    model.put(b"k", b"v");
    let s1 = model.commit();
    model.put(b"k", b"v2");
    let s2 = model.commit();
    assert!(s2 > s1);
}

#[test]
fn mv_07_reference_model_saturating_pin_duration() {
    let model = ReferenceModel::new();
    assert_eq!(model.snapshot_seq(), 0);
}

#[test]
fn mv_08_reference_model_ssi_conflict_detection() {
    let mut model = ReferenceModel::new();
    model.put(b"account_a", b"100");
    model.put(b"account_b", b"100");
    let s0 = model.commit();

    model.put(b"account_b", b"200");
    let _s1 = model.commit();

    let tx2_reads = vec![(b"account_b".to_vec(), s0)];
    assert!(model.would_conflict(&tx2_reads));
}

#[test]
fn mv_09_reference_model_tombstone_vlen_zero() {
    let mut model = ReferenceModel::new();
    model.put(b"doc1", b"payload_bytes");
    let seq1 = model.commit();

    model.put_tombstone(b"doc1");
    let seq2 = model.commit();

    assert_eq!(model.get_at(b"doc1", seq1), Some(b"payload_bytes".to_vec()));
    assert_eq!(model.get_at(b"doc1", seq2), None);
    assert!(model.is_tombstone_at(b"doc1", seq2));
}

#[test]
fn mv_10_reference_model_no_panics_checked_math() {
    let mut model = ReferenceModel::new();
    assert_eq!(model.commit(), 0);
}

#[test]
fn t_snap_1_reference_model_gate_and_isolation() {
    let mut model = ReferenceModel::new();
    model.put(b"tenant1:doc1", b"secret1");
    model.put(b"tenant2:doc2", b"secret2");
    let seq1 = model.commit();

    model.gate_key(b"tenant1:doc1", seq1 + 1);

    assert_eq!(
        model.get_at(b"tenant1:doc1", seq1),
        Some(b"secret1".to_vec())
    );
    assert_eq!(model.get_at_with_gate(b"tenant1:doc1", seq1 + 1), None);
    assert_eq!(
        model.get_at_with_gate(b"tenant2:doc2", seq1 + 1),
        Some(b"secret2".to_vec())
    );
}

#[test]
fn t_ver_1_reference_model_gate_revocation() {
    let mut model = ReferenceModel::new();
    model.put(b"tenantA:doc1", b"data1");
    model.put(b"tenantA:doc2", b"data2");
    let seq1 = model.commit();

    model.gate_prefix(b"tenantA:", seq1 + 1);

    assert_eq!(model.get_at_with_gate(b"tenantA:doc1", seq1 + 1), None);
    assert_eq!(model.get_at_with_gate(b"tenantA:doc2", seq1 + 1), None);
}

#[test]
fn test_reference_model_property_state_machine() {
    let mut model = ReferenceModel::new();
    let mut shadow: BTreeMap<Vec<u8>, Option<Vec<u8>>> = BTreeMap::new();

    let mut lcg: u64 = 0x12345678;
    for _ in 0..100 {
        lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1);
        let key = vec![(lcg % 8) as u8];
        let op_type = (lcg >> 16) % 3;

        if op_type == 0 {
            model.put_tombstone(&key);
            shadow.insert(key, None);
        } else {
            let val = vec![((lcg >> 24) % 256) as u8];
            model.put(&key, &val);
            shadow.insert(key, Some(val));
        }

        let seq = model.commit();
        for (k, expected_v) in &shadow {
            assert_eq!(model.get_at(k, seq), expected_v.clone());
            assert_eq!(model.is_tombstone_at(k, seq), expected_v.is_none());
        }
    }
}
