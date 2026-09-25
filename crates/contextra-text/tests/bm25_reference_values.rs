// FILE-CONTEXT: Reference value verification tests for BM25 and BM25F implementation.
// ZWECK: Verifiziert BM25 score_term_with_params und score_term_bm25f gegen unabhängig berechnete Referenzwerte der publizierten Formeln.
// INVARIANTEN: #![forbid(unsafe_code)], alle Erwartungswerte werden unabhängig in Rust (f64) nachgerechnet.

#![forbid(unsafe_code)]

use contextra_text::bm25::{score_term_bm25f, score_term_with_params};

/// Calculates the expected Robertson-Spärck-Jones BM25 score independently in f64.
fn compute_reference_bm25_score(
    tf: u32,
    doc_len: u32,
    avg_doc_len: f64,
    df: u32,
    n: u32,
    k1: f64,
    b: f64,
) -> f64 {
    if n == 0 || df == 0 || tf == 0 {
        return 0.0;
    }

    let tf = tf as f64;
    let doc_len = doc_len as f64;
    let df = (df.min(n)) as f64;
    let n = n as f64;

    let idf = (1.0 + (n - df + 0.5) / (df + 0.5)).ln();

    let avg_doc = avg_doc_len.max(1.0);
    let norm_doc_len = doc_len / avg_doc;

    let tf_numerator = tf * (k1 + 1.0);
    let tf_denominator = tf + k1 * (1.0 - b + b * norm_doc_len);

    idf * (tf_numerator / tf_denominator)
}

/// Calculates the expected Robertson & Zaragoza (2004) BM25F score independently in f64.
fn compute_reference_bm25f_score(
    field_tfs: &[(u32, u32, u32, f64)], // (field_id, tf_f, len_f, avg_len_f)
    field_weights: &[(u32, f64, f64)],  // (field_id, weight_f, b_f)
    k1: f64,
    df: u32,
    n: u32,
) -> f64 {
    if n == 0 || df == 0 || field_tfs.is_empty() {
        return 0.0;
    }

    let df_f = df.min(n) as f64;
    let n_f = n as f64;

    let idf = (1.0 + (n_f - df_f + 0.5) / (df_f + 0.5)).ln();
    if idf <= 0.0 {
        return 0.0;
    }

    let mut tilde_tf = 0.0f64;
    for &(field_id, tf_f, len_f, avg_len_f) in field_tfs {
        if tf_f == 0 {
            continue;
        }

        let (w_f, b_f) = field_weights
            .iter()
            .find(|(fid, _, _)| *fid == field_id)
            .map(|(_, w, b)| (*w, *b))
            .unwrap_or((1.0, 0.75));

        if w_f <= 0.0 {
            continue;
        }

        let tf_val = tf_f as f64;
        let len_val = len_f as f64;
        let avg_len_val = avg_len_f.max(1.0);
        let b_val = b_f.clamp(0.0, 1.0);

        let norm_len = len_val / avg_len_val;
        let denom = 1.0 - b_val + b_val * norm_len;
        let denom_clamped = if denom <= 1e-6 { 1e-6 } else { denom };

        let contrib = w_f * (tf_val / denom_clamped);
        if contrib > 0.0 {
            tilde_tf += contrib;
        }
    }

    if tilde_tf <= 0.0 {
        return 0.0;
    }

    let k1_val = if k1 < 0.0 { 1.5 } else { k1 };
    let tf_numerator = tilde_tf * (k1_val + 1.0);
    let tf_denominator = tilde_tf + k1_val;

    if tf_denominator <= 0.0 {
        return 0.0;
    }

    let score = idf * (tf_numerator / tf_denominator);
    if score < 0.0 {
        0.0
    } else {
        score
    }
}

#[test]
fn test_bm25_known_reference_value() {
    // Reference scenario from spec/task description:
    // N = 100, df = 10, tf = 3, doc_len = 200, avg_doc_len = 150.0, k1 = 1.5, b = 0.75
    // Formula steps:
    //   IDF = ln(1 + (100 - 10 + 0.5) / (10 + 0.5)) = ln(1 + 90.5 / 10.5) = ln(9.6190476...) ≈ 2.26376
    //   norm_dl = 200 / 150 = 1.333333...
    //   tf_num = 3 * (1.5 + 1) = 7.5
    //   tf_den = 3 + 1.5 * (1 - 0.75 + 0.75 * (200 / 150)) = 3 + 1.5 * (0.25 + 1.0) = 4.875
    //   expected score = 2.26376 * (7.5 / 4.875) ≈ 3.48270...
    let n = 100u32;
    let df = 10u32;
    let tf = 3u32;
    let doc_len = 200u32;
    let avg_doc_len = 150.0f32;
    let k1 = 1.5f32;
    let b = 0.75f32;

    let expected = compute_reference_bm25_score(
        tf,
        doc_len,
        avg_doc_len as f64,
        df,
        n,
        k1 as f64,
        b as f64,
    );

    let actual = score_term_with_params(tf, doc_len, avg_doc_len, df, n, k1, b);

    // Verify actual value matches independent f64 calculation within single-precision tolerance
    assert!(
        (actual as f64 - expected).abs() < 1e-4,
        "Actual score {} differs from expected reference value {}",
        actual,
        expected
    );
}

#[test]
fn test_bm25_reference_table() {
    // Generate a parameter matrix of at least 20 different parameter combinations
    // tf ∈ {1, 3, 10, 50}
    // doc_len / avg_doc_len ratio ∈ {0.5, 1.0, 2.0}
    // df / n ratio ∈ {0.01, 0.1, 0.5}
    // k1 ∈ {1.2, 1.5, 2.0}
    // b ∈ {0.0, 0.5, 0.75, 1.0}

    let tfs = [1u32, 3u32, 10u32, 50u32];
    let doc_len_ratios = [0.5f64, 1.0f64, 2.0f64];
    let df_ratios = [0.01f64, 0.10f64, 0.50f64];
    let k1_values = [1.2f32, 1.5f32, 2.0f32];
    let b_values = [0.0f32, 0.5f32, 0.75f32, 1.0f32];

    let n = 1000u32;
    let avg_doc_len = 200.0f32;

    let mut combinations_count = 0;

    for &tf in &tfs {
        for &dl_ratio in &doc_len_ratios {
            for &df_ratio in &df_ratios {
                for &k1 in &k1_values {
                    for &b in &b_values {
                        let doc_len = (avg_doc_len as f64 * dl_ratio).round() as u32;
                        let df = (n as f64 * df_ratio).round() as u32;

                        let expected = compute_reference_bm25_score(
                            tf,
                            doc_len,
                            avg_doc_len as f64,
                            df,
                            n,
                            k1 as f64,
                            b as f64,
                        );

                        let actual = score_term_with_params(
                            tf,
                            doc_len,
                            avg_doc_len,
                            df,
                            n,
                            k1,
                            b,
                        );

                        let diff = (actual as f64 - expected).abs();
                        assert!(
                            diff < 1e-4,
                            "Mismatch for combination tf={}, doc_len={}, avg_doc_len={}, df={}, n={}, k1={}, b={}: expected {}, got {}, diff={}",
                            tf, doc_len, avg_doc_len, df, n, k1, b, expected, actual, diff
                        );

                        combinations_count += 1;
                    }
                }
            }
        }
    }

    assert!(
        combinations_count >= 20,
        "Must test at least 20 combinations, tested {}",
        combinations_count
    );
}

#[test]
fn test_bm25f_reference_value() {
    // BM25F field-weighted scoring reference test:
    // Field 0: Title (weight = 2.0, b = 0.5, len = 10, avg_len = 10.0, tf = 2)
    // Field 1: Body  (weight = 1.0, b = 0.8, len = 500, avg_len = 250.0, tf = 5)
    // Corpus: N = 1000, df = 50, k1 = 1.5
    let field_0_id = 0u32;
    let field_1_id = 1u32;

    let field_tfs_f32 = vec![
        (field_0_id, 2u32, 10u32, 10.0f32),
        (field_1_id, 5u32, 500u32, 250.0f32),
    ];
    let field_weights_f32 = vec![
        (field_0_id, 2.0f32, 0.5f32),
        (field_1_id, 1.0f32, 0.8f32),
    ];

    let field_tfs_f64 = vec![
        (field_0_id, 2u32, 10u32, 10.0f64),
        (field_1_id, 5u32, 500u32, 250.0f64),
    ];
    let field_weights_f64 = vec![
        (field_0_id, 2.0f64, 0.5f64),
        (field_1_id, 1.0f64, 0.8f64),
    ];

    let df = 50u32;
    let n = 1000u32;
    let k1 = 1.5f32;

    let expected = compute_reference_bm25f_score(
        &field_tfs_f64,
        &field_weights_f64,
        k1 as f64,
        df,
        n,
    );

    let actual = score_term_bm25f(&field_tfs_f32, &field_weights_f32, k1, df, n);

    let diff = (actual as f64 - expected).abs();
    assert!(
        diff < 1e-4,
        "BM25F score {} differs from reference value {}, diff={}",
        actual,
        expected,
        diff
    );
}
