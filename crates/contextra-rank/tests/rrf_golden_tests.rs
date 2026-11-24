//! # Golden-Tests für Reciprocal Rank Fusion (RRF)
//!
//! **Crate:** `contextra-rank`
//!
//! Alle Erwartungswerte in dieser Testsuite sind per Hand gemäß der RRF-Formel
//! $\text{score}(d) = \sum_{l} \frac{w_l}{k + \text{rank}_l(d)}$
//! exakt vorgerechnet und als Kommentar dokumentiert. Keiner der Erwartungswerte
//! wird durch Aufruf einer Fusionsfunktion erzeugt (Anforderung WP-61).
//!
//! Standard-Parameter:
//! In `weighted_reciprocal_rank_fusion` ist $k = 60$ fest verankert.
//! Ränge sind 1-basiert ($\text{rank} \ge 1$).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::doc_lazy_continuation,
    clippy::doc_overindented_list_items
)]

use contextra_rank::fusion::{weighted_reciprocal_rank_fusion, SearchResult};

fn make_doc(id: &str, score: f32) -> SearchResult {
    SearchResult {
        id: id.to_string(),
        score,
        metadata: None,
        matched_signals: vec![],
        provenance: None,
    }
}

/// Fall 1: 2 Listen (Overlap und Disjoint)
/// Liste 1 (Text, w=1.0):   [doc_A, doc_B]
/// Liste 2 (Vector, w=1.0): [doc_B, doc_C]
///
/// Handrechnung ($k=60$):
/// - doc_A: Rang 1 in L1 -> 1 / (60 + 1) = 1 / 61 ≈ 0.0163934426
/// - doc_B: Rang 2 in L1 -> 1 / (60 + 2) = 1 / 62 ≈ 0.0161290323
///          Rang 1 in L2 -> 1 / (60 + 1) = 1 / 61 ≈ 0.0163934426
///          Summe = 1/61 + 1/62 = (62 + 61) / (61 * 62) = 123 / 3782 ≈ 0.0325224749
/// - doc_C: Rang 2 in L2 -> 1 / (60 + 2) = 1 / 62 ≈ 0.0161290323
///
/// Rangfolge:
/// 1. doc_B (Score ≈ 0.032522475)
/// 2. doc_A (Score ≈ 0.016393443)
/// 3. doc_C (Score ≈ 0.016129032)
#[test]
fn test_golden_two_lists() {
    let list1 = vec![make_doc("doc_A", 10.0), make_doc("doc_B", 5.0)];
    let list2 = vec![make_doc("doc_B", 0.9), make_doc("doc_C", 0.8)];

    let input = vec![
        ("text".to_string(), list1, 1.0_f32),
        ("vector".to_string(), list2, 1.0_f32),
    ];

    let fused = weighted_reciprocal_rank_fusion(input, 10);
    assert_eq!(fused.len(), 3);

    // doc_B
    assert_eq!(fused[0].id, "doc_B");
    let expected_b = (1.0 / 61.0_f32) + (1.0 / 62.0_f32);
    assert!((fused[0].score - expected_b).abs() < 1e-6);

    // doc_A
    assert_eq!(fused[1].id, "doc_A");
    let expected_a = 1.0 / 61.0_f32;
    assert!((fused[1].score - expected_a).abs() < 1e-6);

    // doc_C
    assert_eq!(fused[2].id, "doc_C");
    let expected_c = 1.0 / 62.0_f32;
    assert!((fused[2].score - expected_c).abs() < 1e-6);
}

/// Fall 2: 3 Listen mit Schnittmengen
/// Liste 1 (Text, w=1.0):   [doc_1, doc_2, doc_3]
/// Liste 2 (Vector, w=1.0): [doc_2, doc_1, doc_4]
/// Liste 3 (Graph, w=1.0):  [doc_3, doc_4, doc_1]
///
/// Handrechnung ($k=60$):
/// - doc_1: Rang 1 in L1 (1/61), Rang 2 in L2 (1/62), Rang 3 in L3 (1/63)
///          Summe = 1/61 + 1/62 + 1/63
///          1/61 ≈ 0.0163934426
///          1/62 ≈ 0.0161290323
///          1/63 ≈ 0.0158730159
///          Summe ≈ 0.0483954908
/// - doc_2: Rang 2 in L1 (1/62), Rang 1 in L2 (1/61)
///          Summe = 1/62 + 1/61 ≈ 0.0325224749
/// - doc_3: Rang 3 in L1 (1/63), Rang 1 in L3 (1/61)
///          Summe = 1/63 + 1/61 ≈ 0.0322664585
/// - doc_4: Rang 3 in L2 (1/63), Rang 2 in L3 (1/62)
///          Summe = 1/63 + 1/62 ≈ 0.0320020482
///
/// Rangfolge:
/// 1. doc_1 (Score ≈ 0.04839549)
/// 2. doc_2 (Score ≈ 0.032522475)
/// 3. doc_3 (Score ≈ 0.03226646)
/// 4. doc_4 (Score ≈ 0.03200205)
#[test]
fn test_golden_three_lists() {
    let list1 = vec![
        make_doc("doc_1", 30.0),
        make_doc("doc_2", 20.0),
        make_doc("doc_3", 10.0),
    ];
    let list2 = vec![
        make_doc("doc_2", 0.95),
        make_doc("doc_1", 0.90),
        make_doc("doc_4", 0.70),
    ];
    let list3 = vec![
        make_doc("doc_3", 0.8),
        make_doc("doc_4", 0.6),
        make_doc("doc_1", 0.4),
    ];

    let input = vec![
        ("text".to_string(), list1, 1.0_f32),
        ("vector".to_string(), list2, 1.0_f32),
        ("graph".to_string(), list3, 1.0_f32),
    ];

    let fused = weighted_reciprocal_rank_fusion(input, 10);
    assert_eq!(fused.len(), 4);

    assert_eq!(fused[0].id, "doc_1");
    let expected_1 = (1.0 / 61.0_f32) + (1.0 / 62.0_f32) + (1.0 / 63.0_f32);
    assert!((fused[0].score - expected_1).abs() < 1e-6);

    assert_eq!(fused[1].id, "doc_2");
    let expected_2 = (1.0 / 62.0_f32) + (1.0 / 61.0_f32);
    assert!((fused[1].score - expected_2).abs() < 1e-6);

    assert_eq!(fused[2].id, "doc_3");
    let expected_3 = (1.0 / 63.0_f32) + (1.0 / 61.0_f32);
    assert!((fused[2].score - expected_3).abs() < 1e-6);

    assert_eq!(fused[3].id, "doc_4");
    let expected_4 = (1.0 / 63.0_f32) + (1.0 / 62.0_f32);
    assert!((fused[3].score - expected_4).abs() < 1e-6);
}

/// Fall 3: Gleichstand (Tie-Break)
/// Liste 1 (Text, w=1.0):   [doc_b, doc_a]
/// Liste 2 (Vector, w=1.0): [doc_a, doc_b]
///
/// Handrechnung ($k=60$):
/// - doc_b: Rang 1 in L1 (1/61), Rang 2 in L2 (1/62) -> Summe = 1/61 + 1/62 ≈ 0.032522475
/// - doc_a: Rang 2 in L1 (1/62), Rang 1 in L2 (1/61) -> Summe = 1/62 + 1/61 ≈ 0.032522475
/// Beide Scores sind mathematisch und numerisch identisch.
/// Spezifikationsvorgabe: Deterministische Gleichstandsauflösung nach ID lexikografisch aufsteigend.
/// Daher MUSS doc_a vor doc_b rangieren ("doc_a" < "doc_b").
#[test]
fn test_golden_tie_break_ascending_id() {
    let list1 = vec![make_doc("doc_b", 1.0), make_doc("doc_a", 0.5)];
    let list2 = vec![make_doc("doc_a", 1.0), make_doc("doc_b", 0.5)];

    let input = vec![
        ("s1".to_string(), list1, 1.0_f32),
        ("s2".to_string(), list2, 1.0_f32),
    ];

    let fused = weighted_reciprocal_rank_fusion(input, 10);
    assert_eq!(fused.len(), 2);

    let expected_score = (1.0 / 61.0_f32) + (1.0 / 62.0_f32);
    assert_eq!(fused[0].id, "doc_a");
    assert!((fused[0].score - expected_score).abs() < 1e-6);

    assert_eq!(fused[1].id, "doc_b");
    assert!((fused[1].score - expected_score).abs() < 1e-6);
}

/// Fall 4: ID fehlt in einer Liste
/// Liste 1 (Text, w=1.0):   [doc_common, doc_only1]
/// Liste 2 (Vector, w=1.0): [doc_common, doc_only2]
///
/// Handrechnung ($k=60$):
/// - doc_common: Rang 1 in L1 (1/61), Rang 1 in L2 (1/61) -> 2 / 61 ≈ 0.032786885
/// - doc_only1:  Rang 2 in L1 (1/62), fehlt in L2 (0)   -> 1 / 62 ≈ 0.016129032
/// - doc_only2:  fehlt in L1 (0), Rang 2 in L2 (1/62)   -> 1 / 62 ≈ 0.016129032
///
/// doc_only1 und doc_only2 haben identische Scores (1/62).
/// Tie-Break: "doc_only1" < "doc_only2".
///
/// Rangfolge:
/// 1. doc_common (Score 2/61 ≈ 0.032786885)
/// 2. doc_only1  (Score 1/62 ≈ 0.016129032)
/// 3. doc_only2  (Score 1/62 ≈ 0.016129032)
#[test]
fn test_golden_missing_id_in_one_list() {
    let list1 = vec![make_doc("doc_common", 1.0), make_doc("doc_only1", 0.5)];
    let list2 = vec![make_doc("doc_common", 0.9), make_doc("doc_only2", 0.4)];

    let input = vec![
        ("text".to_string(), list1, 1.0_f32),
        ("vector".to_string(), list2, 1.0_f32),
    ];

    let fused = weighted_reciprocal_rank_fusion(input, 10);
    assert_eq!(fused.len(), 3);

    assert_eq!(fused[0].id, "doc_common");
    let expected_common = 2.0 / 61.0_f32;
    assert!((fused[0].score - expected_common).abs() < 1e-6);

    let expected_single = 1.0 / 62.0_f32;
    assert_eq!(fused[1].id, "doc_only1");
    assert!((fused[1].score - expected_single).abs() < 1e-6);

    assert_eq!(fused[2].id, "doc_only2");
    assert!((fused[2].score - expected_single).abs() < 1e-6);
}

/// Fall 5: Gewichte 0 und 1
/// Liste 1 (Aktiv, w=1.0):       [doc_active]
/// Liste 2 (Inaktiv, w=0.0):     [doc_inactive, doc_active]
///
/// Gemäß Fusionsspezifikation:
/// Kanäle mit Gewicht <= 0.0 oder nicht-endlichen Gewichten werden übersprungen.
/// Handrechnung:
/// - doc_active:   Rang 1 in L1 mit w=1.0 -> 1.0 / (60 + 1) = 1 / 61 ≈ 0.016393443.
///                 L2 hat w=0.0 und trägt 0 bei.
/// - doc_inactive: Nur in L2 (w=0.0) enthalten -> wird nicht akkumuliert bzw. erhält 0 Treffer.
///
/// Rangfolge:
/// 1. doc_active (Score 1/61 ≈ 0.016393443)
/// doc_inactive erscheint gar nicht im Ergebnis.
#[test]
fn test_golden_weights_zero_and_one() {
    let list1 = vec![make_doc("doc_active", 1.0)];
    let list2 = vec![make_doc("doc_inactive", 2.0), make_doc("doc_active", 1.0)];

    let input = vec![
        ("active".to_string(), list1, 1.0_f32),
        ("inactive".to_string(), list2, 0.0_f32),
    ];

    let fused = weighted_reciprocal_rank_fusion(input, 10);
    assert_eq!(fused.len(), 1);
    assert_eq!(fused[0].id, "doc_active");

    let expected_active = 1.0 / 61.0_f32;
    assert!((fused[0].score - expected_active).abs() < 1e-6);
}

/// Fall 6: Leere Eingabe
/// (a) Leere Liste von Signalen (`vec![]`)
/// (b) Signale mit leeren Ergebnislisten (`vec![("s1", vec![], 1.0)]`)
/// (c) max_results = 0
///
/// Erwartungswert:
/// Leerer Vektor `vec![]`, keine Panics.
#[test]
fn test_golden_empty_inputs() {
    // (a) Leere Signalliste
    let fused_a = weighted_reciprocal_rank_fusion(vec![], 10);
    assert!(fused_a.is_empty());

    // (b) Signal mit leerer Liste
    let input_b = vec![
        ("s1".to_string(), vec![], 1.0_f32),
        ("s2".to_string(), vec![], 1.0_f32),
    ];
    let fused_b = weighted_reciprocal_rank_fusion(input_b, 10);
    assert!(fused_b.is_empty());

    // (c) max_results = 0
    let input_c = vec![("s1".to_string(), vec![make_doc("doc_1", 1.0)], 1.0_f32)];
    let fused_c = weighted_reciprocal_rank_fusion(input_c, 0);
    assert!(fused_c.is_empty());
}

/// Fall 7: Doppelte IDs innerhalb einer einzelnen Liste
/// Liste 1 (Text, w=1.0): [doc_dup (Score 10.0), doc_other (Score 5.0), doc_dup (Score 1.0)]
///
/// Handrechnung ($k=60$):
/// Gemäß RRF-Spezifikation wird die erste bzw. beste Position (Rang 1) für doc_dup gewertet,
/// Duplikate innerhalb derselben Liste erhöhen den RRF-Score nicht mehrfach:
/// - doc_dup:   Rang 1 in L1 -> 1 / (60 + 1) = 1 / 61 ≈ 0.016393443
/// - doc_other: Rang 2 in L1 -> 1 / (60 + 2) = 1 / 62 ≈ 0.016129032
///
/// Rangfolge:
/// 1. doc_dup   (Score 1/61 ≈ 0.016393443)
/// 2. doc_other (Score 1/62 ≈ 0.016129032)
#[test]
fn test_golden_duplicate_ids_within_single_list() {
    let list1 = vec![
        make_doc("doc_dup", 10.0),
        make_doc("doc_other", 5.0),
        make_doc("doc_dup", 1.0),
    ];

    let input = vec![("text".to_string(), list1, 1.0_f32)];

    let fused = weighted_reciprocal_rank_fusion(input, 10);
    // Nur 2 distinkte Dokumente im Ergebnis
    assert_eq!(fused.len(), 2);

    assert_eq!(fused[0].id, "doc_dup");
    let expected_dup = 1.0 / 61.0_f32;
    assert!((fused[0].score - expected_dup).abs() < 1e-6);

    assert_eq!(fused[1].id, "doc_other");
    let expected_other = 1.0 / 62.0_f32;
    assert!((fused[1].score - expected_other).abs() < 1e-6);
}
