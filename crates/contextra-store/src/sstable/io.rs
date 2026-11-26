//! # SSTable I/O-Routinen (Ring 1)
//!
//! Kapselt Low-Level Datei- und Lesevorgänge über [`contextra_ports::VfsFile`]
//! innerhalb von Block-Level I/O-Grenzen.

use contextra_ports::VfsFile;

pub(super) fn pread_exact(
    file: &dyn VfsFile,
    mut buf: &mut [u8],
    mut offset: u64,
) -> std::io::Result<()> {
    while !buf.is_empty() {
        let n = file.read_at(buf, offset)?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "failed to fill whole buffer",
            ));
        }
        buf = &mut buf[n..];
        offset += n as u64;
    }
    Ok(())
}
