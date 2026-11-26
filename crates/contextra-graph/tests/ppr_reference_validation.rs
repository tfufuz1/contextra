//! WP-70: Personalized PageRank (PPR) Referenzwerte- und Validierungstests.
//!
//! # Zweck & Spezifikation
//! 1. Referenz-Vergleich gegen exakte handgerechnete Werte auf kleinen Graphen (≤ 10 Knoten).
//! 2. Referenz-Vergleich gegen klassische Power-Iteration auf Zufallsgraphen mit festen Seeds.
//!
//! Toleranzfestlegung VOR der Messung:
//! - Handgerechnete Graphen: Absolute Toleranz `TOL_HAND = 1e-4`
//! - Zufallsgraphen gegen unabhängige Power-Iteration: Absolute Toleranz `TOL_RANDOM = 1e-3`

use contextra_graph::CsrGraph;
use contextra_ports::GraphIndex;
use contextra_types::{Edge, Entity, EntityId, PprAlgorithm, PprConfig, TxId};
use std::collections::HashMap;

/// Handgerechneter 4-Knoten-Diamantgraph (Diamond Graph):
///
/// Knotenmenge V = {1, 2, 3, 4}
/// Kanten:
///   1 -> 2, 1 -> 3 (Knoten 1 teilt seine Kantenmasse gleichmäßig: p_{1->2} = 0.5, p_{1->3} = 0.5)
///   2 -> 4 (Knoten 2: p_{2->4} = 1.0)
///   3 -> 4 (Knoten 3: p_{3->4} = 1.0)
///   4 -> 1 (Knoten 4: p_{4->1} = 1.0)
///
/// Teleportations-Vektor: Seed = [1] -> v = (1, 0, 0, 0)^T
/// Dämpfungsfaktor: alpha = 0.8
///
/// Exakte PageRank-Gleichungen:
///   p_1 = (1 - alpha) * 1 + alpha * (1.0 * p_4)
///   p_2 = alpha * (0.5 * p_1)
///   p_3 = alpha * (0.5 * p_1)
///   p_4 = alpha * (1.0 * p_2 + 1.0 * p_3) = alpha * (0.5 * alpha * p_1 + 0.5 * alpha * p_1) = alpha^2 * p_1
///
/// Einsetzen von p_4 in p_1:
///   p_1 = (1 - alpha) + alpha^3 * p_1
///   p_1 * (1 - alpha^3) = 1 - alpha
///   p_1 = (1 - alpha) / (1 - alpha^3) = 1 / (1 + alpha + alpha^2)
///
/// Für alpha = 0.8:
///   1 + alpha + alpha^2 = 1 + 0.8 + 0.64 = 2.44 = 61 / 25
///   => p_1 = 25 / 61 ≈ 0.40983607
///   => p_2 = 0.8 * 0.5 * (25 / 61) = 10 / 61 ≈ 0.16393443
///   => p_3 = 0.8 * 0.5 * (25 / 61) = 10 / 61 ≈ 0.16393443
///   => p_4 = 0.64 * (25 / 61) = 16 / 61 ≈ 0.26229508
///   Summe: (25 + 10 + 10 + 16) / 61 = 61 / 61 = 1.00000000
#[tokio::test]
async fn test_ppr_hand_calculated_diamond_4_nodes() {
    let graph = CsrGraph::new();
    let tx = TxId::new(101);

    for id in 1..=4 {
        graph
            .add_entity(
                tx,
                Entity::new(EntityId::new(id), format!("Entity_{id}"), "Node"),
            )
            .await
            .expect("Entity addition must succeed");
    }

    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(2), "rel"))
        .await
        .expect("Edge addition must succeed");
    graph
        .add_edge(tx, Edge::new(EntityId::new(1), EntityId::new(3), "rel"))
        .await
        .expect("Edge addition must succeed");
    graph
        .add_edge(tx, Edge::new(EntityId::new(2), EntityId::new(4), "rel"))
        .await
        .expect("Edge addition must succeed");
    graph
        .add_edge(tx, Edge::new(EntityId::new(3), EntityId::new(4), "rel"))
        .await
        .expect("Edge addition must succeed");
    graph
        .add_edge(tx, Edge::new(EntityId::new(4), EntityId::new(1), "rel"))
        .await
        .expect("Edge addition must succeed");

    graph.commit(tx).await.expect("Commit must succeed");

    let alpha = 0.80f32;
    let config = PprConfig {
        damping_factor: alpha,
        max_iterations: 150,
        convergence_epsilon: 1e-7,
        algorithm: PprAlgorithm::DensePowerIteration,
        warn_on_non_convergence: true,
    };

    let results = graph
        .personalized_page_rank(&[EntityId::new(1)], &config)
        .await
        .expect("PPR execution must succeed");

    let score_map: HashMap<EntityId, f32> = results.into_iter().collect();

    let expected_p1 = 25.0 / 61.0;
    let expected_p2 = 10.0 / 61.0;
    let expected_p3 = 10.0 / 61.0;
    let expected_p4 = 16.0 / 61.0;

    let tol = 1e-4f32; // Vorab festgelegte Toleranz

    let actual_p1 = score_map.get(&EntityId::new(1)).copied().unwrap_or(0.0);
    let actual_p2 = score_map.get(&EntityId::new(2)).copied().unwrap_or(0.0);
    let actual_p3 = score_map.get(&EntityId::new(3)).copied().unwrap_or(0.0);
    let actual_p4 = score_map.get(&EntityId::new(4)).copied().unwrap_or(0.0);

    assert!(
        (actual_p1 - expected_p1).abs() < tol,
        "Node 1 mismatch: actual {actual_p1}, expected {expected_p1}, delta {}",
        (actual_p1 - expected_p1).abs()
    );
    assert!(
        (actual_p2 - expected_p2).abs() < tol,
        "Node 2 mismatch: actual {actual_p2}, expected {expected_p2}, delta {}",
        (actual_p2 - expected_p2).abs()
    );
    assert!(
        (actual_p3 - expected_p3).abs() < tol,
        "Node 3 mismatch: actual {actual_p3}, expected {expected_p3}, delta {}",
        (actual_p3 - expected_p3).abs()
    );
    assert!(
        (actual_p4 - expected_p4).abs() < tol,
        "Node 4 mismatch: actual {actual_p4}, expected {expected_p4}, delta {}",
        (actual_p4 - expected_p4).abs()
    );
}

/// Unabhängige Standard Power-Iteration Referenzimplementierung.
fn compute_reference_power_iteration(
    num_nodes: usize,
    adj_list: &[Vec<usize>],
    seeds: &[usize],
    alpha: f32,
    max_iter: usize,
    tol: f32,
) -> Vec<f32> {
    let mut p = vec![0.0f32; num_nodes];
    if seeds.is_empty() || num_nodes == 0 {
        return p;
    }
    let restart_prob = 1.0 / (seeds.len() as f32);
    for &s in seeds {
        p[s] = restart_prob;
    }

    for _ in 0..max_iter {
        let mut next_p = vec![0.0f32; num_nodes];

        // Dangling mass sum (nodes with out_degree == 0 distribute to teleport targets)
        let mut dangling_sum = 0.0f32;
        for u in 0..num_nodes {
            if adj_list[u].is_empty() {
                dangling_sum += p[u];
            }
        }

        let teleport_factor = (1.0 - alpha) + alpha * dangling_sum;
        for &s in seeds {
            next_p[s] += teleport_factor * restart_prob;
        }

        // Kantenübertragung
        for u in 0..num_nodes {
            let out_deg = adj_list[u].len();
            if out_deg > 0 && p[u] > 0.0 {
                let share = (alpha * p[u]) / (out_deg as f32);
                for &v in &adj_list[u] {
                    next_p[v] += share;
                }
            }
        }

        let diff: f32 = p
            .iter()
            .zip(next_p.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        p = next_p;
        if diff < tol {
            break;
        }
    }

    // Normalisierung wie in CsrGraph extract_dense_results
    let sum: f32 = p.iter().sum();
    if sum > 0.0 {
        for v in &mut p {
            *v /= sum;
        }
    }

    p
}

/// LCG-Zufallsgenerator für deterministische Zufallsgraphen.
struct FixedLcg {
    state: u64,
}

impl FixedLcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    fn next_range(&mut self, max_exclusive: usize) -> usize {
        (self.next_u32() as usize) % max_exclusive
    }
}

/// Differentialtest: CsrGraph Dense PPR gegen die unabhängige Power-Iteration Referenz.
#[tokio::test]
async fn test_ppr_differential_against_power_iteration_fixed_seeds() {
    let test_seeds: [u64; 3] = [0xDEADBEEF, 0x12345678, 0xCAFEBABE];

    for (test_idx, &rng_seed) in test_seeds.iter().enumerate() {
        let mut rng = FixedLcg::new(rng_seed);
        let num_nodes = 20 + (rng.next_range(15)); // 20 bis 34 Knoten
        let num_edges = num_nodes * 2;

        let mut adj = vec![Vec::new(); num_nodes];
        let graph = CsrGraph::new();
        let tx = TxId::new((test_idx + 1) as u64);

        for id in 0..num_nodes {
            graph
                .add_entity(
                    tx,
                    Entity::new(EntityId::new((id + 1) as u64), format!("N{id}"), "Node"),
                )
                .await
                .expect("Add entity must succeed");
        }

        for _ in 0..num_edges {
            let u = rng.next_range(num_nodes);
            let v = rng.next_range(num_nodes);
            if u != v && !adj[u].contains(&v) {
                adj[u].push(v);
                graph
                    .add_edge(
                        tx,
                        Edge::new(
                            EntityId::new((u + 1) as u64),
                            EntityId::new((v + 1) as u64),
                            "link",
                        ),
                    )
                    .await
                    .expect("Add edge must succeed");
            }
        }

        graph.commit(tx).await.expect("Commit must succeed");

        let seed_idx = rng.next_range(num_nodes);
        let seed_entity = EntityId::new((seed_idx + 1) as u64);

        let alpha = 0.85f32;
        let max_iter = 250;
        let tol = 1e-6f32;

        let ref_scores =
            compute_reference_power_iteration(num_nodes, &adj, &[seed_idx], alpha, max_iter, tol);

        let config = PprConfig {
            damping_factor: alpha,
            max_iterations: max_iter as u32,
            convergence_epsilon: tol,
            algorithm: PprAlgorithm::DensePowerIteration,
            warn_on_non_convergence: false,
        };

        let results = graph
            .personalized_page_rank(&[seed_entity], &config)
            .await
            .expect("PPR execution must succeed");

        let result_map: HashMap<EntityId, f32> = results.into_iter().collect();

        let allowed_tolerance = 1e-3f32; // Vorab festgelegte Schranke
        for (i, &expected) in ref_scores.iter().enumerate() {
            let entity_id = EntityId::new((i + 1) as u64);
            let actual = result_map.get(&entity_id).copied().unwrap_or(0.0);
            assert!(
                (actual - expected).abs() < allowed_tolerance,
                "Seed 0x{rng_seed:X}, Node {i} ({entity_id:?}): actual {actual}, expected {expected}, delta {}",
                (actual - expected).abs()
            );
        }
    }
}
