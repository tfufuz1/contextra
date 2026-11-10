//! Kompatibilitätsbrücken für `contextra-ports` und `contextra-types`.
//!
//! Wird nur kompiliert, wenn das Feature `ports-compat` aktiviert ist.

pub mod error;
pub mod types;

#[cfg(feature = "client")]
pub use crate::client::trait_impls;
