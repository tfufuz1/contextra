//! Integrationstest-Suite: `text_extractors_test.rs`
//!
//! # Zweck & Testabdeckung
//! Verifiziert das Verhalten, Zero-Panic-Invariante und deterministische Ausgabe
//! von `contextra-ingest` fuer dieses Modul/Format gemaess Spezifikation.
//!
// FILE-CONTEXT
// ZWECK: Golden-Tests, Kodierungstests und RTF-Spezialtests für Text-Extraktoren (WP M4).
// INVARIANTEN: Zero-Panic, JSON-Snapshots, UTF-16/Windows-1252/Unicode-Tag Handling.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_ingest::extractor::{DocumentExtractor, ExtractContext, ResourceBudget};
use contextra_ingest::format::FormatId;
use contextra_ingest::formats::text::TextExtractor;
use contextra_ingest::ir::{BlockKind, DocumentIr, WarningCode};
use contextra_ingest::limits::{ExtractPolicy, IngestLimits};
use contextra_ingest::registry::ExtractorRegistry;

fn make_ctx<'a>(
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
        filename: Some("test_file"),
    }
}

fn extract_doc(format: FormatId, bytes: &[u8], policy: ExtractPolicy) -> DocumentIr {
    let limits = IngestLimits::default();
    let budget = ResourceBudget::new(limits.clone());
    let registry = ExtractorRegistry::new_empty();
    let ctx = make_ctx(&limits, &policy, &budget, &registry);
    let extractor = TextExtractor::new();
    extractor
        .extract(format, bytes, &ctx)
        .expect("Extraction failed")
}

// --- GOLDEN TESTS (mit Umlauten äöü, ß, Emoji 🦀) ---

// ANCHOR[contextra-ingest:TST-0426]
// Testet die Funktionsfaehigkeit von `test_golden_txt`.
#[test]
fn test_golden_txt() {
    let text =
        "Erster Absatz mit Umlauten: äöü und ß.\nZweite Zeile.\n\nZweiter Absatz mit Emoji 🦀!";
    let ir = extract_doc(FormatId::Txt, text.as_bytes(), ExtractPolicy::default());

    let json_snapshot = serde_json::to_string_pretty(&ir).expect("JSON serialize failed");
    assert!(json_snapshot.contains("Erster Absatz mit Umlauten: äöü und ß."));
    assert!(json_snapshot.contains("Zweiter Absatz mit Emoji 🦀!"));
    assert_eq!(ir.blocks.len(), 2);
}

// ANCHOR[contextra-ingest:TST-0427]
// Testet die Funktionsfaehigkeit von `test_golden_markdown`.
#[test]
fn test_golden_markdown() {
    let text = "---\ntitle: \"Titel mit Umlauten äöü und ß 🦀\"\n---\n\n# Überschrift\n\nText";
    let ir = extract_doc(
        FormatId::Markdown,
        text.as_bytes(),
        ExtractPolicy::default(),
    );

    assert_eq!(
        ir.meta.title.as_deref(),
        Some("Titel mit Umlauten äöü und ß 🦀")
    );
    assert_eq!(ir.blocks.len(), 1);
    assert_eq!(ir.canonical().text, text);
}

// ANCHOR[contextra-ingest:TST-0428]
// Testet die Funktionsfaehigkeit von `test_golden_rst`.
#[test]
fn test_golden_rst() {
    let text = "Rst Dokument äöü\n================\n\nAbsatz mit ß und 🦀.\n\n.. code-block:: rust\n\n  fn main() {}\n";
    let ir = extract_doc(FormatId::Rst, text.as_bytes(), ExtractPolicy::default());

    assert_eq!(ir.meta.title.as_deref(), Some("Rst Dokument äöü"));
    let json_snapshot = serde_json::to_string_pretty(&ir).unwrap();
    assert!(json_snapshot.contains("rust"));
}

// ANCHOR[contextra-ingest:TST-0429]
// Testet die Funktionsfaehigkeit von `test_golden_asciidoc`.
#[test]
fn test_golden_asciidoc() {
    let text =
        "= AsciiDoc Titel äöü\n\nNOTE: Hinweis mit ß und 🦀.\n\n|===\n| Spalte 1 | Spalte 2\n|===";
    let ir = extract_doc(
        FormatId::AsciiDoc,
        text.as_bytes(),
        ExtractPolicy::default(),
    );

    assert_eq!(ir.meta.title.as_deref(), Some("AsciiDoc Titel äöü"));
    assert!(ir
        .blocks
        .iter()
        .any(|b| matches!(b.kind, BlockKind::Table(_))));
}

// ANCHOR[contextra-ingest:TST-0430]
// Testet die Funktionsfaehigkeit von `test_golden_tex`.
#[test]
fn test_golden_tex() {
    let text = "% TeX Kommentar\n\\section{TeX Kapitel äöü}\nAbsatz mit ß und 🦀.\n\n\\[ a^2 + b^2 = c^2 \\]\n";
    let ir = extract_doc(FormatId::Tex, text.as_bytes(), ExtractPolicy::default());

    assert_eq!(ir.meta.title.as_deref(), Some("TeX Kapitel äöü"));
    assert!(ir.canonical().text.contains("a^2 + b^2 = c^2"));
}

// ANCHOR[contextra-ingest:TST-0431]
// Testet die Funktionsfaehigkeit von `test_golden_rtf`.
#[test]
fn test_golden_rtf() {
    let rtf_str = r#"{\rtf1\ansi{\info{\title RTF Titel \'e4\'f6\'fc}}\f0 Hallo \u223? und \u129408?!\par Ende.}"#;
    let ir = extract_doc(FormatId::Rtf, rtf_str.as_bytes(), ExtractPolicy::default());

    assert_eq!(ir.meta.title.as_deref(), Some("RTF Titel äöü"));
}

// --- KODIERUNGSTESTS ---

// ANCHOR[contextra-ingest:TST-0432]
// Testet die Funktionsfaehigkeit von `test_encoding_utf16le_with_bom`.
#[test]
fn test_encoding_utf16le_with_bom() {
    let mut bytes = vec![0xFF, 0xFE]; // UTF-16LE BOM
    for u in "Hallo UTF-16LE äöü".encode_utf16() {
        bytes.extend_from_slice(&u.to_le_bytes());
    }

    let ir = extract_doc(FormatId::Txt, &bytes, ExtractPolicy::default());
    assert!(ir.canonical().text.contains("Hallo UTF-16LE äöü"));
}

// ANCHOR[contextra-ingest:TST-0433]
// Testet die Funktionsfaehigkeit von `test_encoding_windows1252_fallback`.
#[test]
fn test_encoding_windows1252_fallback() {
    // 0xE4, 0xF6, 0xFC, 0xDF in Windows-1252 are äöüß
    let bytes = [0xE4, 0xF6, 0xFC, 0xDF];

    let ir = extract_doc(FormatId::Txt, &bytes, ExtractPolicy::default());
    assert!(ir.canonical().text.contains("äöüß"));
    assert!(ir
        .warnings
        .iter()
        .any(|w| w.code == WarningCode::EncodingGuessed));
}

// ANCHOR[contextra-ingest:TST-0434]
// Testet die Funktionsfaehigkeit von `test_encoding_unicode_tags_removed_and_reported`.
#[test]
fn test_encoding_unicode_tags_removed_and_reported() {
    // Input containing Unicode tag character U+E0001
    let text = "Text mit \u{E0001} Tag-Zeichen.";

    let ir = extract_doc(FormatId::Txt, text.as_bytes(), ExtractPolicy::default());
    let clean_text = ir.canonical().text;
    assert!(!clean_text.contains('\u{E0001}'));
    assert!(ir
        .warnings
        .iter()
        .any(|w| w.code == WarningCode::TextSanitized));
}

// --- RTF SPEZIALTESTS ---

// ANCHOR[contextra-ingest:TST-0435]
// Testet die Funktionsfaehigkeit von `test_rtf_nesting_depth_limit`.
#[test]
fn test_rtf_nesting_depth_limit() {
    // 10,000 nested brackets => LimitExceeded without stack overflow
    let mut deep_rtf = Vec::new();
    for _ in 0..10_000 {
        deep_rtf.push(b'{');
    }
    deep_rtf.extend_from_slice(b"nested");
    for _ in 0..10_000 {
        deep_rtf.push(b'}');
    }

    let mut limits = IngestLimits::default();
    limits.max_xml_depth = 256;
    let policy = ExtractPolicy::default();
    let budget = ResourceBudget::new(limits.clone());
    let registry = ExtractorRegistry::new_empty();
    let ctx = make_ctx(&limits, &policy, &budget, &registry);

    let res = contextra_ingest::formats::text::rtf::parse_rtf(&deep_rtf, &ctx);
    assert!(res.is_err());
    if let Err(contextra_ingest::error::IngestError::LimitExceeded { kind, .. }) = res {
        assert_eq!(kind, contextra_ingest::error::LimitKind::NestingDepth);
    } else {
        panic!("Expected LimitExceeded for deep RTF nesting");
    }
}

// ANCHOR[contextra-ingest:TST-0436]
// Testet die Funktionsfaehigkeit von `test_rtf_unicode_escapes_and_tables`.
#[test]
fn test_rtf_unicode_escapes_and_tables() {
    let rtf_bytes = br#"{\rtf1\ansi\trowd\cell \u20013?\cell \u25991?\row}"#;
    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = ResourceBudget::new(limits.clone());
    let registry = ExtractorRegistry::new_empty();
    let ctx = make_ctx(&limits, &policy, &budget, &registry);

    let ir = contextra_ingest::formats::text::rtf::parse_rtf(rtf_bytes, &ctx).unwrap();
    assert!(ir
        .blocks
        .iter()
        .any(|b| matches!(b.kind, BlockKind::Table(_))));
}

// ANCHOR[contextra-ingest:TST-0437]
// Testet die Funktionsfaehigkeit von `test_rtf_hidden_text_policy`.
#[test]
fn test_rtf_hidden_text_policy() {
    let rtf_bytes = br#"{\rtf1\ansi Visible text \v Hidden secret\v0  visible again.}"#;

    // 1. policy include_hidden = false
    let limits = IngestLimits::default();
    let mut policy_no_hidden = ExtractPolicy::default();
    policy_no_hidden.include_hidden = false;
    let budget1 = ResourceBudget::new(limits.clone());
    let registry1 = ExtractorRegistry::new_empty();
    let ctx1 = make_ctx(&limits, &policy_no_hidden, &budget1, &registry1);

    let ir1 = contextra_ingest::formats::text::rtf::parse_rtf(rtf_bytes, &ctx1).unwrap();
    let text1 = ir1.canonical().text;
    assert!(!text1.contains("secret"));

    // 2. policy include_hidden = true
    let mut policy_hidden = ExtractPolicy::default();
    policy_hidden.include_hidden = true;
    let budget2 = ResourceBudget::new(limits.clone());
    let registry2 = ExtractorRegistry::new_empty();
    let ctx2 = make_ctx(&limits, &policy_hidden, &budget2, &registry2);

    let ir2 = contextra_ingest::formats::text::rtf::parse_rtf(rtf_bytes, &ctx2).unwrap();
    let text2 = ir2.canonical().text;
    assert!(text2.contains("secret"));
}
