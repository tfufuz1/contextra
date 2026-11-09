//! Beispiel für crash-sichere Dateioperationen mit `contextra-durable-fs`.

use contextra_durable_fs::vfs::{StdVfs, Vfs};
use contextra_durable_fs::{atomic_replace, DirLock, LockMode};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = Path::new("/tmp/my_storage");
    std::fs::create_dir_all(dir)?;

    // 1. Verzeichnissperre erwerben
    let _lock = DirLock::acquire(dir, LockMode::Normal)?;

    // 2. Atomares Ersetzen mit fsync und Directory-Sync
    let target = dir.join("data.bin");
    atomic_replace(&target, b"durable-content")?;

    // 3. Zugriff über den synchronen Vfs-Port
    let vfs = StdVfs;
    let data = vfs.read(&target)?;
    assert_eq!(data, b"durable-content");

    println!("Erfolgreich ausgeführt: {:?}", target);

    Ok(())
}
