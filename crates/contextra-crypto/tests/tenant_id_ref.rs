// Herkunfts-Commit: ca103a3c743df21a70c3b0b0e2b3972a693a5ca4
//! Referenz-Differenztest: TenantId aus contextra-ids vs historische Validierung.

use contextra_ids::TenantId;

#[test]
fn test_tenant_id_canonical_properties() {
    let t = TenantId::try_new(42).expect("valid tenant");
    assert_eq!(t.inner(), 42);
    assert_eq!(format!("{t:?}"), "TenantId(42)");

    // Zero-Tenant is system reserved in try_new
    let t0 = TenantId::try_new(0);
    assert!(
        t0.is_err(),
        "TenantId 0 returns IdError::TenantSystemReserved"
    );
    assert_eq!(TenantId::SYSTEM.inner(), 0);

    // U64 max boundary
    let t_max = TenantId::try_new(u64::MAX);
    assert!(t_max.is_ok());
    assert_eq!(t_max.unwrap().inner(), u64::MAX);
}
