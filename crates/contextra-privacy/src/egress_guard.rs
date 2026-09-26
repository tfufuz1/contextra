#![forbid(unsafe_code)]
//! Layer-4 EgressGuard — Bulk-Exfiltrations-Erkennung für Cloud Egress
//!
//! Kapselt die k-NN-Ähnlichkeitssuche gegen einen Text-Sucher-Trait / Closure
//! und garantiert striktes Fail-Closed-Verhalten bei Index-Fehlern, Timeouts
//! oder nicht vorhandenen/leeren Suchergebnissen.

use crate::egress_vault::{BlockReason, BoxFuture, EgressClassification, EgressClassifier};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Simplified candidate result from text similarity search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSearchResult {
    pub id: String,
    pub score: f32,
}

/// Trait for text similarity search without depending on `contextra-db::Collection` (Ring 3 DB).
pub trait TextSearchEngine: Send + Sync {
    fn search_text<'a>(
        &'a self,
        text: &'a str,
        limit: usize,
    ) -> BoxFuture<'a, Result<Vec<TextSearchResult>, String>>;
}

/// Standard-Timeout für EgressGuard Vector-Search (200 ms).
pub const DEFAULT_EGRESS_GUARD_TIMEOUT: Duration = Duration::from_millis(200);
/// Standard-Schwellwert für HNSW Cosine Similarity Match (0.85).
pub const DEFAULT_EGRESS_GUARD_THRESHOLD: f32 = 0.85;
/// Minimum Byte-Länge des Payloads, ab der die HNSW-Prüfung durchgeführt wird (128 Bytes).
pub const DEFAULT_EGRESS_GUARD_MIN_BYTES: usize = 128;

/// Layer-4 EgressGuard zur Erkennung und Blockierung von Bulk-Exfiltrationen.
///
/// Baut auf einem Text-Sucher (`TextSearchEngine`) auf.
/// Outbound-Payloads mit mindestens `min_bytes` werden über `search_text` abgefragt.
/// Bei Cosine Similarity $\ge$ `threshold` wird die Anfrage blockiert.
/// Strikte **Fail-Closed**-Semantik bei Index-Fehlern, Timeouts oder leeren Ergebnissen.
#[derive(Clone)]
pub struct EgressGuard {
    search_engine: Arc<dyn TextSearchEngine>,
    threshold: f32,
    min_bytes: usize,
    timeout: Duration,
}

impl EgressGuard {
    /// Erstellt eine neue `EgressGuard`-Instanz mit Standard-Timeout (200 ms).
    pub fn new(search_engine: Arc<dyn TextSearchEngine>, threshold: f32, min_bytes: usize) -> Self {
        Self {
            search_engine,
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

    /// Checks a `TenantScoped` egress payload for bulk exfiltration only if the bound `TenantId`
    /// matches `expected_tenant_id`. Returns `Block(BlockReason::PolicyDenied(...))` on tenant scope mismatch.
    pub async fn check_scoped(
        &self,
        payload: contextra_types::TenantScoped<&str>,
        expected_tenant_id: &contextra_types::TenantId,
    ) -> EgressClassification {
        match payload.into_inner_checked(expected_tenant_id) {
            Ok(unpacked) => self.check(unpacked).await,
            Err(err) => EgressClassification::Block(BlockReason::PolicyDenied(err.to_string())),
        }
    }

    /// Prüft einen Egress-Payload auf mögliche Bulk-Exfiltration gegen die Collection.
    ///
    /// - Liefert `Allow`, wenn `payload.len() < min_bytes`.
    /// - Führt k-NN Suche ($k=1$) via `search_text` aus.
    /// - Bei Score $\ge$ `threshold`: `Block(BlockReason::SensitivePattern("bulk-exfiltration-hnsw-match"))`.
    /// - Bei Score < `threshold`: `Allow`.
    /// - Bei Timeout, Index-Fehler oder leeren Ergebnissen: strikt **Fail-Closed** mit
    ///   `Block(BlockReason::InternalError("egress guard index unavailable — fail-closed"))`.
    pub async fn check(&self, payload: &str) -> EgressClassification {
        if payload.len() < self.min_bytes {
            return EgressClassification::Allow;
        }

        let search_fut = self.search_engine.search_text(payload, 1);
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
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    struct MockSearchEngine {
        results: Result<Vec<TextSearchResult>, String>,
        delay: Option<Duration>,
    }

    impl TextSearchEngine for MockSearchEngine {
        fn search_text<'a>(
            &'a self,
            _text: &'a str,
            _limit: usize,
        ) -> BoxFuture<'a, Result<Vec<TextSearchResult>, String>> {
            let res = self.results.clone();
            let delay = self.delay;
            Box::pin(async move {
                if let Some(d) = delay {
                    tokio::time::sleep(d).await;
                }
                res
            })
        }
    }

    #[tokio::test]
    async fn test_check_scoped_tenant_isolation() {
        use contextra_types::{TenantId, TenantScoped};

        let engine = Arc::new(MockSearchEngine {
            results: Ok(vec![]),
            delay: None,
        });
        let guard = EgressGuard::new(engine, 0.85, 128);

        let tenant_a = TenantId::try_new(11).expect("valid tenant_a");
        let tenant_b = TenantId::try_new(22).expect("valid tenant_b");

        let scoped_payload = TenantScoped::new(tenant_a, "short text");

        // Matching tenant succeeds
        let res_ok = guard.check_scoped(scoped_payload.clone(), &tenant_a).await;
        assert_eq!(res_ok, EgressClassification::Allow);

        // Mismatched tenant blocks
        let res_mismatch = guard.check_scoped(scoped_payload, &tenant_b).await;
        if let EgressClassification::Block(BlockReason::PolicyDenied(reason)) = res_mismatch {
            assert!(reason.contains("tenant scope mismatch"));
        } else {
            panic!("Expected PolicyDenied block on mismatch, got {:?}", res_mismatch);
        }
    }

    #[tokio::test]
    async fn test_payload_shorter_than_min_bytes_allows() {
        let engine = Arc::new(MockSearchEngine {
            results: Ok(vec![]),
            delay: None,
        });
        let guard = EgressGuard::new(engine, 0.85, 128);
        let res = guard.check("short text").await;
        assert_eq!(res, EgressClassification::Allow);
    }

    #[tokio::test]
    async fn test_empty_results_blocks_fail_closed() {
        let engine = Arc::new(MockSearchEngine {
            results: Ok(vec![]),
            delay: None,
        });
        let guard = EgressGuard::new(engine, 0.85, 10);
        let res = guard
            .check("Payload text exceeding minimum byte limit")
            .await;
        assert_eq!(
            res,
            EgressClassification::Block(BlockReason::InternalError(
                "egress guard index unavailable — fail-closed".to_string()
            ))
        );
    }

    #[tokio::test]
    async fn test_similarity_above_threshold_blocks() {
        let engine = Arc::new(MockSearchEngine {
            results: Ok(vec![TextSearchResult {
                id: "doc1".to_string(),
                score: 0.9,
            }]),
            delay: None,
        });
        let guard = EgressGuard::new(engine, 0.85, 10);
        let res = guard
            .check("Payload text exceeding minimum byte limit")
            .await;
        assert_eq!(
            res,
            EgressClassification::Block(BlockReason::SensitivePattern(
                "bulk-exfiltration-hnsw-match".to_string()
            ))
        );
    }

    #[tokio::test]
    async fn test_similarity_below_threshold_allows() {
        let engine = Arc::new(MockSearchEngine {
            results: Ok(vec![TextSearchResult {
                id: "doc1".to_string(),
                score: 0.5,
            }]),
            delay: None,
        });
        let guard = EgressGuard::new(engine, 0.85, 10);
        let res = guard
            .check("Payload text exceeding minimum byte limit")
            .await;
        assert_eq!(res, EgressClassification::Allow);
    }

    #[tokio::test]
    async fn test_timeout_blocks_fail_closed() {
        let engine = Arc::new(MockSearchEngine {
            results: Ok(vec![]),
            delay: Some(Duration::from_millis(100)),
        });
        let guard = EgressGuard::new(engine, 0.85, 10).with_timeout(Duration::from_millis(5));
        let res = guard
            .check("Payload text exceeding minimum byte limit")
            .await;
        assert_eq!(
            res,
            EgressClassification::Block(BlockReason::InternalError(
                "egress guard index unavailable — fail-closed".to_string()
            ))
        );
    }
}
