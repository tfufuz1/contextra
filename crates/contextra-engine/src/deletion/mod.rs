// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Deletion Subsystem Modules (WP-1.1, C1).

pub mod copy_observer;
pub mod environment;
pub mod footprint;
pub mod footprint_handle;
pub mod forget_options;
pub mod proof_issue;
pub mod registry;
pub mod relations_witness;
pub mod restore_filter;
pub mod reverse_sweep;
pub mod text_witness;

pub use footprint::{
    check_nonempty_or_observed_empty, ErasureFootprint, FprRamFootprint, KeyScope, StorageScope,
};
pub use registry::{
    DeletionScopeRef, EraseUnit, Footprint, KeyFamilyEntry, KeyFamilyRegistry, PurgeScopeTarget,
    ALL_KEY_FAMILIES,
};
pub use reverse_sweep::{
    reverse_sweep, FprNeedles, FprSweepReport, FprSweepSource, FprUnregisteredResidue, SweepDepth,
};
