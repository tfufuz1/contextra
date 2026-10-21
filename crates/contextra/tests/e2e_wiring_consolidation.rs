#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::builder::ContextraBuilder;
use contextra_ports::SystemClock;

#[tokio::test]
async fn test_feature_consolidation_wiring() -> Result<(), Box<dyn std::error::Error>> {
    let clock = SystemClock::new();
    println!(
        "e2e_wiring_consolidation clock_nanos: {}",
        clock.now_unix_nanos()
    );

    let tmp_dir = std::env::temp_dir().join(format!("e2e_wiring_consolidation_{}", clock.now_unix_nanos()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    // Standard open with consolidation disabled
    let db = ContextraBuilder::new(4)
        .with_storage_path(&tmp_dir)
        .with_consolidation(false, std::time::Duration::from_secs(300))
        .build()
        .await?;

    assert_eq!(db.config().dimension, 4);
    db.close().await?;

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}

#[tokio::test]
async fn test_feature_consolidation_enabled_wiring() -> Result<(), Box<dyn std::error::Error>> {
    let clock = SystemClock::new();
    println!(
        "e2e_wiring_consolidation_enabled clock_nanos: {}",
        clock.now_unix_nanos()
    );

    let tmp_dir = std::env::temp_dir().join(format!(
        "e2e_wiring_consolidation_enabled_{}",
        clock.now_unix_nanos()
    ));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    // Open with consolidation enabled
    let db = ContextraBuilder::new(4)
        .with_storage_path(&tmp_dir)
        .with_consolidation(true, std::time::Duration::from_secs(1))
        .build()
        .await?;

    assert_eq!(db.config().dimension, 4);
    assert!(db.config().consolidation_enabled);
    db.close().await?;

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}
