use contextra_core::{ContextraError, StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_store::manifest::{Manifest, ManifestEntry};
use contextra_store::wal::Wal;
use tempfile::tempdir;
use tokio::fs;

#[tokio::test]
async fn test_aead_tail_truncation_without_manifest_anchor_causes_resurrection() {
    let dir = tempdir().expect("create tempdir");
    let path = dir.path().to_path_buf();

    let manifest_path = path.join("MANIFEST");

    let key = b"key_resurrection";
    let val = b"val_resurrection";

    // Step 1: Create storage, put key (tx1), commit tx1, delete key (tx2), commit tx2
    let (hwm_tx2, wal_path, offset_tx2_start) = {
        let config = LsmConfig {
            path: path.clone(),
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.expect("create storage");

        let tx1 = TxId::new(1);
        storage.put(tx1, key, val).await.expect("put tx1");
        storage.commit(tx1).await.expect("commit tx1");

        let tx2 = TxId::new(2);
        storage.delete(tx2, key).await.expect("delete tx2");
        storage.commit(tx2).await.expect("commit tx2");

        // Find the wal file path in directory
        let mut wal_file_path = None;
        let mut read_dir = fs::read_dir(&path).await.expect("read dir");
        while let Some(entry) = read_dir.next_entry().await.expect("next entry") {
            let p = entry.path();
            if p.extension().map_or(false, |ext| ext == "wal") || p.file_name().map_or(false, |name| name == "wal.log") {
                wal_file_path = Some(p);
                break;
            }
        }
        let wal_p = wal_file_path.expect("found wal file");

        // Replay WAL directly to inspect offsets and HMACs
        let wal_read = Wal::open(&wal_p).await.expect("open wal read");
        let entries = wal_read.replay().await.expect("replay entries");

        assert!(entries.len() >= 4, "Expected at least 4 WAL entries");

        // Last entry is Commit(tx2).
        let (_, ref entry_tx2_commit, _) = entries.last().unwrap();
        let hwm2 = entry_tx2_commit.checksum;

        // Entry 2 is Delete(tx2). Its offset is offset_tx2_start.
        let (_, _, offset_entry_tx2) = entries[2];

        drop(wal_read);
        drop(storage);

        (hwm2, wal_p, offset_entry_tx2)
    };

    // Explicitly add WalCheckpoint for tx2 to MANIFEST so it acts as manifest anchor for tx2
    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");
    manifest
        .append(&ManifestEntry::WalCheckpoint { hmac: hwm_tx2 })
        .await
        .expect("append hwm_tx2");
    drop(manifest);

    // Step 2: Truncate WAL file precisely at offset_tx2_start (removing tx2 Delete + Commit)
    let mut wal_bytes = fs::read(&wal_path).await.expect("read wal_path");
    let initial_wal_len = wal_bytes.len();
    assert!(
        initial_wal_len > offset_tx2_start as usize,
        "WAL should be larger than tx1 commit offset"
    );

    wal_bytes.truncate(offset_tx2_start as usize);
    fs::write(&wal_path, wal_bytes)
        .await
        .expect("write truncated wal");

    // Replay remaining WAL bytes to get actual_hmac at tail
    let wal_remaining = Wal::open(&wal_path).await.expect("open wal remaining");
    let remaining_entries = wal_remaining.replay().await.expect("replay remaining");
    let actual_tail_hmac = remaining_entries.last().unwrap().1.checksum;
    drop(wal_remaining);

    // Scenario A: Manifest HWM anchor IS hwm_tx2, but WAL is truncated to tx1.
    // LsmStorage opening MUST fail closed with WalTruncationDetected, preventing resurrection!
    {
        let config = LsmConfig {
            path: path.clone(),
            ..Default::default()
        };
        let reopen_res = LsmStorage::new(config).await;
        assert!(
            reopen_res.is_err(),
            "Opening LsmStorage MUST detect WAL truncation against Manifest HWM anchor"
        );
        match reopen_res.err().unwrap() {
            ContextraError::WalTruncationDetected {
                expected_hmac,
                actual_hmac,
            } => {
                assert_eq!(expected_hmac, hwm_tx2);
                assert_eq!(actual_hmac, actual_tail_hmac);
            }
            err => panic!("Expected WalTruncationDetected, got: {:?}", err),
        }
    }

    // Scenario B: If Manifest anchor WAS NOT updated to tx2 (simulating crash before manifest checkpoint),
    // resetting manifest HWM to actual_tail_hmac allows LsmStorage to open, but key IS resurrected.
    {
        // Replace MANIFEST with only checkpoint up to tx1
        fs::remove_file(&manifest_path)
            .await
            .expect("remove manifest");
        let manifest = Manifest::open(&manifest_path)
            .await
            .expect("recreate manifest");
        manifest
            .append(&ManifestEntry::WalCheckpoint {
                hmac: actual_tail_hmac,
            })
            .await
            .expect("append actual_tail_hmac");
        drop(manifest);

        let config = LsmConfig {
            path: path.clone(),
            ..Default::default()
        };
        let storage = LsmStorage::new(config)
            .await
            .expect("reopen storage when manifest anchor is at actual_tail_hmac");

        let get_res = storage.get(key).await.expect("get key");
        assert_eq!(
            get_res,
            Some(bytes::Bytes::from_static(val)),
            "Key MUST be resurrected as val_resurrection because WAL truncation removed tombstone AND Manifest anchor was not advanced to tx2"
        );
    }
}
