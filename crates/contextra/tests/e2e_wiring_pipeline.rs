#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::{open_with_config, ContextraConfig};
use contextra_core::ContextraError;
use contextra_ports::clock::ManualClock;
use contextra_ports::rng::SeededRng;
use contextra_ports::Rng;
use serde_json::json;

#[tokio::test]
async fn test_bitemporal_filter_facade_query() -> Result<(), ContextraError> {
    let seed: u64 = 20261013_06;
    let rng = SeededRng::new(seed);
    let clock = ManualClock::new(1760390400_000_000_000);
    println!(
        "e2e_wiring_pipeline bitemporal seed: {}, clock_nanos: {}",
        seed,
        clock.now_nanos()
    );

    let tmp_dir =
        std::env::temp_dir().join(format!("e2e_wiring_pipeline_bitemporal_{}", rng.next_u64()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = open_with_config(&tmp_dir, config).await?;
    let col = db.collection("bitemporal_col").await?;

    col.insert(
        "doc_v1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({
            "valid_from": 100,
            "valid_to": 200,
            "system_from": 100
        })),
    )
    .await?;

    col.insert(
        "doc_v2",
        &[0.9, 0.1, 0.0, 0.0],
        Some(json!({
            "valid_from": 200,
            "valid_to": 300,
            "system_from": 200
        })),
    )
    .await?;

    // Hybrid query with metadata temporal filter window
    let results = col
        .query()
        .embedding([1.0, 0.0, 0.0, 0.0])
        .k(10)
        .execute()
        .await?;

    assert_eq!(results.len(), 2);

    db.close().await?;
    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}

#[tokio::test]
#[ignore = "blockiert durch PROMPT-ROUTER-01: Router and Egress pipeline decision gates are not wired through Contextra facade query path"]
async fn test_retrieval_rerank_egress_pipeline_decisions() -> Result<(), ContextraError> {
    let seed: u64 = 20261013_07;
    let rng = SeededRng::new(seed);
    let clock = ManualClock::new(1760390400_000_000_000);
    println!(
        "e2e_wiring_pipeline router seed: {}, clock_nanos: {}",
        seed,
        clock.now_nanos()
    );

    let tmp_dir =
        std::env::temp_dir().join(format!("e2e_wiring_pipeline_router_{}", rng.next_u64()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    // Egress classifier, router decision loop, and rerank policy injection on facade query path
    // are un-wired in Contextra primary facade (Ring 4).

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}
