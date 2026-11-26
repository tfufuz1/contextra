//! Port bridge adapter for `contextra-text` BM25 scoring.

use contextra_ports::{StorageEngine, TextIndex, TextIndexStats};
use contextra_text::inverted::InvertedIndex;
use contextra_types::{DocId, Result, ScoredDocument, TxId};
use std::sync::Arc;

/// Adapter implementing [`TextIndex`] on top of [`InvertedIndex`].
pub struct Bm25Scorer<S: StorageEngine> {
    index: InvertedIndex<S>,
}

impl<S: StorageEngine> Bm25Scorer<S> {
    /// Creates a new `Bm25Scorer` with the given storage and namespace.
    pub fn new(storage: Arc<S>, namespace: &str) -> Self {
        Self {
            index: InvertedIndex::new(storage, namespace),
        }
    }
}

impl<S: StorageEngine> TextIndex for Bm25Scorer<S> {
    async fn search(&self, query: &str, k: usize) -> Result<Vec<ScoredDocument>> {
        TextIndex::search(&self.index, query, k).await
    }

    async fn search_at(&self, query: &str, k: usize, seq_no: u64) -> Result<Vec<ScoredDocument>> {
        TextIndex::search_at(&self.index, query, k, seq_no).await
    }

    async fn insert(&self, tx: TxId, id: DocId, text: &str) -> Result<()> {
        TextIndex::insert(&self.index, tx, id, text).await
    }

    async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        TextIndex::delete(&self.index, tx, id).await
    }

    async fn commit(&self, tx: TxId) -> Result<()> {
        TextIndex::commit(&self.index, tx).await
    }

    async fn rollback(&self, tx: TxId) -> Result<()> {
        TextIndex::rollback(&self.index, tx).await
    }

    async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()> {
        TextIndex::rollback_to_tx(&self.index, tx_id).await
    }

    async fn last_tx_id(&self) -> Result<TxId> {
        TextIndex::last_tx_id(&self.index).await
    }

    async fn len(&self) -> usize {
        TextIndex::len(&self.index).await
    }

    async fn stats(&self) -> Result<TextIndexStats> {
        TextIndex::stats(&self.index).await
    }
}
