//! Integration layer for Contextra's storage, retrieval, lineage, and erasure paths.
//!
//! This crate owns orchestration only. Persistence, index algorithms, cryptography,
//! and the erasure state machine remain in their owning crates.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod lineage;
mod erasure;
mod storage;

pub use erasure::ErasureWiring;
pub use lineage::{DerivedRef, LineageIndex, LineageRef, NonEmpty, SourceId, SourceRef};
pub use storage::{Document, GateAudit, HybridCandidate, TenantScopedStorage};
