//! Test to verify that a read error during directory scanning in `retire_segments_up_to`
//! (a) returns an error to the caller, and
//! (b) leaves all segment files unchanged (no files deleted or modified).

use contextra_durable_fs::ScrubPolicy;
use contextra_store::wal::{retire_segments_with_stream, Wal, WalDirEntry, WalDirStream, WalOp};
use contextra_types::{ContextraError, Result, TxId};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

struct MockErrorDirStream {
    entries: Vec<PathBuf>,
    idx: usize,
    error_at_idx: usize,
}

impl WalDirStream for MockErrorDirStream {
    async fn next_entry(&mut self) -> Result<Option<WalDirEntry>> {
        if self.idx == self.error_at_idx {
            return Err(ContextraError::Storage(
                "Simulated directory read error".to_string(),
            ));
        }
        if self.idx >= self.entries.len() {
            return Ok(None);
        }
        let path = self.entries[self.idx].clone();
        self.idx += 1;
        Ok(Some(WalDirEntry {
            path,
            is_dir: false,
        }))
    }
}

fn compute_dir_hashes(dir: &Path) -> HashMap<PathBuf, Vec<u8>> {
    let mut map = HashMap::new();
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("wal") {
                if let Ok(bytes) = std::fs::read(&path) {
                    let mut hasher = Sha256::new();
                    hasher.update(&bytes);
                    map.insert(path, hasher.finalize().to_vec());
                }
            }
        }
    }
    map
}

#[tokio::test]
async fn test_retire_segments_aborts_on_read_error_and_preserves_files() -> Result<()> {
    let dir = tempdir()?;
    let dir_path = dir.path();

    let seg1_path = dir_path.join("wal-00000000000000000001.wal");
    let wal1 = Wal::open(&seg1_path).await?;
    let op1 = WalOp::Put {
        tx_id: TxId::try_new(5).unwrap(),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal1.prepare_batch(vec![(op1, 1)]).await?;
    wal1.append_batch(batch1).await?;
    wal1.close().await?;

    let seg2_path = dir_path.join("wal-00000000000000000002.wal");
    let wal2 = Wal::open(&seg2_path).await?;
    let op2 = WalOp::Put {
        tx_id: TxId::try_new(8).unwrap(),
        key: b"k2".to_vec(),
        value: b"v2".to_vec(),
    };
    let (batch2, _) = wal2.prepare_batch(vec![(op2, 2)]).await?;
    wal2.append_batch(batch2).await?;
    wal2.close().await?;

    let hashes_before = compute_dir_hashes(dir_path);
    assert_eq!(
        hashes_before.len(),
        2,
        "Expected 2 segment files before retirement"
    );

    let stream = MockErrorDirStream {
        entries: vec![seg1_path.clone(), seg2_path.clone()],
        idx: 0,
        error_at_idx: 1, // Error on second entry
    };

    let result = retire_segments_with_stream(stream, 10, ScrubPolicy::Disabled).await;

    assert!(
        result.is_err(),
        "retire_segments_with_stream must return an error when stream fails"
    );

    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("Simulated directory read error"),
        "Unexpected error message: {err_str}"
    );

    let hashes_after = compute_dir_hashes(dir_path);
    assert_eq!(
        hashes_before, hashes_after,
        "Segment files were altered despite read error during directory iteration!"
    );

    Ok(())
}
