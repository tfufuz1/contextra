// FILE-CONTEXT
// ZWECK: Campaign coverage test for CloudResponseRehydrator and BulkExfiltrationDetector fail-closed policy evaluation.
// INVARIANTEN: Egress Gateway enforces PII vault surrogate rehydration and volume rate limits fail-closed.
// NICHT-OFFENSICHTLICH: Uses independent oracle (R4) validating surrogate vault roundtrip and sliding-window limits.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_privacy::bulk_exfiltration_detector::{
    BulkExfiltrationDetector, SessionId,
};
use contextra_privacy::egress_gateway::{
    check_bulk_exfiltration, pii_vault_forces_crypto_shred, CloudResponseRehydrator,
};
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_cloud_response_rehydrator_and_pii_vault_shred_rules() {
    let surrogate_1 = "[USER_ENTITY_0001]".to_string();
    let surrogate_2 = "[USER_ENTITY_0002]".to_string();

    let mut vault_map = HashMap::new();
    vault_map.insert(surrogate_1.clone(), "Alice Smith".to_string());
    vault_map.insert(surrogate_2.clone(), "alice@example.com".to_string());

    let rehydrator = CloudResponseRehydrator::new(vault_map)
        .scoped_to_request(vec![surrogate_1.clone(), surrogate_2.clone()]);

    // 1. Rehydrate cloud response text containing surrogates
    let cloud_response = format!("User {surrogate_1} with email {surrogate_2} confirmed.");
    let rehydrated = rehydrator.rehydrate(&cloud_response);

    // Oracle Check (R4): Rehydrated string MUST match exact expected plaintext
    assert_eq!(
        rehydrated,
        "User Alice Smith with email alice@example.com confirmed."
    );

    // 2. Test pii_vault_forces_crypto_shred rule matrix
    assert!(pii_vault_forces_crypto_shred(true, false));
    assert!(!pii_vault_forces_crypto_shred(false, false));
    assert!(!pii_vault_forces_crypto_shred(true, true));
}

#[test]
fn test_check_bulk_exfiltration_rate_limiter_fail_closed() {
    let detector = BulkExfiltrationDetector::new(100, Duration::from_secs(10));
    let session = SessionId::from("egress_session_campaign");

    // First request: 80 bytes -> Allow (Ok(()))
    let payload_1 = "x".repeat(80);
    let outcome1 = check_bulk_exfiltration(&detector, session.clone(), &payload_1);
    assert!(outcome1.is_ok());

    // Second request: 30 bytes -> Exceeds max_bytes (110 > 100) -> Err(EgressError)
    let payload_2 = "x".repeat(30);
    let outcome2 = check_bulk_exfiltration(&detector, session, &payload_2);
    assert!(
        outcome2.is_err(),
        "Exceeding sliding window volume MUST trigger fail-closed Err"
    );
}
