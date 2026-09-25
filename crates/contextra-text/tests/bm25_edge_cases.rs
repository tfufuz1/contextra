// FILE-CONTEXT: Integration tests for BM25 score edge cases.
// ZWECK: Testet score_term_with_params und score_term mit degenerierten/Grenzfall-Eingaben.
// INVARIANTEN: BM25-Scores dürfen niemals NaN oder Infinity sein und müssen für alle Kantenfälle definiert reagieren.

use contextra_text::bm25::{score_term, score_term_with_params};

/// Test 1: tf=0 (Term kommt nicht im Dokument vor) → Score soll 0.0 sein.
#[test]
fn test_bm25_tf_zero() {
    let score1 = score_term_with_params(0, 100, 100.0, 5, 50, 1.5, 0.75);
    assert_eq!(score1, 0.0, "Score for tf = 0 must be exactly 0.0");

    let score2 = score_term(0, 100, 100.0, 5, 50);
    assert_eq!(score2, 0.0, "score_term for tf = 0 must be exactly 0.0");
}

/// Test 2: n=0 (leeres Korpus) → Score soll 0.0 sein, kein Panic durch Division durch Null in der IDF-Formel.
#[test]
fn test_bm25_n_zero() {
    let score1 = score_term_with_params(1, 100, 100.0, 0, 0, 1.5, 0.75);
    assert_eq!(score1, 0.0, "Score for n = 0 must be exactly 0.0");

    let score2 = score_term(1, 100, 100.0, 0, 0);
    assert_eq!(score2, 0.0, "score_term for n = 0 must be exactly 0.0");
}

/// Test 3: df=0 (Term in keinem Dokument, aber n>0) → Score soll 0.0 sein (nicht negativ, kein NaN).
#[test]
fn test_bm25_df_zero() {
    let score1 = score_term_with_params(1, 100, 100.0, 0, 100, 1.5, 0.75);
    assert_eq!(score1, 0.0, "Score for df = 0 must be 0.0");

    let score2 = score_term(1, 100, 100.0, 0, 100);
    assert_eq!(score2, 0.0, "score_term for df = 0 must be 0.0");
}

/// Test 4: df > n (inkonsistente Statistiken, z. B. df=150, n=100) → Score soll ≥ 0.0 sein.
#[test]
fn test_bm25_df_greater_than_n() {
    let score1 = score_term_with_params(2, 100, 100.0, 150, 100, 1.5, 0.75);
    assert!(
        score1 >= 0.0,
        "Score for df > n must be non-negative, got {score1}"
    );
    assert!(
        score1.is_finite(),
        "Score for df > n must be finite, got {score1}"
    );

    let score2 = score_term(2, 100, 100.0, 150, 100);
    assert!(score2 >= 0.0);
    assert!(score2.is_finite());
}

/// Test 5: avg_doc_len = 0.0 (bei leerem Korpus) → kein Division-durch-Null-Panic, definiertes Verhalten.
#[test]
fn test_bm25_avg_doc_len_zero() {
    let score_n0 = score_term_with_params(1, 0, 0.0, 0, 0, 1.5, 0.75);
    assert_eq!(
        score_n0, 0.0,
        "Score for empty corpus (n=0, avg_doc_len=0.0) must be 0.0"
    );

    let score_pos = score_term_with_params(1, 10, 0.0, 2, 10, 1.5, 0.75);
    assert!(
        score_pos.is_finite(),
        "Score with avg_doc_len = 0.0 must be finite"
    );
    assert!(
        score_pos >= 0.0,
        "Score with avg_doc_len = 0.0 must be non-negative"
    );
}

/// Test 6: doc_len = 0 (leeres Dokument) → Score soll 0.0 sein für tf = 0.
#[test]
fn test_bm25_doc_len_zero() {
    // Standard empty document in a search engine: tf = 0 and doc_len = 0
    let score = score_term_with_params(0, 0, 100.0, 5, 50, 1.5, 0.75);
    assert_eq!(score, 0.0, "Score for an empty document (tf=0, doc_len=0) must be 0.0");

    let score_std = score_term(0, 0, 100.0, 5, 50);
    assert_eq!(score_std, 0.0, "score_term for an empty document must be 0.0");
}

/// Test 7: k1 = 0.0 (keine TF-Sättigung) → Score soll gleich dem reinen IDF-Wert entsprechen (unabhängig von tf > 0 und doc_len).
#[test]
fn test_bm25_k1_zero() {
    let tf1 = 1;
    let tf2 = 10;
    let doc_len1 = 50;
    let doc_len2 = 500;
    let avg_doc_len = 100.0;
    let df = 10;
    let n = 100;

    let score1 = score_term_with_params(tf1, doc_len1, avg_doc_len, df, n, 0.0, 0.75);
    let score2 = score_term_with_params(tf2, doc_len2, avg_doc_len, df, n, 0.0, 0.75);

    // Robertson-Spärck-Jones BM25+ IDF formula used in score_term_with_params:
    let n_f = n as f32;
    let df_f = df as f32;
    let expected_idf = (1.0 + (n_f - df_f + 0.5) / (df_f + 0.5)).ln();

    assert!(
        (score1 - expected_idf).abs() < 1e-6,
        "When k1=0, score ({score1}) must equal pure IDF ({expected_idf})"
    );
    assert_eq!(
        score1, score2,
        "When k1=0, score must be independent of tf (>0) and doc_len"
    );
}

/// Test 8: b = 0.0 (keine Längennormalisierung) → Score soll unabhängig vom doc_len/avg_doc_len-Verhältnis sein.
#[test]
fn test_bm25_b_zero() {
    let score_short_doc = score_term_with_params(2, 20, 100.0, 5, 100, 1.5, 0.0);
    let score_long_doc = score_term_with_params(2, 500, 100.0, 5, 100, 1.5, 0.0);

    assert_eq!(
        score_short_doc, score_long_doc,
        "When b=0.0, score must be identical regardless of doc_len"
    );
}

/// Test 9: b = 1.0 (volle Längennormalisierung) → maximaler Effekt der Längennormalisierung (Score sinkt merklich bei doc_len > avg_doc_len im Vergleich zu b=0.0).
#[test]
fn test_bm25_b_one() {
    let tf = 2;
    let doc_len = 300; // doc_len > avg_doc_len (300 > 100)
    let avg_doc_len = 100.0;
    let df = 5;
    let n = 100;
    let k1 = 1.5;

    let score_b0 = score_term_with_params(tf, doc_len, avg_doc_len, df, n, k1, 0.0);
    let score_b1 = score_term_with_params(tf, doc_len, avg_doc_len, df, n, k1, 1.0);

    assert!(
        score_b1 < score_b0,
        "Score with b=1.0 ({score_b1}) must be strictly smaller than with b=0.0 ({score_b0}) when doc_len > avg_doc_len"
    );
}

/// Test 10: tf = u32::MAX (Integer-Overflow-Risiko bei Cast zu f32) → kein Panic, ein endlicher Score wird zurückgegeben.
#[test]
fn test_bm25_tf_u32_max() {
    let score = score_term_with_params(u32::MAX, 100, 100.0, 5, 100, 1.5, 0.75);

    assert!(
        score.is_finite(),
        "Score for tf = u32::MAX must be finite, got {score}"
    );
    assert!(
        score >= 0.0,
        "Score for tf = u32::MAX must be non-negative, got {score}"
    );
}
