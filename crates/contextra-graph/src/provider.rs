//! Compatibility re-export for [`crate::compat::provider`].

// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

#[cfg(any(feature = "types-compat", feature = "ports-compat"))]
pub use crate::compat::provider::*;

// SECTION Provider Core Implementation

// !SECTION Provider Core Implementation
