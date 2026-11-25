#![allow(clippy::expect_used, clippy::unwrap_used)]
//! # Testsuite: basic.rs
//!
//! Zweck: Basistests für atomares Ersetzen (`atomic_replace`).

use contextra_durable_fs::atomic_replace;
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
