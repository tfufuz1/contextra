use std::fs::File;
use std::io;

/// Map a file into memory read-only using `memmap2`.
///
/// # Safety / Invariants
/// The caller guarantees that `file` is an open read-only file handle.
/// The memory map remains valid as long as the underlying storage exists.
pub fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap> {
    // SAFETY:
    // 1. `file` is a valid open read-only file descriptor.
    // 2. Read-only mapping prevents data mutation races in Rust address space.
    // 3. Memory pages are managed safely by the kernel page tables.
    unsafe { memmap2::Mmap::map(file) }
}
