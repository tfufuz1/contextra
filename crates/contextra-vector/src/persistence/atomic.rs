// FILE-CONTEXT
// ZWECK: Atomares Schreiben von Persistenz-Dateien (temp -> sync_data -> rename -> dir sync).
// INVARIANTEN: INV-FSYNC-1 (Fail-closed durability ohne stille Retries).

use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

/// Writes data to `destination` atomically via a temporary file in the same parent directory,
/// ensuring both file data and parent directory entry are durably synchronized to disk.
pub fn write_atomic(destination: &Path, data: &[u8]) -> io::Result<()> {
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    let file_name = destination
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("index.hnsw");
    let pid = std::process::id();
    let tmp_path = parent.join(format!(".{file_name}.{pid}.tmp"));

    // 1. Create and write to temporary file
    {
        let mut tmp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp_path)?;

        tmp_file.write_all(data)?;
        tmp_file.sync_data()?;
    }

    // 2. Atomic rename
    if let Err(e) = std::fs::rename(&tmp_path, destination) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(e);
    }

    // 3. Sync parent directory
    if let Ok(dir) = File::open(parent) {
        let _ = dir.sync_all();
    }

    Ok(())
}
