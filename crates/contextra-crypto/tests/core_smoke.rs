//! Smoke-Test für den entkoppelten Kern von `contextra-crypto` ohne Default-Features und ohne Workspace.

use contextra_crypto::aead::{open_record, seal_record_with_nonce};
use contextra_crypto::dek::Dek;
use contextra_crypto::kdf::{derive_key_argon2id, KdfHeader, KdfParams};
use contextra_crypto::wal_crypto::WalHmac;

#[test]
fn test_crypto_core_smoke_without_workspace() {
    // 1. KDF Primitiv
    let params = KdfParams::new_for_test(64 * 1024, 3, 1);
    let header = KdfHeader::new(params, vec![0x11u8; 32]).expect("kdf header");
    let derived_key = derive_key_argon2id("smoke-passphrase-valid", &header).expect("kdf derive");
    assert_eq!(derived_key.0.len(), 32);

    // 2. WAL-HMAC Primitiv
    let key = [0x55u8; 32];
    let mut mac = WalHmac::new(&key).expect("wal hmac init");
    mac.update(b"core_smoke_wal_entry");
    let tag = mac.finalize();
    assert_ne!(tag, [0u8; 32]);

    // 3. AEAD Primitiv
    let dek = Dek::new([0x88u8; 32]);
    let doc = [0x22u8; 16];
    let key_bytes = b"smoke_record_key";
    let plain = b"smoke payload data for ring 0 crypto";
    let nonce = [0x33u8; 12];

    let ct = seal_record_with_nonce(&dek, doc, key_bytes, plain, nonce).expect("seal");
    let recovered = open_record(&dek, key_bytes, &ct).expect("open");
    assert_eq!(recovered.as_slice(), plain);
}
