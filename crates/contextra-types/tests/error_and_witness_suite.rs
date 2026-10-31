//! Integrations-Testsuite für Fehlerpfade, RequestContext und Saga-Protokolle.
//! Stellt sicher, dass Restmessungstypen und Tombstones fehlerfrei interagieren.

//! Error, Witness, Tombstone, and TenantScoped isolation test suite for `contextra-types`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use contextra_types::*;
use proptest::prelude::*;

#[test]
/// Prüft Konstruktion, Feldzugriff und JSON-Serialisierung von `RequestContext`.
fn test_request_context_creation_and_fields() {
    let t_id = TenantId::try_new(42).unwrap();
    let tx_id = TxId::new(100);

    let ctx = RequestContext::new(t_id, tx_id);
    assert_eq!(ctx.tenant_id, t_id);
    assert_eq!(ctx.tx_id, tx_id);

    let json = serde_json::to_string(&ctx).unwrap();
    let deser: RequestContext = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.tenant_id, t_id);
    assert_eq!(deser.tx_id, tx_id);
}

#[test]
/// Prüft `SagaRecord`-Hash-Felder und die Serialisierung aller `ScanMethod`-Varianten.
fn test_saga_record_and_scan_methods() {
    let hash = [0x55u8; 32];
    let saga = SagaRecord::new(hash);
    assert_eq!(saga.deleted_keys_hash, hash);

    let json = serde_json::to_string(&saga).unwrap();
    let deser: SagaRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.deleted_keys_hash, hash);

    for method in [
        ScanMethod::Structural,
        ScanMethod::NotMeasured("compaction pending".to_string()),
        ScanMethod::FeatureDisabled,
    ] {
        let json_m = serde_json::to_string(&method).unwrap();
        let deser_m: ScanMethod = serde_json::from_str(&json_m).unwrap();
        assert_eq!(method, deser_m);
        assert!(!format!("{method:?}").is_empty());
    }
}

#[test]
/// Testet die Durchsetzung von Mandantengrenzen beim Entpacken mittels `into_inner_checked`.
fn test_tenant_scoped_wrapper_and_checked_extraction() {
    let tenant1 = TenantId::try_new(100).unwrap();
    let tenant2 = TenantId::try_new(200).unwrap();

    let scoped = TenantScoped::new(tenant1, "secret_data".to_string());
    assert_eq!(scoped.tenant_id(), &tenant1);

    // Mapped value
    let mapped = scoped.map(|s| s.len());
    assert_eq!(mapped.tenant_id(), &tenant1);

    // Valid extraction
    let val = mapped
        .into_inner_checked(&tenant1)
        .expect("Matching tenant must succeed");
    assert_eq!(val, 11);

    // Mismatched extraction
    let scoped_mismatch = TenantScoped::new(tenant1, "sensitive_value".to_string());
    let err = scoped_mismatch.into_inner_checked(&tenant2);
    assert!(err.is_err());
    if let Err(TenantScopeViolation::Mismatch { expected, actual }) = err {
        assert_eq!(expected, tenant2);
        assert_eq!(actual, tenant1);
    } else {
        panic!("Expected TenantScopeViolation error");
    }
}

#[test]
/// Prüft Bitmanipulationen und Flags zur Erkennung von Tombstones in Sequenznummern.
fn test_tombstone_bit_manipulations_and_semantics() {
    let seq_raw = 123456789u64;
    let tombstone_bytes = SeqBitTombstone::make_tombstone(seq_raw);
    assert_eq!(tombstone_bytes.len(), 8);

    let arr: [u8; 8] = tombstone_bytes.as_slice().try_into().unwrap();
    let encoded_seq = u64::from_le_bytes(arr);

    assert!(SeqBitTombstone::is_tombstone(encoded_seq, 0));
    assert_eq!(encoded_seq & !TOMBSTONE_BIT, seq_raw);

    // Flags bit 0 check
    assert!(SeqBitTombstone::is_tombstone(0, 0x01));
    assert!(!SeqBitTombstone::is_tombstone(0, 0x00));
    assert!(!SeqBitTombstone::is_tombstone(0, 0xFE));
}

proptest! {
    #[test]
        /// Property-Test: Tombstone-Bit 63 bleibt für beliebige u64-Sequenzen und Flags stets gesetzt.
    fn prop_tombstone_bit_invariants(seq in proptest::num::u64::ANY, flags in proptest::num::u8::ANY) {
        let tombstone_bytes = SeqBitTombstone::make_tombstone(seq);
        let arr: [u8; 8] = tombstone_bytes.as_slice().try_into().unwrap();
        let encoded_seq = u64::from_le_bytes(arr);

        // Bit 63 must always be set in encoded seq
        prop_assert!(SeqBitTombstone::is_tombstone(encoded_seq, flags));
        prop_assert_eq!(encoded_seq & !TOMBSTONE_BIT, seq & !TOMBSTONE_BIT);
    }

    #[test]
        /// Property-Test: Unberechtigtes Entpacken mit abweichender TenantId schlägt immer fehl.
    fn prop_tenant_scoped_isolation(t1_raw in 1u64..=1000, t2_raw in 1001u64..=2000, data in "\\PC*") {
        let t1 = TenantId::try_new(t1_raw).unwrap();
        let t2 = TenantId::try_new(t2_raw).unwrap();

        let scoped = TenantScoped::new(t1, data.clone());
        let err = scoped.into_inner_checked(&t2);
        prop_assert!(err.is_err());
    }
}
