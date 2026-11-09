// Verbatim backup of Bm25Scorer port implementation from crates/contextra-text/src/lib.rs
// For handoff to Class C consumer / adapter.

use contextra_ports::{StorageEngine, TextIndex, TextIndexStats};
use contextra_types::{DocId, Result, ScoredDocument, TxId};
use crate::inverted::InvertedIndex;

pub struct Bm25Scorer<S: StorageEngine> {
    index: InvertedIndex<S>,
}

impl<S: StorageEngine> Bm25Scorer<S> {
    pub fn new(storage: std::sync::Arc<S>, namespace: &str) -> Self {
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

    async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        self.index.delete(tx, id).await
    }

    async fn commit(&self, tx: TxId) -> Result<()> {
        self.index.commit(tx).await
    }

    async fn rollback(&self, tx: TxId) -> Result<()> {
        self.index.rollback(tx).await
    }

    async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()> {
        self.index.rollback_to_tx(tx_id).await
    }

    async fn last_tx_id(&self) -> Result<TxId> {
        self.index.last_tx_id().await
    }

    async fn len(&self) -> usize {
        self.index.len().await
    }

    async fn stats(&self) -> Result<TextIndexStats> {
        self.index.stats().await
    }
}
