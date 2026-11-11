//! Error types for Thresholded Local Hyper-Flow Diffusion (TL-HFD).

// FILE-CONTEXT
// STAND: 2026-11-15T12:00:00Z
// ZWECK: Topologisches Lovász-High-Order-Diffusionsverfahren.
// INVARIANTEN: Submodulare Optimierung, schattenmodus-fähig.
// HOTSPOTS: Keine kritischen Bottlenecks identifiziert
// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

use thiserror::Error;

/// Error type for TL-HFD operations and parameter validations.
#[derive(Error, Debug, Clone, PartialEq)]
// ANCHOR[contextra-graph:SRC-TL-HFD-ERROR-001]

// SECTION Error Core Implementation
pub enum TlHfdError {
    /// Invalid parameter value provided to TL-HFD.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Non-finite value encountered during diffusion calculation.
    #[error("Non-finite value encountered during diffusion calculation")]
    NonFinite,
}

#[cfg(feature = "types-compat")]
impl From<TlHfdError> for contextra_types::ContextraError {
    fn from(err: TlHfdError) -> Self {
        contextra_types::ContextraError::InvalidInput(err.to_string())
    }
}

// !SECTION Error Core Implementation
