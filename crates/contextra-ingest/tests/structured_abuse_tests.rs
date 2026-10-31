//! Integrationstest-Suite: `structured_abuse_tests.rs`
//!
//! # Zweck & Testabdeckung
//! Verifiziert das Verhalten, Zero-Panic-Invariante und deterministische Ausgabe
//! von `contextra-ingest` fuer dieses Modul/Format gemaess Spezifikation.
//!
// FILE-CONTEXT
// ZWECK: Missbrauchs- und Proptest-Tests für strukturierte Extraktoren (P00B/WP M4).
// INVARIANTEN: Zero-Panic, LimitExceeded bei Tiefen- und Textgrößenüberschreitungen.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use contextra_ingest::error::{IngestError, LimitKind};
use contextra_ingest::extractor::{DocumentExtractor, ExtractContext, ResourceBudget};
use contextra_ingest::format::FormatId;
use contextra_ingest::formats::structured::StructuredExtractor;
use contextra_ingest::limits::{ExtractPolicy, IngestLimits};
use contextra_ingest::registry::ExtractorRegistry;
use proptest::prelude::*;

fn create_ctx<'a>(
    limits: &'a IngestLimits,
    policy: &'a ExtractPolicy,
    budget: &'a ResourceBudget,
    registry: &'a ExtractorRegistry,
) -> ExtractContext<'a> {
    ExtractContext {
        limits,
        policy,
        budget,
        registry,
        filename: None,
    }
}

// ANCHOR[contextra-ingest:TST-0412]
// Testet die Funktionsfaehigkeit von `test_json_depth_limit_exceeded`.
#[test]
fn test_json_depth_limit_exceeded() {
    let extractor = StructuredExtractor::new();
    let depth = 500usize;
    let mut deep_json = String::with_capacity(depth * 10);
    for _ in 0..depth {
        deep_json.push_str("{\"a\":");
    }
    deep_json.push('1');
    for _ in 0..depth {
        deep_json.push('}');
    }

    let mut limits = IngestLimits::default();
    limits.max_xml_depth = 50;
    let policy = ExtractPolicy::default();
    let budget = ResourceBudget::new(limits.clone());
    let reg = ExtractorRegistry::new_empty();
    let ctx = create_ctx(&limits, &policy, &budget, &reg);

    let err = extractor
        .extract(FormatId::Json, deep_json.as_bytes(), &ctx)
        .unwrap_err();
    assert!(matches!(
        err,
        IngestError::LimitExceeded {
            kind: LimitKind::NestingDepth,
            ..
        }
    ));
}

// ANCHOR[contextra-ingest:TST-0413]
// Testet die Funktionsfaehigkeit von `test_yaml_deep_nesting_limit`.
#[test]
fn test_yaml_deep_nesting_limit() {
    let extractor = StructuredExtractor::new();
    let depth = 500usize;
    let mut deep_yaml = String::new();
    for i in 0..depth {
        deep_yaml.push_str(&" ".repeat(i * 2));
        deep_yaml.push_str("node:\n");
    }
    deep_yaml.push_str(&" ".repeat(depth * 2));
    deep_yaml.push_str("val: 1\n");

    let mut limits = IngestLimits::default();
    limits.max_xml_depth = 50;
    let policy = ExtractPolicy::default();
    let budget = ResourceBudget::new(limits.clone());
    let reg = ExtractorRegistry::new_empty();
    let ctx = create_ctx(&limits, &policy, &budget, &reg);

    let err = extractor
        .extract(FormatId::Yaml, deep_yaml.as_bytes(), &ctx)
        .unwrap_err();
    assert!(matches!(
        err,
        IngestError::LimitExceeded {
            kind: LimitKind::NestingDepth,
            ..
        }
    ));
}

// ANCHOR[contextra-ingest:TST-0414]
// Testet die Funktionsfaehigkeit von `test_xml_billion_laughs_and_entities_discarded`.
#[test]
fn test_xml_billion_laughs_and_entities_discarded() {
    let extractor = StructuredExtractor::new();
    let xml_bomb = br#"<?xml version="1.0"?>
    <!DOCTYPE lolz [
     <!ENTITY lol "lol">
     <!ELEMENT lolz (#PCDATA)>
     <!ENTITY lol1 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
    ]>
    <lolz>&lol1;</lolz>"#;

    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = ResourceBudget::new(limits.clone());
    let reg = ExtractorRegistry::new_empty();
    let ctx = create_ctx(&limits, &policy, &budget, &reg);

    let ir = extractor
        .extract(FormatId::Xml, xml_bomb, &ctx)
        .expect("XML Entities must be discarded without expansion");
    // DOCTYPE/Entities werden ignoriert (P00B)
    assert!(!ir.blocks.is_empty());
}

// ANCHOR[contextra-ingest:TST-0415]
// Testet die Funktionsfaehigkeit von `test_text_bytes_limit_exceeded`.
#[test]
fn test_text_bytes_limit_exceeded() {
    let extractor = StructuredExtractor::new();
    let mut huge_json = String::from("{\"key\":\"");
    huge_json.push_str(&"x".repeat(1024 * 1024)); // 1 MiB String
    huge_json.push_str("\"}");

    let mut limits = IngestLimits::default();
    limits.max_text_bytes = 100 * 1024; // 100 KiB Limit
    let policy = ExtractPolicy::default();
    let budget = ResourceBudget::new(limits.clone());
    let reg = ExtractorRegistry::new_empty();
    let ctx = create_ctx(&limits, &policy, &budget, &reg);

    let err = extractor
        .extract(FormatId::Json, huge_json.as_bytes(), &ctx)
        .unwrap_err();
    assert!(matches!(
        err,
        IngestError::LimitExceeded {
            kind: LimitKind::TextBytes,
            ..
        }
    ));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

// ANCHOR[contextra-ingest:TST-0416]
// Testet die Funktionsfaehigkeit von `test_property_zero_panic_arbitrary_bytes`.
    #[test]
    fn test_property_zero_panic_arbitrary_bytes(
        fmt_idx in 0u8..5u8,
        bytes in prop::collection::vec(any::<u8>(), 0..1024)
    ) {
        let format = match fmt_idx {
            0 => FormatId::Json,
            1 => FormatId::Jsonl,
            2 => FormatId::Xml,
            3 => FormatId::Yaml,
            _ => FormatId::Toml,
        };

        let extractor = StructuredExtractor::new();
        let limits = IngestLimits::default();
        let policy = ExtractPolicy::default();
        let budget = ResourceBudget::new(limits.clone());
        let reg = ExtractorRegistry::new_empty();
        let ctx = create_ctx(&limits, &policy, &budget, &reg);

        // Darf niemals paniken!
        let _ = extractor.extract(format, &bytes, &ctx);
    }
}
