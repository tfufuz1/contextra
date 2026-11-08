//! # Gemeinsame Test-Fixtures, Mock-Provider und Hilfsstrukturen
//!
//! **Ring:** Ring 3 (Applikations- und Testschicht).
//! **Zweck:** Gemeinsame Test-Fixtures, Mock-Provider und Hilfsstrukturen.

use crate::{RouterEngine, SlmProfile};
use contextra_ports::clock::Clock;
use contextra_ports::{BoxFuture, CommunityResolver, ContextPreparer, HybridSearchProvider};
use contextra_types::{ContextChunk, ContextWindow, EntityId, Result, TokenBudget, TxId};
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct TestClock {
    pub nanos: AtomicU64,
}

impl TestClock {
    pub fn new(initial_nanos: u64) -> Self {
        Self {
            nanos: AtomicU64::new(initial_nanos),
        }
    }

    #[allow(dead_code)]
    pub fn advance(&self, duration: std::time::Duration) {
        let nanos = u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX);
        self.nanos.fetch_add(nanos, Ordering::Relaxed);
    }
}

impl Clock for TestClock {
    fn now_unix_nanos(&self) -> u64 {
        self.nanos.load(Ordering::Relaxed)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.nanos.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone)]
pub struct MockDoc {
    pub entity_id: Option<EntityId>,
    pub key: String,
    pub vector: Vec<f32>,
    pub text: String,
}

#[derive(Default)]
#[allow(dead_code)]
pub struct TestCollection {
    pub dimension: usize,
    pub docs: RwLock<Vec<MockDoc>>,
    pub communities: RwLock<BTreeMap<EntityId, u64>>,
    pub raw_comm_keys: RwLock<BTreeMap<Vec<u8>, u64>>,
    pub next_doc_id: AtomicU64,
}

impl TestCollection {
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            docs: RwLock::new(Vec::new()),
            communities: RwLock::new(BTreeMap::new()),
            raw_comm_keys: RwLock::new(BTreeMap::new()),
            next_doc_id: AtomicU64::new(1),
        }
    }

    pub async fn insert(
        &self,
        key: &str,
        vector: &[f32],
        metadata: Option<Value>,
    ) -> Result<()> {
        let entity_id = EntityId::from_key(key).ok();
        let text = metadata
            .and_then(|m| m.get("text").and_then(|t| t.as_str().map(|s| s.to_string())))
            .unwrap_or_else(|| key.to_string());
        self.docs.write().push(MockDoc {
            entity_id,
            key: key.to_string(),
            vector: vector.to_vec(),
            text,
        });
        Ok(())
    }

    #[allow(dead_code)]
pub fn set_community(&self, entity_id: EntityId, comm_id: u64) {
        self.communities.write().insert(entity_id, comm_id);
    }

    pub async fn relate(&self, _key_a: &str, _key_b: &str, _relation: &str) -> Result<()> {
        Ok(())
    }
}

impl HybridSearchProvider for TestCollection {
    fn search_hybrid<'a>(
        &'a self,
        query_text: &'a str,
        query_embedding: &'a [f32],
        top_k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>> {
        Box::pin(async move {
            let docs = self.docs.read().clone();
            if docs.is_empty() {
                return Ok(vec![]);
            }

            let mut scored: Vec<(f32, MockDoc)> = docs
                .into_iter()
                .map(|doc| {
                    let mut score = 0.0f32;
                    if !query_embedding.is_empty() && query_embedding.len() == doc.vector.len() {
                        let dot: f32 = query_embedding
                            .iter()
                            .zip(doc.vector.iter())
                            .map(|(a, b)| a * b)
                            .sum();
                        score += (dot * 0.3).clamp(0.0, 0.3);
                    }
                    if !query_text.is_empty() && (doc.text.contains(query_text) || query_text.contains(&doc.text)) {
                        score += 0.05;
                    }
                    if score <= 0.0 {
                        score = 0.1;
                    }
                    (score.clamp(0.0, 1.0), doc)
                })
                .collect();

            scored.sort_by(|a, b| b.0.total_cmp(&a.0));

            let mut chunks = Vec::with_capacity(top_k.min(scored.len()));
            for (score, doc) in scored.into_iter().take(top_k) {
                let doc_id = doc.entity_id.map(|e| contextra_types::DocId::new(e.inner())).unwrap_or_else(|| {
                    contextra_types::DocId::new(self.next_doc_id.fetch_add(1, Ordering::Relaxed))
                });
                let metadata = serde_json::json!({ "key": doc.key });
                chunks.push(ContextChunk {
                    doc_id,
                    content: doc.text.clone(),
                    relevance: score,
                    token_count: doc.text.split_whitespace().count().max(1),
                    metadata: Some(metadata),
                    contextual_prefix: None,
                    links: vec![],
                });
            }
            Ok(chunks)
        })
    }
}

impl CommunityResolver for TestCollection {
    fn get_community<'a>(&'a self, entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>> {
        let comms = self.communities.read();
        let comm_id = comms.get(&entity_id).copied();
        Box::pin(async move { Ok(comm_id) })
    }
}

#[derive(Clone)]
pub struct CollectionAdapter {
    pub collection: Arc<TestCollection>,
}

impl CollectionAdapter {
    pub fn new(collection: Arc<TestCollection>) -> Self {
        Self { collection }
    }
}

impl HybridSearchProvider for CollectionAdapter {
    fn search_hybrid<'a>(
        &'a self,
        query_text: &'a str,
        query_embedding: &'a [f32],
        top_k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>> {
        self.collection.search_hybrid(query_text, query_embedding, top_k)
    }
}

impl CommunityResolver for CollectionAdapter {
    fn get_community<'a>(&'a self, entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>> {
        self.collection.get_community(entity_id)
    }
}

pub struct TestContextPreparer;

impl ContextPreparer for TestContextPreparer {
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        budget: &TokenBudget,
        relevance_threshold: f32,
    ) -> Result<ContextWindow> {
        let mut filtered: Vec<ContextChunk> = chunks
            .into_iter()
            .filter(|c| c.relevance >= relevance_threshold)
            .collect();
        filtered.sort_by(|a, b| b.relevance.total_cmp(&a.relevance));

        let max_tokens = budget.effective_limit();
        let mut total_tokens = 0;
        let mut selected = Vec::new();
        let mut truncated = false;

        for chunk in filtered {
            if total_tokens + chunk.token_count <= max_tokens {
                total_tokens += chunk.token_count;
                selected.push(chunk);
            } else {
                truncated = true;
                break;
            }
        }

        Ok(ContextWindow {
            chunks: selected,
            total_tokens,
            truncated,
        })
    }
}

pub struct TestInnerStorage {
    pub collection: Arc<TestCollection>,
}

impl TestInnerStorage {
    pub async fn put(&self, _tx: TxId, key: &[u8], val: &[u8]) -> Result<()> {
        if let Ok(comm_id) = serde_json::from_slice::<u64>(val) {
            self.collection.raw_comm_keys.write().insert(key.to_vec(), comm_id);
            if let Ok(key_str) = std::str::from_utf8(key) {
                if let Some(eid_str) = key_str.strip_prefix("__graph:community:") {
                    if let Ok(eid_num) = eid_str.parse::<u64>() {
                        self.collection
                            .communities
                            .write()
                            .insert(EntityId::new(eid_num), comm_id);
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn commit(&self, _tx: TxId) -> Result<()> {
        Ok(())
    }
}

pub struct ContextraConfig {
    pub dimension: usize,
    pub _reserved: u64,
}

impl Default for ContextraConfig {
    fn default() -> Self {
        Self { dimension: 4, _reserved: 0 }
    }
}

pub struct Contextra {
    pub collection: Arc<TestCollection>,
}

impl Contextra {
    pub async fn open_with_config(
        _path: impl AsRef<std::path::Path>,
        config: ContextraConfig,
    ) -> Result<Self> {
        Ok(Self {
            collection: Arc::new(TestCollection::new(config.dimension)),
        })
    }

    pub async fn collection(&self, _name: &str) -> Result<Arc<TestCollection>> {
        Ok(self.collection.clone())
    }

    pub fn allocate_tx(&self) -> Result<TxId> {
        Ok(TxId(1))
    }

    pub fn inner_storage(&self) -> TestInnerStorage {
        TestInnerStorage {
            collection: self.collection.clone(),
        }
    }
}

pub fn create_test_router(
    collection: Arc<TestCollection>,
    profiles: Vec<SlmProfile>,
    calibration_store_path: Option<std::path::PathBuf>,
) -> RouterEngine {
    let adapter = Arc::new(CollectionAdapter::new(collection));
    let preparer = Arc::new(TestContextPreparer);
    RouterEngine::new(
        adapter.clone(),
        adapter,
        preparer,
        profiles,
        calibration_store_path,
    )
    .with_clock(Arc::new(TestClock::new(1_000_000_000_000)))
}

pub fn try_create_test_router(
    collection: Arc<TestCollection>,
    profiles: Vec<SlmProfile>,
    calibration_store_path: Option<std::path::PathBuf>,
) -> Result<RouterEngine> {
    let adapter = Arc::new(CollectionAdapter::new(collection));
    let preparer = Arc::new(TestContextPreparer);
    let router = RouterEngine::try_new(
        adapter.clone(),
        adapter,
        preparer,
        profiles,
        calibration_store_path,
    )?;
    Ok(router.with_clock(Arc::new(TestClock::new(1_000_000_000_000))))
}

#[derive(Clone)]
pub(crate) struct LogCaptureLayer(pub(crate) Arc<std::sync::Mutex<Vec<String>>>);

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for LogCaptureLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = StringVisitor(String::new());
        event.record(&mut visitor);
        if let Ok(mut guard) = self.0.lock() {
            guard.push(visitor.0);
        }
    }
}

pub(crate) struct StringVisitor(pub(crate) String);
impl tracing::field::Visit for StringVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        use std::fmt::Write;
        write!(self.0, "{}={:?} ", field.name(), value).ok();
    }
}

#[allow(dead_code)]
fn _use_fixtures_helpers(clock: &TestClock) {
    use tracing::field::Visit;
    use tracing_subscriber::Layer;
    let _ = clock.now_unix_nanos();
    let _ = <LogCaptureLayer as Layer<tracing_subscriber::Registry>>::on_event;
    let _ = <StringVisitor as Visit>::record_debug;
}
