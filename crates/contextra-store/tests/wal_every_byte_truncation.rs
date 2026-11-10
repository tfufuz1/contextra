//! WP-30 (a): Schneidet ein WAL an JEDER einzelnen Byte-Grenze ab.
//! Verifiziert: Wiederöffnen lädt konsistent bis zum letzten vollständigen Eintrag
//! ODER endet mit einem sauberen Fehler, niemals mit einer Panic.

#![forbid(unsafe_code)]

use contextra_ports::StorageEngine;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::TxId;
use tempfile::TempDir;

#[tokio::test]
async fn test_wal_truncation_at_every_single_byte_boundary() {
    let tmp = TempDir::new().expect("temp dir");
    let primary_dir = tmp.path().join("primary");

    let config = LsmConfig {
        path: primary_dir.clone(),
        group_commit_window_micros: 0,
        ..Default::default()
    };

    let storage = LsmStorage::new(config.clone()).await.expect("open storage");
    for i in 1..=5 {
        let tx = TxId::new(i);
        let k = format!("key_{i:04}").into_bytes();
        let v = format!("value_{i:04}_payload_data").into_bytes();
        storage.put(tx, &k, &v).await.expect("put");
        storage.commit(tx).await.expect("commit");
    }

    storage.wait_shutdown().await;
    drop(storage);

    let mut wal_path = None;
    let mut entries = tokio::fs::read_dir(&primary_dir).await.expect("read_dir");
    while let Ok(Some(entry)) = entries.next_entry().await {
        let p = entry.path();
        if p.extension().map_or(false, |ext| ext == "log") {
            wal_path = Some(p);
            break;
        }
    }
    let wal_path = wal_path.expect("wal file exists");
    let initial_wal_bytes = tokio::fs::read(&wal_path).await.expect("read wal file");
    let total_len = initial_wal_bytes.len();
    assert!(total_len > 100, "WAL must have sufficient bytes, got {total_len}");

    for cutoff in 0..=total_len {
        let test_dir = tmp.path().join(format!("trunc_byte_{cutoff}"));
        tokio::fs::create_dir_all(&test_dir).await.expect("create test_dir");

        let mut rd = tokio::fs::read_dir(&primary_dir).await.expect("read primary");
        while let Ok(Some(e)) = rd.next_entry().await {
            let dest = test_dir.join(e.file_name());
            if e.path().is_file() {
                tokio::fs::copy(e.path(), &dest).await.expect("copy file");
            }
        }

        let test_wal = test_dir.join(wal_path.file_name().unwrap());
        tokio::fs::write(&test_wal, &initial_wal_bytes[..cutoff])
            .await
            .expect("write truncated wal");

        let test_cfg = LsmConfig {
            path: test_dir,
            group_commit_window_micros: 0,
            ..Default::default()
        };

        let res = std::panic::AssertUnwindSafe(LsmStorage::new(test_cfg));
        let open_res = res.await;

        match open_res {
            Ok(recovered_storage) => {
                let mut prev_found = true;
                for i in 1..=5 {
                    let k = format!("key_{i:04}").into_bytes();
                    let expected_val = format!("value_{i:04}_payload_data").into_bytes();
                    let got = recovered_storage.get(&k).await.expect("get cannot fail");
                    if let Some(val) = got {
                        assert!(prev_found, "Partial transaction sequence gap at {i}");
                        assert_eq!(val.as_ref(), expected_val.as_slice());
                    } else {
                        prev_found = false;
                    }
                }
                recovered_storage.wait_shutdown().await;
            }
            Err(e) => {
                let msg = e.to_string();
                assert!(!msg.is_empty(), "Error message must not be empty");
            }
        }
    }
}
