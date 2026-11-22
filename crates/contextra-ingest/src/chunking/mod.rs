// FILE-CONTEXT
// ZWECK: Root-Modul für Chunking-Strategien und Infrastructure (WP M3).
// INVARIANTEN: Zero-Panic, deterministisch.
// STAND: TS:2026-10-31T00:00:00Z

pub mod common;
pub mod config;
pub mod fallback;
pub mod meta;
pub mod stubs;

pub mod email {
    pub use crate::chunking::stubs::EmailChunker;
}
pub mod paged {
    pub use crate::chunking::stubs::PagedChunker;
}
pub mod prose {
    pub use crate::chunking::stubs::ProseChunker;
}
pub mod structured {
    pub use crate::chunking::stubs::StructuredChunker;
}
pub mod tabular {
    pub use crate::chunking::stubs::TabularChunker;
}

use crate::canonical::CanonicalText;
use crate::chunking::config::ChunkingConfig;
use crate::error::IngestError;
use crate::format::ChunkStrategy;
use crate::ir::{DocumentIr, Locator};
use serde_json::{Map, Value};
use std::ops::Range;

/// Entwurf eines Chunks vor der Finalisierung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkDraft {
    pub text: String,
    pub span: Range<usize>,
    pub locator: Locator,
    pub breadcrumb: Option<String>,
    pub heading_level: Option<u8>,
    pub extra: Map<String, Value>,
}

/// Trait für strategie-spezifische IrChunker.
pub trait IrChunker: Send + Sync {
    fn strategy(&self) -> ChunkStrategy;
    fn draft(
        &self,
        ir: &DocumentIr,
        canon: &CanonicalText,
        cfg: &ChunkingConfig,
    ) -> Result<Vec<ChunkDraft>, IngestError>;
}

/// Erzeugt ChunkDrafts aus einer `DocumentIr` und deren `CanonicalText`.
pub fn draft_document(
    ir: &DocumentIr,
    canon: &CanonicalText,
    cfg: &ChunkingConfig,
) -> Result<Vec<ChunkDraft>, IngestError> {
    let strategy = cfg.strategy.unwrap_or_else(|| ir.format.default_strategy());

    let chunker: Box<dyn IrChunker> = match strategy {
        ChunkStrategy::Prose => Box::new(prose::ProseChunker),
        ChunkStrategy::Tabular => Box::new(tabular::TabularChunker),
        ChunkStrategy::Paged => Box::new(paged::PagedChunker),
        ChunkStrategy::Structured => Box::new(structured::StructuredChunker),
        ChunkStrategy::Email => Box::new(email::EmailChunker),
    };

    chunker.draft(ir, canon, cfg)
}

/// Erzeugt ChunkDrafts für einen Teilbereich von Blöcken einer `DocumentIr`.
pub fn draft_block_range(
    ir: &DocumentIr,
    canon: &CanonicalText,
    blocks_range: Range<usize>,
    strategy: ChunkStrategy,
    cfg: &ChunkingConfig,
) -> Result<Vec<ChunkDraft>, IngestError> {
    let mut sub_ir = DocumentIr::new(ir.format);
    sub_ir.meta = ir.meta.clone();

    for idx in blocks_range {
        if let Some(block) = ir.blocks.get(idx) {
            sub_ir.blocks.push(block.clone());
        }
    }

    let sub_canon = sub_ir.canonical();
    let mut sub_cfg = cfg.clone();
    sub_cfg.strategy = Some(strategy);

    let drafts = draft_document(&sub_ir, &sub_canon, &sub_cfg)?;

    // Span auf den Parent-Text zurückrechnen
    let mut mapped_drafts = Vec::with_capacity(drafts.len());
    for mut draft in drafts {
        if let Some(pos) = canon.text.find(&draft.text) {
            draft.span = pos..pos + draft.text.len();
        }
        mapped_drafts.push(draft);
    }

    Ok(mapped_drafts)
}
