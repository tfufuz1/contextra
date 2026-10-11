// FILE-CONTEXT
// STAND: 2026-10-06T00:00:00Z (SESSION: p02a-fix)
// ZWECK: Integrationstest für P02 F-01 CRITICAL — Compaction Engine respektiert MVCC TxBuffer Reader.
// INVARIANTEN: Invariante I-2: Reader bei Seq N über TxBuffer schützt Put(50)+Tombstone(90) vor vorzeitiger Purging/GC.

use contextra_core::{ResourceTracker, SnapshotRegistry, TxBuffer, TxId, TOMBSTONE_BIT};
use contextra_store::compaction::{CompactionConfig, CompactionEngine};
use contextra_store::lsm::GcFloor;
use contextra_store::sstable::{create_block_cache_with_shards, SstableBuilder, SstableReader};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_compaction_respects_txbuffer_active_reader() {
    let temp_dir = tempfile::tempdir().expect("tempdir created");
    let dir_path = temp_dir.path();

    let block_cache = create_block_cache_with_shards(16, 4);
    let budget = Arc::new(ResourceTracker::new(contextra_core::ResourceBudget {
        memory_limit: 128 * 1024 * 1024,
    }));
    let snapshot_registry = Arc::new(SnapshotRegistry::new());
    let tx_buffer = Arc::new(TxBuffer::<(Vec<u8>, Vec<u8>)>::new());

    let comp_config = CompactionConfig {
        min_sstables_per_tier: 2,
        enable_adaptive_compaction: false,
        ..Default::default()
    };

    let gc_floor: Arc<dyn contextra_mvcc::snapshot::SnapshotFloor> = Arc::new(GcFloor::new(
        Arc::clone(&snapshot_registry),
        Arc::clone(&tx_buffer),
        Arc::new(std::sync::atomic::AtomicU64::new(100)),
    ));

    let engine = CompactionEngine::new(
        comp_config,
        gc_floor,
        Arc::clone(&block_cache),
        None,
        budget,
        None,
    );

    // SSTable 1: Put(50) for key K = "key_k"
    let sst_path_1 = dir_path.join("sst1.sst");
    let mut builder1 = SstableBuilder::create(&sst_path_1)
        .await
        .expect("builder1 created");
    builder1
        .add(b"key_k", b"val_50", 50, 1)
        .await
        .expect("add 50");
    builder1.finish().await.expect("finish 1");
    let reader1 = Arc::new(
        SstableReader::open(&sst_path_1, Arc::clone(&block_cache))
            .await
            .expect("open reader 1"),
    );

    // SSTable 2: Tombstone(90) for key K = "key_k"
    let sst_path_2 = dir_path.join("sst2.sst");
    let mut builder2 = SstableBuilder::create(&sst_path_2)
        .await
        .expect("builder2 created");
    builder2
        .add(b"key_k", b"", 90 | TOMBSTONE_BIT, 2)
        .await
        .expect("add 90 tombstone");
    builder2.finish().await.expect("finish 2");
    let reader2 = Arc::new(
        SstableReader::open(&sst_path_2, Arc::clone(&block_cache))
            .await
            .expect("open reader 2"),
    );

    let sstables = Arc::new(RwLock::new(vec![reader1, reader2]));

    // Start an active reader transaction at snapshot seq 80 registered via TxBuffer
    let reader_tx = TxId::new(100);
    tx_buffer.begin(reader_tx);
    tx_buffer.register_read(reader_tx, b"key_k".to_vec(), 80);

    // Run full compaction
    let compacted = engine
        .maybe_compact(&sstables, dir_path)
        .await
        .expect("compact executed");
    assert!(compacted, "Compaction should merge 2 SSTables");

    // Read compacted output SSTable
    let sst_lock = sstables.read().await;
    assert_eq!(sst_lock.len(), 1);
    let merged_reader = &sst_lock[0];

    let mut stream = merged_reader.stream().await.expect("stream created");
    let mut entries = Vec::new();
    while let Some((k, v, seq, tx)) = stream.next_entry().await.expect("next_entry") {
        entries.push((k, v, seq, tx));
    }

    // Because reader at seq 80 is active, min_snapshot_seq_bound = min(MAX, 80) = 80.
    // Tombstone(90) has raw_seq = 90 > 80, so it is kept.
    // Put(50) has raw_seq = 50 <= 80 and is the floor version below 80, so it is also kept!
    // A reader querying at snapshot seq 80 (where 50 <= 80 < 90) sees Put(50) ("val_50").
    assert_eq!(
        entries.len(),
        2,
        "Both Tombstone(90) and Put(50) must be retained for reader at seq 80"
    );
    assert_eq!(entries[0].2, 90 | TOMBSTONE_BIT);
    assert_eq!(entries[1].2, 50);
    assert_eq!(entries[1].1.as_ref(), b"val_50");
}

#[tokio::test]
async fn test_compaction_without_active_readers_purges_tombstone() {
    let temp_dir = tempfile::tempdir().expect("tempdir created");
    let dir_path = temp_dir.path();

    let block_cache = create_block_cache_with_shards(16, 4);
    let budget = Arc::new(ResourceTracker::new(contextra_core::ResourceBudget {
        memory_limit: 128 * 1024 * 1024,
    }));
    let snapshot_registry = Arc::new(SnapshotRegistry::new());
    let tx_buffer = Arc::new(TxBuffer::<(Vec<u8>, Vec<u8>)>::new());

    let comp_config = CompactionConfig {
        min_sstables_per_tier: 2,
        enable_adaptive_compaction: false,
        ..Default::default()
    };

    let gc_floor: Arc<dyn contextra_mvcc::snapshot::SnapshotFloor> = Arc::new(GcFloor::new(
        Arc::clone(&snapshot_registry),
        Arc::clone(&tx_buffer),
        Arc::new(std::sync::atomic::AtomicU64::new(100)),
    ));

    let engine = CompactionEngine::new(
        comp_config,
        gc_floor,
        Arc::clone(&block_cache),
        None,
        budget,
        None,
    );

    // SSTable 1: Put(50) for key K = "key_k"
    let sst_path_1 = dir_path.join("sst1.sst");
    let mut builder1 = SstableBuilder::create(&sst_path_1)
        .await
        .expect("builder1 created");
    builder1
        .add(b"key_k", b"val_50", 50, 1)
        .await
        .expect("add 50");
    builder1.finish().await.expect("finish 1");
    let reader1 = Arc::new(
        SstableReader::open(&sst_path_1, Arc::clone(&block_cache))
            .await
            .expect("open reader 1"),
    );

    // SSTable 2: Tombstone(90) for key K = "key_k"
    let sst_path_2 = dir_path.join("sst2.sst");
    let mut builder2 = SstableBuilder::create(&sst_path_2)
        .await
        .expect("builder2 created");
    builder2
        .add(b"key_k", b"", 90 | TOMBSTONE_BIT, 2)
        .await
        .expect("add 90 tombstone");
    builder2.finish().await.expect("finish 2");
    let reader2 = Arc::new(
        SstableReader::open(&sst_path_2, Arc::clone(&block_cache))
            .await
            .expect("open reader 2"),
    );

    let sstables = Arc::new(RwLock::new(vec![reader1, reader2]));

    // NO active readers in TxBuffer or SnapshotRegistry -> min_snapshot_seq_bound = u64::MAX
    // Run full compaction
    let compacted = engine
        .maybe_compact(&sstables, dir_path)
        .await
        .expect("compact executed");
    assert!(compacted, "Compaction should merge 2 SSTables");

    let sst_lock = sstables.read().await;
    assert_eq!(sst_lock.len(), 1);
    let merged_reader = &sst_lock[0];

    let mut stream = merged_reader.stream().await.expect("stream created");
    let mut entries = Vec::new();
    while let Some((k, v, seq, tx)) = stream.next_entry().await.expect("next_entry") {
        entries.push((k, v, seq, tx));
    }

    // Without active readers, raw_seq = 90 < min_snapshot_seq_bound (u64::MAX), full compaction = true,
    // so Tombstone(90) and older Put(50) are safely garbage collected!
    assert_eq!(
        entries.len(),
        0,
        "Tombstone(90) and Put(50) must be garbage collected when no active reader exists"
    );
}
