#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::{open_with_config, ContextraConfig};
use contextra_core::ContextraError;
use contextra_ports::clock::ManualClock;
use contextra_ports::rng::SeededRng;
use contextra_ports::Rng;

#[tokio::test]
async fn test_consolidation_builder_config_lifecycle() -> Result<(), ContextraError> {
    let seed: u64 = 20261013_06;
    let rng = SeededRng::new(seed);
    let clock = ManualClock::new(1760390400_000_000_000);
    println!(
        "e2e_wiring_consolidation seed: {}, clock_nanos: {}",
        seed,
        clock.now_nanos()
    );

    let tmp_dir = std::env::temp_dir().join(format!("e2e_wiring_consolidation_{}", rng.next_u64()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: true,
        ..Default::default()
    };

    let db = open_with_config(&tmp_dir, config).await?;
    assert!(db.config().consolidation_enabled);

    // Verify graceful worker termination upon facade close
    db.close().await?;

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}

#[tokio::test]
#[ignore = "blockiert durch PROMPT-CONSOLIDATION-01: Background consolidation worker loop and task queue are not wired to Contextra facade builder"]
async fn test_consolidation_background_worker_and_task_lifecycle() -> Result<(), ContextraError> {
    let seed: u64 = 20261013_07;
    let rng = SeededRng::new(seed);
    let clock = ManualClock::new(1760390400_000_000_000);
    println!(
        "e2e_wiring_consolidation worker seed: {}, clock_nanos: {}",
        seed,
        clock.now_nanos()
    );

    let tmp_dir = std::env::temp_dir().join(format!(
        "e2e_wiring_consolidation_worker_{}",
        rng.next_u64()
    ));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    // Background consolidation loop spawning & task queue tracking require Contextra
    // background worker orchestration wiring in ContextraBuilder::build().

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}
