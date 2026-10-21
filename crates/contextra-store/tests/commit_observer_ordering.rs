use contextra_store::{CommittedBatch, LsmConfig, LsmStorage, WalObserver};
use contextra_types::TxId;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct VisibilityVerifyingObserver {
    last_applied_seq: Arc<std::sync::atomic::AtomicU64>,
    last_committed_tx: Arc<std::sync::atomic::AtomicU64>,
    visibility_verified: Arc<AtomicBool>,
}

impl WalObserver for VisibilityVerifyingObserver {
    fn on_commit(&self, batch: &CommittedBatch<'_>, seq_no: u64, tx_id: TxId) {
        // Observers are notified during commit. Check if sequence number and visibility horizon were updated.
        let applied_seq = self.last_applied_seq.load(Ordering::Acquire);
        let committed_tx = self.last_committed_tx.load(Ordering::Acquire);

        if applied_seq >= seq_no && committed_tx >= tx_id.inner() && !batch.entries.is_empty() {
            self.visibility_verified.store(true, Ordering::SeqCst);
        }
    }
}

#[tokio::test]
async fn test_group_commit_observer_sees_visible_data() -> contextra_types::Result<()> {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    let mut config = LsmConfig::default();
    // Enable group commit with a 10ms window
    config.group_commit_window_micros = 10_000;

    let verified_flag = Arc::new(AtomicBool::new(false));

    let storage = Arc::new(LsmStorage::open_with_config(temp_dir.path(), config).await?);

    let observer = Arc::new(VisibilityVerifyingObserver {
        last_applied_seq: Arc::clone(&storage.last_applied_seq),
        last_committed_tx: Arc::clone(&storage.last_committed_tx),
        visibility_verified: Arc::clone(&verified_flag),
    });

    storage.register_observer(observer);

    // Perform two concurrent transactions to trigger group commit (leader + follower)
    let s1 = Arc::clone(&storage);
    let handle1 = tokio::spawn(async move {
        let tx = TxId::new(100);
        s1.put(tx, b"key1", b"val1").await?;
        s1.commit(tx).await
    });

    let s2 = Arc::clone(&storage);
    let handle2 = tokio::spawn(async move {
        let tx = TxId::new(101);
        s2.put(tx, b"key2", b"val2").await?;
        s2.commit(tx).await
    });

    let (res1, res2) = tokio::join!(handle1, handle2);
    res1.expect("handle1 panicked")?;
    res2.expect("handle2 panicked")?;

    assert!(
        verified_flag.load(Ordering::SeqCst),
        "Observer was called before data was visible to readers in LSM storage"
    );

    Ok(())
}
