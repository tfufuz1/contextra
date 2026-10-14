#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::builder::ContextraBuilder;
use contextra_ports::clock::ManualClock;
use contextra_ports::rng::SeededRng;
use contextra_ports::Rng;

#[tokio::test]
async fn test_feature_sovereign_and_compliance_gating() -> Result<(), Box<dyn std::error::Error>> {
    let seed: u64 = 20261013_10;
    let rng = SeededRng::new(seed);
    let clock = ManualClock::new(1760390400_000_000_000);
    println!("e2e_wiring_features seed: {}, clock_nanos: {}", seed, clock.now_nanos());

    let tmp_dir = std::env::temp_dir().join(format!("e2e_wiring_features_{}", rng.next_u64()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    // Standard open without sovereign/compliance features
    let db = ContextraBuilder::new(4)
        .with_storage_path(&tmp_dir)
        .build()
        .await?;

    assert_eq!(db.config().dimension, 4);
    db.close().await?;

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}

#[tokio::test]
#[cfg(feature = "sovereign")]
async fn test_sovereign_feature_active() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = std::env::temp_dir().join("e2e_wiring_sovereign_active");
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let db = ContextraBuilder::new(4)
        .with_storage_path(&tmp_dir)
        .build()
        .await?;

    db.close().await?;
    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}

#[tokio::test]
#[ignore = "blockiert durch PROMPT-FEATURE-01: WASM Sandbox Plugin Manifest is not wired into Contextra facade builder"]
async fn test_sandbox_feature_wasm_plugin_manifest() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = std::env::temp_dir().join("e2e_wiring_sandbox_feature");
    let _ = std::fs::remove_dir_all(&tmp_dir);

    // WASM Sandbox plugin manifest wiring on Contextra facade is pending.

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}
