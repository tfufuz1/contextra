#![no_main]

use acta_store::wal::Wal;
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

        // Fuzz Wal open and replay on arbitrary byte streams
        if let Ok(wal) = Wal::open(&wal_path).await {
            let _ = wal.replay().await;
        }
    });
});
