//! # Kontext-Kompaktierung (`context_compaction`)
//!
//! Dieses Submodul stellt Mechanismen zur Verdichtung langer Kontextverläufe
//! und veralteter Werkzeugausgaben bereit, um das Token-Budget im LLM-Kontextfenster
//! optimal auszunutzen.
//!
//! ## Ring-Zugehörigkeit
//! Teil der Kognitionsschicht (Ring 3). Verarbeitet Chunks aus `contextra-types`.
//!
//! ## Kernbestandteile
//! - [`ContextCompactor`]: Führt die kontextuelle Verdichtung anhand konfigurierter Strategien aus.
//! - [`ConsolidationSession`]: Verwaltet eine transaktionale Kompaktierungssitzung mit Status-Token.
//! - [`cleanup_orphaned_consolidation_intents`]: Bereinigt verwaiste Kompaktierungs-Absichten (Intents).
//! - [`CompactionStrategy`], [`StatusToken`], [`CompactedContext`]: Daten- und Statusmodelle.
//!
//! ## Invarianten
//! - **Provenienzerhalt:** Chunks behalten stets ihre Herkunfts-Dokument-IDs.
//! - **Deterministischer Fallback:** Bei Ausfall von LLMs greifen Truncate- und Heuristikstrategien.

// FILE-CONTEXT
// ZWECK: Kontextkompaktierung und Zusammenfassung langer Gesprächs- und Dokumentverläufe.
// INVARIANTEN: Provenance-Erhalt via Token-Budgeting; Rückfall auf Truncate/Summarize bei LLM-Ausfall.
// NICHT-OFFENSICHTLICH: StatusToken ermöglicht feingranulare Verfolgung des Kompaktierungszustands.
// STAND: TS:2026-08-29T17:22:29Z (SESSION: 0dcb9f3b)

// contextra-db/src/context_compaction.rs
// Context Compaction Engine (Contextra Context-Window Compaction)

//! Context Compaction Engine (Contextra Context-Window Compaction)
//!
//! Replaces stale tool outputs and long conversation histories with compact status tokens
//! to preserve the LLM context window.

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
