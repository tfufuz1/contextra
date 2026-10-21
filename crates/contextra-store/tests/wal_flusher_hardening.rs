use contextra_durable_fs::FaultInjector;
use contextra_store::wal::{
    flusher::WalFlusherConfig, PreparedBatch, Wal, WalEntry, WalOp, WalVersion,
};
use contextra_types::{ContextraError, Result, TxId};
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

struct FailAllWritesInjector;

impl FaultInjector for FailAllWritesInjector {
    fn on_write(&self, _path: Option<&Path>, _len: usize) -> std::io::Result<()> {
        Err(Error::new(
            ErrorKind::Other,
            "simulated write error for poison test",
        ))
    }
}

struct FailRenameInjector;

impl FaultInjector for FailRenameInjector {
    fn on_rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
        Err(Error::new(
            ErrorKind::Other,
            "simulated rename crash during rewrite",
        ))
    }
}

#[tokio::test]
async fn test_poison_state_answers_all_commands_with_typed_error() -> Result<()> {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("poison_all_commands.wal");

    let _file = contextra_store::wal::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(&path)
        .await
        .expect("open file");

    let mut wal_config = contextra_store::wal::WalConfig::default();
    wal_config.flusher_config = WalFlusherConfig::default();
    let _wal = Wal::open_with_config(&path, wal_config).await?;

    // Force append failure to poison handle
    let entry = WalEntry::try_new(
        WalOp::Put {
            key: b"k".to_vec(),
            value: b"v".to_vec(),
            tx_id: TxId(1),
        },
        1,
        &[1u8; 32],
        [0u8; 32],
    )?;

    // We replace the flusher with one that fails writes
    let file = contextra_store::wal::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(&path)
        .await?;

    // Create a new WAL handle with failing injector
    let wal_fail = Wal::open_read_only(&path, None).await?;
    wal_fail.enable_flusher_with_injector(
        file,
        WalFlusherConfig::default(),
        FailAllWritesInjector,
    )?;

    let res = wal_fail
        .append_batch(PreparedBatch::for_test(vec![entry]))
        .await;
    assert!(res.is_err(), "Append must fail with FailAllWritesInjector");
    assert!(wal_fail.is_poisoned(), "WAL handle must be poisoned");

    // Now send Truncate, Seal, Rewrite, Scan while poisoned
    let res_truncate = wal_fail.truncate(0, [0u8; 32]).await;
    assert!(
        res_truncate.is_err(),
        "Truncate on poisoned WAL must return Err"
    );
    let err_str = res_truncate.unwrap_err().to_string();
    assert!(
        err_str.contains("poisoned"),
        "Truncate error must indicate handle is poisoned: {}",
        err_str
    );

    let res_seal = wal_fail.rotate_and_seal().await;
    assert!(res_seal.is_err(), "Seal on poisoned WAL must return Err");
    let err_str = res_seal.unwrap_err().to_string();
    assert!(
        err_str.contains("poisoned"),
        "Seal error must indicate handle is poisoned: {}",
        err_str
    );

    let res_rewrite = wal_fail.rewrite_as_v3(&[]).await;
    assert!(
        res_rewrite.is_err(),
        "Rewrite on poisoned WAL must return Err"
    );
    let err_str = res_rewrite.unwrap_err().to_string();
    assert!(
        err_str.contains("poisoned"),
        "Rewrite error must indicate handle is poisoned: {}",
        err_str
    );

    let res_scan = wal_fail
        .scan_entries_with_callback(1024, |_seq, _entry, _pos| true)
        .await;
    assert!(res_scan.is_err(), "Scan on poisoned WAL must return Err");
    let err_str = res_scan.unwrap_err().to_string();
    assert!(
        err_str.contains("poisoned"),
        "Scan error must indicate handle is poisoned: {}",
        err_str
    );

    Ok(())
}

#[tokio::test]
async fn test_rewrite_crash_consistency_preserves_original_wal() -> Result<()> {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("crash_rewrite.wal");

    let wal = Wal::open(&path).await?;
    let entry = WalEntry::try_new(
        WalOp::Put {
            key: b"original_key".to_vec(),
            value: b"original_val".to_vec(),
            tx_id: TxId(100),
        },
        1,
        &[1u8; 32],
        [0u8; 32],
    )?;
    wal.append_batch(PreparedBatch::for_test(vec![entry.clone()]))
        .await?;

    let size_before = wal.size();
    assert!(size_before > 0);

    let entries = wal.replay().await?;
    assert_eq!(entries.len(), 1);

    // Re-enable flusher with FailRenameInjector
    let file = contextra_store::wal::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(&path)
        .await?;

    let wal_crash = Wal::open_read_only(&path, None).await?;
    wal_crash.enable_flusher_with_injector(
        file,
        WalFlusherConfig::default(),
        FailRenameInjector,
    )?;

    let rewrite_res = wal_crash.rewrite_as_v3(&entries).await;
    assert!(rewrite_res.is_err(), "Rewrite must fail on rename fault");

    // Reopen original path and verify entries are completely intact!
    let wal_reopened = Wal::open(&path).await?;
    let replayed = wal_reopened.replay().await?;
    assert_eq!(
        replayed.len(),
        1,
        "Original WAL must preserve all entries after failed rewrite"
    );
    if let WalOp::Put { key, value, tx_id } = &replayed[0].1.op {
        assert_eq!(key, b"original_key");
        assert_eq!(value, b"original_val");
        assert_eq!(*tx_id, TxId(100));
    } else {
        panic!("Unexpected entry op");
    }

    Ok(())
}
