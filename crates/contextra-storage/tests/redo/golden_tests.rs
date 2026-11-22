//! # Golden A.5 Tests (byte-genau)
//!
//! SPEC Anhang A.5: Redo-Frame und Seal-Frame im Plain-Modus.

use contextra_storage::redo::frame::{Durability, RedoFrame};
use contextra_storage::redo::op::RedoOp;
use contextra_storage::redo::replay_sm::{ReplayEffect, ReplayEvent, ReplayStateMachine};
use contextra_storage::redo::header::RedoHeader;

#[test]
fn test_golden_a5_frame_plain_byte_exact() {
    let chain_seed: u32 = 0xA5A5A5A5;

    // Op (8 B): kind(1) space(1) opflags(0) klen(1) key("k") vlen(1) value("v")
    let op = RedoOp::new_put(1, None, b"k".to_vec(), b"v".to_vec());

    // Frame (29 B): len = 21, stamp = 1, op_count = 1, flags = 0 (Always)
    let (frame, next_chk) = RedoFrame::encode_plain(1, Durability::Always, &[op], chain_seed)
        .expect("encode succeeds");

    assert_eq!(next_chk, 0x6F638DC4);
    assert_eq!(frame.len, 21);
    assert_eq!(frame.stamp, 1);
    assert_eq!(frame.op_count, 1);
    assert_eq!(frame.flags, 0);

    let bytes = frame.to_bytes();
    let hex_frame: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    assert_eq!(
        hex_frame,
        "150000000100000000000000010000000001010000016b0176c48d636f",
        "Golden A.5 Frame muss byte-genau übereinstimmen"
    );

    // Seal-Frame (33 B) nach diesem Frame:
    // frame_count = 1, final_chain = 0x6F638DC4, stamp = 1
    let (seal, seal_chk) = RedoFrame::new_seal_plain(1, 1, 0x6F638DC4, next_chk);
    assert_eq!(seal_chk, 0xDE8370DB);
    assert_eq!(seal.len, 25);
    assert_eq!(seal.stamp, 1);
    assert_eq!(seal.op_count, 0xFFFF_FFFF);
    assert_eq!(seal.flags, 0x80);

    let seal_bytes = seal.to_bytes();
    let hex_seal: String = seal_bytes.iter().map(|b| format!("{:02x}", b)).collect();
    assert_eq!(
        hex_seal,
        "190000000100000000000000ffffffff800100000000000000c48d636fdb7083de",
        "Golden A.5 Seal muss byte-genau übereinstimmen"
    );
}

#[test]
fn test_golden_a5_replay_roundtrip() {
    let chain_seed: u32 = 0xA5A5A5A5;
    let header = RedoHeader::new(0x12, 1, chain_seed, 64 * 1024 * 1024);
    let mut segment_data = Vec::new();
    segment_data.extend_from_slice(&header.to_bytes());

    let op = RedoOp::new_put(1, None, b"k".to_vec(), b"v".to_vec());
    let (frame, next_chk) = RedoFrame::encode_plain(1, Durability::Always, &[op], chain_seed).unwrap();
    segment_data.extend_from_slice(&frame.to_bytes());

    let (seal, _) = RedoFrame::new_seal_plain(1, 1, next_chk, next_chk);
    segment_data.extend_from_slice(&seal.to_bytes());

    let mut rsm = ReplayStateMachine::new();
    let effects = rsm.step(ReplayEvent::Chunk(&segment_data));

    assert_eq!(effects.len(), 3);
    match &effects[0] {
        ReplayEffect::HeaderParsed(h) => {
            assert_eq!(h.segment_id, 0x12);
            assert_eq!(h.chain_seed, chain_seed);
        }
        _ => panic!("Erwartet HeaderParsed"),
    }
    match &effects[1] {
        ReplayEffect::ValidFrame { frame: f, ops, .. } => {
            assert_eq!(f.stamp, 1);
            assert_eq!(ops.len(), 1);
            assert_eq!(ops[0].key, b"k");
            assert_eq!(ops[0].value, b"v");
        }
        _ => panic!("Erwartet ValidFrame"),
    }
    match &effects[2] {
        ReplayEffect::FoundSeal { frame_count, final_chain, stamp, .. } => {
            assert_eq!(*frame_count, 1);
            assert_eq!(*final_chain, 0x6F638DC4);
            assert_eq!(*stamp, 1);
        }
        _ => panic!("Erwartet FoundSeal"),
    }
}
