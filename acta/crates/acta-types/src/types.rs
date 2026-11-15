//! Kern-Domänen-, Budget-, Filter-, Relevanz- und Suchabfragetypen.
//!
//! # Architektur-Rolle: Vertragsebene (contracts)
//!
//! Dieses Modul bündelt alle strukturellen Datentypen des Gesamtsystems:
//! - `domain`: Fundamentale Identifikatoren (`DocId`, `TxId`, `TenantId`) und Wissensgraphen-Elemente.
//! - `budget`: Token- und Ressourcen-Budgetierung für Inferenz und Abfrageverarbeitung.
//! - `filter`: Strukturierte Metadaten-Filterausdrücke mit logischen Operatoren.
//! - `importance`: Relevanzbewertung und zeitlicher Verfall (Decay) von Erinnerungen.
//! - `saos`: Einheitliche 4-Signal-Suchabfragestrukturen (Semantik, lexikalischer Text, Graph, Recency).

// ANCHOR[acta-types:TYPES-BUDGET]
/// Memory and token resource budget management types.
// ANCHOR[acta-types:MOD-BUDGET]
pub mod budget;
// ANCHOR[acta-types:TYPES-DOMAIN]
/// Fundamental domain models (`DocId`, `TxId`, `Entity`, `DistanceMetric`, etc.).
// ANCHOR[acta-types:MOD-DOMAIN]
pub mod domain;
// ANCHOR[acta-types:TYPES-FILTER]
/// Structured metadata expression filter types.
// ANCHOR[acta-types:MOD-FILTER]
pub mod filter;
// ANCHOR[acta-types:TYPES-IMPORTANCE]
/// Memory importance and recency decay scoring types.
// ANCHOR[acta-types:MOD-IMPORTANCE]
pub mod importance;
// ANCHOR[acta-types:TYPES-SAOS]
/// Unified 4-signal search query and context types.
// ANCHOR[acta-types:MOD-SAOS]
pub mod saos;

pub use budget::*;
pub use domain::*;
pub use filter::*;
pub use importance::*;
pub use saos::*;
