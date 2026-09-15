//! Text index, text embedding engine, and segment synthesizer traits.

// FILE-CONTEXT
// STAND: 2026-09-15T00:00:00Z
// ZWECK: TextIndex, TextEmbeddingEngine & SegmentSynthesizer Trait-Definitionen für BM25/Inverted Index.
// INVARIANTEN: AFIT for TextIndex, BoxFuture for TextEmbeddingEngine.

use super::BoxFuture;
use crate::types::{DocId, ScoredDocument, TxId};
use crate::Result;
use serde::{Deserialize, Serialize};
use std::future::Future;

/// Text embedding engine trait.
pub trait TextEmbeddingEngine: Send + Sync + 'static {
    /// Generates an embedding for the given text.
    fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>>>;

    /// Generates embeddings for multiple texts.
    /// Default implementation executes sequential calls.
    fn embed_batch<'a>(&'a self, texts: &'a [&'a str]) -> BoxFuture<'a, Result<Vec<Vec<f32>>>> {
        Box::pin(async move {
            let mut results = Vec::with_capacity(texts.len());
            for text in texts {
                results.push(self.embed(text).await?);
            }
            Ok(results)
        })
    }
}

/// Trait-Abstraktion für LLM-Synthesizer zur Segment-Zusammenfassung (REM-Phase).
pub trait SegmentSynthesizer: Send + Sync {
    /// Synthetisiert ein Segment von Texten zu einer abstrakten Zusammenfassung.
    fn synthesize_segment<'a>(
        &'a self,
        segment_texts: &'a [&'a str],
    ) -> BoxFuture<'a, Result<String>>;
    /// Gibt die Modell-ID des Synthesizers zurück.
    fn model_id(&self) -> &str;
}

/// Statistics for a text index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextIndexStats {
    /// Total number of documents indexed.
    pub num_documents: usize,
    /// Total number of tokens across all documents.
    pub num_tokens: usize,
    /// Estimated memory usage in bytes.
    pub memory_usage_bytes: usize,
}

/// Text-Index Trait — abstrahiert BM25/Inverted-Index-Operationen.
///
/// # Dyn-Kompatibilität
/// Verwendet native `async fn` (AFIT) für statischen Dispatch.
pub trait TextIndex: Send + Sync + 'static {
    /// Searches for documents matching the query.
    fn search(
        &self,
        query: &str,
        k: usize,
    ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send;

    /// Searches for documents matching the query at a specific sequence number.
    ///
    /// # Errors
    /// Returns [`MemFuseError::CapabilityUnsupported`][crate::MemFuseError::CapabilityUnsupported]
    /// with capability `"snapshot_read_at"` if snapshot-isolated text search is not implemented.
    /// Tested via `capability_coverage` test module.
    fn search_at(
        &self,
        query: &str,
        k: usize,
        seq_no: u64,
    ) -> impl Future<Output = Result<Vec<ScoredDocument>>> + Send {
        async move {
            let _ = (query, k, seq_no);
            Err(crate::error::MemFuseError::capability_unsupported(
                "snapshot_read_at",
                "Text search snapshot isolation (search_at) is not supported by default — tracked in ADR-024",
            ))
        }
    }

    /// Inserts or updates a document in the index.
    fn insert(&self, tx: TxId, id: DocId, text: &str) -> impl Future<Output = Result<()>> + Send;

    /// Deletes a document from the index.
    fn delete(&self, tx: TxId, id: DocId) -> impl Future<Output = Result<()>> + Send;

    /// Commits a transaction.
    fn commit(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Rolls back a transaction.
    fn rollback(&self, tx: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Rolls back the entire index state to a specific transaction ID.
    fn rollback_to_tx(&self, tx_id: TxId) -> impl Future<Output = Result<()>> + Send;

    /// Returns the last transaction ID processed by the index.
    fn last_tx_id(&self) -> impl Future<Output = Result<TxId>> + Send;

    /// Returns the number of documents in the index.
    fn len(&self) -> impl Future<Output = usize> + Send;

    /// Returns true if the index is empty.
    fn is_empty(&self) -> impl Future<Output = bool> + Send {
        async { self.len().await == 0 }
    }

    /// Returns index statistics.
    fn stats(&self) -> impl Future<Output = Result<TextIndexStats>> + Send;
}
