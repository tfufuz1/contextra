use contextra_core::{StorageEngine, TxId};
use contextra_store::compaction::CompactionConfig;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use tempfile::TempDir;

fn get_open_fd_count() -> usize {
    if let Ok(entries) = std::fs::read_dir("/proc/self/fd") {
        entries.filter_map(|e| e.ok()).count()
    } else {
        0
    }
}

#[tokio::test]
async fn test_reopen_fd_leak() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let config = LsmConfig {
        path: db_path.clone(),
        compaction: CompactionConfig {
            check_interval: std::time::Duration::from_millis(10),
            ..Default::default()
        },
        ..Default::default()
    };

    // Initial open & close to populate SALT / MANIFEST / WAL
    {
        let db = LsmStorage::open(config.clone()).await.unwrap();
        db.close().await.unwrap();
    }

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let initial_fd = get_open_fd_count();

    for i in 1..=200 {
        let db = LsmStorage::open(config.clone()).await.unwrap();
        for j in 0..5 {
            let key = format!("key_{i}_{j}").into_bytes();
            let val = format!("val_{i}_{j}").into_bytes();
            db.put(TxId::new((i * 10 + j) as u64), &key, &val)
                .await
                .unwrap();
            db.commit(TxId::new((i * 10 + j) as u64)).await.unwrap();
        }
        if i % 10 == 0 {
            db.flush().await.unwrap();
            db.maybe_compact_for_test().await.unwrap();
        }
        db.close().await.unwrap();
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let final_fd = get_open_fd_count();
    println!(
        "Reopen test - FD count initial: {}, final: {}",
        initial_fd, final_fd
    );

    assert!(
        final_fd <= initial_fd + 2,
        "FD leak detected across reopen cycles! Initial: {}, Final: {}",
        initial_fd,
        final_fd
    );
}
