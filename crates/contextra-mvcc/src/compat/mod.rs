// ANCHOR[contextra-mvcc:COMPAT-ROOT]
//! # Kompatibilitätsmodul für Contextra-Workspace
//!
//! Re-exportiert Adapter und Konvertierungen für bestehende Workspace-Verbraucher.

/// Brücke zur Konvertierung lokaler `MvccError`-Fehler in `contextra_types::ContextraError`.
pub mod error;
