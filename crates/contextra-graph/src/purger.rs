//! Compatibility re-export for [`crate::compat::purger`].

// FILE-CONTEXT
// STAND: 2026-11-15T12:00:00Z
// ZWECK: Neutrale Bereinigungs- und Wartungsroutinen.
// INVARIANTEN: Sichere physische Bereinigung ohne Seiteneffekte.
// HOTSPOTS: Keine kritischen Bottlenecks identifiziert
// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

#[cfg(feature = "ports-compat")]
pub use crate::compat::purger::*;

// SECTION Purger Core Implementation

// !SECTION Purger Core Implementation
