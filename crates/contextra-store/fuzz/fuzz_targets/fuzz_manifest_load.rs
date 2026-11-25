#![no_main]

use libfuzzer_sys::fuzz_target;
use contextra_store::manifest::Manifest;
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
        let manifest_path = dir.path().join("MANIFEST");

        if tokio::fs::write(&manifest_path, data).await.is_err() {
            return;
        }

        let _ = Manifest::load(&manifest_path).await;
    });
});
