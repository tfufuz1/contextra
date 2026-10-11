// FILE-CONTEXT
// STAND: 2026-10-11
// ZWECK: Deletion Subsystem Modules (WP-1.1, C1).

pub mod footprint;
pub mod registry;

pub use footprint::{ErasureFootprint, KeyScope, StorageScope};
pub use registry::{
    DeletionScopeRef, EraseUnit, Footprint, KeyFamilyEntry, KeyFamilyRegistry, PurgeScopeTarget,
    ALL_KEY_FAMILIES,
};
