// FILE-CONTEXT
// STAND: 2026-10-17
// ZWECK: Hardening-Tests für scrub_and_remove: Hardlink-Sicherheit, CoW-Erkennung und Receipt-Eigenschaften.
// INVARIANTEN: Zero-Panic, pure safe Rust, Hardlinks werden bei scrub_and_remove NIEMALS überschrieben.

use contextra_durable_fs::{
    scrub_and_remove, scrub_and_remove_with_injector, scrub_and_remove_with_mountinfo,
    scrub_if_safe_with_mountinfo, FaultInjector, NoFault, ScrubMethod, ScrubOutcome,
    ScrubPolicy, ScrubReceipt, ScrubRefusal,
};
use tempfile::TempDir;

#[test]
fn test_scrub_hardlink_preserves_second_link() -> std::io::Result<()> {
    let tmp_dir = TempDir::new()?;
    let original = tmp_dir.path().join("original.dat");
    let link = tmp_dir.path().join("hardlink.dat");

    let data = b"CRITICAL_DATA_DO_NOT_CORRUPT_1234567890";
    std::fs::write(&original, data)?;
    std::fs::hard_link(&original, &link)?;

    // original path has nlink == 2
    let receipt: ScrubReceipt = scrub_and_remove(&original)?;

    // Check that receipt indicates UnlinkOnly
    assert_eq!(receipt.method, ScrubMethod::UnlinkOnly);
    assert_eq!(receipt.bytes_scrubbed, 0);
    assert!(receipt.nlink > 1);

    // Original file path was removed (unlinked)
    assert!(!original.exists());

    // Hardlinked file MUST exist and be byte-identical to original content!
    assert!(link.exists());
    let remaining_data = std::fs::read(&link)?;
    assert_eq!(remaining_data, data);

    Ok(())
}

#[test]
fn test_scrub_normal_file_overwrites_and_removes() -> std::io::Result<()> {
    let tmp_dir = TempDir::new()?;
    let file_path = tmp_dir.path().join("normal.dat");

    let data = vec![0xABu8; 1024];
    std::fs::write(&file_path, &data)?;

    let receipt: ScrubReceipt = scrub_and_remove(&file_path)?;

    assert_eq!(receipt.method, ScrubMethod::OverwriteZeros);
    assert_eq!(receipt.bytes_scrubbed, 1024);
    assert_eq!(receipt.nlink, 1);
    assert!(!file_path.exists());

    Ok(())
}

#[test]
fn test_cow_detection_with_injected_mountinfo() -> std::io::Result<()> {
    let tmp_dir = TempDir::new()?;
    let file_path = tmp_dir.path().join("cow_test.dat");

    let data = b"COW_FILESYSTEM_TEST_DATA";
    std::fs::write(&file_path, data)?;

    // Mock mountinfo text indicating btrfs on /tmp or target dir
    let mount_dir = tmp_dir.path().to_string_lossy();
    let injected_mountinfo = format!(
        "28 21 0:25 / {} rw,relatime master:1 - btrfs /dev/sda1 rw,space_cache\n",
        mount_dir
    );

    // Test scrub_and_remove_with_mountinfo
    let receipt = scrub_and_remove_with_mountinfo(&file_path, &NoFault, Some(&injected_mountinfo))?;

    assert_eq!(receipt.method, ScrubMethod::UnlinkOnly);
    assert_eq!(receipt.bytes_scrubbed, 0);
    assert!(receipt.cow_fs);

    // Test scrub_if_safe_with_mountinfo on a fresh file
    let file_path_2 = tmp_dir.path().join("cow_test_2.dat");
    std::fs::write(&file_path_2, data)?;

    let outcome = scrub_if_safe_with_mountinfo(
        &file_path_2,
        ScrubPolicy::OverwriteZeros,
        false,
        false,
        &NoFault,
        Some(&injected_mountinfo),
    )?;

    assert_eq!(outcome, ScrubOutcome::Refused(ScrubRefusal::CopyOnWriteFilesystem));
    assert!(file_path_2.exists());

    Ok(())
}

struct FailingInjector;

impl FaultInjector for FailingInjector {
    fn on_write(&self, _path: Option<&std::path::Path>, _bytes: usize) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::WriteZero,
            "simulated write error",
        ))
    }
}

#[test]
fn test_scrub_fault_injector_failure_propagates() -> std::io::Result<()> {
    let tmp_dir = TempDir::new()?;
    let file_path = tmp_dir.path().join("fault.dat");

    let data = vec![0x33u8; 512];
    std::fs::write(&file_path, &data)?;

    let injector = FailingInjector;
    let res = scrub_and_remove_with_injector(&file_path, &injector);

    assert!(res.is_err());
    // File remains present because scrubbing failed before deletion
    assert!(file_path.exists());

    Ok(())
}
