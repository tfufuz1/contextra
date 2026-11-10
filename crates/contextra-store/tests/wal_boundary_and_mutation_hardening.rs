//! Invariant and mutation tests for WAL framing, replay, and fail-closed durability (INV-FSYNC-1).

use contextra_store::wal::{PoisonCause, Wal, WalOp};
use contextra_types::TxId;
use tempfile::tempdir;

#[tokio::test]
async fn test_inv_fsync_1_handle_poisoning_contract() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("inv_fsync_contract.wal");
    let wal = Wal::open(&wal_path).await?;

    assert!(!wal.is_poisoned());
    assert_eq!(wal.poison_cause(), None);

    // Irrevocable poisoning on fsync failure
    wal.poison_with_cause(PoisonCause::FsyncFailed);
    assert!(wal.is_poisoned());
    assert_eq!(wal.poison_cause(), Some(PoisonCause::FsyncFailed));

    // Subsequent appends must fail immediately
    let op = WalOp::Put {
        tx_id: TxId::new(10),
        key: b"poison_key".to_vec(),
        value: b"poison_val".to_vec(),
    };
    let (batch, _) = wal.prepare_batch(vec![(op, 10)]).await?;
    let append_res = wal.append_batch(batch).await;
    assert!(
        append_res.is_err(),
        "Append must be rejected on poisoned handle"
    );

    // Recovery from fsync failure must be rejected fail-closed
    let rec_res = wal.recover_from_poison().await;
    assert!(rec_res.is_err(), "Cannot recover from FsyncFailed cause");
    assert!(wal.is_poisoned());

    Ok(())
}

#[tokio::test]
async fn test_wal_mutation_bitflip_rejection_in_middle() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("mutation_middle.wal");
    let wal = Wal::open(&wal_path).await?;

    // Append 3 valid entries
    for i in 1..=3 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("k{i}").into_bytes(),
            value: format!("v{i}").into_bytes(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, i)]).await?;
        wal.append_batch(batch).await?;
    }

    let raw_bytes = tokio::fs::read(&wal_path).await?;
    assert!(raw_bytes.len() > 100);

    // Flip a bit in the middle of entry 2 (offset between header and EOF)
    let mut corrupt_bytes = raw_bytes.clone();
    let corrupt_offset = 60usize;
    corrupt_bytes[corrupt_offset] ^= 0x40;

    let corrupt_path = dir.path().join("corrupted_middle.wal");
    tokio::fs::write(&corrupt_path, &corrupt_bytes).await?;

    let corrupt_wal = Wal::open(&corrupt_path).await?;
    let replay_res = corrupt_wal.replay().await;

    // Fail-closed: corruption in the middle MUST fail rather than silently truncating
    assert!(
        replay_res.is_err(),
        "Middle-file corruption must return Err"
    );

    Ok(())
}

#[tokio::test]
async fn test_wal_clean_torn_tail_cutoff() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("torn_tail.wal");
    let wal = Wal::open(&wal_path).await?;

    for i in 1..=2 {
        let op = WalOp::Put {
            tx_id: TxId::new(i),
            key: format!("k{i}").into_bytes(),
            value: format!("v{i}").into_bytes(),
        };
        let (batch, _) = wal.prepare_batch(vec![(op, i)]).await?;
        wal.append_batch(batch).await?;
    }

    let mut raw_bytes = tokio::fs::read(&wal_path).await?;

    // Append partial torn write at tail (e.g. only 3 bytes of a new entry length header)
    raw_bytes.extend_from_slice(&[0x10, 0x00, 0x00]);
    let torn_path = dir.path().join("torn_tail_file.wal");
    tokio::fs::write(&torn_path, &raw_bytes).await?;

    let torn_wal = Wal::open(&torn_path).await?;
    let replayed = torn_wal.replay().await?;

    // Exactly the 2 confirmed entries must be recovered
    assert_eq!(replayed.len(), 2);

    Ok(())
}
