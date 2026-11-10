//! Compatibility bridge for `contextra-graph` (SPEC-v2 §4.3, §6.2).
//!
//! Exposes ports- and types-dependent implementations behind the compat features.

#[cfg(feature = "types-compat")]
pub mod error;

#[cfg(any(feature = "types-compat", feature = "ports-compat"))]
pub mod provider;

#[cfg(feature = "ports-compat")]
pub mod purger;

#[cfg(feature = "ports-compat")]
pub mod graph_index;
