//! Modul `golden_tests` im Ring 0 (Graph-Subsystem) von Contextra.
//!
//! Stellt Kernkomponenten für Graphtraversierung, CSR-Repräsentationen und
//! Analysemethoden bereit. Wird von Integrationsschichten und Algorithmenmodulen verwendet.

use crate::csr::{CsrGraph, GraphInner};
use crate::ppr::{compute_ppr, DeletedView};
use contextra_types::{Entity, EntityId, PprConfig};
use std::sync::Arc;

fn create_entity(id: u64) -> Entity {
    Entity::new(EntityId::new(id), "label", "type")
}

#[tokio::test]
async fn test_golden_ppr_chain_star_clusters_sinks() {
    // 1. Chain Graph: 1 -> 2 -> 3 -> 4
    let g_chain = Arc::new(CsrGraph::new());
    for i in 1..=4 {
        g_chain.insert_entity_direct(create_entity(i)).unwrap();
    }
    g_chain
        .add_edge(
            EntityId::new(1),
            EntityId::new(2),
            1.0,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    g_chain
        .add_edge(
            EntityId::new(2),
            EntityId::new(3),
            1.0,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    g_chain
        .add_edge(
            EntityId::new(3),
            EntityId::new(4),
            1.0,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    g_chain.compact_async().await.unwrap();

    let inner_chain = g_chain.inner_read();
    let config = PprConfig::default();
    let view = DeletedView::empty();

    let ppr_chain = compute_ppr(&inner_chain, &[EntityId::new(1)], &config, &view);
    assert!(!ppr_chain.is_empty());
    let rank_map_chain: std::collections::HashMap<EntityId, f32> = ppr_chain.into_iter().collect();
    assert!(rank_map_chain[&EntityId::new(1)] > rank_map_chain[&EntityId::new(2)]);
    assert!(rank_map_chain[&EntityId::new(2)] > rank_map_chain[&EntityId::new(3)]);
    assert!(rank_map_chain[&EntityId::new(3)] > rank_map_chain[&EntityId::new(4)]);

    // 2. Star Graph: Center 1 connected to leaves 2, 3, 4
    let g_star = Arc::new(CsrGraph::new());
    for i in 1..=4 {
        g_star.insert_entity_direct(create_entity(i)).unwrap();
    }
    for leaf in 2..=4 {
        g_star
            .add_edge(
                EntityId::new(1),
                EntityId::new(leaf),
                1.0,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .unwrap();
    }
    g_star.compact_async().await.unwrap();

    let inner_star = g_star.inner_read();
    let ppr_star = compute_ppr(&inner_star, &[EntityId::new(1)], &config, &view);
    let rank_map_star: std::collections::HashMap<EntityId, f32> = ppr_star.into_iter().collect();
    let r2 = rank_map_star[&EntityId::new(2)];
    let r3 = rank_map_star[&EntityId::new(3)];
    let r4 = rank_map_star[&EntityId::new(4)];
    assert!((r2 - r3).abs() < 1e-5, "r2 vs r3 mismatch: {r2} != {r3}");
    assert!((r3 - r4).abs() < 1e-5, "r3 vs r4 mismatch: {r3} != {r4}");

    // 3. Two Clusters
    let g_clusters = Arc::new(CsrGraph::new());
    for i in 1..=4 {
        g_clusters.insert_entity_direct(create_entity(i)).unwrap();
    }
    g_clusters
        .add_edge(
            EntityId::new(1),
            EntityId::new(2),
            1.0,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    g_clusters
        .add_edge(
            EntityId::new(2),
            EntityId::new(1),
            1.0,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    g_clusters
        .add_edge(
            EntityId::new(3),
            EntityId::new(4),
            1.0,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    g_clusters
        .add_edge(
            EntityId::new(4),
            EntityId::new(3),
            1.0,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    g_clusters
        .add_edge(
            EntityId::new(2),
            EntityId::new(3),
            0.1,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    g_clusters.compact_async().await.unwrap();

    let inner_clusters = g_clusters.inner_read();
    let ppr_clusters = compute_ppr(&inner_clusters, &[EntityId::new(1)], &config, &view);
    let rank_map_clusters: std::collections::HashMap<EntityId, f32> =
        ppr_clusters.into_iter().collect();
    let mass_a = rank_map_clusters.get(&EntityId::new(1)).unwrap_or(&0.0)
        + rank_map_clusters.get(&EntityId::new(2)).unwrap_or(&0.0);
    let mass_b = rank_map_clusters.get(&EntityId::new(3)).unwrap_or(&0.0)
        + rank_map_clusters.get(&EntityId::new(4)).unwrap_or(&0.0);
    assert!(
        mass_a > mass_b * 2.0,
        "Cluster A mass {mass_a} must exceed Cluster B mass {mass_b}"
    );

    // 4. Graph with Sink: 1 -> 2
    let g_sink = Arc::new(CsrGraph::new());
    g_sink.insert_entity_direct(create_entity(1)).unwrap();
    g_sink.insert_entity_direct(create_entity(2)).unwrap();
    g_sink
        .add_edge(
            EntityId::new(1),
            EntityId::new(2),
            1.0,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
    g_sink.compact_async().await.unwrap();

    let inner_sink = g_sink.inner_read();
    let ppr_sink = compute_ppr(&inner_sink, &[EntityId::new(1)], &config, &view);
    let total_mass: f32 = ppr_sink.iter().map(|(_, r)| r).sum();
    assert!(
        (total_mass - 1.0).abs() < 1e-4,
        "Mass conservation in sink graph failed: {total_mass}"
    );
}

#[tokio::test]
async fn test_ppr_edge_cases_handling() {
    let g = Arc::new(CsrGraph::new());
    let e1 = create_entity(1);
    let e2 = create_entity(2);
    g.insert_entity_direct(e1).unwrap();
    g.insert_entity_direct(e2).unwrap();

    let res_nan = g
        .add_edge(
            EntityId::new(1),
            EntityId::new(2),
            f32::NAN,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;
    assert!(res_nan.is_err());

    let res_inf = g
        .add_edge(
            EntityId::new(1),
            EntityId::new(2),
            f32::INFINITY,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;
    assert!(res_inf.is_err());

    let res_neg = g
        .add_edge(
            EntityId::new(1),
            EntityId::new(2),
            -0.5,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;
    assert!(res_neg.is_err());

    g.add_edge(
        EntityId::new(1),
        EntityId::new(1),
        1.0,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    g.add_edge(
        EntityId::new(1),
        EntityId::new(2),
        0.5,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    g.add_edge(
        EntityId::new(1),
        EntityId::new(2),
        0.5,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();
    g.compact_async().await.unwrap();

    let inner = g.inner_read();
    let config = PprConfig::default();
    let view = DeletedView::empty();

    let ppr = compute_ppr(&inner, &[EntityId::new(1)], &config, &view);
    assert!(!ppr.is_empty());
    let total_mass: f32 = ppr.iter().map(|(_, r)| r).sum();
    assert!((total_mass - 1.0).abs() < 1e-4);

    let empty_inner = GraphInner::new();
    let ppr_empty = compute_ppr(&empty_inner, &[EntityId::new(1)], &config, &view);
    assert!(ppr_empty.is_empty());
}
