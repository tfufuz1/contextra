//! Replay Filter Hook Integration Tests.
//!
//! Verifies that `WalReplayFilter` correctly filters WAL entries during replay,
//! respects transaction ID thresholds, and accurately updates `WalReplayReport`.

use contextra_core::{ContextraError, TxId};
use contextra_store::wal::{Wal, WalOp, WalReplayFilter};
use tempfile::tempdir;

struct TestPrefixAndTxFilter {
    drop_prefix: Vec<u8>,
    max_drop_tx: u64,
}

impl WalReplayFilter for TestPrefixAndTxFilter {
    fn drop_record(&self, key: &[u8], tx: u64) -> bool {
        key.starts_with(&self.drop_prefix) && tx <= self.max_drop_tx
    }
}

#[tokio::test]
async fn test_wal_replay_filter_dropping_and_counters() -> Result<(), ContextraError> {
    let dir = tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let wal_path = dir.path().join("filter_test.wal");

    let wal = Wal::open(&wal_path).await?;

    let ops = vec![
        (
            WalOp::Put {
                tx_id: TxId::new(5),
                key: b"drop_1".to_vec(),
                value: b"v1".to_vec(),
            },
            1,
        ),
        (
            WalOp::Put {
                tx_id: TxId::new(5),
                key: b"keep_1".to_vec(),
                value: b"v2".to_vec(),
            },
            2,
        ),
        (
            WalOp::Put {
                tx_id: TxId::new(15),
                key: b"drop_2".to_vec(),
                value: b"v3".to_vec(),
            },
            3,
        ),
        (
            WalOp::Delete {
                tx_id: TxId::new(8),
                key: b"drop_3".to_vec(),
            },
            4,
        ),
    ];

    let (batch, _) = wal.prepare_batch(ops).await?;
    wal.append_batch(batch).await?;

    // 1. Replay with None filter -> all 4 entries replayed
    let (entries_no_filter, report_no_filter) = wal.replay_with_filter(None).await?;
    if entries_no_filter.len() != 4
        || report_no_filter.records_replayed != 4
        || report_no_filter.records_dropped_by_filter != 0
    {
        return Err(ContextraError::Storage("replay_with_filter failed".into()));
    }

    // 2. Replay with TestPrefixAndTxFilter (drop_prefix = "drop_", max_drop_tx = 10)
    let filter = TestPrefixAndTxFilter {
        drop_prefix: b"drop_".to_vec(),
        max_drop_tx: 10,
    };

    let (filtered_entries, report_filtered) = wal.replay_with_filter(Some(&filter)).await?;

    if filtered_entries.len() != 2
        || report_filtered.records_replayed != 2
        || report_filtered.records_dropped_by_filter != 2
    {
        return Err(ContextraError::Storage(
            "replay_with_filter report mismatch".into(),
        ));
    }

    // Verify remaining keys
    let remaining_keys: Vec<Vec<u8>> = filtered_entries
        .into_iter()
        .map(|(_, entry, _)| match entry.op {
            WalOp::Put { key, .. } => key,
            WalOp::Delete { key, .. } => key,
            WalOp::TxEnd { .. } => Vec::new(),
        })
        .collect();

    if remaining_keys != vec![b"keep_1".to_vec(), b"drop_2".to_vec()] {
        return Err(ContextraError::Storage("remaining_keys mismatch".into()));
    }

    Ok(())
}
