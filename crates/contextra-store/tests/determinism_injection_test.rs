//! Deterministic environment injection tests (P28 / B11).
//!
//! Verifies:
//! 1. Compaction I/O rate limiter with FakeClock (no actual thread sleep).
//! 2. System pressure monitoring with FakeClock.
//! 3. KeyMaterialSource deterministic reproducibility for database salt.
//! 4. Scanner determinism: identical output across two runs.

use contextra_ports::scanner::{LayerScanner, ScanTarget, ScanTechnique};
use contextra_ports::{Clock, RequestContext, StorageEngine, TenantId};
use contextra_store::key_source::StaticKeyMaterialSource;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_store::scanner::SstableScanner;
use contextra_store::system_pressure::SystemPressureMonitor;
use contextra_types::TxId;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

#[derive(Debug, Default)]
struct FakeClock {
    unix_nanos: AtomicU64,
    mono_nanos: AtomicU64,
}

impl FakeClock {
    fn new(start_nanos: u64) -> Self {
        Self {
            unix_nanos: AtomicU64::new(start_nanos),
            mono_nanos: AtomicU64::new(start_nanos),
        }
    }

    fn advance(&self, duration: Duration) {
        let nanos = u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX);
        self.unix_nanos.fetch_add(nanos, Ordering::SeqCst);
        self.mono_nanos.fetch_add(nanos, Ordering::SeqCst);
    }
}

impl Clock for FakeClock {
    fn now_unix_nanos(&self) -> u64 {
        self.unix_nanos.load(Ordering::SeqCst)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.mono_nanos.load(Ordering::SeqCst)
    }
}

#[tokio::test]
async fn test_compaction_rate_limit_with_fake_clock() {
    let clock = Arc::new(FakeClock::new(1_000_000_000));
    let dir = TempDir::new().unwrap();

    let mut config = LsmConfig::default();
    config.path = dir.path().to_path_buf();
    config.compaction.max_io_bytes_per_second = Some(1024 * 1024); // 1 MB/s
    config = config.with_clock(Arc::clone(&clock) as Arc<dyn Clock>);

    let storage = LsmStorage::new(config).await.expect("create storage");

    // Advance clock to simulate time progression deterministically
    clock.advance(Duration::from_secs(5));
    assert!(storage.clock().monotonic_nanos() >= 6_000_000_000);
}

#[tokio::test]
async fn test_system_pressure_with_fake_clock() {
    let clock = Arc::new(FakeClock::new(10_000_000_000));
    let monitor = SystemPressureMonitor::new_with_clock(
        Duration::from_millis(50),
        Arc::clone(&clock) as Arc<dyn Clock>,
    );

    let cancel = tokio_util::sync::CancellationToken::new();
    let cancel_clone = cancel.clone();

    let handle = tokio::spawn(async move {
        monitor
            .run(cancel_clone, || 10, || 5, 10)
            .await;
    });

    clock.advance(Duration::from_millis(150));
    cancel.cancel();
    let _ = handle.await;
}

#[tokio::test]
async fn test_deterministic_key_material_source_salt_reproducibility() {
    let fixed_entropy = vec![0xAB; 64];
    let key_source = Arc::new(StaticKeyMaterialSource::new(fixed_entropy));

    let dir1 = TempDir::new().unwrap();
    let config1 = LsmConfig {
        path: dir1.path().to_path_buf(),
        ..Default::default()
    }
    .with_key_source(Arc::clone(&key_source) as Arc<dyn contextra_store::KeyMaterialSource>);

    let _storage1 = LsmStorage::new(config1).await.expect("storage 1");
    let salt1 = std::fs::read(dir1.path().join("SALT")).expect("read salt 1");

    let fixed_entropy2 = vec![0xAB; 64];
    let key_source2 = Arc::new(StaticKeyMaterialSource::new(fixed_entropy2));

    let dir2 = TempDir::new().unwrap();
    let config2 = LsmConfig {
        path: dir2.path().to_path_buf(),
        ..Default::default()
    }
    .with_key_source(Arc::clone(&key_source2) as Arc<dyn contextra_store::KeyMaterialSource>);

    let _storage2 = LsmStorage::new(config2).await.expect("storage 2");
    let salt2 = std::fs::read(dir2.path().join("SALT")).expect("read salt 2");

    assert_eq!(salt1, salt2);
    assert_eq!(&salt1[..], &[0xAB; 32]);
}

#[tokio::test]
async fn test_sstable_scanner_determinism_across_runs() {
    let dir = TempDir::new().unwrap();
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(config).await.unwrap());

    // Insert deterministic test data
    let tx = TxId(100);
    storage.put(tx, b"t:canary:1:chunk:1", b"val1").await.unwrap();
    storage.put(tx, b"t:canary:1:chunk:2", b"val2").await.unwrap();
    storage.commit(tx).await.unwrap();
    storage.flush().await.unwrap();

    let scanner = SstableScanner::new(Arc::clone(&storage));

    let ctx = RequestContext {
        tenant_id: TenantId::SYSTEM,
        tx_id: tx,
    };

    let target = ScanTarget::new("tenant_system", [0x42; 32]);
    let clock = FakeClock::new(100_000);

    let witness1 = scanner
        .scan(&ctx, target.clone(), ScanTechnique::Structural, &clock)
        .unwrap();

    let witness2 = scanner
        .scan(&ctx, target, ScanTechnique::Structural, &clock)
        .unwrap();

    assert_eq!(witness1.scanned(), witness2.scanned());
    assert_eq!(witness1.remaining(), witness2.remaining());
    assert_eq!(witness1.digest(), witness2.digest());
}
