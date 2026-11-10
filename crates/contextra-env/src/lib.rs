//! Canonical environment abstractions for Contextra (Clock, Rng, IdGen, Metrics).
//!
//! # Architecture Role
//! Zero internal workspace dependencies. Pure standard library environment contracts.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod clock;
pub mod env;
pub mod id_gen;
pub mod metrics;
pub mod rng;

#[cfg(feature = "test-utils")]
pub mod manual_clock;

pub use clock::{Clock, SystemClock};
pub use env::*;
pub use id_gen::{IdGen, SequentialIdGen};
pub use metrics::{
    HistogramSnapshot, InMemoryMetricsSink, MetricKey, MetricSnapshot, MetricsSink, NoopMetricsSink,
};
pub use rng::{Rng, SeededRng};

#[cfg(feature = "test-utils")]
pub use manual_clock::ManualClock;

/// Converts clock time to unix milliseconds by rounding down (SPEC-v2 §4.2, B02).
#[inline]
pub fn unix_ms(c: &dyn Clock) -> u64 {
    c.now_unix_nanos() / 1_000_000
}
