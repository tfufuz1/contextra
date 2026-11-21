//! Canonical environment abstractions for Contextra (Clock, Rng, IdGen, Metrics).
//!
//! # Architecture Role
//! Internal workspace dependencies: `contextra-ids` (DocId, TOMBSTONE_BIT) for the
//! versioned sequence log `seq_log`. Otherwise standard library and `tracing` only.
//! Environment contracts and deterministic test doubles.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// =========================================================================
// SECTION: Module Declarations & Core Architecture
// NOTE: Alle Submodule sind autark und verwenden ausschließlich die Rust-Standardbibliothek.
// LINK docs/crates/contextra-env.md §2
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

/// Versioniertes Sequenz-Log für snapshot-isolierte Suchen (ehemals contextra-version).
pub mod seq_log;

/// Controllable manual clock implementation for testing determinism.
/// Gated behind optional `test-utils` feature (SPEC K13).
/// LINK manual_clock.rs#manual-clock-root
#[cfg(feature = "test-utils")]
pub mod manual_clock;

// !SECTION: Module Declarations & Core Architecture

// =========================================================================
// SECTION: Public Re-exports & Convenience API
// NOTE: Bietet direkten Zugriff auf alle kanonischen Typen an der Crate-Wurzel.
// =========================================================================

pub use clock::{Clock, SystemClock};
pub use env::*;
pub use id_gen::{IdGen, SequentialIdGen};
pub use metrics::{
    HistogramSnapshot, InMemoryMetricsSink, MetricKey, MetricSnapshot, MetricsSink, NoopMetricsSink,
};
pub use rng::{Rng, SeededRng};
pub use seq_log::{SeqLogChange, SeqLogEntry, SequenceLog};

#[cfg(feature = "test-utils")]
pub use manual_clock::ManualClock;

// =========================================================================
// SECTION: Time Utility Functions
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

// =========================================================================
// SECTION: Unit Tests
// NOTE: Prüft die Zeit-Hilfsfunktion und die Arc-Delegation ohne Test-Features.
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// Minimale feste Uhr, unabhängig vom `test-utils`-Feature.
    struct FixedClock(u64);

    impl Clock for FixedClock {
        fn now_unix_nanos(&self) -> u64 {
            self.0
        }

        fn monotonic_nanos(&self) -> u64 {
            self.0
        }
    }

    #[test]
    fn test_unix_ms_rounds_down() {
        // NOTE: Verifiziert SPEC-v2 §4.2 / B02 (abrundende Ganzzahldivision).
        assert_eq!(unix_ms(&FixedClock(0)), 0);
        assert_eq!(unix_ms(&FixedClock(999_999)), 0);
        assert_eq!(unix_ms(&FixedClock(1_000_000)), 1);
        assert_eq!(unix_ms(&FixedClock(1_000_001)), 1);
        assert_eq!(unix_ms(&FixedClock(1_500_000_000)), 1_500);
        assert_eq!(unix_ms(&FixedClock(u64::MAX)), u64::MAX / 1_000_000);
    }

    #[test]
    fn test_arc_clock_delegates() {
        // NOTE: Verifiziert INV-ENV-TIME-1/2 über die Arc-Delegation.
        let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
        let a = clock.monotonic_nanos();
        let b = clock.monotonic_nanos();
        assert!(b >= a);
        assert!(clock.now_unix_nanos() > 0);
        assert_eq!(unix_ms(&clock), clock.now_unix_nanos() / 1_000_000);
    }
}

// !SECTION: Unit Tests
