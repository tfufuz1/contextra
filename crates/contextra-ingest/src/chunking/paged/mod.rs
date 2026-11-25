//! Seitenbasiertes Chunking fuer PDF- und Praesentationsdokumente.
//!
//! # Zweck & Architektur
//! Gehoert zur Segmentierungsschicht (Ring 4) in `contextra-ingest`.
//! Zerlegt die hierarchische `DocumentIr` in semantisch zusammenhaengende,
//! budgetierte Einheiten (Chunks) fuer Embeddings und Volltext-Indexierung.
//!
// FILE-CONTEXT
// ZWECK: PagedChunker für seitenbasierte Dokumente (PDF, PPTX, ODP, PPT) (WP-ING-14).
// INVARIANTEN: Zero-Panic, deterministisch, präzise Zitierbarkeit (page/slide locators), Spans monoton.
// STAND: TS:2026-10-31T00:00:00Z

pub mod page;

use crate::canonical::CanonicalText;
use crate::chunking::config::ChunkingConfig;
use crate::chunking::{ChunkDraft, IrChunker};
use crate::error::IngestError;
use crate::format::ChunkStrategy;
use crate::ir::DocumentIr;

/// Chunker für seitenbasierte Formate (PDF, Präsentationen).
#[derive(Debug, Default)]
// ANCHOR[contextra-ingest:CHK-0218]
// struct `PagedChunker` fuer Seitenbasiertes Chunking fuer PDF- und Praesentationsdokumente.
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
        draft_paged(ir, canon, cfg)
    }
}

/// Hauptfunktion für das Paged-Chunking.
// ANCHOR[contextra-ingest:CHK-0219]
// fn `draft_paged` fuer Seitenbasiertes Chunking fuer PDF- und Praesentationsdokumente.
pub fn draft_paged(
    ir: &DocumentIr,
    canon: &CanonicalText,
    cfg: &ChunkingConfig,
) -> Result<Vec<ChunkDraft>, IngestError> {
    page::process_paged_document(ir, canon, cfg)
}
