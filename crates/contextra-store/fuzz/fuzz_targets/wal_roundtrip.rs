#![no_main]

use libfuzzer_sys::fuzz_target;
use contextra_store::wal::{Wal, WalConfig, WalVersion};
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

        // FIX(2026-10-07): Use public builder method for WalConfig to avoid private field access error
        let mut config = WalConfig::default().with_legacy_fallback(true);
        config.min_wal_version = WalVersion::V1;

        match Wal::open_with_config(&wal_path, config).await {
            Ok(wal) => {
                let _ = wal.replay().await;
            }
            Err(_) => {}
        }
    });
});
