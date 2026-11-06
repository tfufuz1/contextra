//! Tests für Tresor-Slot-Layout nach SPEC D-111, F.3 und I-23.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_erasure::vault::slot::{
    compute_slot_mac, decode_slot, encode_slot_body, slot_probe_start, tombstone_image, SlotData,
    SlotType, STATUS_ACTIVE, STATUS_TOMBSTONE, VAULT_SLOT_SIZE,
};

/// SPEC: T-VAULT-1
#[test]
fn test_vault_slot_size_d111() {
    assert_eq!(VAULT_SLOT_SIZE, 64);
}

/// SPEC: T-VAULT-1
#[test]
fn test_slot_type_all_d125() {
    assert_eq!(SlotType::from_u8(0x01), Some(SlotType::Tenant));
    assert_eq!(SlotType::from_u8(0x02), Some(SlotType::Collection));
    assert_eq!(SlotType::from_u8(0x03), Some(SlotType::Subject));
    assert_eq!(SlotType::from_u8(0x04), Some(SlotType::Document));
    assert_eq!(SlotType::from_u8(0x05), Some(SlotType::File));
    assert_eq!(SlotType::from_u8(0x06), Some(SlotType::Epoch));
}

/// SPEC: T-VAULT-1
#[test]
fn test_slot_encode_decode_roundtrip_f3() {
    let vault_id = [0x77u8; 16];
    let mac_key = [0x55u8; 32];
    let tag = [0xaa; 16];
    let payload = [0xbb; 32];
    let gen = 0x1234_5678_9abc_u64;

    let sd = SlotData {
        slot_type: SlotType::Document,
        tag,
        payload,
        gen,
    };

    let mut raw = encode_slot_body(&sd);
    let mac = compute_slot_mac(&mac_key, &vault_id, &raw[..56]);
    raw[56..64].copy_from_slice(&mac);

    // Layout-Prüfung
    assert_eq!(raw[0], SlotType::Document as u8);
    assert_eq!(raw[1], STATUS_ACTIVE);
    // gen u48 big endian: 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc
    assert_eq!(&raw[2..8], &[0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc]);
    assert_eq!(&raw[8..24], &tag);
    assert_eq!(&raw[24..56], &payload);
    assert_eq!(&raw[56..64], &mac);

    let decoded = decode_slot(&raw, &vault_id, &mac_key).expect("decode slot");
    assert_eq!(decoded, Some(sd));
}

/// SPEC: T-VAULT-1
#[test]
fn test_tombstone_image_and_verification_d111() {
    let vault_id = [0x42u8; 16];
    let mac_key = [0x99u8; 32];

    let tombstone = tombstone_image(&vault_id, &mac_key);

    assert_eq!(tombstone.len(), 64);
    assert_eq!(tombstone[0], 0);
    assert_eq!(tombstone[1], STATUS_TOMBSTONE);
    assert!(tombstone[2..56].iter().all(|&b| b == 0));

    // Decodieren eines Grabsteins ergibt Ok(None)
    let decoded = decode_slot(&tombstone, &vault_id, &mac_key).expect("decode tombstone");
    assert_eq!(decoded, None);

    // Manipulierter Grabstein (z.B. falscher MAC) muss Corrupt liefern
    let mut corrupted = tombstone;
    corrupted[60] ^= 0xff;
    let err = decode_slot(&corrupted, &vault_id, &mac_key);
    assert!(err.is_err(), "manipulierter Grabstein muss Corrupt liefern");
}

/// SPEC: T-VAULT-1
#[test]
fn test_slot_probe_start_d111() {
    let tag = [0x11u8; 16];
    let (shard, idx) = slot_probe_start(SlotType::Document, &tag, 1024);
    assert!(shard < 16, "16 Shards nach D-111");
    assert!(idx < 1024);
}
