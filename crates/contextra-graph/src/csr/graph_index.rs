//! Compatibility re-export of `graph_index` from `crate::compat::graph_index`.

// FILE-CONTEXT
// STAND: 2026-11-15T12:00:00Z
// ZWECK: Compressed Sparse Row (CSR) Graph-Engine Kern.
// INVARIANTEN: Bitemporale Sichtbarkeit, lock-freie Lesepfade.
// HOTSPOTS: Keine kritischen Bottlenecks identifiziert
// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

// TODO(baukasten): CSR-Graphkern von contextra-ports entkoppeln (AD-3/AD-6). Graph-Repräsentation und CSR-Speicher müssen ohne Port-Abhängigkeit im Kern nutzbar sein.
#[allow(unused_imports)]
#[cfg(feature = "ports-compat")]
pub use crate::compat::graph_index;

// SECTION Graph Index Core Implementation

// !SECTION Graph Index Core Implementation
