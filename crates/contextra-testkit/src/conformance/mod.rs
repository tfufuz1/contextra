//! # Konformitätsprüfsuiten für Contextra-Port-Traits (`conformance`)
//!
//! Dieses Modul stellt wiederverwendbare, vertragliche Prüfsuiten (Contract Verification Suites)
//! für alle zentralen Adapter-Implementierungen von Contextra-Ports bereit.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Regel 1 (SPEC §3.2):** Jeder Adapter muss die semantischen Verträge der Ports vollständig einhalten.
//! - **Regel 2 (SPEC §3.2):** Die Prüfsuiten panicken selbst nicht, sondern liefern ein typisiertes
//!   `ConformanceResult`. Erst der Testläufer wandelt Fehlschläge bei Bedarf in Panics um
//!   (z. B. mit `assert_conformance!`).
//!
//! ## Funktionsweise
//! Jede Konformitätsfunktion akzeptiert entweder eine Referenz auf einen Adapter oder eine
//! Factory-Closure (für asynchrone Neuerstellung im sauberen Zustand). Die Suite führt eine
//! Batterie strenger Tests durch (z. B. Rollback-Isolation, Randfälle, Dimensionen) und sammelt
//! Fehler deterministisch in `ConformanceFailure`.
//!
//! # Anwendungsbeispiel
//!
//! ```ignore
//! #[cfg(test)]
//! mod tests {
//!     use contextra_testkit::assert_conformance;
//!     use contextra_testkit::conformance::*;
//!
//!     #[tokio::test]
//!     async fn test_my_storage_conformance() {
//!         let res = run_storage_engine_suite(|| async { MyStorageEngine::new() }).await;
//!         assert_conformance!(res);
//!     }
//! }
//! ```

// ANCHOR[contextra-testkit:CONF-MOD-CLOCK-001]
/// Konformitätssuite für den `Clock`-Port.
pub mod clock;

// ANCHOR[contextra-testkit:CONF-MOD-FAILURE-001]
/// Typen zur detaillierten Erfassung von Konformitätsfehlschlägen.
pub mod failure;

// ANCHOR[contextra-testkit:CONF-MOD-GRAPH-001]
/// Konformitätssuite für den `GraphIndex`-Port.
pub mod graph;

// ANCHOR[contextra-testkit:CONF-MOD-MACROS-001]
/// Hilfsmakros für Konformitätsprüfungen (`assert_conformance!`).
pub mod macros;

// ANCHOR[contextra-testkit:CONF-MOD-METRICS-001]
/// Konformitätssuite für den `MetricsSink`-Port.
pub mod metrics;

// ANCHOR[contextra-testkit:CONF-MOD-RNG-001]
/// Konformitätssuite für den Zufallszahlen-Port (`Rng`).
pub mod rng;

// ANCHOR[contextra-testkit:CONF-MOD-STORAGE-001]
/// Konformitätssuite für den `StorageEngine`-Port.
pub mod storage;

// ANCHOR[contextra-testkit:CONF-MOD-TEXT-001]
/// Konformitätssuite für den `TextIndex`-Port.
pub mod text;

// ANCHOR[contextra-testkit:CONF-MOD-VECTOR-001]
/// Konformitätssuite für den `VectorIndex`-Port.
pub mod vector;

// Re-Exports für bequemen Zugriff
pub use clock::run_clock_suite;
pub use failure::{ConformanceFailure, ConformanceResult};
pub use graph::run_graph_index_suite;
pub use metrics::run_metrics_sink_suite;
pub use rng::run_rng_suite;
pub use storage::run_storage_engine_suite;
pub use text::run_text_index_suite;
pub use vector::run_vector_index_suite;
