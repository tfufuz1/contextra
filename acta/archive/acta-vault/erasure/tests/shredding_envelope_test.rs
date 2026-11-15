#![cfg(all(feature = "signing", feature = "vault"))]

//! # Integrationstest: `shredding_envelope_test.rs`
//!
//! Validiert die kryptographische Unumkehrbarkeit des KV-Shreddings,
//! Envelope Encryption, O(1) Key Destruction sowie Mandanten/Gruppenisolation.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use acta_erasure::error::ErasureError;
use acta_erasure::kv_shredding::{
    derive_subkey, EncryptedRecordPayload, KeyRegistry, DEFAULT_SHRED_KEY_GROUP_SIZE,
};
use acta_erasure::revocation_log::{RevocationLog, RevocationTarget};
use acta_erasure::types_local::{Clock, SystemClock};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::sync::Arc;

#[test]
fn test_envelope_shredding_64_records_isolation() {
    let registry = KeyRegistry::new();
    let group_id = 1001;

    let mut payloads: Vec<EncryptedRecordPayload> =
        Vec::with_capacity(DEFAULT_SHRED_KEY_GROUP_SIZE);

    for rec_id in 0..DEFAULT_SHRED_KEY_GROUP_SIZE as u64 {
        let plaintext = format!("Sensitive payload data for item #{rec_id}");
        let payload = registry
            .encrypt_envelope_record(group_id, rec_id, plaintext.as_bytes())
            .expect("encryption must succeed");
        payloads.push(payload);
    }

    // Verify all can decrypt initially
    for (i, p) in payloads.iter().enumerate() {
        let dec = registry.decrypt_envelope_record(p).expect("decryption ok");
        assert_eq!(
            dec,
            format!("Sensitive payload data for item #{i}").as_bytes()
        );
    }

    // Revoke a single record #7
    let revoked = registry.revoke_record_num(group_id, 7);
    assert!(revoked, "revoke_record_num must succeed");

    // Decrypting record #7 must fail fail-closed
    let dec_7 = registry.decrypt_envelope_record(&payloads[7]);
    assert!(
        matches!(dec_7, Err(ErasureError::Refused)),
        "Decryption of shredded record MUST be refused"
    );

    // Neighbor records must remain decryptable
    for (i, p) in payloads.iter().enumerate() {
        if i == 7 {
            continue;
        }
        let dec = registry
            .decrypt_envelope_record(p)
            .expect("neighbor decryption ok");
        assert_eq!(
            dec,
            format!("Sensitive payload data for item #{i}").as_bytes()
        );
    }
}

#[test]
fn test_envelope_group_shredding_atomicity() {
    let (sk, vk) = {
        let signing = SigningKey::generate(&mut OsRng);
        let verifying = signing.verifying_key();
        (signing, verifying)
    };
    let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
    let rev_log = Arc::new(RevocationLog::new_in_memory(clock, Some(sk), vk));
    let registry = KeyRegistry::new().with_revocation_log(rev_log.clone());

    let group_id = 999;
    let payload = registry
        .encrypt_envelope_record(group_id, 1, b"classified-data")
        .expect("encrypt ok");

    assert!(registry.is_group_active(group_id));

    // Destroy the entire group key
    registry.revoke_group(group_id).expect("revoke ok");
    assert!(!registry.is_group_active(group_id));
    assert!(registry.is_group_revoked(group_id));
    assert!(rev_log.is_revoked(&RevocationTarget::Group(group_id)));

    // Decryption must fail
    let dec = registry.decrypt_envelope_record(&payload);
    assert!(matches!(dec, Err(ErasureError::Refused)));

    // Direct group operations must fail
    assert!(registry.encrypt_with_group(group_id, b"more data").is_err());
    assert!(registry.get_or_derive(group_id).is_err());
}

#[test]
fn test_derive_subkey_deterministic_kdf() {
    let master = [0x42u8; 32];
    let k1 = derive_subkey(&master, 1, 0);
    let k2 = derive_subkey(&master, 1, 0);
    let k3 = derive_subkey(&master, 2, 0);

    assert_eq!(k1.0, k2.0);
    assert_ne!(k1.0, k3.0);
}
