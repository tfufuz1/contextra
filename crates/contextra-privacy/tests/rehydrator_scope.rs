use contextra_privacy::egress_gateway::CloudResponseRehydrator;
use std::collections::HashMap;

#[test]
fn test_unscoped_rehydrator_leaves_known_surrogate_unchanged() {
    let mut vault_map = HashMap::new();
    vault_map.insert(
        "[USER_ENTITY_00a1]".to_string(),
        "Secret Corporate Data".to_string(),
    );

    // Unscoped rehydrator (allowed_tokens is None)
    let rehydrator = CloudResponseRehydrator::new(vault_map);

    let response = "Cloud returned: [USER_ENTITY_00a1] in response.";
    let rehydrated = rehydrator.rehydrate(response);

    // Fail-closed behavior: no tokens substituted
    assert_eq!(rehydrated, response);
}

#[test]
fn test_scoped_rehydrator_replaces_in_scope_token_leaves_out_of_scope_unchanged() {
    let mut vault_map = HashMap::new();
    vault_map.insert(
        "[USER_ENTITY_00a1]".to_string(),
        "Token A Value".to_string(),
    );
    vault_map.insert(
        "[USER_ENTITY_00b2]".to_string(),
        "Token B Value".to_string(),
    );

    // Scoped only to Token A ([USER_ENTITY_00a1])
    let allowed = vec!["[USER_ENTITY_00a1]".to_string()];
    let rehydrator = CloudResponseRehydrator::new(vault_map).scoped_to_request(allowed);

    let response = "Values: [USER_ENTITY_00a1] and [USER_ENTITY_00b2].";
    let rehydrated = rehydrator.rehydrate(response);

    // Token A is in scope and vault_map -> replaced
    // Token B is in vault_map but NOT in scope -> left unchanged
    assert_eq!(rehydrated, "Values: Token A Value and [USER_ENTITY_00b2].");
}

#[test]
fn test_token_in_response_not_in_vault_map_stays_unchanged() {
    let mut vault_map = HashMap::new();
    vault_map.insert(
        "[USER_ENTITY_00a1]".to_string(),
        "Token A Value".to_string(),
    );

    // Token C ([USER_ENTITY_00c3]) is in scope, but NOT in vault_map
    let allowed = vec![
        "[USER_ENTITY_00a1]".to_string(),
        "[USER_ENTITY_00c3]".to_string(),
    ];
    let rehydrator = CloudResponseRehydrator::new(vault_map).scoped_to_request(allowed);

    let response = "Values: [USER_ENTITY_00a1] and [USER_ENTITY_00c3].";
    let rehydrated = rehydrator.rehydrate(response);

    // Token A is replaced
    // Token C is in scope but NOT in vault_map -> stays unchanged
    assert_eq!(rehydrated, "Values: Token A Value and [USER_ENTITY_00c3].");
}
