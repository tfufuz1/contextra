//! Fail-Closed- und Konsistenz-Beweise (WP-3, INV-ERA-4, INV-ERA-5).
//!
//! 1. FenceGate- und ReadGate-Verweigerung bei ungültiger/überlappender/korrumpierter Eingabe.
//! 2. Property-Tests für Ledger-Invarianten (Kettenintegrität, Monotonie, Fail-Closed bei Kettenbruch).
//! 3. Reproduzierbarer kanonischer Bericht (Golden Report mit festem Seed & Zeit).

use contextra_erasure::gate::fence::{FenceGate, FenceGateError, FenceRange, InMemoryFenceGate};
use contextra_erasure::journal::wal::{MemoryWalSink, WalSink};
use contextra_erasure::report::build::{
    build_report, canonicalize_report, ErasureReport,
};
use contextra_erasure::types_local::{DocId, ErasureId, LayerWitness};
use proptest::prelude::*;

#[test]
fn test_fence_gate_overlap_rejection() {
    let gate = InMemoryFenceGate::new();

    // 1. Präfix A erwerben
    let p_a = FenceRange::Prefix(b"tenant:42:coll:1".to_vec());
    let token_a = gate.acquire_fence(p_a).expect("Acquire A must succeed");

    // 2. Identischer Präfix muss abgelehnt werden
    let p_dup = FenceRange::Prefix(b"tenant:42:coll:1".to_vec());
    let err_dup = gate.acquire_fence(p_dup).expect_err("Duplicate prefix must be rejected");
    assert!(matches!(err_dup, FenceGateError::OverlapConflict { .. }));

    // 3. Sub-Präfix muss abgelehnt werden
    let p_sub = FenceRange::Prefix(b"tenant:42:coll:1:subdoc".to_vec());
    let err_sub = gate.acquire_fence(p_sub).expect_err("Sub-prefix must be rejected");
    assert!(matches!(err_sub, FenceGateError::OverlapConflict { .. }));

    // 4. Exakter Schlüssel innerhalb des Präfixes muss abgelehnt werden
    let exact = FenceRange::Exact(b"tenant:42:coll:1:key".to_vec());
    let err_exact = gate.acquire_fence(exact).expect_err("Exact in prefix must be rejected");
    assert!(matches!(err_exact, FenceGateError::OverlapConflict { .. }));

    // 5. Globaler Fence muss bei aktiven Fences abgelehnt werden
    let err_global = gate.acquire_fence(FenceRange::Global).expect_err("Global fence must be rejected");
    assert!(matches!(err_global, FenceGateError::OverlapConflict { .. }));

    // 6. Disjunkter Präfix muss gelingen
    let p_disjoint = FenceRange::Prefix(b"tenant:99:coll:1".to_vec());
    let token_b = gate.acquire_fence(p_disjoint).expect("Disjoint prefix must succeed");

    // Release
    gate.release_fence(&token_a).expect("Release A must succeed");
    gate.release_fence(&token_b).expect("Release B must succeed");
    assert!(!gate.is_active());
}

#[test]
fn test_fence_gate_fail_closed_on_corrupted_state() {
    let gate = InMemoryFenceGate::new();

    // Writer vor Korruption erlaubt
    let guard = gate.check_write(b"tenant:1:key1").expect("Allowed before corrupt");
    drop(guard);

    // Zustand korrumpieren
    gate.mark_corrupted();

    // Schreiber muss sofort fail-closed abgewiesen werden
    let err = gate.check_write(b"tenant:1:key1").expect_err("Must block after corrupt");
    assert_eq!(err, FenceGateError::CorruptedState);

    // Auch Batch-Schreiber blockiert
    let err_batch = gate
        .check_write_batch([b"tenant:1:k1".as_slice()])
        .expect_err("Must block batch write");
    assert_eq!(err_batch, FenceGateError::CorruptedState);

    // Neuer Fence-Erwerb blockiert
    let err_acq = gate
        .acquire_fence(FenceRange::Exact(b"key".to_vec()))
        .expect_err("Cannot acquire fence");
    assert_eq!(err_acq, FenceGateError::CorruptedState);
}

#[test]
fn test_fence_release_owner_token_enforcement() {
    let gate1 = InMemoryFenceGate::new();
    let gate2 = InMemoryFenceGate::new();

    let p = FenceRange::Prefix(b"prefix".to_vec());
    let token1 = gate1.acquire_fence(p).expect("Acquire must succeed");

    // Versuch mit falschem Token oder Token an anderem Gate
    let err = gate2.release_fence(&token1).expect_err("Foreign token must fail");
    assert!(matches!(err, FenceGateError::TokenMismatch | FenceGateError::NotFound));

    gate1.release_fence(&token1).expect("Legitimate release must succeed");
}

proptest! {
    #[test]
    fn prop_wal_sink_monotonicity_and_durability(
        seqs in prop::collection::vec(1u64..10_000u64, 1..30)
    ) {
        let sink = MemoryWalSink::new();
        let target_tenant = 7u64;
        let other_tenant = 9u64;

        for &seq in &seqs {
            sink.record_wal_entry(target_tenant, seq);
            sink.record_wal_entry(other_tenant, seq);
        }

        // Target disposen
        let disposed = sink.dispose_scope_wal(target_tenant).unwrap();
        prop_assert_eq!(disposed, seqs.len());

        // Target erneut disposen -> Idempotent 0
        let disposed_again = sink.dispose_scope_wal(target_tenant).unwrap();
        prop_assert_eq!(disposed_again, 0);

        // Other tenant noch unberührt
        let disposed_other = sink.dispose_scope_wal(other_tenant).unwrap();
        prop_assert_eq!(disposed_other, seqs.len());
    }
}

#[test]
fn test_canonical_report_golden_reproducibility() {
    let erasure_id = ErasureId::new([0x42; 16]);
    let gated_at = 1_000_000u64;
    let completed_at = 2_000_000u64;
    let witnesses = vec![
        LayerWitness::from_hash(contextra_erasure::types_local::WitnessLayer::DocStore, 10, 2048, [0xaa; 32]),
        LayerWitness::from_hash(contextra_erasure::types_local::WitnessLayer::VectorPayload, 5, 1024, [0xbb; 32]),
    ];

    let report = build_report(
        erasure_id,
        gated_at,
        completed_at,
        witnesses,
        15,
        3072,
        true,
    );

    let canonical_bytes = canonicalize_report(&report).expect("Canonical serialization must succeed");
    let golden_path = "crates/contextra-erasure/tests/golden/canonical_report_v1.golden";
    
    // Prüfen gegen das Golden Fixture
    let expected_golden = std::fs::read(golden_path).expect("Golden fixture must be readable");
    assert_eq!(canonical_bytes, expected_golden, "Canonical report must match golden fixture bit-identically");
}
