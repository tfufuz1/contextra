#![cfg(not(loom))]
use contextra_engine::{
    AdaptiveDecayController, Contextra, ContextraConfig, DecayControllerConfig, DecaySignalInputs,
};
use contextra_types::MemoryType;
use serde_json::json;
use tempfile::TempDir;

#[test]
fn test_adaptive_decay_controller_temperature_and_eviction() {
    let controller = AdaptiveDecayController::new(DecayControllerConfig {
        kappa: 2.0,
        base_half_life_tx: 100,
        eviction_threshold: 0.1,
    });

    let inputs_fresh = DecaySignalInputs::default();

    let score_fresh = controller.effective_score(1.0, 0, &inputs_fresh);
    assert!(
        (score_fresh - 1.0).abs() < 1e-4,
        "Fresh chunk (elapsed_tx=0) score must be ~1.0, got: {}",
        score_fresh
    );

    let inputs_hot = DecaySignalInputs {
        tombstone_ratio: 1.0,
        query_load_inverse: 1.0,
        ..Default::default()
    };

    let score_decayed = controller.effective_score(1.0, 10_000, &inputs_hot);
    assert!(
        score_decayed < 0.1,
        "Chunk after 10,000 tx at high temperature must decay below eviction threshold 0.1, got: {}",
        score_decayed
    );

    assert!(
        controller.should_evict(1.0, 10_000, &inputs_hot),
        "should_evict must return true when effective score falls below threshold"
    );
}

#[tokio::test]
async fn test_expiry_reaper_cleans_expired_working_memory() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("decay_expiry_col").await?;

    let vec = [0.1, 0.2, 0.3, 0.4];

    // 1. Insert document with TTL = 3 transaction commits
    col.insert_with_ttl("wm_doc_1", &vec, Some(json!({ "type": "working" })), 3)
        .await?;

    // 2. Insert Permanent Semantic Memory (no TTL)
    col.insert_typed(
        "sem_doc_1",
        &[0.5, 0.6, 0.7, 0.8],
        MemoryType::Semantic,
        Some(json!({ "title": "Permanent Knowledge" })),
    )
    .await?;

    // Immediately after insert, wm_doc_1 must be retrievable
    let doc_wm_before = col.get("wm_doc_1").await?;
    assert!(
        doc_wm_before.is_some(),
        "wm_doc_1 must exist before TTL expires"
    );

    // 3. Perform 3 transaction commits to exceed TTL
    for i in 0..3 {
        col.insert(&format!("dummy_tx_{i}"), &vec, None).await?;
    }

    // 4. Trigger expiry cleanup
    let reaped_count = col.reap_expired_documents(100).await?;

    assert_eq!(
        reaped_count, 1,
        "reap_expired_documents must report 1 expired working memory document reaped"
    );

    // Verify wm_doc_1 is reaped, sem_doc_1 remains intact
    let doc_wm_after = col.get("wm_doc_1").await?;
    assert!(
        doc_wm_after.is_none(),
        "wm_doc_1 must be deleted after TTL expiration"
    );

    let doc_sem_after = col.get("sem_doc_1").await?;
    assert!(
        doc_sem_after.is_some(),
        "Semantic memory sem_doc_1 must NEVER be expired or reaped"
    );

    db.close().await?;
    Ok(())
}

struct TestClock {
    nanos: std::sync::atomic::AtomicU64,
}

impl contextra_ports::Clock for TestClock {
    fn now_unix_nanos(&self) -> u64 {
        self.nanos.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.nanos.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[tokio::test]
async fn test_trigger_expiry_cleanup_with_custom_clock() -> contextra_types::Result<()> {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    let tmp = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("custom_clock_col").await?;

    let test_clock = Arc::new(TestClock {
        nanos: AtomicU64::new(10_000_000_000), // 10,000 ms
    });
    col.set_clock(test_clock.clone());

    let vec = [0.1, 0.2, 0.3, 0.4];

    // Document created at 10,000 ms with TTL 5,000 ms (expires at 15,000 ms)
    col.insert(
        "doc_ttl_wall",
        &vec,
        Some(json!({
            "created_at_ms": 10_000,
            "ttl_ms": 5_000
        })),
    )
    .await?;

    // 1. At 10,000 ms (now_ms < expire_at 15,000 ms) -> not expired
    let cleaned = col.trigger_expiry_cleanup().await?;
    assert_eq!(cleaned, 0, "Document must not be expired at 10,000 ms");
    assert!(col.get("doc_ttl_wall").await?.is_some());

    // 2. Advance clock to 14,999 ms -> not expired
    test_clock.nanos.store(14_999_000_000, Ordering::Relaxed);
    let cleaned = col.trigger_expiry_cleanup().await?;
    assert_eq!(cleaned, 0, "Document must not be expired at 14,999 ms");
    assert!(col.get("doc_ttl_wall").await?.is_some());

    // 3. Advance clock to 15,000 ms -> expired
    test_clock.nanos.store(15_000_000_000, Ordering::Relaxed);
    let cleaned = col.trigger_expiry_cleanup().await?;
    assert_eq!(cleaned, 1, "Document must be expired at 15,000 ms");
    assert!(
        col.get("doc_ttl_wall").await?.is_none(),
        "doc_ttl_wall must be deleted after expiry cleanup"
    );

    db.close().await?;
    Ok(())
}
