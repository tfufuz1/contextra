//! Kern-Domänen-, Budget-, Filter-, Relevanz- und Suchabfragetypen.
//!
//! # Architektur-Rolle: Ring 0
//!
//! Dieses Modul bündelt alle strukturellen Datentypen des Gesamtsystems:
//! - `domain`: Fundamentale Identifikatoren (`DocId`, `TxId`, `TenantId`) und Wissensgraphen-Elemente.
//! - `budget`: Token- und Ressourcen-Budgetierung für Inferenz und Abfrageverarbeitung.
//! - `filter`: Strukturierte Metadaten-Filterausdrücke mit logischen Operatoren.
//! - `importance`: Relevanzbewertung und zeitlicher Verfall (Decay) von Erinnerungen.
//! - `saos`: Einheitliche 4-Signal-Suchabfragestrukturen (Semantik, BM25-Text, Graph, Recency).

/// Memory and token resource budget management types.
// ANCHOR[contextra-types:MOD-BUDGET]
pub mod budget;
/// Fundamental domain models (`DocId`, `TxId`, `Entity`, `DistanceMetric`, etc.).
// ANCHOR[contextra-types:MOD-DOMAIN]
pub mod domain;
/// Structured metadata expression filter types.
// ANCHOR[contextra-types:MOD-FILTER]
pub mod filter;
/// Memory importance and recency decay scoring types.
// ANCHOR[contextra-types:MOD-IMPORTANCE]
pub mod importance;
/// Unified 4-signal search query and context types.
// ANCHOR[contextra-types:MOD-SAOS]
pub mod saos;

pub use budget::*;
pub use domain::*;
pub use filter::*;
pub use importance::*;
pub use saos::*;
