// ZWECK: Block-Bloom FPR Test (S-04).
// INVARIANTE: Block-Bloom FPR muss bei nicht vorhandenen deterministischen Keys unter 2% liegen.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_store::sstable::{BlockCache, SstableBuilder, SstableReader};
use std::sync::Arc;
use tempfile::TempDir;

fn create_block_cache() -> Arc<BlockCache> {
    Arc::new(BlockCache::new(10))
}

#[tokio::test]
async fn test_block_bloom_fpr_under_2_percent_on_10k_negative_keys() {
    let tmp = TempDir::new().expect("temp dir");
    let sst_path = tmp.path().join("block_bloom_fpr.sst");
    let bc = create_block_cache();

    let mut builder = SstableBuilder::create(&sst_path)
        .await
        .expect("create builder");

    // Populate SSTable with 10,000 entries across multiple blocks (~100 entries/block)
    for i in 0..10_000 {
        let key = format!("contained_key_{:05}", i);
        let val = format!("value_payload_data_{:05}_extra_bytes_for_block_capacity", i);
        builder
            .add(key.as_bytes(), val.as_bytes(), i as u64 + 1, 1)
            .await
            .expect("add key");
    }
    builder.finish().await.expect("finish builder");

    let reader = SstableReader::open(&sst_path, bc)
        .await
        .expect("open reader");

    let mut total_in_range_checks = 0;
    let mut block_bloom_false_positives = 0;

    // Test 10,000 non-existent deterministic keys
    for i in 0..10_000 {
        let key = format!("contained_key_{:05}.neg{:04}", i, i);
        let (_whole_bloom, in_range, block_bloom_passed, found) =
            reader.lookup_metrics(key.as_bytes()).await;

        assert!(
            !found,
            "Deterministic negative key must not be found in SSTable"
        );

        if in_range {
            total_in_range_checks += 1;
            if block_bloom_passed {
                block_bloom_false_positives += 1;
            }
        }
    }

    assert!(
        total_in_range_checks > 100,
        "At least 100 negative queries must pass whole-SSTable bloom and be checked against block bloom (got {})",
        total_in_range_checks
    );

    let fpr = block_bloom_false_positives as f64 / total_in_range_checks as f64;
    println!(
        "Block Bloom FPR measurement: {} false positives out of {} block bloom checks ({:.4} = {:.2}%)",
        block_bloom_false_positives,
        total_in_range_checks,
        fpr,
        fpr * 100.0
    );

    assert!(
        fpr < 0.02,
        "Block Bloom false positive rate ({:.2}%) must be < 2.0% (fp_count: {}, total_in_range: {})",
        fpr * 100.0,
        block_bloom_false_positives,
        total_in_range_checks
    );
}
