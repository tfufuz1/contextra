//! Integration test for TTL expiration during LSM SSTable compaction (§4.12, P28).

use contextra_core::{ResourceBudget, ResourceTracker, SnapshotRegistry};
use contextra_ports::Clock;
use contextra_store::{
    sstable::{BlockCache, SstableBuilder, SstableReader},
    CompactionConfig, CompactionEngine, TtlMetadata,
};
use contextra_testkit::ManualClock;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_ttl_compaction_expiration_with_manual_clock() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let path = temp_dir.path();

    // 1. Initialize ManualClock at baseline T = 2,000,000,000 nanos (2 seconds)
    let manual_clock = Arc::new(ManualClock::new(2_000_000_000));

    // 2. Prepare 3 entries across 2 SSTables:
    // - "key1": Expired TTL (expires_at = 1_000_000_000 nanos < clock 2_000_000_000 nanos)
    // - "key2": Future TTL (expires_at = 5_000_000_000 nanos > clock 2_000_000_000 nanos)
    // - "key3": Plain value without TTL
    let expired_meta = TtlMetadata::new(1_000_000_000);
    let future_meta = TtlMetadata::new(5_000_000_000);

    let expired_val = expired_meta.encode_value(b"expired_value_data");
    let future_val = future_meta.encode_value(b"future_value_data");
    let plain_val = b"plain_value_data".to_vec();

    // SSTable 1: Contains key1 (expired) and key2 (future)
    let sst1_path = path.join("sst1.sst");
    let mut builder1 = SstableBuilder::create_with_key_manager(&sst1_path, None).await?;
    builder1.add(b"key1", &expired_val, 10, 1).await?;
    builder1.add(b"key2", &future_val, 11, 2).await?;
    builder1.finish().await?;

    // SSTable 2: Contains key3 (plain)
    let sst2_path = path.join("sst2.sst");
    let mut builder2 = SstableBuilder::create_with_key_manager(&sst2_path, None).await?;
    builder2.add(b"key3", &plain_val, 12, 3).await?;
    builder2.finish().await?;

    let block_cache = Arc::new(BlockCache::new(16 * 1024 * 1024));
    let sst1_reader = Arc::new(SstableReader::open_with_key_manager(&sst1_path, Arc::clone(&block_cache), None).await?);
    let sst2_reader = Arc::new(SstableReader::open_with_key_manager(&sst2_path, Arc::clone(&block_cache), None).await?);

    let sstables = Arc::new(RwLock::new(vec![sst1_reader, sst2_reader]));

    // 3. Configure CompactionEngine with TTL enabled via ManualClock
    let config = CompactionConfig {
        min_sstables_per_tier: 2,
        ..Default::default()
    };
    let snapshot_registry = Arc::new(SnapshotRegistry::new());
    let budget = Arc::new(ResourceTracker::new(ResourceBudget {
        memory_limit: 1024 * 1024,
    }));

    let engine = CompactionEngine::new_with_ttl(
        config,
        snapshot_registry,
        block_cache,
        None,
        budget,
        None,
        manual_clock.clone() as Arc<dyn Clock>,
    );

    // 4. Trigger compaction
    let compacted = engine.maybe_compact(&sstables, path).await?;
    assert!(compacted, "Compaction must run for 2 input SSTables");

    // 5. Inspect resulting compacted SSTable list
    let guard = sstables.read().await;
    assert_eq!(guard.len(), 1, "Compaction must produce exactly 1 output SSTable");

    let output_reader = &guard[0];
    let mut stream = output_reader.stream().await?;

    let mut found_keys = Vec::new();
    while let Some((k, v, _seq, _tx)) = stream.next_entry().await? {
        found_keys.push((String::from_utf8_lossy(&k).to_string(), v.to_vec()));
    }

    // Verify key1 (expired) was discarded, key2 (future) and key3 (plain) were retained
    let keys_only: Vec<String> = found_keys.iter().map(|(k, _)| k.clone()).collect();
    assert_eq!(
        keys_only,
        vec!["key2".to_string(), "key3".to_string()],
        "Expired key1 must be discarded during compaction; future key2 and plain key3 must remain"
    );

    // Verify value payloads
    let key2_val = &found_keys[0].1;
    let key2_parsed = TtlMetadata::parse_from_value(key2_val);
    assert_eq!(key2_parsed, Some(future_meta));
    assert_eq!(TtlMetadata::unwrap_value(key2_val), b"future_value_data");

    let key3_val = &found_keys[1].1;
    assert_eq!(key3_val, b"plain_value_data");

    Ok(())
}

#[tokio::test]
async fn test_ttl_compaction_advancing_manual_clock_expires_remaining_entry() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let path = temp_dir.path();

    let manual_clock = Arc::new(ManualClock::new(2_000_000_000));
    let meta = TtlMetadata::new(3_000_000_000); // Expires at T = 3s
    let val_bytes = meta.encode_value(b"temporary_doc");

    // Round 1: Compaction at T = 2s (entry not yet expired)
    let sst1_path = path.join("round1_a.sst");
    let mut b1 = SstableBuilder::create_with_key_manager(&sst1_path, None).await?;
    b1.add(b"temp_key", &val_bytes, 10, 1).await?;
    b1.finish().await?;

    let sst2_path = path.join("round1_b.sst");
    let mut b2 = SstableBuilder::create_with_key_manager(&sst2_path, None).await?;
    b2.add(b"perm_key", b"permanent_doc", 11, 2).await?;
    b2.finish().await?;

    let block_cache = Arc::new(BlockCache::new(16 * 1024 * 1024));
    let r1 = Arc::new(SstableReader::open_with_key_manager(&sst1_path, Arc::clone(&block_cache), None).await?);
    let r2 = Arc::new(SstableReader::open_with_key_manager(&sst2_path, Arc::clone(&block_cache), None).await?);

    let sstables = Arc::new(RwLock::new(vec![r1, r2]));

    let engine = CompactionEngine::new_with_ttl(
        CompactionConfig { min_sstables_per_tier: 2, ..Default::default() },
        Arc::new(SnapshotRegistry::new()),
        Arc::clone(&block_cache),
        None,
        Arc::new(ResourceTracker::new(ResourceBudget {
            memory_limit: 1024 * 1024,
        })),
        None,
        manual_clock.clone() as Arc<dyn Clock>,
    );

    let res1 = engine.maybe_compact(&sstables, path).await?;
    assert!(res1);

    {
        let guard = sstables.read().await;
        let mut stream = guard[0].stream().await?;
        let mut keys = Vec::new();
        while let Some((k, _, _, _)) = stream.next_entry().await? {
            keys.push(String::from_utf8_lossy(&k).to_string());
        }
        assert_eq!(keys, vec!["perm_key".to_string(), "temp_key".to_string()]);
    }

    // Round 2: Advance ManualClock past expiry (T = 4,000,000,000 nanos) and compact again with a 2nd SSTable
    manual_clock.advance(std::time::Duration::from_secs(2)); // T is now 4s

    let sst3_path = path.join("round2_c.sst");
    let mut b3 = SstableBuilder::create_with_key_manager(&sst3_path, None).await?;
    b3.add(b"another_key", b"another_doc", 12, 3).await?;
    b3.finish().await?;

    let r3 = Arc::new(SstableReader::open_with_key_manager(&sst3_path, Arc::clone(&block_cache), None).await?);
    sstables.write().await.push(r3);

    let res2 = engine.maybe_compact(&sstables, path).await?;
    assert!(res2);

    {
        let guard = sstables.read().await;
        let mut stream = guard[0].stream().await?;
        let mut keys = Vec::new();
        while let Some((k, _, _, _)) = stream.next_entry().await? {
            keys.push(String::from_utf8_lossy(&k).to_string());
        }
        assert_eq!(
            keys,
            vec!["another_key".to_string(), "perm_key".to_string()],
            "temp_key must be expired and removed after advancing ManualClock past 3s"
        );
    }

    Ok(())
}
