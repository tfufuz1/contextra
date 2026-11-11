//! Kompatibilitätsbrücken für `contextra-ports` und `contextra-types`.
//!
//! Wird nur kompiliert, wenn das Feature `workspace` aktiviert ist.

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub mod embedding;
pub mod error;
#[cfg(feature = "client")]
pub mod query_rewriter;
#[cfg(feature = "client")]
pub mod trait_impls;
pub mod types;

#[cfg(all(test, feature = "client"))]
mod client_tests;
