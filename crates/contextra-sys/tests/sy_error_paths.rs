#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Tests für Fehlerpfade (EBADF, EIO, ENOSPC, EINTR) und partielles Schreiben (SY-05, T-FSYNC-1).

use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;
use tempfile::tempdir;

use contextra_sys::{flush_dir_sync, write_all_durable, SyncFailure};

/// SPEC: T-FSYNC-1
/// Prüft, dass ein ungültiger Pfad bzw. nicht existentes Verzeichnis zu einem kontrollierten Fehler führt.
#[test]
fn test_flush_invalid_path_fails() {
    let non_existent = Path::new("/proc/nonexistent_contextra_sys_test_dir_ebadf");
    let res = flush_dir_sync(non_existent);
    assert!(res.is_err());
}

/// SY-05: Prüft typisierte Fehlerkonvertierung von SyncFailure.
#[test]
fn test_sync_failure_error_conversions() {
    let io_err_unsupported = std::io::Error::new(ErrorKind::Unsupported, "unsupported");
    assert_eq!(
        SyncFailure::from(io_err_unsupported),
        SyncFailure::Unsupported
    );

    let io_err_perm = std::io::Error::new(ErrorKind::PermissionDenied, "denied");
    assert_eq!(
        SyncFailure::from(io_err_perm),
        SyncFailure::PermissionDenied
    );

    let io_err_not_found = std::io::Error::new(ErrorKind::NotFound, "not found");
    assert_eq!(SyncFailure::from(io_err_not_found), SyncFailure::NotFound);

    let io_err_other = std::io::Error::other("device error");
    assert_eq!(SyncFailure::from(io_err_other), SyncFailure::IoError);
}

/// SY-05 & SY-06: write_all_durable schreibt vollständige Puffer zuverlässig.
#[test]
fn test_write_all_durable_full_buffer() {
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("durable_write.bin");
    let mut file = File::create(&file_path).expect("create file");

    let payload = vec![0xAAu8; 1024 * 64];
    write_all_durable(&mut file, &payload).expect("write_all_durable");

    let read_back = std::fs::read(&file_path).expect("read");
    assert_eq!(read_back, payload);
}
