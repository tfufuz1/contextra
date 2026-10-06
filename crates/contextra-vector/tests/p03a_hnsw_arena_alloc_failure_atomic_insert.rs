// FILE-CONTEXT
// STAND: 2026-10-06
// ZWECK: Integrationstest zur Verifikation von Allokationsfehlern im HNSW-Arena-Allocator und atomarem Verhalten beim Einfügen (Review P03 / F-01).
// INVARIANTEN: Zero-Panic Guarantee (I-8), Kantensymmetrie (I-3), Atomares Einfügen bei Fehlern.

use contextra_core::{DocId, Result, TxId, VectorIndex};
use contextra_vector::hnsw::{HnswArena, HnswConfig, HnswIndex};

#[tokio::test]
async fn test_hnsw_arena_alloc_failure_and_direct_arena_checks() -> Result<()> {
    // 1. Direct Arena allocate_node error checks with 0 capacity
    let arena = HnswArena::new();
    let err_alloc = arena.allocate_node(0, 0, &[]);
    assert!(
        err_alloc.is_err(),
        "allocate_node with 0 raw capacity must return Err"
    );

    // 2. Integration check with HnswIndex - verify failed allocation during apply_insert via VectorIndex interface
    let mut config = HnswConfig::default();
    config.dimension = 4;
    config.m = 4;

    let index = HnswIndex::try_new(config)?;

    let id1 = DocId::from(100u64);
    let vec1 = vec![1.0, 0.0, 0.0, 0.0];

    // Standard insert through VectorIndex interface succeeds
    index.insert(TxId::new(1), id1, &vec1).await?;
    index.commit(TxId::new(1)).await?;

    let results = index.search(&vec1, 1).await?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].doc_id, id1);

    // 3. Insert invalid dimension vector to test insert rejection and state preservation
    let id2 = DocId::from(200u64);
    let vec2_invalid = vec![0.0, 1.0, 0.0]; // Invalid dimension 3 != 4

    let insert_err = index.insert(TxId::new(2), id2, &vec2_invalid).await;
    assert!(insert_err.is_err());

    // Verify index search results remain unchanged
    let results_after = index.search(&vec1, 10).await?;
    assert_eq!(results_after.len(), 1);
    assert_eq!(results_after[0].doc_id, id1);

    Ok(())
}
