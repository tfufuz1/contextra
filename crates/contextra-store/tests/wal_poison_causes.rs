// ANCHOR[TEST:WAL-POISON-CAUSES] STATUS:DONE (TS:2026-11-08)
//! Test suite verifying fail-closed WAL durability and PoisonCause discipline (WP-STO-3 / INV-FSYNC-1).

#![cfg(feature = "fault-injection")]

use contextra_store::wal::{
    PoisonCause, Wal, WalOp, FAIL_APPEND_AFTER_PARTIAL_BYTES, FAIL_APPEND_PARTIAL_ONCE,
    FAIL_SYNC_ONCE, FAIL_TRUNCATE_ONCE,
};
use contextra_types::TxId;
use tempfile::tempdir;

static TEST_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tokio::test]
async fn fsync_fail_poisons_irreversibly() -> contextra_types::Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let dir = tempdir()?;
    let wal_path = dir.path().join("fsync_fail.wal");
    let wal = Wal::open(&wal_path).await?;

    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await?;
    wal.append_batch(batch1).await?;

    assert_eq!(wal.poison_cause(), None);
    assert!(!wal.is_poisoned());

    // Inject fsync failure
    FAIL_SYNC_ONCE.store(true, std::sync::atomic::Ordering::SeqCst);

    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"k2".to_vec(),
        value: b"v2".to_vec(),
    };
    let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await?;
    let res = wal.append_batch(batch2).await;
    assert!(res.is_err(), "Append must fail when fsync fails");

    // Handle must be poisoned with FsyncFailed
    assert!(wal.is_poisoned());
    assert_eq!(wal.poison_cause(), Some(PoisonCause::FsyncFailed));

    // Recovery must be rejected for FsyncFailed
    let rec_res = wal.recover_from_poison().await;
    assert!(
        rec_res.is_err(),
        "recover_from_poison must reject FsyncFailed cause"
    );

    // Poison must remain irreversible
    assert!(wal.is_poisoned());
    assert_eq!(wal.poison_cause(), Some(PoisonCause::FsyncFailed));

    Ok(())
}

#[tokio::test]
async fn set_len_fail_poisons() -> contextra_types::Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let dir = tempdir()?;
    let wal_path = dir.path().join("set_len_fail.wal");
    let wal = Wal::open(&wal_path).await?;

    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await?;
    wal.append_batch(batch1).await?;

    // Inject truncate failure (which simulates failure in truncate command in flusher)
    FAIL_TRUNCATE_ONCE.store(true, std::sync::atomic::Ordering::SeqCst);

    let trunc_res = wal.truncate(4, [0u8; 32]).await;
    assert!(
        trunc_res.is_err(),
        "Truncate must fail under fault injection"
    );

    // Handle must be poisoned with TruncateFailed
    assert!(wal.is_poisoned());
    assert_eq!(wal.poison_cause(), Some(PoisonCause::TruncateFailed));

    // Subsequent append must fail immediately with Err
    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"k2".to_vec(),
        value: b"v2".to_vec(),
    };
    let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await?;
    let append_res = wal.append_batch(batch2).await;
    assert!(append_res.is_err(), "Append must fail on poisoned handle");

    // Recovery must be rejected for TruncateFailed
    let rec_res = wal.recover_from_poison().await;
    assert!(
        rec_res.is_err(),
        "recover_from_poison must reject TruncateFailed cause"
    );
    assert_eq!(wal.poison_cause(), Some(PoisonCause::TruncateFailed));

    Ok(())
}

#[tokio::test]
async fn rollback_contention_poisons_handle() -> contextra_types::Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let dir = tempdir()?;
    let wal_path = dir.path().join("rollback_contention.wal");
    let wal = Wal::open(&wal_path).await?;

    let prev_hmac = [42u8; 32];

    // Hold last_hmac lock to simulate contention
    let hmac_lock = wal.last_hmac_lock_for_test();
    let lock_guard = hmac_lock.lock().await;

    // rollback_hmac_sync cannot acquire lock -> returns false
    let rollback_ok = wal.rollback_hmac_sync(prev_hmac);
    assert!(
        !rollback_ok,
        "rollback_hmac_sync must fail under contention"
    );

    // Poison handle with HmacChainDiverged
    wal.poison_with_cause(PoisonCause::HmacChainDiverged);
    assert!(wal.is_poisoned());
    assert_eq!(wal.poison_cause(), Some(PoisonCause::HmacChainDiverged));

    drop(lock_guard);

    // Recovery must be rejected for HmacChainDiverged
    let rec_res = wal.recover_from_poison().await;
    assert!(
        rec_res.is_err(),
        "recover_from_poison must reject HmacChainDiverged cause"
    );

    Ok(())
}

#[tokio::test]
async fn recover_from_poison_rejects_fsync_cause() -> contextra_types::Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let dir = tempdir()?;
    let wal_path = dir.path().join("reject_fsync.wal");
    let wal = Wal::open(&wal_path).await?;

    wal.poison_with_cause(PoisonCause::FsyncFailed);
    assert_eq!(wal.poison_cause(), Some(PoisonCause::FsyncFailed));

    let res = wal.recover_from_poison().await;
    assert!(res.is_err());
    assert_eq!(wal.poison_cause(), Some(PoisonCause::FsyncFailed));

    // Also check WriteFailed
    let wal2_path = dir.path().join("reject_write.wal");
    let wal2 = Wal::open(&wal2_path).await?;
    wal2.poison_with_cause(PoisonCause::WriteFailed);
    assert!(wal2.recover_from_poison().await.is_err());
    assert_eq!(wal2.poison_cause(), Some(PoisonCause::WriteFailed));

    Ok(())
}

#[tokio::test]
async fn recover_from_poison_accepts_torn_write() -> contextra_types::Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let dir = tempdir()?;
    let wal_path = dir.path().join("torn_write.wal");
    let wal = Wal::open(&wal_path).await?;

    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await?;
    wal.append_batch(batch1).await?;

    // Inject partial write
    FAIL_APPEND_AFTER_PARTIAL_BYTES.store(15, std::sync::atomic::Ordering::SeqCst);
    FAIL_APPEND_PARTIAL_ONCE.store(true, std::sync::atomic::Ordering::SeqCst);

    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"k2".to_vec(),
        value: b"v2_large_payload_for_partial_write".to_vec(),
    };
    let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await?;
    let _ = wal.append_batch(batch2).await;

    assert!(wal.is_poisoned());
    assert_eq!(wal.poison_cause(), Some(PoisonCause::TornWrite));

    // Recovery must succeed for TornWrite!
    wal.recover_from_poison().await?;
    assert!(!wal.is_poisoned());
    assert_eq!(wal.poison_cause(), None);

    Ok(())
}
