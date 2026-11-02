//! I/O-Primitive und Vfs-Interaktion für den Löschkern (SPEC §8.12, D-04).
//!
//! Alle I/O-Operationen des Löschkerns laufen ausschließlich über dieses Modul
//! und den Port `Vfs` (D-04). Kein `std::fs`, kein `tokio::fs`.

use crate::error::ErasureError;
use contextra_ports::vfs::VfsFile;

/// Schreibt einen vollständigen Puffer an eine Dateiposition und liefert einen Fehler,
/// wenn weniger als `buf.len()` Bytes geschrieben wurden.
pub fn write_all_at(
    f: &dyn VfsFile,
    buf: &[u8],
    offset: u64,
) -> Result<(), ErasureError> {
    let mut written = 0usize;
    while written < buf.len() {
        let n = f
            .write_at(
                buf.get(written..).unwrap_or(&[]),
                offset + written as u64,
            )
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
pub fn read_exact_at(
    f: &dyn VfsFile,
    buf: &mut [u8],
    offset: u64,
) -> Result<(), ErasureError> {
    let mut read = 0usize;
    while read < buf.len() {
        let n = f
            .read_at(
                buf.get_mut(read..).unwrap_or(&mut []),
                offset + read as u64,
            )
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

/// Schreibt Nullen an eine Dateiposition.
pub fn zero_at(f: &dyn VfsFile, offset: u64, len: usize) -> Result<(), ErasureError> {
    let zeros = vec![0u8; len];
    write_all_at(f, &zeros, offset)
}
