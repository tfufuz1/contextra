// FILE-CONTEXT
// ZWECK: Haupt-Pipeline für Ingestion (Format-Erkennung, Extraktion, Kanonisierung & Chunking) (WP M3).
// INVARIANTEN: Zero-Panic, Limit-Einhaltung (max_document_bytes, max_text_bytes, max_chunks_per_document).
// STAND: TS:2026-10-31T00:00:00Z

use crate::chunking::config::ChunkingConfig;
use crate::chunking::{common::finalize, draft_document};
use crate::detect::{detect, DetectHints};
use crate::error::{IngestError, LimitKind};
use crate::extractor::{ExtractContext, ResourceBudget};
use crate::format::FormatId;
use crate::ir::{DocMeta, ExtractWarning};
use crate::limits::{ExtractPolicy, IngestLimits};
use crate::registry::ExtractorRegistry;
use contextra_types::{ContextChunk, DocId};

/// Ingestion-Anforderung.
pub struct IngestRequest<'a> {
    pub bytes: &'a [u8],
    pub filename: Option<&'a str>,
    pub mime: Option<&'a str>,
    pub format_override: Option<FormatId>,
    pub doc_id: DocId,
}

/// Ingestion-Konfiguration.
#[derive(Debug, Clone, Default)]
pub struct IngestConfig {
    pub limits: IngestLimits,
    pub policy: ExtractPolicy,
    pub chunking: ChunkingConfig,
}

/// Ergebnis der Ingestion-Pipeline.
#[derive(Debug, Clone)]
pub struct IngestOutput {
    pub format: FormatId,
    pub meta: DocMeta,
    pub chunks: Vec<ContextChunk>,
    pub warnings: Vec<ExtractWarning>,
    pub canonical_hash: String,
}

/// Führt die vollständige Ingestion-Pipeline für ein Dokument durch.
pub fn ingest(
    req: &IngestRequest<'_>,
    cfg: &IngestConfig,
    registry: &ExtractorRegistry,
) -> Result<IngestOutput, IngestError> {
    // 1. Größencheck
    let doc_bytes = req.bytes.len() as u64;
    if doc_bytes > cfg.limits.max_document_bytes {
        return Err(IngestError::LimitExceeded {
            kind: LimitKind::DocumentBytes,
            limit: cfg.limits.max_document_bytes,
            observed: doc_bytes,
        });
    }

    // 2. Format-Erkennung / Override
    let format = match req.format_override {
        Some(fmt) => fmt,
        None => {
            let hints = DetectHints {
                filename: req.filename,
                mime: req.mime,
            };
            detect(req.bytes, &hints)?.format
        }
    };

    // 3. Extraktion
    let budget = ResourceBudget::new(cfg.limits.clone());
    let ctx = ExtractContext {
        limits: &cfg.limits,
        policy: &cfg.policy,
        budget: &budget,
        registry,
        filename: req.filename,
    };

    let ir = registry.extract(format, req.bytes, &ctx)?;

    // 4. Kanonisierung
    let canon = ir.canonical();
    let text_bytes = canon.text.len() as u64;
    if text_bytes > cfg.limits.max_text_bytes {
        return Err(IngestError::LimitExceeded {
            kind: LimitKind::TextBytes,
            limit: cfg.limits.max_text_bytes,
            observed: text_bytes,
        });
    }

    let canonical_hash = ir.canonical_hash();

    // 5. Chunking
    let drafts = draft_document(&ir, &canon, &cfg.chunking)?;

    // 6. Finalisierung
    let chunks = finalize(drafts, &ir, &canon, &cfg.chunking, req.doc_id, &cfg.limits)?;

    Ok(IngestOutput {
        format,
        meta: ir.meta,
        chunks,
        warnings: ir.warnings,
        canonical_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::DocumentExtractor;
    use crate::ir::{BlockKind, DocumentIr};

    struct MockExtractor;
    impl DocumentExtractor for MockExtractor {
        fn formats(&self) -> &'static [FormatId] {
            &[FormatId::Txt]
        }
        fn extract(
            &self,
            format: FormatId,
            _input: &[u8],
            _ctx: &ExtractContext<'_>,
        ) -> Result<DocumentIr, IngestError> {
            let mut ir = DocumentIr::new(format);
            ir.push(
                BlockKind::Paragraph {
                    text: "Mock paragraph text for pipeline test.".into(),
                },
                Default::default(),
            );
            Ok(ir)
        }
    }

    #[test]
    fn test_ingest_pipeline_end_to_end() {
        let mut reg = ExtractorRegistry::new_empty();
        reg.register(Box::new(MockExtractor));

        let req = IngestRequest {
            bytes: b"Mock paragraph text for pipeline test.",
            filename: Some("test.txt"),
            mime: Some("text/plain"),
            format_override: Some(FormatId::Txt),
            doc_id: DocId::new(42),
        };

        let cfg = IngestConfig::default();
        let output = ingest(&req, &cfg, &reg).unwrap();

        assert_eq!(output.format, FormatId::Txt);
        assert!(!output.chunks.is_empty());
        assert_eq!(output.chunks[0].doc_id, DocId::new(42));
    }
}
