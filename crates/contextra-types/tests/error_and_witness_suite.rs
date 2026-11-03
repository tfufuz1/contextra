//! Integrations-Testsuite für Löschtypen, Zeugen, Mandantenisolation und Tombstones (SPEC 5.1, 7.3, 8.1, 12.1, 12.2, TY-10).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use contextra_types::*;
use core::time::Duration;
use proptest::prelude::*;

#[test]
/// Prüft Konstruktion, Feldzugriff und JSON-Serialisierung von `TenantContext` (SPEC D-07, 7.3).
fn test_tenant_context_creation_and_fields() {
    let t_ref = TenantRef::try_new(b"tenant-42".to_vec()).unwrap();
    let ctx = TenantContext::new(t_ref.clone());
    assert_eq!(ctx.tenant(), &t_ref);

    let json = serde_json::to_string(&ctx).unwrap();
    let deser: TenantContext = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.tenant(), &t_ref);
}

#[test]
/// Prüft typisierten Scanner, Positivkontrolle (ControlEvidence) und Zeugenerzeugung (SPEC 8.1, 12.2, D-72, TY-10).
fn test_typed_scanner_witness_and_residue_semantics() {
    let layer = LayerId::new([0x01; 16]);
    let class = MeasureClass::Mk2;
    let method_id = 42u16;

    // 1. Unarmed Scanner anlegen
    let scanner_unarmed = Scanner::new(layer, class, method_id);

    // 2. Scharfschalten via ControlEvidence (Positivkontrolle)
    let evidence = ControlEvidence::observed();
    let mut scanner_armed = scanner_unarmed.arm(evidence);

    // 3. Durchsuchte Einheiten aufzeichnen
    scanner_armed.record(10, 2048);

    // 4. Erfolgreicher Abschluss ohne Funde (matches == 0) -> erzeugt LayerWitness
    let hash = [0x55u8; 32];
    let duration = Duration::from_millis(15);
    let witness_res = scanner_armed.finish(0, duration, hash);
    assert!(
        witness_res.is_ok(),
        "finish(0) muss Ok(LayerWitness) liefern"
    );

    let witness = witness_res.unwrap();
    assert_eq!(witness.layer(), layer);
    assert_eq!(witness.class(), class);
    assert_eq!(witness.method_id(), method_id);
    assert_eq!(witness.files(), 10);
    assert_eq!(witness.bytes(), 2048);
    assert_eq!(witness.duration(), duration);
    assert_eq!(witness.hash(), &hash);
    assert_eq!(
        witness.matches(),
        0,
        "LayerWitness darf per Konstruktion 0 Funde haben (I-19)"
    );

    // 5. Fehlgeschlagener Abschluss bei Funden (matches > 0) -> erzeugt Residue
    let scanner_fail = Scanner::new(layer, class, method_id).arm(evidence);
    let residue_res = scanner_fail.finish(3, duration, hash);
    assert!(
        residue_res.is_err(),
        "finish(matches > 0) muss Err(Residue) liefern"
    );
    let residue = residue_res.unwrap_err();
    assert_eq!(residue.matches, 3);
    assert_eq!(
        residue.to_string(),
        "erasure residue detected: 3 matches found"
    );

    // 6. Messklassen und Serde-Prüfung
    for mc in [MeasureClass::Mk1, MeasureClass::Mk2, MeasureClass::Mk3] {
        let json_mc = serde_json::to_string(&mc).unwrap();
        let deser_mc: MeasureClass = serde_json::from_str(&json_mc).unwrap();
        assert_eq!(mc, deser_mc);
        assert!(!format!("{mc:?}").is_empty());
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
