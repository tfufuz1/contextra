use std::sync::Arc;
use std::time::Duration;
use contextra_engine::EngineBuilder;
use contextra_ports::{clock::SystemClock, rng::SeededRng};
use contextra_testkit::FaultVfs;

/// SPEC: T-CFG-3
#[tokio::test]
async fn test_builder_missing_mandatory_parameters() {
    let handle = tokio::runtime::Handle::current();
    let vfs = Arc::new(FaultVfs::new());
    let clock = Arc::new(SystemClock::new());
    let rng = Arc::new(SeededRng::new(42));

    // 1. Fehlen von data_dir
    let builder = EngineBuilder::new()
        .passphrase(b"secret_passphrase_123")
        .erase_sla(Duration::from_secs(3600))
        .backup_retention(Duration::from_secs(30 * 86400))
        .runtime(handle.clone())
        .vfs(vfs.clone())
        .clock(clock.clone())
        .rng(rng.clone());
    assert!(builder.open().await.is_err());

    // 2. Zu kurze Passphrase (< 12 Bytes)
    let builder = EngineBuilder::new()
        .data_dir("/tmp/db")
        .passphrase(b"kurz")
        .erase_sla(Duration::from_secs(3600))
        .backup_retention(Duration::from_secs(30 * 86400))
        .runtime(handle.clone())
        .vfs(vfs.clone())
        .clock(clock.clone())
        .rng(rng.clone());
    assert!(builder.open().await.is_err());

    // 3. Fehlendes backup_retention (SPEC D-74: open ohne Angabe wird verweigert)
    let builder = EngineBuilder::new()
        .data_dir("/tmp/db")
        .passphrase(b"secret_passphrase_123")
        .erase_sla(Duration::from_secs(3600))
        .runtime(handle.clone())
        .vfs(vfs.clone())
        .clock(clock.clone())
        .rng(rng.clone());
    assert!(builder.open().await.is_err());

    // 4. batch_window > erase_sla / 2 (SPEC D-39)
    let builder = EngineBuilder::new()
        .data_dir("/tmp/db")
        .passphrase(b"secret_passphrase_123")
        .erase_sla(Duration::from_secs(100))
        .batch_window(Duration::from_secs(60)) // 60 > 50
        .backup_retention(Duration::from_secs(30 * 86400))
        .runtime(handle.clone())
        .vfs(vfs.clone())
        .clock(clock.clone())
        .rng(rng.clone());
    assert!(builder.open().await.is_err());
}

/// SPEC: T-LIMIT-1
#[tokio::test]
async fn test_builder_and_collection_limits() -> Result<(), Box<dyn std::error::Error>> {
    let handle = tokio::runtime::Handle::current();
    let vfs = Arc::new(FaultVfs::new());
    let clock = Arc::new(SystemClock::new());
    let rng = Arc::new(SeededRng::new(42));

    // Gültiger Start
    let db = EngineBuilder::new()
        .data_dir("/tmp/db")
        .passphrase(b"secret_passphrase_123")
        .erase_sla(Duration::from_secs(3600))
        .batch_window(Duration::from_secs(60))
        .backup_retention(Duration::from_secs(30 * 86400))
        .runtime(handle.clone())
        .vfs(vfs.clone())
        .clock(clock.clone())
        .rng(rng.clone())
        .open()
        .await?;

    let tenant_id = contextra_types::TenantId::try_new(1)?;
    let col = db
        .collection_for_tenant("test_col", tenant_id, Default::default())
        .await?;

    // Limit-Test: k > 1000 bei search_text
    let res = col.search_text("term", 1001).await;
    assert!(res.is_err());

    // Limit-Test: k > 1000 bei search_vector
    let res_vec = col.search_vector(vec![0.1; 768], 1001).await;
    assert!(res_vec.is_err());
    Ok(())
}
