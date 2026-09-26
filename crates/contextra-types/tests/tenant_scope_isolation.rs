#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_types::{TenantId, TenantScopeViolation, TenantScoped};

#[test]
fn test_tenant_scoped_new_and_tenant_id_getter() {
    let tenant_id = TenantId::try_new(42).expect("valid tenant_id");
    let scoped = TenantScoped::new(tenant_id, "sensitive_data");

    assert_eq!(scoped.tenant_id(), &tenant_id);
}

#[test]
fn test_into_inner_checked_matching_tenant() {
    let tenant = TenantId::try_new(100).expect("valid tenant_id");
    let value = String::from("confidential document text");
    let scoped = TenantScoped::new(tenant, value.clone());

    let result = scoped.into_inner_checked(&tenant);
    assert!(result.is_ok());
    assert_eq!(result.expect("unpacked value"), value);
}

#[test]
fn test_into_inner_checked_mismatched_tenant_does_not_leak_value() {
    let owner_tenant = TenantId::try_new(100).expect("valid owner_tenant");
    let requester_tenant = TenantId::try_new(200).expect("valid requester_tenant");

    let secret_payload = String::from("top secret legal advice");
    let scoped = TenantScoped::new(owner_tenant, secret_payload);

    let result = scoped.into_inner_checked(&requester_tenant);
    assert!(result.is_err());

    let err = result.expect_err("should be mismatch error");
    match err {
        TenantScopeViolation::Mismatch { expected, actual } => {
            assert_eq!(expected, requester_tenant);
            assert_eq!(actual, owner_tenant);
        }
    }

    // Verify error display string contains tenant IDs but no inner value leak
    let err_msg = err.to_string();
    assert!(err_msg.contains("TenantId(200)"));
    assert!(err_msg.contains("TenantId(100)"));
    assert!(!err_msg.contains("top secret legal advice"));
}

#[test]
fn test_map_preserves_tenant_id() {
    let tenant_id = TenantId::try_new(777).expect("valid tenant_id");
    let scoped_number = TenantScoped::new(tenant_id, 21);

    let scoped_transformed = scoped_number.map(|x| x * 2);

    assert_eq!(scoped_transformed.tenant_id(), &tenant_id);

    let unpacked = scoped_transformed.into_inner_checked(&tenant_id);
    assert_eq!(unpacked, Ok(42));
}
