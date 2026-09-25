use crate::{RouterEngine, SlmProfile};
use contextra_db::{Collection, ContextManager};
use contextra_ports::{BoxFuture, StorageEngine, StorageStats};
use contextra_types::{ContextChunk, ContextWindow, EntityId, Result, TokenBudget, TxId};
use std::sync::Arc;

pub(crate) struct MockStorageEngine;

impl StorageEngine for MockStorageEngine {
    fn get<'a>(&'a self, _: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { Ok(None) })
    }
    fn get_at_seq<'a>(
        &'a self,
        _: &'a [u8],
        _: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { Ok(None) })
    }
    fn put<'a>(&'a self, _: TxId, _: &'a [u8], _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn delete<'a>(&'a self, _: TxId, _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn rollback_to_tx<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        Box::pin(async move {
            Ok(StorageStats {
                num_segments: 0,
                total_size_bytes: 0,
                memtable_size_bytes: 0,
            })
        })
    }
    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move { Ok(0) })
    }
    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(TxId(0)) })
    }
    fn pin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn unpin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
    fn scan_prefix<'a>(&'a self, _: &'a [u8]) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { Ok(vec![]) })
    }
    fn scan<'a>(
        &'a self,
        _: std::ops::Bound<&'a [u8]>,
        _: std::ops::Bound<&'a [u8]>,
        _: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { Ok(vec![]) })
    }
}

pub(crate) struct CollectionAdapter<S: StorageEngine + 'static> {
    collection: Arc<Collection<S>>,
}

impl<S: StorageEngine + 'static> CollectionAdapter<S> {
    pub(crate) fn new(collection: Arc<Collection<S>>) -> Self {
        Self { collection }
    }
}

impl<S: StorageEngine + 'static> crate::ports_local::HybridSearchProvider for CollectionAdapter<S> {
    fn search_hybrid<'a>(
        &'a self,
        query_text: &'a str,
        query_embedding: &'a [f32],
        top_k: usize,
    ) -> BoxFuture<'a, Result<Vec<ContextChunk>>> {
        Box::pin(async move {
            let search_results = self
                .collection
                .query()
                .text(query_text)
                .embedding(query_embedding)
                .k(top_k)
                .execute()
                .await?;

            let mut chunks = Vec::with_capacity(search_results.len());
            for res in search_results {
                if let Ok(mut chunk) = ContextChunk::try_from(res) {
                    chunk.content = chunk.combined_text_owned();
                    chunks.push(chunk);
                }
            }
            Ok(chunks)
        })
    }
}

impl<S: StorageEngine + 'static> crate::ports_local::CommunityResolver for CollectionAdapter<S> {
    fn get_community<'a>(&'a self, entity_id: EntityId) -> BoxFuture<'a, Result<Option<u64>>> {
        Box::pin(async move { self.collection.get_community(entity_id).await })
    }
}

pub(crate) struct TestContextPreparer;

impl crate::ports_local::ContextPreparer for TestContextPreparer {
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        budget: &TokenBudget,
        relevance_threshold: f32,
    ) -> Result<ContextWindow> {
        let mut manager = ContextManager::new(budget.clone());
        manager.set_relevance_threshold(relevance_threshold);
        manager.prepare_context(chunks)
    }
}

pub(crate) fn create_test_router<S: StorageEngine + 'static>(
    collection: Arc<Collection<S>>,
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
}

pub(crate) fn try_create_test_router<S: StorageEngine + 'static>(
    collection: Arc<Collection<S>>,
    profiles: Vec<SlmProfile>,
    calibration_store_path: Option<std::path::PathBuf>,
) -> Result<RouterEngine> {
    let adapter = Arc::new(CollectionAdapter::new(collection));
    let preparer = Arc::new(TestContextPreparer);
    RouterEngine::try_new(
        adapter.clone(),
        adapter,
        preparer,
        profiles,
        calibration_store_path,
    )
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
