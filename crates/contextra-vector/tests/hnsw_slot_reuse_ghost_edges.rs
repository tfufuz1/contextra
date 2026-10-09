use contextra_core::{DocId, Result, TxId, VectorIndex};
use contextra_vector::hnsw::deletion::GhostFreeVectorIndex;
use contextra_vector::{HnswConfig, HnswIndex};

fn check_ghost_edges_for_slot(
    index: &HnswIndex,
    target_slot: usize,
    new_node_connections: &[Vec<u32>],
) -> (usize, usize) {
    let inner = index.inner_core();
    let total_nodes = inner.hot.nodes.read().len();

    let mut inbound_before_new_insert = 0usize;
    let mut ghost_edges_after_new_insert = 0usize;

    for i in 0..total_nodes {
        if inner.cold.deleted_nodes.read().contains(i as u64) {
            continue;
        }
        let mut max_layer = 0;
        while inner.get_node_connections(i, max_layer + 1).is_ok()
            && !inner.get_node_connections(i, max_layer + 1).unwrap().is_empty()
        {
            max_layer += 1;
        }
        for layer in 0..=max_layer {
            let conns = inner.get_node_connections(i, layer).unwrap_or_default();
            if conns.contains(&(target_slot as u32)) {
                inbound_before_new_insert += 1;
                let is_known_by_new_node = if layer < new_node_connections.len() {
                    new_node_connections[layer].contains(&(i as u32))
                } else {
                    false
                };
                if !is_known_by_new_node {
                    ghost_edges_after_new_insert += 1;
                }
            }
        }
    }

    (inbound_before_new_insert, ghost_edges_after_new_insert)
}

#[tokio::test]
async fn test_slot_reuse_ghost_edges_path_a_graph_repair() -> Result<()> {
    let dimension = 8;
    let m = 8;
    let config = HnswConfig {
        dimension,
        m,
        ef_construction: 32,
        ..Default::default()
    };

    let mut index = HnswIndex::try_new(config)?;

    // Insert 60 nodes
    for i in 1..=60 {
        let doc_id = DocId::new(i as u64);
        let vec = vec![(i as f32) / 60.0; dimension];
        index.insert(TxId::new(i as u64), doc_id, &vec).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    // Target node to delete: doc 15
    let delete_doc = DocId::new(15);
    let target_slot = {
        let doc_map = index.inner_core().hot.doc_to_node.read();
        doc_map
            .get(&delete_doc.inner())
            .copied()
            .expect("doc 15 exists") as usize
    };

    // Delete via path (a) remove_with_graph_repair
    let stats = index.remove_with_graph_repair(delete_doc)?;
    assert_eq!(stats.doc_id, delete_doc);
    assert!(stats.verified_no_ghost_pointers);

    // Verify slot is in free_list
    {
        let free_list = index.inner_core().hot.arena.free_list.read();
        assert!(
            free_list.contains(&target_slot),
            "Freed slot {target_slot} must be in free_list"
        );
    }

    // Check inbound edges before new insert
    let (inbound_before, _) = check_ghost_edges_for_slot(&index, target_slot, &[]);
    println!(
        "[Path A] Inbound edges pointing to slot {target_slot} after remove_with_graph_repair: {inbound_before}"
    );

    // Force slot reuse by inserting new node 1001 with vector close to doc 15's old position
    let new_doc = DocId::new(1001);
    let new_vec = vec![15.0 / 60.0; dimension];
    index.insert(TxId::new(1001), new_doc, &new_vec).await?;
    index.commit(TxId::new(1001)).await?;

    let reused_slot = {
        let doc_map = index.inner_core().hot.doc_to_node.read();
        doc_map
            .get(&new_doc.inner())
            .copied()
            .expect("new doc exists") as usize
    };

    assert_eq!(
        reused_slot, target_slot,
        "Slot {target_slot} should have been reused by new insert"
    );

    // Get new node's outgoing connections
    let mut new_node_max_layer = 0;
    while index.inner_core().get_node_connections(reused_slot, new_node_max_layer + 1).is_ok()
        && !index.inner_core().get_node_connections(reused_slot, new_node_max_layer + 1).unwrap().is_empty()
    {
        new_node_max_layer += 1;
    }
    let mut new_node_conns = Vec::new();
    for l in 0..=new_node_max_layer {
        new_node_conns.push(index.inner_core().get_node_connections(reused_slot, l)?);
    }

    let (_, ghost_edges) = check_ghost_edges_for_slot(&index, reused_slot, &new_node_conns);
    println!("[Path A] Stale/Ghost incoming edges after slot reuse: {ghost_edges}");

    assert_eq!(
        ghost_edges, 0,
        "Path A: Reused slot {reused_slot} must have 0 ghost edges"
    );

    Ok(())
}

#[tokio::test]
async fn test_slot_reuse_ghost_edges_path_b_commit_deletion() -> Result<()> {
    let dimension = 8;
    let m = 8;
    let config = HnswConfig {
        dimension,
        m,
        ef_construction: 32,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config)?;

    // Insert 60 nodes
    for i in 1..=60 {
        let doc_id = DocId::new(i as u64);
        let vec = vec![(i as f32) / 60.0; dimension];
        index.insert(TxId::new(i as u64), doc_id, &vec).await?;
        index.commit(TxId::new(i as u64)).await?;
    }

    // Target node to delete: doc 25
    let delete_doc = DocId::new(25);
    let target_slot = {
        let doc_map = index.inner_core().hot.doc_to_node.read();
        doc_map
            .get(&delete_doc.inner())
            .copied()
            .expect("doc 25 exists") as usize
    };

    // Delete via path (b) do_delete (standard commit deletion path)
    index.delete(TxId::new(9999), delete_doc).await?;
    index.commit(TxId::new(9999)).await?;

    // Check if slot was placed in free_list (if no retention pin is holding it)
    {
        let free_list = index.inner_core().hot.arena.free_list.read();
        assert!(
            free_list.contains(&target_slot),
            "Freed slot {target_slot} must be in free_list"
        );
    }

    // Check inbound edges before new insert
    let (inbound_before, _) = check_ghost_edges_for_slot(&index, target_slot, &[]);
    println!(
        "[Path B] Inbound edges pointing to slot {target_slot} after do_delete: {inbound_before}"
    );

    // Force slot reuse by inserting new node 2001
    let new_doc = DocId::new(2001);
    let new_vec = vec![25.0 / 60.0; dimension];
    index.insert(TxId::new(2001), new_doc, &new_vec).await?;
    index.commit(TxId::new(2001)).await?;

    let reused_slot = {
        let doc_map = index.inner_core().hot.doc_to_node.read();
        doc_map
            .get(&new_doc.inner())
            .copied()
            .expect("new doc exists") as usize
    };

    assert_eq!(
        reused_slot, target_slot,
        "Slot {target_slot} should have been reused by new insert"
    );

    // Get new node's outgoing connections
    let mut new_node_max_layer = 0;
    while index.inner_core().get_node_connections(reused_slot, new_node_max_layer + 1).is_ok()
        && !index.inner_core().get_node_connections(reused_slot, new_node_max_layer + 1).unwrap().is_empty()
    {
        new_node_max_layer += 1;
    }
    let mut new_node_conns = Vec::new();
    for l in 0..=new_node_max_layer {
        new_node_conns.push(index.inner_core().get_node_connections(reused_slot, l)?);
    }

    let (_, ghost_edges) = check_ghost_edges_for_slot(&index, reused_slot, &new_node_conns);
    println!("[Path B] Stale/Ghost incoming edges after slot reuse: {ghost_edges}");

    assert_eq!(
        ghost_edges, 0,
        "Path B: Reused slot {reused_slot} must have 0 ghost edges"
    );

    Ok(())
}
