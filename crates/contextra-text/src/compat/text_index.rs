//! Port bridge for TextIndex implementation on InvertedIndex (SPEC §4.3).

use crate::inverted::{InvertedIndex, TextIndexMetadata};
use crate::error::TextError;
use crate::storage::PostingStore;
use contextra_ports::{TextIndex, TextIndexStats};
use contextra_types::{ScoredDocument, TxId};
use std::sync::atomic::Ordering;

#[cfg(feature = "workspace")]
impl<S: PostingStore + 'static> TextIndex for InvertedIndex<S> {
    async fn search(&self, query: &str, k: usize) -> contextra_types::Result<Vec<ScoredDocument>> {
        let results = self.search_bm25(query, k, None).await.map_err(contextra_types::ContextraError::from)?;
        Ok(results
            .into_iter()
            .map(|(doc_id, score)| ScoredDocument { doc_id: doc_id.into(), score })
            .collect())
    }

    /// Searches the inverted index at a specific MVCC sequence number snapshot.
    ///
    /// Snapshot isolation is preserved by delegating to `search_bm25_at()`, which uses
    /// `scan_prefix_at()` and `get_at_seq()` on the underlying `StorageEngine`.
    async fn search_at(&self, query: &str, k: usize, seq_no: u64) -> contextra_types::Result<Vec<ScoredDocument>> {
        let results = self.search_bm25(query, k, Some(seq_no)).await.map_err(contextra_types::ContextraError::from)?;
        Ok(results
            .into_iter()
            .map(|(doc_id, score)| ScoredDocument { doc_id: doc_id.into(), score })
            .collect())
    }

    async fn insert(&self, tx: TxId, id: contextra_types::DocId, text: &str) -> contextra_types::Result<()> {
        self.upsert_document(tx.into(), id.into(), text).await.map_err(Into::into)
    }

    async fn delete(&self, tx: TxId, id: contextra_types::DocId) -> contextra_types::Result<()> {
        self.delete_document(tx.into(), id.into()).await.map_err(Into::into)
    }

    async fn commit(&self, tx: TxId) -> contextra_types::Result<()> {
        self.commit_stats(tx.into()).await.map_err(contextra_types::ContextraError::from)?;
        self.storage().commit(tx.into()).await.map_err(Into::into)
    }

    async fn rollback(&self, tx: TxId) -> contextra_types::Result<()> {
        self.rollback_stats(tx.into()).await.map_err(contextra_types::ContextraError::from)?;
        self.storage().rollback(tx.into()).await.map_err(Into::into)
    }

    async fn rollback_to_tx(&self, tx_id: TxId) -> contextra_types::Result<()> {
        self.storage().rollback_to_tx(tx_id.into()).await.map_err(Into::into)
    }

    async fn last_tx_id(&self) -> contextra_types::Result<TxId> {
        self.storage().last_tx_id().await.map(|tx| tx.into()).map_err(Into::into)
    }

    async fn len(&self) -> usize {
        self.total_docs.load(Ordering::Acquire) as usize
    }

    async fn stats(&self) -> contextra_types::Result<TextIndexStats> {
        let meta_key = self.key("meta:stats");
        let meta = if let Some(ref bytes) = self.storage().get(&meta_key).await.map_err(contextra_types::ContextraError::from)? {
            bincode::deserialize::<TextIndexMetadata>(&bytes)
                .map_err(|e| TextError::Storage(format!("bincode: {}", e)))?
        } else {
            TextIndexMetadata::default()
        };

        Ok(TextIndexStats {
            num_documents: meta.total_docs as usize,
            num_tokens: meta.total_tokens as usize,
            memory_usage_bytes: 0,
        })
    }
}
