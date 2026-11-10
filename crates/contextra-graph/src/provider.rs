//! Compatibility re-export for [`crate::compat::provider`].

#[cfg(any(feature = "types-compat", feature = "ports-compat"))]
pub use crate::compat::provider::*;
