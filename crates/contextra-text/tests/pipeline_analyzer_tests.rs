#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
//! Umfassende Tests für die Analyzer-Pipeline (Soll-Zustand TEXT-1).
//!
//! Prüft Token-Vertrag (Offsets auf Zeichengrenzen, Term, Positionsinkrement),
//! Index/Query-Identität, Fingerprint-Stabilität, Normalisierungs-Idempotenz und Grenzen.

use contextra_text::tokenizer::{
    Analyzer, AnalyzerPipeline, NormalizationPolicy, OverflowPolicy, PipelineLimits,
};
use proptest::prelude::*;

#[test]
fn test_pipeline_token_offsets_strictly_on_char_boundaries() {
    let pipeline = AnalyzerPipeline::default_german();
    let text = "Überprüfung der Softwareentwicklung in München: https://example.com/v1 und mail@domain.de! 🦀";

    let tokens = pipeline.analyze(text).expect("Analyse erfolgreich");
    assert!(!tokens.is_empty());

    for t in &tokens {
        assert!(
            text.is_char_boundary(t.start_offset),
            "start_offset {} not on char boundary",
            t.start_offset
        );
        assert!(
            text.is_char_boundary(t.end_offset),
            "end_offset {} not on char boundary",
            t.end_offset
        );
        assert!(t.start_offset <= t.end_offset);
    }
}

#[test]
fn test_pipeline_index_query_identity_and_fingerprint() {
    let index_pipeline = AnalyzerPipeline::default_german();
    let query_pipeline = AnalyzerPipeline::default_german();

    assert_eq!(
        index_pipeline.fingerprint(),
        query_pipeline.fingerprint(),
        "Index- und Query-Pipeline müssen denselben Fingerprint besitzen"
    );

    let query = "Softwarearchitektur und Datenschutzrichtlinie";
    let index_tokens = index_pipeline.analyze(query).unwrap();
    let query_tokens = query_pipeline.analyze(query).unwrap();

    assert_eq!(
        index_tokens, query_tokens,
        "Index- und Query-Analyse müssen 100% identisch sein"
    );
}

#[test]
fn test_normalization_idempotency_and_special_cases() {
    let policies = [
        NormalizationPolicy::Lowercase,
        NormalizationPolicy::Nfkc,
        NormalizationPolicy::GermanUmlauts,
        NormalizationPolicy::Complete,
    ];

    let samples = [
        "Großschreibung und STRAßE",
        "Ligaturen: ﬁnding ﬂow ﬆart",
        "Umlaute: ÄÖÜäöüß",
        "Emoji mit ZWJ: 👨‍👩‍👧‍👦 Familie 🦀",
        "Türkisch: İSTANBUL",
        "RTL-Arabisch: مرحبا بالعالم",
        "Kombinierende Zeichen: e\u{0301} (é)",
    ];

    for policy in &policies {
        for s in &samples {
            let n1 = policy.normalize(s);
            let n2 = policy.normalize(&n1);
            assert_eq!(
                n1, n2,
                "Normalisierung mit {:?} muss idempotent sein für '{}'",
                policy, s
            );
        }
    }
}

#[test]
fn test_pipeline_limits_enforcement() {
    let limits = PipelineLimits {
        max_input_length: 50,
        max_token_length: 10,
        max_tokens_per_doc: 5,
        overflow_policy: OverflowPolicy::Error,
    };

    let p = AnalyzerPipeline::new("test_limits", 1).with_limits(limits);

    // 1. Input too long
    let long_input = "a".repeat(100);
    assert!(p.analyze(&long_input).is_err());

    // 2. Token too long
    let long_word = "Supercalifragilisticexpialidocious short";
    assert!(p.analyze(long_word).is_err());

    // 3. Too many tokens
    let many_words = "one two three four five six seven";
    assert!(p.analyze(many_words).is_err());

    // 4. Truncate mode
    let limits_trunc = PipelineLimits {
        max_input_length: 50,
        max_token_length: 10,
        max_tokens_per_doc: 3,
        overflow_policy: OverflowPolicy::Truncate,
    };
    let p_trunc = AnalyzerPipeline::new("test_trunc", 1).with_limits(limits_trunc);
    let tokens = p_trunc.analyze(many_words).unwrap();
    assert_eq!(
        tokens.len(),
        3,
        "Truncate mode must limit token count to max_tokens_per_doc"
    );
}

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(5000))]

    #[test]
    fn prop_pipeline_arbitrary_utf8_never_panics_and_offsets_valid(s in ".*") {
        let p_en = AnalyzerPipeline::default_english();
        let p_de = AnalyzerPipeline::default_german();

        if let Ok(tokens) = p_en.analyze(&s) {
            for t in tokens {
                prop_assert!(s.is_char_boundary(t.start_offset));
                prop_assert!(s.is_char_boundary(t.end_offset));
                prop_assert!(t.start_offset <= t.end_offset);
            }
        }

        if let Ok(tokens) = p_de.analyze(&s) {
            for t in tokens {
                prop_assert!(s.is_char_boundary(t.start_offset));
                prop_assert!(s.is_char_boundary(t.end_offset));
                prop_assert!(t.start_offset <= t.end_offset);
            }
        }
    }

    #[test]
    fn prop_normalization_idempotency_arbitrary_utf8(s in ".*") {
        let norm = NormalizationPolicy::Complete;
        let step1 = norm.normalize(&s);
        let step2 = norm.normalize(&step1);
        prop_assert_eq!(step1, step2);
    }
}
