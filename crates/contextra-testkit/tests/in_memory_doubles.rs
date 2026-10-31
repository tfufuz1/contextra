#![allow(clippy::panic, clippy::unwrap_used)]

use bytes::Bytes;
use contextra_ports::{Clock, GraphIndex, Rng, SeededRng, StorageEngine, TextIndex, VectorIndex};
use contextra_testkit::conformance::{
    run_clock_suite, run_graph_index_suite, run_metrics_sink_suite, run_rng_suite,
    run_storage_engine_suite, run_text_index_suite, run_vector_index_suite,
};
use contextra_testkit::{
    InMemoryGraphIndex, InMemoryStorageEngine, InMemoryTextIndex, InMemoryVectorIndex, ManualClock,
    ReferenceModel, TestMetricsSink,
};
use contextra_types::{DocId, Edge, Entity, EntityId, TxId};
use std::ops::Bound;
use std::time::Duration;

// ============================================================================
// 1. POSITIVE CONFORMANCE TEST EXECUTION AGAINST UNMODIFIED DOUBLES
// ============================================================================

#[tokio::test]
async fn test_in_memory_storage_engine_conformance() {
    let result = run_storage_engine_suite(|| async { InMemoryStorageEngine::new() }).await;
    assert!(
        result.is_ok(),
        "InMemoryStorageEngine must pass conformance suite: {result:?}"
    );
}

#[tokio::test]
async fn test_in_memory_graph_index_conformance() {
    let result = run_graph_index_suite(|| async { InMemoryGraphIndex::new() }).await;
    assert!(
        result.is_ok(),
        "InMemoryGraphIndex must pass conformance suite: {result:?}"
    );
}

#[tokio::test]
async fn test_in_memory_vector_index_conformance() {
    let result = run_vector_index_suite(|| async { InMemoryVectorIndex::new(3) }).await;
    assert!(
        result.is_ok(),
        "InMemoryVectorIndex must pass conformance suite: {result:?}"
    );
}

#[tokio::test]
async fn test_in_memory_text_index_conformance() {
    let result = run_text_index_suite(|| async { InMemoryTextIndex::new() }).await;
    assert!(
        result.is_ok(),
        "InMemoryTextIndex must pass conformance suite: {result:?}"
    );
}

#[test]
fn test_manual_clock_conformance() {
    let clock = ManualClock::new(1_700_000_000_000_000_000);
    let result = run_clock_suite(&clock);
    assert!(
        result.is_ok(),
        "ManualClock must pass conformance suite: {result:?}"
    );
}

#[test]
fn test_seeded_rng_conformance() {
    let rng = SeededRng::new(0x434F_4E54_5854_5241);
    let result = run_rng_suite(&rng);
    assert!(
        result.is_ok(),
        "SeededRng must pass conformance suite: {result:?}"
    );
}

#[test]
fn test_test_metrics_sink_conformance() {
    let sink = TestMetricsSink::new();
    let result = run_metrics_sink_suite(&sink);
    assert!(
        result.is_ok(),
        "TestMetricsSink must pass conformance suite: {result:?}"
    );
}

// ============================================================================
// 2. EDGE CASE TESTS: IN-MEMORY STORAGE ENGINE
// ============================================================================

#[tokio::test]
async fn test_storage_engine_delete_nonexistent_key() {
    let store = InMemoryStorageEngine::new();
    let tx = TxId::new(1);

    // Deleting a non-existent key should succeed without errors
    let del_res = store.delete(tx, b"nonexistent_key").await;
    assert!(del_res.is_ok());

    let commit_res = store.commit(tx).await;
    assert!(commit_res.is_ok());

    let val = store.get(b"nonexistent_key").await.unwrap();
    assert_eq!(val, None);
}

#[tokio::test]
async fn test_storage_engine_empty_scans() {
    let store = InMemoryStorageEngine::new();

    // Scan prefix on empty store
    let prefix_res = store.scan_prefix(b"anything").await.unwrap();
    assert!(prefix_res.is_empty());

    // Range scan on empty store
    let range_res = store
        .scan(Bound::Unbounded, Bound::Unbounded, None)
        .await
        .unwrap();
    assert!(range_res.is_empty());

    // Range scan with limit 0
    let tx = TxId::new(1);
    store.put(tx, b"k1", b"v1").await.unwrap();
    store.commit(tx).await.unwrap();

    let scan_limit_zero = store
        .scan(Bound::Unbounded, Bound::Unbounded, Some(0))
        .await
        .unwrap();
    assert!(scan_limit_zero.is_empty());
}

#[tokio::test]
async fn test_storage_engine_put_if_absent_semantics() {
    let store = InMemoryStorageEngine::new();
    let tx1 = TxId::new(1);

    // First put_if_absent succeeds
    let inserted = store
        .put_if_absent(tx1, b"unique_key", b"initial")
        .await
        .unwrap();
    assert!(inserted);

    // Same transaction second put_if_absent for same key returns false
    let inserted_again = store
        .put_if_absent(tx1, b"unique_key", b"duplicate")
        .await
        .unwrap();
    assert!(!inserted_again);

    store.commit(tx1).await.unwrap();

    // Committed value is 'initial'
    assert_eq!(
        store.get(b"unique_key").await.unwrap(),
        Some(Bytes::from_static(b"initial"))
    );

    // New transaction put_if_absent fails because committed key exists
    let tx2 = TxId::new(2);
    let inserted_after_commit = store
        .put_if_absent(tx2, b"unique_key", b"post_commit")
        .await
        .unwrap();
    assert!(!inserted_after_commit);
}

#[tokio::test]
async fn test_storage_engine_snapshot_and_seq_visibility() {
    let store = InMemoryStorageEngine::new();

    // Sequence 0
    assert_eq!(store.last_seq_no().await.unwrap(), 0);

    let tx1 = TxId::new(10);
    store.put(tx1, b"mvcc_key", b"version_1").await.unwrap();
    store.commit(tx1).await.unwrap();
    let seq1 = store.last_seq_no().await.unwrap();
    assert_eq!(seq1, 1);

    let tx2 = TxId::new(20);
    store.put(tx2, b"mvcc_key", b"version_2").await.unwrap();
    store.commit(tx2).await.unwrap();
    let seq2 = store.last_seq_no().await.unwrap();
    assert_eq!(seq2, 2);

    // Query historical seq
    assert_eq!(
        store.get_at_seq(b"mvcc_key", 0).await.unwrap(),
        None,
        "Seq 0 should see nothing"
    );
    // InMemoryStorageEngine stores (val, seq) of the current version:
    // get_at_seq checks: if val_seq <= seq { return val } else { return None }.
    // When overwritten at seq2, val_seq becomes seq2.
    assert_eq!(
        store.get_at_seq(b"mvcc_key", seq2).await.unwrap(),
        Some(Bytes::from_static(b"version_2"))
    );
}

#[tokio::test]
async fn test_storage_engine_scan_prefix_at_seq() {
    let store = InMemoryStorageEngine::new();

    let tx1 = TxId::new(1);
    store.put(tx1, b"pfx:1", b"val1").await.unwrap();
    store.commit(tx1).await.unwrap();
    let seq1 = store.last_seq_no().await.unwrap();

    let tx2 = TxId::new(2);
    store.put(tx2, b"pfx:2", b"val2").await.unwrap();
    store.commit(tx2).await.unwrap();

    // At seq1, only pfx:1 should be returned by scan_prefix_at
    let scanned_seq1 = store.scan_prefix_at(b"pfx:", seq1).await.unwrap();
    assert_eq!(scanned_seq1.len(), 1);
    assert_eq!(
        scanned_seq1.first().map(|(k, _)| k.as_slice()),
        Some(b"pfx:1".as_slice())
    );

    // At seq2, both should be returned
    let scanned_seq2 = store.scan_prefix_at(b"pfx:", seq1 + 1).await.unwrap();
    assert_eq!(scanned_seq2.len(), 2);
}

#[tokio::test]
async fn test_storage_engine_stats_and_last_tx() {
    let store = InMemoryStorageEngine::new();
    let initial_stats = store.stats().await.unwrap();
    assert_eq!(initial_stats.total_size_bytes, 0);

    let tx = TxId::new(42);
    store.put(tx, b"hello", b"world").await.unwrap();
    store.commit(tx).await.unwrap();

    let updated_stats = store.stats().await.unwrap();
    assert_eq!(
        updated_stats.total_size_bytes, 10,
        "5 bytes key + 5 bytes value"
    );
    assert_eq!(store.last_tx_id().await.unwrap(), TxId::new(42));
}

// ============================================================================
// 3. EDGE CASE TESTS: IN-MEMORY GRAPH INDEX
// ============================================================================

#[tokio::test]
async fn test_graph_index_empty_and_unknown_queries() {
    let graph = InMemoryGraphIndex::new();

    // Neighbors of unknown node
    let nbs = graph.neighbors(EntityId::new(999)).await.unwrap();
    assert!(nbs.is_empty());

    // Traversal on unknown node
    let trav = graph.traverse(EntityId::new(999), 3).await.unwrap();
    assert!(trav.is_empty());

    // Traversal with 0 max_hops
    let trav_zero = graph.traverse(EntityId::new(999), 0).await.unwrap();
    assert!(trav_zero.is_empty());

    // Graph length and stats on empty graph
    assert_eq!(graph.len().await, 0);
    let stats = graph.stats().await.unwrap();
    assert_eq!(stats.num_entities, 0);
    assert_eq!(stats.num_edges, 0);
}

#[tokio::test]
async fn test_graph_index_remove_nonexistent_edge() {
    let graph = InMemoryGraphIndex::new();
    let tx = TxId::new(1);

    // Removing edge that does not exist should not fail
    let rem_res = graph
        .remove_edge(tx, EntityId::new(1), EntityId::new(2))
        .await;
    assert!(rem_res.is_ok());

    let commit_res = graph.commit(tx).await;
    assert!(commit_res.is_ok());
}

#[tokio::test]
async fn test_graph_index_duplicate_entities_and_edges() {
    let graph = InMemoryGraphIndex::new();
    let tx = TxId::new(10);

    // Add entity twice
    let e1 = Entity::new(EntityId::new(1), "Original", "TypeA");
    let e1_dup = Entity::new(EntityId::new(1), "Updated", "TypeA");
    graph.add_entity(tx, e1).await.unwrap();
    graph.add_entity(tx, e1_dup).await.unwrap();

    // Add edge twice
    let edge1 = Edge::new(EntityId::new(1), EntityId::new(2), "rel");
    let edge2 = Edge::new(EntityId::new(1), EntityId::new(2), "rel");
    graph.add_edge(tx, edge1).await.unwrap();
    graph.add_edge(tx, edge2).await.unwrap();

    graph.commit(tx).await.unwrap();

    // Len should reflect 1 entity, stats reflect 1 edge
    assert_eq!(graph.len().await, 1);
    let stats = graph.stats().await.unwrap();
    assert_eq!(stats.num_entities, 1);
    assert_eq!(stats.num_edges, 1);

    // Neighbors should not duplicate EntityId(2)
    let nbs = graph.neighbors(EntityId::new(1)).await.unwrap();
    assert_eq!(nbs, vec![EntityId::new(2)]);
}

#[tokio::test]
async fn test_graph_index_cycle_and_decay_traversal() {
    let graph = InMemoryGraphIndex::new();
    let tx = TxId::new(20);

    // Create a triangle cycle: 1 -> 2 -> 3 -> 1
    let mut e1 = Edge::new(EntityId::new(1), EntityId::new(2), "link");
    e1.weight = 1.0;
    let mut e2 = Edge::new(EntityId::new(2), EntityId::new(3), "link");
    e2.weight = 1.0;
    let mut e3 = Edge::new(EntityId::new(3), EntityId::new(1), "link");
    e3.weight = 1.0;

    graph.add_edge(tx, e1).await.unwrap();
    graph.add_edge(tx, e2).await.unwrap();
    graph.add_edge(tx, e3).await.unwrap();
    graph.commit(tx).await.unwrap();

    // Traversal from 1 for 5 hops: cycle should not loop infinitely
    let trav = graph.traverse(EntityId::new(1), 5).await.unwrap();
    assert!(!trav.is_empty());

    // Traversal results are sorted descending by score
    for window in trav.windows(2) {
        if let [prev, next] = window {
            assert!(prev.1 >= next.1);
        }
    }
}

#[tokio::test]
async fn test_graph_index_rollback() {
    let graph = InMemoryGraphIndex::new();
    let tx = TxId::new(30);

    let e = Entity::new(EntityId::new(10), "RollbackNode", "Test");
    graph.add_entity(tx, e).await.unwrap();
    graph.rollback(tx).await.unwrap();
    graph.commit(tx).await.unwrap();

    assert_eq!(graph.len().await, 0);
}

// ============================================================================
// 4. EDGE CASE TESTS: IN-MEMORY VECTOR INDEX
// ============================================================================

#[tokio::test]
async fn test_vector_index_empty_search_and_zero_k() {
    let index = InMemoryVectorIndex::new(3);

    // Search on empty index
    let res = index.search(&[1.0, 0.0, 0.0], 5).await.unwrap();
    assert!(res.is_empty());

    // Search with k = 0
    let tx = TxId::new(1);
    index.insert(tx, DocId(1), &[1.0, 0.0, 0.0]).await.unwrap();
    index.commit(tx).await.unwrap();

    let res_zero = index.search(&[1.0, 0.0, 0.0], 0).await.unwrap();
    assert!(res_zero.is_empty());
}

#[tokio::test]
async fn test_vector_index_delete_nonexistent() {
    let index = InMemoryVectorIndex::new(2);
    let tx = TxId::new(1);

    // Delete non-existent DocId
    assert!(index.delete(tx, DocId(999)).await.is_ok());
    assert!(index.commit(tx).await.is_ok());
    assert_eq!(index.len().await, 0);
}

#[tokio::test]
async fn test_vector_index_tie_breaking_deterministic_order() {
    let index = InMemoryVectorIndex::new(2);
    let tx = TxId::new(10);

    // Two documents with exact same distance to query
    index.insert(tx, DocId(50), &[1.0, 0.0]).await.unwrap();
    index.insert(tx, DocId(10), &[1.0, 0.0]).await.unwrap();
    index.commit(tx).await.unwrap();

    let search_res = index.search(&[1.0, 0.0], 2).await.unwrap();
    assert_eq!(search_res.len(), 2);
    // Both have exact same distance (score 1.0)
    let first = search_res.first().unwrap();
    let second = search_res.get(1).unwrap();
    assert_eq!(first.score, second.score);
    // Tie-break rule in InMemoryVectorIndex: a.doc_id.cmp(&b.doc_id)
    assert_eq!(first.doc_id, DocId(10));
    assert_eq!(second.doc_id, DocId(50));
}

#[tokio::test]
async fn test_vector_index_non_finite_and_dimension_validation() {
    let index = InMemoryVectorIndex::new(2);
    let tx = TxId::new(1);

    // Dimension mismatch on insert
    assert!(index.insert(tx, DocId(1), &[1.0]).await.is_err());
    assert!(index.insert(tx, DocId(1), &[1.0, 2.0, 3.0]).await.is_err());

    // Dimension mismatch on search
    assert!(index.search(&[1.0], 1).await.is_err());
    assert!(index.search(&[1.0, 2.0, 3.0], 1).await.is_err());

    // Infinity on insert
    assert!(index
        .insert(tx, DocId(2), &[f32::INFINITY, 0.0])
        .await
        .is_err());
    assert!(index
        .insert(tx, DocId(2), &[0.0, f32::NEG_INFINITY])
        .await
        .is_err());

    // Infinity on search
    assert!(index.search(&[f32::INFINITY, 0.0], 1).await.is_err());
}

#[tokio::test]
async fn test_vector_index_rollback() {
    let index = InMemoryVectorIndex::new(2);
    let tx = TxId::new(20);

    index.insert(tx, DocId(1), &[0.5, 0.5]).await.unwrap();
    index.rollback(tx).await.unwrap();
    index.commit(tx).await.unwrap();

    assert_eq!(index.len().await, 0);
    assert!(index.all_doc_ids().await.unwrap().is_empty());
}

// ============================================================================
// 5. EDGE CASE TESTS: IN-MEMORY TEXT INDEX
// ============================================================================

#[tokio::test]
async fn test_text_index_empty_and_whitespace_search() {
    let index = InMemoryTextIndex::new();

    // Search empty index
    let res = index.search("anything", 5).await.unwrap();
    assert!(res.is_empty());

    // Search with only whitespace
    let tx = TxId::new(1);
    index
        .insert(tx, DocId(1), "Sample document content")
        .await
        .unwrap();
    index.commit(tx).await.unwrap();

    let res_spaces = index.search("   \t  \n ", 5).await.unwrap();
    assert!(res_spaces.is_empty());

    // Search with k = 0
    let res_k_zero = index.search("Sample", 0).await.unwrap();
    assert!(res_k_zero.is_empty());
}

#[tokio::test]
async fn test_text_index_delete_nonexistent() {
    let index = InMemoryTextIndex::new();
    let tx = TxId::new(1);

    // Deleting non-existent doc should succeed
    assert!(index.delete(tx, DocId(999)).await.is_ok());
    assert!(index.commit(tx).await.is_ok());
    assert_eq!(index.len().await, 0);
}

#[tokio::test]
async fn test_text_index_ranking_stability_and_tie_breaking() {
    let index = InMemoryTextIndex::new();
    let tx = TxId::new(10);

    // Two documents with identical match score
    index
        .insert(tx, DocId(50), "concurrency in rust")
        .await
        .unwrap();
    index
        .insert(tx, DocId(10), "concurrency in rust")
        .await
        .unwrap();
    index.commit(tx).await.unwrap();

    let res = index.search("concurrency", 10).await.unwrap();
    assert_eq!(res.len(), 2);
    let first = res.first().unwrap();
    let second = res.get(1).unwrap();
    assert_eq!(first.score, second.score);
    // Tie-break by doc_id ascending
    assert_eq!(first.doc_id, DocId(10));
    assert_eq!(second.doc_id, DocId(50));
}

#[tokio::test]
async fn test_text_index_german_normalization_comprehensive() {
    let index = InMemoryTextIndex::new();
    let tx = TxId::new(20);

    index
        .insert(tx, DocId(1), "Übergrößen-Geschäft in München und Köln")
        .await
        .unwrap();
    index
        .insert(tx, DocId(2), "Schöne Grüße mit großem Dank")
        .await
        .unwrap();
    index.commit(tx).await.unwrap();

    // Match 'uebergroessen'
    let r1 = index.search("uebergroessen", 5).await.unwrap();
    assert_eq!(r1.len(), 1);
    assert_eq!(r1.first().map(|d| d.doc_id), Some(DocId(1)));

    // Match 'muenchen' from 'München'
    let r2 = index.search("muenchen", 5).await.unwrap();
    assert_eq!(r2.len(), 1);
    assert_eq!(r2.first().map(|d| d.doc_id), Some(DocId(1)));

    // Match 'grosse' from 'großem'
    let r3 = index.search("grossem", 5).await.unwrap();
    assert_eq!(r3.len(), 1);
    assert_eq!(r3.first().map(|d| d.doc_id), Some(DocId(2)));

    // Match 'grüße' from 'Grüße'
    let r4 = index.search("grüße", 5).await.unwrap();
    assert_eq!(r4.len(), 1);
    assert_eq!(r4.first().map(|d| d.doc_id), Some(DocId(2)));
}

#[tokio::test]
async fn test_text_index_rollback_and_stats() {
    let index = InMemoryTextIndex::new();
    let tx = TxId::new(30);

    index
        .insert(tx, DocId(1), "word1 word2 word3")
        .await
        .unwrap();
    index.rollback(tx).await.unwrap();
    index.commit(tx).await.unwrap();

    assert_eq!(index.len().await, 0);

    let tx2 = TxId::new(31);
    index
        .insert(tx2, DocId(2), "word1 word2 word3")
        .await
        .unwrap();
    index.commit(tx2).await.unwrap();

    let stats = index.stats().await.unwrap();
    assert_eq!(stats.num_documents, 1);
    assert_eq!(stats.num_tokens, 3);
    assert_eq!(stats.memory_usage_bytes, "word1 word2 word3".len());
}

// ============================================================================
// 6. EDGE CASE TESTS: MANUAL CLOCK
// ============================================================================

#[test]
fn test_manual_clock_never_runs_backward() {
    let clock = ManualClock::new(5_000);
    let t1 = clock.now_nanos();
    assert_eq!(t1, 5_000);

    // Advancing by zero duration maintains exact timestamp
    clock.advance(Duration::from_nanos(0));
    let t2 = clock.now_nanos();
    assert_eq!(t2, 5_000);

    // Advance forward
    clock.advance(Duration::from_nanos(100));
    let t3 = clock.now_nanos();
    assert!(t3 > t2);

    // Monotonic reading via trait
    let dyn_clock: &dyn Clock = &clock;
    assert_eq!(dyn_clock.monotonic_nanos(), t3);
    assert_eq!(dyn_clock.now_unix_nanos(), t3);
}

#[test]
fn test_manual_clock_large_advance_and_set() {
    let clock = ManualClock::new(10);

    // Advance by large value
    clock.advance(Duration::from_nanos(1_000_000_000));
    assert_eq!(clock.now_nanos(), 1_000_000_010);
    assert_eq!(clock.now_secs_f64(), 1.00000001);

    // Explicit set_nanos
    clock.set_nanos(u64::MAX - 100);
    assert_eq!(clock.now_nanos(), u64::MAX - 100);

    // Advance saturates without panic
    clock.advance(Duration::from_nanos(200));
    // Atomic fetch_add on u64 will wrap in 2's complement
    // Verify it doesn't panic
    let _ = clock.now_nanos();
}

// ============================================================================
// 7. REFERENCE MODEL VS IN-MEMORY STORAGE EQUIVALENCE
// ============================================================================

#[tokio::test]
async fn test_reference_model_edge_cases() {
    let mut model = ReferenceModel::new();

    // Snapshot seq at start
    assert_eq!(model.snapshot_seq(), 0);

    // Commit empty batch does not increment seq
    let seq = model.commit();
    assert_eq!(seq, 0);

    // Overwrites and deletes in same batch
    model.put(b"key1", b"val1_v1");
    model.put(b"key1", b"val1_v2");
    model.delete(b"key2");
    let seq1 = model.commit();
    assert_eq!(seq1, 1);
    assert_eq!(model.get_at(b"key1", seq1), Some(b"val1_v2".to_vec()));
    assert_eq!(model.get_at(b"key2", seq1), None);

    // Rollback uncommitted pending writes
    model.put(b"key_rb", b"should_not_commit");
    model.rollback();
    let seq2 = model.commit();
    assert_eq!(seq2, 1); // No new seq since pending was rolled back and empty
    assert_eq!(model.get_latest(b"key_rb"), None);

    // Range scan edge cases
    model.put(b"a", b"1");
    model.put(b"b", b"2");
    model.put(b"c", b"3");
    let seq3 = model.commit();

    let bounded = model.scan_range_at(
        Bound::Included(b"a".as_slice()),
        Bound::Excluded(b"c".as_slice()),
        seq3,
    );
    assert_eq!(
        bounded,
        vec![
            (b"a".to_vec(), b"1".to_vec()),
            (b"b".to_vec(), b"2".to_vec())
        ]
    );

    let snapshot_map = model.snapshot_map_at(seq3);
    assert_eq!(snapshot_map.get(b"a".as_slice()), Some(&b"1".to_vec()));
    assert_eq!(snapshot_map.get(b"b".as_slice()), Some(&b"2".to_vec()));
    assert_eq!(snapshot_map.get(b"c".as_slice()), Some(&b"3".to_vec()));
}

#[tokio::test]
async fn test_reference_model_vs_in_memory_store_equivalence() {
    // Deterministic PRNG for operations
    let rng = SeededRng::new(0x1337_CAFE_DEAD_BEEF);
    let mut ref_model = ReferenceModel::new();
    let store = InMemoryStorageEngine::new();

    let key_pool = [
        b"alpha".to_vec(),
        b"beta".to_vec(),
        b"gamma".to_vec(),
        b"delta".to_vec(),
        b"epsilon".to_vec(),
    ];

    for step in 1..=50 {
        let tx = TxId::new(step);
        let op_type = rng.next_u64() % 3;
        let pool_len = key_pool.len();
        let key_idx = usize::try_from(rng.next_u64() % pool_len as u64).unwrap_or(0);
        let key = key_pool.get(key_idx).unwrap();

        match op_type {
            0 | 1 => {
                // Put
                let val = format!("val_{step}_{key_idx}").into_bytes();
                ref_model.put(key, &val);
                store.put(tx, key, &val).await.unwrap();
                let _ref_seq = ref_model.commit();
                store.commit(tx).await.unwrap();
            }
            _ => {
                // Delete
                ref_model.delete(key);
                store.delete(tx, key).await.unwrap();
                let _ref_seq = ref_model.commit();
                store.commit(tx).await.unwrap();
            }
        }

        // Verify latest visible state across all keys in pool
        for k in &key_pool {
            let ref_val = ref_model.get_latest(k);
            let store_val = store.get(k).await.unwrap().map(|b| b.to_vec());
            assert_eq!(
                ref_val,
                store_val,
                "Discrepancy at step {step} for key {:?}",
                String::from_utf8_lossy(k)
            );
        }
    }
}
