//! Integration tests for [`PprCandidateStream`] (Spec §21.2 / AK-17).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_types::{ContextraError, DocId, Edge, Entity, EntityId, PprConfig, TxId};
use contextra_ports::{GraphIndex, GraphIndexStats};
use contextra_graph::{CsrGraph, PprCandidateStream, DEFAULT_PPR_STREAM_BATCH_SIZE};
use std::collections::HashSet;

struct MockNoPprGraph;

impl GraphIndex for MockNoPprGraph {
    fn traverse<'a>(
        &'a self,
        _start_node: EntityId,
        _max_hops: usize,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move { Ok(vec![]) })
    }

    fn add_entity<'a>(
        &'a self,
        _tx: TxId,
        _entity: Entity,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn add_edge<'a>(
        &'a self,
        _tx: TxId,
        _edge: Edge,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn commit<'a>(&'a self, _tx: TxId) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback<'a>(&'a self, _tx: TxId) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback_to_tx<'a>(
        &'a self,
        _tx_id: TxId,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn last_tx_id<'a>(&'a self) -> contextra_ports::BoxFuture<'a, contextra_types::Result<TxId>> {
        Box::pin(async move { Ok(TxId::new(0)) })
    }

    fn len<'a>(&'a self) -> contextra_ports::BoxFuture<'a, usize> {
        Box::pin(async move { 0 })
    }

    fn stats<'a>(
        &'a self,
    ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<GraphIndexStats>> {
        Box::pin(async move {
            Ok(GraphIndexStats {
                num_entities: 0,
                num_edges: 0,
                memory_usage_bytes: 0,
            })
        })
    }
}

#[tokio::test]
async fn test_chain_graph_batches_and_ordering() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    const NODE_COUNT: usize = 45;

    for i in 1..=NODE_COUNT {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i as u64), format!("N{i}"), "Node"))
            .await
            .unwrap();
    }

    for i in 1..NODE_COUNT {
        graph
            .add_edge(
                tx,
                Edge::new(
                    EntityId::new(i as u64),
                    EntityId::new((i + 1) as u64),
                    "next",
                ),
            )
            .await
            .unwrap();
    }
    graph.commit(tx).await.unwrap();

    let seeds = vec![EntityId::new(1)];
    let mut stream = PprCandidateStream::new(&graph, seeds, PprConfig::default());

    assert_eq!(PprCandidateStream::<CsrGraph>::max_depth(), 1000);
    assert!(!stream.is_exhausted());

    let mut all_yielded = Vec::new();
    let mut batch_count = 0;

    loop {
        let batch = stream.next_batch().await.unwrap();
        if batch.is_empty() {
            break;
        }
        assert!(
            batch.len() <= DEFAULT_PPR_STREAM_BATCH_SIZE,
            "Batch size {} exceeds max batch size {}",
            batch.len(),
            DEFAULT_PPR_STREAM_BATCH_SIZE
        );
        batch_count += 1;
        all_yielded.extend(batch);
    }

    assert_eq!(all_yielded.len(), NODE_COUNT);
    assert_eq!(stream.yielded(), NODE_COUNT);
    assert!(stream.is_exhausted());
    assert_eq!(batch_count, 3); // 16 + 16 + 13 = 45

    let unique_eids: HashSet<EntityId> = all_yielded.iter().map(|(eid, _)| *eid).collect();
    assert_eq!(unique_eids.len(), NODE_COUNT);

    for window in all_yielded.windows(2) {
        let (eid_a, score_a) = window[0];
        let (eid_b, score_b) = window[1];
        assert!(
            score_a >= score_b,
            "Score order violated: {score_a} < {score_b}"
        );
        if score_a == score_b {
            assert!(
                eid_a < eid_b,
                "Tie breaker violated: EntityId({:?}) should be < EntityId({:?})",
                eid_a,
                eid_b
            );
        }
    }
}

#[tokio::test]
async fn test_snapshot_behavior_isolated_from_subsequent_mutations() {
    let graph = CsrGraph::new();
    let tx1 = TxId::new(1);

    for i in 1..=20 {
        graph
            .add_entity(tx1, Entity::new(EntityId::new(i), format!("N{i}"), "Node"))
            .await
            .unwrap();
        if i > 1 {
            graph
                .add_edge(tx1, Edge::new(EntityId::new(1), EntityId::new(i), "edge"))
                .await
                .unwrap();
        }
    }
    graph.commit(tx1).await.unwrap();

    let seeds = vec![EntityId::new(1)];
    let mut stream = PprCandidateStream::new(&graph, seeds, PprConfig::default());

    let b1 = stream.next_batch().await.unwrap();
    assert_eq!(b1.len(), DEFAULT_PPR_STREAM_BATCH_SIZE);

    let tx2 = TxId::new(2);
    for i in 21..=30 {
        graph
            .add_entity(tx2, Entity::new(EntityId::new(i), format!("N{i}"), "Node"))
            .await
            .unwrap();
        graph
            .add_edge(tx2, Edge::new(EntityId::new(1), EntityId::new(i), "new_edge"))
            .await
            .unwrap();
    }
    graph.commit(tx2).await.unwrap();

    let b2 = stream.next_batch().await.unwrap();
    assert_eq!(b2.len(), 4);

    let b3 = stream.next_batch().await.unwrap();
    assert!(b3.is_empty());
    assert_eq!(stream.yielded(), 20);
}

#[tokio::test]
async fn test_exclude_seeds_option() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    let seed_id = EntityId::new(1);

    for i in 1..=5 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("N{i}"), "Node"))
            .await
            .unwrap();
    }
    for i in 1..5 {
        graph
            .add_edge(
                tx,
                Edge::new(EntityId::new(i), EntityId::new(i + 1), "edge"),
            )
            .await
            .unwrap();
    }
    graph.commit(tx).await.unwrap();

    let mut stream = PprCandidateStream::new(&graph, vec![seed_id], PprConfig::default())
        .with_exclude_seeds(true);

    let mut yielded = Vec::new();
    while let Some(eid) = stream.next_entity().await.unwrap() {
        yielded.push(eid);
    }

    assert!(!yielded.contains(&seed_id));
    assert_eq!(yielded.len(), 4);
}

#[tokio::test]
async fn test_empty_seeds_exhausted_immediately() {
    let graph = CsrGraph::new();
    let empty_seeds: Vec<EntityId> = Vec::new();
    let mut stream = PprCandidateStream::new(&graph, empty_seeds, PprConfig::default());

    assert!(stream.is_exhausted());
    let batch = stream.next_batch().await.unwrap();
    assert!(batch.is_empty());
    assert_eq!(stream.yielded(), 0);
    assert!(stream.is_exhausted());

    let single = stream.next_entity().await.unwrap();
    assert!(single.is_none());
}

#[tokio::test]
async fn test_unsupported_ppr_capability_propagated() {
    let mock_graph = MockNoPprGraph;
    let seeds = vec![EntityId::new(1)];
    let mut stream = PprCandidateStream::new(&mock_graph, seeds, PprConfig::default());

    let res = stream.next_batch().await;
    match res {
        Err(ContextraError::CapabilityUnsupported { capability, .. }) => {
            assert_eq!(capability, "graph_ppr");
        }
        _ => panic!("Expected CapabilityUnsupported error, got {res:?}"),
    }
}

#[tokio::test]
async fn test_next_batch_docs_skips_unresolvable_entities() {
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
                Edge::new(EntityId::new(i), EntityId::new(i + 1), "edge"),
            )
            .await
            .unwrap();
    }
    graph.commit(tx).await.unwrap();

    let seeds = vec![EntityId::new(1)];
    let mut stream = PprCandidateStream::new(&graph, seeds, PprConfig::default())
        .with_batch_size(4);

    let resolver = |eid: EntityId| {
        if eid.inner() % 2 == 0 {
            Some(DocId::new(eid.inner()))
        } else {
            None
        }
    };

    let doc_batch1 = stream.next_batch_docs(resolver).await.unwrap();
    assert_eq!(doc_batch1.len(), 4);
    for doc_id in &doc_batch1 {
        assert_eq!(doc_id.inner() % 2, 0);
    }
}

#[tokio::test]
async fn test_determinism_across_multiple_streams() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);

    for i in 1..=15 {
        graph
            .add_entity(tx, Entity::new(EntityId::new(i), format!("N{i}"), "Node"))
            .await
            .unwrap();
    }
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(1), EntityId::new(2), "edge").with_weight(1.0),
        )
        .await
        .unwrap();
    graph
        .add_edge(
            tx,
            Edge::new(EntityId::new(1), EntityId::new(3), "edge").with_weight(1.0),
        )
        .await
        .unwrap();
    graph.commit(tx).await.unwrap();

    let seeds = vec![EntityId::new(1)];
    let mut stream1 = PprCandidateStream::new(&graph, seeds.clone(), PprConfig::default());
    let mut stream2 = PprCandidateStream::new(&graph, seeds, PprConfig::default());

    let mut res1 = Vec::new();
    while let Some(item) = stream1.next_entity().await.unwrap() {
        res1.push(item);
    }

    let mut res2 = Vec::new();
    while let Some(item) = stream2.next_entity().await.unwrap() {
        res2.push(item);
    }

    assert_eq!(res1, res2);
}
