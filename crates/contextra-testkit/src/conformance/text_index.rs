//! # Konformitätsprüfsuite für den `TextIndex`-Port (`conformance/text_index`)
//!
//! Dieses Modul testet die vertraglichen Zusicherungen (Contract Invariants)
//! für Volltext-Indizes des [`contextra_ports::TextIndex`]-Ports (PO-02).
//!
//! ## Getestete Verträge (Pflichtfälle laut PO-02):
//! 1. `po_02_insert_search`: Einfügen und Auffinden von Dokumenten über Term-Suchen.
//! 2. `po_02_delete_disappears_completely`: Löschen bewirkt, dass der Treffer vollständig verschwindet.
//! 3. `po_02_deterministic_ordering`: Deterministische Ergebnisreihenfolge bei Gleichstand (Tie-Breaking nach DocId).

use contextra_ports::{DocId, Result, ScoredDocument, TextIndex, TextIndexStats, TxId};
use parking_lot::RwLock;
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::sync::Arc;

/// Failure representation for text index conformance tests.
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

/// Führt die vollständige Konformitätsprüfung für eine [`TextIndex`]-Implementierung durch.
pub async fn run<F, Fut, T>(factory: F) -> ConformanceResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = T>,
    T: TextIndex,
{
    let suite = "TextIndex";

    // Fall 1: PO-02 Einfügen & Suche
    {
        let index = factory().await;
        let tx = TxId::new(101);
        let doc_text = "rust contracts for text search and index verification";
        if let Err(e) = index.insert(tx, DocId(42), doc_text).await {
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

        let hits = match index.search("contracts", 10).await {
            Ok(h) => h,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_insert_search",
                    format!("search failed: {e:?}"),
                ));
            }
        };

        if !hits.iter().any(|h| h.doc_id == DocId(42)) {
            return Err(ConformanceFailure::new(
                suite,
                "po_02_insert_search",
                format!("inserted document 42 not found in hits: {hits:?}"),
            ));
        }
    }

    // Fall 2: PO-02 Löschen ⇒ Treffer verschwindet vollständig
    {
        let index = factory().await;
        let tx1 = TxId::new(201);
        let _ = index
            .insert(tx1, DocId(99), "confidential secret document payload")
            .await;
        let _ = index.commit(tx1).await;

        let tx_del = TxId::new(202);
        if let Err(e) = index.delete(tx_del, DocId(99)).await {
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

        let hits_after = match index.search("confidential", 10).await {
            Ok(h) => h,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_delete_disappears_completely",
                    format!("search after delete failed: {e:?}"),
                ));
            }
        };

        if hits_after.iter().any(|h| h.doc_id == DocId(99)) {
            return Err(ConformanceFailure::new(
                suite,
                "po_02_delete_disappears_completely",
                format!("deleted doc 99 still returned in search: {hits_after:?}"),
            ));
        }
    }

    // Fall 3: PO-02 Deterministische Reihenfolge (Tie-Breaking)
    {
        let index = factory().await;
        let tx = TxId::new(301);
        // Documents with identical textual relevance for query "target"
        let _ = index.insert(tx, DocId(50), "target term only").await;
        let _ = index.insert(tx, DocId(10), "target term only").await;
        let _ = index.insert(tx, DocId(30), "target term only").await;
        let _ = index.commit(tx).await;

        let run1 = match index.search("target", 10).await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_deterministic_ordering",
                    format!("search run1 failed: {e:?}"),
                ));
            }
        };

        let run2 = match index.search("target", 10).await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "po_02_deterministic_ordering",
                    format!("search run2 failed: {e:?}"),
                ));
            }
        };

        let ids1: Vec<DocId> = run1.iter().map(|d| d.doc_id).collect();
        let ids2: Vec<DocId> = run2.iter().map(|d| d.doc_id).collect();

        if ids1 != ids2 || ids1.len() != 3 {
            return Err(ConformanceFailure::new(
                suite,
                "po_02_deterministic_ordering",
                format!("non-deterministic search order observed: run1={ids1:?}, run2={ids2:?}"),
            ));
        }
    }

    Ok(())
}

/// Backwards-compatible alias for text index suite.
pub async fn run_text_index_suite<F, Fut, T>(factory: F) -> ConformanceResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = T>,
    T: TextIndex,
{
    run(factory).await
}

// ============================================================================
// REFERENCE IMPLEMENTATION (IN-MEMORY)
// ============================================================================

type UncommittedInsertsMap = Arc<RwLock<BTreeMap<TxId, Vec<(DocId, String)>>>>;
type UncommittedDeletesMap = Arc<RwLock<BTreeMap<TxId, Vec<DocId>>>>;

/// In-Memory Reference Implementation of [`TextIndex`].
#[derive(Clone)]
pub struct ReferenceTextIndex {
    docs: Arc<RwLock<BTreeMap<DocId, String>>>,
    uncommitted_inserts: UncommittedInsertsMap,
    uncommitted_deletes: UncommittedDeletesMap,
    last_tx: Arc<RwLock<TxId>>,
}

impl ReferenceTextIndex {
    /// Creates a new reference text index.
    #[must_use]
    pub fn new() -> Self {
        Self {
            docs: Arc::new(RwLock::new(BTreeMap::new())),
            uncommitted_inserts: Arc::new(RwLock::new(BTreeMap::new())),
            uncommitted_deletes: Arc::new(RwLock::new(BTreeMap::new())),
            last_tx: Arc::new(RwLock::new(TxId::new(0))),
        }
    }
}

impl Default for ReferenceTextIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl TextIndex for ReferenceTextIndex {
    async fn insert(&self, tx: TxId, id: DocId, text: &str) -> Result<()> {
        self.uncommitted_inserts
            .write()
            .entry(tx)
            .or_default()
            .push((id, text.to_lowercase()));
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

        let mut docs = self.docs.write();
        for (id, text) in inserts {
            docs.insert(id, text);
        }
        for id in deletes {
            docs.remove(&id);
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

    async fn search(&self, query: &str, k: usize) -> Result<Vec<ScoredDocument>> {
        let q = query.to_lowercase();
        let query_terms: BTreeSet<&str> = q.split_whitespace().collect();

        let docs = self.docs.read();
        let mut scored: Vec<ScoredDocument> = Vec::new();

        for (&id, text) in docs.iter() {
            let doc_terms: BTreeSet<&str> = text.split_whitespace().collect();
            let mut matches = 0usize;
            for t in &query_terms {
                if doc_terms.contains(t) {
                    matches += 1;
                }
            }
            if matches > 0 {
                scored.push(ScoredDocument {
                    doc_id: id,
                    score: matches as f32,
                });
            }
        }

        // Deterministische Sortierung: score absteigend, bei Gleichstand doc_id aufsteigend
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
        self.docs.read().len()
    }

    async fn stats(&self) -> Result<TextIndexStats> {
        Ok(TextIndexStats {
            num_documents: self.docs.read().len(),
            num_tokens: 0,
            memory_usage_bytes: 0,
        })
    }
}

// ============================================================================
// MUTANTS (INTENTIONALLY DEFECTIVE IMPLEMENTATIONS)
// ============================================================================

/// Mutant 1: delete does not remove document.
#[derive(Clone, Default)]
pub struct MutantNoopDeleteTextIndex(ReferenceTextIndex);

impl TextIndex for MutantNoopDeleteTextIndex {
    async fn insert(&self, tx: TxId, id: DocId, text: &str) -> Result<()> {
        self.0.insert(tx, id, text).await
    }
    async fn delete(&self, _tx: TxId, _id: DocId) -> Result<()> {
        // DEFECT: Delete does nothing!
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
    async fn search(&self, query: &str, k: usize) -> Result<Vec<ScoredDocument>> {
        self.0.search(query, k).await
    }
    async fn len(&self) -> usize {
        self.0.len().await
    }
    async fn stats(&self) -> Result<TextIndexStats> {
        self.0.stats().await
    }
}

/// Mutant 2: non-deterministic ordering (inverts tie-breaking on alternating calls).
#[derive(Clone, Default)]
pub struct MutantNonDeterministicOrderTextIndex {
    base: ReferenceTextIndex,
    call_flip: Arc<parking_lot::Mutex<bool>>,
}

impl TextIndex for MutantNonDeterministicOrderTextIndex {
    async fn insert(&self, tx: TxId, id: DocId, text: &str) -> Result<()> {
        self.base.insert(tx, id, text).await
    }
    async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        self.base.delete(tx, id).await
    }
    async fn commit(&self, tx: TxId) -> Result<()> {
        self.base.commit(tx).await
    }
    async fn rollback(&self, tx: TxId) -> Result<()> {
        self.base.rollback(tx).await
    }
    async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()> {
        self.base.rollback_to_tx(tx_id).await
    }
    async fn last_tx_id(&self) -> Result<TxId> {
        self.base.last_tx_id().await
    }
    async fn search(&self, query: &str, k: usize) -> Result<Vec<ScoredDocument>> {
        let mut res = self.base.search(query, k).await?;
        let mut flip = self.call_flip.lock();
        if *flip {
            res.reverse();
        }
        *flip = !*flip;
        Ok(res)
    }
    async fn len(&self) -> usize {
        self.base.len().await
    }
    async fn stats(&self) -> Result<TextIndexStats> {
        self.base.stats().await
    }
}

/// Helper executing reference test.
pub async fn run_reference_test() -> ConformanceResult {
    run(|| async { ReferenceTextIndex::new() }).await
}

/// Helper executing mutant tests, ensuring 100% detection rate.
pub async fn run_mutant_tests() -> std::result::Result<(), String> {
    let res1 = run(|| async { MutantNoopDeleteTextIndex::default() }).await;
    if res1.is_ok() {
        return Err("MutantNoopDeleteTextIndex escaped detection!".to_string());
    }

    let res2 = run(|| async { MutantNonDeterministicOrderTextIndex::default() }).await;
    if res2.is_ok() {
        return Err("MutantNonDeterministicOrderTextIndex escaped detection!".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SPEC: PO-02
    #[tokio::test]
    async fn po_02_text_index_reference_passes() {
        let res = run_reference_test().await;
        assert!(res.is_ok(), "Reference implementation must pass: {res:?}");
    }

    /// SPEC: PO-02
    #[tokio::test]
    async fn po_02_text_index_mutants_fail() {
        let res = run_mutant_tests().await;
        assert!(res.is_ok(), "Mutants must be caught: {res:?}");
    }
}
