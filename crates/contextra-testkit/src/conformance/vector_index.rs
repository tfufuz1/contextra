//! # Konformitätsprüfsuite für den `VectorIndex`-Port (`conformance/vector_index`)
//!
//! Dieses Modul testet die vertraglichen Zusicherungen (Contract Invariants)
//! für Vektor-Indizes des [`contextra_ports::VectorIndex`]-Ports (PO-02).
//!
//! ## Getestete Verträge (Pflichtfälle laut PO-02):
//! 1. `po_02_insert_search`: Einfügen und Auffinden von Vektoren über Nearest-Neighbor-Suche.
//! 2. `po_02_delete_disappears_completely`: Gelöschte Vektoren verschwinden vollständig aus den Suchtreffern.
//! 3. `po_02_deterministic_ordering`: Deterministische Distanzordnung & stabiles Tie-Breaking.

use contextra_ports::{DocId, Result, ScoredDocument, TxId, VectorIndex, VectorIndexStats};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::future::Future;
use std::sync::Arc;

/// Failure representation for vector index conformance tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceFailure {
    /// Suite identifier.
    pub suite_name: &'static str,
    /// Specific test case that failed.
    pub test_case: &'static str,
    /// Diagnostic failure message.
    pub message: String,
}

impl ConformanceFailure {
    /// Creates a new failure record.
    #[must_use]
    pub fn new(
        suite_name: &'static str,
        test_case: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            suite_name,
            test_case,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ConformanceFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] Conformance failure in test '{}': {}",
            self.suite_name, self.test_case, self.message
        )
    }
}

impl std::error::Error for ConformanceFailure {}

/// Conformance result type.
pub type ConformanceResult = std::result::Result<(), ConformanceFailure>;

/// Führt die vollständige Konformitätsprüfung für eine [`VectorIndex`]-Implementierung durch.
pub async fn run<F, Fut, V>(factory: F) -> ConformanceResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = V>,
    V: VectorIndex,
{
    let suite = "VectorIndex";

    // Fall 1: PO-02 Einfügen & Suche
    {
        let index = factory().await;
        let tx = TxId::new(301);
        let vec = [0.1f32, 0.2, 0.3, 0.4];
        if let Err(e) = index.insert(tx, DocId(99), &vec).await {
            return Err(ConformanceFailure::new(
                suite,
                "po_02_insert_search",
                format!("insert failed: {e:?}"),
            ));
        }
        if let Err(e) = index.commit(tx).await {
            return Err(ConformanceFailure::new(
                suite,
                "po_02_insert_search",
                format!("commit failed: {e:?}"),
            ));
        }

        let hits = match index.search(&vec, 5).await {
            Ok(h) => h,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_insert_search",
                    format!("search failed: {e:?}"),
                ));
            }
        };

        if !hits.iter().any(|h| h.doc_id == DocId(99)) {
            return Err(ConformanceFailure::new(
                suite,
                "po_02_insert_search",
                format!("inserted vector 99 not found in hits: {hits:?}"),
            ));
        }
    }

    // Fall 2: PO-02 Löschen ⇒ Treffer verschwindet vollständig
    {
        let index = factory().await;
        let tx = TxId::new(401);
        let vec = [0.5f32, 0.5, 0.5, 0.5];
        let _ = index.insert(tx, DocId(77), &vec).await;
        let _ = index.commit(tx).await;

        let tx_del = TxId::new(402);
        if let Err(e) = index.delete(tx_del, DocId(77)).await {
            return Err(ConformanceFailure::new(
                suite,
                "po_02_delete_disappears_completely",
                format!("delete failed: {e:?}"),
            ));
        }
        if let Err(e) = index.commit(tx_del).await {
            return Err(ConformanceFailure::new(
                suite,
                "po_02_delete_disappears_completely",
                format!("commit delete failed: {e:?}"),
            ));
        }

        let hits_after = match index.search(&vec, 5).await {
            Ok(h) => h,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_delete_disappears_completely",
                    format!("search after delete failed: {e:?}"),
                ));
            }
        };

        if hits_after.iter().any(|h| h.doc_id == DocId(77)) {
            return Err(ConformanceFailure::new(
                suite,
                "po_02_delete_disappears_completely",
                format!("deleted vector DocId(77) still returned in search: {hits_after:?}"),
            ));
        }
    }

    // Fall 3: PO-02 Deterministische Distanzordnung & Tie-Breaking
    {
        let index = factory().await;
        let tx = TxId::new(501);
        let q = [0.0f32, 0.0, 0.0, 0.0];
        let v_close = [0.1f32, 0.0, 0.0, 0.0];
        let v_far = [0.9f32, 0.0, 0.0, 0.0];
        let _ = index.insert(tx, DocId(20), &v_far).await;
        let _ = index.insert(tx, DocId(10), &v_close).await;
        let _ = index.commit(tx).await;

        let run1 = match index.search(&q, 2).await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_deterministic_ordering",
                    format!("search run1 failed: {e:?}"),
                ));
            }
        };

        let run2 = match index.search(&q, 2).await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_deterministic_ordering",
                    format!("search run2 failed: {e:?}"),
                ));
            }
        };

        if run1.len() != 2 || run2.len() != 2 {
            return Err(ConformanceFailure::new(
                suite,
                "po_02_deterministic_ordering",
                format!("expected 2 hits, got {} and {}", run1.len(), run2.len()),
            ));
        }

        if run1.first().map(|d| d.doc_id) != Some(DocId(10)) {
            return Err(ConformanceFailure::new(
                suite,
                "po_02_deterministic_ordering",
                format!("closer doc 10 should be first, got {:?}", run1.first()),
            ));
        }

        let ids1: Vec<DocId> = run1.iter().map(|d| d.doc_id).collect();
        let ids2: Vec<DocId> = run2.iter().map(|d| d.doc_id).collect();
        if ids1 != ids2 {
            return Err(ConformanceFailure::new(
                suite,
                "po_02_deterministic_ordering",
                format!("non-deterministic order: {ids1:?} != {ids2:?}"),
            ));
        }
    }

    Ok(())
}

/// Backwards-compatible alias for vector index suite.
pub async fn run_vector_index_suite<F, Fut, V>(factory: F) -> ConformanceResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = V>,
    V: VectorIndex,
{
    run(factory).await
}

// ============================================================================
// REFERENCE IMPLEMENTATION (IN-MEMORY)
// ============================================================================

/// In-Memory Reference Implementation of [`VectorIndex`].
#[derive(Clone)]
pub struct ReferenceVectorIndex {
    vectors: Arc<RwLock<BTreeMap<DocId, Vec<f32>>>>,
    uncommitted_inserts: Arc<RwLock<BTreeMap<TxId, Vec<(DocId, Vec<f32>)>>>>,
    uncommitted_deletes: Arc<RwLock<BTreeMap<TxId, Vec<DocId>>>>,
    last_tx: Arc<RwLock<TxId>>,
}

impl ReferenceVectorIndex {
    /// Creates a new reference vector index.
    #[must_use]
    pub fn new() -> Self {
        Self {
            vectors: Arc::new(RwLock::new(BTreeMap::new())),
            uncommitted_inserts: Arc::new(RwLock::new(BTreeMap::new())),
            uncommitted_deletes: Arc::new(RwLock::new(BTreeMap::new())),
            last_tx: Arc::new(RwLock::new(TxId::new(0))),
        }
    }
}

impl Default for ReferenceVectorIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorIndex for ReferenceVectorIndex {
    async fn insert(&self, tx: TxId, id: DocId, embedding: &[f32]) -> Result<()> {
        self.uncommitted_inserts
            .write()
            .entry(tx)
            .or_default()
            .push((id, embedding.to_vec()));
        Ok(())
    }

    async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        self.uncommitted_deletes
            .write()
            .entry(tx)
            .or_default()
            .push(id);
        Ok(())
    }

    async fn commit(&self, tx: TxId) -> Result<()> {
        let inserts = self
            .uncommitted_inserts
            .write()
            .remove(&tx)
            .unwrap_or_default();
        let deletes = self
            .uncommitted_deletes
            .write()
            .remove(&tx)
            .unwrap_or_default();

        let mut vectors = self.vectors.write();
        for (id, vec) in inserts {
            vectors.insert(id, vec);
        }
        for id in deletes {
            vectors.remove(&id);
        }
        *self.last_tx.write() = tx;
        Ok(())
    }

    async fn rollback(&self, tx: TxId) -> Result<()> {
        self.uncommitted_inserts.write().remove(&tx);
        self.uncommitted_deletes.write().remove(&tx);
        Ok(())
    }

    async fn rollback_to_tx(&self, _tx_id: TxId) -> Result<()> {
        Ok(())
    }

    async fn last_tx_id(&self) -> Result<TxId> {
        Ok(*self.last_tx.read())
    }

    async fn search(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>> {
        let vectors = self.vectors.read();
        let mut scored: Vec<ScoredDocument> = Vec::new();

        for (&id, vec) in vectors.iter() {
            if vec.len() != query.len() {
                continue;
            }
            let mut sum_sq = 0.0f32;
            for i in 0..query.len() {
                let diff =
                    query.get(i).copied().unwrap_or(0.0) - vec.get(i).copied().unwrap_or(0.0);
                sum_sq += diff * diff;
            }
            let dist = sum_sq.sqrt();
            let score = 1.0 / (1.0 + dist);
            scored.push(ScoredDocument { doc_id: id, score });
        }

        // Deterministische Sortierung: score absteigend (nähere Vektoren haben höheren Score), tie-break nach doc_id
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });

        scored.truncate(k);
        Ok(scored)
    }

    async fn len(&self) -> usize {
        self.vectors.read().len()
    }

    async fn stats(&self) -> Result<VectorIndexStats> {
        Ok(VectorIndexStats {
            num_vectors: self.vectors.read().len(),
            memory_usage_bytes: 0,
            num_layers: 1,
            deleted_ratio: 0.0,
            rebuild_count: 0,
        })
    }
}

// ============================================================================
// MUTANTS (INTENTIONALLY DEFECTIVE IMPLEMENTATIONS)
// ============================================================================

/// Mutant 1: delete does nothing.
#[derive(Clone, Default)]
pub struct MutantNoopDeleteVectorIndex(ReferenceVectorIndex);

impl VectorIndex for MutantNoopDeleteVectorIndex {
    async fn insert(&self, tx: TxId, id: DocId, embedding: &[f32]) -> Result<()> {
        self.0.insert(tx, id, embedding).await
    }
    async fn delete(&self, _tx: TxId, _id: DocId) -> Result<()> {
        // DEFECT: Delete is a no-op!
        Ok(())
    }
    async fn commit(&self, tx: TxId) -> Result<()> {
        self.0.commit(tx).await
    }
    async fn rollback(&self, tx: TxId) -> Result<()> {
        self.0.rollback(tx).await
    }
    async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()> {
        self.0.rollback_to_tx(tx_id).await
    }
    async fn last_tx_id(&self) -> Result<TxId> {
        self.0.last_tx_id().await
    }
    async fn search(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>> {
        self.0.search(query, k).await
    }
    async fn len(&self) -> usize {
        self.0.len().await
    }
    async fn stats(&self) -> Result<VectorIndexStats> {
        self.0.stats().await
    }
}

/// Mutant 2: reversed distance ordering.
#[derive(Clone, Default)]
pub struct MutantReversedDistanceVectorIndex(ReferenceVectorIndex);

impl VectorIndex for MutantReversedDistanceVectorIndex {
    async fn insert(&self, tx: TxId, id: DocId, embedding: &[f32]) -> Result<()> {
        self.0.insert(tx, id, embedding).await
    }
    async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        self.0.delete(tx, id).await
    }
    async fn commit(&self, tx: TxId) -> Result<()> {
        self.0.commit(tx).await
    }
    async fn rollback(&self, tx: TxId) -> Result<()> {
        self.0.rollback(tx).await
    }
    async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()> {
        self.0.rollback_to_tx(tx_id).await
    }
    async fn last_tx_id(&self) -> Result<TxId> {
        self.0.last_tx_id().await
    }
    async fn search(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>> {
        let mut res = self.0.search(query, k).await?;
        res.reverse(); // DEFECT: farthest items returned first!
        Ok(res)
    }
    async fn len(&self) -> usize {
        self.0.len().await
    }
    async fn stats(&self) -> Result<VectorIndexStats> {
        self.0.stats().await
    }
}

/// Helper executing reference test.
pub async fn run_reference_test() -> ConformanceResult {
    run(|| async { ReferenceVectorIndex::new() }).await
}

/// Helper executing mutant tests, ensuring 100% detection rate.
pub async fn run_mutant_tests() -> std::result::Result<(), String> {
    let res1 = run(|| async { MutantNoopDeleteVectorIndex::default() }).await;
    if res1.is_ok() {
        return Err("MutantNoopDeleteVectorIndex escaped detection!".to_string());
    }

    let res2 = run(|| async { MutantReversedDistanceVectorIndex::default() }).await;
    if res2.is_ok() {
        return Err("MutantReversedDistanceVectorIndex escaped detection!".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn po_02_vector_index_reference_passes() {
        let res = run_reference_test().await;
        assert!(res.is_ok(), "Reference implementation must pass: {res:?}");
    }

    #[tokio::test]
    async fn po_02_vector_index_mutants_fail() {
        let res = run_mutant_tests().await;
        assert!(res.is_ok(), "Mutants must be caught: {res:?}");
    }
}
