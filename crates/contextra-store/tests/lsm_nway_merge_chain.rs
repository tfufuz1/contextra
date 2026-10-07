#![allow(clippy::unwrap_used, clippy::expect_used)]

use bytes::Bytes;
use contextra_core::{ResourceBudget, ResourceTracker, Result, SnapshotRegistry};
use contextra_store::compaction::{CompactionConfig, CompactionEngine, MergeOperator};
use contextra_store::sstable::{BlockCache, SstableBuilder, SstableReader};
use contextra_store::wal::KeyManager;
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

struct StringConcatMergeOperator;

impl MergeOperator for StringConcatMergeOperator {
    fn merge(&self, existing_val: &[u8], new_val: &[u8]) -> Result<Vec<u8>> {
        let s1 = std::str::from_utf8(existing_val).unwrap_or("");
        let s2 = std::str::from_utf8(new_val).unwrap_or("");
        Ok(format!("{s1}{s2}").into_bytes())
    }
}

struct ErrorOnSecondMergeOperator;

impl MergeOperator for ErrorOnSecondMergeOperator {
    fn merge(&self, existing_val: &[u8], new_val: &[u8]) -> Result<Vec<u8>> {
        let s1 = std::str::from_utf8(existing_val).unwrap_or("");
        let s2 = std::str::from_utf8(new_val).unwrap_or("");
        if s1.contains('b') || s2.contains('b') {
            Err(contextra_core::ContextraError::Internal(
                "Forced merge error on 'b'".into(),
            ))
        } else {
            Ok(format!("{s1}{s2}").into_bytes())
        }
    }
}

fn create_tracker() -> Arc<ResourceTracker> {
    let budget = ResourceBudget {
        memory_limit: 100 * 1024 * 1024,
    };
    Arc::new(ResourceTracker::new(budget))
}

type Entry = (Bytes, Bytes, u64, u64);

/// 6a) N = 4 and N = 5 versions, Counter-Operator, sum is correct.
#[tokio::test]
async fn test_nway_merge_n4_and_n5_counter() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());

    // N = 4 SSTables: values 10, 20, 30, 40 (seqs 1, 2, 3, 4)
    let mut readers = Vec::new();
    for (idx, val) in [10u64, 20, 30, 40].iter().enumerate() {
        let path = dir.path().join(format!("sst_n4_{idx}.sst"));
        let mut builder = SstableBuilder::create(&path).await.unwrap();
        let seq = (idx + 1) as u64;
        builder.add(b"key_n4", &val.to_le_bytes(), seq, seq).await.unwrap();
        builder.finish().await.unwrap();
        readers.push(Arc::new(SstableReader::open(&path, cache.clone()).await.unwrap()));
    }

    let engine = CompactionEngine::new(
        CompactionConfig::default(),
        snapshot_reg.clone(),
        cache.clone(),
        None,
        create_tracker(),
        None,
    )
    .with_merge_operator(Arc::new(CounterMergeOperator));

    let out_path = dir.path().join("compacted_n4.sst");
    let min_seq = snapshot_reg.min_active_seqno(); // u64::MAX

    engine
        .merge_sstables(&readers, &out_path, min_seq, true)
        .await
        .unwrap();

    let reader = Arc::new(SstableReader::open(&out_path, cache.clone()).await.unwrap());
    let mut stream = reader.stream().await.unwrap();
    let mut entries: Vec<Entry> = Vec::new();
    while let Some(e) = stream.next_entry().await.unwrap() {
        entries.push(e);
    }

    assert_eq!(entries.len(), 1);
    let (key, val, seq, tx) = &entries[0];
    assert_eq!(key.as_ref(), b"key_n4");
    assert_eq!(u64::from_le_bytes(val.as_ref().try_into().unwrap()), 100);
    // Preserves seq and tx of latest consumed version (seq 4, tx 4)
    assert_eq!(*seq, 4);
    assert_eq!(*tx, 4);

    // N = 5 SSTables: values 1, 2, 3, 4, 5 (seqs 1..=5)
    let mut readers_5 = Vec::new();
    for (idx, val) in [1u64, 2, 3, 4, 5].iter().enumerate() {
        let path = dir.path().join(format!("sst_n5_{idx}.sst"));
        let mut builder = SstableBuilder::create(&path).await.unwrap();
        let seq = (idx + 1) as u64;
        builder.add(b"key_n5", &val.to_le_bytes(), seq, seq).await.unwrap();
        builder.finish().await.unwrap();
        readers_5.push(Arc::new(SstableReader::open(&path, cache.clone()).await.unwrap()));
    }

    let out_path_5 = dir.path().join("compacted_n5.sst");
    engine
        .merge_sstables(&readers_5, &out_path_5, min_seq, true)
        .await
        .unwrap();

    let reader_5 = Arc::new(SstableReader::open(&out_path_5, cache.clone()).await.unwrap());
    let mut stream_5 = reader_5.stream().await.unwrap();
    let mut entries_5: Vec<Entry> = Vec::new();
    while let Some(e) = stream_5.next_entry().await.unwrap() {
        entries_5.push(e);
    }

    assert_eq!(entries_5.len(), 1);
    let (key_5, val_5, seq_5, tx_5) = &entries_5[0];
    assert_eq!(key_5.as_ref(), b"key_n5");
    assert_eq!(u64::from_le_bytes(val_5.as_ref().try_into().unwrap()), 15);
    assert_eq!(*seq_5, 5);
    assert_eq!(*tx_5, 5);
}

/// 6b) Non-commutative operator ("a", "b", "c"): folding order is deterministic.
/// Folding from oldest to newest: merge(existing="a", new="b") -> "ab", then merge(existing="ab", new="c") -> "abc".
#[tokio::test]
async fn test_nway_merge_non_commutative_order() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());

    // Seqs 1, 2, 3 with values "a", "b", "c"
    let mut readers = Vec::new();
    for (idx, val) in ["a", "b", "c"].iter().enumerate() {
        let path = dir.path().join(format!("sst_concat_{idx}.sst"));
        let mut builder = SstableBuilder::create(&path).await.unwrap();
        let seq = (idx + 1) as u64;
        builder.add(b"strkey", val.as_bytes(), seq, seq).await.unwrap();
        builder.finish().await.unwrap();
        readers.push(Arc::new(SstableReader::open(&path, cache.clone()).await.unwrap()));
    }

    let engine = CompactionEngine::new(
        CompactionConfig::default(),
        snapshot_reg.clone(),
        cache.clone(),
        None,
        create_tracker(),
        None,
    )
    .with_merge_operator(Arc::new(StringConcatMergeOperator));

    let out_path = dir.path().join("compacted_concat.sst");
    let min_seq = snapshot_reg.min_active_seqno();

    engine
        .merge_sstables(&readers, &out_path, min_seq, true)
        .await
        .unwrap();

    let reader = Arc::new(SstableReader::open(&out_path, cache.clone()).await.unwrap());
    let mut stream = reader.stream().await.unwrap();
    let mut entries: Vec<Entry> = Vec::new();
    while let Some(e) = stream.next_entry().await.unwrap() {
        entries.push(e);
    }

    assert_eq!(entries.len(), 1);
    let (key, val, seq, _tx) = &entries[0];
    assert_eq!(key.as_ref(), b"strkey");
    assert_eq!(std::str::from_utf8(val.as_ref()).unwrap(), "abc");
    assert_eq!(*seq, 3);
}

/// 6c) Tombstone as middle version (seq 2): seq 3 remains, seq 1 is NOT merged across tombstone.
#[tokio::test]
async fn test_nway_merge_tombstone_in_middle() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());

    // seq 1: Put "10"
    let path1 = dir.path().join("sst1.sst");
    let mut builder1 = SstableBuilder::create(&path1).await.unwrap();
    builder1.add(b"key_tomb", &10u64.to_le_bytes(), 1, 1).await.unwrap();
    builder1.finish().await.unwrap();

    // seq 2: Tombstone (seq = 2 | TOMBSTONE_BIT)
    let path2 = dir.path().join("sst2.sst");
    let mut builder2 = SstableBuilder::create(&path2).await.unwrap();
    builder2
        .add(b"key_tomb", &[], 2 | contextra_core::TOMBSTONE_BIT, 2)
        .await
        .unwrap();
    builder2.finish().await.unwrap();

    // seq 3: Put "30"
    let path3 = dir.path().join("sst3.sst");
    let mut builder3 = SstableBuilder::create(&path3).await.unwrap();
    builder3.add(b"key_tomb", &30u64.to_le_bytes(), 3, 3).await.unwrap();
    builder3.finish().await.unwrap();

    let readers = vec![
        Arc::new(SstableReader::open(&path1, cache.clone()).await.unwrap()),
        Arc::new(SstableReader::open(&path2, cache.clone()).await.unwrap()),
        Arc::new(SstableReader::open(&path3, cache.clone()).await.unwrap()),
    ];

    let engine = CompactionEngine::new(
        CompactionConfig::default(),
        snapshot_reg.clone(),
        cache.clone(),
        None,
        create_tracker(),
        None,
    )
    .with_merge_operator(Arc::new(CounterMergeOperator));

    let out_path = dir.path().join("compacted_tomb.sst");
    let min_seq = snapshot_reg.min_active_seqno();

    engine
        .merge_sstables(&readers, &out_path, min_seq, true)
        .await
        .unwrap();

    let reader = Arc::new(SstableReader::open(&out_path, cache.clone()).await.unwrap());
    let mut stream = reader.stream().await.unwrap();
    let mut entries: Vec<Entry> = Vec::new();
    while let Some(e) = stream.next_entry().await.unwrap() {
        entries.push(e);
    }

    // Full compaction with min_seq > 3:
    // seq 3 is Put "30".
    // seq 2 is Tombstone. Since seq 2 < min_seq and is_full_compaction, tombstone is GC'd.
    // seq 1 is below tombstone, so it is blocked by tombstone and not resurrectable.
    // Final result must be value 30 (NOT 30 + 10 = 40).
    assert_eq!(entries.len(), 1);
    let (key, val, seq, _tx) = &entries[0];
    assert_eq!(key.as_ref(), b"key_tomb");
    assert_eq!(u64::from_le_bytes(val.as_ref().try_into().unwrap()), 30);
    assert_eq!(*seq, 3);
}

/// 6d) Boundary case raw_seq == min_snapshot_seq: version at min_snapshot_seq must be included in merge chain.
#[tokio::test]
async fn test_nway_merge_exact_min_snapshot_seq() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());

    // Seqs 1, 2, 3 with values 10, 20, 30
    let mut readers = Vec::new();
    for (idx, val) in [10u64, 20, 30].iter().enumerate() {
        let path = dir.path().join(format!("sst_exact_{idx}.sst"));
        let mut builder = SstableBuilder::create(&path).await.unwrap();
        let seq = (idx + 1) as u64;
        builder.add(b"exactkey", &val.to_le_bytes(), seq, seq).await.unwrap();
        builder.finish().await.unwrap();
        readers.push(Arc::new(SstableReader::open(&path, cache.clone()).await.unwrap()));
    }

    let engine = CompactionEngine::new(
        CompactionConfig::default(),
        snapshot_reg.clone(),
        cache.clone(),
        None,
        create_tracker(),
        None,
    )
    .with_merge_operator(Arc::new(CounterMergeOperator));

    let out_path = dir.path().join("compacted_exact.sst");
    // Pin min_snapshot_seq EXACTLY to 3 (seq 3 == min_snapshot_seq)
    let min_seq = 3;

    engine
        .merge_sstables(&readers, &out_path, min_seq, true)
        .await
        .unwrap();

    let reader = Arc::new(SstableReader::open(&out_path, cache.clone()).await.unwrap());
    let mut stream = reader.stream().await.unwrap();
    let mut entries: Vec<Entry> = Vec::new();
    while let Some(e) = stream.next_entry().await.unwrap() {
        entries.push(e);
    }

    // Since min_seq = 3, all versions <= 3 are at or below the floor and must be merged into sum 60.
    assert_eq!(entries.len(), 1);
    let (key, val, seq, _tx) = &entries[0];
    assert_eq!(key.as_ref(), b"exactkey");
    assert_eq!(u64::from_le_bytes(val.as_ref().try_into().unwrap()), 60);
    assert_eq!(*seq, 3);
}

/// 6e) Active snapshot pinned at min_seq = 2:
/// seq 3 > min_seq stays separate.
/// seqs 1, 2 <= min_seq are folded (10 + 20 = 30).
#[tokio::test]
async fn test_nway_merge_with_active_snapshot_floor() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());

    // Seqs 1, 2, 3 with values 10, 20, 30
    let mut readers = Vec::new();
    for (idx, val) in [10u64, 20, 30].iter().enumerate() {
        let path = dir.path().join(format!("sst_snap_{idx}.sst"));
        let mut builder = SstableBuilder::create(&path).await.unwrap();
        let seq = (idx + 1) as u64;
        builder.add(b"snapkey", &val.to_le_bytes(), seq, seq).await.unwrap();
        builder.finish().await.unwrap();
        readers.push(Arc::new(SstableReader::open(&path, cache.clone()).await.unwrap()));
    }

    let engine = CompactionEngine::new(
        CompactionConfig::default(),
        snapshot_reg.clone(),
        cache.clone(),
        None,
        create_tracker(),
        None,
    )
    .with_merge_operator(Arc::new(CounterMergeOperator));

    let out_path = dir.path().join("compacted_snap.sst");
    // Pin min_seq = 2
    let min_seq = 2;

    engine
        .merge_sstables(&readers, &out_path, min_seq, true)
        .await
        .unwrap();

    let reader = Arc::new(SstableReader::open(&out_path, cache.clone()).await.unwrap());
    let mut stream = reader.stream().await.unwrap();
    let mut entries: Vec<Entry> = Vec::new();
    while let Some(e) = stream.next_entry().await.unwrap() {
        entries.push(e);
    }

    // Expected 2 output entries:
    // Entry 1: seq 3 (value 30) because 3 > min_seq (2)
    // Entry 2: seq 2 (value 30 = 10 + 20) folded floor version for seqs <= 2
    assert_eq!(entries.len(), 2);

    let (k1, v1, s1, _) = &entries[0];
    assert_eq!(k1.as_ref(), b"snapkey");
    assert_eq!(u64::from_le_bytes(v1.as_ref().try_into().unwrap()), 30);
    assert_eq!(*s1, 3);

    let (k2, v2, s2, _) = &entries[1];
    assert_eq!(k2.as_ref(), b"snapkey");
    assert_eq!(u64::from_le_bytes(v2.as_ref().try_into().unwrap()), 30);
    assert_eq!(*s2, 2);
}

/// 6f) Merge error mid-chain: no version lost, fail-safe retention.
#[tokio::test]
async fn test_nway_merge_error_mid_chain_failsafe() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());

    // Seqs 1, 2, 3 with values "a", "b", "c"
    let mut readers = Vec::new();
    for (idx, val) in ["a", "b", "c"].iter().enumerate() {
        let path = dir.path().join(format!("sst_err_{idx}.sst"));
        let mut builder = SstableBuilder::create(&path).await.unwrap();
        let seq = (idx + 1) as u64;
        builder.add(b"errkey", val.as_bytes(), seq, seq).await.unwrap();
        builder.finish().await.unwrap();
        readers.push(Arc::new(SstableReader::open(&path, cache.clone()).await.unwrap()));
    }

    let engine = CompactionEngine::new(
        CompactionConfig::default(),
        snapshot_reg.clone(),
        cache.clone(),
        None,
        create_tracker(),
        None,
    )
    .with_merge_operator(Arc::new(ErrorOnSecondMergeOperator));

    let out_path = dir.path().join("compacted_err.sst");
    let min_seq = snapshot_reg.min_active_seqno();

    engine
        .merge_sstables(&readers, &out_path, min_seq, true)
        .await
        .unwrap();

    let reader = Arc::new(SstableReader::open(&out_path, cache.clone()).await.unwrap());
    let mut stream = reader.stream().await.unwrap();
    let mut entries: Vec<Entry> = Vec::new();
    while let Some(e) = stream.next_entry().await.unwrap() {
        entries.push(e);
    }

    // ErrorOnSecondMergeOperator fails when merging with "b".
    // Under fail-safe rules, all versions must be retained without data loss.
    assert!(
        entries.len() >= 2,
        "Fail-safe retention must preserve versions on merge error, got {}",
        entries.len()
    );
}

/// 6g) key_manager.is_some(): merge operator is bypassed (encrypted values cannot be merged).
#[tokio::test]
async fn test_nway_merge_key_manager_bypasses_merge_op() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());
    let km = Arc::new(KeyManager::try_new("test_passphrase_for_compaction_test", b"salt12345678").unwrap());

    let mut readers = Vec::new();
    for (idx, val) in [10u64, 20, 30].iter().enumerate() {
        let path = dir.path().join(format!("sst_enc_{idx}.sst"));
        let mut builder =
            SstableBuilder::create_with_key_manager(&path, Some(km.clone()))
                .await
                .unwrap();
        let seq = (idx + 1) as u64;
        builder.add(b"enckey", &val.to_le_bytes(), seq, seq).await.unwrap();
        builder.finish().await.unwrap();
        readers.push(Arc::new(
            SstableReader::open_with_key_manager(&path, cache.clone(), Some(km.clone()))
                .await
                .unwrap(),
        ));
    }

    let engine = CompactionEngine::new(
        CompactionConfig::default(),
        snapshot_reg.clone(),
        cache.clone(),
        Some(km.clone()),
        create_tracker(),
        None,
    )
    .with_merge_operator(Arc::new(CounterMergeOperator));

    let out_path = dir.path().join("compacted_enc.sst");
    let min_seq = snapshot_reg.min_active_seqno();

    engine
        .merge_sstables(&readers, &out_path, min_seq, true)
        .await
        .unwrap();

    let reader = Arc::new(SstableReader::open_with_key_manager(&out_path, cache.clone(), Some(km.clone()))
        .await
        .unwrap());
    let mut stream = reader.stream().await.unwrap();
    let mut entries: Vec<Entry> = Vec::new();
    while let Some(e) = stream.next_entry().await.unwrap() {
        entries.push(e);
    }

    // With key_manager present, merge operator is bypassed.
    // The standard retention rule applies: newest floor version (seq 3, value 30) is retained.
    assert_eq!(entries.len(), 1);
    let (key, val, seq, _tx) = &entries[0];
    assert_eq!(key.as_ref(), b"enckey");
    assert_eq!(u64::from_le_bytes(val.as_ref().try_into().unwrap()), 30);
    assert_eq!(*seq, 3);
}
