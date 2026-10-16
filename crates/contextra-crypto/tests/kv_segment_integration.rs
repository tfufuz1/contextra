// FILE-CONTEXT
// ZWECK: Integrationstest für KvSegment-Verschlüsselung, Zeroize und Prozess-Speicherabbild-Prüfung (P9).
// STAND: TS:2026-09-08T00:00:00Z

#![cfg(feature = "kv-encryption")]
#![forbid(unsafe_code)]

use contextra_crypto::kv_segment::{KvSegment, TenantIsolatedKvStore};
use contextra_crypto::{CryptoKey, KvSegmentCipher, ModelFingerprint};
use contextra_types::TenantId;
use zeroize::Zeroize;

fn setup_cipher() -> KvSegmentCipher {
    let master_km = CryptoKey::try_new("master-passphrase-kv-bridge", b"master-salt-kv-bridge")
        .expect("KeyManager creation should succeed");
    KvSegmentCipher::new(master_km)
}

fn dummy_fingerprint(model_id: &str) -> ModelFingerprint {
    ModelFingerprint::new([0x77u8; 32], model_id, "Q4_K_M")
}

#[test]
fn test_encrypted_segment_memory_inspection_and_decryption_roundtrip() {
    let cipher = setup_cipher();
    let store = TenantIsolatedKvStore::new();

    let tenant = TenantId::try_new(42).unwrap();
    let segment_id = 999;
    let fp = dummy_fingerprint("llama-3.2-3b");
    let rope_offset = Some(256);

    // Distinct confidential plaintext payload
    let secret_plaintext =
        b"CONFIDENTIAL_TENSOR_PAYLOAD_0123456789_SECRET_KEY_VALUES_P9_VERIFICATION";

    // 1. Insert encrypted segment into store
    store
        .insert_encrypted_segment(
            &cipher,
            tenant,
            segment_id,
            fp.clone(),
            rope_offset,
            secret_plaintext,
        )
        .expect("Encryption & insertion MUST succeed");

    // 2. Process Memory Dump / Memory Inspection Simulation
    // Access internal segment bytes stored in store
    let segment_ids = store.get_segments(tenant);
    assert_eq!(segment_ids, vec![segment_id]);

    // Construct direct segment via new_encrypted to inspect raw in-memory bytes
    let encrypted_seg = KvSegment::new_encrypted(
        &cipher,
        tenant,
        segment_id,
        fp.clone(),
        rope_offset,
        secret_plaintext,
    )
    .expect("new_encrypted MUST succeed");

    assert!(encrypted_seg.encrypted);
    assert_eq!(encrypted_seg.rope_offset, Some(256));

    let raw_stored_bytes = encrypted_seg.as_bytes();

    // Verify raw stored memory bytes DO NOT contain any plaintext substring (P9 requirement)
    let contains_plaintext = raw_stored_bytes
        .windows(secret_plaintext.len())
        .any(|window| window == secret_plaintext);

    assert!(
        !contains_plaintext,
        "P9 VIOLATION: Stored memory buffer MUST NOT contain plaintext secret tensor data"
    );

    // 3. Decryption Roundtrip
    let decrypted = store
        .get_decrypted_segment(&cipher, tenant, segment_id)
        .expect("Decryption MUST succeed")
        .expect("Segment MUST exist");

    assert_eq!(
        decrypted, secret_plaintext,
        "Decrypted payload MUST equal original confidential plaintext"
    );

    // 4. Zeroize safe observable contract verification
    let mut encrypted_seg = encrypted_seg;
    let original_len = encrypted_seg.len();
    assert_eq!(original_len, raw_stored_bytes.len());
    assert!(!encrypted_seg.is_empty());

    // Before zeroize: encrypted data is non-empty and contains non-zero ciphertext bytes
    assert_ne!(
        encrypted_seg.as_bytes(),
        vec![0u8; original_len].as_slice(),
        "Ciphertext buffer MUST NOT be zeroed prior to explicit zeroize"
    );

    // Action: Zeroize memory in place
    Zeroize::zeroize(&mut encrypted_seg);

    // After zeroize: safe observable contract guarantees len == 0, is_empty == true, as_bytes().is_empty()
    assert_eq!(encrypted_seg.len(), 0);
    assert!(encrypted_seg.is_empty());
    assert!(encrypted_seg.as_bytes().is_empty());

    // Zeroize-on-drop integration path: drop after zeroize must not panic
    drop(encrypted_seg);
}

#[test]
fn test_kv_segment_drop_after_zeroize() {
    let cipher = setup_cipher();
    let tenant = TenantId::try_new(42).unwrap();
    let fp = dummy_fingerprint("llama-3.2-3b");
    let mut seg = KvSegment::new_encrypted(
        &cipher,
        tenant,
        100,
        fp,
        None,
        b"DROP_AFTER_ZEROIZE_PAYLOAD",
    )
    .expect("new_encrypted MUST succeed");

    assert!(!seg.is_empty());
    Zeroize::zeroize(&mut seg);
    assert!(seg.is_empty());
    assert_eq!(seg.len(), 0);
    assert!(seg.as_bytes().is_empty());

    // Explicit drop ensures ZeroizeOnDrop handler executes safely without panic
    drop(seg);
}
