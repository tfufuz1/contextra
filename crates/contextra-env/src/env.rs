//! Environment abstraction ports (Clock, Rng, IdGen, MetricsSink).
//!
//! Provides injected dependencies for deterministic execution, simulation,
//! and metric reporting across Contextra subsystems (Contract M4).

// =========================================================================
// SECTION: Aggregated Port Re-exports
// ANCHOR[id=env-root,epic=env-core,seq=3]
// NOTE: Bündelt alle Kernports des Environment-Ökosystems für saubere Entkopplung.
// LINK crates/contextra-env/SPEC.md#1-zweck
// =========================================================================

pub use crate::clock::{Clock, SystemClock};
pub use crate::id_gen::{IdGen, SequentialIdGen};
pub use crate::metrics::{
    HistogramSnapshot, InMemoryMetricsSink, MetricKey, MetricSnapshot, MetricsSink, NoopMetricsSink,
};
pub use crate::rng::{Rng, SeededRng};

// !SECTION: Aggregated Port Re-exports

// =========================================================================
// SECTION: Prelude Module
// ANCHOR[id=env-prelude,epic=env-core,seq=4]
// NOTE: Ergonomischer Wildcard-Import (`use contextra_env::env::prelude::*;`).
// =========================================================================

/// Prelude for environment ports.
///
/// Exports the four essential port traits required to build injectable components:
/// - [`Clock`]: Time operations
/// - [`Rng`]: Pseudorandom number generation
/// - [`IdGen`]: Monotonic and unique ID generation
/// - [`MetricsSink`]: Observability and telemetry
pub mod prelude {
    pub use super::{Clock, IdGen, MetricsSink, Rng};
}

// !SECTION: Prelude Module
