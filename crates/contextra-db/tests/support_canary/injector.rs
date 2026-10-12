use std::fs::{self, OpenOptions};
use std::io::{Result as IoResult, Write};
use std::path::Path;

/// Injects a byte needle into a target file at the end or at a specific offset.
#[allow(dead_code)]
pub fn inject_needle_into_file(file_path: &Path, needle_bytes: &[u8]) -> IoResult<()> {
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(file_path)?;

    file.write_all(needle_bytes)?;
    file.flush()?;
    Ok(())
}
