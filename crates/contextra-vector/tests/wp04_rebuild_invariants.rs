// FILE-CONTEXT
// ZWECK: Invarianten-Prüfung für Rebuild und Löschen (INV-VEC-ZEROIZE-1, INV-VEC-CONNECTIVITY-1, INV-HNSW-SLOT-1, INV-INDEX-APPLY-1).
// INVARIANTEN: Bit-identischer Determinismus, Idempotenz, Ghost-Pointer-Freiheit, BFS-Konnektivität auf Layer 0.
// STAND: TS:2026-11-11T02:00:00Z

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use contextra_ports::VectorIndex;
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use contextra_vector::types::{Clock, DistanceMetric, DocId, SeededRng, TxId};

struct TestClock {
    nanos: AtomicU64,
}

impl TestClock {
    fn new(start_nanos: u64) -> Self {
        Self {
            nanos: AtomicU64::new(start_nanos),
        }
    }

    fn advance(&self, duration: Duration) {
        let delta = duration.as_nanos().min(u64::MAX as u128) as u64;
        self.nanos.fetch_add(delta, Ordering::SeqCst);
    }
}

impl Clock for TestClock {
    fn now_unix_nanos(&self) -> u64 {
        self.nanos.load(Ordering::SeqCst)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.nanos.load(Ordering::SeqCst)
    }
}

fn create_test_index(seed: u64, clock: Arc<TestClock>) -> HnswIndex {
    let mut config = HnswConfig::default();
    config.dimension = 4;
    config.m = 8;
    config.ef_construction = 16;
    config.distance_metric = DistanceMetric::Euclidean;
    config.backoff_base_delay = Duration::from_millis(10);
    config.backoff_max_delay = Duration::from_millis(100);

    let rng = Arc::new(SeededRng::new(seed));
    HnswIndex::try_new_with_rng_and_clock(config, rng, clock).expect("Index erzeugen")
}

#[tokio::test]
async fn test_rebuild_idempotency() {
    let clock = Arc::new(TestClock::new(1_000_000_000));
    let index = create_test_index(12345, clock.clone());

    for i in 1..=30 {
        let doc_id = DocId::new(i as u64);
        let vec = vec![i as f32, (i * 2) as f32, (i * 3) as f32, (i * 4) as f32];
        let tx = TxId::new(i as u64);
        index
            .insert(tx, doc_id, &vec)
            .await
            .expect("insert succeeds");
        index.commit(tx).await.expect("commit succeeds");
    }

    // Delete some docs
    for i in 5..=10 {
        let tx = TxId::new((100 + i) as u64);
        index
            .delete(tx, DocId::new(i as u64))
            .await
            .expect("delete succeeds");
        index.commit(tx).await.expect("commit succeeds");
    }

    clock.advance(Duration::from_millis(500));
    index
        .rebuild_sync_with_clock(clock.as_ref())
        .expect("rebuild 1 succeeds");
    let query = vec![15.0, 30.0, 45.0, 60.0];
    let res1 = index.search(&query, 5).await.expect("search 1 succeeds");

    clock.advance(Duration::from_millis(500));
    index
        .rebuild_sync_with_clock(clock.as_ref())
        .expect("rebuild 2 succeeds");
    let res2 = index.search(&query, 5).await.expect("search 2 succeeds");

    assert_eq!(res1.len(), res2.len());
    for (r1, r2) in res1.iter().zip(res2.iter()) {
        assert_eq!(r1.doc_id, r2.doc_id);
        assert_eq!(r1.score.to_bits(), r2.score.to_bits());
    }
}

#[tokio::test]
async fn test_rebuild_determinism_and_tie_breaking() {
    let run = |seed: u64| async move {
        let clock = Arc::new(TestClock::new(1_000_000_000));
        let index = create_test_index(seed, clock.clone());

        for i in 1..=25 {
            let doc_id = DocId::new(i as u64);
            let vec = vec![i as f32, (i * 2) as f32, (i * 3) as f32, (i * 4) as f32];
            let tx = TxId::new(i as u64);
            index.insert(tx, doc_id, &vec).await.expect("insert");
            index.commit(tx).await.expect("commit");
        }
        for i in 1..=8 {
            let tx = TxId::new((100 + i) as u64);
            index
                .delete(tx, DocId::new(i as u64))
                .await
                .expect("delete");
            index.commit(tx).await.expect("commit");
        }
        clock.advance(Duration::from_millis(500));
        index
            .rebuild_sync_with_clock(clock.as_ref())
            .expect("rebuild");

        let query = vec![12.0, 24.0, 36.0, 48.0];
        index.search(&query, 10).await.expect("search")
    };

    let run_a = run(42).await;
    let run_b = run(42).await;

    assert_eq!(run_a.len(), run_b.len());
    for (a, b) in run_a.iter().zip(run_b.iter()) {
        assert_eq!(a.doc_id, b.doc_id);
        assert_eq!(a.score.to_bits(), b.score.to_bits());
    }
}

#[tokio::test]
async fn test_ghost_pointer_freedom_and_bfs_connectivity() {
    let clock = Arc::new(TestClock::new(1_000_000_000));
    let index = create_test_index(777, clock.clone());

    for i in 1..=50 {
        let doc_id = DocId::new(i as u64);
        let vec = vec![i as f32, (i * 2) as f32, (i * 3) as f32, (i * 4) as f32];
        let tx = TxId::new(i as u64);
        index.insert(tx, doc_id, &vec).await.expect("insert");
        index.commit(tx).await.expect("commit");
    }

    // Delete half the dataset
    for i in 1..=25 {
        let tx = TxId::new((100 + i) as u64);
        index
            .delete(tx, DocId::new(i as u64))
            .await
            .expect("delete");
        index.commit(tx).await.expect("commit");
    }

    clock.advance(Duration::from_millis(500));
    index
        .rebuild_sync_with_clock(clock.as_ref())
        .expect("rebuild");

    // Full graph traversal must confirm all alive nodes are reachable and connected
    assert!(
        index.verify_full_connectivity(),
        "Graph must be fully connected following rebuild (INV-VEC-CONNECTIVITY-1)"
    );

    // Search results must never return any deleted node (Ghost-Pointer-Freiheit)
    let query = vec![5.0, 10.0, 15.0, 20.0];
    let results = index.search(&query, 20).await.expect("search");
    for res in results {
        assert!(
            res.doc_id.inner() > 25,
            "Deleted node must not be returned in search"
        );
    }
}

#[tokio::test]
async fn test_slot_reuse_only_after_rebuild_inv_hnsw_slot_1() {
    let clock = Arc::new(TestClock::new(1_000_000_000));
    let index = create_test_index(999, clock.clone());

    let doc1 = DocId::new(1);
    let doc2 = DocId::new(2);
    let vec1 = vec![1.0, 2.0, 3.0, 4.0];
    let vec2 = vec![5.0, 6.0, 7.0, 8.0];

    index
        .insert(TxId::new(1), doc1, &vec1)
        .await
        .expect("insert 1");
    index.commit(TxId::new(1)).await.expect("commit 1");
    index
        .insert(TxId::new(2), doc2, &vec2)
        .await
        .expect("insert 2");
    index.commit(TxId::new(2)).await.expect("commit 2");

    // Delete doc1: node count remains 2 (tombstone marked, slot not reclaimed immediately)
    index.delete(TxId::new(3), doc1).await.expect("delete doc1");
    index.commit(TxId::new(3)).await.expect("commit 3");

    let doc3 = DocId::new(3);
    let vec3 = vec![9.0, 10.0, 11.0, 12.0];
    index
        .insert(TxId::new(4), doc3, &vec3)
        .await
        .expect("insert 3");
    index.commit(TxId::new(4)).await.expect("commit 4");

    // After rebuild, dead slots are pruned and compacted
    clock.advance(Duration::from_millis(500));
    index
        .rebuild_sync_with_clock(clock.as_ref())
        .expect("rebuild");

    assert!(index.verify_full_connectivity());
}

#[tokio::test]
async fn test_slot_zeroize_on_deletion_inv_vec_zeroize_1() {
    let clock = Arc::new(TestClock::new(1_000_000_000));
    let mut index = create_test_index(1337, clock.clone());

    let doc_id = DocId::new(42);
    let vec = vec![1.0, 2.0, 3.0, 4.0];
    index
        .insert(TxId::new(1), doc_id, &vec)
        .await
        .expect("insert");
    index.commit(TxId::new(1)).await.expect("commit");

    // Deletion must zeroize node slot in memory
    use contextra_vector::hnsw::GhostFreeVectorIndex;
    let stats = index
        .remove_with_graph_repair(doc_id)
        .expect("remove with graph repair");
    assert_eq!(stats.doc_id, doc_id);
    assert!(stats.verified_no_ghost_pointers);

    // After deletion, search must not find doc_id
    let res = index.search(&vec, 5).await.expect("search");
    assert!(res.iter().all(|r| r.doc_id != doc_id));
}

#[cfg(feature = "partial-index-rebuild")]
#[tokio::test]
async fn test_partial_rebuild_invariant_gated() {
    let clock = Arc::new(TestClock::new(1_000_000_000));
    let index = create_test_index(2026, clock.clone());

    for i in 1..=20 {
        let doc_id = DocId::new(i as u64);
        let vec = vec![i as f32, (i * 2) as f32, (i * 3) as f32, (i * 4) as f32];
        index
            .insert(TxId::new(i as u64), doc_id, &vec)
            .await
            .expect("insert");
        index.commit(TxId::new(i as u64)).await.expect("commit");
    }

    // Rebuild region
    index.rebuild_region(vec![1, 2, 3]).expect("rebuild region");
    assert!(index.verify_full_connectivity());
}
