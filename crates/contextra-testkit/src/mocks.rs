//! # Mock-Implementierungen für Tests (`mocks`)
//!
//! Dieses Modul stellt Test-Doppelgänger (Mocks) für Ports bereit, die in Testumgebungen
//! deterministisches Verhalten ohne externe Dienste sicherstellen:
//! - [`MockEmbedder`]: Liefert deterministische Vektor-Embeddings für Textanalyse.
//! - [`TestMetricsSink`]: Zeichnet Metrik-Ereignisse (Counter, Gauge, Histogram) im Speicher auf.
//! - [`MetricEvent`]: Repräsentiert ein einzelnes aufgezeichnetes Metrik-Ereignis.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Implementierte Ports:**
//!   - [`contextra_ports::EmbeddingProvider`]
//!   - [`contextra_ports::MetricsSink`]
//!
//! ## Invarianten
//! - Kein `unsafe` (`#![forbid(unsafe_code)]`).
//! - Thread-sichere Aufzeichnung von Metriken über Mutex-geschützte Ereignislisten.

// FILE-CONTEXT
// STAND: 2026-10-25
// ZWECK: Mock implementations for testing (MockEmbedder, TestMetricsSink, MetricEvent).
// INVARIANTEN: No unsafe code allowed in testkit (#![forbid(unsafe_code)]).

use contextra_ports::{BoxFuture, EmbeddingError, EmbeddingProvider, MetricsSink};

// ANCHOR[contextra-testkit:MOCK-EMBEDDER-STRUCT-001]
/// Deterministischer Mock-Embedding-Provider für Testläufe.
///
/// **Zweck:** Ermöglicht semantische Vektortests ohne teure oder nicht-deterministische
/// externe KI-/ML-Modelle oder Inferenz-Engines.
///
/// **Verhalten:**
/// Gibt entweder einen statischen Vektor (`fixed_output`) oder einen Default-Vektor
/// aus wiederholten `0.1f32`-Werten der gewünschten Dimension zurück.
pub struct MockEmbedder {
    /// Ziel-Dimensionalität der generierten Embeddings.
    pub dim: usize,
    /// Fester Ausgabevektor für reproduzierbare Assertions in Tests.
    pub fixed_output: Option<Vec<f32>>,
}

impl MockEmbedder {
    // ANCHOR[contextra-testkit:MOCK-EMBEDDER-NEW-001]
    /// Erzeugt einen `MockEmbedder` mit der vorgegebenen Dimension.
    ///
    /// Standardmäßig wird ein Vektor aus `0.1f32` der Länge `dim` erzeugt.
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            fixed_output: None,
        }
    }

    // ANCHOR[contextra-testkit:MOCK-EMBEDDER-FIXED-001]
    /// Erzeugt einen `MockEmbedder` mit einem festen Vektor.
    ///
    /// Die Dimension wird automatisch aus der Länge von `fixed_output` abgeleitet.
    pub fn with_fixed_output(fixed_output: Vec<f32>) -> Self {
        let dim = fixed_output.len();
        Self {
            dim,
            fixed_output: Some(fixed_output),
        }
    }
}

// ANCHOR[contextra-testkit:MOCK-EMBEDDER-TRAIT-IMPL-001]
impl EmbeddingProvider for MockEmbedder {
    /// Gibt den festen Provider-Namen `"mock"` zurück.
    fn provider_name(&self) -> &str {
        "mock"
    }

    /// Asynchrone Embedding-Erzeugung für einen Eingabetext.
    ///
    /// **Ablauf:**
    /// Liefert sofort ein `Ok` mit dem geklonten `fixed_output` oder
    /// einem mit `0.1f32` gefüllten Vektor der Dimension `dim`.
    fn embed<'a>(&'a self, _text: &'a str) -> BoxFuture<'a, Result<Vec<f32>, EmbeddingError>> {
        Box::pin(async move {
            Ok(self
                .fixed_output
                .clone()
                .unwrap_or_else(|| vec![0.1f32; self.dim]))
        })
    }

    /// Liefert die konfigurierte Vektordimension.
    fn embedding_dim(&self) -> usize {
        self.dim
    }
}

// ANCHOR[contextra-testkit:METRIC-EVENT-ENUM-001]
/// Ein einzelnes aufgezeichnetes Metrik-Ereignis aus [`TestMetricsSink`].
///
/// **Zweck:** Dient in Test-Assertions der genauen Überprüfung, ob Instrumentierungen
/// die richtigen Namen, Werte und Labels gemeldet haben.
#[derive(Debug, Clone, PartialEq)]
pub enum MetricEvent {
    /// Inkrement-Ereignis eines monotonen Zählers (Counter).
    Counter {
        /// Name der Metrik.
        name: String,
        /// Um wie viel der Zähler erhöht wurde.
        value: u64,
        /// Schlüssel-Wert-Labels (Tags) zur Dimensionierung.
        labels: Vec<(String, String)>,
    },
    /// Absoluter Füllstandsmesser (Gauge).
    Gauge {
        /// Name der Metrik.
        name: String,
        /// Aktueller Wert.
        value: f64,
        /// Schlüssel-Wert-Labels.
        labels: Vec<(String, String)>,
    },
    /// Beobachtung einer Wertverteilung (Histogramm, z. B. Latenz).
    Histogram {
        /// Name der Metrik.
        name: String,
        /// Gemessener Wert.
        value: f64,
        /// Schlüssel-Wert-Labels.
        labels: Vec<(String, String)>,
    },
}

// ANCHOR[contextra-testkit:TEST-METRICS-SINK-STRUCT-001]
/// In-Memory-Implementierung von [`contextra_ports::MetricsSink`] für Tests.
///
/// **Zweck:** Fängt alle Metrik-Aufrufe ab und speichert sie chronologisch
/// in einer threadsicheren Liste zur späteren Auswertung in Unit-Tests.
#[derive(Debug, Default, Clone)]
pub struct TestMetricsSink {
    /// Mutex-geschützte Liste aller eingegangenen Ereignisse.
    events: std::sync::Arc<std::sync::Mutex<Vec<MetricEvent>>>,
}

impl TestMetricsSink {
    // ANCHOR[contextra-testkit:TEST-METRICS-SINK-NEW-001]
    /// Erstellt eine leere [`TestMetricsSink`].
    pub fn new() -> Self {
        Self::default()
    }

    // ANCHOR[contextra-testkit:TEST-METRICS-SINK-EVENTS-001]
    /// Erstellt einen Schnappschuss (Snapshot) aller bisher aufgezeichneten Ereignisse.
    ///
    /// Behandelt vergiftete Mutexe (`PoisonError`) defensiv via `into_inner()`.
    pub fn events(&self) -> Vec<MetricEvent> {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    // ANCHOR[contextra-testkit:TEST-METRICS-SINK-CLEAR-001]
    /// Leert die Liste aller aufgezeichneten Metrik-Ereignisse.
    pub fn clear(&self) {
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clear();
    }
}

// ANCHOR[contextra-testkit:TEST-METRICS-SINK-TRAIT-IMPL-001]
impl MetricsSink for TestMetricsSink {
    /// Zeichnet ein Zähler-Ereignis (Counter) auf.
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        let labels_vec = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(MetricEvent::Counter {
                name: name.to_string(),
                value,
                labels: labels_vec,
            });
    }

    /// Zeichnet einen absoluten Messwert (Gauge) auf.
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let labels_vec = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(MetricEvent::Gauge {
                name: name.to_string(),
                value,
                labels: labels_vec,
            });
    }

    /// Zeichnet eine Histogramm-Beobachtung auf.
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        let labels_vec = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self.events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(MetricEvent::Histogram {
                name: name.to_string(),
                value,
                labels: labels_vec,
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_ports::TextEmbeddingEngine;

    // ANCHOR[contextra-testkit:TEST-MOCKS-EMBEDDER-001]
    // Prüft Einbettungs-Dimension und deterministische Vektorerzeugung von MockEmbedder.
    #[tokio::test]
    async fn test_mock_embedder_methods() -> Result<(), Box<dyn std::error::Error>> {
        let embedder = MockEmbedder::new(4);
        assert_eq!(embedder.provider_name(), "mock");
        assert_eq!(embedder.embedding_dim(), 4);

        let vec = EmbeddingProvider::embed(&embedder, "test").await?;
        assert_eq!(vec, vec![0.1, 0.1, 0.1, 0.1]);

        let fixed = vec![1.0, 2.0, 3.0];
        let fixed_embedder = MockEmbedder::with_fixed_output(fixed.clone());
        assert_eq!(fixed_embedder.embedding_dim(), 3);
        let res = EmbeddingProvider::embed(&fixed_embedder, "hello").await?;
        assert_eq!(res, fixed);
        Ok(())
    }

    // ANCHOR[contextra-testkit:TEST-MOCKS-BATCH-001]
    // Prüft Batch-Verarbeitung und Blanket-Trait-Implementierung für TextEmbeddingEngine.
    #[tokio::test]
    async fn test_embedding_provider_batch_and_blanket_trait(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let embedder = MockEmbedder::new(2);
        let batch_res = EmbeddingProvider::embed_batch(&embedder, &["a", "b"]).await?;
        assert_eq!(batch_res.len(), 2);
        assert_eq!(batch_res.first(), Some(&vec![0.1, 0.1]));

        let engine: &dyn TextEmbeddingEngine = &embedder;
        let single = engine.embed("text").await?;
        assert_eq!(single, vec![0.1, 0.1]);

        let batch = engine.embed_batch(&["x", "y"]).await?;
        assert_eq!(batch.len(), 2);
        Ok(())
    }

    // ANCHOR[contextra-testkit:TEST-MOCKS-SINK-001]
    // Prüft die chronologische Aufzeichnung und das Leeren von Counter-, Gauge- und Histogramm-Ereignissen.
    #[test]
    fn test_test_metrics_sink() {
        let sink = TestMetricsSink::new();
        sink.record_counter("c1", 5, &[("env", "test")]);
        sink.record_gauge("g1", 1.5, &[]);
        sink.record_histogram("h1", 0.02, &[("op", "commit")]);

        let events = sink.events();
        assert_eq!(events.len(), 3);
        assert_eq!(
            events.first(),
            Some(&MetricEvent::Counter {
                name: "c1".to_string(),
                value: 5,
                labels: vec![("env".to_string(), "test".to_string())],
            })
        );
        assert_eq!(
            events.get(1),
            Some(&MetricEvent::Gauge {
                name: "g1".to_string(),
                value: 1.5,
                labels: vec![],
            })
        );
        assert_eq!(
            events.get(2),
            Some(&MetricEvent::Histogram {
                name: "h1".to_string(),
                value: 0.02,
                labels: vec![("op".to_string(), "commit".to_string())],
            })
        );

        sink.clear();
        assert!(sink.events().is_empty());
    }
}

// ============================================================================
// KV, ATTENTION & ROUTING FAKES (WP-TK-1.2 & WP-TK-1.3)
// ============================================================================

use bytes::Bytes;
use contextra_ports::{
    policy::{EgressDecision, EgressPolicy},
    AttentionExporter, CommunityResolver, ContextChunk, ContextPreparer, ContextWindow,
    ContextraError, DriftStatusProvider, HybridSearchProvider, KvBlock, KvBridgeStorage,
    KvPrefixHit, KvPrefixStore, PrefixKey, RequestId, TenantId, TokenBudget, TxId,
};
use contextra_types::{EntityId, RequestContext};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

type TxStagingMap = BTreeMap<TxId, BTreeMap<Vec<u8>, Bytes>>;
type KvPrefixEntryMap = BTreeMap<(u64, String), (Vec<u32>, Vec<KvBlock>)>;

// ANCHOR[contextra-testkit:KV-BRIDGE-FAKES-001]
/// Deterministische In-Memory-Implementierung von [`KvBridgeStorage`].
#[derive(Debug, Default, Clone)]
pub struct InMemoryKvBridgeStorage {
    committed: Arc<RwLock<BTreeMap<Vec<u8>, Bytes>>>,
    staging: Arc<RwLock<TxStagingMap>>,
}

impl InMemoryKvBridgeStorage {
    /// Erzeugt einen neuen leeren `InMemoryKvBridgeStorage`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gibt die Anzahl der aktuell committeten Schlüssel zurück.
    pub fn len(&self) -> usize {
        self.committed.read().len()
    }

    /// Prüft, ob der committete Speicher leer ist.
    pub fn is_empty(&self) -> bool {
        self.committed.read().is_empty()
    }
}

impl KvBridgeStorage for InMemoryKvBridgeStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, contextra_ports::Result<Option<Bytes>>> {
        Box::pin(async move {
            let guard = self.committed.read();
            Ok(guard.get(key).cloned())
        })
    }

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, contextra_ports::Result<()>> {
        Box::pin(async move {
            let mut guard = self.staging.write();
            guard.entry(tx_id).or_default().insert(key.to_vec(), Bytes::copy_from_slice(value));
            Ok(())
        })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, contextra_ports::Result<()>> {
        Box::pin(async move {
            let staged_ops = {
                let mut staging_guard = self.staging.write();
                staging_guard.remove(&tx_id)
            };

            if let Some(ops) = staged_ops {
                let mut comm_guard = self.committed.write();
                for (k, v) in ops {
                    comm_guard.insert(k, v);
                }
            }
            Ok(())
        })
    }
}

// ANCHOR[contextra-testkit:ATTENTION-FAKES-001]
/// Skriptbare, deterministische Implementierung von [`AttentionExporter`].
#[derive(Debug, Default, Clone)]
pub struct ScriptedAttentionExporter {
    weights: Arc<RwLock<BTreeMap<u64, Vec<f32>>>>,
}

impl ScriptedAttentionExporter {
    /// Erstellt einen leeren `ScriptedAttentionExporter`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registriert oder aktualisiert die Aufmerksamkeitsgewichte für einen Request.
    pub fn set_weights(&self, request_id: RequestId, weights: Vec<f32>) {
        self.weights.write().insert(request_id.0, weights);
    }

    /// Entfernt die registrierten Gewichte für einen Request.
    pub fn remove_weights(&self, request_id: RequestId) -> Option<Vec<f32>> {
        self.weights.write().remove(&request_id.0)
    }
}

impl AttentionExporter for ScriptedAttentionExporter {
    fn export_attention_weights(&self, request_id: RequestId) -> Option<Vec<f32>> {
        self.weights.read().get(&request_id.0).cloned()
    }
}

// ANCHOR[contextra-testkit:KV-PREFIX-FAKES-001]
/// Deterministische In-Memory-Implementierung von [`KvPrefixStore`].
#[derive(Debug, Default, Clone)]
pub struct InMemoryKvPrefixStore {
    entries: Arc<RwLock<KvPrefixEntryMap>>,
}

impl InMemoryKvPrefixStore {
    /// Erzeugt einen neuen leeren `InMemoryKvPrefixStore`.
    pub fn new() -> Self {
        Self::default()
    }

    fn make_key(tenant: TenantId, key: &PrefixKey) -> (u64, String) {
        let key_repr = format!(
            "{}:{}:{}:{}",
            key.model.model_id,
            key.layout.dtype,
            key.tokenizer_hash.iter().map(|b| format!("{:02x}", b)).collect::<String>(),
            key.rope.base
        );
        (tenant.as_u64(), key_repr)
    }
}

impl KvPrefixStore for InMemoryKvPrefixStore {
    fn lookup(&self, tenant: TenantId, key: &PrefixKey, tokens: &[u32]) -> Option<KvPrefixHit> {
        let internal_key = Self::make_key(tenant, key);
        let guard = self.entries.read();
        let (cached_tokens, blocks) = guard.get(&internal_key)?;

        let match_len = tokens
            .iter()
            .zip(cached_tokens.iter())
            .take_while(|(a, b)| a == b)
            .count();

        if match_len == 0 {
            None
        } else {
            Some(KvPrefixHit {
                matched_tokens: match_len,
                blocks: blocks.clone(),
            })
        }
    }

    fn insert(
        &self,
        tenant: TenantId,
        key: &PrefixKey,
        tokens: &[u32],
        blocks: Vec<KvBlock>,
    ) -> Result<(), ContextraError> {
        let internal_key = Self::make_key(tenant, key);
        let mut guard = self.entries.write();
        guard.insert(internal_key, (tokens.to_vec(), blocks));
        Ok(())
    }

    fn evict(&self, tenant: TenantId, key: &PrefixKey) -> Result<u64, ContextraError> {
        let internal_key = Self::make_key(tenant, key);
        let mut guard = self.entries.write();
        if let Some((_, blocks)) = guard.remove(&internal_key) {
            Ok(blocks.len() as u64)
        } else {
            Ok(0)
        }
    }
}

// ANCHOR[contextra-testkit:ROUTING-FAKES-001]
/// Skriptbare, deterministische Implementierung von [`HybridSearchProvider`].
#[derive(Debug, Default, Clone)]
pub struct ScriptedHybridSearchProvider {
    chunks: Arc<RwLock<Vec<ContextChunk>>>,
}

impl ScriptedHybridSearchProvider {
    /// Erzeugt einen neuen leeren `ScriptedHybridSearchProvider`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Setzt die vorgegebenen Context-Chunks.
    pub fn set_chunks(&self, mut chunks: Vec<ContextChunk>) {
        chunks.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });
        *self.chunks.write() = chunks;
    }
}

impl HybridSearchProvider for ScriptedHybridSearchProvider {
    fn search_hybrid<'a>(
        &'a self,
        _query_text: &'a str,
        _query_embedding: &'a [f32],
        top_k: usize,
    ) -> BoxFuture<'a, contextra_ports::Result<Vec<ContextChunk>>> {
        Box::pin(async move {
            if top_k == 0 {
                return Ok(Vec::new());
            }
            let guard = self.chunks.read();
            let count = guard.len().min(top_k);
            Ok(guard.get(..count).map(|s| s.to_vec()).unwrap_or_default())
        })
    }
}

// ANCHOR[contextra-testkit:ROUTING-FAKES-002]
/// Statische, deterministische Implementierung von [`CommunityResolver`].
#[derive(Debug, Default, Clone)]
pub struct StaticCommunityResolver {
    communities: Arc<RwLock<BTreeMap<EntityId, u64>>>,
}

impl StaticCommunityResolver {
    /// Erzeugt einen neuen leeren `StaticCommunityResolver`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registriert die Community-Zuordnung für eine Entity.
    pub fn insert(&self, entity_id: EntityId, community_id: u64) {
        self.communities.write().insert(entity_id, community_id);
    }
}

impl CommunityResolver for StaticCommunityResolver {
    fn get_community<'a>(&'a self, entity_id: EntityId) -> BoxFuture<'a, contextra_ports::Result<Option<u64>>> {
        Box::pin(async move {
            let guard = self.communities.read();
            Ok(guard.get(&entity_id).copied())
        })
    }
}

// ANCHOR[contextra-testkit:ROUTING-FAKES-003]
/// Deterministische Implementierung von [`ContextPreparer`].
#[derive(Debug, Default, Clone)]
pub struct BudgetTruncatingContextPreparer;

impl BudgetTruncatingContextPreparer {
    /// Erzeugt einen neuen `BudgetTruncatingContextPreparer`.
    pub fn new() -> Self {
        Self
    }
}

impl ContextPreparer for BudgetTruncatingContextPreparer {
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        budget: &TokenBudget,
        relevance_threshold: f32,
    ) -> contextra_ports::Result<ContextWindow> {
        let max_available = budget.available();
        let mut included = Vec::new();
        let mut total_tokens = 0usize;
        let mut truncated = false;

        let mut candidate_chunks: Vec<ContextChunk> = chunks
            .into_iter()
            .filter(|c| c.relevance >= relevance_threshold)
            .collect();

        candidate_chunks.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });

        for chunk in candidate_chunks {
            if total_tokens.saturating_add(chunk.token_count) <= max_available {
                total_tokens = total_tokens.saturating_add(chunk.token_count);
                included.push(chunk);
            } else {
                truncated = true;
            }
        }

        Ok(ContextWindow {
            chunks: included,
            total_tokens,
            truncated,
        })
    }
}

// ANCHOR[contextra-testkit:ROUTING-FAKES-004]
/// Statische Implementierung von [`DriftStatusProvider`].
#[derive(Debug, Clone)]
pub struct StaticDriftStatusProvider {
    status: Arc<RwLock<String>>,
}

impl Default for StaticDriftStatusProvider {
    fn default() -> Self {
        Self {
            status: Arc::new(RwLock::new("stabil".to_string())),
        }
    }
}

impl StaticDriftStatusProvider {
    /// Erzeugt einen `StaticDriftStatusProvider` mit Default-Status `"stabil"`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Erzeugt einen `StaticDriftStatusProvider` mit spezifischem Status.
    pub fn with_status(status: impl Into<String>) -> Self {
        Self {
            status: Arc::new(RwLock::new(status.into())),
        }
    }

    /// Ändert den aktuellen Status.
    pub fn set_status(&self, status: impl Into<String>) {
        *self.status.write() = status.into();
    }
}

impl DriftStatusProvider for StaticDriftStatusProvider {
    fn overall_drift_status(&self) -> String {
        self.status.read().clone()
    }
}

// ANCHOR[contextra-testkit:ROUTING-FAKES-005]
/// Skriptbare Implementierung von [`EgressPolicy`].
#[derive(Debug, Default, Clone)]
pub struct ScriptedEgressPolicy {
    deny_patterns: Arc<RwLock<Vec<String>>>,
    redact_patterns: Arc<RwLock<Vec<String>>>,
}

impl ScriptedEgressPolicy {
    /// Erzeugt eine neue leere `ScriptedEgressPolicy` (erlaubt standardmäßig alles).
    pub fn new() -> Self {
        Self::default()
    }

    /// Fügt ein Muster hinzu, das zu `Deny` führt.
    pub fn add_deny_pattern(&self, pattern: impl Into<String>) {
        self.deny_patterns.write().push(pattern.into());
    }

    /// Fügt ein Muster hinzu, das zu `Redact` führt.
    pub fn add_redact_pattern(&self, pattern: impl Into<String>) {
        self.redact_patterns.write().push(pattern.into());
    }
}

impl EgressPolicy for ScriptedEgressPolicy {
    fn evaluate(&self, _ctx: &RequestContext, payload: &str) -> EgressDecision {
        let deny_guard = self.deny_patterns.read();
        for pattern in deny_guard.iter() {
            if payload.contains(pattern) {
                return EgressDecision::Deny(format!("payload contains denied pattern '{pattern}'"));
            }
        }

        let redact_guard = self.redact_patterns.read();
        for pattern in redact_guard.iter() {
            if payload.contains(pattern) {
                return EgressDecision::Redact(format!("payload contains pattern '{pattern}' requiring redaction"));
            }
        }

        EgressDecision::Allow
    }
}
