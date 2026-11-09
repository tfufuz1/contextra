//! Deterministische Fake-Implementierungen für Embedder und Reranker.
//!
//! Ermöglicht vollständige Tests und Offline-Ausführung ohne externe ONNX Runtime Binaries
//! oder C-Abhängigkeiten.

use crate::error::{Error, Result};
use crate::traits::{Embedder, Reranker};

/// Deterministischer Fake-Embedder für Tests und isolierte Umgebungen.
#[derive(Debug, Clone)]
pub struct FakeEmbedder {
    dimension: usize,
}

impl FakeEmbedder {
    /// Erstellt einen neuen `FakeEmbedder` mit der gewünschten Vektordimension.
    ///
    /// # Errors
    /// Gibt `Error::InvalidDimension` zurück, wenn dimension == 0 ist.
    pub fn new(dimension: usize) -> Result<Self> {
        if dimension == 0 {
            return Err(Error::InvalidDimension {
                expected: 1,
                actual: 0,
            });
        }
        Ok(Self { dimension })
    }
}

impl Embedder for FakeEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Deterministische Pseudozufalls-Generierung via BLAKE3 über den Text
        let mut hasher = blake3::Hasher::new();
        hasher.update(text.as_bytes());
        let hash = hasher.finalize();
        let bytes = hash.as_bytes();

        let mut vec = Vec::with_capacity(self.dimension);
        let mut norm_sq = 0.0f32;

        for i in 0..self.dimension {
            let byte_idx = i % bytes.len();
            let b = match bytes.get(byte_idx) {
                Some(&val) => val,
                None => 0,
            };
            // Deterministischer float im Bereich [-1.0, 1.0]
            let val = ((f32::from(b) / 127.5f32) - 1.0f32) + (i as f32 * 0.001f32);
            norm_sq += val * val;
            vec.push(val);
        }

        // L2-Normalisierung
        let norm = norm_sq.sqrt();
        if norm > 1e-6 {
            for v in &mut vec {
                *v /= norm;
            }
        }

        Ok(vec)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Deterministischer Fake-Reranker für Tests.
#[derive(Debug, Clone, Default)]
pub struct FakeReranker;

impl FakeReranker {
    /// Erstellt eine neue Instanz des Fake-Rerankers.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Reranker for FakeReranker {
    fn score(&self, query: &str, docs: &[&str]) -> Result<Vec<f32>> {
        let query_lower = query.to_lowercase();
        let query_tokens: Vec<&str> = query_lower.split_whitespace().collect();

        let mut scores = Vec::with_capacity(docs.len());
        for doc in docs {
            let doc_lower = doc.to_lowercase();
            let doc_tokens: Vec<&str> = doc_lower.split_whitespace().collect();

            if query_tokens.is_empty() || doc_tokens.is_empty() {
                scores.push(0.0f32);
                continue;
            }

            let mut matches = 0usize;
            for q in &query_tokens {
                if doc_tokens.contains(q) {
                    matches = matches.saturating_add(1);
                }
            }

            // Normalisierter Jaccard-artiger Score im Intervall [0.0, 1.0]
            let union_len = (query_tokens.len() + doc_tokens.len()).saturating_sub(matches);
            let score = if union_len > 0 {
                matches as f32 / union_len as f32
            } else {
                0.0f32
            };
            scores.push(score);
        }

        Ok(scores)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fake_embedder_dimension_and_determinism() {
        let embedder = FakeEmbedder::new(128).unwrap();
        assert_eq!(embedder.dimension(), 128);

        let v1 = embedder.embed("test text").unwrap();
        let v2 = embedder.embed("test text").unwrap();
        assert_eq!(v1, v2);
        assert_eq!(v1.len(), 128);

        // L2 Norm prüfen
        let norm_sq: f32 = v1.iter().map(|x| x * x).sum();
        assert!((norm_sq.sqrt() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_fake_reranker_scoring() {
        let reranker = FakeReranker::new();
        let query = "contextra baukasten";
        let docs = ["contextra baukasten rust", "völlig andere wörter", "nur contextra"];
        let scores = reranker.score(query, &docs).unwrap();

        assert_eq!(scores.len(), 3);
        assert!(scores[0] > scores[2]);
        assert!(scores[2] > scores[1]);
    }
}
