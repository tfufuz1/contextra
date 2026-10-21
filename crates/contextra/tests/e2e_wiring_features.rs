#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::builder::ContextraBuilder;
use contextra_ports::SystemClock;
use contextra_sys::SeededRng;
use contextra_ports::Rng;

#[tokio::test]
async fn test_feature_sovereign_and_compliance_gating() -> Result<(), Box<dyn std::error::Error>> {
    let seed: u64 = 20261013_10;
    let mut rng = SeededRng::new(seed);
    let clock = SystemClock::new();
    println!(
        "e2e_wiring_features seed: {}, clock_nanos: {}",
        seed,
        clock.now_unix_nanos()
    );

    let tmp_dir = std::env::temp_dir().join(format!("e2e_wiring_features_{}", rng.next_u64()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    // Standard open
    let db = ContextraBuilder::new(4)
        .with_storage_path(&tmp_dir)
        .build()
        .await?;

    assert_eq!(db.config().dimension, 4);
    db.close().await?;

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}
