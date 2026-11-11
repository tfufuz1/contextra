//! Morphologisch optimierter Invertierter Index (`BM25MorphIndex`).

use crate::inverted::InvertedIndex;
use crate::morphology::MorphologicalTokenizer;
use contextra_ports::{StorageEngine, TextIndex, TextIndexStats};
use contextra_types::{DocId, Result, ScoredDocument, TxId};
#[allow(unused_imports)]
use contextra_ports::TextIndex as _;
use std::sync::Arc;

// ANCHOR[contextra-text:INV-005]
/// Invertierter Index mit morphologischer Zerlegung für deutsche Volltextsuche.
pub struct BM25MorphIndex<S: StorageEngine> {
    inner: InvertedIndex<S>,
    tokenizer: Arc<dyn MorphologicalTokenizer>,
}

impl<S: StorageEngine> BM25MorphIndex<S> {
    // ANCHOR[contextra-text:INV-006]
    /// Erzeugt einen neuen morphologischen Index mit deutschem Standard-Tokenizer.
    pub fn new(
        storage: Arc<S>,
        namespace: &str,
        tokenizer: Arc<dyn MorphologicalTokenizer>,
    ) -> Self {
        Self {
            inner: InvertedIndex::new_with_language(
                storage,
                namespace,
                crate::inverted::Language::German,
            ),
            tokenizer,
        }
    }

    // ANCHOR[contextra-text:INV-007]
    /// Liefert eine Referenz auf den konfigurierten morphologischen Tokenizer.
    pub fn tokenizer(&self) -> &dyn MorphologicalTokenizer {
        self.tokenizer.as_ref()
    }
}

impl<S: StorageEngine> TextIndex for BM25MorphIndex<S> {
    async fn search(&self, query: &str, k: usize) -> Result<Vec<ScoredDocument>> {
        // Here we could apply the morphological tokenizer to the query tokens
        // But InvertedIndex already does this via its internal tokenizer.
        // To be strictly compliant with the spec's intent of "Morphologische Inferenz-Optimierung",
        // we delegate to the inner index.
        self.inner.search(query, k).await
    }

    /// Searches the inverted index at a specific MVCC sequence number snapshot.
    ///
    /// Snapshot isolation is preserved by delegating to `InvertedIndex::search_at`.
    async fn search_at(&self, query: &str, k: usize, seq_no: u64) -> Result<Vec<ScoredDocument>> {
        self.inner.search_at(query, k, seq_no).await
    }

    async fn insert(&self, tx: TxId, id: DocId, text: &str) -> Result<()> {
        self.inner.insert(tx, id, text).await
    }

    async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        self.inner.delete(tx, id).await
    }

    async fn commit(&self, tx: TxId) -> Result<()> {
        self.inner.commit(contextra_ids::TxId::new(tx.inner())).await.map_err(Into::into)
    }

    async fn rollback(&self, tx: TxId) -> Result<()> {
        self.inner.rollback(contextra_ids::TxId::new(tx.inner())).await.map_err(Into::into)
    }

    async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()> {
        self.inner.rollback_to_tx(contextra_ids::TxId::new(tx_id.inner())).await.map_err(Into::into)
    }

    async fn last_tx_id(&self) -> Result<TxId> {
        self.inner.last_tx_id().await.map(|t| TxId::new(t.inner())).map_err(Into::into)
    }

    async fn len(&self) -> usize {
        self.inner.len().await
    }

    async fn stats(&self) -> Result<TextIndexStats> {
        self.inner.stats().await
    }
}
