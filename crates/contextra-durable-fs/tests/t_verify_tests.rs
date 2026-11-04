//! Tests für D-113 Verifikation am Medium auf RealVfs
//! SPEC: T-VERIFY-1, T-VERIFY-2

use contextra_durable_fs::RealVfs;
use contextra_ports::vfs::{Vfs, VfsOpenOptions};
use tempfile::tempdir;

/// SPEC: T-VERIFY-1
#[test]
fn test_real_vfs_read_verified() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let vfs = RealVfs::new();
    let file_path = dir.path().join("verify_real.dat");

    let file = vfs.open(
        &file_path,
        &VfsOpenOptions {
            read: true,
            write: true,
            create: true,
            ..Default::default()
        },
    )?;

    file.write_at(b"HELLO_VERIFIED", 0)?;
    file.sync_data()?;

    let (bytes, via) = file.read_verified(0, 14)?;
    assert_eq!(&bytes[..], b"HELLO_VERIFIED");
    assert!(matches!(
        via,
        contextra_ports::vfs::VerifiedVia::Direct
            | contextra_ports::vfs::VerifiedVia::CacheDropped
            | contextra_ports::vfs::VerifiedVia::Cached
    ));
    Ok(())
}

/// SPEC: T-VERIFY-2
#[test]
fn test_real_vfs_read_at_and_list_and_unlink() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let vfs = RealVfs::new();
    let file_path = dir.path().join("sample.dat");

    vfs.write_atomic(&file_path, b"TEST_PAYLOAD")?;

    let mut buf = [0u8; 12];
    let n = vfs.read_at(&file_path, 0, &mut buf)?;
    assert_eq!(n, 12);
    assert_eq!(&buf, b"TEST_PAYLOAD");

    let list = vfs.list(dir.path())?;
    assert!(list.contains(&"sample.dat".to_string()));

    vfs.unlink(&file_path)?;
    assert!(!vfs.exists(&file_path)?);
    Ok(())
}
