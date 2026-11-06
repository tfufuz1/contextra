//! Tests für FileVault Core (T-VAULT-1, T-VAULT-3, T-VAULT-5).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use contextra_erasure::vault::{
    FileVault, SlotType, VaultShard, VAULT_NUM_SHARDS, VAULT_SLOT_BASE, VAULT_SLOT_SIZE,
};

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

    let destroyed = vault
        .destroy_slot(SlotType::Document, tag)
        .expect("destroy failed");
    assert_eq!(
        destroyed.verified_via,
        contextra_erasure::vault::VerifiedVia::Direct
    );

    let slot_after = vault
        .get_slot(SlotType::Document, tag)
        .expect("get_slot failed");
    assert!(slot_after.is_none());
}

#[test]
fn t_vault_3_constants_and_layout() {
    // Nach SPEC D-111, F.3: 64 Byte pro Slot, Basis-Offset 128 (Kopf 64 B + Metablock 64 B)
    assert_eq!(VAULT_SLOT_SIZE, 64);
    assert_eq!(VAULT_SLOT_BASE, 128);
    assert_eq!(VAULT_NUM_SHARDS, 16);
}

#[test]
fn t_vault_5_probing_and_load_factor() {
    // Einzel-Shard-Test für Lastfaktor-Grenzen nach D-111 Abs. 4:
    // λ = (live + tombstones) / capacity
    // λ > 0.90 liefert Limit::Capacity
    let vault_id = [1u8; 16];
    let mac_key = [2u8; 32];
    let mut shard = VaultShard::new(16, vault_id, mac_key);

    assert_eq!(shard.capacity(), 16);

    // 14 Slots einfügen: 14/16 = 0.875 <= 0.90
    for i in 0..14 {
        let mut tag = [0u8; 16];
        tag[0] = i as u8;
        tag[1] = 0xAA;
        let sd = contextra_erasure::vault::VaultSlotData {
            slot_type: SlotType::Document,
            tag,
            payload: [i as u8; 32],
            gen: 0,
        };
        shard.put(sd).expect("put within load factor");
    }

    // 15 Slots: 15/16 = 0.9375 > 0.90 -> Limit::Capacity
    let mut tag15 = [0u8; 16];
    tag15[0] = 15;
    tag15[1] = 0xAA;
    let sd15 = contextra_erasure::vault::VaultSlotData {
        slot_type: SlotType::Document,
        tag: tag15,
        payload: [15u8; 32],
        gen: 0,
    };
    let res = shard.put(sd15);
    assert!(res.is_err(), "λ > 0.90 muss fehlschlagen");
}

#[test]
fn t_vault_1_and_t_crash_5_slot_tombstone_verification() {
    let mut vault = FileVault::new_in_memory(64);
    let tag = [0x55u8; 16];
    let payload = [0x11u8; 32];

    vault
        .put_slot(SlotType::Document, tag, payload)
        .expect("put slot");
    assert_eq!(vault.live_count(), 1);

    // Vernichten und sicherstellen dass Grabstein verifiziert wird
    vault
        .destroy_slot(SlotType::Document, tag)
        .expect("destroy slot");
    assert_eq!(vault.live_count(), 0);
    assert_eq!(vault.tombstone_count(), 1);

    // Zweites Vernichten schlägt fehl (bereits Grabstein)
    let err = vault.destroy_slot(SlotType::Document, tag);
    assert!(err.is_err());
}
