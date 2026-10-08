use contextra_core::{ContextraError, DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{GhostFreeVectorIndex, HnswConfig, HnswIndex};

#[tokio::test]
async fn test_mmap_ghost_pointer_repair_and_search() -> contextra_core::Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("mmap_repair_test.hnsw");

    let config = HnswConfig {
        dimension: 4,
        m: 16,
        ef_construction: 64,
        ef_search: 32,
        ..Default::default()
    };

    // 1. Build index with A (doc 1), B (doc 2), C (doc 3) where B is connected to A and C.
    let index = HnswIndex::try_new(config.clone())?;
    let tx1 = TxId::new(1);

    let vec_a = vec![1.0, 0.0, 0.0, 0.0];
    let vec_b = vec![0.9, 0.1, 0.0, 0.0];
    let vec_c = vec![0.8, 0.2, 0.0, 0.0];

    index.insert(tx1, DocId::new(1), &vec_a).await?;
    index.insert(tx1, DocId::new(2), &vec_b).await?;
    index.insert(tx1, DocId::new(3), &vec_c).await?;
    index.commit(tx1).await?;

    index.save(&path).await?;

    // 2. Load as mmap index
    let mmap_index = HnswIndex::try_new(config)?;
    mmap_index.load_mmap(&path).await?;

    // 3. Remove node A with graph repair
    let mut repair_index = mmap_index;
    let stats = repair_index.remove_with_graph_repair(DocId::new(1))?;

    assert_eq!(stats.doc_id, DocId::new(1));
    assert!(stats.verified_no_ghost_pointers);

    // 4. Search for vector A: doc 1 must not be returned
    let search_results = repair_index.search(&vec_a, 10).await?;
    let returned_doc_ids: Vec<DocId> = search_results.iter().map(|r| r.doc_id).collect();
    assert!(!returned_doc_ids.contains(&DocId::new(1)));

    // 5. Connectivity: Search near C returns C
    let search_c = repair_index.search(&vec_c, 5).await?;
    let returned_c_ids: Vec<DocId> = search_c.iter().map(|r| r.doc_id).collect();
    assert!(returned_c_ids.contains(&DocId::new(3)));

    Ok(())
}

#[tokio::test]
async fn test_overlay_cleared_on_reload() -> contextra_core::Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("mmap_reload_test.hnsw");

    let config = HnswConfig {
        dimension: 4,
        m: 16,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config.clone())?;
    let tx1 = TxId::new(1);
    index
        .insert(tx1, DocId::new(1), &[1.0, 0.0, 0.0, 0.0])
        .await?;
    index
        .insert(tx1, DocId::new(2), &[0.9, 0.1, 0.0, 0.0])
        .await?;
    index.commit(tx1).await?;
    index.save(&path).await?;

    let mmap_index = HnswIndex::try_new(config.clone())?;
    mmap_index.load_mmap(&path).await?;

    let mut repair_index = mmap_index;
    let _ = repair_index.remove_with_graph_repair(DocId::new(1))?;

    assert!(repair_index.mmap_overlay_len() > 0);

    // Reload mmap index -> overlay must be cleared
    repair_index.load_mmap(&path).await?;
    assert_eq!(repair_index.mmap_overlay_len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_concurrent_search_during_repair() -> contextra_core::Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("mmap_concurrent_test.hnsw");

    let config = HnswConfig {
        dimension: 4,
        m: 16,
        ..Default::default()
    };

    let index = HnswIndex::try_new(config.clone())?;
    let tx1 = TxId::new(1);
    for i in 1..=20u64 {
        let vec = vec![(i as f32) / 20.0, 0.0, 0.0, 0.0];
        index.insert(tx1, DocId::new(i), &vec).await?;
    }
    index.commit(tx1).await?;
    index.save(&path).await?;

    let mut mmap_index = HnswIndex::try_new(config)?;
    mmap_index.load_mmap(&path).await?;

    // Clone HnswIndex handle for concurrent search tasks
    let index_clone = mmap_index.clone();
    let search_handle = tokio::spawn(async move {
        let query = vec![0.5, 0.0, 0.0, 0.0];
        for _ in 0..100 {
            let res = index_clone.search(&query, 5).await;
            assert!(res.is_ok());
            tokio::task::yield_now().await;
        }
    });

    let stats = mmap_index.remove_with_graph_repair(DocId::new(1))?;
    assert!(stats.verified_no_ghost_pointers);

    search_handle.await.unwrap();

    let search_after = mmap_index.search(&[0.05, 0.0, 0.0, 0.0], 10).await?;
    let doc_ids: Vec<DocId> = search_after.iter().map(|r| r.doc_id).collect();
    assert!(!doc_ids.contains(&DocId::new(1)));

    Ok(())
}
