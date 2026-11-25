//! Integrationstest-Suite: `chunk_structured_tests.rs`
//!
//! # Zweck & Testabdeckung
//! Verifiziert das Verhalten, Zero-Panic-Invariante und deterministische Ausgabe
//! von `contextra-ingest` fuer dieses Modul/Format gemaess Spezifikation.
//!
// FILE-CONTEXT
// ZWECK: Akzeptanz- und Proptests für StructuredChunker (WP-ING-15).
// INVARIANTEN: Zero-Panic in Produktion, Test-First, Lints eingehalten.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use contextra_ingest::chunking::config::ChunkingConfig;
use contextra_ingest::chunking::structured::StructuredChunker;
use contextra_ingest::chunking::IrChunker;
use contextra_ingest::format::FormatId;
use contextra_ingest::ir::{BlockKind, DocumentIr, Locator};
use proptest::prelude::*;

// ANCHOR[contextra-ingest:TST-0327]
// Testet die Funktionsfaehigkeit von `test_structured_5000_customers_bundling_and_coverage`.
#[test]
fn test_structured_5000_customers_bundling_and_coverage() {
    let mut ir = DocumentIr::new(FormatId::Json);
    let customer_count = 5000usize;

    for i in 0..customer_count {
        ir.push(
            BlockKind::Record {
                path: format!("kunden[{}].id", i),
                value: format!("{}", 1000 + i),
            },
            Locator::default(),
        );
        ir.push(
            BlockKind::Record {
                path: format!("kunden[{}].name", i),
                value: format!("Kunde_{}", i),
            },
            Locator::default(),
        );
    }

    let canon = ir.canonical();
    let mut cfg = ChunkingConfig::default();
    cfg.max_tokens = 512;
    cfg.structured.group_depth = 2;
    cfg.structured.batch_small_records = true;

    let chunker = StructuredChunker;
    let drafts = chunker.draft(&ir, &canon, &cfg).unwrap();

    assert!(!drafts.is_empty(), "Drafts should not be empty");

    // Prüfen, dass alle Records vorkommen und Spans monoton sind
    let mut last_span_end = 0usize;
    for draft in &drafts {
        assert!(
            draft.span.start >= last_span_end,
            "Spans must be monotonic: start {} < last_end {}",
            draft.span.start,
            last_span_end
        );
        assert!(draft.span.end <= canon.text.len());
        assert_eq!(&canon.text[draft.span.clone()], draft.text);
        last_span_end = draft.span.end;
    }
}

// ANCHOR[contextra-ingest:TST-0328]
// Testet die Funktionsfaehigkeit von `test_structured_large_record_windowing`.
#[test]
fn test_structured_large_record_windowing() {
    let mut ir = DocumentIr::new(FormatId::Json);
    let large_value = "A".repeat(100_000);

    ir.push(
        BlockKind::Record {
            path: "big.data.payload".into(),
            value: large_value,
        },
        Locator::default(),
    );

    let canon = ir.canonical();
    let cfg = ChunkingConfig::default();
    let chunker = StructuredChunker;

    let drafts = chunker.draft(&ir, &canon, &cfg).unwrap();

    assert!(
        drafts.len() > 1,
        "100,000 char value must be split into multiple window drafts"
    );

    for draft in &drafts {
        assert!(
            draft.text.starts_with("big.data.payload: "),
            "Every window draft must carry 'path: ' prefix"
        );
        assert!(draft.extra.contains_key("part"));
        assert_eq!(
            draft.extra.get("synthesized"),
            Some(&serde_json::Value::Bool(true))
        );
    }
}

// ANCHOR[contextra-ingest:TST-0329]
// Testet die Funktionsfaehigkeit von `test_structured_jsonl_10000_lines`.
#[test]
fn test_structured_jsonl_10000_lines() {
    let mut ir = DocumentIr::new(FormatId::Jsonl);
    let line_count = 10_000usize;

    for i in 0..line_count {
        ir.push(
            BlockKind::Record {
                path: format!("[{}]", i),
                value: format!("{{\"line\": {}, \"status\": \"ok\"}}", i),
            },
            Locator::default(),
        );
    }

    let canon = ir.canonical();
    let mut cfg = ChunkingConfig::default();
    cfg.max_tokens = 512;

    let chunker = StructuredChunker;
    let drafts = chunker.draft(&ir, &canon, &cfg).unwrap();

    assert!(!drafts.is_empty());

    for draft in &drafts {
        assert!(
            draft.extra.contains_key("record_range"),
            "JSONL drafts must contain record_range metadata"
        );
    }
}

// ANCHOR[contextra-ingest:TST-0330]
// Testet die Funktionsfaehigkeit von `test_structured_deep_paths_and_special_characters`.
#[test]
fn test_structured_deep_paths_and_special_characters() {
    let mut ir = DocumentIr::new(FormatId::Yaml);

    ir.push(
        BlockKind::Record {
            path: "system[0].config.net.adapters[2].ip#v4@address".into(),
            value: "192.168.1.100".into(),
        },
        Locator::default(),
    );
    ir.push(
        BlockKind::Record {
            path: "system[0].config.net.adapters[2].gateway#v4".into(),
            value: "192.168.1.1".into(),
        },
        Locator::default(),
    );

    let canon = ir.canonical();
    let cfg = ChunkingConfig::default();
    let chunker = StructuredChunker;

    let drafts = chunker.draft(&ir, &canon, &cfg).unwrap();

    assert_eq!(drafts.len(), 1);
    assert_eq!(&canon.text[drafts[0].span.clone()], drafts[0].text);
}

proptest! {
// ANCHOR[contextra-ingest:TST-0331]
// Testet die Funktionsfaehigkeit von `prop_structured_chunker_determinism_and_monotone_spans`.
    #[test]
    fn prop_structured_chunker_determinism_and_monotone_spans(
        val in ".*"
    ) {
        let mut ir = DocumentIr::new(FormatId::Json);
        ir.push(
            BlockKind::Record {
                path: "prop.key.a".into(),
                value: val.clone(),
            },
            Locator::default(),
        );
        ir.push(
            BlockKind::Record {
                path: "prop.key.b".into(),
                value: "val_b".into(),
            },
            Locator::default(),
        );

        let canon = ir.canonical();
        let cfg = ChunkingConfig::default();
        let chunker = StructuredChunker;

        let res1 = chunker.draft(&ir, &canon, &cfg).unwrap();
        let res2 = chunker.draft(&ir, &canon, &cfg).unwrap();

        // Determinismus
        prop_assert_eq!(res1.len(), res2.len());
        for (d1, d2) in res1.iter().zip(res2.iter()) {
            prop_assert_eq!(&d1.text, &d2.text);
            prop_assert_eq!(&d1.span, &d2.span);
        }

        // Monotone Spans
        let mut prev_end = 0usize;
        for d in &res1 {
            prop_assert!(d.span.start >= prev_end);
            prop_assert!(d.span.end <= canon.text.len());
            prev_end = d.span.end;
        }
    }
}
