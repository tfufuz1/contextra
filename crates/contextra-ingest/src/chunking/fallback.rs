//! Fallback-Chunker fuer unstrukturierte oder unbekannte Dokumentarten.
//!
//! # Zweck & Architektur
//! Gehoert zur Segmentierungsschicht (Ring 4) in `contextra-ingest`.
//! Zerlegt die hierarchische `DocumentIr` in semantisch zusammenhaengende,
//! budgetierte Einheiten (Chunks) fuer Embeddings und Volltext-Indexierung.
//!
// FILE-CONTEXT
// ZWECK: FallbackChunker autonom delegierend an ProseChunker (WP M3).
// INVARIANTEN: Monotone Suche im kanonischen Text, bei Nichtfund approximativ span_approximate=true.
// STAND: TS:2026-10-31T00:00:00Z

use crate::canonical::CanonicalText;
use crate::chunking::config::ChunkingConfig;
use crate::chunking::prose::ProseChunker;
use crate::chunking::{ChunkDraft, IrChunker};
use crate::error::IngestError;
use crate::format::ChunkStrategy;
use crate::ir::DocumentIr;

/// Fallback-Chunker, der unstrukturierte Dokumente oder unbekannte Formate über ProseChunker verarbeitet.
#[derive(Debug, Default)]
// ANCHOR[contextra-ingest:CHK-0212]
// struct `FallbackChunker` fuer Fallback-Chunker fuer unstrukturierte oder unbekannte Dokumentarten.
pub struct FallbackChunker;

impl IrChunker for FallbackChunker {
    fn strategy(&self) -> ChunkStrategy {
        ChunkStrategy::Prose
    }

    fn draft(
        &self,
        ir: &DocumentIr,
        canon: &CanonicalText,
        cfg: &ChunkingConfig,
    ) -> Result<Vec<ChunkDraft>, IngestError> {
        let prose_chunker = ProseChunker;
        prose_chunker.draft(ir, canon, cfg)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )]
    use super::*;

    #[test]
    fn test_fallback_chunker_drafts() {
        let mut ir = DocumentIr::new(crate::format::FormatId::Markdown);
        ir.push(
            crate::ir::BlockKind::Heading {
                level: 1,
                text: "Header".into(),
            },
            crate::ir::Locator::default(),
        );
        ir.push(
            crate::ir::BlockKind::Paragraph {
                text: "Paragraph content".into(),
            },
            crate::ir::Locator::default(),
        );

        let canon = ir.canonical();
        let cfg = ChunkingConfig::default();
        let chunker = FallbackChunker;

        let drafts = chunker.draft(&ir, &canon, &cfg).unwrap();
        assert!(!drafts.is_empty());
        assert_eq!(&canon.text[drafts[0].span.clone()], drafts[0].text);
    }
}
