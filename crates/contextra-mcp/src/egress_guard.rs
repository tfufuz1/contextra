// FILE-CONTEXT
// ZWECK: Layer-4 EgressGuard (Re-export from contextra-privacy)

use contextra::Collection;
use contextra_crypto::egress_vault::{BlockReason, EgressClassification};
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
/// Baut auf dem lokalen HNSW-Vektorindex einer `contextra::Collection` auf.
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

#[cfg(test)]
mod tests {
    use super::*;
    use contextra::Contextra;
    use contextra_core::traits::{BoxFuture, EmbeddingError, EmbeddingProvider, TextEmbeddingEngine};
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
}
