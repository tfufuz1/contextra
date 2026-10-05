#![forbid(unsafe_code)]
//! Integration test suite for J19 contextra-privacy symbol closure verification.

use contextra_privacy::egress_gateway::{
    guard_and_sanitize_payload_scoped, handle_cloud_query_scoped,
    handle_cloud_query_scoped_with_bulk_detector, handle_cloud_query_scoped_with_guard,
    CloudQueryRequest, CloudResponseRehydrator, EgressGuardCheck,
};
use contextra_privacy::egress_guard::{EgressGuard, TextSearchEngine, TextSearchResult};
use contextra_privacy::egress_vault::{
    EgressClassification, EgressClassifier, EgressVault, NoOpRecognizer, PolicyCategory,
    SurrogateVault,
};
use contextra_privacy::error::EgressError;
use contextra_privacy::guarded_payload::{GuardedPayload, Sanitized, Unsanitized};
use contextra_types::{TenantId, TenantScoped};
use std::sync::Arc;

struct DummyEngine;

impl TextSearchEngine for DummyEngine {
    fn search_text<'a>(
        &'a self,
        _text: &'a str,
        _limit: usize,
    ) -> contextra_privacy::egress_vault::BoxFuture<'a, Result<Vec<TextSearchResult>, String>>
    {
        Box::pin(async move { Ok(vec![]) })
    }
}

#[tokio::test]
async fn test_j19_closure_11_symbols_verification() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_a = TenantId::try_new(101)?;
    let tenant_b = TenantId::try_new(202)?;

    // 1. with_surrogate_vault & policy_category & get_entity
    let surrogate_vault = Arc::new(SurrogateVault::new([99u8; 16]));
    let vault = EgressVault::try_default()?.with_surrogate_vault(surrogate_vault);

    // Populate surrogate in vault for testing
    let generated_16hex = vault
        .surrogate_vault()
        .generate_surrogate("bob@company.org")?;
    assert!(vault.get_entity(&generated_16hex).is_some());

    let _ = vault
        .sanitize_and_vault("alice@company.org", &NoOpRecognizer)?;

    assert_eq!(vault.policy_category(), PolicyCategory::CloudEgress);

    // Test 4-hex token rehydration via CloudResponseRehydrator
    let token_4hex = "[USER_ENTITY_00a1]".to_string();
    let rehydrator_manual = CloudResponseRehydrator::new(
        [(token_4hex.clone(), "alice@company.org".to_string())]
            .into_iter()
            .collect(),
    )
    .scoped_to_request(vec![token_4hex.clone()]);
    let cloud_output = format!("Referenced user {}", token_4hex);
    assert_eq!(
        rehydrator_manual.rehydrate(&cloud_output),
        "Referenced user alice@company.org"
    );

    // 2. CloudResponseRehydrator::from_vault (using get_entity)
    let rehydrator_from_vault =
        CloudResponseRehydrator::from_vault(&vault, vec![generated_16hex.clone()]);
    assert_eq!(
        rehydrator_from_vault.rehydrate(&format!("Referenced {}", generated_16hex)),
        format!("Referenced {}", generated_16hex) // 16-hex token unchanged by 4-hex regex rehydrator
    );

    // 3. classify_scoped on EgressClassifier trait & EgressVault
    let scoped_clean = TenantScoped::new(tenant_a, "Harmless query text");
    let classifier: &dyn EgressClassifier = &vault;
    let res_scoped_ok = classifier
        .classify_scoped(scoped_clean.clone(), &tenant_a)
        .await;
    assert_eq!(res_scoped_ok, EgressClassification::Allow);

    let res_scoped_mismatch = classifier
        .classify_scoped(scoped_clean.clone(), &tenant_b)
        .await;
    assert!(matches!(
        res_scoped_mismatch,
        EgressClassification::Block(_)
    ));

    // 4. sanitize_and_vault_scoped & guard_and_sanitize_payload_scoped
    let (sanitized_scoped, count) = vault.sanitize_and_vault_scoped(
        TenantScoped::new(tenant_a, "Contact info bob@company.org"),
        &tenant_a,
        &NoOpRecognizer,
    )?;
    assert_eq!(count, 1);
    assert_eq!(sanitized_scoped.tenant_id(), &tenant_a);

    let raw_guarded = GuardedPayload::<Unsanitized>::new(
        "Contact info bob@company.org".to_string(),
        "session_101".to_string(),
    );
    let scoped_guarded_unpacked = guard_and_sanitize_payload_scoped(
        TenantScoped::new(tenant_a, raw_guarded),
        &tenant_a,
        &vault,
        &NoOpRecognizer,
    )
    .await?;
    let sanitized_payload = scoped_guarded_unpacked.into_inner_checked(&tenant_a)?;
    assert_eq!(sanitized_payload.session_id(), "session_101");

    // 5. check_scoped on EgressGuardCheck trait & EgressGuard
    let engine = Arc::new(DummyEngine);
    let guard = EgressGuard::new(engine, 0.85, 128);
    let guard_check: &dyn EgressGuardCheck = &guard;

    let res_guard_ok = guard_check
        .check_scoped(TenantScoped::new(tenant_a, "Short payload"), &tenant_a)
        .await;
    assert_eq!(res_guard_ok, EgressClassification::Allow);

    let res_guard_mismatch = guard_check
        .check_scoped(TenantScoped::new(tenant_a, "Short payload"), &tenant_b)
        .await;
    assert!(matches!(
        res_guard_mismatch,
        EgressClassification::Block(_)
    ));

    // 6. handle_cloud_query_scoped, handle_cloud_query_scoped_with_guard, handle_cloud_query_scoped_with_bulk_detector
    let query_req = CloudQueryRequest {
        query: "Find product documentation".to_string(),
        collection: None,
        max_results: Some(5),
    };
    let scoped_req = TenantScoped::new(tenant_a, query_req);

    let resp_1 = handle_cloud_query_scoped(scoped_req.clone(), &tenant_a, &vault).await?;
    assert_eq!(resp_1.status, "success");

    let resp_2 =
        handle_cloud_query_scoped_with_guard(scoped_req.clone(), &tenant_a, &vault, Some(&guard))
            .await?;
    assert_eq!(resp_2.status, "success");

    let resp_3 = handle_cloud_query_scoped_with_bulk_detector(
        scoped_req.clone(),
        &tenant_a,
        &vault,
        Some(&guard),
        None,
    )
    .await?;
    assert_eq!(resp_3.status, "success");

    // 7. policy_violation & from_sanitized
    let err = EgressError::policy_violation("Access denied by policy");
    assert!(matches!(err, EgressError::PolicyViolation(msg) if msg == "Access denied by policy"));

    let sanitized_payload =
        GuardedPayload::<Sanitized>::from_sanitized("Clean text".to_string(), "sess_999".to_string());
    assert_eq!(sanitized_payload.session_id(), "sess_999");
    assert_eq!(sanitized_payload.into_inner(), "Clean text");

    Ok(())
}
