#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![expect(clippy::unwrap_used)]

use memfuse_core::{DocId, EntityId, StorageEngine, TxId};
use memfuse_graph::csr::{CsrGraph, EdgeType};
use memfuse_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId, HYPEREDGE_PREFIX};
use memfuse_store::{LsmConfig, LsmStorage};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_hyperedge_persistence_survives_restart() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_path_buf();

    let doc_oracle = DocId::from_key("doc-persistence-oracle").unwrap();
    const ROLE_SUBJ: RoleId = RoleId::new(1);
    const ROLE_OBJ: RoleId = RoleId::new(2);
    const ROLE_CTX: RoleId = RoleId::new(3);

    let he1 = HyperEdge::new(
        HyperEdgeId::new(101),
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_SUBJ, EntityId::new(1001)),
            RoleBinding::new(ROLE_OBJ, EntityId::new(1002)),
            RoleBinding::new(ROLE_CTX, EntityId::new(1003)),
        ],
        0.95,
    )
    .with_tx_validity(Some(TxId::new(1)), None)
    .with_business_validity(Some(100000), Some(200000))
    .with_source_doc_id(Some(doc_oracle));

    let he2 = HyperEdge::new(
        HyperEdgeId::new(102),
        EdgeType::Default,
        vec![
            RoleBinding::new(ROLE_SUBJ, EntityId::new(2001)),
            RoleBinding::new(ROLE_OBJ, EntityId::new(2002)),
        ],
        0.80,
    )
    .with_tx_validity(Some(TxId::new(1)), None)
    .with_source_doc_id(Some(doc_oracle));

    // Phase 1: Initialize Storage & Graph, persist hyperedges to LSM
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
        graph.insert_hyperedge(he1.clone());
        graph.insert_hyperedge(he2.clone());

        let tx = TxId::new(1);
        for edge in [&he1, &he2] {
            let key = format!("{}{}", HYPEREDGE_PREFIX, edge.id.inner());
            let bytes = edge.serialize().unwrap();
            storage.put(tx, key.as_bytes(), &bytes).await.unwrap();
        }

        storage.commit(tx).await.unwrap();

        // Verify in-memory graph state prior to shutdown
        assert_eq!(graph.get_hyperedge(he1.id).as_deref(), Some(&he1));
        assert_eq!(graph.get_hyperedge(he2.id).as_deref(), Some(&he2));
        assert_eq!(graph.hyperedges_for_doc(doc_oracle).len(), 2);
    }

    // Phase 2: Simulate process restart — reopen LSM Storage and load persisted hyperedges
    {
        let storage = Arc::new(
            LsmStorage::new(LsmConfig {
                path: db_path,
                ..Default::default()
            })
            .await
            .unwrap(),
        );

        let recovered_graph = CsrGraph::with_storage(storage.clone());

        let (entries, _) = storage
            .scan_prefix_bounded(HYPEREDGE_PREFIX.as_bytes(), 100, None)
            .await
            .unwrap();

        assert_eq!(entries.len(), 2);

        for (_key, value) in entries {
            let recovered_he = HyperEdge::deserialize(&value).unwrap();
            recovered_graph.insert_hyperedge(recovered_he);
        }

        // Phase 3: Assert exact parity between reconstructed hyperedges and original oracle
        let recovered_he1 = recovered_graph.get_hyperedge(he1.id).expect("he1 missing");
        let recovered_he2 = recovered_graph.get_hyperedge(he2.id).expect("he2 missing");

        assert_eq!(*recovered_he1, he1);
        assert_eq!(*recovered_he2, he2);

        let doc_hes = recovered_graph.hyperedges_for_doc(doc_oracle);
        assert_eq!(doc_hes.len(), 2);
        assert!(doc_hes.contains(&he1.id));
        assert!(doc_hes.contains(&he2.id));

        let e1_hes = recovered_graph.hyperedges_for_entity(EntityId::new(1001));
        assert_eq!(e1_hes, vec![he1.id]);
    }
}
