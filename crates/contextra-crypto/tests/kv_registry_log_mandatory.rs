// FILE-CONTEXT
// ZWECK: TDD Unit/Integrationstests für pflichtmäßiges RevocationLog in KeyRegistry (W4-02 / T-2026-0237).
// INVARIANTEN:
// - Persist-before-Mutate: revoke_group mutiert keinen RAM-Zustand, wenn Log-Append fehlschlägt.
// - Mandatory Log: KeyRegistry besitzt keinen Zustand ohne RevocationLog.

#![forbid(unsafe_code)]

use contextra_crypto::{
    crypto::KeyManager,
    kv_shredding::KeyRegistry,
    revocation_log::RevocationLog,
};
use contextra_ports::SystemClock;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_persist_before_mutate_revoke_group_fails_without_ram_mutation() {
    let dir = tempdir().expect("Failed to create tempdir");
    let log_path = dir.path().join("failing_revocation.log");

    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    let clock = Arc::new(SystemClock::new());

    let log = Arc::new(
        RevocationLog::open_or_create_if_fresh(&log_path, clock, Some(sk), vk, false)
            .expect("Failed to create log"),
    );

    let registry = KeyRegistry::new_for_test(log.clone());
    let km = KeyManager::try_new("test-passphrase", b"salt1").unwrap();
    let group_id = 42;

    // Register group key
    let _subkey = registry.get_or_derive(&km, group_id).unwrap();
    assert!(registry.is_group_active(group_id));

    // Intentionally break the log file path by creating a directory with the same name
    // so write_atomic_file inside log.append will fail with an I/O error
    std::fs::remove_file(&log_path).expect("Failed to remove log file");
    std::fs::create_dir(&log_path).expect("Failed to create blocking directory");

    // Call revoke_group; must fail due to append failure
    let res = registry.revoke_group(group_id);
    assert!(res.is_err(), "revoke_group MUST fail when log.append fails");

    // RAM state MUST remain unmutated (Persist-before-Mutate invariant)
    assert!(
        !registry.is_group_revoked(group_id),
        "Group MUST NOT be marked revoked in RAM when persist fails"
    );
    assert!(
        registry.is_group_active(group_id),
        "Group MUST remain active in RAM when persist fails"
    );
}

#[test]
fn test_key_registry_requires_revocation_log_type_signature() {
    let clock = Arc::new(SystemClock::new());
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    let log = Arc::new(RevocationLog::new_in_memory(clock, Some(sk), vk));

    // KeyRegistry::new accepts Arc<RevocationLog>
    let registry = KeyRegistry::new_for_test(log.clone());
    assert!(!registry.is_group_revoked(999));
}
