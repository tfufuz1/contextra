//! Memory budget for contextra-store (A2 of the decoupling plan).
//!
//! Canonical definition lives in `contextra-types`. The store re-exports it so
//! that existing `crate::budget::*` paths keep working.

pub use contextra_types::types::budget::{ResourceBudget, ResourceTracker};
