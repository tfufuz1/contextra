use std::collections::HashMap;
use contextra_privacy::egress_gateway::{extract_surrogate_tokens, CloudResponseRehydrator};

#[test]
fn test_scoped_rehydration_allowed_vs_unallowed_tokens() {
    let mut vault_map = HashMap::new();
    vault_map.insert(
        "[USER_ENTITY_00a1]".to_string(),
        "Confidential Alice".to_string(),
    );
    vault_map.insert(
        "[USER_ENTITY_00b2]".to_string(),
        "Confidential Bob".to_string(),
    );

    // Rehydrator with vault containing entries for both Alice (00a1) and Bob (00b2),
    // but scoped ONLY to Alice (00a1) which was actually sent in the request.
    let rehydrator = CloudResponseRehydrator::new(vault_map)
        .scoped_to_request(vec!["[USER_ENTITY_00a1]".to_string()]);

    let cloud_resp = "Response contains [USER_ENTITY_00a1] and injected [USER_ENTITY_00b2].";
    let rehydrated = rehydrator.rehydrate(cloud_resp);

    assert_eq!(
        rehydrated,
        "Response contains Confidential Alice and injected [USER_ENTITY_00b2]."
    );
}

#[test]
fn test_extract_surrogate_tokens_multibyte_utf8_and_invalid_patterns() {
    let sanitized_request =
        "Hallo 🌍! [USER_ENTITY_00a1] test [USER_ENTITY_äöü1] 🚀 [USER_ENTITY_00B2] [USER_ENTITY_zzzz] [USER_ENTITY_123] end";

    let tokens = extract_surrogate_tokens(sanitized_request);

    assert_eq!(tokens.len(), 2);
    assert!(tokens.contains("[USER_ENTITY_00a1]"));
    assert!(tokens.contains("[USER_ENTITY_00B2]"));
    assert!(!tokens.contains("[USER_ENTITY_äöü1]"));
    assert!(!tokens.contains("[USER_ENTITY_zzzz]"));
    assert!(!tokens.contains("[USER_ENTITY_123]"));
}

#[test]
fn test_mixed_text_with_legitimate_and_injected_surrogates() {
    let request_text = "Analysis for [USER_ENTITY_1234] regarding project [USER_ENTITY_abcd].";
    let allowed_tokens = extract_surrogate_tokens(request_text);

    assert_eq!(allowed_tokens.len(), 2);

    let mut vault_map = HashMap::new();
    vault_map.insert("[USER_ENTITY_1234]".to_string(), "Project Alpha".to_string());
    vault_map.insert("[USER_ENTITY_abcd]".to_string(), "Top Secret".to_string());
    vault_map.insert("[USER_ENTITY_ffff]".to_string(), "Unrelated Vault Data".to_string());

    let rehydrator = CloudResponseRehydrator::new(vault_map).scoped_to_request(allowed_tokens);

    let response_text =
        "Result for [USER_ENTITY_1234] ([USER_ENTITY_abcd]). Injected: [USER_ENTITY_ffff] and [USER_ENTITY_9999].";

    let rehydrated = rehydrator.rehydrate(response_text);

    assert_eq!(
        rehydrated,
        "Result for Project Alpha (Top Secret). Injected: [USER_ENTITY_ffff] and [USER_ENTITY_9999]."
    );
}
