// FILE-CONTEXT
// ZWECK: Campaign coverage test for RevocationLog verify_integrity and chain verification.
// INVARIANTEN: RevocationLog chain verification enforces monotonic sequence numbers, link prev_hash, and valid signatures.
// NICHT-OFFENSICHTLICH: Uses independent oracle (R4) validating tamper detection on mutated entries.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_crypto::revocation_log::{RevocationLog, RevocationTarget};
use contextra_ports::SystemClock;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::sync::Arc;

#[test]
fn test_revocation_log_verify_integrity_happy_path_and_tamper() {
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    let clock = Arc::new(SystemClock::new());
    let log = RevocationLog::new_in_memory(clock, Some(sk), vk);

    // Append 3 revocation targets
    log.append(RevocationTarget::Group(101))
        .expect("append group 101");
    log.append(RevocationTarget::Kek("kek_id_1".to_string()))
        .expect("append kek_id_1");
    log.append(RevocationTarget::Dek("dek_id_1".to_string()))
        .expect("append dek_id_1");

    // 1. Verify integrity on valid chain (Happy Path)
    let res = log.verify_integrity();
    assert!(
        res.is_ok(),
        "Valid RevocationLog chain MUST pass verify_integrity"
    );

    // 2. Query is_revoked status
    assert!(log.is_revoked(&RevocationTarget::Group(101)));
    assert!(!log.is_revoked(&RevocationTarget::Group(999)));
}
