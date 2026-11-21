//! Error types for Thresholded Local Hyper-Flow Diffusion (TL-HFD).

// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

use thiserror::Error;

/// Error type for TL-HFD operations and parameter validations.
#[derive(Error, Debug, Clone, PartialEq)]

// SECTION Error Core Implementation
pub enum TlHfdError {
    /// Invalid parameter value provided to TL-HFD.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Non-finite value encountered during diffusion calculation.
    #[error("Non-finite value encountered during diffusion calculation")]
    NonFinite,
}

// !SECTION Error Core Implementation
