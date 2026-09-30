use contextra_core::{DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{GhostFreeVectorIndex, HnswConfig, HnswIndex};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]
    #[test]
    fn prop_ghost_free_deletion_removes_all_pointers(
        num_nodes in 10usize..50,
        m in 4usize..12,
        seed in 0u64..1000,
    ) {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async {
            let dimension = 8;
            let config = HnswConfig {
                dimension,
                m,
                ef_construction: 32,
                ..Default::default()
            };

            let index = HnswIndex::try_new(config).expect("Created HNSW index");

            // Deterministic pseudo-random vector generation using seed
            for i in 0..num_nodes {
                let doc_id = DocId::new(((i + 1) as u64).into());
                let mut vector = vec![0.0f32; dimension];
                for d in 0..dimension {
                    let val = (((i * dimension + d) as u64 + seed) % 100) as f32 / 100.0;
                    vector[d] = val;
                }
                index.insert(TxId::new((i + 1) as u64), doc_id, &vector).await.expect("insert");
                index.commit(TxId::new((i + 1) as u64)).await.expect("commit");
            }

            // Pick a target node to delete that exists in index (e.g. node 1)
            let target_doc_id = DocId::new(1);

            let mut mut_index = index;
            let stats = mut_index.remove_with_graph_repair(target_doc_id).expect("remove_with_graph_repair succeeded");

            assert_eq!(stats.doc_id, target_doc_id);
            assert!(stats.verified_no_ghost_pointers, "verified_no_ghost_pointers MUST be true");

            // Exhaustive verification scan using all_doc_ids_from_map
            let remaining_doc_ids = mut_index.all_doc_ids_from_map();
            for &id in &remaining_doc_ids {
                assert_ne!(id, target_doc_id, "Deleted doc_id MUST NOT exist in remaining doc_ids");
            }
        });
    }
}
