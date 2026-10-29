// FILE-CONTEXT
// ZWECK: Stubs für spezifische Chunking-Strategien (Prose, Tabular, Paged, Structured, Email) (WP M3/M4).
// INVARIANTEN: Zero-Panic, Delegierung an FallbackChunker.
// STAND: TS:2026-10-31T00:00:00Z

use crate::canonical::CanonicalText;
use crate::chunking::config::ChunkingConfig;
use crate::chunking::fallback::FallbackChunker;
use crate::chunking::{ChunkDraft, IrChunker};
use crate::error::IngestError;
use crate::format::ChunkStrategy;
use crate::ir::DocumentIr;

#[derive(Debug, Default)]
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
        FallbackChunker.draft(ir, canon, cfg)
    }
}

#[path = "tabular/mod.rs"]
mod tabular_impl;

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
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
        FallbackChunker.draft(ir, canon, cfg)
    }
}

#[derive(Debug, Default)]
pub struct StructuredChunker;

impl IrChunker for StructuredChunker {
    fn strategy(&self) -> ChunkStrategy {
        ChunkStrategy::Structured
    }

    fn draft(
        &self,
        ir: &DocumentIr,
        canon: &CanonicalText,
        cfg: &ChunkingConfig,
    ) -> Result<Vec<ChunkDraft>, IngestError> {
        FallbackChunker.draft(ir, canon, cfg)
    }
}

#[derive(Debug, Default)]
pub struct EmailChunker;

impl IrChunker for EmailChunker {
    fn strategy(&self) -> ChunkStrategy {
        ChunkStrategy::Email
    }

    fn draft(
        &self,
        ir: &DocumentIr,
        canon: &CanonicalText,
        cfg: &ChunkingConfig,
    ) -> Result<Vec<ChunkDraft>, IngestError> {
        FallbackChunker.draft(ir, canon, cfg)
    }
}
