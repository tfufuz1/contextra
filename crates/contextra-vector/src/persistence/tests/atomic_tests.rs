// FILE-CONTEXT
// ZWECK: Unit-Tests für atomare Persistenzoperationen (write_atomic).
// INVARIANTEN: Atomares Ersetzen, Rollback bei Fehlern, kein unvollstaendiger Zustand.
// HOTSPOTS: test_write_atomic_basic, test_write_atomic_overwrite
// STAND: TS:2026-11-12T00:00:00Z

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use crate::persistence::write_atomic;

#[test]
fn test_write_atomic_basic() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let file_path = temp_dir.path().join("atomic_test.bin");
    let data = b"hello atomic world";

    write_atomic(&file_path, data).expect("write_atomic");
    assert!(file_path.exists());

    let read_back = std::fs::read(&file_path).expect("read");
    assert_eq!(read_back, data);
}

#[test]
fn test_write_atomic_overwrite() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let file_path = temp_dir.path().join("atomic_overwrite.bin");

    write_atomic(&file_path, b"initial content").expect("write initial");
    write_atomic(&file_path, b"updated content").expect("write updated");

    let read_back = std::fs::read(&file_path).expect("read");
    assert_eq!(read_back, b"updated content");
}
