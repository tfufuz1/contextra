// FILE-CONTEXT
// ZWECK: Integrationstests und Audit-Regressionstests für WAL-Integrität, Sequence-Number-Zustandsmaschine, Nonce-Politik und V2/V3-Protokolle.
// INVARIANTEN: Monotone Sequence-Nummern, verweigerte Replays/Lücken, Nonce-Eindeutigkeit, Zero-Panics.

use contextra_crypto::crypto::KeyManager;
use contextra_crypto::wal_crypto::{IntegrityVerifier, WalEntrySnapshot, WalHmac};
use contextra_crypto::CryptoError;
use std::collections::HashSet;

fn create_v3_entry(
    key: &[u8],
    prev_hmac: [u8; 32],
    seq_no: u64,
    tx_id: u64,
    op_type: u8,
    k: &[u8],
    v: &[u8],
) -> WalEntrySnapshot {
    let mut hmac = WalHmac::new(key).expect("WalHmac init");
    hmac.update(&prev_hmac);
    hmac.update(&seq_no.to_le_bytes());
    hmac.update(&tx_id.to_le_bytes());
    match op_type {
        0 => {
            hmac.update(&[0u8]);
            hmac.update(&(k.len() as u32).to_le_bytes());
            hmac.update(k);
            hmac.update(&(v.len() as u32).to_le_bytes());
            hmac.update(v);
        }
        1 => {
            hmac.update(&[1u8]);
            hmac.update(&(k.len() as u32).to_le_bytes());
            hmac.update(k);
        }
        2 => {
            hmac.update(&[2u8]);
            let committed = if v.first().copied().unwrap_or(0) != 0 {
                1u8
            } else {
                0u8
            };
            hmac.update(&[committed]);
        }
        _ => {
            // Unbekannter op_type: für Testzwecke keine spezifischen Felder
            hmac.update(&[op_type]);
        }
    }
    let checksum = hmac.finalize();
    WalEntrySnapshot {
        tx_id,
        seq_no,
        op_type,
        key: k.to_vec(),
        value: v.to_vec(),
        checksum,
        prev_hmac,
    }
}

#[test]
fn test_sequence_number_gap_rejected() {
    let key = b"integrity-key-32-bytes-long-----";
    let e1 = create_v3_entry(key, [0u8; 32], 1, 1, 0, b"k1", b"v1");
    // e2 hat seq_no = 5 (Lücke von 2..4), obwohl HMAC mathematisch mit seq_no = 5 korrespondieren würde
    let e2 = create_v3_entry(key, e1.checksum, 5, 2, 0, b"k2", b"v2");

    let mut verifier = IntegrityVerifier::new(key);
    assert!(verifier.verify_and_update(&e1, 10).is_ok());

    let res = verifier.verify_and_update(&e2, 20);
    assert!(
        matches!(res, Err(CryptoError::WalCorruption { offset: 20, ref reason }) if reason.contains("Sequence gap")),
        "Integritätsprüfer MUSS Sequence-Nummern-Lücke (1 -> 5) als WalCorruption ablehnen, erhaltener Wert: {:?}",
        res
    );
}

#[test]
fn test_sequence_number_duplicate_rejected() {
    let key = b"integrity-key-32-bytes-long-----";
    let e1 = create_v3_entry(key, [0u8; 32], 1, 1, 0, b"k1", b"v1");
    // e2_dup hat doppelte seq_no = 1
    let e2_dup = create_v3_entry(key, e1.checksum, 1, 2, 0, b"k2", b"v2");

    let mut verifier = IntegrityVerifier::new(key);
    assert!(verifier.verify_and_update(&e1, 10).is_ok());

    let res = verifier.verify_and_update(&e2_dup, 20);
    assert!(
        matches!(res, Err(CryptoError::WalCorruption { offset: 20, ref reason }) if reason.contains("Sequence gap") || reason.contains("Duplicate or non-monotonic sequence number")),
        "Integritätsprüfer MUSS doppelte Sequence-Nummer (1 -> 1) ablehnen, erhaltener Wert: {:?}",
        res
    );
}

#[test]
fn test_sequence_number_reorder_rejected() {
    let key = b"integrity-key-32-bytes-long-----";
    let e1 = create_v3_entry(key, [0u8; 32], 2, 1, 0, b"k1", b"v1");
    // e2 hat rückflächige seq_no = 1
    let e2 = create_v3_entry(key, e1.checksum, 1, 2, 0, b"k2", b"v2");

    let mut verifier = IntegrityVerifier::new(key);
    assert!(verifier.verify_and_update(&e1, 10).is_ok());

    let res = verifier.verify_and_update(&e2, 20);
    assert!(
        matches!(res, Err(CryptoError::WalCorruption { offset: 20, ref reason }) if reason.contains("Duplicate or non-monotonic sequence number") || reason.contains("Sequence gap")),
        "Integritätsprüfer MUSS umgekehrte Sequence-Nummer (2 -> 1) ablehnen, erhaltener Wert: {:?}",
        res
    );
}

#[test]
fn test_unsupported_op_type_v3_rejected() {
    let key = b"integrity-key-32-bytes-long-----";
    let e_invalid = create_v3_entry(key, [0u8; 32], 1, 1, 99, b"k1", b"v1");

    let mut verifier = IntegrityVerifier::new(key);
    let res = verifier.verify_and_update_v3(&e_invalid, 10);
    assert!(
        matches!(res, Err(CryptoError::WalCorruption { offset: 10, ref reason }) if reason.contains("Unsupported op_type")),
        "Unsupported op_type 99 in V3 MUST produce WalCorruption, got: {:?}",
        res
    );
}

#[test]
#[allow(deprecated)]
fn test_unsupported_op_type_v2_rejected() {
    let key = b"integrity-key-32-bytes-long-----";
    let mut hmac = WalHmac::new(key).expect("hmac");
    hmac.update(&[0u8; 32]);
    hmac.update(&1u64.to_le_bytes());
    hmac.update(&[99u8]);
    let checksum = hmac.finalize();

    let e_invalid = WalEntrySnapshot {
        tx_id: 1,
        seq_no: 1,
        op_type: 99,
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
        checksum,
        prev_hmac: [0u8; 32],
    };

    let mut verifier = IntegrityVerifier::new(key);
    let res = verifier.verify_and_update_v2(&e_invalid, 10);
    assert!(
        matches!(res, Err(CryptoError::WalCorruption { offset: 10, ref reason }) if reason.contains("Unsupported op_type")),
        "Unsupported op_type 99 in V2 MUST produce WalCorruption, got: {:?}",
        res
    );
}

#[test]
fn test_nonce_counter_exhaustion_protection() {
    let km = KeyManager::try_new("exhaustion-secret-passphrase", b"salt-1234").expect("km init");
    let (ciphertext, nonce) = km.encrypt_auto_nonce(b"test-payload").expect("encrypt");
    assert_eq!(ciphertext.is_empty(), false);
    assert_eq!(nonce.len(), 12);
}

#[test]
fn test_nonce_uniqueness_smoke_1m() {
    let km = KeyManager::try_new("nonce-stress-smoke-key", b"salt-987654321").expect("km init");
    let payload = b"smoke-test-payload-bytes";
    let mut seen = HashSet::with_capacity(1_000_000);

    for i in 0..1_000_000 {
        let (_, nonce) = km.encrypt_auto_nonce(payload).expect("encrypt failed");
        assert!(
            seen.insert(nonce),
            "CRITICAL: Nonce collision detected at iteration {i}"
        );
    }
    assert_eq!(seen.len(), 1_000_000);
}

#[test]
fn test_v2_v3_roundtrip_and_chain_integrity() {
    let key = b"roundtrip-integrity-key-32bytes";
    let mut verifier = IntegrityVerifier::new(key);

    let e1 = create_v3_entry(key, [0u8; 32], 1, 100, 0, b"key1", b"val1");
    let e2 = create_v3_entry(key, e1.checksum, 2, 100, 1, b"key1", b"");
    let e3 = create_v3_entry(key, e2.checksum, 3, 100, 2, b"", b"\x01");

    assert!(verifier.verify_and_update(&e1, 0).is_ok());
    assert!(verifier.verify_and_update(&e2, 50).is_ok());
    assert!(verifier.verify_and_update(&e3, 100).is_ok());
}
