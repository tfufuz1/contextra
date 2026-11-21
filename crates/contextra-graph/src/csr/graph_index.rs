//! Compatibility re-export of `graph_index` from `crate::compat::graph_index`.

// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

// TODO(baukasten): CSR-Graphkern von contextra-ports entkoppeln (AD-3/AD-6). Graph-Repräsentation und CSR-Speicher müssen ohne Port-Abhängigkeit im Kern nutzbar sein.
#[allow(unused_imports)]
#[cfg(feature = "ports-compat")]
pub use crate::compat::graph_index;

// SECTION Graph Index Core Implementation

// !SECTION Graph Index Core Implementation
