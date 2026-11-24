//! # Modul-Re-Exports und Submodul-Deklarationen für Contextra PyO3 Bindings
//!
//! Dieses Modul bündelt alle Submodule der Python-Schnittstelle und re-exportiert
//! die wichtigsten Typen und Funktionen direkt in den Namespace von `contextra_py`.
//!
//! ## Ring-Zugehörigkeit
//! Ring 4 (FFI-Boundary / Bindings).
//!
//! ## Enthaltene Komponenten
//! - `collection`: Definiert `PyCollection`, die Python-Schnittstelle zu Collections (Dokumentensammlungen).
//! - `common`: FFI-Hilfsfunktionen, Fehlerübersetzung (`contextra_err`), GIL-Management und FFI-Panikabsicherung.
//! - `crud_macros`: Makros zur einheitlichen Generierung von synchronen CRUD-Methoden (Create, Read, Update, Delete).
//! - `db`: `PyContextra`, das zentrale Datenbankhandle für Öffnen, Schließen und Verwalten von Collections.
//! - `db_stats`, `storage_stats`, `vector_index_stats`: Python-taugliche Statistik-Datenstrukturen.
//! - `document`, `search_result`: Rückgabetypen für gelesene Dokumente und Vektor- bzw. Hybridsuchergebnisse.
//! - `functions`: Top-Level Funktionen wie `open()` zur Datenbankinitialisierung.
//! - `hyperedge`: Python-Bindings für Hypergraph-Relationen und Rollenbindungen.
//! - `runtime_state`: Verwaltung der per-Interpreter isolierten Tokio-Laufzeitumgebung.
//! - `tests`: Unit- und Regressionstests für die Bindings.

#[macro_use]
pub mod crud_macros;

pub mod collection;
pub mod common;
pub mod db;
pub mod db_stats;
pub mod document;
pub mod functions;
pub mod hyperedge;
pub mod runtime_state;
pub mod search_result;
pub mod storage_stats;

#[cfg(test)]
mod tests;

pub mod vector_index_stats;

pub use collection::PyCollection;
pub use common::*;
pub use db::PyContextra;
pub use db_stats::PyDbStats;
pub use document::PyDocument;
pub use functions::*;
pub use runtime_state::*;
pub use search_result::PySearchResult;
pub use storage_stats::PyStorageStats;
pub use vector_index_stats::PyVectorIndexStats;
