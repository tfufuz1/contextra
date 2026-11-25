// FILE-CONTEXT: Integration tests for field-weighted BM25F scoring (§11.3).
// ZWECK: Prüft Formel-Korrektheit, Degeneration zu einfeldigem BM25 und Zero-Panic Invarianten.

use contextra_text::{score_term_bm25f, FieldId, FieldWeight, BM25F};

#[test]
fn test_bm25f_degeneration_exact_match_single_field() {
    let tf = 3u32;
    let doc_len = 120u32;
    let avg_doc_len = 180.0f32;
    let df = 15u32;
    let n = 500u32;
    let k1 = 1.5f32;
    let b = 0.75f32;

    let std_bm25_score =
        contextra_text::bm25::score_term_with_params(tf, doc_len, avg_doc_len, df, n, k1, b);

    let field_id: FieldId = 0;
    let field_tf = vec![(field_id, tf, doc_len, avg_doc_len)];
    let field_weights = vec![(field_id, 1.0f32, b)];

    let bm25f_score = score_term_bm25f(&field_tf, &field_weights, k1, df, n);

    assert!(
        (std_bm25_score - bm25f_score).abs() < 1e-6,
        "Single-field BM25F ({}) must be equal to standard BM25 ({})",
        bm25f_score,
        std_bm25_score
    );
}

#[test]
fn test_bm25f_title_body_context_prefix_weighting() {
    // Field 0: Title (weight = 3.0, b = 0.75)
    // Field 1: Context Prefix (weight = 2.0, b = 0.75)
    // Field 2: Body (weight = 1.0, b = 0.75)
    let weights = vec![
        (0, 3.0f32, 0.75f32),
        (1, 2.0f32, 0.75f32),
        (2, 1.0f32, 0.75f32),
    ];

    let k1 = 1.5f32;
    let df = 10u32;
    let n = 1000u32;

    // Doc 1: Match in Title
    let doc1 = vec![
        (0, 1u32, 10u32, 15.0f32),
        (1, 0u32, 20u32, 30.0f32),
        (2, 0u32, 100u32, 150.0f32),
    ];
    // Doc 2: Match in Context Prefix
    let doc2 = vec![
        (0, 0u32, 10u32, 15.0f32),
        (1, 1u32, 20u32, 30.0f32),
        (2, 0u32, 100u32, 150.0f32),
    ];
    // Doc 3: Match in Body
    let doc3 = vec![
        (0, 0u32, 10u32, 15.0f32),
        (1, 0u32, 20u32, 30.0f32),
        (2, 1u32, 100u32, 150.0f32),
    ];

    let score1 = score_term_bm25f(&doc1, &weights, k1, df, n);
    let score2 = score_term_bm25f(&doc2, &weights, k1, df, n);
    let score3 = score_term_bm25f(&doc3, &weights, k1, df, n);

    assert!(
        score1 > score2,
        "Title match (score1: {}) must exceed Context Prefix match (score2: {})",
        score1,
        score2
    );
    assert!(
        score2 > score3,
        "Context Prefix match (score2: {}) must exceed Body match (score3: {})",
        score2,
        score3
    );
}

#[test]
fn test_bm25f_zero_weights_or_frequencies_return_zero() {
    let weights = vec![(0, 0.0f32, 0.75f32)];
    let doc = vec![(0, 10u32, 100u32, 100.0f32)];
    let score = score_term_bm25f(&doc, &weights, 1.5, 10, 1000);
    assert_eq!(score, 0.0, "Zero weight field must yield score 0.0");

    let valid_weights = vec![(0, 1.0f32, 0.75f32)];
    let doc_zero_tf = vec![(0, 0u32, 100u32, 100.0f32)];
    let score_zero_tf = score_term_bm25f(&doc_zero_tf, &valid_weights, 1.5, 10, 1000);
    assert_eq!(
        score_zero_tf, 0.0,
        "Zero term frequency must yield score 0.0"
    );

    let score_zero_n = score_term_bm25f(&doc, &valid_weights, 1.5, 0, 0);
    assert_eq!(score_zero_n, 0.0, "Zero N must yield score 0.0");

    let score_zero_df = score_term_bm25f(&doc, &valid_weights, 1.5, 0, 1000);
    assert_eq!(score_zero_df, 0.0, "Zero DF must yield score 0.0");
}

#[test]
fn test_bm25f_zero_panic_extreme_inputs() {
    let extreme_cases = [
        // (tf, len, avg_len)
        (u32::MAX, u32::MAX, 0.0f32),
        (0, u32::MAX, 1.0f32),
        (1, 0, 0.0f32),
        (u32::MAX, 0, u32::MAX as f32),
    ];

    let weights = vec![(0, 1.0f32, 0.75f32)];

    for (tf, len, avg_len) in extreme_cases {
        let doc = vec![(0, tf, len, avg_len)];
        let score = score_term_bm25f(&doc, &weights, 1.5, 100, 1000);
        assert!(
            !score.is_nan(),
            "Score must never be NaN for {:?}",
            (tf, len, avg_len)
        );
        assert!(
            score.is_finite(),
            "Score must be finite for {:?}",
            (tf, len, avg_len)
        );
        assert!(
            score >= 0.0,
            "Score must be non-negative for {:?}",
            (tf, len, avg_len)
        );
    }
}

#[test]
fn test_bm25f_struct_integration() {
    let fw0 = FieldWeight::new(0, 3.0, 0.75).expect("valid FieldWeight 0");
    let fw1 = FieldWeight::new(1, 1.0, 0.75).expect("valid FieldWeight 1");

    let bm25f = BM25F::new(1.5, vec![fw0, fw1]).expect("valid BM25F model");

    let doc = vec![(0, 2u32, 15u32, 20.0f32), (1, 5u32, 100u32, 200.0f32)];
    let score = bm25f.score_term(&doc, 5, 100);

    assert!(score > 0.0);
    assert!(score.is_finite());
}
