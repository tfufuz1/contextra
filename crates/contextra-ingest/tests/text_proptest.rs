//! Integrationstest-Suite: `text_proptest.rs`
//!
//! # Zweck & Testabdeckung
//! Verifiziert das Verhalten, Zero-Panic-Invariante und deterministische Ausgabe
//! von `contextra-ingest` fuer dieses Modul/Format gemaess Spezifikation.
//!
// FILE-CONTEXT
// ZWECK: Property-based Tests (proptest) für Identitätsinvariante und Zero-Panic (WP M4).
// INVARIANTEN: Zero-Panic bei beliebigen Bytes, Identität bei Markdown/Txt.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_ingest::extractor::{DocumentExtractor, ExtractContext, ResourceBudget};
use contextra_ingest::format::FormatId;
use contextra_ingest::formats::text::TextExtractor;
use contextra_ingest::limits::{ExtractPolicy, IngestLimits};
use contextra_ingest::registry::ExtractorRegistry;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Identitätsinvariante für Markdown:
    /// Saubere UTF-8 Strings ohne \r und Steuerzeichen (außer \n/\t) müssen
    /// bei Markdown byte-identisch in canonical().text enthalten sein.
// ANCHOR[contextra-ingest:TST-0438]
// Testet die Funktionsfaehigkeit von `prop_markdown_identity`.
    #[test]
    fn prop_markdown_identity(s in "[a-zA-Z0-9 \\n\\täöüß🦀#=-]+") {
        let limits = IngestLimits::default();
        let policy = ExtractPolicy::default();
        let budget = ResourceBudget::new(limits.clone());
        let registry = ExtractorRegistry::new_empty();
        let ctx = ExtractContext {
            limits: &limits,
            policy: &policy,
            budget: &budget,
            registry: &registry,
            filename: Some("prop_identity.md"),
        };

        let extractor = TextExtractor::new();
        if let Ok(ir) = extractor.extract(FormatId::Markdown, s.as_bytes(), &ctx) {
            let canon = ir.canonical();
            prop_assert_eq!(canon.text, s);
        }
    }

    /// Zero-Panic Garantie: Beliebige Bytes in alle 6 Textformate dürfen NIEMALS panikieren.
// ANCHOR[contextra-ingest:TST-0439]
// Testet die Funktionsfaehigkeit von `prop_zero_panic_arbitrary_bytes`.
    #[test]
    fn prop_zero_panic_arbitrary_bytes(bytes in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let limits = IngestLimits::default();
        let policy = ExtractPolicy::default();
        let budget = ResourceBudget::new(limits.clone());
        let registry = ExtractorRegistry::new_empty();
        let ctx = ExtractContext {
            limits: &limits,
            policy: &policy,
            budget: &budget,
            registry: &registry,
            filename: Some("prop_zero_panic"),
        };

        let extractor = TextExtractor::new();
        let formats = [
            FormatId::Txt,
            FormatId::Markdown,
            FormatId::Rst,
            FormatId::AsciiDoc,
            FormatId::Tex,
            FormatId::Rtf,
        ];

        for &fmt in &formats {
            // Der Aufruf darf fehlschlagen (Result::Err), aber NIEMALS panikieren!
            let _ = extractor.extract(fmt, &bytes, &ctx);
        }
    }
}
