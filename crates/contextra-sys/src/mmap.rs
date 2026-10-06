use std::fs::File;
use std::io;

/// Map a file into memory read-only using `memmap2`.
///
/// # Safety / Invariants
/// The caller guarantees that `file` is an open read-only file handle.
/// The memory map remains valid as long as the underlying storage exists.
pub fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap> {
    // TODO(Implementer): [P05 / F-03 / MEDIUM]
    // Virtuelle Speicher-Anfälligkeit (SIGBUS) bei externer Dateitrunkierung von mmap-Slices:
    // Externe Kürzung gemappter Dateien führt zu SIGBUS bei Seitenzugriffen außerhalb der neuen Dateigröße.
    // 1. Vertraglich dokumentieren: Das unterliegende Dateisystem darf nicht extern mutiert/getrunkt werden.
    // 2. Wo nötig, Advisory/Mandatory Locks (flock) auf File-Deskriptoren halten oder SIGBUS-Signalhandler/Stream-Reader-Fallbacks vorsehen.
    // SAFETY:
    // 1. `file` is a valid open read-only file descriptor.
    // 2. Read-only mapping prevents data mutation races in Rust address space.
    // 3. Memory pages are managed safely by the kernel page tables.
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
}
