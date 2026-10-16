use contextra_graph::community::{
    detect_communities, CommunityAssignment, CommunityDetectionConfig,
};
use contextra_graph::csr::CsrGraph;
use contextra_ports::GraphIndex;
use contextra_types::{Edge, Entity, EntityId, TxId};
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
struct KarateClubDataset {
    nodes: usize,
    edges: Vec<(usize, usize)>,
}

async fn load_karate_club_graph() -> CsrGraph {
    let raw_json = include_str!("fixtures/karate_club.json");
    let dataset: KarateClubDataset =
        serde_json::from_str(raw_json).expect("Karate club fixture JSON must be valid");

    assert_eq!(dataset.nodes, 34, "Karate club graph must contain 34 nodes");
    assert_eq!(
        dataset.edges.len(),
        78,
        "Karate club graph must contain 78 edges"
    );

    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    // EntityId::new(0) is reserved as sentinel in CsrGraph, so 0-indexed node IDs 0..33 map to 1..34
    for i in 0..dataset.nodes {
        let eid = EntityId::new((i + 1) as u64);
        graph
            .add_entity(tx, Entity::new(eid, format!("Node_{i}"), "KarateMember"))
            .await
            .expect("add_entity must succeed");
    }

    for &(u, v) in &dataset.edges {
        let u_id = EntityId::new((u + 1) as u64);
        let v_id = EntityId::new((v + 1) as u64);
        graph
            .add_edge(tx, Edge::new(u_id, v_id, "member_link"))
            .await
            .expect("add_edge forward must succeed");
        graph
            .add_edge(tx, Edge::new(v_id, u_id, "member_link"))
            .await
            .expect("add_edge backward must succeed");
    }

    graph.commit(tx).await.expect("commit must succeed");

    assert_eq!(graph.entity_count(), 34);
    assert_eq!(graph.edge_count(), 156);

    graph
}

/// Computes Newman-Girvan Modularity Q for an undirected graph and community assignments:
/// Q = (1 / 2m) * \sum_{i,j} [ A_ij - (k_i * k_j) / (2m) ] * \delta(c_i, c_j)
async fn compute_modularity(graph: &CsrGraph, assignments: &[CommunityAssignment]) -> f32 {
    let num_nodes = assignments.len();
    if num_nodes == 0 {
        return 0.0;
    }

    let mut neighbors_map: std::collections::HashMap<EntityId, HashSet<EntityId>> =
        std::collections::HashMap::new();
    let mut degrees: std::collections::HashMap<EntityId, f32> = std::collections::HashMap::new();

    for assignment in assignments {
        let eid = assignment.entity_id;
        let neighs: HashSet<EntityId> = graph
            .neighbors(eid)
            .await
            .expect("neighbors lookup must succeed")
            .into_iter()
            .collect();
        degrees.insert(eid, neighs.len() as f32);
        neighbors_map.insert(eid, neighs);
    }

    let two_m: f32 = degrees.values().sum();
    if two_m <= 0.0 {
        return 0.0;
    }

    let mut q = 0.0f32;

    for a1 in assignments {
        let u = a1.entity_id;
        let c_u = a1.community_id;
        let k_u = degrees.get(&u).copied().unwrap_or(0.0);
        let u_neighs = neighbors_map.get(&u);

        for a2 in assignments {
            let v = a2.entity_id;
            let c_v = a2.community_id;

            if c_u == c_v {
                let k_v = degrees.get(&v).copied().unwrap_or(0.0);
                let a_uv = if u_neighs.map_or(false, |set| set.contains(&v)) {
                    1.0f32
                } else {
                    0.0f32
                };
                let expected = (k_u * k_v) / two_m;
                q += a_uv - expected;
            }
        }
    }

    q / two_m
}

#[tokio::test]
async fn leiden_on_karate_club_matches_reference() {
    let graph = load_karate_club_graph().await;
    let config = CommunityDetectionConfig {
        max_iterations: 100,
        seed: 42,
        hyperedges_included: false,
    };

    let communities = detect_communities(&graph, &config)
        .await
        .expect("Leiden community detection on Karate Club must succeed");

    assert_eq!(communities.len(), 34);

    let modularity = compute_modularity(&graph, &communities).await;
    assert!(
        modularity >= 0.3,
        "Modularity Q must be at least 0.3, got: {modularity}"
    );

    let num_communities = communities
        .iter()
        .map(|a| a.community_id)
        .collect::<HashSet<_>>()
        .len();
    assert!(
        (2..=10).contains(&num_communities),
        "Number of detected communities must be between 2 and 10, got: {num_communities}"
    );
}

#[tokio::test]
async fn leiden_is_deterministic_with_fixed_seed() {
    let graph = load_karate_club_graph().await;

    let config1 = CommunityDetectionConfig {
        max_iterations: 100,
        seed: 12345,
        hyperedges_included: false,
    };
    let config2 = CommunityDetectionConfig {
        max_iterations: 100,
        seed: 12345,
        hyperedges_included: false,
    };

    let res1 = detect_communities(&graph, &config1)
        .await
        .expect("Community detection run 1 must succeed");
    let res2 = detect_communities(&graph, &config2)
        .await
        .expect("Community detection run 2 must succeed");

    assert_eq!(
        res1, res2,
        "Two community detection runs with identical seed and graph must produce identical assignments"
    );
}
