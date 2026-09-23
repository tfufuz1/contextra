use contextra_core::{DocId, Edge, Entity, EntityId, GraphIndex, TxId};
use contextra_graph::{CsrGraph, PathGraph};
use proptest::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// PROP-1: Adjazenzliste nach Compact enthält keine tombstonierten Kanten
    /// Für beliebige Insert-Sequenzen gefolgt von N Deletes: Nach compact() enthält keine Adjazenzliste
    /// eine Kante zu einem tombstoned-Knoten oder gelöschten Kante.
    #[test]
    fn prop_1_adjacency_after_compact_has_no_tombstones(
        num_nodes in 2u64..20u64,
        edges in proptest::collection::vec((0usize..20, 0usize..20), 1..40),
        delete_edges in proptest::collection::vec((0usize..20, 0usize..20), 0..10),
        delete_nodes in proptest::collection::vec(0usize..20, 0..5),
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let graph = Arc::new(CsrGraph::new());
            let tx1 = TxId::new(1);

            // 1. Entities einfügen
            for i in 1..=num_nodes {
                let id = EntityId::new(i);
                graph
                    .add_entity(tx1, Entity::new(id, format!("Entity_{i}"), "Type"))
                    .await
                    .unwrap();
            }

            // 2. Kanten einfügen
            for &(src_idx, dst_idx) in &edges {
                let src = EntityId::new((src_idx % num_nodes as usize) as u64 + 1);
                let dst = EntityId::new((dst_idx % num_nodes as usize) as u64 + 1);
                if src != dst {
                    let _ = GraphIndex::add_edge(
                        graph.as_ref(),
                        tx1,
                        Edge::new(src, dst, "relates").with_weight(1.0),
                    )
                    .await;
                }
            }
            graph.commit(tx1).await.unwrap();

            // 3. Kanten / Knoten löschen (Tombstones)
            let tx2 = TxId::new(2);
            let mut removed_edges: HashSet<(EntityId, EntityId)> = HashSet::new();
            for &(src_idx, dst_idx) in &delete_edges {
                let src = EntityId::new((src_idx % num_nodes as usize) as u64 + 1);
                let dst = EntityId::new((dst_idx % num_nodes as usize) as u64 + 1);
                let _ = graph.remove_edge(tx2, src, dst).await;
                removed_edges.insert((src, dst));
            }

            let mut removed_nodes: HashSet<EntityId> = HashSet::new();
            for &node_idx in &delete_nodes {
                let node_id = EntityId::new((node_idx % num_nodes as usize) as u64 + 1);
                let _ = graph.remove_entity(tx2, node_id).await;
                removed_nodes.insert(node_id);
            }
            graph.commit(tx2).await.unwrap();

            // 4. Compact ausführen
            graph.compact();

            // 5. Invariante prüfen: Keine Adjazenzliste enthält tombstonierte/gelöschte Kanten oder Knoten
            for i in 1..=num_nodes {
                let src = EntityId::new(i);
                if removed_nodes.contains(&src) {
                    continue;
                }

                let neighbors = graph.neighbors(src).await.unwrap();
                let weighted_neighbors = graph.neighbors_with_weights(src);

                for dst in &neighbors {
                    prop_assert!(
                        !removed_nodes.contains(dst),
                        "Adjazenzliste von {:?} enthält gelöschten Knoten {:?}",
                        src,
                        dst
                    );
                    prop_assert!(
                        !removed_edges.contains(&(src, *dst)),
                        "Adjazenzliste von {:?} enthält gelöschte Kante {:?}",
                        src,
                        dst
                    );
                }

                for (dst, _w) in &weighted_neighbors {
                    prop_assert!(
                        !removed_nodes.contains(dst),
                        "neighbors_with_weights({:?}) enthält gelöschten Knoten {:?}",
                        src,
                        dst
                    );
                    prop_assert!(
                        !removed_edges.contains(&(src, *dst)),
                        "neighbors_with_weights({:?}) enthält gelöschte Kante {:?}",
                        src,
                        dst
                    );
                }
            }
            Ok(())
        }).unwrap();
    }

    /// PROP-2: Kanten-Gewicht ist nicht-negativ
    /// Für beliebige Inserts mit weight ∈ [0.0..1000.0]: neighbors_with_weights(src) liefert alle Einträge mit weight ≥ 0.
    #[test]
    fn prop_2_edge_weights_non_negative(
        num_nodes in 2u64..15u64,
        edge_specs in proptest::collection::vec((0usize..15, 0usize..15, 0.0f32..1000.0f32), 1..30),
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let graph = Arc::new(CsrGraph::new());
            let tx = TxId::new(1);

            for i in 1..=num_nodes {
                graph
                    .add_entity(tx, Entity::new(EntityId::new(i), format!("N_{i}"), "Type"))
                    .await
                    .unwrap();
            }

            for &(src_idx, dst_idx, weight) in &edge_specs {
                let src = EntityId::new((src_idx % num_nodes as usize) as u64 + 1);
                let dst = EntityId::new((dst_idx % num_nodes as usize) as u64 + 1);
                if src != dst {
                    let _ = GraphIndex::add_edge(
                        graph.as_ref(),
                        tx,
                        Edge::new(src, dst, "rel").with_weight(weight),
                    )
                    .await;
                }
            }
            graph.commit(tx).await.unwrap();
            graph.compact();

            for i in 1..=num_nodes {
                let src = EntityId::new(i);
                let weighted_neighbors = graph.neighbors_with_weights(src);
                for (dst, weight) in weighted_neighbors {
                    prop_assert!(
                        weight >= 0.0,
                        "Kanten-Gewicht von {:?} -> {:?} ist negativ: {}",
                        src,
                        dst,
                        weight
                    );
                    prop_assert!(
                        weight.is_finite(),
                        "Kanten-Gewicht von {:?} -> {:?} ist nicht finite: {}",
                        src,
                        dst,
                        weight
                    );
                }
            }
            Ok(())
        }).unwrap();
    }

    /// PROP-3: source_doc_ids nach Compact nicht None (für inserte Knoten/Kanten)
    /// Für N Kanten: Nach compact() ist get_source_doc_id(i) für alle i in [0..num_inserted_sources] = Some(_).
    #[test]
    fn prop_3_source_doc_ids_not_none_after_compact(
        num_edges in 1usize..30usize,
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let graph = Arc::new(CsrGraph::new());
            let tx = TxId::new(1);

            let mut inserted_edges = Vec::new();

            for i in 0..num_edges {
                let src = EntityId::new((i * 2 + 1) as u64);
                let dst = EntityId::new((i * 2 + 2) as u64);
                let doc_id = DocId::new((1000 + i) as u64);

                graph
                    .add_entity(tx, Entity::new(src, format!("N_{src:?}"), "T"))
                    .await
                    .unwrap();
                graph
                    .add_entity(tx, Entity::new(dst, format!("N_{dst:?}"), "T"))
                    .await
                    .unwrap();

                let edge = Edge::new(src, dst, "link").with_source_doc_id(doc_id);
                GraphIndex::add_edge(graph.as_ref(), tx, edge)
                    .await
                    .unwrap();

                inserted_edges.push((src, dst, doc_id));
            }
            graph.commit(tx).await.unwrap();

            graph.compact();

            // Prp-3a: get_source_doc_id(idx) für alle indizes 0..num_edges ist Some(_)
            for idx in 0..num_edges {
                let source_doc_id = graph.get_source_doc_id(idx);
                prop_assert!(
                    source_doc_id.is_some(),
                    "get_source_doc_id({}) nach compact() ist None",
                    idx
                );
            }

            // Prop-3b: source_doc_id_at(from, to) liefert Some(doc_id) für alle eingefügten Kanten
            for (src, dst, expected_doc) in inserted_edges {
                let doc_id = graph.source_doc_id_at(src, dst);
                prop_assert_eq!(
                    doc_id,
                    Some(expected_doc),
                    "source_doc_id_at({:?}, {:?}) nach compact() stimmt nicht mit expected {:?}",
                    src,
                    dst,
                    expected_doc
                );
            }

            Ok(())
        }).unwrap();
    }

    /// PROP-4: compact() ist idempotent
    /// compact(); compact(); → gleiche Adjacency-Struktur wie compact() einmal.
    #[test]
    fn prop_4_compact_is_idempotent(
        num_nodes in 2u64..20u64,
        edge_specs in proptest::collection::vec((0usize..20, 0usize..20, 0.1f32..10.0f32), 1..40),
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let graph = Arc::new(CsrGraph::new());
            let tx = TxId::new(1);

            for i in 1..=num_nodes {
                graph
                    .add_entity(tx, Entity::new(EntityId::new(i), format!("N_{i}"), "Type"))
                    .await
                    .unwrap();
            }

            for &(src_idx, dst_idx, weight) in &edge_specs {
                let src = EntityId::new((src_idx % num_nodes as usize) as u64 + 1);
                let dst = EntityId::new((dst_idx % num_nodes as usize) as u64 + 1);
                if src != dst {
                    let _ = GraphIndex::add_edge(
                        graph.as_ref(),
                        tx,
                        Edge::new(src, dst, "rel").with_weight(weight),
                    )
                    .await;
                }
            }
            graph.commit(tx).await.unwrap();

            // 1. Erstes compact()
            graph.compact();

            let entity_count_1 = graph.entity_count();
            let edge_count_1 = graph.edge_count();

            let mut adj_1 = Vec::new();
            for i in 1..=num_nodes {
                let src = EntityId::new(i);
                let neighbors = graph.neighbors(src).await.unwrap();
                let weighted = graph.neighbors_with_weights(src);
                adj_1.push((src, neighbors, weighted));
            }

            // 2. Zweites compact()
            graph.compact();

            let entity_count_2 = graph.entity_count();
            let edge_count_2 = graph.edge_count();

            let mut adj_2 = Vec::new();
            for i in 1..=num_nodes {
                let src = EntityId::new(i);
                let neighbors = graph.neighbors(src).await.unwrap();
                let weighted = graph.neighbors_with_weights(src);
                adj_2.push((src, neighbors, weighted));
            }

            prop_assert_eq!(entity_count_1, entity_count_2);
            prop_assert_eq!(edge_count_1, edge_count_2);
            prop_assert_eq!(adj_1, adj_2, "Adjazenzstruktur nach 2. compact() unterscheidet sich vom 1. compact()");

            Ok(())
        }).unwrap();
    }

    /// PROP-5: Knotenanzahl monoton wachsend (kein implizites Löschen)
    /// Anzahl der Knoten nach compact ist ≥ Anzahl vor compact.
    #[test]
    fn prop_5_node_count_monotonically_non_decreasing(
        num_nodes in 1u64..25u64,
        edge_specs in proptest::collection::vec((0usize..25, 0usize..25), 0..30),
    ) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let graph = Arc::new(CsrGraph::new());
            let tx = TxId::new(1);

            for i in 1..=num_nodes {
                graph
                    .add_entity(tx, Entity::new(EntityId::new(i), format!("N_{i}"), "Type"))
                    .await
                    .unwrap();
            }

            for &(src_idx, dst_idx) in &edge_specs {
                let src = EntityId::new((src_idx % num_nodes as usize) as u64 + 1);
                let dst = EntityId::new((dst_idx % num_nodes as usize) as u64 + 1);
                if src != dst {
                    let _ = GraphIndex::add_edge(
                        graph.as_ref(),
                        tx,
                        Edge::new(src, dst, "rel"),
                    )
                    .await;
                }
            }
            graph.commit(tx).await.unwrap();

            let count_before = graph.entity_count();
            graph.compact();
            let count_after = graph.entity_count();

            prop_assert!(
                count_after >= count_before,
                "Knotenanzahl nach compact ({count_after}) ist kleiner als vor compact ({count_before})"
            );

            Ok(())
        }).unwrap();
    }
}
