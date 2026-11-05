//! I/O-Primitive und Vfs-Interaktion für den Löschkern (SPEC §8.12, D-04).
//!
//! Alle I/O-Operationen des Löschkerns laufen ausschließlich über dieses Modul
//! und den Port `Vfs` (D-04). Kein `std::fs`, kein `tokio::fs`.

use crate::error::ErasureError;
use contextra_ports::vfs::VfsFile;

/// Schreibt einen vollständigen Puffer an eine Dateiposition und liefert einen Fehler,
/// wenn weniger als `buf.len()` Bytes geschrieben wurden.
pub fn write_all_at(f: &dyn VfsFile, buf: &[u8], offset: u64) -> Result<(), ErasureError> {
    let mut written = 0usize;
    while written < buf.len() {
        let n = f
            .write_at(buf.get(written..).unwrap_or(&[]), offset + written as u64)
            .map_err(ErasureError::Io)?;
        if n == 0 {
            return Err(ErasureError::Io(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "write_at returned 0",
            )));
        }
        written += n;
    }
    Ok(())
}

/// Liest einen vollständigen Puffer von einer Dateiposition.
pub fn read_exact_at(f: &dyn VfsFile, buf: &mut [u8], offset: u64) -> Result<(), ErasureError> {
    let mut read = 0usize;
    while read < buf.len() {
        let n = f
            .read_at(buf.get_mut(read..).unwrap_or(&mut []), offset + read as u64)
            .map_err(ErasureError::Io)?;
        if n == 0 {
            return Err(ErasureError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "read_at returned 0",
            )));
        }
        read += n;
    }
    Ok(())
}

/// Puffergröße für Nullen-Schreibschleife ohne dynamische Dauer-Allokation (SPEC §8.12, §11.5).
const ZERO_BUFFER_CHUNK_SIZE: usize = 4096;
/// Statischer Null-Puffer zur Vermeidung wiederholter Heap-Allokationen bei Slot- und Block-Zeroizing.
const ZERO_BLOCK: [u8; ZERO_BUFFER_CHUNK_SIZE] = [0u8; ZERO_BUFFER_CHUNK_SIZE];

/// Schreibt Nullen an eine Dateiposition ohne Heap-Allokation in einer Block-Schleife.
///
/// Alle I/O-Fehler sind typisiert und enthalten keine Dateipfade oder Kennungen (P13).
pub fn zero_at(f: &dyn VfsFile, offset: u64, len: usize) -> Result<(), ErasureError> {
    let mut written = 0usize;
    while written < len {
        let remaining = len - written;
        let chunk_size = remaining.min(ZERO_BUFFER_CHUNK_SIZE);
        let slice = ZERO_BLOCK.get(..chunk_size).unwrap_or(&[]);
        write_all_at(f, slice, offset + written as u64)?;
        written += chunk_size;
    }
    Ok(())
}
