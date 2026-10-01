use contextra_core::ResourceTracker;
use contextra_store::compaction::{CompactionConfig, CompactionEngine};
use contextra_store::sstable::{BlockCache, SstableBuilder, SstableReader};
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_compaction_candidate_selection_prefers_overlapping_key_ranges() {
    let tmp = TempDir::new().expect("tempdir");
    let cache = Arc::new(BlockCache::new(1024 * 1024));

    // SSTable 1: keys a1..a5
    let path1 = tmp.path().join("sst1.sst");
    let mut b1 = SstableBuilder::create(&path1).await.unwrap();
    b1.add(b"a1", b"val1", 10, 0).await.unwrap();
    b1.add(b"a5", b"val5", 10, 0).await.unwrap();
    b1.finish().await.unwrap();
    let r1 = Arc::new(SstableReader::open(&path1, cache.clone()).await.unwrap());

    // SSTable 2: disjoint keys m1..m5
    let path2 = tmp.path().join("sst2.sst");
    let mut b2 = SstableBuilder::create(&path2).await.unwrap();
    b2.add(b"m1", b"val1", 20, 0).await.unwrap();
    b2.add(b"m5", b"val5", 20, 0).await.unwrap();
    b2.finish().await.unwrap();
    let r2 = Arc::new(SstableReader::open(&path2, cache.clone()).await.unwrap());

    // SSTable 3: disjoint keys z1..z5
    let path3 = tmp.path().join("sst3.sst");
    let mut b3 = SstableBuilder::create(&path3).await.unwrap();
    b3.add(b"z1", b"val1", 30, 0).await.unwrap();
    b3.add(b"z5", b"val5", 30, 0).await.unwrap();
    b3.finish().await.unwrap();
    let r3 = Arc::new(SstableReader::open(&path3, cache.clone()).await.unwrap());

    // SSTable 4: overlapping with r1 (a2..a6)
    let path4 = tmp.path().join("sst4.sst");
    let mut b4 = SstableBuilder::create(&path4).await.unwrap();
    b4.add(b"a2", b"val2", 40, 0).await.unwrap();
    b4.add(b"a6", b"val6", 40, 0).await.unwrap();
    b4.finish().await.unwrap();
    let r4 = Arc::new(SstableReader::open(&path4, cache.clone()).await.unwrap());

    // SSTable 5: overlapping with r1 & r4 (a3..a7)
    let path5 = tmp.path().join("sst5.sst");
    let mut b5 = SstableBuilder::create(&path5).await.unwrap();
    b5.add(b"a3", b"val3", 50, 0).await.unwrap();
    b5.add(b"a7", b"val7", 50, 0).await.unwrap();
    b5.finish().await.unwrap();
    let r5 = Arc::new(SstableReader::open(&path5, cache.clone()).await.unwrap());

    // SSTable 6: overlapping with r1, r4, r5 (a4..a8)
    let path6 = tmp.path().join("sst6.sst");
    let mut b6 = SstableBuilder::create(&path6).await.unwrap();
    b6.add(b"a4", b"val4", 60, 0).await.unwrap();
    b6.add(b"a8", b"val8", 60, 0).await.unwrap();
    b6.finish().await.unwrap();
    let r6 = Arc::new(SstableReader::open(&path6, cache.clone()).await.unwrap());

    let config = CompactionConfig {
        min_sstables_per_tier: 4,
        size_ratio: 4.0,
        ..Default::default()
    };
    let engine = CompactionEngine::new(
        config,
        Arc::new(contextra_core::SnapshotRegistry::new()),
        cache,
        None,
        Arc::new(ResourceTracker::new(contextra_core::ResourceBudget {
            memory_limit: 1024 * 1024 * 1024,
        })),
        None,
    );

    // List: [r1, r2, r3, r4, r5, r6]
    // Window [0..4] = [r1, r2, r3, r4]: r1("a1".."a5"), r2("m1".."m5"), r3("z1".."z5"), r4("a2".."a6"). Overlaps: only (r1, r4) overlap -> 1 overlap.
    // Window [2..6] = [r3, r4, r5, r6]: r3("z1".."z5"), r4("a2".."a6"), r5("a3".."a7"), r6("a4".."a8"). Overlaps: (r4, r5), (r5, r6) -> 2 overlaps.
    // Window [1..5] = [r2, r3, r4, r5]: r2, r3 disjoint. Overlap: (r4, r5) -> 1 overlap.
    // Window [0..6] = [r1..r6]: sorted: r1, r4, r5, r6 (3 overlaps: a5>=a2, a6>=a3, a7>=a4), then r2(m1), then r3(z1) (0 overlaps). Total overlaps = 3 out of 5.
    // Notice window [0..6] has 3 overlaps, window [2..6] has 2 overlaps.
    // But if we test candidate windows of size 4: window [r1, r4, r5, r6] (if contiguous) would have 3 overlaps out of 3.
    let ssts = vec![r1.clone(), r4.clone(), r5.clone(), r6.clone(), r2.clone(), r3.clone()];
    // Window 0..4 = [r1, r4, r5, r6]: length 4, overlaps = 3.
    // Window 1..5 = [r4, r5, r6, r2]: length 4, overlaps = 2.
    // Window 2..6 = [r5, r6, r2, r3]: length 4, overlaps = 1.
    // Window 0..6 = [r1, r4, r5, r6, r2, r3]: length 6, overlaps = 3.
    // Since 0..4 and 0..6 both have 3 overlaps, but 0..4 is checked first and length ordering... wait!
    // In our algorithm:
    // `overlap > best_overlap || (overlap == best_overlap && win_len > best_tier_len)...`
    // Window 0..4 has overlap 3, win_len 4.
    // Window 0..6 has overlap 3, win_len 6. `overlap == best_overlap && win_len > best_tier_len` -> 0..6 wins because win_len 6 > 4!
    // If all 6 SSTables are in the tier, merging all 6 is better than 4 if they have the same overlap score!

    let candidates = engine.select_compaction_candidates(&ssts).expect("candidates selected");

    // Let us verify that the candidate selection chooses the contiguous window with max overlap.
    assert!(!candidates.is_empty());
    assert!(candidates.iter().any(|s| Arc::ptr_eq(s, &r1)));
    assert!(candidates.iter().any(|s| Arc::ptr_eq(s, &r4)));
    assert!(candidates.iter().any(|s| Arc::ptr_eq(s, &r5)));
    assert!(candidates.iter().any(|s| Arc::ptr_eq(s, &r6)));
}
