use contextra_durable_fs::{atomic_replace, durable_remove};
use tempfile::tempdir;

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
    let err = durable_remove(&missing_file).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);

    Ok(())
}
