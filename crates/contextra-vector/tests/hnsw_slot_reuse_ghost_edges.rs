// FILE-CONTEXT
// STAND: 2026-10-08
// ZWECK: Slot-Wiederverwendung und Ghost-Kanten-Verifikation für HNSW-Graphen.
// INVARIANTEN: INV-DELETION-2, Zero-Panic, Keine Geisterkanten nach Slot-Wiederverwendung.

use contextra_core::{DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{GhostFreeVectorIndex, HnswConfig, HnswIndex};

fn count_incoming_edges(index: &HnswIndex, target_idx: u32, max_layer: usize) -> usize {
    let core = index.inner_core();
    let total_nodes = core.hot.nodes.read().len();
    let mut count = 0;
    for node_i in 0..total_nodes {
        if node_i as u32 == target_idx {
            continue;
        }
        for layer in 0..=max_layer {
            if let Ok(conns) = core.get_node_connections(node_i, layer) {
                for &c in &conns {
                    if c == target_idx {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

fn count_ghost_edges(index: &HnswIndex, target_idx: u32, max_layer: usize) -> usize {
    let core = index.inner_core();
    let total_nodes = core.hot.nodes.read().len();
    let mut ghost_count = 0;

    let mut new_node_conns_by_layer: Vec<Vec<u32>> = Vec::new();
    for layer in 0..=max_layer {
        let conns = core
            .get_node_connections(target_idx as usize, layer)
            .unwrap_or_default();
        new_node_conns_by_layer.push(conns);
    }

    for node_i in 0..total_nodes {
        if node_i as u32 == target_idx {
            continue;
        }
        for layer in 0..=max_layer {
            if let Ok(conns) = core.get_node_connections(node_i, layer) {
                if conns.contains(&target_idx) {
                    let is_chosen_neighbor = layer < new_node_conns_by_layer.len()
                        && new_node_conns_by_layer[layer].contains(&(node_i as u32));
                    if !is_chosen_neighbor {
                        ghost_count += 1;
                    }
                }
            }
        }
    }
    ghost_count
}

#[tokio::test]
async fn test_slot_reuse_path_a_remove_with_graph_repair() -> contextra_core::Result<()> {
    let dim = 16;
    let m = 16;
    let config = HnswConfig {
        dimension: dim,
        m,
        ef_construction: 64,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config.clone())?;

    // Insert 60 documents
    for i in 1..=60 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 60.0; dim];
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    let target_doc = DocId::new(25);
    let target_idx = {
        let doc_map = index.inner_core().hot.doc_to_node.read();
        *doc_map.get(&target_doc.inner()).expect("doc 25 exists") as u32
    };

    let max_layer = index.inner_core().hot.max_layer.load(std::sync::atomic::Ordering::Relaxed) as usize;

    let pre_incoming = count_incoming_edges(&index, target_idx, max_layer);
    assert!(
        pre_incoming > 0,
        "Target node {target_idx} should have incoming edges before deletion"
    );

    // Path (a): remove_with_graph_repair
    let mut mut_index = index;
    let stats = mut_index.remove_with_graph_repair(target_doc)?;
    assert!(stats.verified_no_ghost_pointers);

    // VOR Neu-Insert check
    let vor_insert_incoming = count_incoming_edges(&mut_index, target_idx, max_layer);
    println!("[PATH A] VOR Neu-Insert stale incoming edges: {vor_insert_incoming}");
    assert_eq!(
        vor_insert_incoming, 0,
        "Path (a) VOR Neu-Insert: no incoming edges should remain pointing to freed slot"
    );

    // Insert new document 100 to force slot reuse
    let new_doc = DocId::new(100);
    let new_vector = vec![0.5f32; dim];
    mut_index.insert(TxId::new(100), new_doc, &new_vector).await?;
    mut_index.commit(TxId::new(100)).await?;

    let reused_idx = {
        let doc_map = mut_index.inner_core().hot.doc_to_node.read();
        *doc_map.get(&new_doc.inner()).expect("doc 100 inserted") as u32
    };

    assert_eq!(
        reused_idx, target_idx,
        "New node should reuse freed slot {target_idx}"
    );

    // NACH Neu-Insert check
    let nach_insert_ghosts = count_ghost_edges(&mut_index, target_idx, max_layer);
    println!("[PATH A] NACH Neu-Insert ghost edges: {nach_insert_ghosts}");
    assert_eq!(
        nach_insert_ghosts, 0,
        "Path (a) NACH Neu-Insert: zero ghost edges should exist"
    );

    Ok(())
}

#[tokio::test]
async fn test_slot_reuse_path_b_commit_deletion() -> contextra_core::Result<()> {
    let dim = 16;
    let m = 16;
    let config = HnswConfig {
        dimension: dim,
        m,
        ef_construction: 64,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config.clone())?;

    // Insert 60 documents
    for i in 1..=60 {
        let doc_id = DocId::new(i as u64);
        let vector = vec![(i as f32) / 60.0; dim];
        index.insert(TxId::new(i as u64), doc_id, &vector).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    let target_doc = DocId::new(25);
    let target_idx = {
        let doc_map = index.inner_core().hot.doc_to_node.read();
        *doc_map.get(&target_doc.inner()).expect("doc 25 exists") as u32
    };

    let max_layer = index.inner_core().hot.max_layer.load(std::sync::atomic::Ordering::Relaxed) as usize;

    let pre_incoming = count_incoming_edges(&index, target_idx, max_layer);
    assert!(
        pre_incoming > 0,
        "Target node {target_idx} should have incoming edges before deletion"
    );

    // Path (b): delete + commit
    index.delete(TxId::new(999), target_doc).await?;
    index.commit(TxId::new(999)).await?;

    // VOR Neu-Insert check
    let vor_insert_incoming = count_incoming_edges(&index, target_idx, max_layer);
    println!("[PATH B] VOR Neu-Insert stale incoming edges: {vor_insert_incoming}");

    // Insert new document 100 to force slot reuse
    let new_doc = DocId::new(100);
    let new_vector = vec![0.5f32; dim];
    index.insert(TxId::new(100), new_doc, &new_vector).await?;
    index.commit(TxId::new(100)).await?;

    let reused_idx = {
        let doc_map = index.inner_core().hot.doc_to_node.read();
        *doc_map.get(&new_doc.inner()).expect("doc 100 inserted") as u32
    };

    assert_eq!(
        reused_idx, target_idx,
        "New node should reuse freed slot {target_idx}"
    );

    // NACH Neu-Insert check
    let nach_insert_ghosts = count_ghost_edges(&index, target_idx, max_layer);
    println!("[PATH B] NACH Neu-Insert ghost edges: {nach_insert_ghosts}");

    assert_eq!(
        vor_insert_incoming, 0,
        "Path (b) VOR Neu-Insert: no incoming edges should remain pointing to freed slot (found {vor_insert_incoming})"
    );
    assert_eq!(
        nach_insert_ghosts, 0,
        "Path (b) NACH Neu-Insert: zero ghost edges should exist (found {nach_insert_ghosts})"
    );

    Ok(())
}
