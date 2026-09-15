//! Observability module re-exports and lifecycle observability traits.

// FILE-CONTEXT
// STAND: 2026-09-15T00:00:00Z
// ZWECK: Re-Exports für Lifecycle- und Grounding-Observability-Traits (in lifecycle.rs definiert).
// INVARIANTEN: Abwärtskompatibilität für Importpfade unter memfuse_core::traits::observability.

pub use super::lifecycle::*;
