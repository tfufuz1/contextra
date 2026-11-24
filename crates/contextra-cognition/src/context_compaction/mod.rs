//! # Kontext-Kompaktierung (`context_compaction`)
//!
//! Dieses Submodul stellt Mechanismen zur Verdichtung langer Kontextverläufe
//! und veralteter Werkzeugausgaben bereit, um das Token-Budget im LLM-Kontextfenster
//! optimal auszunutzen.
//!
//! ## Ebene und Einordnung
//! Teil der Kognitionsschicht (Ebene E2). Verarbeitet Chunks aus `contextra-types`.
//!
//! ## Kernbestandteile
//! - [`ContextCompactor`]: Führt die kontextuelle Verdichtung anhand konfigurierter Strategien aus.
//! - [`ConsolidationSession`]: Verwaltet eine transaktionale Kompaktierungssitzung mit Status-Token.
//! - [`cleanup_orphaned_consolidation_intents`]: Bereinigt verwaiste Kompaktierungs-Absichten (Intents).
//! - [`CompactionStrategy`], [`StatusToken`], [`CompactedContext`]: Daten- und Statusmodelle.
//!
//! ## Invarianten
//! - **Provenienzerhalt:** Chunks behalten stets ihre Herkunfts-Dokument-IDs (`source_doc_ids`).
//! - **Deterministischer Fallback:** Bei Ausfall von LLMs greifen deterministische Truncate- und Status-Token-Strategien.
//
// FILE-CONTEXT
// ZWECK: Kontextkompaktierung und Zusammenfassung langer Gesprächs- und Dokumentverläufe.
// INVARIANTEN: Provenance-Erhalt via Token-Budgeting; Rückfall auf Truncate/Summarize bei LLM-Ausfall.
// STAND: 2026-11-11

// =========================================================================
// SECTION: Submodule-Deklaration
// =========================================================================

// ANCHOR[contextra-cognition:COMPACT-MOD-001]
mod cleanup;
mod compactor;
mod session;
mod types;

#[cfg(test)]
mod tests;

pub use cleanup::cleanup_orphaned_consolidation_intents;
pub use compactor::ContextCompactor;
pub use session::ConsolidationSession;
pub use types::{CompactedContext, CompactionStrategy, StatusToken};
// !SECTION: Submodule-Deklaration
