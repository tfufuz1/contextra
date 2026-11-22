//! # Authenticated und Encrypted Frame Tests
//!
//! SPEC §9.4 und §15.2: Mac16, ChaCha20Poly1305 und Domänenversiegelung (Nonce 0x03).

use contextra_crypto_ref::{Blake3Mac, ChaChaCipher};
use contextra_storage::redo::frame::{Durability, RedoFrame};
use contextra_storage::redo::op::{RedoOp, OPFLAG_DOMAIN, OPFLAG_SEALED};

#[test]
fn test_authenticated_frame() {
    let mac = Blake3Mac;
    let mac_key = [0x5Au8; 32];
    let segment_id = 0x42u64;
    let frame_index = 0u32;

    let op = RedoOp::new_put(2, None, b"auth_key".to_vec(), b"auth_value".to_vec());
    let frame = RedoFrame::encode_authenticated(
        10,
        Durability::Always,
        &[op],
        segment_id,
        frame_index,
        &mac_key,
        &mac,
    )
    .expect("encode authenticated succeeds");

    assert_eq!(frame.chk.len(), 16);
    assert_eq!(frame.stamp, 10);
    assert_eq!(frame.op_count, 1);
}

#[test]
fn test_encrypted_frame() {
    let cipher = ChaChaCipher;
    let redo_key = [0x7Eu8; 32];
    let segment_id = 0x100u64;
    let frame_index = 1u32;

    let op = RedoOp::new_put(3, None, b"secret_key".to_vec(), b"top_secret_value".to_vec());
    let frame = RedoFrame::encode_encrypted(
        15,
        Durability::Always,
        &[op],
        segment_id,
        frame_index,
        &redo_key,
        &cipher,
    )
    .expect("encode encrypted succeeds");

    // Im verschlüsselten Modus gibt es kein separates chk, payload ist Ciphertext (+16 Byte Tag)
    assert!(frame.chk.is_empty());
    assert_ne!(&frame.payload, b"top_secret_value");
}

#[test]
fn test_domain_sealed_op_body() {
    let cipher = ChaChaCipher;
    let domain_key = [0x33u8; 32];
    let space = 1u16;
    let domain = 42u32;
    let key = b"patient-doc";
    let header = b"HDR:";
    let body = b"confidential health data";

    let segment_id = 0x12u64;
    let frame_index = 3u32;
    let op_index = 1u32;

    let sealed_op = RedoOp::new_sealed_put(
        space,
        domain,
        key,
        header,
        body,
        segment_id,
        frame_index,
        op_index,
        &domain_key,
        &cipher,
    )
    .expect("seal succeeds");

    assert_eq!(sealed_op.opflags, OPFLAG_DOMAIN | OPFLAG_SEALED);
    assert_eq!(sealed_op.domain, Some(domain));
    assert!(sealed_op.value.starts_with(header));
    assert_ne!(&sealed_op.value[header.len()..], body);

    // Unseal mit korrektem Schlüssel
    let unsealed = sealed_op
        .unseal_body(header.len(), segment_id, frame_index, op_index, &domain_key, &cipher)
        .expect("unseal succeeds");
    assert_eq!(unsealed, body);

    // Unseal mit falschem Schlüssel muss fehlschlagen
    let wrong_key = [0x99u8; 32];
    let err = sealed_op.unseal_body(header.len(), segment_id, frame_index, op_index, &wrong_key, &cipher);
    assert!(err.is_err());
}
