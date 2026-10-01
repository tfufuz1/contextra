#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_core::{
    ContextraError, ResourceBudget, ResourceTracker, Result, SnapshotRegistry, TOMBSTONE_BIT,
};
use contextra_store::compaction::{CompactionConfig, CompactionEngine, MergeOperator};
use contextra_store::sstable::{BlockCache, SstableBuilder, SstableReader};
use contextra_testkit::ManualClock;
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

struct CounterMergeOperator;

impl MergeOperator for CounterMergeOperator {
    fn merge(&self, existing_val: &[u8], new_val: &[u8]) -> Result<Vec<u8>> {
        let v1 = if existing_val.len() == 8 {
            u64::from_le_bytes(existing_val.try_into().unwrap())
        } else {
            0
        };
        let v2 = if new_val.len() == 8 {
            u64::from_le_bytes(new_val.try_into().unwrap())
        } else {
            0
        };
        Ok((v1 + v2).to_le_bytes().to_vec())
    }
}

struct AlwaysErrorMergeOperator;

impl MergeOperator for AlwaysErrorMergeOperator {
    fn merge(&self, _existing_val: &[u8], _new_val: &[u8]) -> Result<Vec<u8>> {
        Err(ContextraError::Storage("Simulated merge failure".into()))
    }
}

fn create_tracker(mem_mb: usize) -> Arc<ResourceTracker> {
    let budget = ResourceBudget {
        memory_limit: (mem_mb * 1024 * 1024) as u64,
    };
    Arc::new(ResourceTracker::new(budget))
}

#[tokio::test]
async fn test_merge_operator_counter_across_sstables() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());
    let budget = create_tracker(100);

    let sst1_path = dir.path().join("sst1.sst");
    let mut builder1 = SstableBuilder::create(&sst1_path).await.unwrap();
    builder1
        .add(b"counter", &10u64.to_le_bytes(), 1, 1)
        .await
        .unwrap();
    builder1.finish().await.unwrap();

    let sst2_path = dir.path().join("sst2.sst");
    let mut builder2 = SstableBuilder::create(&sst2_path).await.unwrap();
    builder2
        .add(b"counter", &20u64.to_le_bytes(), 2, 2)
        .await
        .unwrap();
    builder2.finish().await.unwrap();

    let reader1 = Arc::new(
        SstableReader::open(&sst1_path, cache.clone())
            .await
            .unwrap(),
    );
    let reader2 = Arc::new(
        SstableReader::open(&sst2_path, cache.clone())
            .await
            .unwrap(),
    );

    let config = CompactionConfig::default();
    let engine = CompactionEngine::new(
        config,
        snapshot_reg.clone(),
        cache.clone(),
        None,
        budget,
        None,
    )
    .with_merge_operator(Arc::new(CounterMergeOperator));

    let out_path = dir.path().join("compacted.sst");
    let min_seq = snapshot_reg.min_active_seqno();

    engine
        .merge_sstables(&[reader1, reader2], &out_path, min_seq, true)
        .await
        .unwrap();

    let compacted_reader = SstableReader::open(&out_path, cache).await.unwrap();
    let (val, _seq, _tx) = compacted_reader.get(b"counter").await.unwrap().unwrap();
    let count = u64::from_le_bytes(val.as_ref().try_into().unwrap());
    assert_eq!(count, 30);
}

#[tokio::test]
async fn test_merge_operator_fail_safe_retains_both() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());
    let budget = create_tracker(100);

    let sst1_path = dir.path().join("sst1.sst");
    let mut builder1 = SstableBuilder::create(&sst1_path).await.unwrap();
    builder1.add(b"key1", b"val1", 1, 1).await.unwrap();
    builder1.finish().await.unwrap();

    let sst2_path = dir.path().join("sst2.sst");
    let mut builder2 = SstableBuilder::create(&sst2_path).await.unwrap();
    builder2.add(b"key1", b"val2", 2, 2).await.unwrap();
    builder2.finish().await.unwrap();

    let reader1 = Arc::new(
        SstableReader::open(&sst1_path, cache.clone())
            .await
            .unwrap(),
    );
    let reader2 = Arc::new(
        SstableReader::open(&sst2_path, cache.clone())
            .await
            .unwrap(),
    );

    let config = CompactionConfig::default();
    let engine = CompactionEngine::new(
        config,
        snapshot_reg.clone(),
        cache.clone(),
        None,
        budget,
        None,
    )
    .with_merge_operator(Arc::new(AlwaysErrorMergeOperator));

    let out_path = dir.path().join("compacted.sst");
    let min_seq = snapshot_reg.min_active_seqno();

    engine
        .merge_sstables(&[reader1, reader2], &out_path, min_seq, false)
        .await
        .unwrap();

    let compacted_reader = SstableReader::open(&out_path, cache).await.unwrap();
    // Both versions should remain readable in compacted SSTable (v2 newer)
    let (val, _seq, _tx) = compacted_reader
        .get_at(b"key1", 2, 2)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(val.as_ref(), b"val2");

    let (val_old, _seq, _tx) = compacted_reader
        .get_at(b"key1", 1, 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(val_old.as_ref(), b"val1");
}

#[tokio::test]
async fn test_merge_operator_deterministic_output() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());
    let budget = create_tracker(100);

    let sst1_path = dir.path().join("sst1.sst");
    let mut builder1 = SstableBuilder::create(&sst1_path).await.unwrap();
    builder1
        .add(b"key1", &5u64.to_le_bytes(), 1, 1)
        .await
        .unwrap();
    builder1.finish().await.unwrap();

    let sst2_path = dir.path().join("sst2.sst");
    let mut builder2 = SstableBuilder::create(&sst2_path).await.unwrap();
    builder2
        .add(b"key1", &15u64.to_le_bytes(), 2, 2)
        .await
        .unwrap();
    builder2.finish().await.unwrap();

    let reader1 = Arc::new(
        SstableReader::open(&sst1_path, cache.clone())
            .await
            .unwrap(),
    );
    let reader2 = Arc::new(
        SstableReader::open(&sst2_path, cache.clone())
            .await
            .unwrap(),
    );

    let config = CompactionConfig::default();
    let engine = CompactionEngine::new(
        config,
        snapshot_reg.clone(),
        cache.clone(),
        None,
        budget,
        None,
    )
    .with_merge_operator(Arc::new(CounterMergeOperator));

    let out_path1 = dir.path().join("compacted1.sst");
    let out_path2 = dir.path().join("compacted2.sst");
    let min_seq = snapshot_reg.min_active_seqno();

    engine
        .merge_sstables(
            &[reader1.clone(), reader2.clone()],
            &out_path1,
            min_seq,
            true,
        )
        .await
        .unwrap();

    engine
        .merge_sstables(&[reader1, reader2], &out_path2, min_seq, true)
        .await
        .unwrap();

    let content1 = std::fs::read(&out_path1).unwrap();
    let content2 = std::fs::read(&out_path2).unwrap();
    assert_eq!(content1, content2);
}

#[tokio::test]
async fn test_active_snapshot_prevents_merge() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());
    let budget = create_tracker(100);

    // Pin a snapshot at seq 2
    snapshot_reg.pin(2);

    let sst1_path = dir.path().join("sst1.sst");
    let mut builder1 = SstableBuilder::create(&sst1_path).await.unwrap();
    builder1
        .add(b"counter", &10u64.to_le_bytes(), 1, 1)
        .await
        .unwrap();
    builder1.finish().await.unwrap();

    let sst2_path = dir.path().join("sst2.sst");
    let mut builder2 = SstableBuilder::create(&sst2_path).await.unwrap();
    builder2
        .add(b"counter", &20u64.to_le_bytes(), 3, 3)
        .await
        .unwrap();
    builder2.finish().await.unwrap();

    let reader1 = Arc::new(
        SstableReader::open(&sst1_path, cache.clone())
            .await
            .unwrap(),
    );
    let reader2 = Arc::new(
        SstableReader::open(&sst2_path, cache.clone())
            .await
            .unwrap(),
    );

    let config = CompactionConfig::default();
    let engine = CompactionEngine::new(
        config,
        snapshot_reg.clone(),
        cache.clone(),
        None,
        budget,
        None,
    )
    .with_merge_operator(Arc::new(CounterMergeOperator));

    let out_path = dir.path().join("compacted.sst");
    let min_seq = snapshot_reg.min_active_seqno(); // min_seq = 2

    // raw_seq of version 3 is >= min_seq (2), so it cannot be merged with version 1
    engine
        .merge_sstables(&[reader1, reader2], &out_path, min_seq, false)
        .await
        .unwrap();

    let compacted_reader = SstableReader::open(&out_path, cache).await.unwrap();
    let (val_seq3, _seq, _tx) = compacted_reader
        .get_at(b"counter", 3, 3)
        .await
        .unwrap()
        .unwrap();
    let (val_seq1, _seq, _tx) = compacted_reader
        .get_at(b"counter", 1, 1)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        u64::from_le_bytes(val_seq3.as_ref().try_into().unwrap()),
        20
    );
    assert_eq!(
        u64::from_le_bytes(val_seq1.as_ref().try_into().unwrap()),
        10
    );
}

#[tokio::test]
async fn test_tombstone_keys_are_not_merged() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());
    let budget = create_tracker(100);

    let sst1_path = dir.path().join("sst1.sst");
    let mut builder1 = SstableBuilder::create(&sst1_path).await.unwrap();
    builder1
        .add(b"key1", &10u64.to_le_bytes(), 1, 1)
        .await
        .unwrap();
    builder1.finish().await.unwrap();

    let sst2_path = dir.path().join("sst2.sst");
    let mut builder2 = SstableBuilder::create(&sst2_path).await.unwrap();
    // Tombstone at seq 2
    builder2
        .add(b"key1", b"", 2 | TOMBSTONE_BIT, 2)
        .await
        .unwrap();
    builder2.finish().await.unwrap();

    let reader1 = Arc::new(
        SstableReader::open(&sst1_path, cache.clone())
            .await
            .unwrap(),
    );
    let reader2 = Arc::new(
        SstableReader::open(&sst2_path, cache.clone())
            .await
            .unwrap(),
    );

    let config = CompactionConfig::default();
    let engine = CompactionEngine::new(
        config,
        snapshot_reg.clone(),
        cache.clone(),
        None,
        budget,
        None,
    )
    .with_merge_operator(Arc::new(CounterMergeOperator));

    let out_path = dir.path().join("compacted.sst");
    let min_seq = snapshot_reg.min_active_seqno();

    engine
        .merge_sstables(&[reader1, reader2], &out_path, min_seq, false)
        .await
        .unwrap();

    let compacted_reader = SstableReader::open(&out_path, cache).await.unwrap();
    // Tombstone should remain (with TOMBSTONE_BIT set), value shouldn't be merged into a Put
    let (_val, seq, _tx) = compacted_reader.get(b"key1").await.unwrap().unwrap();
    assert_ne!(seq & TOMBSTONE_BIT, 0);
}

struct FuelExhaustingMergeOperator;

impl MergeOperator for FuelExhaustingMergeOperator {
    fn merge(&self, _existing_val: &[u8], _new_val: &[u8]) -> Result<Vec<u8>> {
        Err(ContextraError::Sandbox("fuel exhausted".into()))
    }
}

struct SlowClockAdvancingMergeOperator {
    clock: Arc<ManualClock>,
    delay: Duration,
}

impl MergeOperator for SlowClockAdvancingMergeOperator {
    fn merge(&self, _existing_val: &[u8], _new_val: &[u8]) -> Result<Vec<u8>> {
        self.clock.advance(self.delay);
        Ok(b"merged_val".to_vec())
    }
}

#[tokio::test]
async fn test_merge_operator_fuel_exhausted_fail_safe() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());
    let budget = create_tracker(100);

    let sst1_path = dir.path().join("sst1.sst");
    let mut builder1 = SstableBuilder::create(&sst1_path).await.unwrap();
    builder1.add(b"key_fuel", b"val_old", 1, 1).await.unwrap();
    builder1.finish().await.unwrap();

    let sst2_path = dir.path().join("sst2.sst");
    let mut builder2 = SstableBuilder::create(&sst2_path).await.unwrap();
    builder2.add(b"key_fuel", b"val_new", 2, 2).await.unwrap();
    builder2.finish().await.unwrap();

    let reader1 = Arc::new(
        SstableReader::open(&sst1_path, cache.clone())
            .await
            .unwrap(),
    );
    let reader2 = Arc::new(
        SstableReader::open(&sst2_path, cache.clone())
            .await
            .unwrap(),
    );

    let config = CompactionConfig::default();
    assert_eq!(config.merge_max_fuel, 10_000_000);
    assert_eq!(config.merge_wall_clock_timeout, Duration::from_millis(5_000));

    let engine = CompactionEngine::new(
        config,
        snapshot_reg.clone(),
        cache.clone(),
        None,
        budget,
        None,
    )
    .with_merge_operator(Arc::new(FuelExhaustingMergeOperator));

    let out_path = dir.path().join("compacted.sst");
    let min_seq = snapshot_reg.min_active_seqno();

    engine
        .merge_sstables(&[reader1, reader2], &out_path, min_seq, false)
        .await
        .unwrap();

    let compacted_reader = SstableReader::open(&out_path, cache).await.unwrap();
    // Verify both original versions remain preserved without loss or panic
    let (val_new, _seq, _tx) = compacted_reader
        .get_at(b"key_fuel", 2, 2)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(val_new.as_ref(), b"val_new");

    let (val_old, _seq, _tx) = compacted_reader
        .get_at(b"key_fuel", 1, 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(val_old.as_ref(), b"val_old");
}

#[tokio::test]
async fn test_merge_operator_wall_clock_timeout_fail_safe() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());
    let budget = create_tracker(100);
    let manual_clock = Arc::new(ManualClock::new(1_000_000_000));

    let sst1_path = dir.path().join("sst1.sst");
    let mut builder1 = SstableBuilder::create(&sst1_path).await.unwrap();
    builder1.add(b"key_timeout", b"old_data", 1, 1).await.unwrap();
    builder1.finish().await.unwrap();

    let sst2_path = dir.path().join("sst2.sst");
    let mut builder2 = SstableBuilder::create(&sst2_path).await.unwrap();
    builder2.add(b"key_timeout", b"new_data", 2, 2).await.unwrap();
    builder2.finish().await.unwrap();

    let reader1 = Arc::new(
        SstableReader::open(&sst1_path, cache.clone())
            .await
            .unwrap(),
    );
    let reader2 = Arc::new(
        SstableReader::open(&sst2_path, cache.clone())
            .await
            .unwrap(),
    );

    let config = CompactionConfig {
        merge_wall_clock_timeout: Duration::from_millis(100),
        ..CompactionConfig::default()
    };

    let slow_operator = Arc::new(SlowClockAdvancingMergeOperator {
        clock: manual_clock.clone(),
        delay: Duration::from_millis(200), // exceeds 100ms timeout
    });

    let engine = CompactionEngine::new(
        config,
        snapshot_reg.clone(),
        cache.clone(),
        None,
        budget,
        None,
    )
    .with_clock(manual_clock)
    .with_merge_operator(slow_operator);

    let out_path = dir.path().join("compacted.sst");
    let min_seq = snapshot_reg.min_active_seqno();

    engine
        .merge_sstables(&[reader1, reader2], &out_path, min_seq, false)
        .await
        .unwrap();

    let compacted_reader = SstableReader::open(&out_path, cache).await.unwrap();
    // Because timeout was exceeded, fail-safe retains both versions
    let (val_new, _seq, _tx) = compacted_reader
        .get_at(b"key_timeout", 2, 2)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(val_new.as_ref(), b"new_data");

    let (val_old, _seq, _tx) = compacted_reader
        .get_at(b"key_timeout", 1, 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(val_old.as_ref(), b"old_data");
}

#[tokio::test]
async fn test_merge_operator_pure_capabilities_enforcement() {
    use contextra_sandbox::MergeOperatorCapabilities;

    let pure_caps = MergeOperatorCapabilities::pure();
    assert!(!pure_caps.allow_stdout, "stdout must be disabled in pure MergeOperatorCapabilities");
    assert!(!pure_caps.allow_stderr, "stderr must be disabled in pure MergeOperatorCapabilities");
    assert!(!pure_caps.allow_filesystem, "filesystem must be disabled in pure MergeOperatorCapabilities");
    assert!(!pure_caps.allow_network, "network must be disabled in pure MergeOperatorCapabilities");
    assert!(!pure_caps.allow_clock, "clock time access must be disabled in pure MergeOperatorCapabilities");
    assert!(pure_caps.random_seed.is_none(), "PRNG seed must be None in pure MergeOperatorCapabilities");
    assert_eq!(pure_caps.max_fuel, 10_000_000);
    assert_eq!(pure_caps.max_wall_clock_ms, 5_000);

    let config = CompactionConfig::default();
    assert_eq!(config.merge_max_fuel, pure_caps.max_fuel);
    assert_eq!(config.merge_wall_clock_timeout, Duration::from_millis(pure_caps.max_wall_clock_ms));
}
