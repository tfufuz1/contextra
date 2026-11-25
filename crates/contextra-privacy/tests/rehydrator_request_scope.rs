use contextra_privacy::egress_gateway::{extract_surrogate_tokens, CloudResponseRehydrator};
use std::collections::HashMap;

#[test]
fn test_rehydrator_scoped_token_isolation() {
    let mut vault_map = HashMap::new();
    vault_map.insert("[USER_ENTITY_00a1]".to_string(), "Alice".to_string());
    vault_map.insert("[USER_ENTITY_00b2]".to_string(), "Bob".to_string());

    // Only allow token 00a1 in scope
    let allowed = vec!["[USER_ENTITY_00a1]".to_string()];

    let rehydrator = CloudResponseRehydrator::new(vault_map).scoped_to_request(allowed);

    let cloud_response =
        "Hallo [USER_ENTITY_00a1], hier ist [USER_ENTITY_00b2] und [USER_ENTITY_00c3].";
    let result = rehydrator.rehydrate(cloud_response);

    // [USER_ENTITY_00a1] is in scope -> replaced with Alice
    // [USER_ENTITY_00b2] is in vault but NOT in scope -> left unchanged
    // [USER_ENTITY_00c3] is NOT in vault and NOT in scope -> left unchanged
    assert_eq!(
        result,
        "Hallo Alice, hier ist [USER_ENTITY_00b2] und [USER_ENTITY_00c3]."
    );
}

#[test]
fn test_extract_surrogate_tokens_multibyte_and_invalid_patterns() {
    let text = "Text mit Umlauten äöü! 🌍 Token: [USER_ENTITY_1234], Emoji 🔥 [USER_ENTITY_abcd]. \
                Ungültig: [USER_ENTITY_zzzz], [USER_ENTITY_12], [USER_ENTITY_12345] und [USER_ENTITY_";

    let extracted = extract_surrogate_tokens(text);

    assert_eq!(extracted.len(), 2);
    assert!(extracted.contains("[USER_ENTITY_1234]"));
    assert!(extracted.contains("[USER_ENTITY_abcd]"));
    assert!(!extracted.contains("[USER_ENTITY_zzzz]"));
    assert!(!extracted.contains("[USER_ENTITY_12]"));
}

#[test]
fn test_rehydrator_mixed_valid_and_injected_surrogate_tokens() {
    let mut vault_map = HashMap::new();
    vault_map.insert(
        "[USER_ENTITY_00a1]".to_string(),
        "Geheime ProjektA".to_string(),
    );
    vault_map.insert(
        "[USER_ENTITY_00b2]".to_string(),
        "StrengGeheim ProjektB".to_string(),
    );

    // Request sent to cloud only contained [USER_ENTITY_00a1]
    let outgoing_request = "Anfrage bzgl. [USER_ENTITY_00a1] in der Cloud.";
    let allowed_tokens = extract_surrogate_tokens(outgoing_request);

    assert_eq!(allowed_tokens.len(), 1);
    assert!(allowed_tokens.contains("[USER_ENTITY_00a1]"));

    let rehydrator = CloudResponseRehydrator::new(vault_map).scoped_to_request(allowed_tokens);

    // Cloud response includes both legitimate token 00a1 and injected token 00b2
    let cloud_response = "Antwort: [USER_ENTITY_00a1] verarbeitet. Injected: [USER_ENTITY_00b2].";
    let rehydrated = rehydrator.rehydrate(cloud_response);

    assert_eq!(
        rehydrated,
        "Antwort: Geheime ProjektA verarbeitet. Injected: [USER_ENTITY_00b2]."
    );
}
