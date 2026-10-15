// FILE-CONTEXT: Chaos test for fsync/I/O error propagation and durability guarantees during WAL write drops. (TS: 2026-08-30) (SESSION: 283abf0f)
//! Chaos test proving fsync/I/O error propagation and non-corruption invariants in `LsmStorage`.
//!
//! Evaluates the "fsync Error Propagation (ABSOLUT)" invariant defined in `crates/contextra-store/AGENTS.md`.

use contextra_core::{ContextraError, StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::fs::Permissions;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

/// Verifies that an I/O failure during `commit()` properly propagates an error,
/// leaves `last_committed_tx` unchanged, allows recovery once write permissions are restored,
/// and preserves previously committed entries without collateral damage.
///
/// # Platform Restriction
/// Active file descriptor manipulation via `/proc/self/fd` and file mode permissions (`0o444`)
/// is target-scoped to Linux (`#[cfg(target_os = "linux")]`). Under POSIX rules, file permissions
/// are evaluated at `open()` time; simulating a mid-flight write/fsync drop on an open file description
/// requires replacing the descriptor via `/proc/self/fd`.
#[tokio::test]
#[cfg(target_os = "linux")]
async fn test_chaos_dropped_write_error_propagation_and_recovery() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().to_path_buf();

    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };

    let storage = LsmStorage::new(config).await.unwrap();

    // 1. Commit initial entry successfully
    let tx1 = TxId::new(1);
    storage.put(tx1, b"key1", b"val1").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let last_tx_1 = storage.last_tx_id().await.unwrap();
    assert_eq!(
        last_tx_1, tx1,
        "last_committed_tx must be updated after initial successful commit"
    );

    // Verify key1 is readable
    let val1 = storage.get(b"key1").await.unwrap();
    assert_eq!(
        val1,
        Some(bytes::Bytes::from_static(b"val1")),
        "key1 must be readable before error injection"
    );

    // 2. Set active WAL file permissions to read-only (0o444)
    // and replace open file description for `wal.log` with a read-only descriptor.
    let wal_path = db_path.join("wal.log");
    std::fs::set_permissions(&wal_path, Permissions::from_mode(0o444)).unwrap();

    // Find open file descriptors pointing to wal_path in /proc/self/fd and replace them with O_RDONLY
    if let Ok(entries) = std::fs::read_dir("/proc/self/fd") {
        for entry in entries.flatten() {
            if let Ok(target) = std::fs::read_link(entry.path()) {
                if target == wal_path {
                    if let Ok(fd_num) = entry.file_name().to_string_lossy().parse::<i32>() {
                        let _ = contextra_sys::reopen_and_dup2(&wal_path, fd_num, 0);
                        // O_RDONLY = 0
                    }
                }
            }
        }
    }

    // 3. Attempt a second commit (tx2).
    // Expectation: Err(ContextraError::Storage(_)) or Err(ContextraError::Io(_))
    let tx2 = TxId::new(2);
    storage.put(tx2, b"key2", b"val2").await.unwrap();

    let commit_result = storage.commit(tx2).await;

    // Verify that commit failed with the expected error variant WITHOUT calling .unwrap()
    match &commit_result {
        Err(ContextraError::Storage(msg)) => {
            assert!(
                msg.contains("WAL batch")
                    || msg.contains("Bad file descriptor")
                    || msg.contains("Permission denied")
                    || msg.contains("Commit failed"),
                "Expected I/O or WAL error in ContextraError::Storage, got: {msg}"
            );
        }
        Err(ContextraError::Io(io_err)) => {
            assert!(
                io_err.kind() == std::io::ErrorKind::PermissionDenied
                    || io_err.kind() == std::io::ErrorKind::Other
                    || io_err.raw_os_error() == Some(9), // EBADF
                "Expected EBADF or PermissionDenied I/O error, got: {:?}",
                io_err
            );
        }
        Err(other) => {
            panic!(
                "Expected ContextraError::Storage or ContextraError::Io on write failure, got: {:?}",
                other
            );
        }
        Ok(()) => {
            panic!(
                "Mutation survival check: commit() MUST NOT succeed when WAL file is read-only!"
            );
        }
    }

    // 4. Restore write permissions to file (0o644)
    std::fs::set_permissions(&wal_path, Permissions::from_mode(0o644)).unwrap();

    // Re-open WAL file handle for wal_path in /proc/self/fd with O_RDWR | O_APPEND
    if let Ok(entries) = std::fs::read_dir("/proc/self/fd") {
        for entry in entries.flatten() {
            if let Ok(target) = std::fs::read_link(entry.path()) {
                if target == wal_path {
                    if let Ok(fd_num) = entry.file_name().to_string_lossy().parse::<i32>() {
                        let _ = contextra_sys::reopen_and_dup2(&wal_path, fd_num, 2 | 1024);
                        // O_RDWR | O_APPEND
                    }
                }
            }
        }
    }

    // 5a. Assert: The failed commit MUST leave last_committed_tx unchanged
    let last_tx_after_failure = storage.last_tx_id().await.unwrap();
    assert_eq!(
        last_tx_after_failure, tx1,
        "last_committed_tx must remain unchanged at tx1 after failed commit attempt"
    );

    // 5b. Assert: Re-committing the same content after restoring write permissions must succeed
    storage.put(tx2, b"key2", b"val2").await.unwrap();
    storage
        .commit(tx2)
        .await
        .expect("Re-committing tx2 after restoring write permissions must succeed");

    let last_tx_final = storage.last_tx_id().await.unwrap();
    assert_eq!(
        last_tx_final, tx2,
        "last_committed_tx must advance to tx2 after successful re-commit"
    );

    // 5c. Assert: All previously committed entries remain correctly readable without collateral damage
    let val1_check = storage.get(b"key1").await.unwrap();
    assert_eq!(
        val1_check,
        Some(bytes::Bytes::from_static(b"val1")),
        "Initial entry key1 must remain intact after recovery"
    );

    let val2_check = storage.get(b"key2").await.unwrap();
    assert_eq!(
        val2_check,
        Some(bytes::Bytes::from_static(b"val2")),
        "Re-committed entry key2 must be correctly readable"
    );
}
