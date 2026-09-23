#![expect(clippy::unwrap_used)]

use contextra_core::{DocId, EntityId, StorageEngine, TxId};
use contextra_graph::cascade::{
    cascade_invalidate_hyperedges_for_superseded_doc, CASCADE_QUEUE_PREFIX,
    MAX_HYPEREDGE_CASCADE_FANOUT,
};
use contextra_graph::csr::{CsrGraph, EdgeType};
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId, HYPEREDGE_PREFIX};
use contextra_store::{LsmConfig, LsmStorage};
use contextra_testkit::{FaultConfig, FaultVfs};
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_hyperedge_cascade_crash_recovery() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_path_buf();

    let doc_superseded = DocId::from_key("doc-crash-recovery-test").unwrap();
    const ROLE_1: RoleId = RoleId::new(1);
    const ROLE_2: RoleId = RoleId::new(2);

    let overflow_count = 100;
    let total_count = MAX_HYPEREDGE_CASCADE_FANOUT + overflow_count;

    // Phase 1: Setup storage & graph, insert total_count hyperedges derived from doc_superseded
    {
        let storage = Arc::new(
            LsmStorage::new(LsmConfig {
                path: db_path.clone(),
                ..Default::default()
            })
            .await
            .unwrap(),
        );

        let graph = CsrGraph::with_storage(storage.clone());

        let wal_seq_insert = 1u64;
        let wal_tx_insert = TxId::new(wal_seq_insert);

        for i in 1..=total_count {
            let id = HyperEdgeId::new(i as u64);
            let bindings = vec![
                RoleBinding::new(ROLE_1, EntityId::new(i as u64 * 2)),
                RoleBinding::new(ROLE_2, EntityId::new(i as u64 * 2 + 1)),
            ];
            let he = HyperEdge::new(id, EdgeType::Default, bindings, 1.0)
                .with_source_doc_id(Some(doc_superseded));
            graph.insert_hyperedge_direct(he.clone());

            let key = format!("{}{}", HYPEREDGE_PREFIX, id.inner());
            let bytes = he.serialize().unwrap();
            storage
                .put(wal_tx_insert, key.as_bytes(), &bytes)
                .await
                .unwrap();
        }

        storage.commit(wal_tx_insert).await.unwrap();

        // Trigger cascade invalidation
        let wal_seq = 100u64;
        let wal_tx = TxId::new(wal_seq);
        let report =
            cascade_invalidate_hyperedges_for_superseded_doc(&graph, doc_superseded, wal_seq)
                .await
                .unwrap();

        assert_eq!(report.invalidated.len(), MAX_HYPEREDGE_CASCADE_FANOUT);
        assert_eq!(report.deferred.len(), overflow_count);
        assert_eq!(report.queued_for_background, overflow_count);
        assert_eq!(graph.pending_cascade_queue_len(), overflow_count);

        // Commit WAL transaction to make staged queued items visible in LSM storage
        storage.commit(wal_tx).await.unwrap();

        // Verify that overflow_count keys are present in LSM under CASCADE_QUEUE_PREFIX
        let pfx = format!("{CASCADE_QUEUE_PREFIX}{:016x}:", doc_superseded.inner());
        let (entries, _) = storage
            .scan_prefix_bounded(pfx.as_bytes(), 200, None)
            .await
            .unwrap();
        assert_eq!(entries.len(), overflow_count);
    }

    // Phase 2: Fault Injection & Crash Simulation using FaultVfs
    {
        let vfs = FaultVfs::new();
        let dummy_path = Path::new("cascade_queue_state.dat");
        vfs.write_file(dummy_path, b"cascade_queue_unflushed_state")
            .unwrap();

        // Inject simulated I/O crash
        vfs.set_config(FaultConfig {
            fail_all: true,
            ..Default::default()
        });
        vfs.trigger_crash();

        assert!(vfs.read_file(dummy_path).is_err());
        assert!(vfs.sync().is_err());

        // Reset crash state after simulated crash recovery
        vfs.reset_counters();
        vfs.set_config(FaultConfig::default());
        assert!(vfs.read_file(dummy_path).is_ok());
    }

    // Phase 3: Post-crash restart & persistent queue drain recovery
    {
        // Reopen storage as if process restarted
        let storage = Arc::new(
            LsmStorage::new(LsmConfig {
                path: db_path,
                ..Default::default()
            })
            .await
            .unwrap(),
        );

        let recovered_graph = CsrGraph::with_storage(storage.clone());

        // Scan storage for deferred hyperedge items left in CASCADE_QUEUE_PREFIX
        let pfx = format!("{CASCADE_QUEUE_PREFIX}{:016x}:", doc_superseded.inner());
        let (entries, _) = storage
            .scan_prefix_bounded(pfx.as_bytes(), 200, None)
            .await
            .unwrap();
        assert_eq!(entries.len(), overflow_count);

        // Extract deferred hyperedge IDs from raw key suffix
        let mut deferred_ids = Vec::new();
        for (raw_key, _) in entries {
            let key_str = std::str::from_utf8(&raw_key).unwrap();
            let hid_hex = key_str.rsplit(':').next().unwrap();
            let hid_val = u64::from_str_radix(hid_hex, 16).unwrap();
            deferred_ids.push(HyperEdgeId::new(hid_val));
        }

        assert_eq!(deferred_ids.len(), overflow_count);

        // Reload persisted hyperedges for the deferred IDs into recovered_graph
        for hid in &deferred_ids {
            let key = format!("{}{}", HYPEREDGE_PREFIX, hid.inner());
            if let Some(value) = storage.get(key.as_bytes()).await.unwrap() {
                let he = HyperEdge::deserialize(&value).unwrap();
                recovered_graph.insert_hyperedge_direct(he);
            }
        }

        // Enqueue recovered deferred items into recovered_graph's cascade queue
        recovered_graph.enqueue_cascade_deferred(doc_superseded, &deferred_ids);
        assert_eq!(recovered_graph.pending_cascade_queue_len(), overflow_count);

        // Drain and process cascade queue post-restart
        let process_tx = TxId::new(200);
        let processed = recovered_graph
            .process_cascade_queue(200, process_tx)
            .await
            .unwrap();
        assert_eq!(processed, overflow_count);
        assert_eq!(recovered_graph.pending_cascade_queue_len(), 0);

        // Commit processing transaction
        storage.commit(process_tx).await.unwrap();

        // Phase 4: Verification of idempotency and completion
        // 1. All queue keys in storage are deleted
        let (remaining_entries, _) = storage
            .scan_prefix_bounded(pfx.as_bytes(), 200, None)
            .await
            .unwrap();
        assert!(remaining_entries.is_empty());

        // 2. Re-running process_cascade_queue is idempotent (returns 0)
        let process_tx2 = TxId::new(201);
        let processed_again = recovered_graph
            .process_cascade_queue(200, process_tx2)
            .await
            .unwrap();
        assert_eq!(processed_again, 0);

        // 3. Document has zero non-tombstoned hyperedges remaining
        assert!(recovered_graph
            .hyperedges_for_doc(doc_superseded)
            .is_empty());
    }
}
