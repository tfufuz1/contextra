//! WP-72: Benchmark and measurement of RuleBasedEntityExtractor against gold standard dataset.

use contextra_graph::entity_extraction::{EntityExtractor, RuleBasedEntityExtractor};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GoldExpected {
    source: String,
    target: String,
    relation_type: String,
}

#[derive(Debug, Deserialize)]
struct GoldItem {
    id: u32,
    lang: String,
    text: String,
    expected: Vec<GoldExpected>,
}

#[test]
fn test_measure_entity_extraction_gold_standard() {
    let json_bytes = include_bytes!("data/entity_gold.json");
    let items: Vec<GoldItem> = serde_json::from_slice(json_bytes).expect("Valid gold JSON");

    assert!(
        items.len() >= 30,
        "Gold standard must have at least 30 items"
    );

    let extractor = RuleBasedEntityExtractor::default();

    let mut total_expected = 0;
    let mut total_extracted = 0;
    let mut true_positives = 0;
    let mut false_positives = 0;
    let mut false_negatives = 0;

    let mut de_tp = 0;
    let mut de_fp = 0;
    let mut de_fn = 0;

    let mut en_tp = 0;
    let mut en_fp = 0;
    let mut en_fn = 0;

    let mut error_examples = Vec::new();

    for item in &items {
        let extracted = extractor.extract(&item.text);
        total_extracted += extracted.len();
        total_expected += item.expected.len();

        let mut matched_expected = vec![false; item.expected.len()];

        for ext in &extracted {
            let mut match_found = false;
            for (i, exp) in item.expected.iter().enumerate() {
                if !matched_expected[i]
                    && ext.source.eq_ignore_ascii_case(&exp.source)
                    && ext.target.eq_ignore_ascii_case(&exp.target)
                    && ext.relation_type == exp.relation_type
                {
                    matched_expected[i] = true;
                    match_found = true;
                    break;
                }
            }

            if match_found {
                true_positives += 1;
                if item.lang == "de" {
                    de_tp += 1;
                } else {
                    en_tp += 1;
                }
            } else {
                false_positives += 1;
                if item.lang == "de" {
                    de_fp += 1;
                } else {
                    en_fp += 1;
                }
                error_examples.push(format!(
                    "FP in #{}: text='{}' -> extracted ({}, {}, {}) but not expected",
                    item.id, item.text, ext.source, ext.target, ext.relation_type
                ));
            }
        }

        for (i, exp) in item.expected.iter().enumerate() {
            if !matched_expected[i] {
                false_negatives += 1;
                if item.lang == "de" {
                    de_fn += 1;
                } else {
                    en_fn += 1;
                }
                error_examples.push(format!(
                    "FN in #{}: text='{}' -> missed expected ({}, {}, {})",
                    item.id, item.text, exp.source, exp.target, exp.relation_type
                ));
            }
        }
    }

    let precision = if total_extracted > 0 {
        true_positives as f64 / total_extracted as f64
    } else {
        0.0
    };
    let recall = if total_expected > 0 {
        true_positives as f64 / total_expected as f64
    } else {
        0.0
    };
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };

    let de_precision = if de_tp + de_fp > 0 {
        de_tp as f64 / (de_tp + de_fp) as f64
    } else {
        0.0
    };
    let de_recall = if de_tp + de_fn > 0 {
        de_tp as f64 / (de_tp + de_fn) as f64
    } else {
        0.0
    };

    let en_precision = if en_tp + en_fp > 0 {
        en_tp as f64 / (en_tp + en_fp) as f64
    } else {
        0.0
    };
    let en_recall = if en_tp + en_fn > 0 {
        en_tp as f64 / (en_tp + en_fn) as f64
    } else {
        0.0
    };

    println!("=== ENTITY EXTRACTION BENCHMARK RESULTS ===");
    println!("Total Sentences: {}", items.len());
    println!("Total Expected Relations: {}", total_expected);
    println!("Total Extracted Relations: {}", total_extracted);
    println!("True Positives (TP): {}", true_positives);
    println!("False Positives (FP): {}", false_positives);
    println!("False Negatives (FN): {}", false_negatives);
    println!("Overall Precision: {:.2}%", precision * 100.0);
    println!("Overall Recall:    {:.2}%", recall * 100.0);
    println!("Overall F1-Score:  {:.2}%", f1 * 100.0);
    println!(
        "DE Precision: {:.2}%, DE Recall: {:.2}%",
        de_precision * 100.0,
        de_recall * 100.0
    );
    println!(
        "EN Precision: {:.2}%, EN Recall: {:.2}%",
        en_precision * 100.0,
        en_recall * 100.0
    );
    println!("Error Examples Count: {}", error_examples.len());
    for ex in error_examples.iter().take(10) {
        println!(" - {}", ex);
    }

    assert!(
        true_positives > 0,
        "Extractor must produce at least some true positives"
    );
}
