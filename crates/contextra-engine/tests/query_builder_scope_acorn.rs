use std::collections::BTreeSet;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::TempDir;

use contextra_engine::collection::query_builder::ScopeConstraint;
use contextra_engine::{Collection, DistanceMetric, Language};
use contextra_graph::CsrGraph;
use contextra_ports::{Result as PortResult, ScoredDocument, TxId, VectorIndex, VectorIndexStats};
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{ContextraError, DocId};
use contextra_vector::acorn::{AcornError, FilteredIndex};
use contextra_vector::{HnswConfig, HnswIndex};
use serde_json::json;

/// Local test-double for a vector index that implements `FilteredIndex`.
#[derive(Clone)]
struct MockFilteredIndex {
    hnsw: Arc<HnswIndex>,
}

impl MockFilteredIndex {
    fn new(dimension: usize) -> Self {
        let config = HnswConfig {
            dimension,
            distance_metric: DistanceMetric::Cosine,
            ..Default::default()
        };
        Self {
            hnsw: Arc::new(HnswIndex::try_new(config).unwrap()),
        }
    }
}

impl VectorIndex for MockFilteredIndex {
    async fn insert(&self, tx: TxId, id: DocId, embedding: &[f32]) -> PortResult<()> {
        self.hnsw.insert(tx, id, embedding).await
    }

    async fn search(&self, query: &[f32], k: usize) -> PortResult<Vec<ScoredDocument>> {
        self.hnsw.search(query, k).await
    }

    async fn search_at(
        &self,
        query: &[f32],
        k: usize,
        seq_no: u64,
    ) -> PortResult<Vec<ScoredDocument>> {
        self.hnsw.search_at(query, k, seq_no).await
    }

    async fn delete(&self, tx: TxId, id: DocId) -> PortResult<()> {
        self.hnsw.delete(tx, id).await
    }

    async fn commit(&self, tx: TxId) -> PortResult<()> {
        self.hnsw.commit(tx).await
    }

    async fn rollback(&self, tx: TxId) -> PortResult<()> {
        self.hnsw.rollback(tx).await
    }

    async fn rollback_to_tx(&self, tx_id: TxId) -> PortResult<()> {
        self.hnsw.rollback_to_tx(tx_id).await
    }

    async fn last_tx_id(&self) -> PortResult<TxId> {
        self.hnsw.last_tx_id().await
    }

    async fn len(&self) -> usize {
        self.hnsw.len().await
    }

    async fn stats(&self) -> PortResult<VectorIndexStats> {
        self.hnsw.stats().await
    }
}

impl FilteredIndex for MockFilteredIndex {
    type Predicate = dyn Fn(DocId) -> bool;

    fn search_knn_acorn(
        &self,
        query: &[f32],
        k: usize,
        predicate: &Self::Predicate,
        _gamma: u32,
    ) -> Result<Vec<(DocId, f32)>, AcornError> {
        let hnsw = self.hnsw.clone();
        let query_vec = query.to_vec();
        let search_res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async move { hnsw.search(&query_vec, 100).await })
        })
        .map_err(|e| AcornError::Internal(e.to_string()))?;

        let mut matched = Vec::new();
        for doc in search_res {
            if predicate(doc.doc_id) {
                matched.push((doc.doc_id, doc.score));
            }
            if matched.len() >= k {
                break;
            }
        }
        Ok(matched)
    }
}

async fn create_mock_collection(name: &str) -> (Collection<LsmStorage, MockFilteredIndex>, TempDir) {
    let dir = TempDir::new().unwrap();
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap());
    let index = Arc::new(MockFilteredIndex::new(4));
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        name.to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        Language::English,
    );
    (col, dir)
}

async fn create_standard_collection(name: &str) -> (Collection<LsmStorage, HnswIndex>, TempDir) {
    let dir = TempDir::new().unwrap();
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap());
    let hnsw_config = HnswConfig {
        dimension: 4,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    let index = Arc::new(HnswIndex::try_new(hnsw_config).unwrap());
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        name.to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        Language::English,
    );
    (col, dir)
}

#[tokio::test(flavor = "multi_thread")]
async fn test_acorn_hard_boundary_scoping_returns_only_allowed_doc_ids() {
    let (col, _dir) = create_mock_collection("test_acorn_scoping").await;
    col.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], Some(json!({"tenant": "A"})))
        .await
        .unwrap();
    col.insert("doc-2", &[0.95, 0.05, 0.0, 0.0], Some(json!({"tenant": "B"})))
        .await
        .unwrap();
    col.insert("doc-3", &[0.90, 0.10, 0.0, 0.0], Some(json!({"tenant": "A"})))
        .await
        .unwrap();

    let doc1_id = DocId::from_key("doc-1").unwrap();
    let doc3_id = DocId::from_key("doc-3").unwrap();

    let mut allowed = BTreeSet::new();
    allowed.insert(doc1_id);
    allowed.insert(doc3_id);

    let constraint = ScopeConstraint::from_allowed_ids(allowed.clone());

    let results = col
        .query()
        .embedding([1.0, 0.0, 0.0, 0.0])
        .scope(constraint)
        .k(10)
        .execute_with_scope()
        .await
        .unwrap();

    assert!(!results.is_empty(), "Results must not be empty");
    for res in &results {
        let doc_id = DocId::from_key(&res.id).unwrap();
        assert!(
            allowed.contains(&doc_id),
            "Returned doc {} ({:?}) was not in allowed_doc_ids constraint (Zero-Cross-Contamination violation)",
            res.id,
            doc_id
        );
    }
}

#[tokio::test]
async fn test_query_builder_without_scope_preserves_standard_execution() {
    let (col, _dir) = create_standard_collection("test_no_scope_regression").await;
    col.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], Some(json!({"v": 1})))
        .await
        .unwrap();
    col.insert("doc-2", &[0.0, 1.0, 0.0, 0.0], Some(json!({"v": 2})))
        .await
        .unwrap();

    let results = col
        .query()
        .embedding([1.0, 0.0, 0.0, 0.0])
        .filter_fn(|d| d == DocId::from_key("doc-1").unwrap())
        .k(10)
        .execute()
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "doc-1");
}

#[tokio::test]
async fn test_scope_with_unsupported_index_returns_capability_error() {
    let (col, _dir) = create_standard_collection("test_unsupported_scope").await;
    col.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
        .await
        .unwrap();

    let mut allowed = BTreeSet::new();
    allowed.insert(DocId::from_key("doc-1").unwrap());
    let constraint = ScopeConstraint::from_allowed_ids(allowed);

    let err = col
        .query()
        .embedding([1.0, 0.0, 0.0, 0.0])
        .scope(constraint)
        .k(10)
        .execute()
        .await
        .unwrap_err();

    match err {
        ContextraError::CapabilityUnsupported { capability, reason } => {
            assert_eq!(capability, "acorn_hard_boundary");
            assert!(
                reason.contains("FilteredIndex"),
                "Error reason must mention FilteredIndex requirement: {reason}"
            );
        }
        other => panic!("Expected CapabilityUnsupported error, got: {other:?}"),
    }
}
