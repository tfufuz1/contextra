#![no_main]

use contextra_store::sstable::{BlockCache, SstableReader};
use libfuzzer_sys::fuzz_target;
use std::sync::Arc;
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
        let sst_path = dir.path().join("legacy.sst");
        if tokio::fs::write(&sst_path, data).await.is_err() {
            return;
        }

        let cache = Arc::new(BlockCache::new(512 * 1024));
        if let Ok(reader) = SstableReader::open(&sst_path, cache).await {
            let _ = reader.get(b"test_key").await;
        }
    });
});
