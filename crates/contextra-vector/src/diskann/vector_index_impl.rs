// FILE-CONTEXT
// ZWECK: DiskANN-Graphindex für Out-of-Core Approximate Nearest Neighbor Search (WP-4.3).
// INVARIANTEN: Lock-Hierarchie: header -> mmap -> cache / quantizer / doc_ids; atomic rename + parent dir sync bei file persistence.
// NICHT-OFFENSICHTLICH: Mmap für Vektor- & Graphlesezugriffe, ausgelagert an contextra-sys::mmap_readonly.
// STAND: TS:2026-09-10T19:30:00Z (SESSION: a9d67eae)

use super::format::compute_adaptive_flush_threshold;
use super::types::DiskAnnIndex;
use contextra_simd::validate_vector;
use contextra_core::{
    DocId, ContextraError, Result, ScoredDocument, TxId, VectorIndex, VectorIndexStats,
};
use std::sync::atomic::Ordering;

impl VectorIndex for DiskAnnIndex {
    async fn insert(&self, tx: TxId, id: DocId, embedding: &[f32]) -> Result<()> {
        let fallback_opt = self.inner.hnsw_fallback.read().clone();
        if let Some(hnsw) = fallback_opt {
            return hnsw.insert(tx, id, embedding).await;
        }

        validate_vector(embedding)?;
        self.check_quantizer_drift(embedding);

        let pending_wal = self.inner.config.index_path.with_extension("pending.wal");
        Self::append_to_pending_wal(&pending_wal, id, embedding).await?;

        let count = {
            let mut pending = self.inner.pending_inserts.write();
            pending.push((id, embedding.to_vec()));
            self.inner.pending_count.fetch_add(1, Ordering::Relaxed) + 1
        };

        // config.pending_flush_threshold dient als Benchmark-Override (z.B. für Tests).
        // Im Normalfall: adaptiver Schwellenwert basierend auf der aktuellen
        // Collection-Größe (ADR-068).
        let n_persisted = self.len().await as u64;
        let threshold = self
            .inner
            .config
            .pending_flush_threshold
            .unwrap_or_else(|| compute_adaptive_flush_threshold(n_persisted));

        if count >= threshold {
            self.trigger_background_persist_delta();
        }
        Ok(())
    }

    async fn search(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>> {
        self.search_internal(query, k).await
    }

    #[allow(clippy::unnecessary_cast)]
    async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        let fallback_opt = self.inner.hnsw_fallback.read().clone();
        if let Some(hnsw) = fallback_opt {
            return hnsw.delete(tx, id).await;
        }

        let doc_id_u64 = id.inner();

        if self.inner.tombstones.read().contains(doc_id_u64 as u64) {
            return Err(ContextraError::NotFound(format!(
                "DocId {} not found in index or already deleted",
                doc_id_u64
            )));
        }

        let exists_in_doc_ids = self.inner.doc_ids.read().contains(&id);
        let exists_in_pending = self
            .inner
            .pending_inserts
            .read()
            .iter()
            .any(|(pid, _)| *pid == id);

        if !exists_in_doc_ids && !exists_in_pending {
            return Err(ContextraError::NotFound(format!(
                "DocId {} not found in DiskANN index",
                doc_id_u64
            )));
        }

        self.inner.tombstones.write().insert(doc_id_u64 as u64);

        let tombstone_wal = self.inner.config.index_path.with_extension("tombstone.wal");
        Self::append_to_tombstone_wal(&tombstone_wal, id).await?;

        Ok(())
    }

    async fn commit(&self, tx: TxId) -> Result<()> {
        let fallback_opt = self.inner.hnsw_fallback.read().clone();
        if let Some(hnsw) = fallback_opt {
            return hnsw.commit(tx).await;
        }
        Ok(())
    }

    async fn rollback(&self, tx: TxId) -> Result<()> {
        let fallback_opt = self.inner.hnsw_fallback.read().clone();
        if let Some(hnsw) = fallback_opt {
            return hnsw.rollback(tx).await;
        }
        Ok(())
    }

    async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()> {
        let fallback_opt = self.inner.hnsw_fallback.read().clone();
        if let Some(hnsw) = fallback_opt {
            return hnsw.rollback_to_tx(tx_id).await;
        }
        Ok(())
    }

    #[allow(clippy::unnecessary_cast)]
    async fn all_doc_ids(&self) -> Result<Vec<DocId>> {
        let fallback_opt = self.inner.hnsw_fallback.read().clone();
        if let Some(hnsw) = fallback_opt {
            return hnsw.all_doc_ids().await;
        }
        let tombstones = self.inner.tombstones.read();
        let mut ids = Vec::new();

        for &id in self.inner.doc_ids.read().iter() {
            if !tombstones.contains(id.inner() as u64) {
                ids.push(id);
            }
        }
        for (id, _) in self.inner.pending_inserts.read().iter() {
            if !tombstones.contains(id.inner() as u64) && !ids.contains(id) {
                ids.push(*id);
            }
        }
        Ok(ids)
    }

    async fn last_tx_id(&self) -> Result<TxId> {
        Ok(TxId(0))
    }

    async fn len(&self) -> usize {
        let fallback_opt = self.inner.hnsw_fallback.read().clone();
        if let Some(hnsw) = fallback_opt {
            return hnsw.len().await;
        }
        self.all_doc_ids().await.map(|ids| ids.len()).unwrap_or(0)
    }

    async fn stats(&self) -> Result<VectorIndexStats> {
        use std::sync::atomic::Ordering;
        let count = self.len().await;
        let node_size = self.inner.node_size_bytes.load(Ordering::SeqCst) as usize;
        let cache_usage = self.inner.cache.read().len() * node_size;

        let total_nodes = {
            let disk_nodes = self
                .inner
                .header
                .read()
                .map(|h| h.node_count as usize)
                .unwrap_or(0);
            let pending_nodes = self.inner.pending_inserts.read().len();
            disk_nodes + pending_nodes
        };
        let deleted_count = self.inner.tombstones.read().len() as usize;
        let deleted_ratio = if total_nodes > 0 {
            deleted_count as f64 / total_nodes as f64
        } else {
            0.0
        };

        Ok(VectorIndexStats {
            num_vectors: count,
            memory_usage_bytes: cache_usage,
            num_layers: 1,
            deleted_ratio,
            rebuild_count: 0,
        })
    }
}
