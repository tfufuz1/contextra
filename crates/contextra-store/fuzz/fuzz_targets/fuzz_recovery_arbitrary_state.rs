#![no_main]

use contextra_core::StorageEngine;
use contextra_store::{LsmConfig, LsmStorage};
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

        let path = dir.path();

        // Slice input bytes across MANIFEST, wal-0.log, and 000001.sst
        let chunk_size = data.len() / 3;
        let manifest_bytes = &data[..chunk_size];
        let wal_bytes = &data[chunk_size..chunk_size * 2];
        let sstable_bytes = &data[chunk_size * 2..];

        if !manifest_bytes.is_empty() {
            if tokio::fs::write(path.join("MANIFEST"), manifest_bytes).await.is_err() {
                return;
            }
        }

        if !wal_bytes.is_empty() {
            if tokio::fs::write(path.join("wal-0.log"), wal_bytes).await.is_err() {
                return;
            }
        }

        if !sstable_bytes.is_empty() {
            if tokio::fs::write(path.join("000001.sst"), sstable_bytes).await.is_err() {
                return;
            }
        }

        let config = LsmConfig {
            db_path: path.to_path_buf(),
            allow_legacy_integrity_key_fallback: true,
            ..Default::default()
        };

        if let Ok(storage) = LsmStorage::new(config).await {
            let _ = storage.get(b"test_key").await;
            let _ = storage.scan_prefix(b"").await;
            let _ = storage.last_seq_no().await;
            let _ = storage.last_tx_id().await;
            let _ = storage.stats().await;
        }
    });
});
