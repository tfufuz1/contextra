//! # contextra-engine (Deprecated Façade)
//!
//! Dieses Crate wurde gemäß SPEC(11).md S-63/S-64 und ADR-0122 aufgelöst.
//! Alle Funktionalitäten sind nach `contextra-db` migriert.
//
// INVARIANTE: INV-2: Name bleibt als #[deprecated]-Fassade erhalten.

#![deprecated(
    since = "0.2.0",
    note = "contextra-engine has been merged into contextra-db (ADR-0122)"
)]
#![forbid(unsafe_code)]

#[allow(deprecated)]
pub use contextra_db::*;
