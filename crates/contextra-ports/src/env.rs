//! Environment abstraction ports (Clock, Rng, IdGen, MetricsSink).
//!
//! Provides injected dependencies for deterministic execution, simulation,
//! and metric reporting across Contextra subsystems (Contract M4).

pub use crate::clock::{Clock, SystemClock};
pub use crate::id_gen::{IdGen, SequentialIdGen};
pub use crate::metrics::{
    HistogramSnapshot, InMemoryMetricsSink, MetricKey, MetricSnapshot, MetricsSink, NoopMetricsSink,
};
pub use crate::rng::{Rng, SeededRng};

/// Prelude for environment ports.
pub mod prelude {
    pub use super::{Clock, IdGen, MetricsSink, Rng};
}
