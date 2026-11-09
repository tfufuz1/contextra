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
        self.index.search(query, k).await
    }

    async fn search_at(&self, query: &str, k: usize, seq_no: u64) -> Result<Vec<ScoredDocument>> {
        self.index.search_at(query, k, seq_no).await
    }

    async fn insert(&self, tx: TxId, id: DocId, text: &str) -> Result<()> {
        self.index.insert(tx, id, text).await
    }

    async fn delete_document(&self, tx: TxId, id: DocId) -> Result<()> {
        self.index.delete_document(tx, id).await
    }

    async fn commit(&self, tx: TxId) -> Result<()> {
        self.index.commit(tx).await
    }

    async fn rollback(&self, tx: TxId) -> Result<()> {
        self.index.rollback(tx).await
    }

    fn stats(&self) -> TextIndexStats {
        self.index.stats()
    }
}
