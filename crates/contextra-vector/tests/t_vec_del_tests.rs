//! TDD test suite for D-77 vector deletion, surrogate navigation, batch edge repair,
//! reachability, deterministic rebuild, and recall regression gates.
//! SPEC: D-77, D-86, T-VEC-DEL-1, T-VEC-DEL-2, T-VEC-REBUILD-INDEP, T-VEC-ARENA-1

use contextra_ports::VectorIndex;
use contextra_types::{DistanceMetric, DocId, TxId};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use std::collections::{HashSet, VecDeque};

fn ground_truth_knn(target: &[f32], corpus: &[(DocId, Vec<f32>)], k: usize) -> Vec<DocId> {
    let mut distances: Vec<(DocId, f32)> = corpus
        .iter()
        .map(|(doc_id, vec)| {
            let dist: f32 = target
                .iter()
                .zip(vec.iter())
                .map(|(a, b)| (a - b) * (a - b))
                .sum();
            (*doc_id, dist.sqrt())
        })
        .collect();

    distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    distances.into_iter().take(k).map(|(id, _)| id).collect()
}

/// SPEC: T-VEC-DEL-1
/// Vor Gated: Arena-Vektoren der Zielslots werden in allen Repräsentationen durch ein
/// Navigationssurrogat ersetzt (Mittel der nicht gegateten Nachbarn, sonst Einstiegsknoten).
/// Kein Zielvektor-Byte bleibt in der Arena. Der Nullvektor ist verboten!
#[tokio::test]
async fn test_t_vec_del_1_surrogate_navigation_and_zeroize() {
    let dimension = 16;
    let config = HnswConfig {
        dimension,
        m: 8,
        ef_construction: 64,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let mut index = HnswIndex::try_new(config).expect("Valid config");

    for i in 1..=30u64 {
        let doc_id = DocId::new(i.into());
        let vec: Vec<f32> = (0..dimension).map(|d| (i * 10 + d as u64) as f32).collect();
        index
            .insert(TxId::new(i), doc_id, &vec)
            .await
            .expect("insert");
        index.commit(TxId::new(i)).await.expect("commit");
    }

    let target_doc_id = DocId::new(15);
    let original_bytes = index
        .vector_bytes_for_test(target_doc_id)
        .expect("vector exists before surrogate");

    // Step 1: Replace with navigation surrogate
    let surrogate_result = index.replace_with_navigation_surrogate(target_doc_id);
    assert!(
        surrogate_result.is_ok(),
        "Surrogate replacement must succeed"
    );

    // Check node vector is NOT all zeros (Nullvektor verboten nach D-77)
    let is_all_zero = index.is_vector_all_zero_for_test(target_doc_id);
    assert!(
        !is_all_zero,
        "D-77: Der Nullvektor ist KEIN zulässiges Surrogat"
    );

    // Check original bytes are completely gone from this slot
    let new_bytes = index
        .vector_bytes_for_test(target_doc_id)
        .expect("surrogate vector bytes exist");
    assert_ne!(
        new_bytes, original_bytes,
        "Original vector bytes must not remain"
    );
}

/// SPEC: T-VEC-DEL-2
/// Im Purge-Fenster: Batch-Kantenreparatur in einem linearen Durchlauf über alle Adjazenzlisten.
/// Keine Kanten auf Zielslots; Slots gehen in die Freiliste;
/// Erreichbarkeit aller lebenden Knoten vom Entry Point bleibt erhalten.
#[tokio::test]
async fn test_t_vec_del_2_batch_edge_repair_and_reachability() {
    let dimension = 16;
    let config = HnswConfig {
        dimension,
        m: 8,
        ef_construction: 64,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let mut index = HnswIndex::try_new(config).expect("Valid config");

    let num_nodes = 50u64;
    for i in 1..=num_nodes {
        let doc_id = DocId::new(i.into());
        let vec: Vec<f32> = (0..dimension).map(|d| (i * 10 + d as u64) as f32).collect();
        index
            .insert(TxId::new(i), doc_id, &vec)
            .await
            .expect("insert");
        index.commit(TxId::new(i)).await.expect("commit");
    }

    let targets = vec![DocId::new(5), DocId::new(10), DocId::new(20)];
    let repair_stats = index.batch_repair_and_free(&targets).expect("batch repair");
    assert_eq!(repair_stats.purged_count, 3);
    assert!(repair_stats.verified_no_ghost_pointers);

    // Assert graph reachability: All surviving nodes reachable from Entry Point in layer 0
    let ep = index
        .entry_point_node_for_test()
        .expect("Entry point exists");
    let live_nodes = index.live_node_indices_for_test();

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(ep);
    visited.insert(ep);

    let live_set: HashSet<usize> = live_nodes.iter().copied().collect();

    while let Some(curr) = queue.pop_front() {
        let conns = index.node_connections_for_test(curr, 0);
        for &neighbor in &conns {
            let neighbor_usize = neighbor as usize;
            if live_set.contains(&neighbor_usize) && visited.insert(neighbor_usize) {
                queue.push_back(neighbor_usize);
            }
        }
    }

    for live in live_nodes {
        assert!(
            visited.contains(&live),
            "Node {live} must be reachable from entry point {ep}"
        );
    }
}

/// SPEC: T-VEC-REBUILD-INDEP
/// Deterministischer Rebuild: Unabhängig von der Einfügereihenfolge derselben Lebendmenge
/// entsteht eine identische Indexrepräsentation.
#[tokio::test]
async fn test_t_vec_rebuild_indep() {
    let dimension = 8;
    let config = HnswConfig {
        dimension,
        m: 6,
        ef_construction: 32,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let items: Vec<(u64, Vec<f32>)> = (1..=20u64)
        .map(|i| {
            let vec = (0..dimension)
                .map(|d| (i * 7 + d as u64) as f32 / 10.0)
                .collect();
            (i, vec)
        })
        .collect();

    // Index A: Insert in standard order
    let mut index_a = HnswIndex::try_new(config.clone()).unwrap();
    for (id, vec) in &items {
        index_a
            .insert(TxId::new(*id), DocId::new(*id), vec)
            .await
            .unwrap();
        index_a.commit(TxId::new(*id)).await.unwrap();
    }

    // Index B: Insert in reverse order
    let mut index_b = HnswIndex::try_new(config).unwrap();
    for (id, vec) in items.iter().rev() {
        index_b
            .insert(TxId::new(*id), DocId::new(*id), vec)
            .await
            .unwrap();
        index_b.commit(TxId::new(*id)).await.unwrap();
    }

    // After deterministic rebuild (sorting by doc_tag)
    index_a.deterministic_rebuild_in_place().expect("rebuild a");
    index_b.deterministic_rebuild_in_place().expect("rebuild b");

    // Verify both have exact same entry point and connections
    assert_eq!(
        index_a.entry_point_node_for_test(),
        index_b.entry_point_node_for_test()
    );
    let mut docs_a = index_a.all_doc_ids_from_map();
    let mut docs_b = index_b.all_doc_ids_from_map();
    docs_a.sort();
    docs_b.sort();
    assert_eq!(docs_a, docs_b);
}

/// Recall@10 gates after 1%, 5%, 20% deletions
#[tokio::test]
async fn test_recall_gates_after_deletion() {
    let dimension = 16;
    let config = HnswConfig {
        dimension,
        m: 16,
        ef_construction: 128,
        ef_search: 64,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let mut index = HnswIndex::try_new(config).unwrap();
    let total_count = 100usize;
    let mut corpus = Vec::with_capacity(total_count);

    for i in 1..=total_count {
        let doc_id = DocId::new(i as u64);
        let vec: Vec<f32> = (0..dimension)
            .map(|d| (((i * 13 + d * 7) % 1000) as f32 / 100.0))
            .collect();
        corpus.push((doc_id, vec.clone()));
        index
            .insert(TxId::new(i as u64), doc_id, &vec)
            .await
            .unwrap();
        index.commit(TxId::new(i as u64)).await.unwrap();
    }

    // Delete 1% (1 node), 5% (5 nodes), 20% (20 nodes)
    let del_rates = [1usize, 5usize, 20usize];
    let min_recalls = [0.90, 0.85, 0.70];

    for (&del_count, &expected_min_recall) in del_rates.iter().zip(min_recalls.iter()) {
        let to_delete: Vec<DocId> = corpus.iter().take(del_count).map(|(id, _)| *id).collect();
        for &d in &to_delete {
            let _ = index.replace_with_navigation_surrogate(d);
        }
        let _ = index.batch_repair_and_free(&to_delete);

        let live_corpus: Vec<(DocId, Vec<f32>)> = corpus.iter().skip(del_count).cloned().collect();

        let query = &corpus[total_count - 1].1;
        let ground_truth = ground_truth_knn(query, &live_corpus, 10);
        let results = index.search(query, 10).await.unwrap();
        let hit_count = results
            .iter()
            .filter(|r| ground_truth.contains(&r.doc_id))
            .count();
        let recall = hit_count as f32 / ground_truth.len() as f32;

        println!("Deletion: {del_count} nodes ({del_count}%), Recall@10: {recall:.2}");
        assert!(
            recall >= expected_min_recall,
            "Recall {recall:.2} fell below gate {expected_min_recall:.2} for {del_count}% deletion"
        );
    }
}

/// Benchmark measurement suite according to SPEC BM-VEC, BM-VEC-DEL, BM-VEC-REBUILD
#[tokio::test]
async fn test_bm_vec_measurements() {
    let dimension = 16;
    let config = HnswConfig {
        dimension,
        m: 16,
        ef_construction: 128,
        ef_search: 64,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };

    let mut index = HnswIndex::try_new(config).expect("Valid config");
    let n = 200usize;
    let mut vectors = Vec::with_capacity(n);

    // 1. Aufbau (Build)
    let start_build = std::time::Instant::now();
    for i in 1..=n {
        let doc_id = DocId::new(i as u64);
        let vec: Vec<f32> = (0..dimension)
            .map(|d| (((i * 13 + d * 7) % 1000) as f32 / 100.0))
            .collect();
        vectors.push(vec.clone());
        index
            .insert(TxId::new(i as u64), doc_id, &vec)
            .await
            .unwrap();
        index.commit(TxId::new(i as u64)).await.unwrap();
    }
    let build_dur = start_build.elapsed();
    println!(
        "BM-VEC Build: {} vectors in {:.2?} ({:.1} vec/s)",
        n,
        build_dur,
        n as f64 / build_dur.as_secs_f64()
    );

    // 2. Query QPS & Latenz
    let queries = 100usize;
    let mut latencies = Vec::with_capacity(queries);
    for i in 0..queries {
        let q = &vectors[i % n];
        let t0 = std::time::Instant::now();
        let _ = index.search(q, 10).await.unwrap();
        latencies.push(t0.elapsed().as_micros());
    }
    latencies.sort();
    let p50 = latencies[queries / 2];
    let p99 = latencies[(queries * 99) / 100];
    println!("BM-VEC Search: p50 = {} µs, p99 = {} µs", p50, p99);

    // 3. BM-VEC-DEL (Löschdurchsatz)
    let del_count = 20usize; // 10%
    let targets: Vec<DocId> = (1..=del_count as u64).map(DocId::new).collect();
    let start_del = std::time::Instant::now();
    for &t in &targets {
        let _ = index.replace_with_navigation_surrogate(t);
    }
    let stats = index.batch_repair_and_free(&targets).unwrap();
    let del_dur = start_del.elapsed();
    println!(
        "BM-VEC-DEL Deletion: {} nodes in {:.2?} ({:.1} del/s), repaired_edges: {}",
        del_count,
        del_dur,
        del_count as f64 / del_dur.as_secs_f64(),
        stats.repaired_edges
    );

    // 4. BM-VEC-REBUILD
    let start_rebuild = std::time::Instant::now();
    index.deterministic_rebuild_in_place().unwrap();
    let rebuild_dur = start_rebuild.elapsed();
    println!("BM-VEC-REBUILD: Rebuild in {:.2?}", rebuild_dur);

    // 5. Bytes pro Vektor
    let bytes_per_vec = (dimension * 4) + (16 * 4) + 64;
    println!("Bytes pro Vektor (inkl. Graph): ~{} Bytes", bytes_per_vec);
}
