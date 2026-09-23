use super::super::*;
use crate::csr::CsrGraph;
use memfuse_core::{DocId, Edge, Entity, EntityId, GraphIndex, MemFuseError, TxId};

use super::*;
use std::sync::Arc;

#[tokio::test]
async fn test_ppr_context_reuse_when_graph_grows() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    graph
        .add_entity(tx, Entity::new(EntityId::new(1), "N1", "Node"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_entity(tx, Entity::new(EntityId::new(2), "N2", "Node"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "link"))
        .await
        .unwrap(); // unwrap allowed
    graph.commit(tx).await.unwrap(); // unwrap allowed

    let mut ctx = PprContext::new();
    let config = PprConfig::default();

    // Call 1 on small graph (n=2)
    let res1 = graph
        .personalized_page_rank_with_context_async(&[EntityId::new(1)], &config, &mut ctx)
        .await;
    assert_eq!(res1.len(), 2);

    // Add 3 more nodes to expand graph to n=5
    let tx2 = TxId::new(2);
    for i in 3..=5 {
        graph
            .add_entity(tx2, Entity::new(EntityId::new(i), format!("N{i}"), "Node"))
            .await
            .unwrap(); // unwrap allowed
        graph
            .add_edge(
                tx2,
                Edge::new(EntityId::new(i - 1), EntityId::new(i), "link"),
            )
            .await
            .unwrap(); // unwrap allowed
    }
    graph.commit(tx2).await.unwrap(); // unwrap allowed

    // Call 2 reusing same ctx on grown graph (n=5)
    let res2 = graph
        .personalized_page_rank_with_context_async(&[EntityId::new(1)], &config, &mut ctx)
        .await;
    assert_eq!(res2.len(), 5);

    // Verify result matches fresh execution without context pollution
    let fresh_res = graph
        .personalized_page_rank(&[EntityId::new(1)], &config)
        .await
        .unwrap(); // unwrap allowed

    assert_eq!(res2.len(), fresh_res.len());
    for (a, b) in res2.iter().zip(fresh_res.iter()) {
        assert_eq!(a.0, b.0);
        assert_eq!(a.1.to_bits(), b.1.to_bits());
    }
}

#[tokio::test]
async fn test_ppr_analytical_5_node_ring() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for i in 1..=5 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("N{i}"), "Node"))
            .await
            .unwrap(); // unwrap allowed
    }

    // 1 -> 2 -> 3 -> 4 -> 5 -> 1
    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "next"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(2), EntityId::new(3), "next"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(3), EntityId::new(4), "next"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(4), EntityId::new(5), "next"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(5), EntityId::new(1), "next"))
        .await
        .unwrap(); // unwrap allowed
    graph.commit(tx).await.unwrap(); // unwrap allowed

    let config = PprConfig {
        damping_factor: 0.85,
        max_iterations: 100,
        convergence_epsilon: 1e-6,
        algorithm: PprAlgorithm::Auto,
        warn_on_non_convergence: true,
    };

    let seed = EntityId::new(1);
    let results = graph
        .personalized_page_rank(&[seed], &config)
        .await
        .unwrap(); // unwrap allowed

    assert_eq!(results.len(), 5);

    // Analytical solution for 5-ring with seed = Node 1 and d = 0.85:
    // r_i = (1 - d) * d^(i-1) / (1 - d^5)
    let d = 0.85f32;
    let denom = 1.0 - d.powi(5);
    let expected_r1 = (1.0 - d) * 1.0 / denom; // ~0.26964
    let expected_r2 = (1.0 - d) * d / denom; // ~0.22920
    let expected_r3 = (1.0 - d) * d.powi(2) / denom; // ~0.19482
    let expected_r4 = (1.0 - d) * d.powi(3) / denom; // ~0.16559
    let expected_r5 = (1.0 - d) * d.powi(4) / denom; // ~0.14075

    let rank_map: std::collections::HashMap<EntityId, f32> = results.into_iter().collect();

    assert!((rank_map[&EntityId::new(1)] - expected_r1).abs() < 1e-3);
    assert!((rank_map[&EntityId::new(2)] - expected_r2).abs() < 1e-3);
    assert!((rank_map[&EntityId::new(3)] - expected_r3).abs() < 1e-3);
    assert!((rank_map[&EntityId::new(4)] - expected_r4).abs() < 1e-3);
    assert!((rank_map[&EntityId::new(5)] - expected_r5).abs() < 1e-3);

    // Monotonic order check
    assert!(rank_map[&EntityId::new(1)] > rank_map[&EntityId::new(2)]);
    assert!(rank_map[&EntityId::new(2)] > rank_map[&EntityId::new(3)]);
    assert!(rank_map[&EntityId::new(3)] > rank_map[&EntityId::new(4)]);
    assert!(rank_map[&EntityId::new(4)] > rank_map[&EntityId::new(5)]);
}

#[tokio::test]
async fn test_ppr_handles_sink_node_correctly() {
    // Graph: A (1) -> B (2), B has NO outgoing edge (Sink), C (3) -> A (1)
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    let id_a = EntityId::new(1);
    let id_b = EntityId::new(2);
    let id_c = EntityId::new(3);

    graph
        .add_entity(tx, Entity::new(id_a, "Node A", "Node"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(id_b, "Node B (Sink)", "Node"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(id_c, "Node C", "Node"))
        .await
        .unwrap(); // unwrap

    // A -> B
    graph
        .add_edge(tx, Edge::new(id_a, id_b, "link"))
        .await
        .unwrap(); // unwrap
                   // C -> A
    graph
        .add_edge(tx, Edge::new(id_c, id_a, "link"))
        .await
        .unwrap(); // unwrap
    graph.commit(tx).await.unwrap(); // unwrap

    let config = PprConfig::default();
    let results = graph
        .personalized_page_rank(&[id_a], &config)
        .await
        .unwrap(); // unwrap

    let total_mass: f32 = results.iter().map(|(_, score)| score).sum();
    assert!(
        (total_mass - 1.0).abs() < 1e-4,
        "PPR mass must conserve to 1.0 when graph contains sink node B, got {total_mass}"
    );

    let rank_map: std::collections::HashMap<EntityId, f32> = results.into_iter().collect();
    assert!(
        rank_map.contains_key(&id_b),
        "Sink node B must receive rank mass from A"
    );
    assert!(rank_map[&id_b] > 0.0, "Sink node B score must be positive");
}

#[tokio::test]
async fn test_ppr_per_iteration_mass_conservation_dangling_and_isolated() {
    // Graph with multiple dangling nodes and isolated components
    // Core: 1 -> 2 -> 3 (3 is dangling)
    // Secondary: 4 -> 5, 4 -> 6 (5 and 6 are dangling)
    // Isolated: 7 (isolated, 0 in/out edges)
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for i in 1..=7 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("N{i}"), "Node"))
            .await
            .unwrap();
    }

    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "edge"))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(EntityId::new(2), EntityId::new(3), "edge"))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(EntityId::new(4), EntityId::new(5), "edge"))
        .await
        .unwrap();
    graph
        .add_edge(tx, Edge::new(EntityId::new(4), EntityId::new(6), "edge"))
        .await
        .unwrap();
    graph.commit(tx).await.unwrap();

    let inner = graph.inner_read();
    let seed = EntityId::new(1);

    let mut ctx = PprContext::new();
    let n = inner.reverse_map.len();
    ctx.prepare(n);

    // Step 1 & 2: Seed initialization
    let seed_idx = inner.id_map[&seed];
    ctx.valid_seeds.push(seed_idx);
    let restart_prob = 1.0f32;
    ctx.ranks[seed_idx] = 1.0;

    // Step 3: Precompute out_weight_sums
    for i in 0..n {
        let start = if i < inner.offsets.len() - 1 {
            inner.offsets[i]
        } else {
            0
        };
        let end = if i < inner.offsets.len() - 1 {
            inner.offsets[i + 1]
        } else {
            0
        };
        let mut sum = 0.0f32;
        for edge_idx in start..end {
            let target = inner.targets[edge_idx];
            let weight = inner.weights[edge_idx];
            if inner.entity_at(target).is_some() && weight > 0.0 {
                sum += weight;
            }
        }
        ctx.out_weight_sums[i] = sum;
    }

    let damping = 0.85f32;
    let mut mass_log = Vec::new();

    // Perform 20 explicit iterations and verify mass conservation AFTER EVERY ITERATION
    for iter in 1..=20 {
        ctx.next_ranks[..n].fill(0.0);

        let mut dangling_sum = 0.0f32;
        for i in 0..n {
            if inner.entity_at(i).is_some() && ctx.out_weight_sums[i] == 0.0 {
                dangling_sum += ctx.ranks[i];
            }
        }

        let teleport_factor = (1.0 - damping) + damping * dangling_sum;
        for &s_idx in &ctx.valid_seeds {
            ctx.next_ranks[s_idx] += teleport_factor * restart_prob;
        }

        for i in 0..n {
            let sum_w = ctx.out_weight_sums[i];
            let r_i = ctx.ranks[i];
            if sum_w > 0.0 && r_i > 0.0 {
                let share = damping * r_i / sum_w;
                let start = if i < inner.offsets.len() - 1 {
                    inner.offsets[i]
                } else {
                    0
                };
                let end = if i < inner.offsets.len() - 1 {
                    inner.offsets[i + 1]
                } else {
                    0
                };

                for edge_idx in start..end {
                    let target = inner.targets[edge_idx];
                    let weight = inner.weights[edge_idx];
                    if inner.entity_at(target).is_some() && weight > 0.0 {
                        ctx.next_ranks[target] += share * weight;
                    }
                }
            }
        }

        // Sum ranks using f64 precision accumulator for precise mass tracking
        let sum_ranks: f64 = ctx.next_ranks[..n].iter().map(|&r| r as f64).sum();
        mass_log.push((iter, sum_ranks));

        assert!(
            (sum_ranks - 1.0).abs() < 1e-6,
            "Iteration {iter}: Rank mass sum must remain 1.0, got {sum_ranks:.9}"
        );

        ctx.ranks[..n].copy_from_slice(&ctx.next_ranks[..n]);
    }

    println!("PPR Per-Iteration Rank Mass Conservation Log (20 Iterations):");
    for (iter, sum) in &mass_log {
        println!("  Iteration {:2}: sum(ranks) = {:.9}", iter, sum);
    }
}

#[tokio::test]
async fn test_ppr_dangling_node_mass_conservation() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for i in 1..=3 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("N{i}"), "Node"))
            .await
            .unwrap(); // unwrap allowed
    }

    // 1 -> 2 -> 3 (Node 3 has out-degree 0, dead-end/dangling)
    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "edge"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(2), EntityId::new(3), "edge"))
        .await
        .unwrap(); // unwrap allowed
    graph.commit(tx).await.unwrap(); // unwrap allowed

    let config = PprConfig::default();
    let results = graph
        .personalized_page_rank(&[EntityId::new(1)], &config)
        .await
        .unwrap(); // unwrap allowed

    let sum: f32 = results.iter().map(|(_, score)| score).sum();
    assert!(
        (sum - 1.0).abs() < 1e-4,
        "PPR rank mass must conserve to 1.0 despite dangling nodes, got {sum}"
    );
}

#[tokio::test]
async fn test_ppr_bit_identical_determinism() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for i in 1..=6 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("N{i}"), "Node"))
            .await
            .unwrap(); // unwrap allowed
    }

    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "rel"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(3), "rel"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(2), EntityId::new(4), "rel"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(3), EntityId::new(4), "rel"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(4), EntityId::new(5), "rel"))
        .await
        .unwrap(); // unwrap allowed
    graph
        .add_edge(tx, Edge::new(EntityId::new(5), EntityId::new(6), "rel"))
        .await
        .unwrap(); // unwrap allowed
    graph.commit(tx).await.unwrap(); // unwrap allowed

    let config = PprConfig::default();
    let run1 = graph
        .personalized_page_rank(&[EntityId::new(1)], &config)
        .await
        .unwrap(); // unwrap allowed
    let run2 = graph
        .personalized_page_rank(&[EntityId::new(1)], &config)
        .await
        .unwrap(); // unwrap allowed

    assert_eq!(run1.len(), run2.len());
    for (a, b) in run1.iter().zip(run2.iter()) {
        assert_eq!(a.0, b.0);
        assert_eq!(
            a.1.to_bits(),
            b.1.to_bits(),
            "Float scores must be bit-identical across runs"
        );
    }
}

#[tokio::test]
async fn test_ppr_single_node_no_edges() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    let seed = EntityId::new(42);

    graph
        .add_entity(tx, Entity::new(seed, "SoleNode", "Node"))
        .await
        .unwrap(); // unwrap
    graph.commit(tx).await.unwrap(); // unwrap

    let config = PprConfig::default();
    let results = graph
        .personalized_page_rank(&[seed], &config)
        .await
        .unwrap(); // unwrap

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, seed);
    assert!((results[0].1 - 1.0).abs() < 1e-4);
}

#[tokio::test]
async fn test_ppr_isolated_nodes_handling() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for i in 1..=4 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("N{i}"), "Node"))
            .await
            .unwrap(); // unwrap
    }

    // Only 1 -> 2 edge. 3 and 4 are completely isolated.
    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "link"))
        .await
        .unwrap(); // unwrap
    graph.commit(tx).await.unwrap(); // unwrap

    let config = PprConfig::default();
    let results = graph
        .personalized_page_rank(&[EntityId::new(1)], &config)
        .await
        .unwrap(); // unwrap

    assert!(!results.is_empty());
    for (_, rank) in &results {
        assert!(!rank.is_nan(), "Rank must not be NaN");
        assert!(!rank.is_infinite(), "Rank must not be infinite");
    }

    let total_mass: f32 = results.iter().map(|(_, r)| r).sum();
    assert!(
        (total_mass - 1.0).abs() < 1e-4,
        "Total rank mass must conserve to 1.0 even with isolated nodes, got {total_mass}"
    );
}

#[tokio::test]
async fn test_ppr_self_loop_handling() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    graph
        .add_entity(tx, Entity::new(EntityId::new(1), "N1", "Node"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(EntityId::new(2), "N2", "Node"))
        .await
        .unwrap(); // unwrap

    // 1 -> 1 (self-loop) and 1 -> 2
    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(1), "self"))
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "link"))
        .await
        .unwrap(); // unwrap
    graph.commit(tx).await.unwrap(); // unwrap

    let config = PprConfig::default();
    let results = graph
        .personalized_page_rank(&[EntityId::new(1)], &config)
        .await
        .unwrap(); // unwrap

    assert_eq!(results.len(), 2);
    for (_, rank) in &results {
        assert!(!rank.is_nan());
    }
    let total_mass: f32 = results.iter().map(|(_, r)| r).sum();
    assert!((total_mass - 1.0).abs() < 1e-4);
}

#[tokio::test]
async fn test_ppr_duplicate_multi_edges_deterministic() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    graph
        .add_entity(tx, Entity::new(EntityId::new(1), "N1", "Node"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(EntityId::new(2), "N2", "Node"))
        .await
        .unwrap(); // unwrap

    // Add 3 duplicate edges 1 -> 2 with weights 1.0, 2.0, 3.0
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(1), EntityId::new(2), "edge1").with_weight(1.0),
        )
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(1), EntityId::new(2), "edge2").with_weight(2.0),
        )
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(1), EntityId::new(2), "edge3").with_weight(3.0),
        )
        .await
        .unwrap(); // unwrap
    graph.commit(tx).await.unwrap(); // unwrap

    let config = PprConfig::default();
    let res1 = graph
        .personalized_page_rank(&[EntityId::new(1)], &config)
        .await
        .unwrap(); // unwrap
    let res2 = graph
        .personalized_page_rank(&[EntityId::new(1)], &config)
        .await
        .unwrap(); // unwrap

    assert_eq!(res1.len(), res2.len());
    for (a, b) in res1.iter().zip(res2.iter()) {
        assert_eq!(a.0, b.0);
        assert_eq!(a.1.to_bits(), b.1.to_bits());
    }
}

#[tokio::test]
async fn test_ppr_exact_score_tie_breaking_by_entity_id() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    // Seed node 1, and symmetric nodes 10, 20 connected identically
    graph
        .add_entity(tx, Entity::new(EntityId::new(1), "Center", "Node"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(EntityId::new(20), "B", "Node"))
        .await
        .unwrap(); // unwrap
    graph
        .add_entity(tx, Entity::new(EntityId::new(10), "A", "Node"))
        .await
        .unwrap(); // unwrap

    // 1 -> 10 and 1 -> 20 with identical weight 1.0
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(1), EntityId::new(10), "link").with_weight(1.0),
        )
        .await
        .unwrap(); // unwrap
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(1), EntityId::new(20), "link").with_weight(1.0),
        )
        .await
        .unwrap(); // unwrap
    graph.commit(tx).await.unwrap(); // unwrap

    let config = PprConfig::default();
    let results = graph
        .personalized_page_rank(&[EntityId::new(1)], &config)
        .await
        .unwrap(); // unwrap

    let rank_10 = results
        .iter()
        .find(|(id, _)| *id == EntityId::new(10))
        .map(|(_, r)| *r)
        .unwrap(); // unwrap
    let rank_20 = results
        .iter()
        .find(|(id, _)| *id == EntityId::new(20))
        .map(|(_, r)| *r)
        .unwrap(); // unwrap

    assert_eq!(
        rank_10, rank_20,
        "Symmetric nodes must have identical PPR scores"
    );

    // Results order must sort tie by EntityId ascending (10 before 20)
    let idx_10 = results
        .iter()
        .position(|(id, _)| *id == EntityId::new(10))
        .unwrap(); // unwrap
    let idx_20 = results
        .iter()
        .position(|(id, _)| *id == EntityId::new(20))
        .unwrap(); // unwrap
    assert!(
        idx_10 < idx_20,
        "Tie-breaking must place EntityId(10) before EntityId(20)"
    );
}

#[tokio::test]
async fn test_ppr_hyperedge_expansion_dense_and_forward_push() {
    use crate::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};

    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    let e1 = EntityId::new(100);
    let e2 = EntityId::new(200);
    let e3 = EntityId::new(300);

    graph
        .add_entity(tx, Entity::new(e1, "E1", "Node"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(e2, "E2", "Node"))
        .await
        .unwrap();
    graph
        .add_entity(tx, Entity::new(e3, "E3", "Node"))
        .await
        .unwrap();

    // Binary edge e1 -> e2
    graph
        .add_edge(tx, Edge::new(e1, e2, "knows").with_weight(1.0))
        .await
        .unwrap();
    graph.commit(tx).await.unwrap();

    // Hyperedge linking e1, e2, e3
    const ROLE: RoleId = RoleId::new(1);
    let hedge = HyperEdge::new(
        HyperEdgeId::new(1),
        crate::csr::EdgeType::Default,
        vec![
            RoleBinding::new(ROLE, e1),
            RoleBinding::new(ROLE, e2),
            RoleBinding::new(ROLE, e3),
        ],
        1.5,
    );
    graph.insert_hyperedge_direct(hedge);

    // Test Dense Power Iteration
    let cfg_dense = PprConfig {
        damping_factor: 0.85,
        max_iterations: 100,
        convergence_epsilon: 1e-6,
        algorithm: PprAlgorithm::DensePowerIteration,
        warn_on_non_convergence: true,
    };

    let res_dense = graph
        .personalized_page_rank(&[e1], &cfg_dense)
        .await
        .unwrap();

    let rank_map_dense: std::collections::HashMap<EntityId, f32> = res_dense.into_iter().collect();

    assert!(
        rank_map_dense.contains_key(&e3),
        "e3 must receive rank mass via hyperedge expansion in dense PPR"
    );
    assert!(
        rank_map_dense[&e3] > 0.0,
        "e3 score must be positive via hyperedge link"
    );

    // Test Forward Push
    let cfg_fp = PprConfig {
        damping_factor: 0.85,
        max_iterations: 100,
        convergence_epsilon: 1e-6,
        algorithm: PprAlgorithm::ForwardPush,
        warn_on_non_convergence: true,
    };

    let res_fp = graph.personalized_page_rank(&[e1], &cfg_fp).await.unwrap();

    let rank_map_fp: std::collections::HashMap<EntityId, f32> = res_fp.into_iter().collect();

    assert!(
        rank_map_fp.contains_key(&e3),
        "e3 must receive rank mass via hyperedge expansion in forward push PPR"
    );
    assert!(
        rank_map_fp[&e3] > 0.0,
        "e3 score must be positive via hyperedge link in forward push"
    );
}

proptest::proptest! {
    #[test]
    fn prop_ppr_rank_mass_conservation(
        node_count in 1..=20usize,
        edge_specs in proptest::collection::vec((0..20usize, 0..20usize, 0.1f32..5.0f32), 0..40),
        seed_idx in 0..20usize,
        damping in 0.1f32..0.99f32,
        max_iters in 1..=50u32,
    ) {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap(); // unwrap
        let res: Result<(), proptest::test_runner::TestCaseError> = rt.block_on(async {
            let graph = CsrGraph::new();
            let tx = TxId::new(1);

            for i in 0..node_count {
                graph
                    .add_entity(tx, Entity::new(EntityId::new(i as u64 + 1), format!("N{i}"), "Node"))
                    .await
                    .unwrap(); // unwrap
            }

            for (src, dst, w) in edge_specs {
                let src_id = EntityId::new((src % node_count) as u64 + 1);
                let dst_id = EntityId::new((dst % node_count) as u64 + 1);
                graph
                    .add_edge(tx, Edge::new(src_id, dst_id, "link").with_weight(w))
                    .await
                    .unwrap(); // unwrap
            }
            graph.commit(tx).await.unwrap(); // unwrap

            let actual_seed = EntityId::new((seed_idx % node_count) as u64 + 1);
            let config = PprConfig {
                damping_factor: damping,
                max_iterations: max_iters,
                convergence_epsilon: 1e-7,
                algorithm: PprAlgorithm::Auto,
                warn_on_non_convergence: true,
            };

            let results = graph
                .personalized_page_rank(&[actual_seed], &config)
                .await
                .unwrap(); // unwrap

            let total_mass: f32 = results.iter().map(|(_, r)| r).sum();
            proptest::prop_assert!(
                (total_mass - 1.0).abs() < 1e-4,
                "Rank mass conservation failed: total mass {} != 1.0 for node_count={}, seed={:?}",
                total_mass,
                node_count,
                actual_seed
            );
            Ok(())
        });
        res?;
    }
}

#[derive(Clone)]
struct LogCaptureLayer(std::sync::Arc<std::sync::Mutex<Vec<String>>>);

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for LogCaptureLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = StringVisitor(String::new());
        event.record(&mut visitor);
        self.0.lock().unwrap().push(visitor.0); // unwrap allowed
    }
}

struct StringVisitor(String);
impl tracing::field::Visit for StringVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        use std::fmt::Write;
        write!(self.0, "{}={:?} ", field.name(), value).ok();
    }
}
