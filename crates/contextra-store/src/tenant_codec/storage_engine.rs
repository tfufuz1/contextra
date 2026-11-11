// MIGRATION(S-70): neuer Pfad crate::compat::engine::tenant_storage
#[cfg(feature = "workspace")]
pub use crate::compat::engine::tenant_storage::*;
