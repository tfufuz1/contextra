//! Canonical environment abstractions for Acta (Clock, Rng, IdGen, Metrics).
//!
//! # Architecture Role
//! Zero internal workspace dependencies (E0 Foundation Micro-Core).
//! Pure standard library environment contracts and deterministic test doubles.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// =========================================================================
// SECTION: Module Declarations & Core Architecture
// ANCHOR[id=env-modules,epic=env-core,seq=1]
// NOTE: Alle Submodule sind autark und verwenden ausschließlich die Rust-Standardbibliothek.
// LINK crates/acta-env/SPEC.md#4-bindender-vertrag-42-s
// =========================================================================

/// Port trait and default system implementations for wall-clock and monotonic time.
/// LINK clock.rs#clock-root
pub mod clock;

/// Aggregated environment port definitions and prelude re-exports.
/// LINK env.rs#env-root
pub mod env;

/// Port trait and thread-safe atomic unique ID generator.
/// LINK id_gen.rs#idgen-root
pub mod id_gen;

/// Observability port trait and in-memory test sink for metrics collection.
/// LINK metrics.rs#metrics-root
pub mod metrics;

/// Pseudorandom number generator contract and deterministic SplitMix64 generator.
/// LINK rng.rs#rng-root
pub mod rng;

/// Controllable manual clock implementation for testing determinism.
/// Gated behind optional `test-utils` feature (SPEC K13).
/// LINK manual_clock.rs#manual-clock-root
#[cfg(feature = "test-utils")]
pub mod manual_clock;

// !SECTION: Module Declarations & Core Architecture

// =========================================================================
// SECTION: Public Re-exports & Convenience API
// ANCHOR[id=env-reexports,epic=env-core,seq=2]
// NOTE: Bietet direkten Zugriff auf alle kanonischen Typen an der Crate-Wurzel.
// =========================================================================

pub use clock::{Clock, SystemClock};
pub use env::*;
pub use id_gen::{IdGen, SequentialIdGen};
pub use metrics::{
    HistogramSnapshot, InMemoryMetricsSink, MetricKey, MetricSnapshot, MetricsSink, NoopMetricsSink,
};
pub use rng::{Rng, SeededRng};

#[cfg(feature = "test-utils")]
pub use manual_clock::ManualClock;

// =========================================================================
// SECTION: Time Utility Functions
// ANCHOR[id=env-time-utils,epic=clock,seq=1]
// NOTE: Verbindliche Hilfsfunktion gemäß SPEC-v2 §4.2 / B02.
// =========================================================================

/// Converts clock time to unix milliseconds by rounding down (SPEC-v2 §4.2, B02).
///
/// Ensures deterministic millisecond calculation across legacy sub-crates
/// (`text`, `graph`, `vector`) by enforcing standard truncation integer division.
///
/// # Fehler
/// Keine; garantiert panikfrei, da [`Clock::now_unix_nanos`] stets einen `u64` liefert.
///
/// # Invarianten
/// - `INV-ENV-TIME-1`: Bei Zeiten vor der Epoche liefert `now_unix_nanos()` 0, somit auch `unix_ms()` 0.
#[inline]
pub fn unix_ms(c: &dyn Clock) -> u64 {
    // NOTE: Abrundende Ganzzahldivision von Nanosekunden zu Millisekunden (/ 1_000_000).
    c.now_unix_nanos() / 1_000_000
}

// !SECTION: Time Utility Functions
