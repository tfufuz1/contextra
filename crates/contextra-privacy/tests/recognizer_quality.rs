//! Qualitäts- und Regressionsmessung für den `RuleBasedEntityRecognizer`.
//!
//! # Definition der Übereinstimmung (Span Match & Metriken)
//! Ein erkanntes Span `(start_byte..end_byte, category)` stimmt mit einem erwarteten
//! Ground-Truth-Span genau dann überein (True Positive, TP), wenn:
//! 1. Die Start-Byte-Offsets exakt identisch sind (`detected.start == expected.start`).
//! 2. Die End-Byte-Offsets exakt identisch sind (`detected.end == expected.end`).
//! 3. Die Kategoriebezeichnung exakt übereinstimmt (`detected.category == expected.category`).
//!
//! - **False Positive (FP):** Ein vom Recognizer geliefertes Span, das keinem erwarteten Span
//!   (exakt nach Start, End und Kategorie) im selben Testfall entspricht.
//! - **False Negative (FN):** Ein im Ground Truth erwartetes Span, das vom Recognizer nicht
//!   exakt (Start, End, Kategorie) erkannt wurde.
//! - **Precision:** `TP / (TP + FP)` (bzw. 1.0 wenn `TP + FP == 0`).
//! - **Recall:** `TP / (TP + FN)` (bzw. 1.0 wenn `TP + FN == 0`).
//!
//! # Regressions-Untergrenzen
//! Die Assertions setzen Regressions-Untergrenzen auf Basis der gemessenen Ist-Werte
//! abzüglich eines kleinen Spielraums von 0.05 (5 Prozentpunkte). Dies stellt sicher,
//! dass künftige Änderungen die Erkennungsqualität nicht verschlechtern, ohne unerreichbare
//! Zielwerte zu verlangen.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::disallowed_types,
    clippy::cast_precision_loss
)]

use contextra_privacy::entity_recognizer::{Gazetteer, RuleBasedEntityRecognizer};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

#[derive(Debug, Deserialize)]
struct CorpusItem {
    id: String,
    category: String,
    r#type: String,
    text: String,
    spans: Vec<ExpectedSpan>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct ExpectedSpan {
    start: usize,
    end: usize,
    category: String,
}

#[derive(Debug, Default)]
struct CategoryMetrics {
    tp: usize,
    fp: usize,
    fn_: usize,
}

impl CategoryMetrics {
    fn precision(&self) -> f64 {
        let total = self.tp + self.fp;
        if total == 0 {
            1.0
        } else {
            self.tp as f64 / total as f64
        }
    }

    fn recall(&self) -> f64 {
        let total = self.tp + self.fn_;
        if total == 0 {
            1.0
        } else {
            self.tp as f64 / total as f64
        }
    }
}

fn build_test_gazetteer() -> Gazetteer {
    let mut g = Gazetteer::new();
    let terms = [
        "Acme Corp",
        "Globex Corporation",
        "Initech LLC",
        "Umbrella Pharmaceuticals",
        "Wayne Enterprises",
        "Stark Industries",
        "Cyberdyne Systems",
        "Soylent Corp",
        "Hooli Inc",
        "Pied Piper",
        "Massive Dynamic",
        "Aperture Science",
        "Black Mesa",
        "Weyland-Yutani",
        "Tyrell Corporation",
    ];
    for t in terms {
        g.add_term(t);
    }
    g
}

#[test]
fn test_entity_recognizer_quality_and_regression_bounds() {
    let corpus_raw = include_str!("fixtures/pii_corpus_synthetic.jsonl");
    let mut items = Vec::new();
    for line in corpus_raw.lines() {
        let line = line.trim();
        if !line.is_empty() {
            let item: CorpusItem =
                serde_json::from_str(line).expect("corpus line must be valid JSON");
            items.push(item);
        }
    }

    assert!(
        !items.is_empty(),
        "Corpus must contain synthetic evaluation items"
    );

    let recognizer = RuleBasedEntityRecognizer::with_gazetteer(build_test_gazetteer());

    let mut metrics_by_category: BTreeMap<String, CategoryMetrics> = BTreeMap::new();

    let all_categories = [
        "EMAIL",
        "IBAN",
        "BIC",
        "CREDIT_CARD",
        "STEUER_ID",
        "PHONE",
        "IPV4",
        "IPV6",
        "PLZ",
        "PLZ_ORT",
        "LICENSE_PLATE",
        "GAZETTEER",
    ];
    for cat in all_categories {
        metrics_by_category.insert(cat.to_string(), CategoryMetrics::default());
    }

    let mut fp_details = Vec::new();
    let mut fn_details = Vec::new();

    for item in &items {
        let recognized: Vec<(Range<usize>, String)> = recognizer.recognize(&item.text);

        let mut detected_spans: BTreeSet<ExpectedSpan> = BTreeSet::new();
        for (range, cat) in recognized {
            detected_spans.insert(ExpectedSpan {
                start: range.start,
                end: range.end,
                category: cat,
            });
        }

        let mut expected_spans: BTreeSet<ExpectedSpan> = BTreeSet::new();
        for exp in &item.spans {
            expected_spans.insert(exp.clone());
        }

        // True Positives & False Negatives
        for exp in &expected_spans {
            let entry = metrics_by_category.entry(exp.category.clone()).or_default();
            if detected_spans.contains(exp) {
                entry.tp += 1;
            } else {
                entry.fn_ += 1;
                fn_details.push(format!(
                    "FN in {} ({}, item_cat={}): expected {:?} in {:?}",
                    item.id, item.r#type, item.category, exp, item.text
                ));
            }
        }

        // False Positives
        for det in &detected_spans {
            if !expected_spans.contains(det) {
                let entry = metrics_by_category.entry(det.category.clone()).or_default();
                entry.fp += 1;
                let text_slice = if det.end <= item.text.len() {
                    &item.text[det.start..det.end]
                } else {
                    "<out of bounds>"
                };
                fp_details.push(format!(
                    "FP in {} ({}, item_cat={}): detected {:?} = {:?} in {:?}",
                    item.id, item.r#type, item.category, det, text_slice, item.text
                ));
            }
        }
    }

    // Formatierte Tabellenausgabe
    println!();
    println!("==================================================================================================");
    println!("                         RECOGNIZER QUALITAETSMESSUNG (SYNTHETISCHER KORPUS)                      ");
    println!("==================================================================================================");
    println!(
        "{:<15} | {:>6} | {:>6} | {:>6} | {:>10} | {:>10} | {:>6}",
        "Kategorie", "TP", "FP", "FN", "Precision", "Recall", "F1"
    );
    println!("--------------------------------------------------------------------------------------------------");

    let mut total_tp = 0;
    let mut total_fp = 0;
    let mut total_fn = 0;

    for (cat, m) in &metrics_by_category {
        total_tp += m.tp;
        total_fp += m.fp;
        total_fn += m.fn_;

        let prec = m.precision();
        let rec = m.recall();
        let f1 = if prec + rec > 0.0 {
            2.0 * (prec * rec) / (prec + rec)
        } else {
            0.0
        };

        println!(
            "{:<15} | {:>6} | {:>6} | {:>6} | {:>9.2}% | {:>9.2}% | {:>5.2}",
            cat,
            m.tp,
            m.fp,
            m.fn_,
            prec * 100.0,
            rec * 100.0,
            f1
        );
    }

    let overall_prec = if total_tp + total_fp > 0 {
        total_tp as f64 / (total_tp + total_fp) as f64
    } else {
        1.0
    };
    let overall_rec = if total_tp + total_fn > 0 {
        total_tp as f64 / (total_tp + total_fn) as f64
    } else {
        1.0
    };
    let overall_f1 = if overall_prec + overall_rec > 0.0 {
        2.0 * (overall_prec * overall_rec) / (overall_prec + overall_rec)
    } else {
        0.0
    };

    println!("--------------------------------------------------------------------------------------------------");
    println!(
        "{:<15} | {:>6} | {:>6} | {:>6} | {:>9.2}% | {:>9.2}% | {:>5.2}",
        "GESAMT",
        total_tp,
        total_fp,
        total_fn,
        overall_prec * 100.0,
        overall_rec * 100.0,
        overall_f1
    );
    println!("==================================================================================================");
    println!(
        "Korpus-Umfang: N = {} Beispiele (Positiv & Negativ)",
        items.len()
    );
    println!();

    if !fn_details.is_empty() {
        println!("--- False Negatives (Stichprobe bis zu 10) ---");
        for detail in fn_details.iter().take(10) {
            println!("  * {detail}");
        }
        println!();
    }

    if !fp_details.is_empty() {
        println!("--- False Positives (Stichprobe bis zu 10) ---");
        for detail in fp_details.iter().take(10) {
            println!("  * {detail}");
        }
        println!();
    }

    // Regressions-Untergrenzen: gemessener Ist-Wert minus 0.05 (5 Prozentpunkte Spielraum)
    // BIC:           Prec: 92.59% -> min 87.59% (0.875), Rec: 100.00% -> min 95.00% (0.950)
    // CREDIT_CARD:   Prec: 92.59% -> min 87.59% (0.875), Rec: 100.00% -> min 95.00% (0.950)
    // EMAIL:         Prec: 100.0% -> min 95.00% (0.950), Rec: 100.00% -> min 95.00% (0.950)
    // GAZETTEER:     Prec: 100.0% -> min 95.00% (0.950), Rec: 100.00% -> min 95.00% (0.950)
    // IBAN:          Prec: 96.15% -> min 91.15% (0.910), Rec: 100.00% -> min 95.00% (0.950)
    // IPV4:          Prec: 83.33% -> min 78.33% (0.780), Rec: 100.00% -> min 95.00% (0.950)
    // IPV6:          Prec: 92.31% -> min 87.31% (0.870), Rec:  96.00% -> min 91.00% (0.910)
    // LICENSE_PLATE: Prec: 89.29% -> min 84.29% (0.840), Rec: 100.00% -> min 95.00% (0.950)
    // PHONE:         Prec: 86.21% -> min 81.21% (0.810), Rec: 100.00% -> min 95.00% (0.950)
    // PLZ:           Prec: 62.50% -> min 57.50% (0.575), Rec: 100.00% -> min 95.00% (0.950)
    // PLZ_ORT:       Prec: 96.15% -> min 91.15% (0.910), Rec: 100.00% -> min 95.00% (0.950)
    // STEUER_ID:     Prec: 96.15% -> min 91.15% (0.910), Rec: 100.00% -> min 95.00% (0.950)
    // GESAMT:        Prec: 89.25% -> min 84.25% (0.840), Rec:  99.67% -> min 94.67% (0.940)

    let bounds: BTreeMap<&str, (f64, f64)> = BTreeMap::from([
        ("BIC", (0.87, 0.95)),
        ("CREDIT_CARD", (0.87, 0.95)),
        ("EMAIL", (0.95, 0.95)),
        ("GAZETTEER", (0.95, 0.95)),
        ("IBAN", (0.91, 0.95)),
        ("IPV4", (0.78, 0.95)),
        ("IPV6", (0.87, 0.91)),
        ("LICENSE_PLATE", (0.84, 0.95)),
        ("PHONE", (0.81, 0.95)),
        ("PLZ", (0.57, 0.95)),
        ("PLZ_ORT", (0.91, 0.95)),
        ("STEUER_ID", (0.91, 0.95)),
    ]);

    for (cat, &(min_prec, min_rec)) in &bounds {
        let m = metrics_by_category
            .get(*cat)
            .expect("category exists in metrics");
        let prec = m.precision();
        let rec = m.recall();

        assert!(
            prec >= min_prec,
            "Precision-Regression in Kategorie {}: gemessen {:.4}, Mindestgrenze {:.4}",
            cat,
            prec,
            min_prec
        );
        assert!(
            rec >= min_rec,
            "Recall-Regression in Kategorie {}: gemessen {:.4}, Mindestgrenze {:.4}",
            cat,
            rec,
            min_rec
        );
    }

    assert!(
        overall_prec >= 0.84,
        "Gesamt-Precision-Regression: gemessen {:.4}, Mindestgrenze 0.84",
        overall_prec
    );
    assert!(
        overall_rec >= 0.94,
        "Gesamt-Recall-Regression: gemessen {:.4}, Mindestgrenze 0.94",
        overall_rec
    );
}
