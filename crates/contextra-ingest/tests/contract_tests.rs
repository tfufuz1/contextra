// FILE-CONTEXT
// ZWECK: Integrationstests für Verträge, Formate, Kanonisierung, Pipeline & Fallback-Chunker (WP M5).
// INVARIANTEN: All-targets / All-features Lints strikt einhalten.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use contextra_engine::chunker::{ChunkerConfig as EngineConfig, MarkdownChunker as EngineChunker};
use contextra_ingest::canonical::sanitize_text;
use contextra_ingest::chunking::config::ChunkingConfig;
use contextra_ingest::chunking::fallback::FallbackChunker;
use contextra_ingest::chunking::IrChunker;
use contextra_ingest::error::IngestError;
use contextra_ingest::extractor::{DocumentExtractor, ExtractContext};
use contextra_ingest::format::{ChunkStrategy, FormatId};
use contextra_ingest::ir::{
    BlockKind, Boundary, DocumentIr, Locator, MessageHeader, NoteKind, Table,
};
use contextra_ingest::limits::IngestLimits;
use contextra_ingest::pipeline::{ingest, IngestConfig, IngestRequest};
use contextra_ingest::registry::ExtractorRegistry;
use contextra_types::{ContextraError, DocId};

struct MockTextExtractor;
impl DocumentExtractor for MockTextExtractor {
    fn formats(&self) -> &'static [FormatId] {
        &[FormatId::Txt, FormatId::Markdown]
    }
    fn extract(
        &self,
        format: FormatId,
        input: &[u8],
        _ctx: &ExtractContext<'_>,
    ) -> Result<DocumentIr, IngestError> {
        let text = std::str::from_utf8(input).map_err(|e| IngestError::Corrupt {
            format: Some(format),
            detail: e.to_string(),
        })?;

        let mut ir = DocumentIr::new(format);
        ir.meta.title = Some("Mock Document".into());
        ir.push(
            BlockKind::Heading {
                level: 1,
                text: "Heading Title".into(),
            },
            Locator::default(),
        );
        ir.push(
            BlockKind::Paragraph {
                text: text.to_string(),
            },
            Locator::default(),
        );
        Ok(ir)
    }
}

struct PanickingExtractor;
impl DocumentExtractor for PanickingExtractor {
    fn formats(&self) -> &'static [FormatId] {
        &[FormatId::Pdf]
    }
    fn extract(
        &self,
        _format: FormatId,
        _input: &[u8],
        _ctx: &ExtractContext<'_>,
    ) -> Result<DocumentIr, IngestError> {
        panic!("Simulated extractor panic");
    }
}

#[test]
fn test_contract_1_format_id_consistency() {
    let formats = [
        FormatId::Txt,
        FormatId::Markdown,
        FormatId::Pdf,
        FormatId::Docx,
        FormatId::Xlsx,
        FormatId::Eml,
        FormatId::Json,
    ];

    for fmt in formats {
        let name = fmt.name();
        assert!(!name.is_empty());
        let ext = fmt.extensions()[0];
        assert_eq!(FormatId::from_extension(ext), Some(fmt));
        let strategy = fmt.default_strategy();
        match fmt {
            FormatId::Pdf => assert_eq!(strategy, ChunkStrategy::Paged),
            FormatId::Xlsx => assert_eq!(strategy, ChunkStrategy::Tabular),
            FormatId::Json => assert_eq!(strategy, ChunkStrategy::Structured),
            FormatId::Eml => assert_eq!(strategy, ChunkStrategy::Email),
            _ => {}
        }
    }
}

#[test]
fn test_contract_2_canonical_golden_and_slicing() {
    let mut ir = DocumentIr::new(FormatId::Markdown);
    ir.push(
        BlockKind::Heading {
            level: 1,
            text: "Main Title".into(),
        },
        Locator::default(),
    );
    ir.push(
        BlockKind::Paragraph {
            text: "A simple paragraph.".into(),
        },
        Locator::default(),
    );
    ir.push(
        BlockKind::ListItem {
            depth: 0,
            ordered: false,
            text: "Item 1".into(),
        },
        Locator::default(),
    );
    let mut table = Table::default();
    table.caption = Some("Sample Table".into());
    table.header_rows = 1;
    table.rows = vec![
        vec!["H1".into(), "H2".into()],
        vec!["V1".into(), "V2".into()],
    ];
    table.origin = (0, 0);

    ir.push(BlockKind::Table(table), Locator::default());
    ir.push(
        BlockKind::Code {
            lang: Some("rust".into()),
            text: "fn main() {}".into(),
        },
        Locator::default(),
    );
    ir.push(
        BlockKind::Quote {
            text: "Quoted line".into(),
        },
        Locator::default(),
    );
    ir.push(
        BlockKind::Note {
            kind: NoteKind::Footnote,
            text: "Footnote text".into(),
        },
        Locator::default(),
    );
    ir.push(
        BlockKind::Boundary(Boundary::Page { number: 1 }),
        Locator::default(),
    );
    ir.push(
        BlockKind::Record {
            path: "user.name".into(),
            value: "Alice".into(),
        },
        Locator::default(),
    );
    let mut msg = MessageHeader::default();
    msg.subject = Some("Hello".into());
    msg.from = vec!["alice@example.com".into()];
    msg.to = vec!["bob@example.com".into()];
    msg.date = Some("2026-10-31".into());

    ir.push(BlockKind::Message(msg), Locator::default());
    ir.push(
        BlockKind::Raw {
            text: "Verbatim raw text".into(),
        },
        Locator::default(),
    );

    let canon = ir.canonical();
    assert_eq!(canon.block_spans.len(), 11);

    for (idx, span) in canon.block_spans.iter().enumerate() {
        let block_slice = &canon.text[span.clone()];
        assert!(!block_slice.is_empty(), "Block {} span is empty", idx);
    }

    assert_eq!(
        &canon.text[canon.block_spans[10].clone()],
        "Verbatim raw text"
    );
}

#[test]
fn test_contract_3_sanitize_text_cases() {
    let raw = "Hello\r\nWorld\rBOM\u{FEFF} Tag\u{E0001} Bidi\u{202A} C0\u{0007}End";
    let (clean, count) = sanitize_text(raw);
    assert_eq!(clean, "Hello\nWorld\nBOM Tag Bidi C0End");
    assert!(count >= 5);
}

#[test]
fn test_contract_4_pipeline_mock_end_to_end_determinism() {
    let mut reg = ExtractorRegistry::new_empty();
    reg.register(Box::new(MockTextExtractor));

    let req = IngestRequest {
        bytes: b"Line 1\nLine 2\nLine 3",
        filename: Some("test.txt"),
        mime: Some("text/plain"),
        format_override: Some(FormatId::Txt),
        doc_id: DocId::new(100),
    };

    let cfg = IngestConfig::default();

    let out1 = ingest(&req, &cfg, &reg).expect("ingest should succeed");
    let out2 = ingest(&req, &cfg, &reg).expect("ingest should succeed");

    assert_eq!(out1.canonical_hash, out2.canonical_hash);
    assert_eq!(out1.chunks.len(), out2.chunks.len());
    assert_eq!(out1.chunks[0].content, out2.chunks[0].content);

    let meta1 = out1.chunks[0].metadata.as_ref().unwrap();
    let meta2 = out2.chunks[0].metadata.as_ref().unwrap();
    assert_eq!(meta1["chunk_id"], meta2["chunk_id"]);
}

#[test]
fn test_contract_5_panic_catching() {
    let mut reg = ExtractorRegistry::new_empty();
    reg.register(Box::new(PanickingExtractor));

    let req = IngestRequest {
        bytes: b"dummy",
        filename: Some("bad.pdf"),
        mime: Some("application/pdf"),
        format_override: Some(FormatId::Pdf),
        doc_id: DocId::new(101),
    };

    let cfg = IngestConfig::default();
    let err = ingest(&req, &cfg, &reg).unwrap_err();
    assert_eq!(
        err,
        IngestError::ParserPanic {
            format: FormatId::Pdf
        }
    );
}

#[test]
fn test_contract_6_error_mapping() {
    let limit_err = IngestError::LimitExceeded {
        kind: contextra_ingest::error::LimitKind::DocumentBytes,
        limit: 100,
        observed: 200,
    };
    let c_err: ContextraError = limit_err.into();
    assert!(matches!(c_err, ContextraError::LimitExceeded { .. }));
}

#[test]
fn test_contract_7_fallback_chunker_markdown_chunker_equality() {
    let _markdown = "# Section 1\nFirst paragraph text.\n## Section 2\nSecond paragraph text.";

    let mut ir = DocumentIr::new(FormatId::Markdown);
    ir.push(
        BlockKind::Heading {
            level: 1,
            text: "Section 1".into(),
        },
        Locator::default(),
    );
    ir.push(
        BlockKind::Paragraph {
            text: "First paragraph text.".into(),
        },
        Locator::default(),
    );
    ir.push(
        BlockKind::Heading {
            level: 2,
            text: "Section 2".into(),
        },
        Locator::default(),
    );
    ir.push(
        BlockKind::Paragraph {
            text: "Second paragraph text.".into(),
        },
        Locator::default(),
    );

    let canon = ir.canonical();

    let mut cfg = ChunkingConfig::default();
    cfg.max_tokens = 512;
    cfg.min_tokens = 0;

    let fallback_drafts = FallbackChunker.draft(&ir, &canon, &cfg).unwrap();

    let engine_cfg = EngineConfig {
        max_tokens: 512,
        min_tokens: 0,
        include_breadcrumbs: true,
        split_levels: vec![1, 2, 3],
    };

    let engine_chunks = EngineChunker::new(engine_cfg).chunk(DocId::new(1), &canon.text);

    assert_eq!(fallback_drafts.len(), engine_chunks.len());
    for (draft, chunk) in fallback_drafts.iter().zip(engine_chunks.iter()) {
        assert_eq!(draft.text, chunk.content);
    }
}

#[test]
fn test_contract_8_stubs_return_unsupported_and_fuzz_support_panics_free() {
    let reg = ExtractorRegistry::with_defaults();
    let limits = IngestLimits::default();
    let policy = contextra_ingest::limits::ExtractPolicy::default();
    let budget = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx = ExtractContext {
        limits: &limits,
        policy: &policy,
        budget: &budget,
        registry: &reg,
        filename: None,
    };

    let err = reg.extract(FormatId::Docx, b"dummy", &ctx).unwrap_err();
    assert!(matches!(err, IngestError::UnsupportedFormat { .. }));

    // Fuzz support muss für beliebige Bytes panikfrei sein
    contextra_ingest::fuzz_support::run(FormatId::Txt, b"random noise 12345");
}
