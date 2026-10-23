#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

//! Tests for Stage 5: Key Classes & TrustList (w1_06_*).

#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::signer::ProofKeyPairSigner;
use contextra_crypto::deletion_proof::ProofSigner;
use contextra_crypto::{KeyClass, TrustList};
use ed25519_dalek::VerifyingKey;
use rand::rngs::OsRng;

#[test]
fn w1_06_proof_key_class_legacy_flagged() {
    let signer_current = ProofKeyPairSigner::generate(&mut OsRng);
    let signer_revoked = ProofKeyPairSigner::generate(&mut OsRng);
    let signer_expired = ProofKeyPairSigner::generate(&mut OsRng);

    let vk_current = VerifyingKey::from_bytes(&signer_current.public_key()).expect("vk");
    let vk_revoked = VerifyingKey::from_bytes(&signer_revoked.public_key()).expect("vk");
    let vk_expired = VerifyingKey::from_bytes(&signer_expired.public_key()).expect("vk");

    let mut trust_list = TrustList::new();

    // 1. Register current key valid from 1000 to 5000
    trust_list.register_key(
        signer_current.key_id(),
        vk_current,
        1000,
        5000,
        KeyClass::Current,
    );

    // 2. Register revoked key
    trust_list.register_key(
        signer_revoked.key_id(),
        vk_revoked,
        1000,
        5000,
        KeyClass::Revoked,
    );

    // 3. Register expired key
    trust_list.register_key(
        signer_expired.key_id(),
        vk_expired,
        1000,
        2000,
        KeyClass::Current,
    );

    // Lookup current key within window
    let (class_current, _) = trust_list
        .lookup(&signer_current.key_id(), 2000)
        .expect("found current");
    assert_eq!(class_current, KeyClass::Current);

    // Lookup revoked key within window -> MUST be KeyClass::Revoked
    let (class_revoked, _) = trust_list
        .lookup(&signer_revoked.key_id(), 2000)
        .expect("found revoked");
    assert_eq!(class_revoked, KeyClass::Revoked);

    // Lookup expired key outside window -> MUST be flagged KeyClass::Legacy
    let (class_expired, _) = trust_list
        .lookup(&signer_expired.key_id(), 3000)
        .expect("found expired");
    assert_eq!(class_expired, KeyClass::Legacy);
}
