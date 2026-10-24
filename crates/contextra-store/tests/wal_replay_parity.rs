use contextra_core::{ContextraError, TxId};
use contextra_store::wal::{KeyManager, Wal, WalOp};
use std::sync::Arc;
use tempfile::tempdir;

async fn create_sample_wal(
    dir: &std::path::Path,
    filename: &str,
    km: Option<Arc<KeyManager>>,
) -> std::path::PathBuf {
    let wal_path = dir.join(filename);
    let wal = Wal::open_with_key_manager(&wal_path, km)
        .await
        .expect("open wal");

    let ops = vec![
        (
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"parity_k1".to_vec(),
                value: b"parity_v1".to_vec(),
            },
            10,
        ),
        (
            WalOp::Put {
                tx_id: TxId::new(2),
                key: b"parity_k2".to_vec(),
                value: b"parity_v2".to_vec(),
            },
            11,
        ),
        (
            WalOp::TxEnd {
                tx_id: TxId::new(2),
                committed: true,
            },
            12,
        ),
    ];

    let (batch, _) = wal.prepare_batch(ops).await.expect("prepare batch");
    wal.append_batch(batch).await.expect("append batch");
    wal_path
}

fn error_class_matches(a: &ContextraError, b: &ContextraError) -> bool {
    match (a, b) {
        (ContextraError::WalCorruption { .. }, ContextraError::WalCorruption { .. }) => true,
        (ContextraError::Serialization(_), ContextraError::Serialization(_)) => true,
        (ContextraError::Storage(_), ContextraError::Storage(_)) => true,
        _ => std::mem::discriminant(a) == std::mem::discriminant(b),
    }
}

async fn check_parity(wal_path: &std::path::Path, km: Option<Arc<KeyManager>>) {
    let open_res = Wal::open_with_key_manager(wal_path, km).await;

    match open_res {
        Ok(wal) => {
            let mmap_res = wal.replay_mmap().await;
            let stream_res = wal.replay_stream().await;

            match (mmap_res, stream_res) {
                (Ok((mmap_entries, _mmap_ver)), Ok(stream_entries)) => {
                    assert_eq!(
                        mmap_entries.len(),
                        stream_entries.len(),
                        "Entry count mismatch for {:?}",
                        wal_path
                    );
                    for (m, s) in mmap_entries.iter().zip(stream_entries.iter()) {
                        assert_eq!(m.0, s.0, "seq_no mismatch");
                        assert_eq!(m.1, s.1, "entry mismatch");
                        assert_eq!(m.2, s.2, "end_pos mismatch");
                    }
                }
                (Err(e_mmap), Err(e_stream)) => {
                    assert!(
                        error_class_matches(&e_mmap, &e_stream),
                        "Error class mismatch between mmap ({:?}) and stream ({:?}) for {:?}",
                        e_mmap,
                        e_stream,
                        wal_path
                    );
                }
                (mmap_res, stream_res) => {
                    panic!(
                        "Parity failure: mmap_res={:?}, stream_res={:?} for path {:?}",
                        mmap_res, stream_res, wal_path
                    );
                }
            }
        }
        Err(_open_err) => {
            // Wal::open_with_key_manager rejected the corrupted WAL file during initial replay.
        }
    }
}

#[tokio::test]
async fn test_wal_replay_parity_clean() {
    let dir = tempdir().expect("tempdir");

    // Plaintext
    let path_plain = create_sample_wal(dir.path(), "clean_plain.wal", None).await;
    check_parity(&path_plain, None).await;

    // Encrypted
    let km = Arc::new(
        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890").expect("km"),
    );
    let path_enc = create_sample_wal(dir.path(), "clean_enc.wal", Some(km.clone())).await;
    check_parity(&path_enc, Some(km)).await;
}

#[tokio::test]
async fn test_wal_replay_parity_bitflips() {
    let dir = tempdir().expect("tempdir");
    let km = Arc::new(
        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890").expect("km"),
    );

    let path_enc = create_sample_wal(dir.path(), "bitflip_enc.wal", Some(km.clone())).await;
    let original_bytes = tokio::fs::read(&path_enc).await.expect("read");

    // Bitflip at every byte position
    for byte_offset in 0..original_bytes.len() {
        let mut corrupted = original_bytes.clone();
        corrupted[byte_offset] ^= 0x01;
        let mutated_path = dir.path().join(format!("mut_flip_{byte_offset}.wal"));
        tokio::fs::write(&mutated_path, &corrupted)
            .await
            .expect("write");

        check_parity(&mutated_path, Some(km.clone())).await;
        let _ = tokio::fs::remove_file(&mutated_path).await;
    }
}

#[tokio::test]
async fn test_wal_replay_parity_truncations() {
    let dir = tempdir().expect("tempdir");
    let km = Arc::new(
        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890").expect("km"),
    );

    let path_enc = create_sample_wal(dir.path(), "trunc_enc.wal", Some(km.clone())).await;
    let original_bytes = tokio::fs::read(&path_enc).await.expect("read");

    // Truncation at every byte length
    for trunc_len in 0..original_bytes.len() {
        let truncated = &original_bytes[..trunc_len];
        let mutated_path = dir.path().join(format!("mut_trunc_{trunc_len}.wal"));
        tokio::fs::write(&mutated_path, truncated)
            .await
            .expect("write");

        check_parity(&mutated_path, Some(km.clone())).await;
        let _ = tokio::fs::remove_file(&mutated_path).await;
    }
}

#[tokio::test]
async fn test_wal_replay_parity_garbage_tail() {
    let dir = tempdir().expect("tempdir");
    let km = Arc::new(
        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890").expect("km"),
    );

    let path_enc = create_sample_wal(dir.path(), "garbage_enc.wal", Some(km.clone())).await;
    let mut bytes = tokio::fs::read(&path_enc).await.expect("read");

    // Append garbage tail
    bytes.extend_from_slice(b"SOME_GARBAGE_TAIL_BYTES_1234567890");
    let mutated_path = dir.path().join("garbage_tail.wal");
    tokio::fs::write(&mutated_path, &bytes)
        .await
        .expect("write");

    check_parity(&mutated_path, Some(km)).await;
}
