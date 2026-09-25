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
    /// Returns [`ContextraError::CapabilityUnsupported`][crate::ContextraError::CapabilityUnsupported]
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
            Err(crate::error::ContextraError::capability_unsupported(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_index_stats_serialization() {
        let t_stats = TextIndexStats {
            num_documents: 10,
            num_tokens: 1000,
            memory_usage_bytes: 256,
        };
        let ser = serde_json::to_string(&t_stats).unwrap();
        let deser: TextIndexStats = serde_json::from_str(&ser).unwrap();
        assert_eq!(t_stats.num_documents, deser.num_documents);
    }

    #[tokio::test]
    async fn test_text_index_search_at_capability() {
        struct TextIndexPlaceholder;
        impl TextIndex for TextIndexPlaceholder {
            async fn search(&self, _: &str, _: usize) -> Result<Vec<ScoredDocument>> {
                Ok(vec![])
            }
            async fn search_at(&self, _: &str, _: usize, _: u64) -> Result<Vec<ScoredDocument>> {
                Ok(vec![])
            }
            async fn insert(&self, _: TxId, _: DocId, _: &str) -> Result<()> {
                Ok(())
            }
            async fn delete(&self, _: TxId, _: DocId) -> Result<()> {
                Ok(())
            }
            async fn commit(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback_to_tx(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn last_tx_id(&self) -> Result<TxId> {
                Ok(TxId(0))
            }
            async fn len(&self) -> usize {
                0
            }
            async fn stats(&self) -> Result<TextIndexStats> {
                Ok(TextIndexStats {
                    num_documents: 0,
                    num_tokens: 0,
                    memory_usage_bytes: 0,
                })
            }
        }
        let text_index = TextIndexPlaceholder;
        let res = text_index.search_at("test", 5, 1).await;
        assert!(
            !matches!(
                res,
                Err(crate::ContextraError::CapabilityUnsupported { .. })
            ),
            "search_at returned CapabilityUnsupported"
        );
    }

    #[tokio::test]
    async fn test_text_index_defaults() {
        struct MockTextIndex;
        impl TextIndex for MockTextIndex {
            async fn search(&self, _: &str, _: usize) -> Result<Vec<ScoredDocument>> {
                Ok(vec![])
            }
            async fn insert(&self, _: TxId, _: DocId, _: &str) -> Result<()> {
                Ok(())
            }
            async fn delete(&self, _: TxId, _: DocId) -> Result<()> {
                Ok(())
            }
            async fn commit(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn rollback_to_tx(&self, _: TxId) -> Result<()> {
                Ok(())
            }
            async fn last_tx_id(&self) -> Result<TxId> {
                Ok(TxId(0))
            }
            async fn len(&self) -> usize {
                0
            }
            async fn stats(&self) -> Result<TextIndexStats> {
                Ok(TextIndexStats {
                    num_documents: 0,
                    num_tokens: 0,
                    memory_usage_bytes: 0,
                })
            }
        }

        let index = MockTextIndex;
        let res = index.search_at("query", 10, 42).await;
        match res {
            Err(crate::error::ContextraError::CapabilityUnsupported { capability, reason }) => {
                assert_eq!(capability, "snapshot_read_at");
                assert!(reason.contains("ADR-024"), "Unexpected reason: {reason}");
            }
            _ => panic!("Expected CapabilityUnsupported for search_at"),
        }
    }
}
