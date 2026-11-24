// ANCHOR[id=contextra-mvcc-compat-root]
//! # Kompatibilitätsmodul für Contextra-Workspace
//!
//! Re-exportiert Adapter und Konvertierungen für bestehende Workspace-Verbraucher.
//! Dieses Modul wird ausschließlich hinter dem Feature `workspace` kompiliert.

/// Brücke zur Konvertierung lokaler `MvccError`-Fehler in `contextra_types::ContextraError`.
pub mod error;

pub use contextra_types::error as workspace_error;
pub use contextra_types::error::ContextraError;
pub use contextra_types::error::Result;
