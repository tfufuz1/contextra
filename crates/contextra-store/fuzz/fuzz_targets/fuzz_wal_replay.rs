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
        let wal_path = dir.path().join("fuzz_replay.wal");

        if tokio::fs::write(&wal_path, data).await.is_err() {
            return;
        }

        let mut config = WalConfig::default();
        config.min_wal_version = WalVersion::V1;
        let config = config.with_legacy_fallback(true);

        if let Ok(wal) = Wal::open_with_config(&wal_path, config).await {
            let _ = wal.replay().await;
            let file_size = tokio::fs::metadata(&wal_path)
                .await
                .map(|m| m.len())
                .unwrap_or(0);
            let _ = wal.scan_entries_buffer(file_size, |_, _, _| true).await;
        }
    });
});
