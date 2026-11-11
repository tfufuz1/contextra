#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]
//! Known-Answer-Tests (KAT) für `contextra-privacy`.
//!
//! # Zweck
//! Verifiziert die Erkennungsleistung und Determiniertheit gegen definierte Korpora
//! in `tests/corpus/` (IBAN, E-Mail, Telefon, Kreditkarten).

use contextra_privacy::RuleBasedEntityRecognizer;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct TestCase {
    input: String,
    expected_match: bool,
    expected_value: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CorpusFile {
    category: String,
    cases: Vec<TestCase>,
}

fn run_corpus_test(filename: &str) {
    let corpus_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("corpus")
        .join(filename);

    let content = fs::read_to_string(&corpus_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read corpus file {}: {}",
            corpus_path.display(),
            e
        )
    });

    let corpus: CorpusFile = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse corpus file {}: {}", filename, e));

    let recognizer = RuleBasedEntityRecognizer::new();

    for (idx, case) in corpus.cases.iter().enumerate() {
        let matches = recognizer.recognize(&case.input);
        let category_matches: Vec<_> = matches
            .into_iter()
            .filter(|(_range, cat)| cat == &corpus.category)
            .collect();

        if case.expected_match {
            assert!(
                !category_matches.is_empty(),
                "Corpus '{}' Case #{}: Expected match for category '{}', but found none in '{}'",
                filename,
                idx,
                corpus.category,
                case.input
            );
            if let Some(ref expected_val) = case.expected_value {
                let matched_texts: Vec<&str> = category_matches
                    .iter()
                    .map(|(range, _)| &case.input[range.clone()])
                    .collect();
                assert!(
                    matched_texts
                        .iter()
                        .any(|&m| m == expected_val || expected_val.contains(m)),
                    "Corpus '{}' Case #{}: Expected value '{}', matched: {:?}",
                    filename,
                    idx,
                    expected_val,
                    matched_texts
                );
            }
        } else {
            assert!(
                category_matches.is_empty(),
                "Corpus '{}' Case #{}: Expected NO match for category '{}', but found {:?} in '{}'",
                filename,
                idx,
                corpus.category,
                category_matches,
                case.input
            );
        }
    }
}

#[test]
fn test_kat_iban_corpus() {
    run_corpus_test("iban.json");
}

#[test]
fn test_kat_email_corpus() {
    run_corpus_test("email.json");
}

#[test]
fn test_kat_phone_corpus() {
    run_corpus_test("phone.json");
}

#[test]
fn test_kat_credit_card_corpus() {
    run_corpus_test("credit_card.json");
}
