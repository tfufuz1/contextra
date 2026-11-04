//! Tests für FileVault Core (T-VAULT-1, T-VAULT-3, T-VAULT-5).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use contextra_erasure::vault::{FileVault, SlotType, VAULT_SLOT_BASE, VAULT_SLOT_SIZE};

#[test]
fn t_vault_1_slot_destruction_and_verification() {
    let mut vault = FileVault::new_in_memory(64);
    let tag = [0x42u8; 16];
    let payload = [0x99u8; 32];

    vault
        .put_slot(SlotType::Document, tag, payload)
        .expect("put_slot failed");

    let slot = vault
        .get_slot(SlotType::Document, tag)
        .expect("get_slot failed");
    assert!(slot.is_some());
    assert_eq!(slot.unwrap().payload, payload);

    vault
        .destroy_slot(SlotType::Document, tag)
        .expect("destroy failed");

    let slot_after = vault
        .get_slot(SlotType::Document, tag)
        .expect("get_slot failed");
    assert!(slot_after.is_none());
}

#[test]
fn t_vault_3_constants_and_layout() {
    assert_eq!(VAULT_SLOT_SIZE, 80);
    assert_eq!(VAULT_SLOT_BASE, 128);
}

#[test]
fn t_vault_5_probing_and_load_factor() {
    let mut vault = FileVault::new_in_memory(16);
    for i in 0..11 {
        let mut tag = [0u8; 16];
        tag[0] = i as u8;
        vault
            .put_slot(SlotType::Document, tag, [i as u8; 32])
            .expect("put within load factor");
    }

    let mut tag = [0u8; 16];
    tag[0] = 11;
    let res = vault.put_slot(SlotType::Document, tag, [11u8; 32]);
    assert!(res.is_err());
}

#[test]
fn t_vault_1_and_t_crash_5_slot_tombstone_verification() {
    let mut vault = FileVault::new_in_memory(32);
    let tag = [0x55u8; 16];
    let payload = [0x11u8; 32];

    vault
        .put_slot(SlotType::Document, tag, payload)
        .expect("put slot");
    assert_eq!(vault.live_count(), 1);

    // Vernichten und sicherstellen dass Nullung verifiziert wird
    vault
        .destroy_slot(SlotType::Document, tag)
        .expect("destroy slot");
    assert_eq!(vault.live_count(), 0);

    // Zweites Vernichten schlägt fehl (bereits Grabstein)
    let err = vault.destroy_slot(SlotType::Document, tag);
    assert!(err.is_err());
}
