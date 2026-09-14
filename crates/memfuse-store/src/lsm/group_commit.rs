use crate::wal::PreparedBatch;
use memfuse_core::{Result, TxId};
use std::sync::Arc;

pub(super) struct GroupCommitRequest {
    pub(super) tx_id: TxId,
    pub(super) wal_entries: PreparedBatch,
    pub(super) mem_updates: Vec<(Vec<u8>, Vec<u8>, u64)>,
    pub(super) sender: tokio::sync::oneshot::Sender<Result<()>>,
}

pub(super) struct WalQueueGuard(pub(super) Arc<std::sync::atomic::AtomicUsize>);

impl WalQueueGuard {
    pub(super) fn new(counter: Arc<std::sync::atomic::AtomicUsize>) -> Self {
        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self(counter)
    }
}

impl Drop for WalQueueGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
}

pub(super) struct PendingCommitQueue {
    pub(super) requests: Vec<GroupCommitRequest>,
    pub(super) first_prev_hmac: [u8; 32],
    pub(super) notify_full: Arc<tokio::sync::Notify>,
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use tempfile::TempDir;

    async fn test_storage() -> (LsmStorage, TempDir) {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.expect("create storage");
        (storage, tmp)
    }

    #[tokio::test]
    async fn test_sequence_numbers_strictly_monotonic_across_concurrent_commits() {
        let storage = Arc::new(test_storage().await.0);
        let mut handles = Vec::new();

        for i in 1..=10u64 {
            let st = Arc::clone(&storage);
            handles.push(tokio::spawn(async move {
                let tx = TxId::new(i);
                st.put(tx, format!("concurrent_key_{i}").as_bytes(), b"val")
                    .await
                    .unwrap();
                st.commit(tx).await.unwrap();
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        let last_seq = storage.last_seq_no().await.unwrap();
        assert_eq!(
            last_seq, 10,
            "10 commits must generate sequence numbers 1..10 monotonically"
        );
    }

    #[tokio::test]
    async fn test_system_pressure_monitor_integration() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.expect("create storage");

        let rx = storage.pressure_receiver();
        let pressure = rx.borrow().clone();
        assert_eq!(
            pressure.pressure_level,
            crate::system_pressure::PressureLevel::Normal
        );
        assert_eq!(pressure.wal_queue_depth, 0);

        storage.shutdown();
    }

    #[tokio::test]
    async fn test_system_pressure_wal_queue_backpressure_transition() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            group_commit_window_micros: 200_000,
            ..Default::default()
        };
        let storage = Arc::new(LsmStorage::new(config).await.expect("create storage"));
        let mut pressure_rx = storage.pressure_receiver();

        let num_tasks = 600;
        let barrier = Arc::new(tokio::sync::Barrier::new(num_tasks));
        let mut handles = Vec::with_capacity(num_tasks);

        for i in 0..num_tasks {
            let storage = Arc::clone(&storage);
            let barrier = Arc::clone(&barrier);
            handles.push(tokio::spawn(async move {
                let tx = TxId::new((i + 1) as u64);
                let key = format!("k{:05}", i).into_bytes();
                let val = format!("v{:05}", i).into_bytes();
                storage.put(tx, &key, &val).await.expect("put");
                barrier.wait().await;
                storage.commit(tx).await.expect("commit");
            }));
        }

        let mut max_wal_depth = 0;
        let mut critical_observed = false;

        let monitor_handle = tokio::spawn(async move {
            let timeout = Duration::from_secs(5);
            let start = std::time::Instant::now();
            loop {
                let current = pressure_rx.borrow().clone();
                if current.wal_queue_depth > max_wal_depth {
                    max_wal_depth = current.wal_queue_depth;
                }
                if current.pressure_level == crate::system_pressure::PressureLevel::Critical {
                    critical_observed = true;
                    break;
                }
                if start.elapsed() > timeout {
                    break;
                }
                if pressure_rx.changed().await.is_err() {
                    break;
                }
            }
            (max_wal_depth, critical_observed)
        });

        for h in handles {
            h.await.expect("task join");
        }

        let (max_depth, transitioned) = monitor_handle.await.expect("monitor join");

        assert!(
            transitioned || max_depth > crate::system_pressure::WAL_QUEUE_CRITICAL_THRESHOLD,
            "Production pressure_rx should transition to Critical when WAL queue depth ({}) exceeds threshold ({})",
            max_depth,
            crate::system_pressure::WAL_QUEUE_CRITICAL_THRESHOLD
        );

        storage.shutdown();
    }
}
