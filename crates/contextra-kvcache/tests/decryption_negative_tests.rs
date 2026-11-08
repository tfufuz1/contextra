//! SPEC: SPEC §4, D-05, INV-FAILCLOSED-1
//! Negativtests für Segment-Entschlüsselung im contextra-kvcache:
//! Verifiziert Fail-Closed-Verhalten bei falscher AAD, Replay-Angriffen und Bit-Flips
//! unter Verwendung der echten `KvSegmentCipher`-Kryptographie aus `contextra-crypto`.

use bytes::Bytes;
use contextra_crypto::{EncryptedKvLayer, KeyManager, KvSegmentCipher, ModelFingerprint};
use contextra_kvcache::segment::{
    EncryptedSegmentPayload, KvSegment, KvSegmentContent, CURRENT_KV_KEY_DERIVATION_VERSION,
};
use contextra_types::TenantId;

fn setup_cipher_and_manager() -> (KvSegmentCipher, TenantId, ModelFingerprint) {
    let tenant = TenantId::try_new(101).expect("valid tenant");
    let fp = ModelFingerprint { hash: [0x42; 32] };
    let key_manager = KeyManager::new(b"01234567890123456789012345678901");
    let cipher = KvSegmentCipher::new(key_manager, tenant, fp);
    (cipher, tenant, fp)
}

/// SPEC: D-05, INV-FAILCLOSED-1
/// Verifiziert, dass ein Entschlüsselungsversuch mit manipulierter / abweichender Segment-ID (falsche AAD)
/// kryptografisch von AES-256-GCM-SIV abgewiesen wird.
#[test]
fn test_decrypt_wrong_aad_fails() {
    let (cipher, tenant, fp) = setup_cipher_and_manager();
    let original_seg_id = 42;
    let wrong_seg_id = 999;
    let plaintext = b"sensitive_llm_kv_cache_tensor_data";

    let encrypted_layer = cipher
        .encrypt(original_seg_id, fp, plaintext)
        .expect("encryption must succeed");

    // KvSegment mit falscher segment_id (AAD Mismatch)
    let segment = KvSegment {
        tenant_id: tenant,
        segment_id: wrong_seg_id,
        key_derivation_version: CURRENT_KV_KEY_DERIVATION_VERSION,
        content: KvSegmentContent::Raw(Bytes::from(encrypted_layer.ciphertext.clone())),
        encrypted_payload: Some(EncryptedSegmentPayload {
            layer: encrypted_layer,
            key_derivation_version: CURRENT_KV_KEY_DERIVATION_VERSION,
        }),
    };

    let result = segment.decrypt_content(&cipher, fp);
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

    let encrypted_layer = cipher
        .encrypt(seg_id, fp, plaintext)
        .expect("encryption must succeed");

    // Replay with outdated key derivation version injected
    let segment = KvSegment {
        tenant_id: tenant,
        segment_id: seg_id,
        key_derivation_version: CURRENT_KV_KEY_DERIVATION_VERSION,
        content: KvSegmentContent::Raw(Bytes::from(encrypted_layer.ciphertext.clone())),
        encrypted_payload: Some(EncryptedSegmentPayload {
            layer: encrypted_layer,
            key_derivation_version: 0, // Injected version mismatch
        }),
    };

    let result = segment.decrypt_content(&cipher, fp);
    assert!(
        result.is_err(),
        "Decryption with mismatched key derivation version must fail closed"
    );
}

/// SPEC: D-05, INV-FAILCLOSED-1
/// Verifiziert, dass Manipulationen an verschlüsselten Daten (Bit-Flips) deterministisch
/// durch die AEAD-Authentifizierung abgewiesen werden (kein stummes Weiterleiten).
#[test]
fn test_decrypt_bit_flip_fails() {
    let (cipher, tenant, fp) = setup_cipher_and_manager();
    let seg_id = 42;
    let plaintext = b"sensitive_llm_kv_cache_tensor_data";

    let mut encrypted_layer = cipher
        .encrypt(seg_id, fp, plaintext)
        .expect("encryption must succeed");

    // Bit-Flip im Ciphertext
    if let Some(first_byte) = encrypted_layer.ciphertext.first_mut() {
        *first_byte ^= 0x01;
    }

    let segment = KvSegment {
        tenant_id: tenant,
        segment_id: seg_id,
        key_derivation_version: CURRENT_KV_KEY_DERIVATION_VERSION,
        content: KvSegmentContent::Raw(Bytes::from(encrypted_layer.ciphertext.clone())),
        encrypted_payload: Some(EncryptedSegmentPayload {
            layer: encrypted_layer,
            key_derivation_version: CURRENT_KV_KEY_DERIVATION_VERSION,
        }),
    };

    let result = segment.decrypt_content(&cipher, fp);
    assert!(
        result.is_err(),
        "Decryption with bit-flipped ciphertext must fail closed via AEAD verification"
    );
}
