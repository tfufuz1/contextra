//! Abwärtskompatibilitäts- und Speicherbindungsbrücke für contextra-agent (SPEC-v2 §6.3, S-63).

// =========================================================================
// SECTION: Storage Bridges
// =========================================================================
// ANCHOR[contextra-agent:compat-mod]


pub mod store_adapter;
pub use store_adapter::*;

// !SECTION
