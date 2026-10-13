// FILE-CONTEXT
// STAND: 2026-09-19T20:12:00Z (SESSION: 01c5be8b)
// ZWECK: Lifecycle integration tests for FaultVfs.

#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use contextra_testkit::{FaultConfig, FaultVfs};
use std::io::ErrorKind;
use std::path::Path;

#[test]
fn test_j37_fault_vfs_lifecycle_integration() {
    let vfs = FaultVfs::new();
    vfs.set_config(FaultConfig {
        fail_writes_after: Some(2),
        fail_syncs_after: Some(1),
        fail_reads_after: None,
        fail_all: false,
    });

    let path1 = Path::new("j37_file1.txt");
    let path2 = Path::new("j37_file2.txt");
    let path3 = Path::new("j37_file3.txt");

    assert!(vfs.write_file(path1, b"hello").is_ok());
    assert!(vfs.write_file(path2, b"world").is_ok());

    let data1 = vfs.read_file(path1).expect("read file1 should succeed");
    assert_eq!(data1, b"hello");

    let data2 = vfs.read_file(path2).expect("read file2 should succeed");
    assert_eq!(data2, b"world");

    // Write 3 must fail due to limit 2
    let err = vfs.write_file(path3, b"payload3").unwrap_err();
    assert_eq!(err.kind(), ErrorKind::WriteZero);

    // Crash trigger test
    vfs.trigger_crash();
    let crash_err = vfs.write_file(path1, b"overwrite").unwrap_err();
    assert_eq!(crash_err.kind(), ErrorKind::BrokenPipe);

    let crash_read_err = vfs.read_file(path1).unwrap_err();
    assert_eq!(crash_read_err.kind(), ErrorKind::BrokenPipe);

    // Reset counters and verify write/read recovery
    vfs.reset_counters();
    assert!(vfs.write_file(path3, b"payload3_ok").is_ok());

    let data3 = vfs
        .read_file(path3)
        .expect("read file3 should succeed after reset");
    assert_eq!(data3, b"payload3_ok");
}
