//! # Testsuite: basic.rs
//!
//! Zweck: Basistests für atomares Ersetzen (`atomic_replace`) und dauerhaftes Löschen (`durable_remove`).

use contextra_durable_fs::{atomic_replace, durable_remove};
use tempfile::tempdir;

// ANCHOR[contextra-durable-fs:TEST-BASIC-ATOMIC-REPLACE]
/// Prüft, dass `atomic_replace` neue Dateien anlegt und bestehende Dateien atomar überschreibt.
#[test]
fn test_atomic_replace_writes_and_overwrites() -> std::io::Result<()> {
    let dir = tempdir()?;
    let target_file = dir.path().join("data.txt");

    // First write
    atomic_replace(&target_file, b"initial content")?;
    let content = std::fs::read(&target_file)?;
    assert_eq!(content, b"initial content");

    // Overwrite
    atomic_replace(&target_file, b"updated content")?;
    let updated = std::fs::read(&target_file)?;
    assert_eq!(updated, b"updated content");

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-BASIC-DURABLE-REMOVE]
/// Prüft, dass `durable_remove` existierende Dateien dauerhaft entfernt und bei fehlenden Pfaden NotFound meldet.
#[test]
fn test_durable_remove_success_and_not_found() -> std::io::Result<()> {
    let dir = tempdir()?;
    let target_file = dir.path().join("to_delete.txt");

    atomic_replace(&target_file, b"delete me")?;
    assert!(target_file.exists());

    // Durable remove existing file
    durable_remove(&target_file)?;
    assert!(!target_file.exists());

    // Durable remove missing file returns NotFound error
    let missing_file = dir.path().join("missing.txt");
    assert!(matches!(
        durable_remove(&missing_file),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound
    ));

    Ok(())
}
