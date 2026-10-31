//! # Modul: crud.rs
//!
//! **Zweck:** High-Level CRUD-Operationen auf Collections über die Haupt-Facade.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, ACID/SSI-Transaktionsgarantien.

use crate::*;
use contextra_store::LsmStorage;
use contextra_types::{FilterExpr, Result};
use serde_json::Value;
use std::sync::Arc;

impl Contextra {
    // ANCHOR[contextra-engine:DEF-015]
    /// Dokumentation für `default_col` im Rahmen der Transaktions- und Facade-Orchestrierung.
    pub(crate) async fn default_col(&self) -> Result<Arc<Collection<LsmStorage>>> {
        self.collection("default").await
    }

    // ANCHOR[contextra-engine:PUT-020]
    /// Dokumentation für `put_kv` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self, value))]
    pub async fn put_kv(&self, id: &str, value: &Value) -> Result<()> {
        self.default_col().await?.put_kv(id, value).await
    }

    // ANCHOR[contextra-engine:GET-025]
    /// Dokumentation für `get_kv` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_kv(&self, id: &str) -> Result<Option<Value>> {
        self.default_col().await?.get_kv(id).await
    }

    // ANCHOR[contextra-engine:INS-030]
    /// Dokumentation für `insert` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn insert(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()> {
        self.default_col()
            .await?
            .insert(id, embedding, metadata)
            .await
    }

    // ANCHOR[contextra-engine:INS-038]
    /// Dokumentation für `insert_typed` im Rahmen der Transaktions- und Facade-Orchestrierung.
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

    // ANCHOR[contextra-engine:UPS-053]
    /// Dokumentation für `upsert` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn upsert(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()> {
        self.default_col()
            .await?
            .upsert(id, embedding, metadata)
            .await
    }

    // ANCHOR[contextra-engine:INS-061]
    /// Dokumentation für `insert_many` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn insert_many(&self, docs: &[(String, Vec<f32>, Option<Value>)]) -> Result<()> {
        self.default_col().await?.insert_many(docs).await
    }

    // ANCHOR[contextra-engine:UPS-066]
    /// Dokumentation für `upsert_many` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn upsert_many(&self, docs: &[(String, Vec<f32>, Option<Value>)]) -> Result<()> {
        self.default_col().await?.upsert_many(docs).await
    }

    // ANCHOR[contextra-engine:GET-071]
    /// Dokumentation für `get` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get(&self, id: &str) -> Result<Option<Document>> {
        self.default_col().await?.get(id).await
    }

    // ANCHOR[contextra-engine:GET-076]
    /// Dokumentation für `get_at_snapshot` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_at_snapshot(&self, id: &str, seq_no: u64) -> Result<Option<Document>> {
        self.default_col().await?.get_at_snapshot(id, seq_no).await
    }

    // ANCHOR[contextra-engine:LAS-081]
    /// Dokumentation für `last_committed_seq` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn last_committed_seq(&self) -> Result<u64> {
        self.storage.last_seq_no().await
    }

    // ANCHOR[contextra-engine:CRE-086]
    /// Dokumentation für `create_snapshot` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn create_snapshot(&self) -> Result<u64> {
        self.storage.last_seq_no().await
    }

    // ANCHOR[contextra-engine:UPD-091]
    /// Dokumentation für `update` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn update(&self, id: &str, embedding: &[f32], metadata: Option<Value>) -> Result<()> {
        self.default_col()
            .await?
            .update(id, embedding, metadata)
            .await
    }

    // ANCHOR[contextra-engine:SEA-100]
    /// Dokumentation für `search` im Rahmen der Transaktions- und Facade-Orchestrierung.
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
    // ANCHOR[contextra-engine:SEA-116]
    /// Dokumentation für `search_with_filter` im Rahmen der Transaktions- und Facade-Orchestrierung.
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

    // ANCHOR[contextra-engine:SEA-132]
    /// Dokumentation für `search_with_filter_expr` im Rahmen der Transaktions- und Facade-Orchestrierung.
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

    // ANCHOR[contextra-engine:INS-147]
    /// Dokumentation für `insert_text_only` im Rahmen der Transaktions- und Facade-Orchestrierung.
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

    // ANCHOR[contextra-engine:UPS-160]
    /// Dokumentation für `upsert_text_only` im Rahmen der Transaktions- und Facade-Orchestrierung.
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

    // ANCHOR[contextra-engine:SEA-174]
    /// Dokumentation für `search_text` im Rahmen der Transaktions- und Facade-Orchestrierung.
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

    // ANCHOR[contextra-engine:SEA-186]
    /// Dokumentation für `search_filtered` im Rahmen der Transaktions- und Facade-Orchestrierung.
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

    // ANCHOR[contextra-engine:HYB-201]
    /// Dokumentation für `hybrid_search` im Rahmen der Transaktions- und Facade-Orchestrierung.
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

    // ANCHOR[contextra-engine:HYB-218]
    /// Dokumentation für `hybrid_search_reranked` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, reranker, anchor_entities))]
    pub async fn hybrid_search_reranked(
        &self,
        text: &str,
        vector: &[f32],
        k: usize,
        reranker: Option<&dyn contextra_ports::Reranker>,
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

    // ANCHOR[contextra-engine:HYB-239]
    /// Dokumentation für `hybrid_search_with_weights` im Rahmen der Transaktions- und Facade-Orchestrierung.
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

    // ANCHOR[contextra-engine:HYB-260]
    /// Dokumentation für `hybrid_search_with_strategy` im Rahmen der Transaktions- und Facade-Orchestrierung.
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

    // ANCHOR[contextra-engine:HYB-285]
    /// Dokumentation für `hybrid_search_with_query` im Rahmen der Transaktions- und Facade-Orchestrierung.
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

    // ANCHOR[contextra-engine:DEL-297]
    /// Dokumentation für `delete` im Rahmen der Transaktions- und Facade-Orchestrierung.
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.default_col().await?.delete(id).await
    }

    // ANCHOR[contextra-engine:REL-302]
    /// Dokumentation für `relate` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn relate(&self, from: &str, to: &str, label: &str) -> Result<()> {
        let col = self.default_col().await?;
        col.relate(from, to, label).await
    }

    // ANCHOR[contextra-engine:REL-308]
    /// Dokumentation für `relate_bidirectional` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn relate_bidirectional(&self, from: &str, to: &str, label: &str) -> Result<()> {
        let col = self.default_col().await?;
        col.relate_bidirectional(from, to, label).await
    }

    // ANCHOR[contextra-engine:SCA-314]
    /// Dokumentation für `scan_prefix` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn scan_prefix(
        &self,
        prefix: &str,
        limit: Option<usize>,
    ) -> Result<Vec<(String, Value)>> {
        self.default_col().await?.scan_prefix(prefix, limit).await
    }

    // ANCHOR[contextra-engine:LEN-323]
    /// Dokumentation für `len` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn len(&self) -> Result<usize> {
        Ok(self.default_col().await?.len().await)
    }

    // ANCHOR[contextra-engine:IS_-328]
    /// Dokumentation für `is_empty` im Rahmen der Transaktions- und Facade-Orchestrierung.
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn is_empty(&self) -> Result<bool> {
        Ok(self.default_col().await?.is_empty().await)
    }

    // ANCHOR[contextra-engine:SCA-333]
    /// Dokumentation für `scan` im Rahmen der Transaktions- und Facade-Orchestrierung.
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
