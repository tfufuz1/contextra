//! # Testsuite: scrub_and_remove.rs
//!
//! Zweck: Tests für das Überschreiben mit Nullen und physische Entfernen (`scrub_and_remove`).

use contextra_durable_fs::{scrub_and_remove, ScrubReceipt};
use std::fs::{self, File};
use tempfile::tempdir;

// ANCHOR[contextra-durable-fs:TEST-SCRUB-HARDLINK-OVERWRITE]
/// Prüft das Überschreiben und Entfernen von Dateien bei einfachen Hardlinks.
#[test]
fn test_scrub_and_remove_hardlink_overwrite() -> std::io::Result<()> {
    let dir = tempdir()?;
    let original = dir.path().join("original.txt");
    let hardlink = dir.path().join("hardlink.txt");

    let secret_data = b"super_secret_payload_that_must_be_scrubbed_with_zeros!";
    fs::write(&original, secret_data)?;
    fs::hard_link(&original, &hardlink)?;

    let receipt: ScrubReceipt = scrub_and_remove(&original)?;
    assert_eq!(receipt.bytes_scrubbed, secret_data.len() as u64);

    // Original file must be removed
    assert!(!original.exists());

    // Hardlink file must exist and contain only zero bytes
    let hardlink_data = fs::read(&hardlink)?;
    assert_eq!(hardlink_data.len(), secret_data.len());
    assert!(
        hardlink_data.iter().all(|&b| b == 0),
        "Hardlink data should contain only zeros"
    );

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-SCRUB-LARGE-UNALIGNED]
/// Prüft das Scrubbing großer, nicht block-ausgerichteter Dateien.
#[test]
fn test_scrub_and_remove_large_unaligned_file() -> std::io::Result<()> {
    let dir = tempdir()?;
    let file_path = dir.path().join("large.bin");

    let file_size = 70_000usize;
    let data = vec![0xABu8; file_size];
    fs::write(&file_path, &data)?;

    let receipt = scrub_and_remove(&file_path)?;
    assert_eq!(receipt.bytes_scrubbed, file_size as u64);

    assert!(!file_path.exists());
    Ok(())
}

#[cfg(unix)]
// ANCHOR[contextra-durable-fs:TEST-SCRUB-SYMLINK-REJECTED]
/// Prüft, dass Symlinks beim Scrubbing sicher abgewiesen werden.
#[test]
fn test_scrub_and_remove_symlink_rejected() -> std::io::Result<()> {
    let dir = tempdir()?;
    let target = dir.path().join("target.txt");
    let symlink = dir.path().join("symlink.txt");

    let initial_data = b"target_file_content";
    fs::write(&target, initial_data)?;
    std::os::unix::fs::symlink(&target, &symlink)?;

    let res = scrub_and_remove(&symlink);
    assert!(matches!(
        res,
        Err(ref e) if e.kind() == std::io::ErrorKind::InvalidInput
    ));

    // Both target and symlink must exist and target content unchanged
    assert!(symlink.symlink_metadata().is_ok());
    assert_eq!(fs::read(&target)?, initial_data);

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-SCRUB-NOT-FOUND]
/// Prüft die Fehlerbehandlung bei nicht existierenden Pfaden.
#[test]
fn test_scrub_and_remove_not_found() -> std::io::Result<()> {
    let dir = tempdir()?;
    let missing = dir.path().join("does_not_exist.txt");

    let res = scrub_and_remove(&missing);
    assert!(matches!(
        res,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound
    ));

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-SCRUB-EMPTY-FILE]
/// Prüft das Bereinigen und Entfernen leerer Dateien (0 Bytes).
#[test]
fn test_scrub_and_remove_empty_file() -> std::io::Result<()> {
    let dir = tempdir()?;
    let empty_file = dir.path().join("empty.txt");
    File::create(&empty_file)?;

    let receipt = scrub_and_remove(&empty_file)?;
    assert_eq!(receipt.bytes_scrubbed, 0);

    assert!(!empty_file.exists());
    Ok(())
}
