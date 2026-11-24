//! Integrationstests für Sonderziele Replace, Supersede, Restricted (SPEC §11.2, D-115, D-116, D-117, D-119).
//!
//! T-IDs:
//! - T-REPL-1, T-REPL-2, T-REPL-3
//! - T-SUP-1, T-SUP-2, T-SUP-3
//! - T-RESTR-1, T-RESTR-2, T-RESTR-3, T-RESTR-4
//! - T-ERID-1, T-ERID-2, T-ERID-3
//! - T-LINT-1

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_erasure::error::ErasureError;
use contextra_erasure::replace::{ReplacePlan, ReplaceStep};
use contextra_erasure::restricted::{
    OperatorAttestation, RestrictedAccess, RestrictedPurpose, RestrictedReadAudit, RestrictedState,
};
use contextra_erasure::supersede::{SupersedeMode, SupersedePlan};
use contextra_erasure::types_local::ErasureId;
use contextra_erasure::types_local::KeyCoverage;

/// SPEC: T-REPL-1 (D-115 Replace-Protokoll R1 bis R4, Reihenfolge I-25 und Fehler ReplacementMissing)
#[test]
fn test_t_repl_1_protocol_order_and_missing_replacement() {
    let erasure_id = ErasureId::new([0x11; 16]);
    let old_tag = [0xaa; 16];
    let new_tag = [0xbb; 16];

    let mut plan = ReplacePlan::new(erasure_id, old_tag, new_tag).expect("Valid plan");
    assert_eq!(plan.step(), ReplaceStep::R1Requested);

    // R1 -> R2
    plan.advance_to_r2_intent().expect("Advance to R2");
    assert_eq!(plan.step(), ReplaceStep::R2PreGate);

    // Wenn neues Dokument vor R3 fehlt: ReplacementMissing
    let missing_err = plan.verify_replacement_present(false).unwrap_err();
    match missing_err {
        ErasureError::ReplacementMissing => {}
        other => panic!("Erwartet ReplacementMissing, erhalten: {:?}", other),
    }

    // Wenn vorhanden: Weiter zu R3 (Gate altes Dokument)
    plan.verify_replacement_present(true)
        .expect("Replacement present");
    plan.advance_to_r3_gate_old().expect("Advance to R3");
    assert_eq!(plan.step(), ReplaceStep::R3GatedOld);

    // R3 -> R4 (KeyDestroyed -> Purging)
    plan.advance_to_r4_key_destroyed().expect("Advance to R4");
    assert_eq!(plan.step(), ReplaceStep::R4KeyDestroyed);
}

/// SPEC: T-REPL-2 (D-115 SameIdentity Ablehnung und Idempotenz)
#[test]
fn test_t_repl_2_same_identity_rejection_and_idempotence() {
    let erasure_id = ErasureId::new([0x12; 16]);
    let same_tag = [0xcc; 16];

    // SameIdentity: doc_tag(new) == doc_tag(old) muss verweigert werden
    let err = ReplacePlan::new(erasure_id, same_tag, same_tag).unwrap_err();
    match err {
        ErasureError::SameIdentity => {}
        other => panic!("Erwartet SameIdentity, erhalten: {:?}", other),
    }

    // Idempotenz bei Schritten
    let diff_tag = [0xdd; 16];
    let mut plan = ReplacePlan::new(erasure_id, same_tag, diff_tag).expect("Valid plan");
    plan.advance_to_r2_intent().expect("R2");
    // Wiederholter Schritt führt nicht zu Fehler (idempotent)
    assert!(plan.advance_to_r2_intent().is_ok());
}

/// SPEC: T-REPL-3 (D-115 replacement_tag im Bericht und Serde-Roundtrip)
#[test]
fn test_t_repl_3_replacement_tag_and_serde() {
    let erasure_id = ErasureId::new([0x13; 16]);
    let old_tag = [0x10; 16];
    let new_tag = [0x20; 16];

    let plan = ReplacePlan::new(erasure_id, old_tag, new_tag).expect("Valid plan");
    assert_eq!(*plan.replacement_tag(), new_tag);

    let serialized = serde_json::to_string(&plan).expect("Serialize ReplacePlan");
    let deserialized: ReplacePlan =
        serde_json::from_str(&serialized).expect("Deserialize ReplacePlan");
    assert_eq!(plan, deserialized);
}

/// SPEC: T-SUP-1 (D-117 Matcher PrefixSeq Match-Logik und Datenstruktur)
#[test]
fn test_t_sup_1_prefix_seq_matcher() {
    let erasure_id = ErasureId::new([0x21; 16]);
    let prefix = [0x42; 65];
    let split_seq = 100u64;

    let plan = SupersedePlan::new(
        erasure_id,
        prefix,
        split_seq,
        SupersedeMode::PurgeOldVersions,
    );
    let matcher = plan.matcher();

    // Schlüssel mit gleichem Präfix und seq < 100 muss matchen
    assert!(matcher.matches(&prefix, 99));
    // Schlüssel mit gleichem Präfix und seq == 100 oder > 100 darf nicht matchen
    assert!(!matcher.matches(&prefix, 100));
    assert!(!matcher.matches(&prefix, 101));

    // Anderes Präfix darf nie matchen
    let other_prefix = [0x43; 65];
    assert!(!matcher.matches(&other_prefix, 50));
}

/// SPEC: T-SUP-2 (D-117 UpdateMissing Prüfung und Zustandsübergang)
#[test]
fn test_t_sup_2_update_missing_and_transitions() {
    let erasure_id = ErasureId::new([0x22; 16]);
    let prefix = [0x01; 65];
    let intent_seq = 50u64;

    let plan = SupersedePlan::new(
        erasure_id,
        prefix,
        intent_seq,
        SupersedeMode::PurgeOldVersions,
    );

    // Neuer Commit fehlt (seq <= intent_seq): UpdateMissing
    let err = plan.verify_updated_seq(50).unwrap_err();
    match err {
        ErasureError::UpdateMissing => {}
        other => panic!("Erwartet UpdateMissing, erhalten: {:?}", other),
    }

    // Neuer Commit vorhanden (seq > intent_seq)
    plan.verify_updated_seq(51).expect("Update present");
}

/// SPEC: T-SUP-3 (D-117 Klasse K ehrlich not_reached, key_coverage = None, Serde-Roundtrip)
#[test]
fn test_t_sup_3_class_k_not_reached_and_serde() {
    let erasure_id = ErasureId::new([0x23; 16]);
    let prefix = [0x05; 65];
    let plan = SupersedePlan::new(erasure_id, prefix, 200, SupersedeMode::PurgeOldVersions);

    // Klasse K wird bei Supersede nicht erreicht (DEK bleibt erhalten)
    assert_eq!(plan.key_coverage_store(), KeyCoverage::None);
    assert_eq!(plan.key_coverage_text(), KeyCoverage::None);
    assert_eq!(plan.key_coverage_vector(), KeyCoverage::None);
    assert!(!plan.is_key_destroyed());

    let serialized = serde_json::to_string(&plan).expect("Serialize SupersedePlan");
    let deserialized: SupersedePlan =
        serde_json::from_str(&serialized).expect("Deserialize SupersedePlan");
    assert_eq!(plan, deserialized);
}

/// SPEC: T-RESTR-1 (D-116, D-15 Zustandsmodell Restricted -> Lifted | Erase)
#[test]
fn test_t_restr_1_state_model_and_erase_override() {
    let mut state = RestrictedState::Restricted;
    assert!(!state.is_lifted());
    assert!(!state.is_terminal());

    // Aufhebung mit Journal-Begründung
    state
        .lift("LegalObligationFulfilled")
        .expect("Lift restriction");
    assert!(state.is_lifted());
    assert!(state.is_terminal());

    // Neuer Zustand Restricted
    let mut state2 = RestrictedState::Restricted;
    // Erase auf Restricted ist zulässig (SupersededByErasure)
    state2.supersede_by_erasure().expect("Supersede by erasure");
    assert_eq!(state2, RestrictedState::SupersededByErasure);
    assert!(state2.is_terminal());
}

/// SPEC: T-RESTR-2 (D-116 Audit-Eintrag vor Auslieferung Code 9 RestrictedRead, case_commitment)
#[test]
fn test_t_restr_2_audit_record_code_9_and_commitment() {
    let erasure_id = ErasureId::new([0x31; 16]);
    let epoch_key = [0x77; 32];
    let case_ref = b"internal-case-9876";

    let attestation = OperatorAttestation::new("operator-1", case_ref, 1_700_000_000, &epoch_key);
    let access = RestrictedAccess::new(RestrictedPurpose::LegalClaims, attestation);

    let audit_record = RestrictedReadAudit::create_entry(
        erasure_id,
        &access,
        100, // doc_count
        1_700_000_010,
    )
    .expect("Audit record created");

    assert_eq!(audit_record.record_code, 9);
    assert_eq!(audit_record.doc_count, 100);
    // case_ref darf nicht im Klartext im Audit stehen (P13, D-116)
    assert_ne!(audit_record.case_commitment, [0u8; 32]);
    let raw_str = String::from_utf8_lossy(&audit_record.case_commitment);
    assert!(!raw_str.contains("internal-case-9876"));
}

/// SPEC: T-RESTR-3 (D-116 Nur Restricted-Bereiche, Erased wird verweigert)
#[test]
fn test_t_restr_3_only_restricted_target() {
    let epoch_key = [0x88; 32];
    let attestation = OperatorAttestation::new("op", b"case", 1_000, &epoch_key);
    let access = RestrictedAccess::new(RestrictedPurpose::DataSubjectConsent, attestation);

    assert!(access.allows_access_to_restricted(true));
    // Gegen Erased liefert es immer False / Verweigerung
    assert!(!access.allows_access_to_restricted(false));
}

/// SPEC: T-RESTR-4 (D-116 Stapelgrenze 1000 Dokumente)
#[test]
fn test_t_restr_4_batch_limit_1000() {
    let epoch_key = [0x99; 32];
    let attestation = OperatorAttestation::new("op", b"case", 1_000, &epoch_key);
    let access = RestrictedAccess::new(RestrictedPurpose::LegalClaims, attestation);

    assert!(access.validate_batch_size(1000).is_ok());
    assert!(access.validate_batch_size(1).is_ok());

    let err = access.validate_batch_size(1001).unwrap_err();
    match err {
        ErasureError::LimitExceeded(_) => {}
        other => panic!("Erwartet LimitExceeded, erhalten: {:?}", other),
    }
}

/// SPEC: T-ERID-1 (D-119 Serde Roundtrip für alle Sonderzieltypen)
#[test]
fn test_t_erid_1_serde_roundtrips() {
    let epoch_key = [0x55; 32];
    let attestation = OperatorAttestation::new("admin", b"case-1", 12345, &epoch_key);
    let access = RestrictedAccess::new(RestrictedPurpose::LegalClaims, attestation);

    let serialized = serde_json::to_string(&access).expect("Serialize RestrictedAccess");
    let deserialized: RestrictedAccess =
        serde_json::from_str(&serialized).expect("Deserialize RestrictedAccess");
    assert_eq!(access, deserialized);
}

/// SPEC: T-LINT-1 (Wortlaut-Prüfung nach SPEC 6.4 auf allen Modul-Fehlertexten)
#[test]
fn test_t_lint_1_wording_compliance() {
    let forbidden = [
        "garantiert",
        "100 %",
        "100%",
        "DSGVO-konform",
        "GDPR-compliant",
        "rechtskonform",
        "unwiederbringlich",
        "irrecoverable",
        "unwiderruflich",
        "informationstheoretisch",
    ];

    let err_samples = [
        format!("{:?}", ErasureError::ReplacementMissing),
        format!("{:?}", ErasureError::UpdateMissing),
        format!("{:?}", ErasureError::SameIdentity),
        format!("{:?}", ErasureError::RestrictionViolated("test".into())),
        format!("{:?}", ErasureError::RestrictedReadAuditRequired),
    ];

    for s in &err_samples {
        for f in &forbidden {
            assert!(
                !s.to_lowercase().contains(&f.to_lowercase()),
                "Verbotener Begriff '{}' in Fehlermeldung: {}",
                f,
                s
            );
        }
    }
}
