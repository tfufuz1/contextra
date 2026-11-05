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

// ANCHOR[contextra-testkit:CONF-MOD-CLOCK-001]
/// Konformitätssuite für den `Clock`-Port.
pub mod clock;

// ANCHOR[contextra-testkit:CONF-MOD-FAILURE-001]
/// Typen zur detaillierten Erfassung von Konformitätsfehlschlägen.
pub mod failure;

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

// ANCHOR[contextra-testkit:CONF-MOD-VFS-001]
/// Konformitätssuite für den `Vfs`-Port.
pub mod vfs;

/// Konformitätssuite für den `ReadGate`-Port.
pub mod read_gate;

/// Konformitätssuite für den `RecordSealer`-Port.
pub mod record_sealer;

/// Konformitätssuite für den `FileKeyring`-Port.
pub mod file_keyring;

/// Konformitätssuite für den `LedgerSink`-Port.
pub mod ledger_sink;

/// Konformitätssuite für den `TextIndex`-Port (erweitert).
pub mod text_index;

/// Konformitätssuite für den `VectorIndex`-Port (erweitert).
pub mod vector_index;

/// Konformitätssuite für den `ErasureLayer`-Port (SPEC §8.2, §11, PO-05).
pub mod erasure_layer;

// Re-Exports für bequemen Zugriff
pub use clock::run_clock_suite;
pub use erasure_layer::run as run_erasure_layer_suite;
pub use failure::{ConformanceFailure, ConformanceResult};
pub use file_keyring::run as run_file_keyring_suite;
pub use ledger_sink::run as run_ledger_sink_suite;
pub use metrics::run_metrics_sink_suite;
pub use read_gate::run as run_read_gate_suite;
pub use record_sealer::run as run_record_sealer_suite;
pub use rng::run_rng_suite;
pub use storage::run_storage_engine_suite;
pub use text::run_text_index_suite;
pub use vector::run_vector_index_suite;
pub use vfs::run_vfs_suite;
