//! SPEC: SPEC §4, D-05, INV-FAILCLOSED-1
//! Negativtests für Segment-Entschlüsselung im contextra-kvcache:
//! Verifiziert Fail-Closed-Verhalten bei falscher AAD, Replay-Angriffen und Bit-Flips
//! unter Verwendung der `KvSegmentCipher`-Kryptographie aus `contextra-crypto`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    unused_imports
)]

use contextra_crypto::{CryptoKey, KeyManager, KvSegmentCipher, ModelFingerprint};
use contextra_kvcache::segment::{KvSegment, CURRENT_KV_KEY_DERIVATION_VERSION};
use contextra_types::TenantId;

fn setup_cipher_and_manager() -> (KvSegmentCipher, TenantId, ModelFingerprint) {
    let tenant = TenantId::try_new(101).expect("valid tenant");
    let fp = ModelFingerprint::new([0x42; 32], "test-model", "fp16");
    let key_manager = CryptoKey::try_new("passphrase-01234567890123456789", b"salt-0123456789")
        .expect("valid key");
    let cipher = KvSegmentCipher::new(key_manager);
    (cipher, tenant, fp)
}

/// SPEC: D-05, INV-FAILCLOSED-1
/// Verifiziert, dass ein Entschlüsselungsversuch mit manipulierte AAD fehlschlägt.
#[test]
fn test_decrypt_wrong_aad_fails() {
    let (cipher, tenant, fp) = setup_cipher_and_manager();
    let original_seg_id = 42;
    let wrong_seg_id = 999;
    let plaintext = b"sensitive_llm_kv_cache_tensor_data";

    let mut segment = KvSegment::new_encrypted(
        &cipher,
        tenant,
        original_seg_id,
        fp.clone(),
        None,
        plaintext,
    )
    .expect("encryption must succeed");

    // Manually tamper segment_id (AAD Mismatch)
    segment.segment_id = wrong_seg_id;

    let result = segment.decrypt_data(&cipher);
    assert!(
        result.is_err(),
        "Decryption with mismatched segment_id AAD must fail closed via AEAD verification"
    );
}

/// SPEC: D-05, INV-FAILCLOSED-1
/// Verifiziert, dass Replay eines Segments mit manipulierter Schlüssel-Derivations-Version scheitert.
#[test]
fn test_decrypt_replay_version_mismatch_fails() {
    let (cipher, tenant, fp) = setup_cipher_and_manager();
    let seg_id = 42;
    let plaintext = b"sensitive_llm_kv_cache_tensor_data";

    let mut segment =
        KvSegment::new_encrypted(&cipher, tenant, seg_id, fp.clone(), None, plaintext)
            .expect("encryption must succeed");

    // Inject version mismatch
    segment.key_derivation_version = 0;

    let result = segment.decrypt_data(&cipher);
    assert!(
        result.is_err(),
        "Decryption with mismatched key derivation version must fail closed"
    );
}

/// SPEC: D-05, INV-FAILCLOSED-1
/// Verifiziert, dass Manipulationen an verschlüsselten Daten (Bit-Flips) deterministisch
/// durch die AEAD-Authentifizierung abgewiesen werden.
#[test]
fn test_decrypt_bit_flip_fails() {
    let (cipher, tenant, fp) = setup_cipher_and_manager();
    let seg_id = 42;
    let plaintext = b"sensitive_llm_kv_cache_tensor_data";

    let _segment = KvSegment::new_encrypted(&cipher, tenant, seg_id, fp.clone(), None, plaintext)
        .expect("encryption must succeed");

    let layer = cipher
        .encrypt_with_version(
            tenant,
            seg_id,
            CURRENT_KV_KEY_DERIVATION_VERSION,
            fp.clone(),
            plaintext,
        )
        .expect("must encrypt");

    let mut tampered_layer = layer;
    if let Some(first_byte) = tampered_layer.ciphertext.first_mut() {
        *first_byte ^= 0x01;
    }

    let decrypt_res =
        cipher.decrypt_with_version(&tampered_layer, seg_id, CURRENT_KV_KEY_DERIVATION_VERSION);

    assert!(
        decrypt_res.is_err(),
        "Decryption with bit-flipped ciphertext must fail closed via AEAD verification"
    );
}
