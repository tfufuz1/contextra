//! Ports-Implementierung von `TextIndex` für `InvertedIndex` in `compat/` (SPEC §6.2 text, B04).
// ANCHOR[contextra-text:INV-PORTS-001]

use crate::inverted::{InvertedIndex, TextIndexMetadata};
use crate::storage::PostingStore;
use contextra_ids::{DocId, TxId};
use contextra_ports::{TextIndex, TextIndexStats};
use contextra_types::{Result, ScoredDocument};

impl<S: PostingStore + 'static> TextIndex for InvertedIndex<S> {
    async fn search(&self, query: &str, k: usize) -> Result<Vec<ScoredDocument>> {
        let results = self
            .search_bm25(query, k, None)
            .await
            .map_err(contextra_types::ContextraError::from)?;
        Ok(results
            .into_iter()
            .map(|(doc_id, score)| ScoredDocument {
                doc_id: contextra_types::DocId::new(doc_id.inner() as _),
                score,
            })
            .collect())
    }

    async fn search_at(&self, query: &str, k: usize, seq_no: u64) -> Result<Vec<ScoredDocument>> {
        let results = self
            .search_bm25(query, k, Some(seq_no))
            .await
            .map_err(contextra_types::ContextraError::from)?;
        Ok(results
            .into_iter()
            .map(|(doc_id, score)| ScoredDocument {
                doc_id: contextra_types::DocId::new(doc_id.inner() as _),
                score,
            })
            .collect())
    }

    async fn insert(&self, tx: contextra_types::TxId, id: contextra_types::DocId, text: &str) -> Result<()> {
        self.upsert_document(TxId::new(tx.inner()), DocId::new(id.inner() as _), text)
            .await
            .map_err(contextra_types::ContextraError::from)
    }

    async fn delete(&self, tx: contextra_types::TxId, id: contextra_types::DocId) -> Result<()> {
        self.delete_document(TxId::new(tx.inner()), DocId::new(id.inner() as _))
            .await
            .map_err(contextra_types::ContextraError::from)
    }

    async fn commit(&self, tx: contextra_types::TxId) -> Result<()> {
        let local_tx = TxId::new(tx.inner());
        self.commit_stats(local_tx)
            .await
            .map_err(contextra_types::ContextraError::from)?;
        self.storage().commit(local_tx)
            .await
            .map_err(contextra_types::ContextraError::from)
    }

    async fn rollback(&self, tx: contextra_types::TxId) -> Result<()> {
        let local_tx = TxId::new(tx.inner());
        self.rollback_stats(local_tx)
            .await
            .map_err(contextra_types::ContextraError::from)?;
        self.storage().rollback(local_tx)
            .await
            .map_err(contextra_types::ContextraError::from)
    }

    async fn rollback_to_tx(&self, tx_id: contextra_types::TxId) -> Result<()> {
        self.storage().rollback_to_tx(TxId::new(tx_id.inner()))
            .await
            .map_err(contextra_types::ContextraError::from)
    }

    async fn last_tx_id(&self) -> Result<contextra_types::TxId> {
        let id = self.storage().last_tx_id()
            .await
            .map_err(contextra_types::ContextraError::from)?;
        Ok(contextra_types::TxId::new(id.inner()))
    }

    async fn len(&self) -> usize {
        self.len_sync()
    }

    async fn stats(&self) -> Result<TextIndexStats> {
        let meta_key = self.key("meta:stats");
        let meta = if let Some(bytes) = self
            .storage()
            .get(&meta_key)
            .await
            .map_err(contextra_types::ContextraError::from)?
        {
            bincode::deserialize::<TextIndexMetadata>(&bytes)
                .map_err(|e| contextra_types::ContextraError::Storage(format!("bincode: {}", e)))?
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
