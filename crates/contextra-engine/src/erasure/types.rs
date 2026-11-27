//! Public result types of the engine-level erase API (Spezifikation §12).

use contextra_types::{ErasureId, ErasureState, Guarantee};

/// Handle returned by `Contextra::erase`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EraseHandle {
    /// Job identifier; use with `Contextra::erase_status`.
    pub id: ErasureId,
    /// Guarantee reached at the time of return (G1 at least, G3 for `AwaitLevel::Purged`).
    pub reached: Guarantee,
    /// State of the job at the time of return.
    pub state: ErasureState,
}

/// Counts-only summary of an erase plan. Never contains keys (I-12).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ErasePlanSummary {
    /// Number of key rules in the resolved scope.
    pub rules: u32,
    /// Number of explicitly named documents (0 for collection/tenant scopes).
    pub docs: u64,
    /// Graph objects without provenance.
    pub unprovenanced: u64,
    /// Registered layers that participate in the plan.
    pub layers: Vec<String>,
    /// Estimated bytes to rewrite (0 if unknown).
    pub est_bytes: u64,
    /// Checkpoints that would be tainted.
    pub checkpoints: u32,
}
