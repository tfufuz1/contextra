#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::collapsible_if, clippy::unnecessary_map_or)]

//! # Referenzvektoren (Goldens aus Anhang A)
//!
//! Verifiziert die Implementierung byte-genau gegen Anhang A der SPEC.md.

use contextra_storage::codec::crc32c::crc32c;
use contextra_storage::codec::tag::{InternalKey, Tag, TagKind};
use contextra_storage::codec::varint::{decode_u64, encode_u64};
use contextra_storage::codec::xxh64::xxh64;
use contextra_storage::keys::nonce::*;

#[test]
fn test_a1_primitives() {
    // CRC32C
    assert_eq!(crc32c(0, b"123456789"), 0xE3069283);
    assert_eq!(crc32c(0, b""), 0x00000000);
    assert_eq!(crc32c(0, b"a"), 0xC1D04330);
    assert_eq!(crc32c(crc32c(0, b"1234"), b"56789"), 0xE3069283);

    // XXH64
    assert_eq!(xxh64(b"", 0), 0xEF46DB3751D8E999);
    assert_eq!(xxh64(b"a", 0), 0xD24EC4F1A98C6E5B);
    assert_eq!(xxh64(b"abc", 0), 0x44BC2CF5AD770999);
}

#[test]
fn test_a2_varint() {
    let vectors: &[(u64, &[u8])] = &[
        (0, &[0x00]),
        (1, &[0x01]),
        (127, &[0x7f]),
        (128, &[0x80, 0x01]),
        (300, &[0xac, 0x02]),
        (16384, &[0x80, 0x80, 0x01]),
        (1u64 << 32, &[0x80, 0x80, 0x80, 0x80, 0x10]),
        (u64::MAX, &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01]),
    ];

    for &(val, expected) in vectors {
        let mut buf = [0u8; 10];
        let len = encode_u64(val, &mut buf);
        assert_eq!(&buf[..len], expected);

        let (decoded, consumed) = decode_u64(expected).unwrap();
        assert_eq!(decoded, val);
        assert_eq!(consumed, expected.len());
    }
}

#[test]
fn test_a3_tag() {
    let t1 = Tag::new(1, TagKind::Put);
    assert_eq!(t1.to_le_bytes(), [0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    let t2 = Tag::new(7, TagKind::Delete);
    assert_eq!(t2.to_le_bytes(), [0x02, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    let max_stamp = (1u64 << 56) - 1;
    let t3 = Tag::new(max_stamp, TagKind::PutRef);
    assert_eq!(t3.to_le_bytes(), [0x05, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);

    let k_del = InternalKey::new(b"k".to_vec(), t2);
    let k_put = InternalKey::new(b"k".to_vec(), t1);
    assert!(k_del < k_put);
}

#[test]
fn test_a4_format_plain() {
    // Eingabe: version = 1, required = 0, optional = 0, integrity_mode = 0,
    // instance_id = 00 01 … 0f, created_ms = 1_700_000_000_000.
    let mut header = [0u8; 47];
    header[0..4].copy_from_slice(b"CTXS");
    header[4..6].copy_from_slice(&1u16.to_le_bytes()); // version = 1
    header[6..14].copy_from_slice(&0u64.to_le_bytes()); // required = 0
    header[14..22].copy_from_slice(&0u64.to_le_bytes()); // optional = 0
    header[22] = 0; // integrity_mode = 0 (Plain)
    for i in 0..16 {
        header[23 + i] = i as u8; // instance_id
    }
    header[39..47].copy_from_slice(&1_700_000_000_000u64.to_le_bytes()); // created_ms

    let chk = crc32c(0, &header);
    assert_eq!(chk, 0x86B86356);

    let mut full = [0u8; 51];
    full[..47].copy_from_slice(&header);
    full[47..51].copy_from_slice(&chk.to_le_bytes());

    let hex_full: String = full.iter().map(|b| format!("{:02x}", b)).collect();
    assert_eq!(
        hex_full,
        "4354585301000000000000000000000000000000000000000102030405060708090a0b0c0d0e0f0068e5cf8b0100005663b886"
    );
}

#[test]
fn test_a5_redo_frame_plain() {
    let chain_seed: u32 = 0xA5A5A5A5;

    // Op (8 B): kind(1) space(1) opflags(0) klen(1) key("k") vlen(1) value("v")
    let op = [0x01, 0x01, 0x00, 0x00, 0x01, 0x6b, 0x01, 0x76];
    assert_eq!(op.len(), 8);

    // Frame payload: len(13 + 8 = 21 = 0x15), stamp(1), op_count(1), flags(0)
    let mut frame_data = Vec::new();
    frame_data.extend_from_slice(&21u32.to_le_bytes());
    frame_data.extend_from_slice(&1u64.to_le_bytes());
    frame_data.extend_from_slice(&1u32.to_le_bytes());
    frame_data.push(0u8);
    frame_data.extend_from_slice(&op);

    let chk = crc32c(chain_seed, &frame_data);
    assert_eq!(chk, 0x6F638DC4);

    let mut full_frame = frame_data;
    full_frame.extend_from_slice(&chk.to_le_bytes());

    let hex_frame: String = full_frame.iter().map(|b| format!("{:02x}", b)).collect();
    assert_eq!(
        hex_frame,
        "150000000100000000000000010000000001010000016b0176c48d636f"
    );

    // Seal-Frame:
    // op_count = 0xFFFFFFFF, flags = 0x80 (SEAL), stamp = 1
    // payload = frame_count(1u64) || final_chain(0x6F638DC4) (12 B)
    // len = 13 + 12 = 25 (0x19)
    let mut seal_data = Vec::new();
    seal_data.extend_from_slice(&25u32.to_le_bytes());
    seal_data.extend_from_slice(&1u64.to_le_bytes());
    seal_data.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());
    seal_data.push(0x80u8);
    seal_data.extend_from_slice(&1u64.to_le_bytes());
    seal_data.extend_from_slice(&0x6F638DC4u32.to_le_bytes());

    let seal_chk = crc32c(0x6F638DC4, &seal_data);
    assert_eq!(seal_chk, 0xDE8370DB);

    let mut full_seal = seal_data;
    full_seal.extend_from_slice(&seal_chk.to_le_bytes());

    let hex_seal: String = full_seal.iter().map(|b| format!("{:02x}", b)).collect();
    assert_eq!(
        hex_seal,
        "190000000100000000000000ffffffff800100000000000000c48d636fdb7083de"
    );
}

#[test]
fn test_a6_blob_ref() {
    // BlobRef (28 B): blob_no = 0x71, offset = 4096, len = 1000, crc32c = 0xDEADBEEF, domain = 42
    let mut buf = [0u8; 28];
    buf[0..8].copy_from_slice(&0x71u64.to_le_bytes());
    buf[8..16].copy_from_slice(&4096u64.to_le_bytes());
    buf[16..20].copy_from_slice(&1000u32.to_le_bytes());
    buf[20..24].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
    buf[24..28].copy_from_slice(&42u32.to_le_bytes());

    assert_eq!(
        buf,
        [
            0x71, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xe8, 0x03, 0x00, 0x00,
            0xef, 0xbe, 0xad, 0xde,
            0x2a, 0x00, 0x00, 0x00,
        ]
    );
}

#[test]
fn test_a6_nonces_all() {
    assert_eq!(
        run_block_nonce(4096),
        [0x01, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        redo_frame_nonce(3),
        [0x02, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        redo_op_body_nonce(0x12, 3, 1),
        [0x03, 0x12, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00]
    );
    assert_eq!(
        blob_record_nonce(4096),
        [0x04, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
    assert_eq!(
        blob_body_nonce(0x71, 4096),
        [0x05, 0x71, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00]
    );
    assert_eq!(
        catalog_record_nonce(2),
        [0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    );
}
