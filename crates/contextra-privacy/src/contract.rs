//! `contract` — Minimale Umwelt- und Port-Traits für `contextra-privacy`.
//!
//! # Zweck
//! Definiert die Kernabstraktionen für Zeit, Richtlinienauswertung, Volltext-/Vektorsuche
//! und Entitätserkennung. Das Crate bleibt vollständig autark; externe Frameworks
//! oder Ports können diese Traits über standardkonforme Adapter implementieren.
//!
//! # Invarianten
//! - **Null-Workspace-Kopplung**: Keine Abhängigkeiten zu `contextra-ports` oder `contextra-types` im Kern.
//! - **Fail-Closed**: Trait-Verträge fordern deterministisches Fehler- und Blockierverhalten.

// ANCHOR[contextra-privacy:CONTRACT-001]
// Re-Exports der kanonischen Trait-Definitionen des Crates
pub use crate::egress_guard::TextSearchEngine;
pub use crate::egress_vault::EntityRecognizer;
pub use crate::types::{Clock, EgressDecision, EgressPolicy};
