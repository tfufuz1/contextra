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

#[path = "structured/mod.rs"]
pub mod structured_impl;

#[path = "email/mod.rs"]
pub mod email_impl;

pub use email_impl::EmailChunker;
pub use structured_impl::StructuredChunker;

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
        FallbackChunker.draft(ir, canon, cfg)
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
