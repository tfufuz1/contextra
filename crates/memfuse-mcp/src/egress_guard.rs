#![forbid(unsafe_code)]
//! Layer-4 EgressGuard — Bulk-Exfiltrations-Erkennung für Cloud MCP Egress
//!
//! Kapselt die k-NN-Ähnlichkeitssuche gegen eine lokale HNSW `Collection`
//! und garantiert striktes Fail-Closed-Verhalten bei Index-Fehlern, Timeouts
//! oder nicht vorhandenen/leeren Suchergebnissen.

use memfuse::Collection;
use memfuse_crypto::egress_vault::{
    BlockReason, BoxFuture, EgressClassification, EgressClassifier,
};
use std::sync::Arc;
use std::time::Duration;

/// Standard-Timeout für EgressGuard Vector-Search (200 ms).
pub const DEFAULT_EGRESS_GUARD_TIMEOUT: Duration = Duration::from_millis(200);
/// Standard-Schwellwert für HNSW Cosine Similarity Match (0.85).
pub const DEFAULT_EGRESS_GUARD_THRESHOLD: f32 = 0.85;
/// Minimum Byte-Länge des Payloads, ab der die HNSW-Prüfung durchgeführt wird (128 Bytes).
pub const DEFAULT_EGRESS_GUARD_MIN_BYTES: usize = 128;

/// Layer-4 EgressGuard zur Erkennung und Blockierung von Bulk-Exfiltrationen.
///
/// Baut auf dem lokalen HNSW-Vektorindex einer `memfuse::Collection` auf.
/// Outbound-Payloads mit mindestens `min_bytes` werden über `search_text` abgefragt.
/// Bei Cosine Similarity $\ge$ `threshold` wird die Anfrage blockiert.
/// Strikte **Fail-Closed**-Semantik bei Index-Fehlern, Timeouts oder leeren Ergebnissen.
#[derive(Clone)]
pub struct EgressGuard {
    collection: Arc<Collection>,
    threshold: f32,
    min_bytes: usize,
    timeout: Duration,
}

impl EgressGuard {
    /// Erstellt eine neue `EgressGuard`-Instanz mit Standard-Timeout (200 ms).
    pub fn new(collection: Arc<Collection>, threshold: f32, min_bytes: usize) -> Self {
        Self {
            collection,
            threshold,
            min_bytes,
            timeout: DEFAULT_EGRESS_GUARD_TIMEOUT,
        }
    }

    /// Setzt ein benutzerdefiniertes Timeout für die Index-Abfrage.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Gibt den gesetzten Schwellwert zurück.
    pub fn threshold(&self) -> f32 {
        self.threshold
    }

    /// Gibt die minimale Payload-Länge in Bytes zurück.
    pub fn min_bytes(&self) -> usize {
        self.min_bytes
    }

    /// Gibt das gesetzte Timeout zurück.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Prüft einen Egress-Payload auf mögliche Bulk-Exfiltration gegen die Collection.
    ///
    /// - Liefert `Allow`, wenn `payload.len() < min_bytes`.
    /// - Führt k-NN Suche ($k=1$) via `search_text` aus.
    /// - Bei Score $\ge$ `threshold`: `Block(BlockReason::SensitivePattern("bulk-exfiltration-hnsw-match"))`.
    /// - Bei Score < `threshold`: `Allow`.
    /// - Bei Timeout, Index-Fehler oder leeren Ergebnissen: strikt **Fail-Closed** mit
    ///   `Block(BlockReason::InternalError("egress guard index unavailable — fail-closed"))`.
    #[allow(deprecated)]
    pub async fn check(&self, payload: &str) -> EgressClassification {
        if payload.len() < self.min_bytes {
            return EgressClassification::Allow;
        }

        let search_fut = self.collection.search_text(payload, 1);
        match tokio::time::timeout(self.timeout, search_fut).await {
            Ok(Ok(results)) => {
                if let Some(top) = results.first() {
                    if top.score >= self.threshold {
                        tracing::warn!(
                            score = top.score,
                            threshold = self.threshold,
                            matched_doc_id = %top.id,
                            "EgressGuard bulk exfiltration match detected"
                        );
                        EgressClassification::Block(BlockReason::SensitivePattern(
                            "bulk-exfiltration-hnsw-match".to_string(),
                        ))
                    } else {
                        EgressClassification::Allow
                    }
                } else {
                    tracing::warn!(
                        "EgressGuard vector search returned empty results (empty collection) — fail-closed"
                    );
                    EgressClassification::Block(BlockReason::InternalError(
                        "egress guard index unavailable — fail-closed".to_string(),
                    ))
                }
            }
            Ok(Err(err)) => {
                tracing::error!(
                    error = %err,
                    "EgressGuard vector search failed — fail-closed"
                );
                EgressClassification::Block(BlockReason::InternalError(
                    "egress guard index unavailable — fail-closed".to_string(),
                ))
            }
            Err(_elapsed) => {
                tracing::warn!(
                    timeout_ms = self.timeout.as_millis(),
                    "EgressGuard vector search timed out — fail-closed"
                );
                EgressClassification::Block(BlockReason::InternalError(
                    "egress guard index unavailable — fail-closed".to_string(),
                ))
            }
        }
    }
}

impl EgressClassifier for EgressGuard {
    fn classify<'a>(&'a self, payload: &'a str) -> BoxFuture<'a, EgressClassification> {
        Box::pin(self.check(payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memfuse::MemFuse;
    use memfuse_core::traits::{BoxFuture, EmbeddingError, EmbeddingProvider, TextEmbeddingEngine};
    use tempfile::TempDir;

    #[derive(Clone, Debug)]
    struct DummyEmbedder {
        dim: usize,
    }

    impl EmbeddingProvider for DummyEmbedder {
        fn provider_name(&self) -> &str {
            "dummy"
        }

        fn embedding_dim(&self) -> usize {
            self.dim
        }

        fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>, EmbeddingError>> {
            let dim = self.dim;
            let mut v = vec![0.1f32; dim];
            if text.contains("secret") {
                v[0] = 1.0;
            }
            Box::pin(async move { Ok(v) })
        }
    }

    #[derive(Clone, Debug)]
    struct SlowEmbedder {
        dim: usize,
    }

    impl EmbeddingProvider for SlowEmbedder {
        fn provider_name(&self) -> &str {
            "slow"
        }

        fn embedding_dim(&self) -> usize {
            self.dim
        }

        fn embed<'a>(&'a self, _text: &'a str) -> BoxFuture<'a, Result<Vec<f32>, EmbeddingError>> {
            let dim = self.dim;
            Box::pin(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(vec![0.1f32; dim])
            })
        }
    }

    async fn create_test_collection(
    ) -> Result<(Arc<Collection>, TempDir), Box<dyn std::error::Error>> {
        let tmp = TempDir::new()?;
        let db = MemFuse::open(tmp.path()).await?;
        let col = db.collection("test_egress").await?;
        let embedder = Arc::new(DummyEmbedder {
            dim: col.dimension(),
        });
        col.set_embedder(embedder as Arc<dyn TextEmbeddingEngine>)
            .await?;
        Ok((col, tmp))
    }

    #[tokio::test]
    async fn test_payload_shorter_than_min_bytes_allows() -> Result<(), Box<dyn std::error::Error>>
    {
        let (col, _tmp) = create_test_collection().await?;
        let guard = EgressGuard::new(col, 0.85, 128);
        let short_payload = "short text";
        assert!(short_payload.len() < 128);

        let res = guard.check(short_payload).await;
        assert_eq!(res, EgressClassification::Allow);
        Ok(())
    }

    #[tokio::test]
    async fn test_empty_collection_blocks_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
        let (col, _tmp) = create_test_collection().await?;
        let guard = EgressGuard::new(col, 0.85, 10);
        let payload = "This is a long payload text exceeding minimum byte limit";

        let res = guard.check(payload).await;
        assert_eq!(
            res,
            EgressClassification::Block(BlockReason::InternalError(
                "egress guard index unavailable — fail-closed".to_string()
            ))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_similarity_above_threshold_blocks() -> Result<(), Box<dyn std::error::Error>> {
        let (col, _tmp) = create_test_collection().await?;
        let secret_vec = vec![1.0f32; col.dimension()];
        col.insert("doc1", &secret_vec, None).await?;

        let guard = EgressGuard::new(col, 0.5, 10);
        let query_payload =
            "This contains secret material that must not be exfiltrated out of the OS";
        let res = guard.check(query_payload).await;

        assert_eq!(
            res,
            EgressClassification::Block(BlockReason::SensitivePattern(
                "bulk-exfiltration-hnsw-match".to_string()
            ))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_similarity_below_threshold_allows() -> Result<(), Box<dyn std::error::Error>> {
        let (col, _tmp) = create_test_collection().await?;
        let mut v = vec![0.0f32; col.dimension()];
        v[0] = 1.0;
        col.insert("doc1", &v, None).await?;

        let guard = EgressGuard::new(col, 0.999, 10);
        let query_payload = "Public non-matching payload string exceeding min_bytes threshold";
        let res = guard.check(query_payload).await;

        assert_eq!(res, EgressClassification::Allow);
        Ok(())
    }

    #[tokio::test]
    async fn test_no_embedder_configured_blocks_fail_closed(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new()?;
        let db = MemFuse::open(tmp.path()).await?;
        let col = db.collection("no_embedder_col").await?;
        let guard = EgressGuard::new(col, 0.85, 10);
        let payload = "Payload text exceeding minimum bytes limit";

        let res = guard.check(payload).await;
        assert_eq!(
            res,
            EgressClassification::Block(BlockReason::InternalError(
                "egress guard index unavailable — fail-closed".to_string()
            ))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_timeout_blocks_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
        let tmp = TempDir::new()?;
        let db = MemFuse::open(tmp.path()).await?;
        let col = db.collection("test_egress_slow").await?;
        let slow_embedder = Arc::new(SlowEmbedder {
            dim: col.dimension(),
        });
        col.set_embedder(slow_embedder as Arc<dyn TextEmbeddingEngine>)
            .await?;

        let guard = EgressGuard::new(col, 0.85, 10).with_timeout(Duration::from_millis(5));
        let payload = "Payload text exceeding minimum bytes limit";

        let res = guard.check(payload).await;
        assert_eq!(
            res,
            EgressClassification::Block(BlockReason::InternalError(
                "egress guard index unavailable — fail-closed".to_string()
            ))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_egress_classifier_trait_implementation() -> Result<(), Box<dyn std::error::Error>>
    {
        let (col, _tmp) = create_test_collection().await?;
        let guard = EgressGuard::new(col, 0.85, 10);
        let classifier: &dyn EgressClassifier = &guard;

        let short_res = classifier.classify("tiny").await;
        assert_eq!(short_res, EgressClassification::Allow);
        Ok(())
    }
}
