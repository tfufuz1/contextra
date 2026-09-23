use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use contextra_core::{DocId, Entity, EntityId, GraphIndex, TxId};
use contextra_graph::csr::CsrGraph;
use std::sync::Arc;
use tokio::runtime::Runtime;

// Helper trait to support benchmark invocation syntax `graph.bfs(...)` and `graph.get_neighbors(...)`
pub trait CsrGraphBenchExt {
    #[allow(clippy::type_complexity)]
    fn bfs(
        &self,
        doc_id: DocId,
        max_hops: usize,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = contextra_core::Result<Vec<(EntityId, f32)>>>
                + Send
                + '_,
        >,
    >;

    fn get_neighbors(
        &self,
        doc_id: DocId,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = contextra_core::Result<Vec<EntityId>>> + Send + '_>,
    >;
}

impl CsrGraphBenchExt for CsrGraph {
    fn bfs(
        &self,
        doc_id: DocId,
        max_hops: usize,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = contextra_core::Result<Vec<(EntityId, f32)>>>
                + Send
                + '_,
        >,
    > {
        Box::pin(self.traverse(EntityId::new(doc_id.inner()), max_hops))
    }

    fn get_neighbors(
        &self,
        doc_id: DocId,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = contextra_core::Result<Vec<EntityId>>> + Send + '_>,
    > {
        Box::pin(self.neighbors(EntityId::new(doc_id.inner())))
    }
}

fn build_test_graph(rt: &Runtime, n_nodes: usize, n_edges: usize) -> Arc<CsrGraph> {
    let graph = Arc::new(CsrGraph::with_config(contextra_graph::csr::CsrGraphConfig {
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
        let tx = TxId::new(1);
        let mut seed = 42u64;
        for _ in 0..n_edges {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let src = (seed as usize) % n_nodes;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let dst = (seed as usize) % n_nodes;
            let _ = GraphIndex::add_edge(
                graph.as_ref(),
                tx,
                contextra_core::Edge::new(
                    EntityId::new(src as u64),
                    EntityId::new(dst as u64),
                    "link",
                )
                .with_weight(1.0),
            )
            .await;
        }
        graph.commit(tx).await.unwrap();
        graph.compact();
    });
    graph
}

fn build_test_graph_with_pending(
    rt: &Runtime,
    n_nodes: usize,
    n_edges: usize,
    pending_count: usize,
) -> Arc<CsrGraph> {
    let graph = build_test_graph(rt, n_nodes, n_edges);
    rt.block_on(async {
        let tx = TxId::new(999_999);
        let mut seed = 12345u64;
        for _ in 0..pending_count {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let src = (seed as usize) % n_nodes;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let dst = (seed as usize) % n_nodes;
            let _ = GraphIndex::add_edge(
                graph.as_ref(),
                tx,
                contextra_core::Edge::new(
                    EntityId::new(src as u64),
                    EntityId::new(dst as u64),
                    "pending_rel",
                )
                .with_weight(1.0),
            )
            .await;
        }
        graph.commit(tx).await.unwrap();
        // Crucial: do NOT call compact() here so pending edges remain in pending_edges delta buffer.
    });
    graph
}

fn bench_csr_traversal(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("CSR_Traversal");

    for (n, e) in [(1_000, 3_000), (10_000, 30_000)] {
        let graph = build_test_graph(&rt, n, e);

        // BFS-2-Hop-Traversal
        group.bench_with_input(
            BenchmarkId::new("bfs_2hop", format!("n={n}_e={e}")),
            &n,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    black_box(
                        graph
                            .bfs(black_box(DocId::new(0)), 2)
                            .await
                            .unwrap_or_default(),
                    )
                });
            },
        );

        // get_neighbors (single-hop)
        group.bench_with_input(
            BenchmarkId::new("get_neighbors", format!("n={n}_e={e}")),
            &n,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    black_box(
                        graph
                            .get_neighbors(black_box(DocId::new(0)))
                            .await
                            .unwrap_or_default(),
                    )
                });
            },
        );

        // Mit pending_edges (vor compact): Overhead messen
        let graph_dirty = build_test_graph_with_pending(&rt, n, e, 100); // 100 uncompacted edges
        group.bench_with_input(
            BenchmarkId::new("bfs_2hop_with_pending", format!("n={n}_e={e}_p=100")),
            &n,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    black_box(
                        graph_dirty
                            .bfs(black_box(DocId::new(0)), 2)
                            .await
                            .unwrap_or_default(),
                    )
                });
            },
        );
    }
    group.finish();
}

// REGRESSIONS-GATE: bfs_2hop mit 100 pending_edges darf maximal 30% langsamer sein
// als ohne pending_edges bei gleicher Graph-Größe.
// Begründung: compact_async() läuft im Insert-Pfad, BFS-Reads sollten pending_edges
// effizient überspringen können (O(pending) statt O(N)).

/*
Expected Baseline JSON (benchmarks/results/csr_traversal_baseline.json):
{
  "bfs_2hop_1k_3k_ns": 4500,
  "bfs_2hop_10k_30k_ns": 48000,
  "bfs_2hop_10k_100k_ns": 150000,
  "get_neighbors_1k_3k_ns": 350,
  "bfs_2hop_with_pending_10k_30k_ns": 52000
}
*/

criterion_group!(benches, bench_csr_traversal);
criterion_main!(benches);
