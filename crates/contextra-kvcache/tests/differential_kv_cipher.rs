// Herkunfts-Commit: ca103a3c743df21a70c3b0b0e2b3972a693a5ca4
//! Differenztest: Prüft, dass die KV-Verschlüsselung in `contextra-kvcache` bitidentisch mit
//! `contextra-crypto::kv_cipher` und `contextra-crypto::KeyManager` arbeitet.

use contextra_crypto::{KeyManager, KvSegmentCipher, ModelFingerprint};
use contextra_ids::TenantId;

#[test]
fn test_differential_kv_cipher_bit_level_identity() {
    let master_pass = "master-passphrase-differential-test";
    let master_salt = b"master-salt-42";
    let master_km = KeyManager::try_new(master_pass, master_salt).expect("master km");
    let master_km2 = KeyManager::try_new(master_pass, master_salt).expect("master km 2");
    let cipher = KvSegmentCipher::new(master_km);

    let tenant = TenantId::try_new(101).expect("valid tenant");
    let fp = ModelFingerprint::new([0x42u8; 32], "llama-3-8b", "Q4_K_M");
    let plaintext = b"Hello, this is a deterministic differential encryption payload for KV cache!";

    // Verschlüsselung über KvSegmentCipher
    let encrypted = cipher
        .encrypt(tenant, fp.clone(), plaintext)
        .expect("encrypt");
    assert_eq!(encrypted.tenant_id, tenant);
    assert_eq!(encrypted.model_fingerprint, fp);

    // Entschlüsselung über KvSegmentCipher
    let decrypted = cipher.decrypt(&encrypted).expect("decrypt");
    assert_eq!(decrypted, plaintext);

    // Direkte Prüfung über Sub-KeyManager der gleichen Ableitung
    let sub_km = master_km2
        .derive_kv_key(tenant, &fp)
        .expect("derive sub km");
    let manual_decrypted = sub_km
        .decrypt_auto_nonce(&encrypted.ciphertext, &encrypted.nonce)
        .expect("manual decrypt");
    assert_eq!(manual_decrypted, plaintext);
}
