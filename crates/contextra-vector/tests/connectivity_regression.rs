#![allow(clippy::unwrap_used, clippy::expect_used, clippy::needless_range_loop, clippy::indexing_slicing, clippy::cast_possible_truncation)]

use ahash::AHashSet;
use contextra_ports::VectorIndex;
use contextra_types::{DistanceMetric, DocId, TxId};
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};
use std::collections::VecDeque;

/// Deterministischer einfacher Zufallsgenerator für wiederholbare Tests
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_f32(&mut self) -> f32 {
        ((self.next_u64() >> 32) as f32) / 4294967296.0
    }

    fn next_unit_vector(&mut self, dim: usize) -> Vec<f32> {
        let v: Vec<f32> = (0..dim).map(|_| self.next_f32() * 2.0 - 1.0).collect();
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-6 {
            v.into_iter().map(|x| x / norm).collect()
        } else {
            v
        }
    }
}

/// BFS-Orakel auf Layer 0: Prüft, ob alle nicht-gelöschten Knoten vom Entry-Point aus erreichbar sind.
fn verify_layer0_bfs_reachability(index: &HnswIndex) -> bool {
    let ep = match index.entry_point() {
        Some(ep) => ep,
        None => {
            return index.all_doc_ids_from_map().is_empty();
        }
    };

    let active_docs = index.all_doc_ids_from_map();
    if active_docs.is_empty() {
        return true;
    }

    let mut reachable = AHashSet::new();
    let mut queue = VecDeque::new();

    reachable.insert(ep);
    queue.push_back(ep);

    while let Some(current) = queue.pop_front() {
        let neighbors = index.layer0_connections(current);
        for n in neighbors {
            let n_usize = n as usize;
            if reachable.insert(n_usize) {
                queue.push_back(n_usize);
            }
        }
    }

    for &doc_id in &active_docs {
        if let Some(node_idx) = index.node_idx_for_doc(doc_id) {
            if !reachable.contains(&node_idx) {
                return false;
            }
        }
    }

    true
}

#[tokio::test]
async fn test_bfs_layer0_reachability_property() {
    let dim = 8;
    let config = HnswConfigBuilder::new(dim)
        .m(8)
        .ef_construction(32)
        .ef_search(16)
        .rebuild_threshold(0.0) // Deaktiviert automatische Hintergrund-Rebuilds für isolierten deterministischen Test
        .build()
        .expect("config");

    let mut index = HnswIndex::try_new(config).expect("index");
    let mut rng = SimpleRng::new(42);

    let mut inserted_ids = Vec::new();

    // Folge von Inserts
    for i in 1..=40u64 {
        let doc_id = DocId::new(i);
        let vec = rng.next_unit_vector(dim);
        let tx = TxId::new(i);
        index.insert(tx, doc_id, &vec).await.expect("insert");
        index.commit(tx).await.expect("commit");
        inserted_ids.push(doc_id);
    }

    // Löschungen einzelner Knoten über delete + commit
    for (step, &doc_id) in inserted_ids.iter().take(15).enumerate() {
        let tx = TxId::new(100 + step as u64);
        index.delete(tx, doc_id).await.expect("delete");
        index.commit(tx).await.expect("commit");

        assert!(
            verify_layer0_bfs_reachability(&index),
            "BFS-Konnektivität auf Layer 0 nach Löschen von {:?} verletzt!",
            doc_id
        );
        assert!(
            index.check_connectivity().is_ok(),
            "check_connectivity() muss nach Löschung Ok sein!"
        );
    }

    // Zusätzlicher Check mit synchroner Index::remove(&mut self, &u64)
    for id in 16..=20u64 {
        let removed = index.remove(&id).expect("remove");
        assert!(removed, "Document {} should be removed", id);
        assert!(
            verify_layer0_bfs_reachability(&index),
            "BFS-Konnektivität nach remove(&{}) verletzt!",
            id
        );
        assert!(index.check_connectivity().is_ok());
    }

    // Doppeltes remove liefert Ok(false)
    let removed_again = index.remove(&16).expect("double remove");
    assert!(!removed_again, "Double remove should return Ok(false)");
}

#[tokio::test]
async fn test_edge_cases_entry_point_and_hub_deletion() {
    let dim = 8;
    let config = HnswConfigBuilder::new(dim)
        .m(8)
        .ef_construction(32)
        .ef_search(16)
        .rebuild_threshold(0.0)
        .build()
        .expect("config");

    let index = HnswIndex::try_new(config).expect("index");
    let mut rng = SimpleRng::new(123);

    // Randfall 1: Leerer Index
    assert_eq!(index.len().await, 0);
    assert!(index.check_connectivity().is_ok());
    let search_empty = index.search(&[0.1; 8], 5).await.expect("search empty");
    assert!(search_empty.is_empty());

    // Randfall 2: Einzelknoten einfügen und löschen
    let tx1 = TxId::new(1);
    let doc1 = DocId::new(1);
    index.insert(tx1, doc1, &rng.next_unit_vector(dim)).await.expect("insert 1");
    index.commit(tx1).await.expect("commit 1");
    assert_eq!(index.len().await, 1);
    assert!(verify_layer0_bfs_reachability(&index));

    let tx2 = TxId::new(2);
    index.delete(tx2, doc1).await.expect("delete 1");
    index.commit(tx2).await.expect("commit 2");
    assert_eq!(index.len().await, 0);
    assert!(verify_layer0_bfs_reachability(&index));

    // Doppeltes Löschen
    let tx3 = TxId::new(3);
    index.delete(tx3, doc1).await.expect("double delete stage");
    index.commit(tx3).await.expect("double delete commit");
    assert_eq!(index.len().await, 0);

    // Wiederaufbau: 30 Knoten einfügen
    let tx4 = TxId::new(4);
    for i in 10..=40u64 {
        index.insert(tx4, DocId::new(i), &rng.next_unit_vector(dim)).await.expect("insert");
    }
    index.commit(tx4).await.expect("commit 4");

    // Randfall 3: Gezieltes Löschen des aktuellen Entry-Points
    let ep_node_idx = index.entry_point().expect("entry point must exist");
    let mut ep_doc_id = None;
    for &d in &index.all_doc_ids_from_map() {
        if index.node_idx_for_doc(d) == Some(ep_node_idx) {
            ep_doc_id = Some(d);
            break;
        }
    }
    let ep_doc = ep_doc_id.expect("ep doc id found");

    let tx5 = TxId::new(5);
    index.delete(tx5, ep_doc).await.expect("delete entry point");
    index.commit(tx5).await.expect("commit delete entry point");

    assert!(
        verify_layer0_bfs_reachability(&index),
        "BFS-Konnektivität nach Löschung des Entry-Points verletzt!"
    );
    assert_ne!(index.entry_point(), Some(ep_node_idx), "Entry-Point muss neu gewählt worden sein!");
}

#[tokio::test]
async fn test_50_percent_deletion_recall_regression() {
    let dim = 16;
    let num_vectors = 200;
    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(64)
        .ef_search(32)
        .rebuild_threshold(0.0)
        .distance_metric(DistanceMetric::Cosine)
        .build()
        .expect("config");

    let index = HnswIndex::try_new(config).expect("index");
    let mut rng = SimpleRng::new(999);

    let mut dataset = Vec::with_capacity(num_vectors);
    let tx = TxId::new(1);
    for i in 1..=num_vectors {
        let doc_id = DocId::new(i as u64);
        let vec = rng.next_unit_vector(dim);
        dataset.push((doc_id, vec.clone()));
        index.insert(tx, doc_id, &vec).await.expect("insert");
    }
    index.commit(tx).await.expect("commit");

    let mut queries = Vec::new();
    for _ in 0..20 {
        queries.push(rng.next_unit_vector(dim));
    }

    let compute_gt = |active_docs: &AHashSet<DocId>| -> Vec<Vec<DocId>> {
        let mut gt = Vec::new();
        for q in &queries {
            let mut scored: Vec<(DocId, f32)> = dataset
                .iter()
                .filter(|(id, _)| active_docs.contains(id))
                .map(|(id, v)| {
                    let dot: f32 = q.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
                    (*id, 1.0 - dot)
                })
                .collect();
            scored.sort_by(|a, b| a.1.total_cmp(&b.1));
            gt.push(scored.into_iter().take(10).map(|(id, _)| id).collect());
        }
        gt
    };

    let mut active_docs: AHashSet<DocId> = dataset.iter().map(|(id, _)| *id).collect();
    let gt_initial = compute_gt(&active_docs);

    async fn evaluate_recall(index: &HnswIndex, queries: &[Vec<f32>], gt: &[Vec<DocId>]) -> f64 {
        let mut total_hits = 0;
        for (i, q) in queries.iter().enumerate() {
            let res = index.search(q, 10).await.expect("search");
            let expected_set: AHashSet<DocId> = gt[i].iter().copied().collect();
            for r in res {
                if expected_set.contains(&r.doc_id) {
                    total_hits += 1;
                }
            }
        }
        total_hits as f64 / (queries.len() * 10) as f64
    }

    let recall_initial = evaluate_recall(&index, &queries, &gt_initial).await;

    // Löschen von 50% der Knoten in 5 Schüben
    for batch in 0..5 {
        let tx_del = TxId::new(20 + batch);
        let start = batch as usize * 20 + 1;
        let end = start + 20;
        for id in start..end {
            let doc_id = DocId::new(id as u64);
            index.delete(tx_del, doc_id).await.expect("delete");
            active_docs.remove(&doc_id);
        }
        index.commit(tx_del).await.expect("commit delete batch");

        assert!(verify_layer0_bfs_reachability(&index));
        assert!(index.check_connectivity().is_ok());
    }

    let gt_post = compute_gt(&active_docs);
    let recall_post = evaluate_recall(&index, &queries, &gt_post).await;

    println!("Recall vor Löschungen: {:.4}, nach 50% Löschungen: {:.4}", recall_initial, recall_post);
    assert!(
        recall_post >= recall_initial - 0.05,
        "Recall@10 Abfall zu groß: initial {:.4}, post {:.4}",
        recall_initial,
        recall_post
    );
}

#[tokio::test]
async fn test_inv_hnsw_slot_1_no_slot_reuse_before_rebuild() {
    let dim = 8;
    let config = HnswConfigBuilder::new(dim)
        .m(8)
        .ef_construction(32)
        .rebuild_threshold(0.0)
        .build()
        .expect("config");

    let index = HnswIndex::try_new(config).expect("index");
    let mut rng = SimpleRng::new(777);

    // 10 Knoten einfügen
    let tx1 = TxId::new(1);
    for i in 1..=10u64 {
        index.insert(tx1, DocId::new(i), &rng.next_unit_vector(dim)).await.expect("insert");
    }
    index.commit(tx1).await.expect("commit 1");

    let deleted_doc = DocId::new(5);
    let deleted_slot = index.node_idx_for_doc(deleted_doc).expect("slot for doc 5");

    // Löschen von doc 5
    let tx2 = TxId::new(2);
    index.delete(tx2, deleted_doc).await.expect("delete 5");
    index.commit(tx2).await.expect("commit delete 5");

    // Neuen Knoten einfügen VOR Rebuild: Slot darf NICHT sofort wiederverwendet werden!
    let tx3 = TxId::new(3);
    let new_doc = DocId::new(11);
    index.insert(tx3, new_doc, &rng.next_unit_vector(dim)).await.expect("insert 11");
    index.commit(tx3).await.expect("commit 3");

    let new_slot = index.node_idx_for_doc(new_doc).expect("slot for doc 11");
    assert_ne!(
        new_slot, deleted_slot,
        "INV-HNSW-SLOT-1 verletzt: Slot {} wurde vor Rebuild wiederverwendet!",
        deleted_slot
    );

    // Jetzt vollständigen Rebuild ausführen
    index.rebuild_sync().expect("rebuild_sync");

    // Nach Rebuild ist der Slot reorganisiert
    assert!(verify_layer0_bfs_reachability(&index));
}
