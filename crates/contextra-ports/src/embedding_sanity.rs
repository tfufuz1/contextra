use crate::EmbeddingProvider;
use thiserror::Error;

/// Raw diagnostic report generated during embedding provider sanity checks.
#[derive(Debug, Clone, PartialEq)]
pub struct SanityReport {
    /// Dimension verified during sanity check.
    pub dimension: usize,
    /// L2 norm of the test vector.
    pub test_norm: f32,
    /// Cosine difference observed during determinism check (1.0 - cos_sim).
    pub determinism_cosine_diff: f32,
    /// Cosine similarity between distinct neutral texts.
    pub neutral_pair_cosine_similarity: f32,
    /// Cosine similarity between German related terms ("Hund", "Welpe").
    pub de_related_cosine_similarity: f32,
    /// Cosine similarity between German unrelated terms ("Hund", "Steuererklärung").
    pub de_unrelated_cosine_similarity: f32,
    /// Cosine similarity between English related terms ("dog", "puppy").
    pub en_related_cosine_similarity: f32,
    /// Cosine similarity between English unrelated terms ("dog", "tax return").
    pub en_unrelated_cosine_similarity: f32,
}

/// Errors encountered during embedding sanity verification.
#[derive(Debug, Error, PartialEq)]
pub enum EmbeddingSanityError {
    /// Dimension mismatch between declared `embedding_dim()` and actual vector length, or zero dimension.
    #[error("Dimension mismatch: expected {expected}, actual {actual}")]
    DimensionMismatch {
        /// Expected dimension declared by provider.
        expected: usize,
        /// Actual vector dimension returned.
        actual: usize,
    },
    /// Degenerate vector encountered (norm == 0 or contains NaN/Inf).
    #[error("Degenerate vector: {reason}")]
    DegenerateVector {
        /// Reason for degeneration.
        reason: String,
    },
    /// Nondeterministic embeddings returned for identical input text.
    #[error("Nondeterministic embedding output: cosine difference {cosine_diff}")]
    Nondeterministic {
        /// Observed cosine difference (1.0 - cos_sim).
        cosine_diff: f32,
    },
    /// Indistinguishable embeddings returned for distinct texts.
    #[error("Indistinguishable embeddings: cosine similarity {cosine_similarity}")]
    Indistinguishable {
        /// Observed cosine similarity.
        cosine_similarity: f32,
    },
    /// Provider failed basic semantic similarity tests.
    #[error(
        "Semantic check failed for pair {pair:?}: related={cos_related}, unrelated={cos_unrelated}"
    )]
    SemanticFailure {
        /// Identifiers of failing language pair/terms.
        pair: (&'static str, &'static str),
        /// Cosine similarity for related pair.
        cos_related: f32,
        /// Cosine similarity for unrelated pair.
        cos_unrelated: f32,
    },
}

async fn embed_text(
    provider: &dyn EmbeddingProvider,
    text: &str,
) -> Result<Vec<f32>, EmbeddingSanityError> {
    provider
        .embed(text)
        .await
        .map_err(|e| EmbeddingSanityError::DegenerateVector {
            reason: format!("Embedding generation error: {e}"),
        })
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn l2_norm(a: &[f32]) -> f32 {
    dot_product(a, a).sqrt()
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let norm_a = l2_norm(a);
    let norm_b = l2_norm(b);
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    (dot_product(a, b) / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

/// Evaluates five automated sanity checks on the provided `EmbeddingProvider`.
///
/// Executes tests sequentially with fail-fast behavior:
/// 1. Dimension check (`dimension > 0` and matches `provider.embedding_dim()`).
/// 2. Vector validity check (`norm > 0` and no `NaN`/`Inf` elements).
/// 3. Determinism check (embedding identical text twice yields cosine diff < 1e-6).
/// 4. Indistinguishability check (two distinct neutral texts yield cosine similarity < 0.98).
/// 5. Semantic check (German & English semantic affinity test pairs).
pub async fn check_embedding_sanity(
    provider: &dyn EmbeddingProvider,
) -> Result<SanityReport, EmbeddingSanityError> {
    let declared_dim = provider.embedding_dim();
    if declared_dim == 0 {
        return Err(EmbeddingSanityError::DimensionMismatch {
            expected: 1,
            actual: 0,
        });
    }

    // Check 1 & 2: First test vector validity and dimension
    let test_text = "sanity_check_probe";
    let v1 = embed_text(provider, test_text).await?;

    if v1.len() != declared_dim {
        return Err(EmbeddingSanityError::DimensionMismatch {
            expected: declared_dim,
            actual: v1.len(),
        });
    }

    for val in &v1 {
        if val.is_nan() || val.is_infinite() {
            return Err(EmbeddingSanityError::DegenerateVector {
                reason: "Vector contains NaN or Inf values".to_string(),
            });
        }
    }

    let norm1 = l2_norm(&v1);
    if norm1 == 0.0 || norm1.is_nan() || !norm1.is_finite() {
        return Err(EmbeddingSanityError::DegenerateVector {
            reason: "Vector L2 norm is zero or non-finite".to_string(),
        });
    }

    // Check 3: Determinism check (embed identical text twice)
    let v2 = embed_text(provider, test_text).await?;
    if v2.len() != declared_dim {
        return Err(EmbeddingSanityError::DimensionMismatch {
            expected: declared_dim,
            actual: v2.len(),
        });
    }

    let cos_det = cosine_similarity(&v1, &v2);
    let cosine_diff = 1.0 - cos_det;
    if cosine_diff >= 1e-6 {
        return Err(EmbeddingSanityError::Nondeterministic { cosine_diff });
    }

    // Check 4: Indistinguishability (two distinct neutral texts)
    let neutral1 = "The quick brown fox jumps over the lazy dog.";
    let neutral2 = "Quantum mechanics describes subatomic particle interactions.";
    let v_neu1 = embed_text(provider, neutral1).await?;
    let v_neu2 = embed_text(provider, neutral2).await?;

    let neu_sim = cosine_similarity(&v_neu1, &v_neu2);
    if neu_sim >= 0.98 {
        return Err(EmbeddingSanityError::Indistinguishable {
            cosine_similarity: neu_sim,
        });
    }

    // Check 5: Semantic checks for German and English pairs
    // German pair
    let v_de_hund = embed_text(provider, "Hund").await?;
    let v_de_welpe = embed_text(provider, "Welpe").await?;
    let v_de_steuer = embed_text(provider, "Steuererklärung").await?;

    let de_rel = cosine_similarity(&v_de_hund, &v_de_welpe);
    let de_unrel = cosine_similarity(&v_de_hund, &v_de_steuer);

    if de_rel <= de_unrel + 0.05 {
        return Err(EmbeddingSanityError::SemanticFailure {
            pair: ("Hund", "Welpe / Steuererklärung"),
            cos_related: de_rel,
            cos_unrelated: de_unrel,
        });
    }

    // English pair
    let v_en_dog = embed_text(provider, "dog").await?;
    let v_en_puppy = embed_text(provider, "puppy").await?;
    let v_en_tax = embed_text(provider, "tax return").await?;

    let en_rel = cosine_similarity(&v_en_dog, &v_en_puppy);
    let en_unrel = cosine_similarity(&v_en_dog, &v_en_tax);

    if en_rel <= en_unrel + 0.05 {
        return Err(EmbeddingSanityError::SemanticFailure {
            pair: ("dog", "puppy / tax return"),
            cos_related: en_rel,
            cos_unrelated: en_unrel,
        });
    }

    Ok(SanityReport {
        dimension: declared_dim,
        test_norm: norm1,
        determinism_cosine_diff: cosine_diff,
        neutral_pair_cosine_similarity: neu_sim,
        de_related_cosine_similarity: de_rel,
        de_unrelated_cosine_similarity: de_unrel,
        en_related_cosine_similarity: en_rel,
        en_unrelated_cosine_similarity: en_unrel,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BoxFuture, EmbeddingError};
    use std::hash::{Hash, Hasher};

    /// Dummy provider that always returns the exact same vector.
    /// Expected to fail check (d): Indistinguishable.
    struct ConstantEmbedder {
        dim: usize,
    }

    impl EmbeddingProvider for ConstantEmbedder {
        fn provider_name(&self) -> &str {
            "constant_embedder"
        }

        fn embed<'a>(&'a self, _text: &'a str) -> BoxFuture<'a, Result<Vec<f32>, EmbeddingError>> {
            Box::pin(async move { Ok(vec![1.0; self.dim]) })
        }

        fn embedding_dim(&self) -> usize {
            self.dim
        }
    }

    /// Dummy provider that returns pseudo-random hash vectors derived from text input.
    /// Vectors are text-dependent and deterministic, but semantically meaningless.
    /// Expected to fail check (e): SemanticFailure.
    struct HashEmbedder {
        dim: usize,
    }

    impl EmbeddingProvider for HashEmbedder {
        fn provider_name(&self) -> &str {
            "hash_embedder"
        }

        fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>, EmbeddingError>> {
            let dim = self.dim;
            let text_buf = text.to_string();
            Box::pin(async move {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                text_buf.hash(&mut hasher);
                let seed = hasher.finish();

                let mut vec = Vec::with_capacity(dim);
                let mut state = seed;
                for _ in 0..dim {
                    // Simple XorShift64 PRNG
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    let val = ((state as f32) / (u64::MAX as f32)) - 0.5;
                    vec.push(val);
                }
                Ok(vec)
            })
        }

        fn embedding_dim(&self) -> usize {
            self.dim
        }
    }

    /// Dummy provider that returns semantically realistic mock vectors.
    /// Expected to pass all 5 sanity checks.
    struct ValidDummyEmbedder {
        dim: usize,
    }

    impl EmbeddingProvider for ValidDummyEmbedder {
        fn provider_name(&self) -> &str {
            "valid_dummy_embedder"
        }

        fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>, EmbeddingError>> {
            let dim = self.dim;
            let text = text.to_string();
            Box::pin(async move {
                let mut v = vec![0.0; dim];
                match text.as_str() {
                    "Hund" => {
                        v[0] = 1.0;
                    }
                    "Welpe" => {
                        v[0] = 0.95;
                        v[1] = 0.31;
                    }
                    "Steuererklärung" => {
                        v[2] = 1.0;
                    }
                    "dog" => {
                        v[3] = 1.0;
                    }
                    "puppy" => {
                        v[3] = 0.95;
                        v[4] = 0.31;
                    }
                    "tax return" => {
                        v[5] = 1.0;
                    }
                    "The quick brown fox jumps over the lazy dog." => {
                        v[6] = 1.0;
                    }
                    "Quantum mechanics describes subatomic particle interactions." => {
                        v[7] = 1.0;
                    }
                    _ => {
                        v[0] = 0.5;
                        v[1] = 0.5;
                        v[2] = 0.5;
                        v[3] = 0.5;
                    }
                }
                Ok(v)
            })
        }

        fn embedding_dim(&self) -> usize {
            self.dim
        }
    }

    #[tokio::test]
    async fn test_constant_embedder_fails_indistinguishable() {
        let provider = ConstantEmbedder { dim: 8 };
        let res = check_embedding_sanity(&provider).await;
        assert!(matches!(
            res,
            Err(EmbeddingSanityError::Indistinguishable { cosine_similarity }) if (cosine_similarity - 1.0).abs() < 1e-4
        ));
    }

    #[tokio::test]
    async fn test_hash_embedder_fails_semantic_failure() {
        let provider = HashEmbedder { dim: 64 };
        let res = check_embedding_sanity(&provider).await;
        assert!(matches!(
            res,
            Err(EmbeddingSanityError::SemanticFailure { .. })
        ));
    }

    #[tokio::test]
    async fn test_valid_dummy_embedder_passes_sanity() {
        let provider = ValidDummyEmbedder { dim: 8 };
        let res = check_embedding_sanity(&provider).await;
        assert!(res.is_ok());
        let report = res.unwrap();
        assert_eq!(report.dimension, 8);
        assert!(report.test_norm > 0.0);
        assert!(report.determinism_cosine_diff < 1e-6);
        assert!(report.neutral_pair_cosine_similarity < 0.98);
        assert!(report.de_related_cosine_similarity > report.de_unrelated_cosine_similarity + 0.05);
        assert!(report.en_related_cosine_similarity > report.en_unrelated_cosine_similarity + 0.05);
    }
}
