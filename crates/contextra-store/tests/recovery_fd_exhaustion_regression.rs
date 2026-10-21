#[cfg(target_os = "linux")]
use contextra_ports::StorageEngine;
#[cfg(target_os = "linux")]
use contextra_store::lsm::{LsmConfig, LsmStorage};
#[cfg(target_os = "linux")]
use contextra_store::sstable::SstableBuilder;
#[cfg(target_os = "linux")]
use contextra_store::wal::{Wal, WalOp};
#[cfg(target_os = "linux")]
use contextra_types::TxId;
#[cfg(target_os = "linux")]
use tempfile::TempDir;

#[cfg(target_os = "linux")]
fn get_open_fd_count() -> usize {
    if let Ok(entries) = std::fs::read_dir("/proc/self/fd") {
        entries.filter_map(|e| e.ok()).count()
    } else {
        0
    }
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_recovery_fd_exhaustion_regression() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    // Write SALT so directory is valid for opening pre-populated WAL/SSTable files
    tokio::fs::write(db_path.join("SALT"), &[0u8; 32])
        .await
        .unwrap();

    // Create 100 small WAL segment files.
    let wal_count = 100;
    for i in 1..=wal_count {
        let wal_path = db_path.join(format!("wal-{:020}.log", i));
        let wal = Wal::open_with_key_manager(&wal_path, None).await.unwrap();
        let (batch, _hmac) = wal
            .prepare_batch(vec![(
                WalOp::Put {
                    key: format!("wal_key{:05}", i).into_bytes(),
                    value: format!("wal_value{:05}", i).into_bytes(),
                    tx_id: TxId::new(i as u64),
                },
                i as u64,
            )])
            .await
            .unwrap();
        wal.append_batch(batch).await.unwrap();
        let (end_batch, _) = wal
            .prepare_batch(vec![(
                WalOp::TxEnd {
                    tx_id: TxId::new(i as u64),
                    committed: true,
                },
                i as u64,
            )])
            .await
            .unwrap();
        wal.append_batch(end_batch).await.unwrap();
    }

    // Create 30 SSTable files.
    let sst_count = 30;
    for i in 1..=sst_count {
        let sst_path = db_path.join(format!("sst-{:020}-{:06}.sst", i, 0));
        let mut builder = SstableBuilder::create_with_key_manager(&sst_path, None)
            .await
            .unwrap();
        let key = format!("sst_key{:05}", i).into_bytes();
        let val = format!("sst_value{:05}", i).into_bytes();
        builder
            .add(&key, &val, (wal_count + i) as u64, (wal_count + i) as u64)
            .await
            .unwrap();
        builder.finish().await.unwrap();
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let initial_fd = get_open_fd_count();

    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };

    let storage = LsmStorage::new(config.clone())
        .await
        .expect("Recovery must succeed without EMFILE (too many open files)");

    // Verify recovery loaded entries correctly from both SSTables and WAL segments
    for i in 1..=wal_count {
        let key = format!("wal_key{:05}", i).into_bytes();
        let val = storage
            .get(&key)
            .await
            .unwrap()
            .expect("Replayed WAL key should exist");
        assert_eq!(val, format!("wal_value{:05}", i).as_bytes());
    }

    for i in 1..=sst_count {
        let key = format!("sst_key{:05}", i).into_bytes();
        let val = storage
            .get(&key)
            .await
            .unwrap()
            .expect("SSTable key should exist");
        assert_eq!(val, format!("sst_value{:05}", i).as_bytes());
    }

    drop(storage);

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let final_fd = get_open_fd_count();

    assert!(
        final_fd <= initial_fd + 5,
        "FD leak during recovery! Initial FD: {initial_fd}, Final FD: {final_fd}"
    );
}
