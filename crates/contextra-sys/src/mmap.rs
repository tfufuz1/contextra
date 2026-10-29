// FILE-CONTEXT
// STAND: 2026-10-28T00:00:00Z
// ZWECK: Read-only memory mapping facade over memmap2 (Ring 0 / SPEC 7.7)
// INVARIANTEN: Zero panic; typed io::Error propagation; read-only mapping
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7

//! Read-only memory mapping facade over `memmap2`.

use std::fs::File;
use std::io;

/// Map a file into memory read-only using `memmap2`.
///
/// # Errors
/// Returns an [`std::io::Error`] if memory mapping fails.
///
/// # Safety / Invariants
/// The caller guarantees that `file` is an open read-only file handle.
/// The memory map remains valid as long as the underlying storage exists.
pub fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap> {
    // SAFETY: `file` is a valid open read-only file descriptor; read-only mapping prevents data mutation races in Rust address space and memory pages are managed safely by kernel page tables.
    unsafe { memmap2::Mmap::map(file) }
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
    fn test_mmap_readonly_empty_file() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "contextra_sys_mmap_empty_{}.bin",
            std::process::id()
        ));
        {
            File::create(&path)?;
        }
        let file = File::open(&path)?;
        let mmap = mmap_readonly(&file)?;
        assert!(mmap.is_empty());
        let _ = std::fs::remove_file(path);
        Ok(())
    }

    #[test]
    fn test_mmap_readonly_large_file() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "contextra_sys_mmap_large_{}.bin",
            std::process::id()
        ));
        let payload = vec![0xABu8; 1024 * 1024]; // 1 MiB
        {
            let mut file = File::create(&path)?;
            file.write_all(&payload)?;
        }
        let file = File::open(&path)?;
        let mmap = mmap_readonly(&file)?;
        assert_eq!(mmap.len(), 1024 * 1024);
        assert_eq!(&mmap[..], &payload[..]);
        let _ = std::fs::remove_file(path);
        Ok(())
    }

    #[test]
    fn test_mmap_readonly_access_boundary() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "contextra_sys_mmap_bound_{}.bin",
            std::process::id()
        ));
        {
            let mut file = File::create(&path)?;
            file.write_all(b"short payload")?;
        }
        let file = File::open(&path)?;
        let mmap = mmap_readonly(&file)?;
        assert_eq!(mmap.get(..13), Some(&b"short payload"[..]));
        assert_eq!(mmap.get(13), None);
        let _ = std::fs::remove_file(path);
        Ok(())
    }
}
