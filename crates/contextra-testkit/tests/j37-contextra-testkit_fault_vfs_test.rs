use contextra_testkit::{FaultConfig, FaultVfs};
use std::io::ErrorKind;
use std::path::Path;

#[test]
fn test_j37_fault_vfs_lifecycle_integration() {
    let vfs = FaultVfs::new();

    // Configure fault injection via set_config
    vfs.set_config(FaultConfig {
        fail_writes_after: Some(2),
        ..Default::default()
    });

    let path1 = Path::new("file1.dat");
    let path2 = Path::new("file2.dat");
    let path3 = Path::new("file3.dat");

    // write_file and read_file verification
    assert!(vfs.write_file(path1, b"payload1").is_ok());
    assert!(vfs.write_file(path2, b"payload2").is_ok());

    let data1 = vfs.read_file(path1).expect("read file1 should succeed");
    assert_eq!(data1, b"payload1");

    let data2 = vfs.read_file(path2).expect("read file2 should succeed");
    assert_eq!(data2, b"payload2");

    // Exceeding write limit triggers WriteZero error
    let err = vfs.write_file(path3, b"payload3").unwrap_err();
    assert_eq!(err.kind(), ErrorKind::WriteZero);

    // Trigger simulated crash
    vfs.trigger_crash();
    let crash_err = vfs.write_file(path1, b"overwrite").unwrap_err();
    assert_eq!(crash_err.kind(), ErrorKind::BrokenPipe);

    let crash_read_err = vfs.read_file(path1).unwrap_err();
    assert_eq!(crash_read_err.kind(), ErrorKind::BrokenPipe);

    // Reset counters and recover VFS
    vfs.reset_counters();
    vfs.set_config(FaultConfig::default());

    assert!(vfs.write_file(path3, b"payload3_recovered").is_ok());
    let data3 = vfs.read_file(path3).expect("read file3 should succeed after reset");
    assert_eq!(data3, b"payload3_recovered");
}
