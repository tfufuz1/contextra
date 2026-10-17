#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_core::{ResourceBudget, ResourceTracker, Result, SnapshotRegistry};
use contextra_store::compaction::{CompactionConfig, CompactionEngine, MergeOperator};
use contextra_store::sstable::{BlockCache, SstableBuilder, SstableReader};
use std::sync::Arc;
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

fn create_tracker(mem_mb: usize) -> Arc<ResourceTracker> {
    let budget = ResourceBudget {
        memory_limit: (mem_mb * 1024 * 1024) as u64,
    };
    Arc::new(ResourceTracker::new(budget))
}

/// Proves whether N-way merge in LSM compaction correctly consumes and merges all versions
/// across 3 or more overlapping SSTables when all version sequence numbers lie below `min_snapshot_seq`.
///
/// A failure in this test indicates that the compaction engine only partially consumes same-key versions
/// (e.g. pairwise merging only 2 versions per outer loop iteration) during N-way merge instead of
/// fully exhausting all same-key versions before emitting the merged result.
#[tokio::test]
async fn test_campaign_deepresearch_three_way_merge_full_consumption() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());
    let budget = create_tracker(100);

    // Create 3 SSTables, each containing the key "counter" with sequence numbers 1, 2, and 3.
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

    let sst3_path = dir.path().join("sst3.sst");
    let mut builder3 = SstableBuilder::create(&sst3_path).await.unwrap();
    builder3
        .add(b"counter", &30u64.to_le_bytes(), 3, 3)
        .await
        .unwrap();
    builder3.finish().await.unwrap();

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
    let reader3 = Arc::new(
        SstableReader::open(&sst3_path, cache.clone())
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
    // All input sequence numbers (1, 2, 3) are strictly below min_seq (which is u64::MAX or unpinned active snapshot seq)
    let min_seq = snapshot_reg.min_active_seqno();

    engine
        .merge_sstables(&[reader1, reader2, reader3], &out_path, min_seq, true)
        .await
        .unwrap();

    let compacted_reader = Arc::new(SstableReader::open(&out_path, cache).await.unwrap());

    // Stream all entries from the compacted SSTable to verify exact entry count and value
    let mut stream = compacted_reader.stream().await.unwrap();
    let mut entries = Vec::new();
    while let Some(entry) = stream.next_entry().await.unwrap() {
        entries.push(entry);
    }

    // Output must contain EXACTLY ONE entry for "counter"
    assert_eq!(
        entries.len(),
        1,
        "Expected exactly 1 entry in compacted SSTable, but found {}: {:?}",
        entries.len(),
        entries
    );

    let (key, val, _seq, _tx) = &entries[0];
    assert_eq!(key.as_ref(), b"counter");

    let count = u64::from_le_bytes(val.as_ref().try_into().unwrap());
    // Expected sum: 10 + 20 + 30 = 60
    assert_eq!(
        count, 60,
        "Expected fully merged sum 60 (10 + 20 + 30), but observed {}",
        count
    );
}
