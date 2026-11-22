// FILE-CONTEXT
// ZWECK: FallbackChunker delegierend an contextra_engine::chunker::MarkdownChunker (WP M3).
// INVARIANTEN: Monotone Suche im kanonischen Text, bei Nichtfund approximativ span_approximate=true.
// STAND: TS:2026-10-31T00:00:00Z

use crate::canonical::CanonicalText;
use crate::chunking::config::ChunkingConfig;
use crate::chunking::{ChunkDraft, IrChunker};
use crate::error::IngestError;
use crate::format::ChunkStrategy;
use crate::ir::DocumentIr;
use contextra_engine::chunker::{ChunkerConfig as EngineConfig, MarkdownChunker as EngineChunker};
use contextra_types::DocId;
use serde_json::{Map, Value};

/// Fallback-Chunker, der die kanonische Textrepräsentation an den MarkdownChunker aus contextra-engine übergibt.
#[derive(Debug, Default)]
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
        let engine_cfg = EngineConfig {
            max_tokens: cfg.max_tokens,
            min_tokens: cfg.min_tokens,
            include_breadcrumbs: cfg.include_breadcrumbs,
            split_levels: cfg.split_levels.clone(),
        };

        let chunker = EngineChunker::new(engine_cfg);
        let dummy_doc_id = DocId::new(1);
        let engine_chunks = chunker.chunk(dummy_doc_id, &canon.text);

        let mut drafts = Vec::with_capacity(engine_chunks.len());
        let mut search_cursor = 0usize;

        for chunk in engine_chunks {
            let chunk_str = &chunk.content;
            let (span, approx) = find_span(&canon.text, chunk_str, search_cursor);
            search_cursor = span.end;

            let mut breadcrumb = None;
            let mut heading_level = None;

            if let Some(ref meta) = chunk.metadata {
                if let Some(bc) = meta.get("breadcrumb").and_then(|v| v.as_str()) {
                    breadcrumb = Some(bc.to_string());
                }
                if let Some(hl) = meta.get("heading_level").and_then(|v| v.as_u64()) {
                    heading_level = u8::try_from(hl).ok();
                }
            }

            let locator = ir
                .blocks
                .first()
                .map(|b| b.locator.clone())
                .unwrap_or_default();

            let mut extra = Map::new();
            if approx {
                extra.insert("span_approximate".into(), Value::Bool(true));
            }

            drafts.push(ChunkDraft {
                text: chunk.content,
                span,
                locator,
                breadcrumb,
                heading_level,
                extra,
            });
        }

        Ok(drafts)
    }
}

fn find_span(full: &str, sub: &str, cursor: usize) -> (std::ops::Range<usize>, bool) {
    if sub.is_empty() {
        return (cursor..cursor, false);
    }

    // Exakte monotone Suche ab cursor
    if let Some(rel_pos) = full[cursor..].find(sub) {
        let start = cursor + rel_pos;
        let end = start + sub.len();
        return (start..end, false);
    }

    // Exakte globale Suche im gesamten Text
    if let Some(start) = full.find(sub) {
        let end = start + sub.len();
        return (start..end, true);
    }

    // Approximativer Fallback auf cursor bis Ende
    let end = (cursor + sub.len()).min(full.len());
    (cursor..end, true)
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
