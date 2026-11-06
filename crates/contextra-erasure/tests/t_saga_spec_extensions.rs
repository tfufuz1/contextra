//! Tests für D-115, D-116, D-117, D-119, D-125 (T-REPL-1..3, T-SUP-1..3, T-RESTR-2..4, T-ERID-1..3, T-EPOCH-1).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_erasure::gate::batch::evaluate_restricted_access;
use contextra_erasure::saga::{derive_open_key, EraseSaga, SagaState};
use contextra_erasure::vault::slot::{
    encode_slot, off, SlotType, STATUS_ACTIVE, STATUS_TOMBSTONE, VAULT_SLOT_SIZE,
};
use contextra_types::ids::ErasureId;

/// SPEC: T-REPL-1, T-REPL-2, T-REPL-3 (D-115 replace_erasing Protokoll & Statusfolge)
#[test]
fn test_t_repl_1_2_3_replace_erasing_protocol() {
    let mut saga = EraseSaga::new(ErasureId::new([0x11; 16]));
    // R1: Requested
    assert_eq!(saga.state(), SagaState::Requested);
    // R2: PreGate run (new document durable in LSM) -> transition to Gated
    assert!(saga.transition_to(SagaState::Gated).is_ok());
    // R3: KeyDestroyed
    assert!(saga.transition_to(SagaState::KeyDestroyed).is_ok());
    // R4: Purging -> Verifying -> Completed
    assert!(saga.transition_to(SagaState::Purging).is_ok());
    assert!(saga.transition_to(SagaState::Verifying).is_ok());
    assert!(saga.transition_to(SagaState::Completed).is_ok());
    assert!(saga.is_terminal());
}

/// SPEC: T-SUP-1, T-SUP-2, T-SUP-3 (D-117 Version-Supersede Protokoll & State)
#[test]
fn test_t_sup_1_2_3_superseded_version_flow() {
    let mut saga = EraseSaga::new(ErasureId::new([0x22; 16]));
    assert_eq!(saga.state(), SagaState::Requested);
    // Gate altes Version-Präfix
    assert!(saga.transition_to(SagaState::Gated).is_ok());
    // D-117: DEK bleibt gleich, keine Schlüsselvernichtung (K not_reached)
    // Direkt zu Purging
    assert!(saga.transition_to(SagaState::Purging).is_ok());
    assert!(saga.transition_to(SagaState::Verifying).is_ok());
    assert!(saga.transition_to(SagaState::Completed).is_ok());
}

/// SPEC: T-RESTR-2, T-RESTR-3, T-RESTR-4 (D-116 Zugriff auf Restricted-Daten & Audit)
#[test]
fn test_t_restr_2_3_4_restricted_access_evaluation() {
    // 1. Zweck LegalClaims erlaubt Zugriff
    let allowed_claims = evaluate_restricted_access("LegalClaims", true, 50).unwrap();
    assert!(allowed_claims);

    // 2. Zweck Consent erlaubt Zugriff
    let allowed_consent = evaluate_restricted_access("DataSubjectConsent", true, 50).unwrap();
    assert!(allowed_consent);

    // 3. T-RESTR-3: Erased Ziel schlägt fehl, selbst wenn Zweck vorliegt
    let denied_erased = evaluate_restricted_access("LegalClaims", false, 50).unwrap();
    assert!(!denied_erased);

    // 4. T-RESTR-4: Stapelgrenze 1000 überschritten
    let batch_overflow = evaluate_restricted_access("LegalClaims", true, 1001);
    assert!(
        batch_overflow.is_err(),
        "Batch > 1000 muss verweigert werden"
    );
}

/// SPEC: T-ERID-1, T-ERID-2, T-ERID-3 (D-119 Idempotenz und Open-Lookup Key)
#[test]
fn test_t_erid_1_2_3_idempotency_open_lookup() {
    let k_idx = [0x55u8; 32];
    let target_a = b"tenant_1/coll_1/doc_1";
    let mode_erase = 0x01u8;
    let mode_restrict = 0x02u8;

    let key1 = derive_open_key(&k_idx, target_a, mode_erase);
    let key2 = derive_open_key(&k_idx, target_a, mode_erase);
    assert_eq!(
        key1, key2,
        "Gleiches Ziel & Modus muss denselben Open-Key liefern"
    );

    let key_other_mode = derive_open_key(&k_idx, target_a, mode_restrict);
    assert_ne!(
        key1, key_other_mode,
        "Anderer Modus muss anderen Open-Key liefern"
    );
}

/// SPEC: T-EPOCH-1 (D-125 Epochenschlüssel-Rotation Slot E)
#[test]
fn test_t_epoch_1_rotation_slot_e() {
    let vault_id = [0x7au8; 16];
    let mac_key = [0x9bu8; 32];
    let epoch_tag = [0x0eu8; 16];
    let epoch_payload = [0x42u8; 32];

    // Slot E anlegen
    let active_slot = encode_slot(
        &vault_id,
        SlotType::Epoch,
        STATUS_ACTIVE,
        1,
        &epoch_tag,
        &epoch_payload,
        &mac_key,
    );
    assert_eq!(active_slot[off::SLOT_TYPE], SlotType::Epoch as u8);
    assert_eq!(active_slot[off::STATUS], STATUS_ACTIVE);

    // Epochenrotation: Slot E wird zum Grabstein
    let tombstone_slot = encode_slot(
        &vault_id,
        SlotType::Epoch,
        STATUS_TOMBSTONE,
        2,
        &epoch_tag,
        &[0u8; 32],
        &mac_key,
    );
    assert_eq!(tombstone_slot[off::STATUS], STATUS_TOMBSTONE);
    assert_eq!(tombstone_slot.len(), VAULT_SLOT_SIZE);
}
