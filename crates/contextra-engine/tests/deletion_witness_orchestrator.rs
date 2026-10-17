//! Integration tests for deletion witness measurement and lock retention.

#![forbid(unsafe_code)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use contextra_engine::deletion::{
    forget_document, ErasureFootprint, DpiDeletionRuntime, KeyFamilyRegistry, FprSweepSource, FprSweepReport, ForgetOptions,
};
use contextra_ports::{ContextChunk, DocId, ScoredDocument, StorageEngine, TxId, VectorIndex, VectorIndexStats, VectorResidue};
use contextra_store::{LsmConfig as LsmStorageConfig, LsmStorage};
use contextra_types::{ContextraError, Result};
use tempfile::TempDir;

struct DummySweepSource;

impl FprSweepSource for DummySweepSource {
    fn for_each_key(&self, _f: &mut dyn for<'a> FnMut(&'a [u8]) -> bool) -> Result<()> {
        Ok(())
    }
}

struct TestVectorIndex {
    pub residue_node_present: std::sync::atomic::AtomicBool,
    pub residue_incoming: std::sync::atomic::AtomicU32,
    pub residue_outgoing: std::sync::atomic::AtomicU32,
}

impl TestVectorIndex {
    fn new(present: bool, incoming: u32, outgoing: u32) -> Self {
        Self {
            residue_node_present: std::sync::atomic::AtomicBool::new(present),
            residue_incoming: std::sync::atomic::AtomicU32::new(incoming),
            residue_outgoing: std::sync::atomic::AtomicU32::new(outgoing),
        }
    }
}

impl VectorIndex for TestVectorIndex {
    async fn insert(&self, _tx: TxId, _id: DocId, _embedding: &[f32]) -> Result<()> {
        Ok(())
    }

    async fn search(&self, _query: &[f32], _k: usize) -> Result<Vec<ScoredDocument>> {
        Ok(vec![])
    }

    async fn residue(&self, _id: DocId) -> Result<VectorResidue> {
        Ok(VectorResidue {
            node_present: self.residue_node_present.load(Ordering::SeqCst),
            incoming_edges: self.residue_incoming.load(Ordering::SeqCst),
            outgoing_edges: self.residue_outgoing.load(Ordering::SeqCst),
            pinned: false,
        })
    }

    async fn delete(&self, _tx: TxId, _id: DocId) -> Result<()> {
        Ok(())
    }

    async fn commit(&self, _tx: TxId) -> Result<()> {
        Ok(())
    }

    async fn rollback(&self, _tx: TxId) -> Result<()> {
        Ok(())
    }

    async fn rollback_to_tx(&self, _tx_id: TxId) -> Result<()> {
        Ok(())
    }

    async fn last_tx_id(&self) -> Result<TxId> {
        Ok(TxId(0))
    }

    async fn len(&self) -> usize {
        0
    }

    async fn stats(&self) -> Result<VectorIndexStats> {
        Ok(VectorIndexStats::default())
    }
}

#[tokio::test]
async fn hnsw_witness_measured_not_asserted() {
    let temp_dir = TempDir::new().unwrap();
    let db_dir = temp_dir.path().join("db");
    std::fs::create_dir_all(&db_dir).unwrap();

    let storage = LsmStorage::new(LsmStorageConfig { path: db_dir.clone(), ..Default::default() }).await.unwrap();
    let runtime = DpiDeletionRuntime::open(&db_dir, None, [1u8; 16]).unwrap();
    let registry = KeyFamilyRegistry::new();
    let sweep_source = DummySweepSource;

    let doc_id = DocId::new(42);
    let doc_str = "1/42";
    let footprint = ErasureFootprint::fuer_document(contextra_types::TenantId::SYSTEM, "test", doc_id);
    let opts = ForgetOptions::default();

    // If node is present or has incoming/outgoing edges (e.g., reinserted node), marked_zeroed will be false
    let vector_index = TestVectorIndex::new(true, 2, 1);

    let res = forget_document(
        &storage,
        &vector_index,
        &runtime,
        doc_str,
        TxId(1),
        &opts,
        &sweep_source,
        &footprint,
        &registry,
    )
    .await;

    // Witness issuer or proof issuer will fail because marked_zeroed is false while node_present is true
    assert!(res.is_err(), "Expected forget_document to fail when vector residue has present node or non-zero edges");
}

#[tokio::test]
async fn witness_error_is_not_swallowed() {
    let temp_dir = TempDir::new().unwrap();
    let db_dir = temp_dir.path().join("db");
    std::fs::create_dir_all(&db_dir).unwrap();

    let storage = LsmStorage::new(LsmStorageConfig { path: db_dir.clone(), ..Default::default() }).await.unwrap();
    let runtime = DpiDeletionRuntime::open(&db_dir, None, [1u8; 16]).unwrap();
    let registry = KeyFamilyRegistry::new();
    let sweep_source = DummySweepSource;

    let doc_id = DocId::new(100);
    let doc_str = "1/100";
    let footprint = ErasureFootprint::fuer_document(contextra_types::TenantId::SYSTEM, "test", doc_id);
    let opts = ForgetOptions::default();

    // Vector index returning an error from residue
    struct FailingVectorIndex;
    impl VectorIndex for FailingVectorIndex {
        async fn insert(&self, _tx: TxId, _id: DocId, _embedding: &[f32]) -> Result<()> { Ok(()) }
        async fn search(&self, _query: &[f32], _k: usize) -> Result<Vec<ScoredDocument>> { Ok(vec![]) }
        async fn residue(&self, _id: DocId) -> Result<VectorResidue> {
            Err(ContextraError::Storage("Simulated vector scan failure".to_string()))
        }
        async fn delete(&self, _tx: TxId, _id: DocId) -> Result<()> { Ok(()) }
        async fn commit(&self, _tx: TxId) -> Result<()> { Ok(()) }
        async fn rollback(&self, _tx: TxId) -> Result<()> { Ok(()) }
        async fn rollback_to_tx(&self, _tx_id: TxId) -> Result<()> { Ok(()) }
        async fn last_tx_id(&self) -> Result<TxId> { Ok(TxId(0)) }
        async fn len(&self) -> usize { 0 }
        async fn stats(&self) -> Result<VectorIndexStats> { Ok(VectorIndexStats::default()) }
    }

    let vector_index = FailingVectorIndex;

    let res = forget_document(
        &storage,
        &vector_index,
        &runtime,
        doc_str,
        TxId(1),
        &opts,
        &sweep_source,
        &footprint,
        &registry,
    )
    .await;

    assert!(res.is_err(), "Store / vector scan error must not be swallowed");
    match res {
        Err(ContextraError::Storage(msg)) => {
            assert!(msg.contains("Simulated vector scan failure"));
        }
        Err(e) => {
            // Other error variants are also acceptable as long as it returned Err
            tracing::info!("Returned error: {:?}", e);
        }
        Ok(_) => panic!("Expected Err when witness scan fails"),
    }
}

#[tokio::test]
async fn no_self_attested_fallback() {
    let temp_dir = TempDir::new().unwrap();
    let db_dir = temp_dir.path().join("db");
    std::fs::create_dir_all(&db_dir).unwrap();

    let storage = LsmStorage::new(LsmStorageConfig { path: db_dir.clone(), ..Default::default() }).await.unwrap();
    let runtime = DpiDeletionRuntime::open(&db_dir, None, [1u8; 16]).unwrap();
    let registry = KeyFamilyRegistry::new();
    let sweep_source = DummySweepSource;

    let doc_id = DocId::new(200);
    let doc_str = "1/200";
    let footprint = ErasureFootprint::fuer_document(contextra_types::TenantId::SYSTEM, "test", doc_id);
    let opts = ForgetOptions::default();

    // With a clean empty vector index (residue returns node_present = false, 0 edges),
    // forget_document generates measured witness and succeeds without fallback
    let vector_index = TestVectorIndex::new(false, 0, 0);

    let res = forget_document(
        &storage,
        &vector_index,
        &runtime,
        doc_str,
        TxId(1),
        &opts,
        &sweep_source,
        &footprint,
        &registry,
    )
    .await;

    assert!(res.is_ok(), "Measured clean witness should succeed: {:?}", res.err());
}

#[tokio::test]
async fn delete_lock_held_through_commit() -> Result<()> {
    use contextra_engine::{Contextra, ContextraConfig};
    use serde_json::json;

    let temp_dir = TempDir::new().unwrap();
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(temp_dir.path(), config).await?;
    let col = Arc::new(db.collection("default").await?);
    let emb = vec![1.0, 0.0, 0.0, 0.0];

    col.insert("doc_lock_test", &emb, Some(json!({ "content": "Lock retention test" }))).await?;

    let col1 = Arc::clone(&col);
    let col2 = Arc::clone(&col);

    let handle1 = tokio::spawn(async move {
        col1.delete("doc_lock_test").await
    });

    let handle2 = tokio::spawn(async move {
        col2.delete("doc_lock_test").await
    });

    let res1 = handle1.await.unwrap();
    let res2 = handle2.await.unwrap();

    // At least one delete succeeds, and the concurrent delete finishes cleanly without panic or corrupt transaction state
    assert!(res1.is_ok() || res2.is_ok(), "At least one delete call must succeed");

    Ok(())
}
