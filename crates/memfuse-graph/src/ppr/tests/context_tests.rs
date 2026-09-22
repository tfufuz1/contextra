use super::super::*;
use crate::csr::CsrGraph;
use memfuse_core::{DocId, Edge, Entity, EntityId, GraphIndex, MemFuseError, TxId};
use std::sync::Arc;

struct LogCaptureLayer(std::sync::Arc<std::sync::Mutex<Vec<String>>>);
impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for LogCaptureLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = StringVisitor(String::new());
        event.record(&mut visitor);
        self.0.lock().unwrap().push(visitor.0);
    }
}
struct StringVisitor(String);
impl tracing::field::Visit for StringVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        use std::fmt::Write;
        write!(self.0, "{}={:?} ", field.name(), value).ok();
    }
}

#[tokio::test]
async fn test_ppr_warn_on_non_convergence_suppressible() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for i in 1..=5 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("N{i}"), "Node"))
            .await
            .unwrap(); // unwrap
    }

    for i in 1..=5 {
        let next = if i == 5 { 1 } else { i + 1 };
        graph
            .add_edge(tx, Edge::new(EntityId::new(i), EntityId::new(next), "next"))
            .await
            .unwrap(); // unwrap
    }
    graph.commit(tx).await.unwrap(); // unwrap

    // Config with max_iterations: 1 and warn_on_non_convergence: false
    let config = PprConfig {
        damping_factor: 0.85,
        max_iterations: 1,
        convergence_epsilon: 1e-12,
        algorithm: PprAlgorithm::Auto,
        warn_on_non_convergence: false,
    };

    let seed = EntityId::new(1);
    let results = graph
        .personalized_page_rank(&[seed], &config)
        .await
        .unwrap(); // unwrap

    assert!(
            !results.is_empty(),
            "Calculation must return best-effort result without error or panic when warn_on_non_convergence is false"
        );
}

#[tokio::test]
async fn test_ppr_non_convergence_logs_warning_and_returns_best_effort() {
    use tracing_subscriber::layer::SubscriberExt;

    let logs = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let capture_layer = LogCaptureLayer(logs.clone());
    let subscriber = tracing_subscriber::registry().with(capture_layer);
    let _guard = tracing::subscriber::set_default(subscriber);

    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for i in 1..=5 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("N{i}"), "Node"))
            .await
            .unwrap(); // unwrap allowed
    }

    // Ring 1 -> 2 -> 3 -> 4 -> 5 -> 1
    for i in 1..=5 {
        let next = if i == 5 { 1 } else { i + 1 };
        graph
            .add_edge(tx, Edge::new(EntityId::new(i), EntityId::new(next), "next"))
            .await
            .unwrap(); // unwrap allowed
    }
    graph.commit(tx).await.unwrap(); // unwrap allowed

    // Set max_iterations very low (2) and convergence_epsilon very small (1e-12)
    // so that power iteration cannot reach convergence in 2 iterations
    let config = PprConfig {
        damping_factor: 0.85,
        max_iterations: 2,
        convergence_epsilon: 1e-12,
        algorithm: PprAlgorithm::Auto,
        warn_on_non_convergence: true,
    };

    let seed = EntityId::new(1);
    let results = graph
        .personalized_page_rank(&[seed], &config)
        .await
        .unwrap(); // unwrap allowed

    assert!(
        !results.is_empty(),
        "Best-effort results must be returned on non-convergence"
    );

    let captured = logs.lock().unwrap(); // unwrap allowed
    let warning_found = captured.iter().any(|msg| {
            msg.contains("Personalized PageRank power iteration reached maximum iterations without reaching convergence")
                && msg.contains("max_iterations=2")
                && msg.contains("convergence_epsilon=")
        });

    assert!(
        warning_found,
        "Expected structured warning log on PPR non-convergence, got logs: {:?}",
        *captured
    );
}

#[tokio::test]
async fn test_ppr_pathological_max_iterations_ceiling() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for i in 1..=4 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("N{i}"), "Node"))
            .await
            .unwrap(); // unwrap allowed
    }

    // Cycle 1 <-> 2 <-> 3 <-> 4
    graph
        .add_bidirectional(tx, EntityId::new(1), EntityId::new(2), "edge")
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_bidirectional(tx, EntityId::new(2), EntityId::new(3), "edge")
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_bidirectional(tx, EntityId::new(3), EntityId::new(4), "edge")
        .await
        .unwrap(); // unwrap allowed
    graph.commit(tx).await.unwrap(); // unwrap allowed

    let config = PprConfig {
        damping_factor: 0.85,
        max_iterations: 5,          // Capped to 5 iterations
        convergence_epsilon: 1e-15, // Unreachable tolerance forces iter cap
        algorithm: PprAlgorithm::Auto,
        warn_on_non_convergence: true,
    };

    let start_time = std::time::Instant::now();
    let results = graph
        .personalized_page_rank(&[EntityId::new(1)], &config)
        .await
        .unwrap(); // unwrap allowed
    let elapsed = start_time.elapsed();

    assert!(!results.is_empty());
    assert!(
        elapsed.as_millis() < 500,
        "Max iterations ceiling must terminate execution promptly without hanging"
    );
}

#[tokio::test]
async fn test_ppr_ignores_deleted_entities() {
    use memfuse_core::StorageEngine;
    use memfuse_store::{LsmConfig, LsmStorage};
    use std::sync::Arc;

    let dir = tempfile::tempdir().unwrap();
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(),
    );
    let graph = CsrGraph::with_storage(storage.clone());
    let tx = TxId::new(1);

    let id_a = EntityId::new(1);
    let id_b = EntityId::new(2);
    let id_c = EntityId::new(3);

    graph
        .add_entity(tx, Entity::new(id_a, "Node A", "Type"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(id_b, "Node B", "Type"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(id_c, "Node C", "Type"))
        .await
        .unwrap();

    // A -> B -> C
    graph
        .add_edge(tx, Edge::new(id_a, id_b, "link"))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(id_b, id_c, "link"))
        .await
        .unwrap();
    graph.commit(tx).await.unwrap();

    // Mark Node B (id_b=2) as deleted in storage
    let tx_del = TxId::new(2);
    let tombstone_key = format!("graph:entity:deleted:{}", id_b.0);
    storage
        .put(tx_del, tombstone_key.as_bytes(), b"deleted")
        .await
        .unwrap();
    storage.commit(tx_del).await.unwrap();

    let config = PprConfig::default();
    let results = graph
        .personalized_page_rank(&[id_a], &config)
        .await
        .unwrap();

    // Node B must NOT be present in results
    let ids: Vec<_> = results.iter().map(|(id, _)| *id).collect();
    assert!(
        !ids.contains(&id_b),
        "Deleted node B must be filtered out of PPR results"
    );

    // Total rank mass of live nodes must conserve to 1.0
    let total_mass: f32 = results.iter().map(|(_, r)| r).sum();
    assert!(
        (total_mass - 1.0).abs() < 1e-4,
        "Total rank mass must be conserved and re-normalized to 1.0, got {total_mass}"
    );
}

#[tokio::test]
async fn test_ppr_phantom_node_no_rank_mass() {
    use memfuse_core::StorageEngine;
    use memfuse_store::{LsmConfig, LsmStorage};
    use std::sync::Arc;

    let dir = tempfile::tempdir().unwrap();
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(),
    );
    let graph = CsrGraph::with_storage(storage.clone());
    let tx = TxId::new(1);

    let id_a = EntityId::new(1);
    let id_phantom_b = EntityId::new(2);
    let id_c = EntityId::new(3);
    let id_d = EntityId::new(4);

    graph
        .add_entity(tx, Entity::new(id_a, "Node A", "Type"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(id_phantom_b, "Phantom B", "Type"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(id_c, "Node C", "Type"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(id_d, "Node D", "Type"))
        .await
        .unwrap();

    // A -> B, C -> B, D -> B (Phantom B has many incoming edges)
    graph
        .add_edge(tx, Edge::new(id_a, id_phantom_b, "in1"))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(id_c, id_phantom_b, "in2"))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(id_d, id_phantom_b, "in3"))
        .await
        .unwrap();
    // A -> C
    graph
        .add_edge(tx, Edge::new(id_a, id_c, "link"))
        .await
        .unwrap();
    graph.commit(tx).await.unwrap();

    // Mark Node B as deleted
    let tx_del = TxId::new(2);
    let tombstone_key = format!("graph:entity:deleted:{}", id_phantom_b.0);
    storage
        .put(tx_del, tombstone_key.as_bytes(), b"deleted")
        .await
        .unwrap();
    storage.commit(tx_del).await.unwrap();

    let config = PprConfig::default();
    let results = graph
        .personalized_page_rank(&[id_a, id_c, id_d], &config)
        .await
        .unwrap();

    // Deleted phantom node B must receive no rank and must not appear in results
    assert!(
        !results.iter().any(|(id, _)| *id == id_phantom_b),
        "Phantom node B must not appear in PPR results despite many incoming edges"
    );

    let total_mass: f32 = results.iter().map(|(_, r)| r).sum();
    assert!(
        (total_mass - 1.0).abs() < 1e-4,
        "Rank mass must conserve to 1.0 across remaining live nodes, got {total_mass}"
    );
}

#[test]
fn test_deleted_view_methods() {
    let empty_view = DeletedView::empty();
    assert!(empty_view.is_empty());
    assert_eq!(empty_view.len(), 0);
    assert!(!empty_view.contains(1));

    let mut set = HashSet::new();
    set.insert(5);
    set.insert(10);
    let view = DeletedView::from_nodes(set);
    assert!(!view.is_empty());
    assert_eq!(view.len(), 2);
    assert!(view.contains(5));
    assert!(view.contains(10));
    assert!(!view.contains(1));
}

#[tokio::test]
async fn test_deleted_view_tombstone_filtering_all_entry_points() {
    use memfuse_core::StorageEngine;
    use std::sync::Arc;
    let temp_dir = tempfile::tempdir().unwrap();
    let config = memfuse_store::LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(memfuse_store::LsmStorage::new(config).await.unwrap());
    let graph = CsrGraph::with_storage(storage.clone());
    let tx = TxId::new(1);

    let id_a = EntityId::new(10);
    let id_b = EntityId::new(20);

    graph
        .add_entity(tx, Entity::new(id_a, "Node A", "Type"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(id_b, "Node B", "Type"))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(id_a, id_b, "link"))
        .await
        .unwrap();
    graph.commit(tx).await.unwrap();

    // Mark Node B as deleted in storage
    let tx_del = TxId::new(2);
    let tombstone_key = format!("graph:entity:deleted:{}", id_b.0);
    storage
        .put(tx_del, tombstone_key.as_bytes(), b"deleted")
        .await
        .unwrap();
    storage.commit(tx_del).await.unwrap();

    let config = PprConfig::default();

    // 1. Check via personalized_page_rank (trait method)
    let res_trait = graph
        .personalized_page_rank(&[id_a], &config)
        .await
        .unwrap();
    assert!(
        !res_trait.iter().any(|(id, _)| *id == id_b),
        "Deleted node B must not appear in personalized_page_rank result"
    );

    // 2. Check via personalized_page_rank_with_context_async
    let mut ctx = PprContext::new();
    let res_async = graph
        .personalized_page_rank_with_context_async(&[id_a], &config, &mut ctx)
        .await;
    assert!(
        !res_async.iter().any(|(id, _)| *id == id_b),
        "Deleted node B must not appear in personalized_page_rank_with_context_async result"
    );

    // 3. Parity assertion: Both entry points MUST produce identical results
    assert_eq!(res_trait.len(), res_async.len());
    for (a, b) in res_trait.iter().zip(res_async.iter()) {
        assert_eq!(a.0, b.0);
        assert_eq!(
            a.1.to_bits(),
            b.1.to_bits(),
            "PPR scores must be bit-identical between trait method and async context method"
        );
    }
}

#[tokio::test]
async fn test_out_weight_sums_incremental_equivalence() {
    let graph = Arc::new(CsrGraph::new());
    let tx = TxId::new(1);

    for i in 1..=10 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("N{i}"), "Type"))
            .await
            .unwrap();
    }

    for i in 1..=9 {
        GraphIndex::add_edge(
            graph.as_ref(),
            tx,
            Edge::new(EntityId::new(i), EntityId::new(i + 1), "edge").with_weight(i as f32 * 0.5),
        )
        .await
        .unwrap();
    }
    graph.commit(tx).await.unwrap();

    // Check out_weight_sums precomputed vs recomputed before compact
    {
        let mut inner = graph.inner_write();
        let precomputed = inner.out_weight_sums.clone();
        let num_nodes = inner.reverse_map.len();
        for i in 0..num_nodes {
            inner.recompute_node_out_weight_sum(i);
        }
        let recomputed = inner.out_weight_sums.clone();
        assert_eq!(
            precomputed, recomputed,
            "Incremental out_weight_sums must equal recomputed sums before compact()"
        );
    }

    // Compact and re-verify
    graph.compact();

    {
        let mut inner = graph.inner_write();
        let precomputed = inner.out_weight_sums.clone();
        let num_nodes = inner.reverse_map.len();
        for i in 0..num_nodes {
            inner.recompute_node_out_weight_sum(i);
        }
        let recomputed = inner.out_weight_sums.clone();
        assert_eq!(
            precomputed, recomputed,
            "Incremental out_weight_sums must equal recomputed sums after compact()"
        );
    }
}

#[tokio::test]
async fn test_compact_async_numerical_equivalence() {
    let graph = Arc::new(CsrGraph::new());
    let tx = TxId::new(1);

    for i in 1..=5 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("N{i}"), "Node"))
            .await
            .unwrap();
    }
    for i in 1..=4 {
        GraphIndex::add_edge(
            graph.as_ref(),
            tx,
            Edge::new(EntityId::new(i), EntityId::new(i + 1), "link").with_weight(0.8),
        )
        .await
        .unwrap();
    }
    graph.commit(tx).await.unwrap();

    let config = PprConfig::default();
    let seed = EntityId::new(1);

    // Result via compact_async
    let res_async = graph
        .personalized_page_rank(&[seed], &config)
        .await
        .unwrap();

    // Direct computation without compact_async (compact already performed by res_async)
    let deleted_view = graph.deleted_view().await;
    let inner = graph.inner_read();
    let res_direct = crate::ppr::compute_ppr(&inner, &[seed], &config, &deleted_view);

    assert_eq!(res_async.len(), res_direct.len());
    for (a, b) in res_async.iter().zip(res_direct.iter()) {
        assert_eq!(a.0, b.0);
        assert_eq!(
            a.1.to_bits(),
            b.1.to_bits(),
            "compact_async PPR scores must be bit-identical to direct compute_ppr"
        );
    }
}

#[tokio::test]
async fn test_forward_push_vs_dense_numerical_equivalence_and_shadow_mode() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    // Build a graph with 20 nodes and complex connections
    for i in 1..=20 {
        graph
            .add_entity(
                tx,
                Entity::new(EntityId::new(i), format!("Node{i}"), "Test"),
            )
            .await
            .unwrap();
    }

    for i in 1..15 {
        graph
            .add_edge(
                tx,
                Edge::new(EntityId::new(i), EntityId::new(i + 1), "next").with_weight(1.0),
            )
            .await
            .unwrap();
        graph
            .add_edge(
                tx,
                Edge::new(EntityId::new(i), EntityId::new((i * 2) % 20 + 1), "jump")
                    .with_weight(0.5),
            )
            .await
            .unwrap();
    }
    graph.commit(tx).await.unwrap();

    let seed = EntityId::new(1);

    let cfg_fp = PprConfig {
        damping_factor: 0.85,
        max_iterations: 100,
        convergence_epsilon: 1e-6,
        algorithm: PprAlgorithm::ForwardPush,
        warn_on_non_convergence: true,
    };

    let cfg_dense = PprConfig {
        damping_factor: 0.85,
        max_iterations: 100,
        convergence_epsilon: 1e-6,
        algorithm: PprAlgorithm::DensePowerIteration,
        warn_on_non_convergence: true,
    };

    let cfg_shadow = PprConfig {
        damping_factor: 0.85,
        max_iterations: 100,
        convergence_epsilon: 1e-6,
        algorithm: PprAlgorithm::ShadowMode,
        warn_on_non_convergence: true,
    };

    let res_fp = graph
        .personalized_page_rank(&[seed], &cfg_fp)
        .await
        .unwrap();
    let res_dense = graph
        .personalized_page_rank(&[seed], &cfg_dense)
        .await
        .unwrap();
    let res_shadow = graph
        .personalized_page_rank(&[seed], &cfg_shadow)
        .await
        .unwrap();

    assert!(!res_fp.is_empty());
    assert_eq!(res_dense.len(), res_shadow.len());

    let dense_map: std::collections::HashMap<EntityId, f32> = res_dense.into_iter().collect();
    let fp_map: std::collections::HashMap<EntityId, f32> = res_fp.into_iter().collect();

    // High relevance nodes near seed should match within convergence_epsilon tolerance
    for (id, fp_score) in fp_map {
        if let Some(&dense_score) = dense_map.get(&id) {
            assert!(
                    (fp_score - dense_score).abs() < 5e-3,
                    "Forward-push score {fp_score} for entity {id:?} deviates from dense score {dense_score}"
                );
        }
    }
}

#[tokio::test]
async fn test_forward_push_100_runs_bit_identical_determinism() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for i in 1..=10 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("N{i}"), "Node"))
            .await
            .unwrap();
    }
    for i in 1..10 {
        graph
            .add_edge(
                tx,
                Edge::new(EntityId::new(i), EntityId::new(i + 1), "link").with_weight(1.0),
            )
            .await
            .unwrap();
        graph
            .add_edge(
                tx,
                Edge::new(EntityId::new(i + 1), EntityId::new(i), "back").with_weight(0.5),
            )
            .await
            .unwrap();
    }
    graph.commit(tx).await.unwrap();

    let config = PprConfig {
        damping_factor: 0.85,
        max_iterations: 100,
        convergence_epsilon: 1e-6,
        algorithm: PprAlgorithm::ForwardPush,
        warn_on_non_convergence: true,
    };

    let baseline = graph
        .personalized_page_rank(&[EntityId::new(1)], &config)
        .await
        .unwrap();

    for run in 1..=100 {
        let res = graph
            .personalized_page_rank(&[EntityId::new(1)], &config)
            .await
            .unwrap();
        assert_eq!(res.len(), baseline.len(), "Run {run} length mismatch");
        for (i, (a, b)) in res.iter().zip(baseline.iter()).enumerate() {
            assert_eq!(a.0, b.0, "Run {run} index {i} EntityId mismatch");
            assert_eq!(
                a.1.to_bits(),
                b.1.to_bits(),
                "Run {run} index {i} score bitwise mismatch: {} vs {}",
                a.1,
                b.1
            );
        }
    }
}

#[tokio::test]
async fn test_forward_push_performance_benchmark_vs_dense() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    // Build synthetic graph: 5,000 nodes, 15,000 edges
    let n = 5_000usize;
    for i in 1..=n {
        graph
            .add_entity(
                tx,
                Entity::new(EntityId::new(i as u64), format!("Node{i}"), "Benchmark"),
            )
            .await
            .unwrap();
    }

    for i in 1..n {
        graph
            .add_edge(
                tx,
                Edge::new(
                    EntityId::new(i as u64),
                    EntityId::new((i + 1) as u64),
                    "next",
                )
                .with_weight(1.0),
            )
            .await
            .unwrap();
        if i % 3 == 0 {
            let target = ((i * 17) % (n - 1) + 1) as u64;
            graph
                .add_edge(
                    tx,
                    Edge::new(EntityId::new(i as u64), EntityId::new(target), "shortcut")
                        .with_weight(0.5),
                )
                .await
                .unwrap();
        }
    }
    graph.commit(tx).await.unwrap();

    let seeds = vec![
        EntityId::new(10),
        EntityId::new(100),
        EntityId::new(500),
        EntityId::new(1000),
        EntityId::new(5000),
    ];

    let cfg_fp = PprConfig {
        damping_factor: 0.85,
        max_iterations: 100,
        convergence_epsilon: 1e-4,
        algorithm: PprAlgorithm::ForwardPush,
        warn_on_non_convergence: false,
    };

    let cfg_dense = PprConfig {
        damping_factor: 0.85,
        max_iterations: 100,
        convergence_epsilon: 1e-4,
        algorithm: PprAlgorithm::DensePowerIteration,
        warn_on_non_convergence: false,
    };

    let start_fp = std::time::Instant::now();
    let res_fp = graph.personalized_page_rank(&seeds, &cfg_fp).await.unwrap();
    let duration_fp = start_fp.elapsed();

    let start_dense = std::time::Instant::now();
    let res_dense = graph
        .personalized_page_rank(&seeds, &cfg_dense)
        .await
        .unwrap();
    let duration_dense = start_dense.elapsed();

    assert!(!res_fp.is_empty());
    assert!(!res_dense.is_empty());

    let speedup = duration_dense.as_secs_f64() / duration_fp.as_secs_f64().max(1e-6);
    println!(
            "PPR Benchmark (5 seeds on 50,000 nodes): Forward-Push = {:.2?}, Dense = {:.2?}, Speedup = {:.2}x",
            duration_fp, duration_dense, speedup
        );

    let expected_min_speedup = if cfg!(debug_assertions) { 2.0 } else { 50.0 };
    assert!(
            speedup >= expected_min_speedup,
            "Forward-push must achieve significant speedup over dense power iteration (expected >= {expected_min_speedup}x, got {speedup:.2}x)"
        );
}
