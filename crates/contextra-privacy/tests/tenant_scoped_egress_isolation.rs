#![forbid(unsafe_code)]
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use contextra_crypto::crypto::KeyManager;
use contextra_crypto::kv_cipher::ModelFingerprint;
use contextra_privacy::egress_gateway::{
    handle_cloud_query_scoped, CloudQueryRequest,
};
use contextra_privacy::egress_vault::{
    BlockReason, EgressClassification, EgressVault, EgressVaultError, NoOpRecognizer,
};
use contextra_types::{TenantId, TenantScoped};

#[tokio::test]
async fn test_egress_gateway_scoped_isolation() {
    let tenant_a = TenantId::try_new(100).expect("tenant_a");
    let tenant_b = TenantId::try_new(200).expect("tenant_b");

    let vault = EgressVault::try_default().expect("valid vault");
    let request = CloudQueryRequest {
        query: "Harmless query text for tenant A".to_string(),
        collection: None,
        max_results: None,
    };

    let scoped_req = TenantScoped::new(tenant_a, request);

    // 1. Success path: Matching TenantId processes successfully
    let res_ok = handle_cloud_query_scoped(scoped_req.clone(), &tenant_a, &vault).await;
    assert!(res_ok.is_ok(), "Matching tenant_id must succeed");
    let resp = res_ok.expect("response");
    assert_eq!(resp.status, "success");

    // 2. Cross-tenant isolation failure: Mismatched TenantId fails deterministically
    let res_mismatch = handle_cloud_query_scoped(scoped_req, &tenant_b, &vault).await;
    assert!(res_mismatch.is_err(), "Mismatched tenant_id must fail");
    let err_msg = res_mismatch.expect_err("mismatch error").to_string();
    assert!(
        err_msg.contains("tenant scope mismatch"),
        "Error message must indicate tenant scope mismatch: {err_msg}"
    );
}

#[tokio::test]
async fn test_egress_vault_classify_scoped_isolation() {
    let tenant_a = TenantId::try_new(100).expect("tenant_a");
    let tenant_b = TenantId::try_new(200).expect("tenant_b");

    let vault = EgressVault::try_default().expect("valid vault");
    let payload = "Public search query";
    let scoped_payload = TenantScoped::new(tenant_a, payload);

    // 1. Success path: Matching TenantId returns Allow
    let res_ok = vault.classify_scoped(scoped_payload.clone(), &tenant_a).await;
    assert_eq!(res_ok, EgressClassification::Allow);

    // 2. Cross-tenant isolation failure: Mismatched TenantId returns PolicyDenied Block
    let res_mismatch = vault.classify_scoped(scoped_payload, &tenant_b).await;
    assert!(
        matches!(res_mismatch, EgressClassification::Block(BlockReason::PolicyDenied(ref reason)) if reason.contains("tenant scope mismatch")),
        "Expected PolicyDenied block for mismatched tenant scope, got {:?}",
        res_mismatch
    );
}

#[test]
fn test_egress_vault_sanitize_scoped_isolation() {
    let tenant_a = TenantId::try_new(100).expect("tenant_a");
    let tenant_b = TenantId::try_new(200).expect("tenant_b");

    let vault = EgressVault::try_default().expect("valid vault");
    let payload = "Sensitive info for tenant A";
    let scoped_payload = TenantScoped::new(tenant_a, payload);

    // 1. Success path: Matching TenantId sanitizes and returns TenantScoped<String>
    let res_ok = vault.sanitize_and_vault_scoped(scoped_payload.clone(), &tenant_a, &NoOpRecognizer);
    assert!(res_ok.is_ok());
    let (sanitized_scoped, _count) = res_ok.expect("sanitized result");
    assert_eq!(sanitized_scoped.tenant_id(), &tenant_a);

    // 2. Cross-tenant isolation failure: Mismatched TenantId returns TenantScopeViolation error
    let res_mismatch = vault.sanitize_and_vault_scoped(scoped_payload, &tenant_b, &NoOpRecognizer);
    assert!(res_mismatch.is_err());
    let err = res_mismatch.expect_err("mismatch error");
    assert!(matches!(err, EgressVaultError::TenantScopeViolation(_)));
}

#[tokio::test]
async fn test_key_manager_derive_kv_key_scoped_isolation() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_a = TenantId::try_new(100).expect("tenant_a");
    let tenant_b = TenantId::try_new(200).expect("tenant_b");

    let km = KeyManager::try_new("master-passphrase-32bytes-long!", b"salt123")?;
    let fp = ModelFingerprint {
        hash: [0x55; 32],
        model_id: "test-model".to_string(),
        quantization: "Q4_K_M".to_string(),
    };

    let scoped_fp = TenantScoped::new(tenant_a, &fp);

    // 1. Success path: Matching TenantId derives subkey matching direct call
    let key_scoped = km.derive_kv_key_scoped(scoped_fp.clone(), &tenant_a)?;
    let key_direct = km.derive_kv_key(tenant_a, &fp)?;
    assert_eq!(
        key_scoped.inspect_key_bytes_for_test(),
        key_direct.inspect_key_bytes_for_test()
    );

    // 2. Cross-tenant isolation failure: Mismatched TenantId fails with InvalidInput error
    let res_mismatch = km.derive_kv_key_scoped(scoped_fp, &tenant_b);
    assert!(res_mismatch.is_err());
    let err_msg = res_mismatch.expect_err("mismatch error").to_string();
    assert!(err_msg.contains("tenant scope mismatch"));

    // 3. Test cipher_for_scoped matching and mismatch
    let scoped_key_id = TenantScoped::new(tenant_a, "key_001");
    let cipher_ok = km.cipher_for_scoped(scoped_key_id.clone(), &tenant_a);
    assert!(cipher_ok.is_ok());

    let cipher_err = km.cipher_for_scoped(scoped_key_id, &tenant_b);
    assert!(cipher_err.is_err());
    let err_cipher_msg = cipher_err.expect_err("cipher mismatch").to_string();
    assert!(err_cipher_msg.contains("tenant scope mismatch"));

    Ok(())
}
