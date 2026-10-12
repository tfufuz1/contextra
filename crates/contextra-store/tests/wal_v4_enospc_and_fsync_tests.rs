//! Status: WAL V4 features (segment preallocation, poisoned handles, V4 format) are work-in-progress.
//! Reference: Contextra Storage Specification v17 §8.4 (WAL V4 Specification).
//! Tests in this file are feature-gated until WAL V4 implementation is completed.
#![cfg(feature = "wal-v4-wip")]

use contextra_core::TxId;
use contextra_store::wal::{Wal, WalConfig, WalFlusherConfig, WalOp, WalVersion};
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_wal_v4_segment_preallocation_option() {
    let tmp = TempDir::new().expect("tempdir");
    let wal_path = tmp.path().join("wal_v4_prealloc.log");

    let prealloc_size = 64 * 1024; // 64 KB pre-allocation

    let config = WalConfig {
        target_wal_version: WalVersion::V4,
        min_wal_version: WalVersion::V4,
        segment_preallocation_bytes: Some(prealloc_size),
        ..Default::default()
    };

    let wal = Wal::open_with_config(&wal_path, config)
        .await
        .expect("open wal");

    let metadata_initial = fs::metadata(&wal_path).await.expect("metadata");
    assert!(
        metadata_initial.len() >= prealloc_size,
        "WAL file size must be preallocated to at least {}",
        prealloc_size
    );

    let _ = wal;
}

#[tokio::test]
async fn test_wal_v4_fsync_failure_poisons_handle() {
    let tmp = TempDir::new().expect("tempdir");
    let wal_path = tmp.path().join("wal_v4_fsync_poison.log");

    let config = WalConfig {
        target_wal_version: WalVersion::V4,
        min_wal_version: WalVersion::V4,
        flusher_config: WalFlusherConfig {
            batch_window_micros: 0,
            queue_capacity: 10,
        },
        ..Default::default()
    };

    let wal = Wal::open_with_config(&wal_path, config)
        .await
        .expect("open wal");

    assert!(!wal.is_poisoned());

    let (batch, _) = wal
        .prepare_batch(vec![(
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"k1".to_vec(),
                value: b"v1".to_vec(),
            },
            1,
        )])
        .await
        .expect("prepare batch");

    wal.append_batch(batch).await.expect("append batch");
    assert!(!wal.is_poisoned());
}
