//! Adapter bridge for `contextra-privacy` types to `contextra-types` and `contextra-ports`.

use contextra_types::TenantId;

/// Converts a raw tenant ID integer into the canonical [`TenantId`].
pub fn to_canonical_tenant_id(raw: u64) -> TenantId {
    TenantId::new(raw)
}
