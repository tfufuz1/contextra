// FILE-CONTEXT
// STAND: 2026-10-17
// ZWECK: Hardening-Tests für scrub_and_remove: Hardlink-Sicherheit, CoW-Erkennung und Receipt-Eigenschaften.
// INVARIANTEN: Zero-Panic, pure safe Rust, Hardlinks werden bei scrub_and_remove NIEMALS überschrieben.

use contextra_durable_fs::{
    scrub_and_remove, scrub_and_remove_with_injector, FaultInjector, ScrubMethod, ScrubReceipt,
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
