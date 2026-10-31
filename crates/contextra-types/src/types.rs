//! Core domain, budget, filter, importance, and search query data types.
//!
//! # Architektur-Rolle
//! Ring 0 (Kern). Bündelt Domänenmodelle, Ressourcenbudgets, Metadaten-Filter,
//! Recency-/Importance-Scoring und 4-Signal-Suchtypen.

// ANCHOR[contextra-types:TYPES-BUDGET]
/// Memory and token resource budget management types.
pub mod budget;
// ANCHOR[contextra-types:TYPES-DOMAIN]
/// Fundamental domain models (`DocId`, `TxId`, `Entity`, `DistanceMetric`, etc.).
pub mod domain;
// ANCHOR[contextra-types:TYPES-FILTER]
/// Structured metadata expression filter types.
pub mod filter;
// ANCHOR[contextra-types:TYPES-IMPORTANCE]
/// Memory importance and recency decay scoring types.
pub mod importance;
// ANCHOR[contextra-types:TYPES-SAOS]
/// Unified 4-signal search query and context types.
pub mod saos;

pub use budget::*;
pub use domain::*;
pub use filter::*;
pub use importance::*;
pub use saos::*;
