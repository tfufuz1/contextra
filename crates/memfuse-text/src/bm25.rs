// FILE-CONTEXT: BM25 Scoring & Parameter Struct.
// ZWECK: Berechnet mathematisch BM25 Term-Scores und validiert Hyperparameter (k1, b).
// INVARIANTEN: k1 >= 0.0 (non-NaN, finite), 0.0 <= b <= 1.0 (non-NaN, finite).
// NICHT-OFFENSICHTLICH: Log-IDF ist mit Robertson-Spärck-Jones Glättung implementiert (ln(1 + (N - df + 0.5) / (df + 0.5))).
// HOTSPOTS: score_term, score_term_with_params
// STAND: TS:2026-08-30T22:01:55Z (SESSION: cf1f75c6)

//! Pure BM25 scoring functions and parameter structure.

use memfuse_core::{MemFuseError, Result};

/// Default k1 parameter for BM25 term frequency saturation scaling.
pub const BM25_K1: f32 = 1.5;
/// Default b parameter for BM25 document length normalization penalty tuning.
pub const BM25_B: f32 = 0.75;

/// BM25 scoring model parameters.
///
/// Recommendations (Robertson-Walker defaults):
/// - `k1`: Term frequency saturation control parameter. Recommended default value is `1.5` (well-studied range `1.2..2.0`). Must be non-negative (`k1 >= 0.0`).
/// - `b`: Document length normalization penalty parameter. Recommended default value is `0.75` (range `0.0..1.0`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BM25 {
    pub k1: f32,
    pub b: f32,
}

impl BM25 {
    /// Creates a new `BM25` configuration after validating `k1` and `b`.
    ///
    /// # Errors
    /// Returns `MemFuseError::InvalidInput` if `k1 < 0.0` or `k1` is NaN,
    /// or if `b` is outside `[0.0, 1.0]` or is NaN.
    pub fn new(k1: f32, b: f32) -> Result<Self> {
        if k1 < 0.0 || k1.is_nan() {
            return Err(MemFuseError::InvalidInput("k1 must be >= 0.0".into()));
        }
        if !(0.0..=1.0).contains(&b) || b.is_nan() {
            return Err(MemFuseError::InvalidInput("b must be in [0.0, 1.0]".into()));
        }
        Ok(Self { k1, b })
    }

    /// Calculates the BM25 score for a single term using this instance's `k1` and `b`.
    pub fn score_term(&self, tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32) -> f32 {
        score_term_with_params(tf, doc_len, avg_doc_len, df, n, self.k1, self.b)
    }
}

impl Default for BM25 {
    /// Returns default `BM25` parameters (`k1 = 1.5`, `b = 0.75`), matching `BM25_K1` and `BM25_B`.
    fn default() -> Self {
        Self {
            k1: BM25_K1,
            b: BM25_B,
        }
    }
}

/// Calculates the BM25 score for a single term in a document using default `BM25_K1` and `BM25_B`.
///
/// # Arguments
/// * `tf` - Term frequency in the document
/// * `doc_len` - Length of the document (number of tokens)
/// * `avg_doc_len` - Average document length in the collection
/// * `df` - Document frequency (number of documents containing the term)
/// * `n` - Total number of documents in the collection
pub fn score_term(tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32) -> f32 {
    score_term_with_params(tf, doc_len, avg_doc_len, df, n, BM25_K1, BM25_B)
}

/// Calculates the BM25 score for a single term in a document using custom `k1` and `b` parameters.
pub fn score_term_with_params(
    tf: u32,
    doc_len: u32,
    avg_doc_len: f32,
    df: u32,
    n: u32,
    k1: f32,
    b: f32,
) -> f32 {
    if n == 0 || df == 0 || tf == 0 {
        return 0.0;
    }

    let tf = tf as f32;
    let doc_len = doc_len as f32;
    let df = df.min(n) as f32;
    let n = n as f32;

    let idf = {
        // Robertson-Spärck-Jones BM25+: ln(1 + (N − df + 0.5) / (df + 0.5))
        // Mathematische Garantie: IDF ≥ ln(1) = 0 für alle df ∈ [0, N].
        // Kein Floor-Artefakt mehr nötig.
        let arg = 1.0 + (n - df + 0.5) / (df + 0.5);
        arg.ln()
    };

    let avg_doc = avg_doc_len.max(1.0);
    let norm_doc_len = doc_len / avg_doc;

    let tf_numerator = tf * (k1 + 1.0);
    let tf_denominator = tf + k1 * (1.0 - b + b * norm_doc_len);

    idf * (tf_numerator / tf_denominator)
}

/// Field identifier for field-weighted BM25F scoring.
pub type FieldId = u32;

/// Field weighting configuration for BM25F scoring.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FieldWeight {
    pub field_id: FieldId,
    pub weight: f32,
    pub b: f32,
}

impl FieldWeight {
    /// Creates a new `FieldWeight` validation configuration.
    ///
    /// # Errors
    /// Returns `MemFuseError::InvalidInput` if `weight < 0.0` or `weight` is NaN,
    /// or if `b` is outside `[0.0, 1.0]` or is NaN.
    pub fn new(field_id: FieldId, weight: f32, b: f32) -> Result<Self> {
        if weight < 0.0 || weight.is_nan() {
            return Err(MemFuseError::InvalidInput(
                "field weight must be >= 0.0".into(),
            ));
        }
        if !(0.0..=1.0).contains(&b) || b.is_nan() {
            return Err(MemFuseError::InvalidInput("b must be in [0.0, 1.0]".into()));
        }
        Ok(Self {
            field_id,
            weight,
            b,
        })
    }
}

/// BM25F field-weighted scoring model configuration (§11.3).
#[derive(Debug, Clone, PartialEq)]
pub struct BM25F {
    pub k1: f32,
    pub field_weights: Vec<FieldWeight>,
    cached_weights: Vec<(FieldId, f32, f32)>,
}

impl BM25F {
    /// Creates a new `BM25F` scoring configuration.
    ///
    /// # Errors
    /// Returns `MemFuseError::InvalidInput` if `k1 < 0.0` or `k1` is NaN.
    pub fn new(k1: f32, field_weights: Vec<FieldWeight>) -> Result<Self> {
        if k1 < 0.0 || k1.is_nan() {
            return Err(MemFuseError::InvalidInput("k1 must be >= 0.0".into()));
        }
        let cached_weights = field_weights
            .iter()
            .map(|fw| (fw.field_id, fw.weight, fw.b))
            .collect();
        Ok(Self {
            k1,
            field_weights,
            cached_weights,
        })
    }

    /// Calculates the BM25F score for a single term across multiple fields.
    pub fn score_term(
        &self,
        field_term_frequencies: &[(FieldId, u32, u32, f32)],
        df: u32,
        n: u32,
    ) -> f32 {
        score_term_bm25f(field_term_frequencies, &self.cached_weights, self.k1, df, n)
    }
}

/// Calculates the field-weighted BM25F score for a single term across document fields.
///
/// Implements the Robertson & Zaragoza (2004) field-weighted BM25 formula:
/// - Computes pseudo-term frequency $\tilde{tf} = \sum_f w_f \cdot \frac{tf_f}{1 - b_f + b_f \cdot \frac{len_f}{avglen_f}}$
/// - Evaluates single-saturation BM25 score with global parameter `k1`.
///
/// # Arguments
/// * `field_term_frequencies` - Slice of tuples `(field_id, tf_f, len_f, avg_len_f)` per field
/// * `field_weights` - Slice of tuples `(field_id, weight_f, b_f)` per field configuration
/// * `k1` - Term frequency saturation control parameter
/// * `df` - Document frequency across corpus
/// * `n` - Total document count in corpus
pub fn score_term_bm25f(
    field_term_frequencies: &[(FieldId, u32, u32, f32)],
    field_weights: &[(FieldId, f32, f32)],
    k1: f32,
    df: u32,
    n: u32,
) -> f32 {
    if n == 0 || df == 0 || field_term_frequencies.is_empty() {
        return 0.0;
    }

    let df_f = df.min(n) as f32;
    let n_f = n as f32;

    let idf = {
        let arg = 1.0 + (n_f - df_f + 0.5) / (df_f + 0.5);
        let val = arg.ln();
        if val < 0.0 || val.is_nan() {
            0.0
        } else {
            val
        }
    };

    if idf <= 0.0 {
        return 0.0;
    }

    let mut tilde_tf = 0.0f32;

    for &(field_id, tf_f, len_f, avg_len_f) in field_term_frequencies {
        if tf_f == 0 {
            continue;
        }

        let (w_f, b_f) = field_weights
            .iter()
            .find(|(fid, _, _)| *fid == field_id)
            .map(|(_, w, b)| (*w, *b))
            .unwrap_or((1.0, BM25_B));

        if w_f <= 0.0 || w_f.is_nan() {
            continue;
        }

        let tf_val = tf_f as f32;
        let len_val = len_f as f32;
        let avg_len_val = avg_len_f.max(1.0);
        let b_val = b_f.clamp(0.0, 1.0);

        let norm_len = len_val / avg_len_val;
        let denom = 1.0 - b_val + b_val * norm_len;
        let denom_clamped = if denom <= 1e-6 || denom.is_nan() {
            1e-6
        } else {
            denom
        };

        let contrib = w_f * (tf_val / denom_clamped);
        if !contrib.is_nan() && contrib > 0.0 {
            tilde_tf += contrib;
        }
    }

    if tilde_tf <= 0.0 || tilde_tf.is_nan() {
        return 0.0;
    }

    let k1_val = if k1 < 0.0 || k1.is_nan() { BM25_K1 } else { k1 };

    let tf_numerator = tilde_tf * (k1_val + 1.0);
    let tf_denominator = tilde_tf + k1_val;

    if tf_denominator <= 0.0 || tf_denominator.is_nan() {
        return 0.0;
    }

    let score = idf * (tf_numerator / tf_denominator);
    if score.is_nan() || score.is_infinite() || score < 0.0 {
        0.0
    } else {
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bm25_no_nan_or_infinity() {
        // Alle Grenzfall-Kombinationen
        let cases = [
            (0, 0, 0.0, 0, 0),               // alles null
            (1, 0, 0.0, 0, 1),               // doc_len = 0
            (1, 1, 0.0, 0, 1),               // avg_doc_len = 0
            (u32::MAX, 1, 1.0, 1, 1),        // maximale tf
            (1, 1, 1.0, u32::MAX, u32::MAX), // df = n (alle Docs)
        ];
        for (tf, doc_len, avg, df, n) in cases {
            let score = score_term(tf, doc_len, avg, df, n);
            assert!(!score.is_nan(), "NaN für {:?}", (tf, doc_len, avg, df, n));
            assert!(
                !score.is_infinite(),
                "Infinity für {:?}",
                (tf, doc_len, avg, df, n)
            );
            assert!(score >= 0.0, "Negativ für {:?}", (tf, doc_len, avg, df, n));
        }
    }

    #[test]
    fn test_bm25_new_validation() {
        assert!(BM25::new(1.5, 0.75).is_ok());
        assert!(BM25::new(0.0, 0.0).is_ok());
        assert!(BM25::new(2.0, 1.0).is_ok());

        // Negative k1
        let err_k1 = BM25::new(-0.1, 0.75).unwrap_err();
        assert!(matches!(err_k1, MemFuseError::InvalidInput(_)));
        if let MemFuseError::InvalidInput(msg) = err_k1 {
            assert_eq!(msg, "k1 must be >= 0.0");
        }

        // NaN k1
        let err_k1_nan = BM25::new(f32::NAN, 0.75).unwrap_err();
        assert!(matches!(err_k1_nan, MemFuseError::InvalidInput(_)));

        // b out of bounds
        let err_b_neg = BM25::new(1.5, -0.01).unwrap_err();
        assert!(matches!(err_b_neg, MemFuseError::InvalidInput(_)));
        if let MemFuseError::InvalidInput(msg) = err_b_neg {
            assert_eq!(msg, "b must be in [0.0, 1.0]");
        }

        let err_b_high = BM25::new(1.5, 1.01).unwrap_err();
        assert!(matches!(err_b_high, MemFuseError::InvalidInput(_)));

        let err_b_nan = BM25::new(1.5, f32::NAN).unwrap_err();
        assert!(matches!(err_b_nan, MemFuseError::InvalidInput(_)));
    }

    #[test]
    fn test_bm25_default() {
        let default_bm25 = BM25::default();
        assert_eq!(default_bm25.k1, 1.5);
        assert_eq!(default_bm25.b, 0.75);
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_bm25_idf_non_negative_for_high_df(
            n in 0..100_000u32,
            df_fraction in 0.0..1.0f64,
        ) {
            let df = ((n as f64) * df_fraction).round() as u32;
            let df = df.min(n);
            let n_f = n as f32;
            let df_f = df as f32;

            let arg = 1.0 + (n_f - df_f + 0.5) / (df_f + 0.5);
            let idf = arg.ln();

            prop_assert!(idf.is_finite(), "IDF must be finite for df <= n");
            prop_assert!(idf >= 0.0, "IDF must be non-negative for all df in [0, n]");
        }

        #[test]
        fn prop_bm25_score_term_finite_and_non_negative(
            tf in 0..10_000u32,
            doc_len in 0..10_000u32,
            avg_doc_len in 0.0..10_000.0f32,
            df in 0..10_000u32,
            n in 0..10_000u32,
        ) {
            let score = score_term(tf, doc_len, avg_doc_len, df, n);
            prop_assert!(score.is_finite(), "Score must be finite");
            prop_assert!(score >= 0.0, "Score must be non-negative");
        }

        #[test]
        fn prop_bm25_score_term_bounded_across_df_relative_to_n(
            tf in 0..100_000u32,
            doc_len in 0..100_000u32,
            avg_doc_len in 0.0..100_000.0f32,
            n in 0..100_000u32,
            df_scenario in 0..5i32,
            df_offset in 0..1_000u32,
        ) {
            let df = match df_scenario {
                0 => 0, // df = 0
                1 => n / 2, // df = n / 2
                2 => n, // df = n
                3 => n.saturating_add(df_offset), // df >= n (including corrupted df > n)
                _ => df_offset % (n.saturating_add(1)), // arbitrary df <= n
            };

            let score = score_term(tf, doc_len, avg_doc_len, df, n);

            // Core safety invariants for ALL inputs
            prop_assert!(!score.is_nan(), "Score must never be NaN");
            prop_assert!(score.is_finite(), "Score must be finite");
            prop_assert!(score >= 0.0, "Score must be non-negative");

            // Independent upper bound assertion:
            // Since (tf * (k1 + 1)) / (tf + k1 * norm_len) <= (k1 + 1),
            // and IDF <= ln((2N + 1) / 1) = ln(2N + 1) for valid df >= 0,
            // or IDF = 1e-6 floor for df > N/2.
            let max_possible_idf = if n == 0 {
                0.0f32
            } else {
                (((2.0 * (n as f64) + 2.0) / 3.0).ln() as f32).max(0.0)
            };
            let upper_bound = (BM25_K1 + 1.0) * max_possible_idf + 1e-4; // float safety margin

            prop_assert!(
                score <= upper_bound,
                "Score {} exceeded independent theoretical upper bound {} for tf={}, df={}, n={}",
                score,
                upper_bound,
                tf,
                df,
                n
            );
        }
    }

    #[test]
    fn score_term_case_exact_anti_mirroring_value() {
        // Independent mathematical calculation:
        // tf = 1, doc_len = 10, avg_doc_len = 10.0, df = 1, n = 10, k1 = 1.5, b = 0.75
        // idf_arg = 1.0 + (10 - 1 + 0.5) / (1 + 0.5) = 1.0 + 9.5 / 1.5 = 22 / 3 = 7.3333333...
        // idf = ln(22/3) = 1.9924302
        // norm_doc_len = 10 / 10 = 1.0
        // tf_num = 1 * 2.5 = 2.5
        // tf_den = 1 + 1.5 * (0.25 + 0.75) = 2.5
        // tf_factor = 2.5 / 2.5 = 1.0
        // expected score = 1.9924302
        let score = score_term(1, 10, 10.0, 1, 10);
        let expected = 1.992_430_2;
        assert!(
            (score - expected).abs() < 1e-6,
            "Expected score close to {}, got {}",
            expected,
            score
        );
    }

    #[test]
    fn test_bm25_high_freq_term_gets_nonzero_idf() {
        // For df = n (term appears in all documents), Robertson-Spärck-Jones BM25+ yields:
        // idf = ln(1 + (n - n + 0.5)/(n + 0.5)) = ln(1 + 0.5 / (n + 0.5))
        // For n = 9, df = 9: idf = ln(1 + 0.5 / 9.5) = ln(10 / 9.5) ≈ 0.051293
        // For n = 1, df = 1: idf = ln(1 + 0.5 / 1.5) = ln(4 / 3) ≈ 0.287682
        let n = 9u32;
        let df = 9u32;
        let idf_arg = 1.0 + (n as f32 - df as f32 + 0.5) / (df as f32 + 0.5);
        let expected_idf = idf_arg.ln();

        let score = score_term(1, 10, 10.0, df, n);
        assert!(score > 0.0, "Score for df = n must be non-zero");
        assert!(
            (score - expected_idf).abs() < 1e-5,
            "IDF for high freq term must match Robertson-Spärck-Jones value {}, got {}",
            expected_idf,
            score
        );
    }

    #[test]
    fn bm25_struct_score_term_case_matches_standalone_function() {
        let bm25 = BM25::default();
        let struct_score = bm25.score_term(2, 50, 50.0, 5, 100);
        let fn_score = score_term(2, 50, 50.0, 5, 100);
        assert_eq!(struct_score, fn_score);
    }

    #[test]
    fn test_bm25_score() {
        let score = score_term(2, 100, 150.0, 10, 1000);
        assert!(score > 0.0);
    }

    #[test]
    fn test_bm25_score_zero_n() {
        let score = score_term(2, 100, 150.0, 10, 0);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_bm25_boundary_params_zero_tf() {
        let bm25 = BM25::new(0.0, 0.0).unwrap();
        assert_eq!(bm25.score_term(0, 10, 10.0, 1, 10), 0.0);
        let bm25_max = BM25::new(2.0, 1.0).unwrap();
        assert!(bm25_max.score_term(1, 10, 10.0, 1, 10) > 0.0);
    }

    #[test]
    fn test_bm25_score_zero_df() {
        // Dedicated test for Befund 3 (df = 0)
        let score_std = score_term(2, 100, 150.0, 0, 1000);
        assert_eq!(
            score_std, 0.0,
            "score_term must strictly return 0.0 when df = 0"
        );

        let score_custom = score_term_with_params(2, 100, 150.0, 0, 1000, 1.2, 0.5);
        assert_eq!(
            score_custom, 0.0,
            "score_term_with_params must strictly return 0.0 when df = 0"
        );
    }

    #[test]
    fn test_bm25_score_zero_doc_len() {
        let score = score_term(2, 0, 150.0, 10, 1000);
        assert!(!score.is_nan());
        assert!(score > 0.0);
    }

    #[test]
    fn test_bm25_score_zero_avg_doc_len() {
        let score = score_term(2, 100, 0.0, 10, 1000);
        assert!(!score.is_nan());
        assert!(score > 0.0);
    }

    #[test]
    fn test_bm25_score_corruption_df_gt_n() {
        // df = 2000, n = 1000 -> (1000 - 2000 + 0.5) / (2000 + 0.5) = -999.5 / 2000.5 -> negative
        // ln of negative is NaN
        let score = score_term(2, 100, 150.0, 2000, 1000);
        assert!(!score.is_nan());
        assert!(!score.is_infinite());
        assert!(score >= 0.0);
    }

    #[test]
    fn bm25_handles_extreme_tf() {
        // tf = u32::MAX (4 billion occurrences)
        let score = score_term(u32::MAX, 1000, 500.0, 1, 1_000_000);
        assert!(
            score.is_finite(),
            "BM25 must return finite score for extreme tf"
        );
        assert!(score >= 0.0);
    }

    #[test]
    fn test_bm25_extreme_values() {
        // Very small avg_doc_len
        let score = score_term(1, 1, 1e-10, 1, 10);
        assert!(!score.is_nan());
        assert!(!score.is_infinite());

        // df very close to n
        let score = score_term(1, 10, 10.0, 10, 10);
        assert!(!score.is_nan());
        assert!(score >= 0.0);
    }

    #[test]
    fn test_bm25_score_monotonicity_more_matches() {
        // Single term match ("Katze" in doc 1) vs dual term match ("Katze" and "Hund" in doc 2)
        // Corpus: N = 100, avg_doc_len = 10.0
        // Term "Katze": df = 5
        // Term "Hund": df = 5
        // Doc 1: "Die Katze saß auf dem Dach" (doc_len = 6, Katze tf = 1, Hund tf = 0)
        // Doc 2: "Die Katze und der Hund auf dem Dach" (doc_len = 8, Katze tf = 1, Hund tf = 1)
        let score_katze = score_term(1, 6, 10.0, 5, 100);
        let score_hund = score_term(1, 8, 10.0, 5, 100);

        let score_1 = score_katze; // 1 term matched
        let score_2 = score_katze + score_hund; // 2 terms matched

        assert!(
            score_2 > score_1,
            "Mehr Term-Matches müssen höheren Score geben (score_2: {}, score_1: {})",
            score_2,
            score_1
        );
    }

    #[test]
    fn test_bm25f_degeneration_to_bm25() {
        // Degeneration test: Single field document with w_0 = 1.0, b_0 = b
        // Must yield numerically identical result to score_term_with_params
        let tf = 2u32;
        let doc_len = 100u32;
        let avg_doc_len = 150.0f32;
        let df = 10u32;
        let n = 1000u32;
        let k1 = 1.5f32;
        let b = 0.75f32;

        let bm25_score = score_term_with_params(tf, doc_len, avg_doc_len, df, n, k1, b);

        let field_id: FieldId = 0;
        let field_tf = vec![(field_id, tf, doc_len, avg_doc_len)];
        let field_weights = vec![(field_id, 1.0f32, b)];

        let bm25f_score = score_term_bm25f(&field_tf, &field_weights, k1, df, n);

        assert!(
            (bm25_score - bm25f_score).abs() < 1e-6,
            "BM25F score ({}) must match standard BM25 score ({}) for single field",
            bm25f_score,
            bm25_score
        );
    }

    #[test]
    fn test_bm25f_field_weight_validation() {
        assert!(FieldWeight::new(0, 1.0, 0.75).is_ok());

        // Negative weight
        assert!(FieldWeight::new(0, -0.1, 0.75).is_err());
        // NaN weight
        assert!(FieldWeight::new(0, f32::NAN, 0.75).is_err());
        // Out of bound b
        assert!(FieldWeight::new(0, 1.0, -0.1).is_err());
        assert!(FieldWeight::new(0, 1.0, 1.1).is_err());
        assert!(FieldWeight::new(0, 1.0, f32::NAN).is_err());
    }

    #[test]
    fn test_bm25f_struct_methods() {
        let fw0 = FieldWeight::new(0, 2.0, 0.75).expect("valid fw0");
        let fw1 = FieldWeight::new(1, 0.5, 0.50).expect("valid fw1");
        let model = BM25F::new(1.2, vec![fw0, fw1]).expect("valid model");

        let field_tfs = vec![(0, 1, 10, 20.0), (1, 3, 50, 100.0)];

        let score = model.score_term(&field_tfs, 5, 100);
        assert!(score > 0.0);
        assert!(!score.is_nan());
        assert!(score.is_finite());
    }

    #[test]
    fn test_bm25f_title_higher_weight_than_body() {
        // Field 0 = Title (weight = 3.0)
        // Field 1 = Body (weight = 1.0)
        let weights = vec![(0, 3.0f32, 0.75f32), (1, 1.0f32, 0.75f32)];

        // Doc A: Term occurs 1 time in title (Field 0)
        let doc_a_tfs = vec![(0, 1u32, 5u32, 10.0f32), (1, 0u32, 100u32, 200.0f32)];
        // Doc B: Term occurs 1 time in body (Field 1)
        let doc_b_tfs = vec![(0, 0u32, 5u32, 10.0f32), (1, 1u32, 100u32, 200.0f32)];

        let score_a = score_term_bm25f(&doc_a_tfs, &weights, 1.5, 10, 1000);
        let score_b = score_term_bm25f(&doc_b_tfs, &weights, 1.5, 10, 1000);

        assert!(
            score_a > score_b,
            "Title match (score_a: {}) must score higher than Body match (score_b: {}) when title has higher weight",
            score_a,
            score_b
        );
    }
}
