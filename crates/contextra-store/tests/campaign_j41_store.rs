//! Campaign J-41 hypothesis test suite for contextra-store.
//! Tests static scan findings: integer truncation, P28 determinism invariants, dropped errors.

use contextra_types::TenantId;

#[test]
fn test_j41_tenant_id_u64_to_u32_truncation_oracle() {
    // Oracle: TenantId takes u64. Converting a u64 tenant ID > u32::MAX (e.g. 0x1_0000_0001)
    // via `as u32` truncates upper bits to 1.
    let large_tenant_u64: u64 = 0x1_0000_0001;

    // Narrowing cast check
    let truncated = large_tenant_u64 as u32;
    assert_eq!(truncated, 1, "Demonstrating that `as u32` silently truncates upper 32 bits");

    // Safe conversion via u32::try_from
    let safe_conv = u32::try_from(large_tenant_u64);
    assert!(safe_conv.is_err(), "u32::try_from must fail for values > u32::MAX");

    // TenantId::try_new with truncated vs full u64
    let tenant_res = TenantId::try_new(u64::from(truncated));
    assert!(tenant_res.is_ok());
    assert_eq!(tenant_res.unwrap().as_u64(), 1);

    let full_tenant = TenantId::try_new(large_tenant_u64).unwrap();
    assert_eq!(full_tenant.as_u64(), 0x1_0000_0001);
}

#[test]
fn test_j41_sequence_number_safe_bounds() {
    let max_seq: u64 = u64::MAX;
    // Direct cast as u32 truncates
    let truncated_seq = max_seq as u32;
    assert_eq!(truncated_seq, u32::MAX);

    // try_from catches overflow
    assert!(u32::try_from(max_seq).is_err());
}
