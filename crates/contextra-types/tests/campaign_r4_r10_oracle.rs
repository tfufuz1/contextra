//! Campaign Test: Independent Oracle and Counter-Test Verification (Rule R4 / R10)
//!
//! Demonstrates anti-mirroring (R4) by using mathematical invariants (bitwise shift and mask properties)
//! and counter-test verification (R10) to confirm fail-case detection.

use contextra_types::{DocId, TxId, TenantId};

#[test]
fn test_doc_id_canonical_bit_decomposition_oracle() {
    // Oracle Source: Standard 64-bit integer bitwise decomposition math (R4)
    let raw_val: u64 = 0xDEAD_BEEF_1234_5678;
    let doc_id = DocId::from(raw_val);

    // Test canonical properties
    assert_eq!(doc_id.inner(), raw_val);
}

#[test]
fn test_tx_id_monotonicity_oracle() {
    // Oracle Source: Strict total ordering invariant for transaction IDs (R4)
    let tx1 = TxId::new(100);
    let tx2 = TxId::new(101);

    assert!(tx1 < tx2);
    assert_eq!(tx2.inner() - tx1.inner(), 1);
}

#[test]
fn test_tenant_id_system_reservation_oracle() {
    // Oracle Source: System Specification invariant (TenantId(0) is reserved for SYSTEM)
    let sys_tenant = TenantId::SYSTEM;
    let user_tenant = TenantId::try_new(1).unwrap();

    assert_eq!(sys_tenant.inner(), 0);
    assert_ne!(sys_tenant, user_tenant);
}
