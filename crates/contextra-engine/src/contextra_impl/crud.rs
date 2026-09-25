use crate::*;
use contextra_store::LsmStorage;
use contextra_types::{FilterExpr, Result};
use serde_json::Value;
use std::sync::Arc;

impl Contextra {
    pub(crate) async fn default_col(&self) -> Result<Arc<Collection<LsmStorage>>> {
        self.collection("default").await
    }

    #[tracing::instrument(level = "trace", skip(self, value))]
    pub async fn put_kv(&self, id: &str, value: &Value) -> Result<()> {
        self.default_col().await?.put_kv(id, value).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_kv(&self, id: &str) -> Result<Option<Value>> {
        self.default_col().await?.get_kv(id).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn insert(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()> {
        self.default_col()
            .await?
            .insert(id, embedding, metadata)
            .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn insert_typed(
        &self,
        collection_name: &str,
        id: &str,
        embedding: &[f32],
        memory_type: contextra_types::MemoryType,
        metadata: Option<Value>,
    ) -> Result<()> {
        self.collection(collection_name)
            .await?
            .insert_typed(id, embedding, memory_type, metadata)
            .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn upsert(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()> {
        self.default_col()
            .await?
            .upsert(id, embedding, metadata)
            .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn insert_many(&self, docs: &[(String, Vec<f32>, Option<Value>)]) -> Result<()> {
        self.default_col().await?.insert_many(docs).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn upsert_many(&self, docs: &[(String, Vec<f32>, Option<Value>)]) -> Result<()> {
        self.default_col().await?.upsert_many(docs).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get(&self, id: &str) -> Result<Option<Document>> {
        self.default_col().await?.get(id).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_at_snapshot(&self, id: &str, seq_no: u64) -> Result<Option<Document>> {
        self.default_col().await?.get_at_snapshot(id, seq_no).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn last_committed_seq(&self) -> Result<u64> {
        self.storage.last_seq_no().await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn create_snapshot(&self) -> Result<u64> {
        self.storage.last_seq_no().await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn update(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()> {
        self.default_col()
            .await?
            .update(id, embedding, metadata)
            .await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        self.default_col()
            .await?
            .query()
            .embedding(query)
            .k(k)
            .execute()
            .await
    }

    #[deprecated(
        since = "0.1.0",
        note = "Use search_with_filter_expr with contextra_types::FilterExpr directly"
    )]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn search_with_filter(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<MetadataFilter>,
    ) -> Result<Vec<SearchResult>> {
        let col = self.default_col().await?;
        let mut builder = col.query().vector(query).k(k);
        if let Some(f) = filter {
            builder = builder.metadata_filter(f);
        }
        builder.execute().await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn search_with_filter_expr(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<FilterExpr>,
    ) -> Result<Vec<SearchResult>> {
        let col = self.default_col().await?;
        let mut builder = col.query().embedding(query).k(k);
        if let Some(f) = filter {
            builder = builder.filter(f);
        }
        builder.execute().await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn insert_text_only(
        &self,
        id: &str,
        text: &str,
        metadata: Option<Value>,
    ) -> Result<()> {
        self.default_col()
            .await?
            .insert_text_only(id, text, metadata)
            .await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn upsert_text_only(
        &self,
        id: &str,
        text: &str,
        metadata: Option<Value>,
    ) -> Result<()> {
        self.default_col()
            .await?
            .upsert_text_only(id, text, metadata)
            .await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn search_text(&self, text: &str, k: usize) -> Result<Vec<SearchResult>> {
        self.default_col()
            .await?
            .query()
            .text(text)
            .k(k)
            .execute()
            .await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, filter))]
    pub async fn search_filtered(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>,
    ) -> Result<Vec<SearchResult>> {
        #[allow(deprecated)]
        self.default_col()
            .await?
            .search_filtered(query, k, filter)
            .await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, anchor_entities))]
    pub async fn hybrid_search(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        anchor_entities: Option<&[contextra_types::EntityId]>,
    ) -> Result<Vec<SearchResult>> {
        let col = self.default_col().await?;
        let mut builder = col.query().text(text).vector(vector).k(k);
        if let Some(anchors) = anchor_entities {
            builder = builder.anchors(anchors.iter().copied());
        }
        builder.execute().await
    }

    #[cfg(feature = "reranking")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, reranker, anchor_entities))]
    pub async fn hybrid_search_reranked(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        reranker: Option<&contextra_infer_onnx::CrossEncoderReranker>,
        anchor_entities: Option<&[contextra_types::EntityId]>,
    ) -> Result<Vec<SearchResult>> {
        let col = self.default_col().await?;
        let mut builder = col.query().text(text).vector(vector).k(k);
        if let Some(r) = reranker {
            builder = builder.reranker(r);
        }
        if let Some(anchors) = anchor_entities {
            builder = builder.anchors(anchors.iter().copied());
        }
        builder.execute().await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, anchor_entities, weights))]
    pub async fn hybrid_search_with_weights(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        anchor_entities: Option<&[contextra_types::EntityId]>,
        weights: Option<&contextra_types::FusionWeights>,
    ) -> Result<Vec<SearchResult>> {
        let col = self.default_col().await?;
        let mut builder = col.query().text(text).vector(vector).k(k);
        if let Some(w) = weights {
            builder = builder.fusion_weights(w.clone());
        }
        if let Some(anchors) = anchor_entities {
            builder = builder.anchors(anchors.iter().copied());
        }
        builder.execute().await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, anchor_entities, weights, strategy))]
    pub async fn hybrid_search_with_strategy(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        anchor_entities: Option<&[contextra_types::EntityId]>,
        weights: Option<&contextra_types::FusionWeights>,
        strategy: Option<&contextra_types::GraphTraversalStrategy>,
    ) -> Result<Vec<SearchResult>> {
        let col = self.default_col().await?;
        let mut builder = col.query().text(text).vector(vector).k(k);
        if let Some(w) = weights {
            builder = builder.fusion_weights(w.clone());
        }
        if let Some(s) = strategy {
            builder = builder.strategy(s.clone());
        }
        if let Some(anchors) = anchor_entities {
            builder = builder.anchors(anchors.iter().copied());
        }
        builder.execute().await
    }

    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, query))]
    pub async fn hybrid_search_with_query(
        &self,
        query: &contextra_types::HybridQuery,
    ) -> Result<Vec<SearchResult>> {
        self.default_col()
            .await?
            .query()
            .query_config(query)
            .execute()
            .await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.default_col().await?.delete(id).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn relate(&self, from: &str, to: &str, label: &str) -> Result<()> {
        let col = self.default_col().await?;
        col.relate(from, to, label).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn relate_bidirectional(&self, from: &str, to: &str, label: &str) -> Result<()> {
        let col = self.default_col().await?;
        col.relate_bidirectional(from, to, label).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn scan_prefix(
        &self,
        prefix: &str,
        limit: Option<usize>,
    ) -> Result<Vec<(String, Value)>> {
        self.default_col().await?.scan_prefix(prefix, limit).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn len(&self) -> Result<usize> {
        Ok(self.default_col().await?.len().await)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn is_empty(&self) -> Result<bool> {
        Ok(self.default_col().await?.is_empty().await)
    }

    #[tracing::instrument(level = "trace", skip(self, start, end))]
    pub async fn scan(
        &self,
        start: std::ops::Bound<&[u8]>,
        end: std::ops::Bound<&[u8]>,
        limit: Option<usize>,
    ) -> Result<Vec<(String, Value)>> {
        self.default_col().await?.scan(start, end, limit).await
    }
}
