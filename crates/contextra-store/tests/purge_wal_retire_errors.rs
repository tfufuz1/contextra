// FILE-CONTEXT
// STAND: 2026-10-16
// ZWECK: Test for WAL segment retirement failure handling when a segment replay fails (P-4 requirement).

use contextra_store::wal::{retire_segments_up_to, Wal, WalOp};
use contextra_types::{Result, TxId};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn compute_file_hashes(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut map = BTreeMap::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext == "log" {
                        if let Ok(bytes) = fs::read(&path) {
                            let mut hasher = Sha256::new();
                            hasher.update(&bytes);
                            map.insert(path, hasher.finalize().to_vec());
                        }
                    }
                }
            }
        }
    }
    map
}

#[tokio::test]
async fn test_wal_retire_replay_error_propagates_and_preserves_hashes() -> Result<()> {
    let dir = tempdir()?;
    let dir_path = dir.path();

    // Create a valid sealed WAL segment 1
    let seg1_path = dir_path.join("wal-00000000000000000001.log");
    let wal1 = Wal::open(&seg1_path).await?;
    let op1 = WalOp::Put {
        tx_id: TxId::try_new(5).unwrap(),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal1.prepare_batch(vec![(op1, 1)]).await?;
    wal1.append_batch(batch1).await?;
    wal1.close().await?;

    // Create a corrupted WAL segment 2
    let seg2_path = dir_path.join("wal-00000000000000000002.log");
    fs::write(&seg2_path, b"CORRUPTED_GARBAGE_WAL_DATA_HEADER_INVALID").unwrap();

    let pre_hashes = compute_file_hashes(dir_path);
    assert_eq!(
        pre_hashes.len(),
        2,
        "Expected 2 files before retirement attempt"
    );

    // Call retire_segments_up_to
    let res = retire_segments_up_to(dir_path, 100).await;

    // Assert it returns Err
    assert!(
        res.is_err(),
        "retire_segments_up_to must propagate replay error as Err"
    );

    // Assert file hashes before and after failure are identical
    let post_hashes = compute_file_hashes(dir_path);
    assert_eq!(
        pre_hashes, post_hashes,
        "File hashes changed after failed segment retirement"
    );

    Ok(())
}
