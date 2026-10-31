//! Stubs und Platzhalter fuer experimentelle Chunking-Strategien.
//!
//! # Zweck & Architektur
//! Gehoert zur Segmentierungsschicht (Ring 4) in `contextra-ingest`.
//! Zerlegt die hierarchische `DocumentIr` in semantisch zusammenhaengende,
//! budgetierte Einheiten (Chunks) fuer Embeddings und Volltext-Indexierung.
//!
// FILE-CONTEXT
// ZWECK: Stubs und Anbindung spezifischer Chunking-Strategien (WP M3/M4 / WP-ING-15).
// INVARIANTEN: Zero-Panic, Delegierung an Strategie-Modul oder FallbackChunker.
// STAND: TS:2026-10-31T00:00:00Z

use crate::canonical::CanonicalText;
use crate::chunking::config::ChunkingConfig;
use crate::chunking::fallback::FallbackChunker;
use crate::chunking::{ChunkDraft, IrChunker};
use crate::error::IngestError;
use crate::format::ChunkStrategy;
use crate::ir::DocumentIr;

#[path = "prose/mod.rs"]
pub mod prose_impl;

pub use prose_impl::*;

#[derive(Debug, Default)]
// ANCHOR[contextra-ingest:CHK-0236]
// struct `ProseChunker` fuer Stubs und Platzhalter fuer experimentelle Chunking-Strategien.
pub struct ProseChunker;

impl IrChunker for ProseChunker {
    fn strategy(&self) -> ChunkStrategy {
        ChunkStrategy::Prose
    }

    fn draft(
        &self,
        ir: &DocumentIr,
        canon: &CanonicalText,
        cfg: &ChunkingConfig,
    ) -> Result<Vec<ChunkDraft>, IngestError> {
        prose_impl::draft_prose(ir, canon, cfg)
    }
}

#[path = "tabular/mod.rs"]
mod tabular_impl;

#[derive(Debug, Default)]
// ANCHOR[contextra-ingest:CHK-0237]
// struct `TabularChunker` fuer Stubs und Platzhalter fuer experimentelle Chunking-Strategien.
pub struct TabularChunker;

impl IrChunker for TabularChunker {
    fn strategy(&self) -> ChunkStrategy {
        ChunkStrategy::Tabular
    }

    fn draft(
        &self,
        ir: &DocumentIr,
        canon: &CanonicalText,
        cfg: &ChunkingConfig,
    ) -> Result<Vec<ChunkDraft>, IngestError> {
        tabular_impl::draft_tabular(ir, canon, cfg)
    }
}

#[path = "email/mod.rs"]
mod email_impl;

pub use email_impl::EmailChunker;

#[path = "structured/mod.rs"]
mod structured_impl;

pub use structured_impl::StructuredChunker;

#[derive(Debug, Default)]
// ANCHOR[contextra-ingest:CHK-0238]
// struct `PagedChunker` fuer Stubs und Platzhalter fuer experimentelle Chunking-Strategien.
pub struct PagedChunker;

impl IrChunker for PagedChunker {
    fn strategy(&self) -> ChunkStrategy {
        ChunkStrategy::Paged
    }

    fn draft(
        &self,
        ir: &DocumentIr,
        canon: &CanonicalText,
        cfg: &ChunkingConfig,
    ) -> Result<Vec<ChunkDraft>, IngestError> {
        paged_impl::draft_paged(ir, canon, cfg)
    }
}
