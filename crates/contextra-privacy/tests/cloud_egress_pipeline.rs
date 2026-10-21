//! Integration tests for `CloudEgressPipeline` and `CloudEgressPipelineBuilder`.

#![forbid(unsafe_code)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    deprecated
)]

use contextra_privacy::{
    handle_cloud_query, BlockReason, BoxFuture, BulkExfiltrationDetector,
    CloudEgressPipelineBuilder, CloudQueryRequest, EgressClassification, EgressError,
    EgressGuardCheck, EgressVault, SessionId, SurrogateVault,
};
use contextra_types::{TenantId, TenantScoped};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

struct MockAlwaysBlockGuard {
    called: Arc<AtomicBool>,
}

impl EgressGuardCheck for MockAlwaysBlockGuard {
    fn check<'a>(&'a self, _payload: &'a str) -> BoxFuture<'a, EgressClassification> {
        self.called.store(true, Ordering::SeqCst);
        Box::pin(async move {
            EgressClassification::Block(BlockReason::PolicyDenied("L4 Guard Security Block".into()))
        })
    }
}

struct MockAllowGuard {
    called: Arc<AtomicBool>,
}

impl EgressGuardCheck for MockAllowGuard {
    fn check<'a>(&'a self, _payload: &'a str) -> BoxFuture<'a, EgressClassification> {
        self.called.store(true, Ordering::SeqCst);
        Box::pin(async move { EgressClassification::Allow })
    }
}

#[tokio::test]
async fn test_pipeline_volume_limit_exceeded_policy_violation() {
    let tenant_id = TenantId::try_new(10).unwrap();
    let vault = EgressVault::new(vec![]).unwrap();
    let detector = Arc::new(BulkExfiltrationDetector::new(100, Duration::from_secs(60)));
    let session = SessionId::from("sess_volume_test");

    let pipeline = CloudEgressPipelineBuilder::new_with_vault(vault, tenant_id)
        .with_bulk_detector(detector, session)
        .build();

    // Query 1: 50 bytes (within 100 limit) -> Success
    let req1 = CloudQueryRequest {
        query: "A".repeat(50),
        collection: None,
        max_results: None,
    };
    let res1 = pipeline.handle(TenantScoped::new(tenant_id, req1)).await;
    assert!(
        res1.is_ok(),
        "First query under volume limit should succeed"
    );

    // Query 2: 60 bytes (50 + 60 = 110 > 100) -> Fails with PolicyViolation
    let req2 = CloudQueryRequest {
        query: "B".repeat(60),
        collection: None,
        max_results: None,
    };
    let res2 = pipeline.handle(TenantScoped::new(tenant_id, req2)).await;

    match res2 {
        Err(EgressError::PolicyViolation(msg)) => {
            assert!(
                msg.contains("Egress volume rate limit exceeded"),
                "Expected volume rate limit violation message, got: {msg}"
            );
        }
        other => panic!("Expected EgressError::PolicyViolation, got {:?}", other),
    }
}

#[tokio::test]
async fn test_pipeline_tenant_mismatch_rejected() {
    let tenant_a = TenantId::try_new(10).unwrap();
    let tenant_b = TenantId::try_new(20).unwrap();
    let vault = EgressVault::new(vec![]).unwrap();

    let pipeline = CloudEgressPipelineBuilder::new_with_vault(vault, tenant_a).build();

    let req = CloudQueryRequest {
        query: "Harmless query".into(),
        collection: None,
        max_results: None,
    };

    // Send request scoped to tenant_b when pipeline expects tenant_a
    let res = pipeline.handle(TenantScoped::new(tenant_b, req)).await;

    match res {
        Err(EgressError::PolicyViolation(msg)) => {
            assert!(
                msg.contains("tenant scope mismatch"),
                "Expected tenant mismatch error, got: {msg}"
            );
        }
        other => panic!(
            "Expected EgressError::PolicyViolation for tenant mismatch, got {:?}",
            other
        ),
    }
}

#[tokio::test]
async fn test_pipeline_guard_block_prevents_gateway() {
    let tenant_id = TenantId::try_new(10).unwrap();
    let vault = EgressVault::new(vec![]).unwrap();
    let guard_called = Arc::new(AtomicBool::new(false));
    let mock_guard: Arc<dyn EgressGuardCheck> = Arc::new(MockAlwaysBlockGuard {
        called: guard_called.clone(),
    });

    let pipeline = CloudEgressPipelineBuilder::new_with_vault(vault, tenant_id)
        .with_guard(mock_guard)
        .build();

    let req = CloudQueryRequest {
        query: "Test query triggers L4 block".into(),
        collection: None,
        max_results: None,
    };

    let res = pipeline.handle(TenantScoped::new(tenant_id, req)).await;

    assert!(
        guard_called.load(Ordering::SeqCst),
        "L4 Guard check should have been invoked"
    );

    match res {
        Err(EgressError::PolicyViolation(msg)) => {
            assert!(
                msg.contains("L4 Guard Security Block"),
                "Expected Guard policy violation, got: {msg}"
            );
        }
        other => panic!("Expected EgressError::PolicyViolation, got {:?}", other),
    }
}

#[tokio::test]
async fn test_pipeline_happy_path_regression_matches_direct_gateway() {
    let tenant_id = TenantId::try_new(10).unwrap();
    let vault = EgressVault::new(vec![r"sk-[a-zA-Z0-9]{32}".into()]).unwrap();
    let guard_called = Arc::new(AtomicBool::new(false));
    let mock_guard: Arc<dyn EgressGuardCheck> = Arc::new(MockAllowGuard {
        called: guard_called.clone(),
    });

    let pipeline = CloudEgressPipelineBuilder::new_with_vault(vault.clone(), tenant_id)
        .with_guard(mock_guard)
        .build();

    let req = CloudQueryRequest {
        query: "Search for public documentation on Rust async".into(),
        collection: Some("docs".into()),
        max_results: Some(5),
    };

    // Direct gateway call
    let direct_res = handle_cloud_query(req.clone(), &vault).await.unwrap();

    // Pipeline call
    let pipeline_res = pipeline
        .handle(TenantScoped::new(tenant_id, req))
        .await
        .unwrap();

    assert_eq!(pipeline_res.status, direct_res.status);
    assert_eq!(pipeline_res.query, direct_res.query);
    assert_eq!(pipeline_res.abstracted, direct_res.abstracted);
    assert_eq!(pipeline_res.results, direct_res.results);
    assert!(
        guard_called.load(Ordering::SeqCst),
        "Guard check executed on happy path"
    );
}

#[tokio::test]
async fn test_pipeline_surrogate_vault_builder_efficacy() {
    let tenant_id = TenantId::try_new(10).unwrap();
    let vault = EgressVault::new(vec![r"alice@example\.com".into()]).unwrap();
    let custom_salt = [0x55u8; 16];
    let custom_surrogate_vault = Arc::new(SurrogateVault::new(custom_salt));

    let configured_vault = vault.with_surrogate_vault(custom_surrogate_vault.clone());

    let _pipeline = CloudEgressPipelineBuilder::new_with_vault(configured_vault.clone(), tenant_id)
        .with_surrogate_vault(custom_surrogate_vault.clone())
        .build();

    // Sanitize using configured vault
    let (sanitized, count) = configured_vault
        .sanitize_and_vault(
            "Contact alice@example.com",
            &contextra_privacy::NoOpRecognizer,
        )
        .unwrap();

    assert_eq!(count, 1);
    assert!(!sanitized.contains("alice@example.com"));

    // Verify the custom surrogate vault recorded the mapping
    assert_eq!(
        custom_surrogate_vault.len(),
        1,
        "Custom SurrogateVault received generated surrogate entity mapping"
    );
}
