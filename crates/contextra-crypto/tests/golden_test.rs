//! Golden test for F.2 HEADER encoding and Keyslot Wrap.
//!
//! SPEC: T-HDR-1, T-FMT-1, T-KEYSLOT-1..4

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::needless_borrows_for_generic_args
)]

use contextra_crypto::*;
use std::fs;
use std::path::Path;

#[test]
fn test_t_hdr_1_f2_golden_file_roundtrip() {
    let golden_path = Path::new("tests/golden/header-v1.bin");
    let repo_golden_path = Path::new("../../tests/golden/header-v1.bin");

    // Deterministic MasterKey, KEK, and HdrKey
    let master = MasterKey::new([0x42; 32]);
    let hierarchy = KeyHierarchy::from_master(master);
    let kek = SecretKey::new([0x33; 32]);

    let store_id = [0x11; 16];
    let slot_salt = [0x55; 32];
    let nonce = [0x77; 12];

    let aad = SlotAad {
        store_id,
        slot_id: 1,
        anchor_id: 1,
        kdf_id: 1,
        m_kib: 131072,
        t: 3,
        p: 4,
        slot_salt,
    };

    let wrapped = wrap_master(&kek, &aad, nonce, hierarchy.master());
    let unwrapped = unwrap_master(&kek, &aad, nonce, &wrapped).expect("unwrap master");
    assert_eq!(unwrapped.as_bytes(), hierarchy.master().as_bytes());

    let mut slot1 = HeaderSlot::empty();
    slot1.slot_id = 1;
    slot1.anchor_id = 1;
    slot1.kdf_id = 1;
    slot1.status = 1; // active
    slot1.m_kib = 131072;
    slot1.t = 3;
    slot1.p = 4;
    slot1.slot_salt = slot_salt;
    slot1.nonce = nonce;
    slot1.wrapped_master = wrapped;

    let header = Header {
        format_version: HEADER_FORMAT_VERSION,
        kind: HEADER_KIND,
        alg_ids: HEADER_ALG_IDS,
        store_id,
        generation: 42,
        slot_count: 1,
        report_pub: [0x99; 32],
        slots: [
            slot1,
            HeaderSlot::empty(),
            HeaderSlot::empty(),
            HeaderSlot::empty(),
        ],
    };

    let encoded = header.encode(hierarchy.hdr());
    assert_eq!(encoded.len(), HEADER_SIZE);

    let active_path = if golden_path.exists() {
        golden_path
    } else if repo_golden_path.exists() {
        repo_golden_path
    } else {
        // Create both
        let _ = fs::write(golden_path, &encoded);
        let _ = fs::write(repo_golden_path, &encoded);
        golden_path
    };

    let golden_bytes = fs::read(active_path).expect("read golden file");
    assert_eq!(
        golden_bytes.as_slice(),
        &encoded[..],
        "Generated header bytes must match golden file exactly"
    );

    // Verify decoded header matches
    let decoded = Header::decode_checked(&encoded).expect("decode checked");
    assert_eq!(decoded.generation, 42);
    assert_eq!(decoded.slot_count, 1);
    assert_eq!(decoded.slots[0].slot_id, 1);
    assert!(decoded.verify_mac(hierarchy.hdr(), &encoded).is_ok());
}
