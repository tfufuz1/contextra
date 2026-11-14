//! Fehlertyp fuer contextra-store (A4 des Entkopplungsplans).
//!
//! Der kanonische Fehlertyp ist `contextra_types::ContextraError`. Die
//! Konvertierungen aus `contextra-crypto` und `contextra-mvcc` liegen bereits in
//! deren `compat`-Modulen, daher gibt es hier keine eigenen `From`-Impls mehr.

/// Fehlertyp des LSM-Speicherkerns (kanonisch: `contextra_types::ContextraError`).
pub type StoreError = contextra_types::ContextraError;

/// Ergebnis-Typ fuer Speicheroperationen.
pub type StoreResult<T> = contextra_types::Result<T>;
