#![no_main]

use contextra_store::wal::{Wal, WalConfig, WalVersion};
use libfuzzer_sys::fuzz_target;
use tempfile::tempdir;

fuzz_target!(|data: &[u8]| {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(_) => return,
    };

    rt.block_on(async {
        let dir = match tempdir() {
            Ok(d) => d,
            Err(_) => return,
        };
        let wal_path = dir.path().join("fuzz.wal");

        if tokio::fs::write(&wal_path, data).await.is_err() {
            return;
        }

        let mut config = WalConfig::default();
        config.min_wal_version = WalVersion::V1;
        let config = config.with_legacy_fallback(true);

        match Wal::open_with_config(&wal_path, config).await {
            Ok(wal) => {
                let _ = wal.replay().await;
            }
            Err(_) => {}
        }
    });
});
