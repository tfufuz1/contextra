// FILE-CONTEXT
// ZWECK: Integration tests for J19 workspace symbols in contextra-privacy.
// INVARIANTEN: All 11 symbols are invoked and verified through production module API calls.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_privacy::bulk_exfiltration_detector::{BulkExfiltrationDetector, SessionId};
use contextra_privacy::egress_gateway::{
    guard_and_sanitize_payload, handle_cloud_query_scoped,
    handle_cloud_query_scoped_with_bulk_detector, handle_cloud_query_scoped_with_guard,
    CloudQueryRequest,
};
use contextra_privacy::egress_guard::{EgressGuard, TextSearchEngine, TextSearchResult};
use contextra_privacy::egress_vault::{
    BlockReason, EgressClassification, EgressVault, NoOpRecognizer, PolicyCategory, SurrogateVault,
};
use contextra_privacy::error::EgressError;
use contextra_privacy::guarded_payload::{GuardedPayload, Sanitized, Unsanitized};
use contextra_types::{TenantId, TenantScoped};
use std::sync::Arc;
use std::time::Duration;

struct MockSearchEngine {
    results: Vec<TextSearchResult>,
}

impl TextSearchEngine for MockSearchEngine {
    fn search_text<'a>(
        &'a self,
        _text: &'a str,
        _limit: usize,
    ) -> contextra_privacy::egress_vault::BoxFuture<'a, Result<Vec<TextSearchResult>, String>> {
        let res = self.results.clone();
        Box::pin(async move { Ok(res) })
    }
}

#[tokio::test]
async fn test_j19_symbols_integration() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_a = TenantId::try_new(100).expect("valid tenant_a");
    let tenant_b = TenantId::try_new(200).expect("valid tenant_b");

    // 1. EgressVault creation with custom surrogate vault (with_surrogate_vault)
    let surrogate_vault = Arc::new(SurrogateVault::new([77u8; 16]));
    let vault = EgressVault::try_default()?.with_surrogate_vault(surrogate_vault.clone());

    // 2. Policy category (policy_category)
    assert_eq!(vault.policy_category(), PolicyCategory::CloudEgress);

    // 3. classify_scoped
    let query_text = "Clean public text";
    let scoped_text = TenantScoped::new(tenant_a, query_text);
    let class_a = vault.classify_scoped(scoped_text.clone(), &tenant_a).await;
    assert_eq!(class_a, EgressClassification::Allow);

    let class_b = vault.classify_scoped(scoped_text.clone(), &tenant_b).await;
    assert!(matches!(
        class_b,
        EgressClassification::Block(BlockReason::PolicyDenied(_))
    ));

    // 4. sanitize_and_vault_scoped & get_entity
    let sensitive_payload = "Contact john@example.com for access";
    let scoped_sensitive = TenantScoped::new(tenant_a, sensitive_payload);

    let (sanitized_scoped, count) =
        vault.sanitize_and_vault_scoped(scoped_sensitive, &tenant_a, &NoOpRecognizer)?;
    assert_eq!(count, 1);
    let sanitized_str = sanitized_scoped.into_inner_checked(&tenant_a)?;
    assert!(!sanitized_str.contains("john@example.com"));

    // Extract surrogate token and test get_entity
    let token_prefix = "[USER_ENTITY_";
    let start_idx = sanitized_str.find(token_prefix).expect("surrogate present");
    let surrogate_token = &sanitized_str[start_idx..start_idx + token_prefix.len() + 16 + 1];

    let original_entity = vault.get_entity(surrogate_token);
    assert_eq!(original_entity, Some("john@example.com".to_string()));

    // 5. check_scoped on EgressGuard
    let engine = Arc::new(MockSearchEngine { results: vec![] });
    let guard = EgressGuard::new(engine, 0.85, 10);
    let guard_class_a = guard.check_scoped(scoped_text.clone(), &tenant_a).await;
    assert!(matches!(
        guard_class_a,
        EgressClassification::Block(BlockReason::InternalError(_))
    )); // fail-closed on empty index

    let guard_class_b = guard.check_scoped(scoped_text, &tenant_b).await;
    assert!(matches!(
        guard_class_b,
        EgressClassification::Block(BlockReason::PolicyDenied(_))
    ));

    // 6. handle_cloud_query_scoped, handle_cloud_query_scoped_with_guard, handle_cloud_query_scoped_with_bulk_detector
    let req = CloudQueryRequest {
        query: "Valid search query text".to_string(),
        collection: None,
        max_results: None,
    };
    let scoped_req = TenantScoped::new(tenant_a, req);

    let resp_scoped = handle_cloud_query_scoped(scoped_req.clone(), &tenant_a, &vault).await?;
    assert_eq!(resp_scoped.status, "success");

    let resp_scoped_guard =
        handle_cloud_query_scoped_with_guard(scoped_req.clone(), &tenant_a, &vault, None).await?;
    assert_eq!(resp_scoped_guard.status, "success");

    let detector = BulkExfiltrationDetector::new(1000, Duration::from_secs(60));
    let session = SessionId::from("j19_session");
    let resp_scoped_bulk = handle_cloud_query_scoped_with_bulk_detector(
        scoped_req,
        &tenant_a,
        &vault,
        None,
        Some((&detector, session)),
    )
    .await?;
    assert_eq!(resp_scoped_bulk.status, "success");

    // 7. EgressError::policy_violation
    let err = EgressError::policy_violation("Policy denied test");
    assert!(matches!(err, EgressError::PolicyViolation(msg) if msg == "Policy denied test"));

    // 9. GuardedPayload::<Sanitized>::from_sanitized via guard_and_sanitize_payload
    let raw_payload =
        GuardedPayload::<Unsanitized>::new("Safe input".to_string(), "sess_42".to_string());
    let sanitized_payload: GuardedPayload<Sanitized> =
        guard_and_sanitize_payload(raw_payload, &vault, None).await?;
    assert_eq!(sanitized_payload.session_id(), "sess_42");
    assert_eq!(sanitized_payload.into_inner(), "Safe input");

    Ok(())
}
