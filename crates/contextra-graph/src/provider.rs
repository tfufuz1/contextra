//! Compatibility re-export for [`crate::compat::provider`].

// FILE-CONTEXT
// STAND: 2026-11-15T12:00:00Z
// ZWECK: Neutrale Provider-Abstraktionen für Embeddings und Zeit.
// INVARIANTEN: Entkoppelte Schnittstelle zu externen Modellen.
// HOTSPOTS: Keine kritischen Bottlenecks identifiziert
// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

#[cfg(any(feature = "types-compat", feature = "ports-compat"))]
pub use crate::compat::provider::*;

// SECTION Provider Core Implementation

// !SECTION Provider Core Implementation
