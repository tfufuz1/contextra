use contextra_engine::health::{EngineState, HealthReport};
use contextra_engine::EngineBuilder;
use contextra_ports::{clock::SystemClock, rng::SeededRng};
use contextra_testkit::FaultVfs;
use std::sync::Arc;
use std::time::Duration;

/// SPEC: T-OPS-1
#[tokio::test]
async fn test_health_report_fields_without_blocking_io() {
    let report = HealthReport::default();
    assert_eq!(report.state, EngineState::Ok);
    assert!(report.fsync_p50 <= Duration::from_millis(10));
    assert_eq!(report.open_erasures, 0);
    assert_eq!(report.sla_violations, 0);
    assert!(report.reasons.is_empty());
}

/// SPEC: T-OPS-2
#[tokio::test]
async fn test_disk_reserve_and_health_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    let handle = tokio::runtime::Handle::current();
    let vfs = Arc::new(FaultVfs::new());
    let clock = Arc::new(SystemClock::new());
    let rng = Arc::new(SeededRng::new(42));

    let db = EngineBuilder::new()
        .data_dir("/tmp/db")
        .passphrase(b"secret_passphrase_123")
        .erase_sla(Duration::from_secs(3600))
        .backup_retention(Duration::from_secs(30 * 86400))
        .runtime(handle.clone())
        .vfs(vfs.clone())
        .clock(clock.clone())
        .rng(rng.clone())
        .open()
        .await?;

    let health = db.health();
    assert_eq!(health.state, EngineState::Ok);
    assert!(health.free_bytes >= 1024 * 1024);

    db.close().await?;
    let after_close = db.health();
    assert_eq!(after_close.state, EngineState::Degraded);
    Ok(())
}
