//! Campaign J-10 Test: HNSW Rebuild Backoff Cooldown Invariant Verification
//! Demonstrates deterministic exponential backoff cooldown behavior under rapid rebuild requests.

use contextra_core::{DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};
use std::time::Duration;

#[tokio::test]
async fn test_campaign_hnsw_backoff_cooldown_exact_bounds() {
    let base_delay = Duration::from_millis(100);
    let config = HnswConfigBuilder::new(4)
        .rebuild_threshold(0.1)
        .backoff_base_delay(base_delay)
        .backoff_max_delay(Duration::from_secs(1))
        .backoff_alert_threshold(5)
        .build()
        .expect("valid config");

    let index = HnswIndex::try_new(config).expect("valid index");

    let tx = TxId::new(1);
    index
        .insert(tx, DocId::from(1u64), &[1.0, 0.0, 0.0, 0.0])
        .await
        .expect("insert 1");
    index
        .insert(tx, DocId::from(2u64), &[0.0, 1.0, 0.0, 0.0])
        .await
        .expect("insert 2");
    index.commit(tx).await.expect("commit 1");

    let tx2 = TxId::new(2);
    index
        .delete(tx2, DocId::from(1u64))
        .await
        .expect("delete 1");
    index.commit(tx2).await.expect("commit 2");

    // Initial rebuild succeeds
    index.rebuild().await.expect("initial rebuild succeeds");

    // Immediate second rebuild fails due to backoff
    let res_immediate = index.rebuild().await;
    assert!(
        res_immediate.is_err(),
        "immediate rebuild must fail due to backoff rate limit"
    );

    // Sleep for base_delay (100ms) + buffer
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Subsequent rebuild after full cooldown expires must succeed
    index
        .rebuild()
        .await
        .expect("rebuild after backoff cooldown expiry must succeed");
}
