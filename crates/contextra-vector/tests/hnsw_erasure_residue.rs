use contextra_core::{DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{
    GhostFreeVectorIndex, HnswConfigBuilder, HnswIndex, HnswResidueReport,
};

#[tokio::test]
async fn test_erasure_canary_pattern_zero_residue() {
    let dim = 8;
    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(32)
        .build()
        .expect("valid config");

    let index = HnswIndex::try_new(config).expect("created index");
    let tx = TxId::try_new(1).unwrap();

    // Canary vector with distinct non-zero values
    let canary_doc = DocId::from(100u64);
    let canary_vec = vec![0.1337f32, 0.42f32, 0.99f32, 0.123f32, 0.555f32, 0.777f32, 0.888f32, 0.999f32];

    index.insert(tx, canary_doc, &canary_vec).await.expect("insert");

    // Insert another doc so the graph has edges
    let doc2 = DocId::from(101u64);
    let vec2 = vec![0.1f32, 0.2f32, 0.3f32, 0.4f32, 0.5f32, 0.6f32, 0.7f32, 0.8f32];
    index.insert(tx, doc2, &vec2).await.expect("insert 2");

    index.commit(tx).await.expect("commit");

    // Delete canary_doc using remove_with_graph_repair
    let mut index_mut = index;
    let _stats = index_mut
        .remove_with_graph_repair(canary_doc)
        .expect("remove_with_graph_repair");

    let report: HnswResidueReport = index_mut.scan_residue_report(canary_doc);
    assert!(!report.node_found, "Node should not be found");
    assert_eq!(report.non_zero_vector_bytes, 0, "Vector bytes must be zeroized");
    assert_eq!(report.incoming_edges, 0, "No incoming edges allowed");
    assert_eq!(report.outgoing_edges, 0, "No outgoing edges allowed");
    assert!(!report.entry_point_is_doc, "Entry point must not be deleted doc");
    assert!(!report.quantized_code_found, "No quantized code should exist");
}

#[tokio::test]
async fn test_fault_injection_incomplete_deletion_detected() {
    let dim = 4;
    let config = HnswConfigBuilder::new(dim).build().expect("valid config");
    let index = HnswIndex::try_new(config).expect("created index");
    let tx = TxId::try_new(1).unwrap();
    let doc = DocId::from(50u64);
    let vec = vec![1.0f32, 2.0f32, 3.0f32, 4.0f32];
    index.insert(tx, doc, &vec).await.expect("insert");
    index.commit(tx).await.expect("commit");

    // Enable fault injection: skip zeroization during delete
    index.set_fault_injection_skip_zeroize(true);

    let mut index_mut = index;
    let _ = index_mut.remove_with_graph_repair(doc);

    let report = index_mut.scan_residue_report(doc);
    // Fault injection skipped zeroize, so report should detect non-zero vector bytes or node residual
    assert!(
        report.non_zero_vector_bytes > 0 || report.node_found,
        "Residue report must detect incomplete zeroization under fault injection"
    );
}

#[tokio::test]
async fn test_slot_reuse_no_ghost_edges() {
    let dim = 4;
    let config = HnswConfigBuilder::new(dim).m(8).build().expect("valid config");
    let index = HnswIndex::try_new(config).expect("created index");

    let tx1 = TxId::try_new(1).unwrap();
    let doc1 = DocId::from(10u64);
    let vec1 = vec![1.0, 0.0, 0.0, 0.0];
    index.insert(tx1, doc1, &vec1).await.expect("insert 1");
    index.commit(tx1).await.expect("commit 1");

    let mut index_mut = index;
    index_mut.remove_with_graph_repair(doc1).expect("remove doc1");

    // Insert doc2 which will reuse the freed slot
    let tx2 = TxId::try_new(2).unwrap();
    let doc2 = DocId::from(20u64);
    let vec2 = vec![0.0, 1.0, 0.0, 0.0];
    index_mut.insert(tx2, doc2, &vec2).await.expect("insert 2");
    index_mut.commit(tx2).await.expect("commit 2");

    let report_doc1 = index_mut.scan_residue_report(doc1);
    assert_eq!(report_doc1.incoming_edges, 0, "No edges to old doc1");
    assert_eq!(report_doc1.outgoing_edges, 0, "No outgoing edges from doc1 slot");
    assert_eq!(report_doc1.non_zero_vector_bytes, 0, "No vector bytes for doc1");

    let report_doc2 = index_mut.scan_residue_report(doc2);
    assert!(report_doc2.node_found, "Doc2 must be present");
    assert!(report_doc2.non_zero_vector_bytes > 0, "Doc2 has vector bytes");
}

#[tokio::test]
async fn test_entry_point_deletion_keeps_index_searchable() {
    let dim = 4;
    let config = HnswConfigBuilder::new(dim).build().expect("valid config");
    let index = HnswIndex::try_new(config).expect("created index");

    let tx = TxId::try_new(1).unwrap();
    let doc_ep = DocId::from(1u64);
    let doc_other = DocId::from(2u64);

    index.insert(tx, doc_ep, &[1.0, 0.0, 0.0, 0.0]).await.unwrap();
    index.insert(tx, doc_other, &[0.0, 1.0, 0.0, 0.0]).await.unwrap();
    index.commit(tx).await.unwrap();

    let mut index_mut = index;
    index_mut.remove_with_graph_repair(doc_ep).expect("remove entry point doc");

    let report_ep = index_mut.scan_residue_report(doc_ep);
    assert!(!report_ep.entry_point_is_doc, "Entry point must no longer point to deleted doc");

    // Search should still succeed and find remaining doc
    let results = index_mut.search(&[0.0, 1.0, 0.0, 0.0], 5).await.expect("search after ep delete");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].doc_id, doc_other);
}

#[tokio::test]
async fn test_concurrency_pinned_reader_reported() {
    let dim = 4;
    let config = HnswConfigBuilder::new(dim).build().expect("valid config");
    let index = HnswIndex::try_new(config).expect("created index");

    let tx = TxId::try_new(1).unwrap();
    let doc = DocId::from(1u64);
    index.insert(tx, doc, &[1.0, 2.0, 3.0, 4.0]).await.unwrap();
    index.commit(tx).await.unwrap();

    // Pin snapshot at seq 1
    index.pin_snapshot(1);

    let report = index.scan_residue_report(doc);
    assert!(report.pinned_by_reader, "Active snapshot pin must be reported as pinned_by_reader");

    index.unpin_snapshot(1);
    let report_unpinned = index.scan_residue_report(doc);
    assert!(!report_unpinned.pinned_by_reader, "After unpinning, pinned_by_reader should be false");
}
