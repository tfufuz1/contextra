#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![expect(clippy::unwrap_used)]

use memfuse_core::{DocId, EntityId, StorageEngine, TxId};
use memfuse_graph::cascade::{
    cascade_invalidate_hyperedges_for_superseded_doc, CASCADE_QUEUE_PREFIX,
    MAX_HYPEREDGE_CASCADE_FANOUT,
};
use memfuse_graph::csr::{CsrGraph, EdgeType};
use memfuse_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use memfuse_store::{LsmConfig, LsmStorage};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_hyperedge_cascade_queue_persistence_and_drain() {
    let dir = tempdir().unwrap();
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(),
    );

    let graph = CsrGraph::with_storage(storage.clone());
    let doc_a = DocId::from_key("doc-cascade-test").unwrap();

    const ROLE_1: RoleId = RoleId::new(1);
    const ROLE_2: RoleId = RoleId::new(2);

    // Insert MAX_HYPEREDGE_CASCADE_FANOUT + 50 hyperedges
    let total_count = MAX_HYPEREDGE_CASCADE_FANOUT + 50;
    for i in 1..=total_count {
        let id = HyperEdgeId::new(i as u64);
        let bindings = vec![
            RoleBinding::new(ROLE_1, EntityId::new(i as u64 * 2)),
            RoleBinding::new(ROLE_2, EntityId::new(i as u64 * 2 + 1)),
        ];
        let he =
            HyperEdge::new(id, EdgeType::Default, bindings, 1.0).with_source_doc_id(Some(doc_a));
        graph.insert_hyperedge_direct(he);
    }

    let wal_seq = 10u64;
    let wal_tx = TxId::new(wal_seq);
    let report = cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_a, wal_seq)
        .await
        .unwrap();

    assert_eq!(report.invalidated.len(), MAX_HYPEREDGE_CASCADE_FANOUT);
    assert_eq!(report.deferred.len(), 50);
    assert_eq!(report.queued_for_background, 50);
    assert_eq!(graph.pending_cascade_queue_len(), 50);

    // Commit WAL transaction to make staged writes visible in LSM storage snapshot
    storage.commit(wal_tx).await.unwrap();

    // Verify LSM storage entries exist for deferred hyperedges
    let pfx = format!("{CASCADE_QUEUE_PREFIX}{:016x}:", doc_a.inner());
    let (entries, _) = storage
        .scan_prefix_bounded(pfx.as_bytes(), 100, None)
        .await
        .unwrap();
    assert_eq!(entries.len(), 50);

    // Process cascade queue in background batch
    let process_tx = TxId::new(20);
    let processed = graph.process_cascade_queue(100, process_tx).await.unwrap();
    assert_eq!(processed, 50);
    assert_eq!(graph.pending_cascade_queue_len(), 0);

    // Commit deletion transaction to verify LSM deletion
    storage.commit(process_tx).await.unwrap();

    // Verify storage keys were deleted
    let (remaining_entries, _) = storage
        .scan_prefix_bounded(pfx.as_bytes(), 100, None)
        .await
        .unwrap();
    assert!(remaining_entries.is_empty());

    // Verify all hyperedges are now tombstoned
    assert!(graph.hyperedges_for_doc(doc_a).is_empty());
}
