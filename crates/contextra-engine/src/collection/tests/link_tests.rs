use super::fixtures::*;

#[tokio::test]
async fn test_relate_success_visible_in_storage_and_graph() {
    use contextra_core::EntityId;
    use contextra_graph::csr::CsrGraph;
    use contextra_store::{LsmConfig, LsmStorage};
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap(); // unwrap
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(), // unwrap
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap
    );
    let graph = Arc::new(CsrGraph::new());
    let col = Collection::new(
        "default".to_string(),
        storage.clone(),
        index,
        graph.clone(),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.relate("doc1", "doc2", "references").await.unwrap(); // unwrap

    // 1. Storage check
    let rels = col.scan_prefix("__rel:", None).await.unwrap(); // unwrap
    assert_eq!(rels.len(), 1);
    assert!(rels[0].0.contains("doc1:references:doc2"));

    // 2. Graph check
    let id1 = EntityId::from_key("doc1").unwrap(); // unwrap
    let id2 = EntityId::from_key("doc2").unwrap(); // unwrap
    let neighbors = graph.neighbors(id1).await.unwrap(); // unwrap
    assert!(neighbors.contains(&id2));
}


#[tokio::test]
async fn test_relate_rollback_semantics_on_storage_commit_failure() {
    use contextra_core::{BoxFuture, Result, StorageEngine, StorageStats, TxId};
    use contextra_graph::csr::CsrGraph;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    struct FailOnStorageCommit;

    impl StorageEngine for FailOnStorageCommit {
        fn get<'a>(&'a self, _: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
            Box::pin(async move { Ok(None) })
        }
        fn get_at_seq<'a>(
            &'a self,
            _: &'a [u8],
            _: u64,
        ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
            Box::pin(async move { Ok(None) })
        }
        fn put<'a>(&'a self, _: TxId, _: &'a [u8], _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn delete<'a>(&'a self, _: TxId, _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                Err(contextra_core::ContextraError::Storage(
                    "Simulated Storage Commit Failure".into(),
                ))
            })
        }
        fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn rollback_to_tx<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
            Box::pin(async move {
                Ok(StorageStats {
                    num_segments: 0,
                    total_size_bytes: 0,
                    memtable_size_bytes: 0,
                })
            })
        }
        fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
            Box::pin(async move { Ok(0) })
        }
        fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
            Box::pin(async move { Ok(TxId(0)) })
        }
        fn pin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn unpin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn scan_prefix<'a>(
            &'a self,
            _: &'a [u8],
        ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { Ok(vec![]) })
        }
        fn scan<'a>(
            &'a self,
            _: std::ops::Bound<&'a [u8]>,
            _: std::ops::Bound<&'a [u8]>,
            _: Option<usize>,
        ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { Ok(vec![]) })
        }
    }

    let storage = Arc::new(FailOnStorageCommit);
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap
    );
    let graph = Arc::new(CsrGraph::new());
    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        graph.clone(),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    let res = col.relate("node_x", "node_y", "links").await;
    assert!(
        res.is_err(),
        "relate() must fail when storage.commit() fails"
    );

    // Graph index should remain empty since relate failed before graph commit
    assert_eq!(graph.entity_count(), 0);
}


#[tokio::test]
async fn test_relate_rollback_semantics_on_graph_commit_failure() {
    use contextra_core::{BoxFuture, Result, StorageEngine, StorageStats, TxId};
    use contextra_graph::csr::{CsrGraph, CsrGraphConfig};
    use contextra_store::{LsmConfig, LsmStorage};
    use contextra_vector::HnswIndex;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Arc;
    use tempfile::tempdir;

    struct FailOnPutStorage {
        should_fail: AtomicBool,
    }

    impl StorageEngine for FailOnPutStorage {
        fn get<'a>(&'a self, _: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
            Box::pin(async move { Ok(None) })
        }
        fn get_at_seq<'a>(
            &'a self,
            _: &'a [u8],
            _: u64,
        ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
            Box::pin(async move { Ok(None) })
        }
        fn put<'a>(&'a self, _: TxId, _: &'a [u8], _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                if self.should_fail.load(Ordering::SeqCst) {
                    Err(contextra_core::ContextraError::Storage(
                        "Simulated Graph Storage Commit Failure".into(),
                    ))
                } else {
                    Ok(())
                }
            })
        }
        fn delete<'a>(&'a self, _: TxId, _: &'a [u8]) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn commit<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn rollback<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn rollback_to_tx<'a>(&'a self, _: TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
            Box::pin(async move {
                Ok(StorageStats {
                    num_segments: 0,
                    total_size_bytes: 0,
                    memtable_size_bytes: 0,
                })
            })
        }
        fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
            Box::pin(async move { Ok(0) })
        }
        fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
            Box::pin(async move { Ok(TxId(0)) })
        }
        fn pin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn unpin_checkpoint<'a>(&'a self, _: u64) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn scan_prefix<'a>(
            &'a self,
            _: &'a [u8],
        ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { Ok(vec![]) })
        }
        fn scan<'a>(
            &'a self,
            _: std::ops::Bound<&'a [u8]>,
            _: std::ops::Bound<&'a [u8]>,
            _: Option<usize>,
        ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { Ok(vec![]) })
        }
    }

    let dir = tempdir().unwrap(); // unwrap
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap()); // unwrap
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap
    );

    let fail_storage = Arc::new(FailOnPutStorage {
        should_fail: AtomicBool::new(true),
    });
    let graph = Arc::new(CsrGraph::with_config_and_storage(
        CsrGraphConfig::default(),
        fail_storage,
    ));
    let next_tx = Arc::new(AtomicU64::new(1));

    let col = Collection::new(
        "default".to_string(),
        storage.clone(),
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    );

    // relate() should fail when graph_index.commit() fails
    let res = col.relate("entity_a", "entity_b", "connects").await;
    assert!(
        res.is_err(),
        "relate() must return Err when graph commit fails"
    );

    // Verification: storage MUST NOT contain the relation key after failed relate()
    let rel_prefix = col.namespaced_key(b"", 2);
    let remaining_rels = storage.scan_prefix(&rel_prefix).await.unwrap(); // unwrap
    assert!(
        remaining_rels.is_empty(),
        "Storage layer MUST NOT contain relation keys after relate() failure! Found: {:?}",
        remaining_rels
    );
}


#[tokio::test]
async fn test_link_memories_cycle_prevention_for_all_relations() -> contextra_core::Result<()> {
    use contextra_core::DocId;
    use contextra_graph::CsrGraph;
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(),
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(),
    );
    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert("node_a", &[1.0, 0.0, 0.0, 0.0], None).await?;
    col.insert("node_b", &[0.0, 1.0, 0.0, 0.0], None).await?;
    col.insert("node_c", &[0.0, 0.0, 1.0, 0.0], None).await?;

    let a = DocId::from_key("node_a")?;
    let b = DocId::from_key("node_b")?;
    let c = DocId::from_key("node_c")?;

    let rel = contextra_core::types::domain::LinkRelation::Elaborates;

    // A -> B
    col.link_memories(a, b, rel).await?;
    // B -> C
    col.link_memories(b, c, rel).await?;

    // C -> A should fail with cycle detection error!
    let cycle_res = col.link_memories(c, a, rel).await;
    assert!(cycle_res.is_err(), "Cyclic link must be rejected");
    let err_str = cycle_res.unwrap_err().to_string();
    assert!(err_str.contains("Cyclic Elaborates relation detected"));

    Ok(())
}


#[cfg(feature = "graph-connectivity-health")]
#[tokio::test]
async fn test_run_percolation_check_rebonding() -> contextra_core::Result<()> {
    use crate::{Contextra, ContextraConfig};
    use contextra_core::EntityId;

    let dir = tempfile::tempdir().unwrap();
    let db = Contextra::open_with_config(
        dir.path(),
        ContextraConfig {
            dimension: 4,
            ..Default::default()
        },
    )
    .await?;

    let col = db.collection("percolation_test").await?;

    // Insert documents with high similarity between doc_0 and doc_1 (at index > 10,000), but no edge
    // To ensure doc_1 is beyond the DEFAULT_SCAN_LIMIT (10,000) boundary, insert filler items
    for i in 0..10_010 {
        let id = if i == 0 {
            "doc_0".to_string()
        } else if i == 10_005 {
            "doc_1".to_string()
        } else {
            format!("doc_{i}")
        };
        let emb = if id == "doc_0" {
            vec![1.0, 0.0, 0.0, 0.0]
        } else if id == "doc_1" {
            vec![0.98, 0.02, 0.0, 0.0]
        } else {
            let val = ((i % 100) as f32) / 100.0;
            vec![0.0, 0.0, val, 1.0 - val]
        };
        col.insert(&id, &emb, None).await?;
    }

    let config = contextra_graph::percolation::PercolationConfig {
        critical_threshold: 0.9,
        rebonding_similarity: 0.85,
        max_new_edges_per_pass: 10,
    };

    let result = col.run_percolation_check(&config).await?;
    assert!(result.health.is_some());
    assert!(result.rebonding_triggered);
    assert!(result.new_edges_added > 0);

    // Verify rebonded relationship now exists in graph
    let neighbors = col
        .graph_index
        .neighbors(EntityId::from_key("doc_0")?)
        .await?;
    assert!(neighbors.contains(&EntityId::from_key("doc_1")?));

    Ok(())
}
