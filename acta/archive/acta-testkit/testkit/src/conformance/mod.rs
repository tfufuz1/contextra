//! # Konformitätsprüfsuiten für Acta-Port-Traits (`conformance`)
//!
//! Dieses Modul stellt wiederverwendbare, vertragliche Prüfsuiten (Contract Verification Suites)
//! für alle zentralen Adapter-Implementierungen von Acta-Ports bereit.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Regel 1:** Jeder Adapter muss die semantischen Verträge der Ports vollständig einhalten.
//! - **Regel 2:** Die Prüfsuiten panicken selbst nicht, sondern liefern ein typisiertes
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
//!     use acta_testkit::assert_conformance;
//!     use acta_testkit::conformance::*;
//!
//!     #[tokio::test]
//!     async fn test_my_storage_conformance() {
//!         let res = run_storage_engine_suite(|| async { MyStorageEngine::new() }).await;
//!         assert_conformance!(res);
//!     }
//! }
//! ```

/// Konformitätssuite für den `Clock`-Port.
pub mod clock;

/// Typen zur detaillierten Erfassung von Konformitätsfehlschlägen.
pub mod failure;

/// Konformitätssuite für den `GraphIndex`-Port.
pub mod graph;

/// Hilfsmakros für Konformitätsprüfungen (`assert_conformance!`).
pub mod macros;

/// Konformitätssuite für den `MetricsSink`-Port.
pub mod metrics;

/// Konformitätssuite für den Zufallszahlen-Port (`Rng`).
pub mod rng;

/// Konformitätssuite für den `StorageEngine`-Port.
pub mod storage;

/// Konformitätssuite für den `TextIndex`-Port.
pub mod text;

/// Konformitätssuite für den `VectorIndex`-Port.
pub mod vector;

/// Konformitätssuite für den `Vfs`-Port.
pub mod vfs;

/// Konformitaetssuiten fuer KV-Ports.
pub mod kv_ports;

/// Konformitaetssuiten fuer Routing-Ports.
pub mod routing_ports;

// Re-Exports für bequemen Zugriff
pub use clock::run_clock_suite;
pub use failure::{ConformanceFailure, ConformanceResult};
pub use graph::run_graph_index_suite;
pub use metrics::run_metrics_sink_suite;
pub use rng::run_rng_suite;
pub use storage::run_storage_engine_suite;
pub use text::run_text_index_suite;
pub use vector::run_vector_index_suite;
pub use vfs::run_vfs_suite;
pub use kv_ports::{
    assert_attention_exporter_conformance, assert_kv_bridge_storage_conformance,
    assert_kv_prefix_store_conformance, run_attention_exporter_suite,
    run_kv_bridge_storage_suite, run_kv_prefix_store_suite,
};
pub use routing_ports::{
    assert_community_resolver_conformance, assert_context_preparer_conformance,
    assert_drift_status_provider_conformance, assert_egress_policy_conformance,
    assert_hybrid_search_provider_conformance, run_community_resolver_suite,
    run_context_preparer_suite, run_drift_status_provider_suite, run_egress_policy_suite,
    run_hybrid_search_provider_suite,
};
