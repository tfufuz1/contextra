#![allow(
    clippy::all,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    warnings
)]
use contextra_ports::{EraseTarget, ErasureLayer, KeyCoverage, VectorIndex};
use contextra_testmacros::ctx_test;
use contextra_types::{DocId, Result, TxId};
use contextra_vector::{HnswConfigBuilder, HnswIndex};

#[ctx_test(id = "T-VEC-DEL-1")]
async fn test_surrogate_zeroizes_original_vector() -> Result<()> {
    let config = HnswConfigBuilder::new(4).build()?;
    let index = HnswIndex::try_new(config)?;

    let v1 = vec![1.0, 0.0, 0.0, 0.0];
    let v2 = vec![0.0, 1.0, 0.0, 0.0];
    index.insert(TxId(1), DocId::new(1), &v1).await?;
    index.insert(TxId(1), DocId::new(2), &v2).await?;
    index.commit(TxId(1)).await?;

    let target = EraseTarget::ExactKeys(vec![1u64.to_be_bytes().to_vec()]);
    let scope = index
        .resolve(&target)
        .await
        .map_err(|e| contextra_types::ContextraError::Storage(e))?;
    index
        .on_gate(&scope)
        .await
        .map_err(|e| contextra_types::ContextraError::Storage(e))?;

    // Verify vector 1 is no longer equal to v1 in the arena
    let slice = index.inner.hot.arena.get_vector_slice(0, 4);
    let stored = slice.expect("Slot 0 vector must be present in arena");
    assert_ne!(
        &stored, &v1,
        "Original vector bytes must not remain in the arena"
    );

    Ok(())
}

#[ctx_test(id = "T-VEC-DEL-2")]
async fn test_batch_edge_repair_purges_references() -> Result<()> {
    let config = HnswConfigBuilder::new(4).build()?;
    let index = HnswIndex::try_new(config)?;

    let v1 = vec![1.0, 0.0, 0.0, 0.0];
    let v2 = vec![0.0, 1.0, 0.0, 0.0];
    index.insert(TxId(1), DocId::new(1), &v1).await?;
    index.insert(TxId(1), DocId::new(2), &v2).await?;
    index.commit(TxId(1)).await?;

    let target = EraseTarget::ExactKeys(vec![1u64.to_be_bytes().to_vec()]);
    let scope = index
        .resolve(&target)
        .await
        .map_err(|e| contextra_types::ContextraError::Storage(e))?;
    let cx = contextra_ports::PurgeCx::new(10_000);
    let outcome = index
        .purge(&scope, &cx)
        .await
        .map_err(|e| contextra_types::ContextraError::Storage(e))?;

    assert_eq!(outcome.purged_count, 1);
    let scanner = contextra_ports::Scanner::new();
    let witness = match index.witness(&scope, &scanner).await {
        Ok(w) => w,
        Err(e) => panic!("Witness error: {}", e),
    };
    assert_ne!(witness.witness_hash(), &[0u8; 32]);

    Ok(())
}

#[ctx_test(id = "T-VEC-ARENA-1")]
fn test_arena_contiguous_layout_and_zeroization() -> Result<()> {
    let arena = contextra_vector::hnsw::HnswArena::new();
    let vec = vec![0.5f32, 0.5, 0.5, 0.5];
    arena.store_vector(0, 4, &vec);

    let retrieved = arena.get_vector_slice(0, 4);
    assert_eq!(retrieved, Some(vec.clone()));

    arena.zeroize_vector(0, 4);
    let after_zeroize = arena.get_vector_slice(0, 4);
    assert_eq!(after_zeroize, Some(vec![0.0, 0.0, 0.0, 0.0]));

    Ok(())
}

#[ctx_test(id = "T-VEC-REBUILD-INDEP")]
async fn test_rebuild_independent_of_insertion_order() -> Result<()> {
    let config1 = HnswConfigBuilder::new(4).build()?;
    let index1 = HnswIndex::try_new(config1)?;

    let config2 = HnswConfigBuilder::new(4).build()?;
    let index2 = HnswIndex::try_new(config2)?;

    let v1 = vec![1.0, 0.0, 0.0, 0.0];
    let v2 = vec![0.0, 1.0, 0.0, 0.0];

    // Order A
    index1.insert(TxId(1), DocId::new(1), &v1).await?;
    index1.insert(TxId(1), DocId::new(2), &v2).await?;
    index1.rebuild().await?;

    // Order B
    index2.insert(TxId(1), DocId::new(2), &v2).await?;
    index2.insert(TxId(1), DocId::new(1), &v1).await?;
    index2.rebuild().await?;

    assert_eq!(index1.len().await, index2.len().await);
    assert_eq!(
        index1.inner.hot.get_entry_point(),
        index2.inner.hot.get_entry_point()
    );
    let conns1 = index1.inner.hot.arena.get_ram_node_connections(0, 0, 16);
    let conns2 = index2.inner.hot.arena.get_ram_node_connections(0, 0, 16);
    assert_eq!(
        conns1, conns2,
        "Graph neighborhood must be bit-identical after deterministic rebuild"
    );
    Ok(())
}

#[ctx_test(id = "T-VEC-5")]
fn test_snapshot_vfs_serialization_roundtrip() -> Result<()> {
    let header = contextra_vector::persistence::VectorSnapshotHeader {
        dimension: 128,
        metric: 0,
        m: 16,
        ef_construction: 200,
        slot_count: 50,
        entry_slot: 0,
        max_level: 3,
        built_through_seq: 100,
    };

    let bytes = header.to_bytes();
    let parsed = contextra_vector::persistence::VectorSnapshotHeader::parse(&bytes)?;
    assert_eq!(header, parsed);

    Ok(())
}

#[ctx_test(id = "T-VEC-6")]
fn test_key_coverage_classification() {
    let config = HnswConfigBuilder::new(4).build().unwrap();
    let index = HnswIndex::try_new(config).unwrap();
    assert_eq!(index.key_coverage(), KeyCoverage::None);
}

#[ctx_test(id = "T-VEC-7")]
fn test_deterministic_layer_assignment() {
    let l1 = contextra_vector::hnsw::HnswIndexCore::deterministic_layer_for_doc(42, 0);
    let l2 = contextra_vector::hnsw::HnswIndexCore::deterministic_layer_for_doc(42, 0);
    assert_eq!(
        l1, l2,
        "Deterministic layer assignment must yield bit-identical layer"
    );
    assert!(l1 <= 15, "Layer must be bounded to 15");
}

#[ctx_test(id = "T-VEC-8")]
fn test_rebuild_required_detection() -> Result<()> {
    let config = HnswConfigBuilder::new(4).build()?;
    let index = HnswIndex::try_new(config)?;
    assert!(!index.is_rebuild_required());
    Ok(())
}
