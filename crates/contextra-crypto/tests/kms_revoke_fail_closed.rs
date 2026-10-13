// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Testet Fail-Closed Verhalten bei KMS Key Destruction wenn KeyRegistry-Vernichtung fehlschlägt.
// INVARIANTEN: Zero-Panic, Fail-Closed (Widerruf eingetragen, Fehler wird zurückgegeben, kein Löschbeleg erzeugbar).

#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use contextra_crypto::{
    KeyManager, KeyRegistry, KmsProvider, LocalKmsProvider, MfksKeystore, RevocationLog,
};
use std::sync::Arc;

#[test]
fn test_kms_revoke_fail_closed_on_registry_error() {
    let (master_key, header) = KeyManager::try_new_argon2id("test-passphrase").unwrap();
    let keystore = MfksKeystore::new(header, vec![], vec![]).unwrap();
    let clock = Arc::new(contextra_ports::SystemClock::new());
    let sk = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    let vk = sk.verifying_key();
    let log = Arc::new(RevocationLog::new_in_memory(clock.clone(), Some(sk), vk));
    let registry = Arc::new(KeyRegistry::new());

    let provider = LocalKmsProvider::new(master_key, keystore, registry.clone(), log, clock);

    // Initialize/derive key 42 in registry so it exists
    let _key_bytes = provider.generate_key("42").unwrap();

    // Poison the registry lock to simulate a write failure during registry.revoke_group
    registry.poison_groups_lock_for_test();

    // Invoking destroy_key on key "42" MUST fail with Err
    let result = provider.destroy_key("42");
    assert!(
        result.is_err(),
        "destroy_key must return Err on registry zeroization failure"
    );

    let err_msg = result.err().unwrap().to_string();
    assert!(
        err_msg.contains("Registry-Vernichtung fehlgeschlagen"),
        "Error message must indicate registry destruction failure: {err_msg}"
    );

    // Verify key 42 is STILL considered revoked fail-closed
    assert!(
        registry.is_group_revoked(42),
        "Key 42 must remain revoked (fail-closed) even when registry zeroization fails"
    );
}
