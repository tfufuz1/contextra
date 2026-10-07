use std::fs::File;
use std::io;

/// Map a file into memory read-only using `memmap2`.
///
/// # Safety / Invariants
/// - The caller guarantees that `file` is an open read-only file handle.
/// - The file length is verified against `file.metadata()?.len()` prior to mapping.
/// - The underlying file length must remain constant and must not be truncated or concurrently modified externally.
/// - Memory mapping relies on kernel page management; pages remain valid as long as the file descriptor exists without external truncation.
pub fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap> {
    let metadata = file.metadata()?;
    let file_len = metadata.len();
    if file_len == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Cannot mmap empty file (0 bytes)",
        ));
    }

    // SAFETY:
    // 1. `file` is a valid, open read-only file descriptor with verified non-zero length (`file_len`).
    // 2. The file length was pre-verified via `file.metadata()?.len()`.
    // 3. Read-only mapping prevents data mutation races within Rust address space.
    // 4. External truncation or concurrent file modification is contractually prohibited.
    // 5. Page allocation and virtual memory mapping are safely managed by OS page tables.
    let mmap = unsafe { memmap2::Mmap::map(file)? };

    if mmap.len() as u64 != file_len {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Mapped memory length mismatch with file metadata length",
        ));
    }

    Ok(mmap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_mmap_readonly() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "contextra_sys_mmap_test_{}.bin",
            std::process::id()
        ));
        {
            let mut file = File::create(&path)?;
            file.write_all(b"contextra mmap readonly test payload")?;
        }
        let file = File::open(&path)?;
        let mmap = mmap_readonly(&file)?;
        assert_eq!(&mmap[..], b"contextra mmap readonly test payload");
        let _ = std::fs::remove_file(path);
        Ok(())
    }

    #[test]
    fn test_mmap_readonly_empty_file_returns_error() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "contextra_sys_mmap_empty_{}.bin",
            std::process::id()
        ));
        {
            let _file = File::create(&path)?;
        }
        let file = File::open(&path)?;
        let res = mmap_readonly(&file);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), io::ErrorKind::InvalidInput);
        let _ = std::fs::remove_file(path);
        Ok(())
    }
}
