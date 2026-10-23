#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

// FILE-CONTEXT (STAND 2026-10-13, ZWECK: Signer und Schlüsselverwaltung Integrationstests, INVARIANTEN: Zeroize-Hygiene, Fail-closed Rotation, UntrustedKeyList)

use contextra_crypto::deletion_proof::keys::{InMemoryCsprngKeySource, ProofKeySource};
use contextra_crypto::deletion_proof::signer::{ProofKeyPairSigner, ProofSigner, UntrustedKeyList};
use rand::rngs::OsRng;

#[test]
fn test_independent_key_sources_produce_distinct_keys() {
    let source1 = InMemoryCsprngKeySource::new();
    let source2 = InMemoryCsprngKeySource::new();

    let pub1 = source1.public_key().expect("pubkey 1");
    let pub2 = source2.public_key().expect("pubkey 2");
    let key_id1 = source1.key_id().expect("key_id 1");
    let key_id2 = source2.key_id().expect("key_id 2");

    assert_ne!(
        pub1, pub2,
        "Independent sources must produce distinct public keys"
    );
    assert_ne!(
        key_id1, key_id2,
        "Independent sources must produce distinct key IDs"
    );

    let kp1 = source1.generate_keypair().expect("generate kp1");
    let kp2 = source1.generate_keypair().expect("generate kp2");
    assert_ne!(
        kp1.verifying_key_bytes(),
        kp2.verifying_key_bytes(),
        "Repeated generate calls must produce distinct keypairs"
    );
}

#[test]
fn test_key_rotation_statement_and_expiration() {
    let old_signer = ProofKeyPairSigner::generate(&mut OsRng);
    let rotation_ts_ms = 1_700_000_000_000u64;

    let statement = old_signer
        .create_rotation_statement(rotation_ts_ms)
        .expect("create_rotation_statement");

    assert!(
        statement.verify().is_ok(),
        "Rotation statement signature must verify"
    );
    assert_eq!(statement.key_id, old_signer.key_id());
    assert_eq!(statement.public_key, old_signer.public_key());

    // Proof issued before rotation timestamp is valid
    assert!(!statement.is_expired_at(rotation_ts_ms - 1));
    // Proof issued at or after rotation timestamp is expired
    assert!(statement.is_expired_at(rotation_ts_ms));
    assert!(statement.is_expired_at(rotation_ts_ms + 1000));
}

#[test]
fn test_untrusted_key_list_rejects_revoked_keys() {
    let signer = ProofKeyPairSigner::generate(&mut OsRng);
    let untrusted = UntrustedKeyList::new();

    assert!(
        untrusted.verify_trusted(&signer.key_id()).is_ok(),
        "Initially key must be trusted"
    );

    untrusted
        .revoke_public_key(&signer.public_key())
        .expect("revoke_public_key");

    assert!(
        untrusted.is_untrusted(&signer.key_id()),
        "Key must be untrusted after revocation"
    );
    assert!(
        untrusted.verify_trusted(&signer.key_id()).is_err(),
        "verify_trusted must fail for revoked key"
    );
}

#[test]
fn test_debug_format_redacts_secret_keys() {
    let source = InMemoryCsprngKeySource::new();
    let signer = ProofKeyPairSigner::generate(&mut OsRng);
    let keypair = source.load_keypair().expect("load_keypair");

    let debug_source = format!("{:?}", source);
    let debug_signer = format!("{:?}", signer);
    let debug_keypair = format!("{:?}", keypair);

    assert!(
        debug_source.contains("***REDACTED***"),
        "Source debug output must redact secret key"
    );
    assert!(
        debug_signer.contains("***REDACTED***"),
        "Signer debug output must redact secret key"
    );
    assert!(
        debug_keypair.contains("***REDACTED***"),
        "KeyPair debug output must redact secret key"
    );
}
