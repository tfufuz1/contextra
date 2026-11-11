//! Compatibility bridge for `contextra-graph` (SPEC-v2 §4.3, §6.2).
//!
//! Exposes ports- and types-dependent implementations behind the compat features.

// FILE-CONTEXT
// STAND: 2026-11-15T12:00:00Z
// ZWECK: Kompatibilitätsbrücke für Ports und Types (SPEC-v2 §4.3).
// INVARIANTEN: Isolierte Workspace-Konvertierungen.
// HOTSPOTS: Keine kritischen Bottlenecks identifiziert
// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

#[cfg(feature = "types-compat")]

// SECTION Compat Core Implementation
pub mod error;

#[cfg(any(feature = "types-compat", feature = "ports-compat"))]
pub mod provider;

#[cfg(feature = "ports-compat")]
pub mod purger;

#[cfg(feature = "ports-compat")]
pub mod graph_index;

#[cfg(feature = "ports-compat")]
pub mod collection_mutation;

// !SECTION Compat Core Implementation
