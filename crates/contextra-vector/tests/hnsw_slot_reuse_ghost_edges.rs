use contextra_core::{DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{GhostFreeVectorIndex, HnswConfig, HnswIndex};
use std::sync::atomic::Ordering;

/// Helper function to count stale incoming ghost edges across all nodes and layers in RAM.
///
/// A "ghost edge" to target slot `slot_idx` occurs when node `A` (where `A != slot_idx`)
/// has `slot_idx` in its connection list at layer `L`, BUT node `slot_idx` does NOT have `A`
/// in its connection list at layer `L` (or when slot `slot_idx` is considered a new node that
/// did not generate `A` in its `final_connections`).
///
/// Specifically, we check for any node `A` pointing to `target_slot` where `target_slot`'s connection
/// list at that layer does NOT contain `A`.
fn count_incoming_ghost_edges_for_slot(index: &HnswIndex, target_slot: usize) -> usize {
    let mmap_count = index
        .inner_core()
        .cold
        .mmap_index
        .read()
        .as_ref()
        .map(|m| m.header.node_count() as usize)
        .unwrap_or(0);

    let nodes = index.inner_core().hot.nodes.read();
    let total_nodes = mmap_count + nodes.len();
    let m = index.inner_core().cold.config.m;

    let target_ram_idx = target_slot.saturating_sub(mmap_count);
    let index_max_layer = index.inner_core().hot.max_layer.load(Ordering::SeqCst) as usize;

    let mut ghost_edges = 0usize;

    for i in 0..total_nodes {
        if i == target_slot {
            continue;
        }

        for layer in 0..=index_max_layer {
            let conns = if i < mmap_count {
                index
                    .inner_core()
                    .get_node_connections(i, layer)
                    .unwrap_or_default()
            } else {
                let ram_i = i - mmap_count;
                index
                    .inner_core()
                    .hot
                    .arena
                    .get_ram_node_connections(ram_i, layer, m)
            };

            if conns.contains(&(target_slot as u32)) {
                // Check if target_slot points back to i at this layer
                let target_conns = if target_slot < mmap_count {
                    index
                        .inner_core()
                        .get_node_connections(target_slot, layer)
                        .unwrap_or_default()
                } else {
                    index
                        .inner_core()
                        .hot
                        .arena
                        .get_ram_node_connections(target_ram_idx, layer, m)
                };

                if !target_conns.contains(&(i as u32)) {
                    ghost_edges += 1;
                }
            }
        }
    }

    ghost_edges
}

/// Helper to count any node pointing to `target_slot` regardless of backlinks (total inbound edges).
fn count_total_inbound_edges_for_slot(index: &HnswIndex, target_slot: usize) -> usize {
    let mmap_count = index
        .inner_core()
        .cold
        .mmap_index
        .read()
        .as_ref()
        .map(|m| m.header.node_count() as usize)
        .unwrap_or(0);

    let nodes = index.inner_core().hot.nodes.read();
    let total_nodes = mmap_count + nodes.len();
    let m = index.inner_core().cold.config.m;
    let index_max_layer = index.inner_core().hot.max_layer.load(Ordering::SeqCst) as usize;

    let mut inbound = 0usize;

    for i in 0..total_nodes {
        if i == target_slot {
            continue;
        }

        for layer in 0..=index_max_layer {
            let conns = if i < mmap_count {
                index
                    .inner_core()
                    .get_node_connections(i, layer)
                    .unwrap_or_default()
            } else {
                let ram_i = i - mmap_count;
                index
                    .inner_core()
                    .hot
                    .arena
                    .get_ram_node_connections(ram_i, layer, m)
            };

            if conns.contains(&(target_slot as u32)) {
                inbound += 1;
            }
        }
    }

    inbound
}

fn generate_distinct_vector(i: usize, dim: usize) -> Vec<f32> {
    let mut vec = vec![0.0f32; dim];
    vec[i % dim] = 1.0;
    vec[(i + 1) % dim] = (i as f32) / 60.0;
    vec
}

#[tokio::test]
async fn test_slot_reuse_ghost_edges_path_a_remove_with_graph_repair(
) -> Result<(), Box<dyn std::error::Error>> {
    let dim = 8;
    let config = HnswConfig {
        dimension: dim,
        m: 8,
        ef_construction: 32,
        ..Default::default()
    };

    let mut index = HnswIndex::try_new(config)?;

    // Insert 60 nodes with distinct vector directions
    for i in 1..=60 {
        let doc_id = DocId::new(i as u64);
        let vector = generate_distinct_vector(i, dim);
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    // Node to delete: doc_id 25 (RAM slot 24)
    let target_doc = DocId::new(25);
    let target_slot = 24usize;

    // Perform path (a) deletion: remove_with_graph_repair
    let stats = index.remove_with_graph_repair(target_doc)?;
    assert_eq!(stats.doc_id, target_doc);
    assert!(stats.verified_no_ghost_pointers);

    // VOR dem Neu-Insert: Prüfen, dass keine Verbindungsliste mehr den alten Slot enthält
    let inbound_before_reinsert = count_total_inbound_edges_for_slot(&index, target_slot);
    println!(
        "[Path A - remove_with_graph_repair] Stale inbound edges VOR Neu-Insert: {inbound_before_reinsert}"
    );
    assert_eq!(
        inbound_before_reinsert, 0,
        "PFAD A BEFUND (VOR Neu-Insert): Es existieren noch {inbound_before_reinsert} eingehende Kanten auf den gelöschten Slot {target_slot}"
    );

    // Verify free_list contains target_slot for slot reuse
    let free_list_has_slot = index
        .inner_core()
        .hot
        .arena
        .free_list
        .read()
        .contains(&target_slot);
    assert!(
        free_list_has_slot,
        "Expected free_list to contain slot {target_slot}"
    );

    // Neu-Insert: doc_id 1000
    let new_doc = DocId::new(1000);
    let new_vec = generate_distinct_vector(25, dim);
    index.insert(TxId::new(1000), new_doc, &new_vec).await?;
    index.commit(TxId::new(1000)).await?;

    // Verify slot reuse: new_doc should occupy target_slot (RAM index 24)
    let reused_slot = {
        let doc_map = index.inner_core().hot.doc_to_node.read();
        *doc_map.get(&1000).expect("new_doc mapped in doc_to_node") as usize
    };
    assert_eq!(
        reused_slot, target_slot,
        "Expected slot reuse for RAM slot {target_slot}, got {reused_slot}"
    );

    // NACH dem Neu-Insert: Prüfen, dass nur symmetrische Kanten existieren, die zum neuen Knoten gehören
    let ghost_edges_after = count_incoming_ghost_edges_for_slot(&index, target_slot);
    println!(
        "[Path A - remove_with_graph_repair] Ghost-Kanten NACH Neu-Insert: {ghost_edges_after}"
    );
    assert_eq!(
        ghost_edges_after, 0,
        "PFAD A BEFUND (NACH Neu-Insert): Es wurden {ghost_edges_after} Geisterkanten auf den wiederverwendeten Slot {target_slot} gefunden!"
    );

    // Direct search verification: search must never return the deleted target_doc (25)
    let search_res = index.search(&new_vec, 10).await?;
    let found_docs: Vec<DocId> = search_res.into_iter().map(|doc| doc.doc_id).collect();
    assert!(
        !found_docs.contains(&target_doc),
        "Search must never return the deleted doc_id"
    );

    Ok(())
}

#[tokio::test]
async fn test_slot_reuse_ghost_edges_path_b_commit_deletion(
) -> Result<(), Box<dyn std::error::Error>> {
    let dim = 8;
    let config = HnswConfig {
        dimension: dim,
        m: 8,
        ef_construction: 32,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config)?;

    // Insert 60 nodes with distinct vector directions
    for i in 1..=60 {
        let doc_id = DocId::new(i as u64);
        let vector = generate_distinct_vector(i, dim);
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    // Node to delete: doc_id 30 (RAM slot 29)
    let target_doc = DocId::new(30);
    let target_slot = 29usize;

    // Perform path (b) deletion: delete(tx, doc_id) and commit(tx)
    let del_tx = TxId::new(100);
    index.delete(del_tx, target_doc).await?;
    index.commit(del_tx).await?;

    // VOR dem Neu-Insert: Prüfen, dass vor Neu-Insert eingehende Kanten für Path B erfasst werden
    let inbound_before_reinsert = count_total_inbound_edges_for_slot(&index, target_slot);
    println!(
        "[Path B - commit deletion] Inbound edges VOR Neu-Insert: {inbound_before_reinsert}"
    );

    // Verify free_list contains target_slot for slot reuse
    let free_list_has_slot = index
        .inner_core()
        .hot
        .arena
        .free_list
        .read()
        .contains(&target_slot);
    assert!(
        free_list_has_slot,
        "Expected free_list to contain slot {target_slot}"
    );

    // Neu-Insert: doc_id 2000
    let new_doc = DocId::new(2000);
    let new_vec = generate_distinct_vector(30, dim);
    let ins_tx = TxId::new(101);
    index.insert(ins_tx, new_doc, &new_vec).await?;
    index.commit(ins_tx).await?;

    // Verify slot reuse: new_doc should occupy target_slot (RAM index 29)
    let reused_slot = {
        let doc_map = index.inner_core().hot.doc_to_node.read();
        *doc_map.get(&2000).expect("new_doc mapped in doc_to_node") as usize
    };
    assert_eq!(
        reused_slot, target_slot,
        "Expected slot reuse for RAM slot {target_slot}, got {reused_slot}"
    );

    // NACH dem Neu-Insert: Prüfen, dass nur symmetrische Kanten existieren
    let ghost_edges_after = count_incoming_ghost_edges_for_slot(&index, target_slot);
    println!(
        "[Path B - commit deletion] Ghost-Kanten NACH Neu-Insert: {ghost_edges_after}"
    );
    assert_eq!(
        ghost_edges_after, 0,
        "PFAD B BEFUND (NACH Neu-Insert): Es wurden {ghost_edges_after} Geisterkanten auf den wiederverwendeten Slot {target_slot} gefunden!"
    );

    // Direct search verification: search must never return the deleted target_doc (30)
    let search_res = index.search(&new_vec, 10).await?;
    let found_docs: Vec<DocId> = search_res.into_iter().map(|doc| doc.doc_id).collect();
    assert!(
        !found_docs.contains(&target_doc),
        "Search must never return the deleted doc_id"
    );

    Ok(())
}
