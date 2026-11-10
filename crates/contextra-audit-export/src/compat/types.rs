//! Compatibility bridge adapters for `contextra-types`.

use crate::TenantId;

impl From<contextra_types::TenantId> for TenantId {
    #[inline]
    fn from(t: contextra_types::TenantId) -> Self {
        Self(t.as_u64())
    }
}

impl From<TenantId> for contextra_types::TenantId {
    #[inline]
    fn from(t: TenantId) -> Self {
        contextra_types::TenantId(t.0)
    }
}
