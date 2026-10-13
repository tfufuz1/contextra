#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_engine::deletion::environment::{
    encode_observation_tlv, observe_environment, EnvFsKind, EnvObservation, EnvOsKind, EnvTri,
};
use tempfile::TempDir;

#[test]
fn test_encode_observation_tlv_frozen_golden_bytes() {
    let obs = EnvObservation {
        os: EnvOsKind::Linux,
        fs_kind: EnvFsKind::Ext4,
        cow_fs: EnvTri::No,
        discard_enabled: EnvTri::Yes,
        block_encrypted: EnvTri::No,
        rotational: EnvTri::No,
        swap_active: EnvTri::No,
        core_dumps_possible: EnvTri::No,
    };

    let bytes = encode_observation_tlv(&obs, false, false);

    // Exact length: 10 fields * 7 bytes = 70 bytes
    assert_eq!(bytes.len(), 70, "TLV encoding must be exactly 70 bytes");

    // Frozen golden byte expectation
    let expected_golden: [u8; 70] = [
        // Tag 0x0001 (os): Linux (1)
        0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x01, // Tag 0x0002 (fs_kind): Ext4 (1)
        0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x01, // Tag 0x0003 (cow_fs): No (2)
        0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x02, // Tag 0x0004 (discard_enabled): Yes (1)
        0x00, 0x04, 0x00, 0x00, 0x00, 0x01, 0x01, // Tag 0x0005 (block_encrypted): No (2)
        0x00, 0x05, 0x00, 0x00, 0x00, 0x01, 0x02, // Tag 0x0006 (rotational): No (2)
        0x00, 0x06, 0x00, 0x00, 0x00, 0x01, 0x02, // Tag 0x0007 (swap_active): No (2)
        0x00, 0x07, 0x00, 0x00, 0x00, 0x01, 0x02, // Tag 0x0008 (core_dumps_possible): No (2)
        0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x02, // Tag 0x0009 (secure_buffers): false (0)
        0x00, 0x09, 0x00, 0x00, 0x00, 0x01, 0x00, // Tag 0x000A (secure_delete_applied): false (0)
        0x00, 0x0A, 0x00, 0x00, 0x00, 0x01, 0x00,
    ];

    assert_eq!(
        bytes, expected_golden,
        "Encoded TLV bytes must match frozen golden fixture exactly"
    );
}

#[test]
fn test_encode_observation_tlv_value_and_security_flag_changes() {
    let obs1 = EnvObservation {
        os: EnvOsKind::MacOs,
        fs_kind: EnvFsKind::Apfs,
        cow_fs: EnvTri::Yes,
        discard_enabled: EnvTri::Unknown,
        block_encrypted: EnvTri::Yes,
        rotational: EnvTri::No,
        swap_active: EnvTri::Yes,
        core_dumps_possible: EnvTri::Unknown,
    };

    let bytes1 = encode_observation_tlv(&obs1, true, true);
    let bytes2 = encode_observation_tlv(&obs1, false, true);

    assert_ne!(
        bytes1, bytes2,
        "Changing secure_buffers flag must yield different TLV bytes"
    );

    // Verify tag 0x0009 and 0x000A values in bytes1 (both set to 1)
    let tag9_val = bytes1[8 * 7 + 6];
    let tag10_val = bytes1[9 * 7 + 6];

    assert_eq!(tag9_val, 1, "Tag 0x0009 must be 1 for secure_buffers=true");
    assert_eq!(
        tag10_val, 1,
        "Tag 0x000A must be 1 for secure_delete_applied=true"
    );
}

#[test]
fn test_observe_environment_temp_dir_never_panics() {
    let tmp = TempDir::new().expect("temp dir creation");
    let bytes = observe_environment(tmp.path(), false, false);

    assert_eq!(
        bytes.len(),
        70,
        "observe_environment on temp dir must produce 70 bytes TLV"
    );

    // Verify tag headers (2 bytes tag, 4 bytes len = 0x00000001)
    for i in 0..10 {
        let offset = i * 7;
        let tag = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
        let len = u32::from_be_bytes([
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
        ]);
        let val = bytes[offset + 6];

        assert_eq!(
            u16::try_from(i + 1).unwrap(),
            tag,
            "Tag at index {i} must be {}",
            i + 1
        );
        assert_eq!(len, 1, "Length for u8 TLV field at index {i} must be 1");
        assert!(val <= 8, "u8 byte code at index {i} must be valid <= 8");
    }
}
