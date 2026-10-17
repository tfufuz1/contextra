// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Shared Test-Harness Modul fuer Loeschtests (Wave 2 & Welle 2+).
// INVARIANTEN: #![forbid(unsafe_code)], Ring-konform, gemeinsame Basis fuer REG-15 & §11.2.

pub mod canary;
pub mod failpoints;
pub mod raw_sweep;
pub mod reference_store;
pub mod tenant_matrix;

pub use canary::{
    CanaryEncoding, CanaryFieldType, CanaryNeedleInfo, CanarySet,
};
#[cfg(feature = "fault-injection")]
pub use failpoints::FailpointHelper;

pub use raw_sweep::{raw_sweep, SweepHit, SweepReport};
pub use reference_store::{KeyState, ReferenceStore};
pub use tenant_matrix::{TenantMatrixContext, TenantMatrixMode, TenantMatrixRunner};
