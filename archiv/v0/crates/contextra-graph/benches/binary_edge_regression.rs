use contextra_graph::csr::{CsrGraph, CsrGraphConfig};
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_types::{Entity, EntityId};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn build_graph_with_binary_edges(rt: &Runtime, n_nodes: usize, n_edges: usize) -> Arc<CsrGraph> {
    let graph = Arc::new(CsrGraph::with_config(CsrGraphConfig {
        rebuild_threshold: 10_000_000,
        ..Default::default()
    }));

    rt.block_on(async {
        for i in 0..n_nodes {
            graph
                .insert_entity_direct(Entity::new(
                    EntityId::new(i as u64),
                    format!("Node_{i}"),
                    "TestEntity",
                ))
                .unwrap();
        }

        let mut seed = 42u64;
        for _ in 0..n_edges {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let src = (seed as usize) % n_nodes;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let dst = (seed as usize) % n_nodes;

            graph
                .insert_edge_direct(EntityId::new(src as u64), EntityId::new(dst as u64), 1.0)
                .await
                .unwrap();
        }
        graph.compact();
    });

    graph
}

fn build_graph_with_hyperedges(rt: &Runtime, n_nodes: usize, n_edges: usize) -> Arc<CsrGraph> {
    let graph = Arc::new(CsrGraph::with_config(CsrGraphConfig {
        rebuild_threshold: 10_000_000,
        ..Default::default()
    }));

    rt.block_on(async {
        for i in 0..n_nodes {
            graph
                .insert_entity_direct(Entity::new(
                    EntityId::new(i as u64),
                    format!("Node_{i}"),
                    "TestEntity",
                ))
                .unwrap();
        }

        let mut seed = 42u64;
        for hid in 0..n_edges {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let e1 = (seed as usize) % n_nodes;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let e2 = (seed as usize) % n_nodes;

            let bindings = vec![
                RoleBinding::new(RoleId::new(1), EntityId::new(e1 as u64)),
                RoleBinding::new(RoleId::new(2), EntityId::new(e2 as u64)),
            ];

            let hedge = HyperEdge::new(
                HyperEdgeId::new(hid as u64 + 1),
                contextra_graph::csr::EdgeType::Default,
                bindings,
                1.0,
            );

            graph.insert_hyperedge_direct(hedge);
        }
        graph.compact();
    });

    graph
}

fn bench_binary_edge_regression(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("binary_edge_regression");

    for (n_nodes, n_edges) in [(1_000, 3_000), (5_000, 15_000)] {
        // 1. Insertion benchmarks
        group.bench_with_input(
            BenchmarkId::new("binary_edge_insert", format!("n={n_nodes}_e={n_edges}")),
            &(n_nodes, n_edges),
            |b, &(n, e)| {
                b.iter_with_setup(
                    || {
                        let g = Arc::new(CsrGraph::with_config(CsrGraphConfig {
                            rebuild_threshold: 10_000_000,
                            ..Default::default()
                        }));
                        for i in 0..n {
                            g.insert_entity_direct(Entity::new(
                                EntityId::new(i as u64),
                                format!("Node_{i}"),
                                "TestEntity",
                            ))
                            .unwrap();
                        }
                        g
                    },
                    |g| {
                        rt.block_on(async {
                            let mut seed = 12345u64;
                            for _ in 0..e {
                                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                                let src = (seed as usize) % n;
                                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                                let dst = (seed as usize) % n;

                                black_box(
                                    g.insert_edge_direct(
                                        EntityId::new(src as u64),
                                        EntityId::new(dst as u64),
                                        1.0,
                                    )
                                    .await
                                    .unwrap(),
                                );
                            }
                        });
                    },
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("hyperedge_2ary_insert", format!("n={n_nodes}_e={n_edges}")),
            &(n_nodes, n_edges),
            |b, &(n, e)| {
                b.iter_with_setup(
                    || {
                        let g = Arc::new(CsrGraph::with_config(CsrGraphConfig {
                            rebuild_threshold: 10_000_000,
                            ..Default::default()
                        }));
                        for i in 0..n {
                            g.insert_entity_direct(Entity::new(
                                EntityId::new(i as u64),
                                format!("Node_{i}"),
                                "TestEntity",
                            ))
                            .unwrap();
                        }
                        g
                    },
                    |g| {
                        let mut seed = 12345u64;
                        for hid in 0..e {
                            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                            let e1 = (seed as usize) % n;
                            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                            let e2 = (seed as usize) % n;

                            let bindings = vec![
                                RoleBinding::new(RoleId::new(1), EntityId::new(e1 as u64)),
                                RoleBinding::new(RoleId::new(2), EntityId::new(e2 as u64)),
                            ];

                            let hedge = HyperEdge::new(
                                HyperEdgeId::new(hid as u64 + 1),
                                contextra_graph::csr::EdgeType::Default,
                                bindings,
                                1.0,
                            );

                            black_box(g.insert_hyperedge_direct(hedge));
                        }
                    },
                );
            },
        );

        // 2. Traversal/Lookup benchmarks
        let binary_graph = build_graph_with_binary_edges(&rt, n_nodes, n_edges);
        let hyper_graph = build_graph_with_hyperedges(&rt, n_nodes, n_edges);

        group.bench_with_input(
            BenchmarkId::new("binary_edge_lookup", format!("n={n_nodes}_e={n_edges}")),
            &n_nodes,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    black_box(
                        binary_graph
                            .neighbors(black_box(EntityId::new(0)))
                            .await
                            .unwrap_or_default(),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("hyperedge_2ary_lookup", format!("n={n_nodes}_e={n_edges}")),
            &n_nodes,
            |b, _| {
                b.iter(|| {
                    black_box(hyper_graph.hyperedges_for_entity(black_box(EntityId::new(0))))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_binary_edge_regression);
criterion_main!(benches);
