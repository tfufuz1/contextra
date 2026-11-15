//! `DurableVfs`: dauerhafter Dateizugriff auf Basis von `std::fs` (Spec 3.7, WP-5 bis WP-7).
//!
//! Dieses Modul ist die einzige Stelle mit `std::fs` im Crate. Plattform: Linux
//! (Spec 1.2). Das Modul ist nur auf Unix-Systemen verfügbar.

use crate::error::FsyncFailed;
use crate::vfs::{VFile, Vfs};
use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind};
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// Ausrichtung der `O_DIRECT`-Puffer und Blockgröße (Spec 3.7: „ausgerichtet auf 4096“).
pub const DIRECT_ALIGN: usize = 4096;

/// Obergrenze der Nullpuffer beim Überschreiben (Scrub).
const SCRUB_CHUNK: usize = 8192;

/// Dauerhafter Dateizugriff ohne Zustand. Jede Operation wirkt direkt auf das Dateisystem.
#[derive(Debug, Default, Clone, Copy)]
pub struct DurableVfs;

impl Vfs for DurableVfs {
    fn create_new(&self, p: &Path) -> io::Result<Box<dyn VFile>> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(p)?;
        Ok(Box::new(DurableFile::new(file, false)))
    }

    fn open_read(&self, p: &Path) -> io::Result<Box<dyn VFile>> {
        let file = File::open(p)?;
        Ok(Box::new(DurableFile::new(file, false)))
    }

    fn open_direct_read(&self, p: &Path) -> io::Result<Box<dyn VFile>> {
        let flag = direct_flag().ok_or_else(|| {
            io::Error::new(
                ErrorKind::Unsupported,
                "O_DIRECT value unknown for this architecture",
            )
        })?;
        let file = OpenOptions::new().read(true).custom_flags(flag).open(p)?;
        Ok(Box::new(DurableFile::new(file, true)))
    }

    fn rename_atomic(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)?;
        sync_parent_dir(to)?;
        if parent_of(from) != parent_of(to) {
            sync_parent_dir(from)?;
        }
        Ok(())
    }

    fn remove_scrub(&self, p: &Path) -> io::Result<()> {
        scrub_file(p)?;
        fs::remove_file(p)
    }

    fn list(&self, dir: &Path) -> io::Result<Vec<PathBuf>> {
        let mut out = Vec::new();
        for entry in fs::read_dir(dir)? {
            out.push(entry?.path());
        }
        out.sort();
        Ok(out)
    }
}

/// Überschreibt den gesamten Inhalt mit Nullen und synchronisiert ihn (WP-6).
///
/// # Errors
/// `io::Error` bei Öffnen, Schreiben oder Synchronisieren.
pub fn scrub_file(p: &Path) -> io::Result<()> {
    let file = OpenOptions::new().write(true).open(p)?;
    let len = file.metadata()?.len();
    let zeros = [0_u8; SCRUB_CHUNK];
    let mut off = 0_u64;
    while off < len {
        let remaining = len - off;
        let n = usize::try_from(remaining.min(SCRUB_CHUNK as u64))
            .map_err(|_| invalid("scrub chunk size"))?;
        let chunk = zeros.get(..n).ok_or_else(|| invalid("scrub chunk slice"))?;
        write_all(&file, off, chunk)?;
        off += n as u64;
    }
    file.sync_all()
}

/// Eine geöffnete Datei. `failed` hält den Fsync-Zustand (Spec 3.7, WP-7).
struct DurableFile {
    file: File,
    direct: bool,
    failed: AtomicBool,
}

impl DurableFile {
    fn new(file: File, direct: bool) -> Self {
        Self {
            file,
            direct,
            failed: AtomicBool::new(false),
        }
    }
}

impl VFile for DurableFile {
    fn write_all_at(&self, off: u64, buf: &[u8]) -> io::Result<()> {
        if self.failed.load(Ordering::SeqCst) {
            return Err(io::Error::other(
                "file refused: an earlier fsync failed (WP-7)",
            ));
        }
        write_all(&self.file, off, buf)
    }

    fn read_exact_at(&self, off: u64, buf: &mut [u8]) -> io::Result<()> {
        if self.direct {
            direct_read(&self.file, off, buf)
        } else {
            self.file.read_exact_at(buf, off)
        }
    }

    fn sync_data(&self) -> Result<(), FsyncFailed> {
        if self.failed.load(Ordering::SeqCst) {
            return Err(FsyncFailed);
        }
        match self.file.sync_data() {
            Ok(()) => Ok(()),
            Err(_) => {
                self.failed.store(true, Ordering::SeqCst);
                Err(FsyncFailed)
            }
        }
    }

    fn len(&self) -> io::Result<u64> {
        Ok(self.file.metadata()?.len())
    }
}

/// Schreibt vollständig und behandelt kurze Schreibvorgänge.
fn write_all(file: &File, mut off: u64, mut buf: &[u8]) -> io::Result<()> {
    while !buf.is_empty() {
        let n = file.write_at(buf, off)?;
        if n == 0 {
            return Err(io::Error::new(
                ErrorKind::WriteZero,
                "write returned zero bytes",
            ));
        }
        off += n as u64;
        buf = buf.get(n..).ok_or_else(|| invalid("write cursor"))?;
    }
    Ok(())
}

/// Liest `buf` vollständig über einen `O_DIRECT`-Handle.
///
/// Der Bereich wird auf 4096-Byte-Blöcke gerundet und in einen ausgerichteten Puffer
/// gelesen. Die angeforderten Bytes werden danach herauskopiert.
fn direct_read(file: &File, off: u64, buf: &mut [u8]) -> io::Result<()> {
    if buf.is_empty() {
        return Ok(());
    }
    let align = DIRECT_ALIGN as u64;
    let want = u64::try_from(buf.len()).map_err(|_| invalid("read length"))?;
    let end = off.checked_add(want).ok_or_else(|| invalid("read range"))?;
    let start = off - off % align;
    let block_end = end
        .div_ceil(align)
        .checked_mul(align)
        .ok_or_else(|| invalid("read range"))?;
    let span = usize::try_from(block_end - start).map_err(|_| invalid("read span"))?;

    // Ausgerichteter Puffer: Überschuss allozieren, Startadresse auf 4096 schieben.
    let mut raw = vec![0_u8; span + DIRECT_ALIGN];
    let pad = (DIRECT_ALIGN - (raw.as_ptr() as usize) % DIRECT_ALIGN) % DIRECT_ALIGN;
    let window = raw
        .get_mut(pad..pad + span)
        .ok_or_else(|| invalid("aligned window"))?;

    let got = read_up_to(file, start, window)?;
    let rel = usize::try_from(off - start).map_err(|_| invalid("relative offset"))?;
    let needed = rel + buf.len();
    if got < needed {
        return Err(io::Error::new(
            ErrorKind::UnexpectedEof,
            "file shorter than requested range",
        ));
    }
    let src = window
        .get(rel..needed)
        .ok_or_else(|| invalid("copy range"))?;
    buf.copy_from_slice(src);
    Ok(())
}

/// Liest, bis `buf` voll ist oder das Dateiende erreicht wurde. Gibt die Anzahl der Bytes zurück.
fn read_up_to(file: &File, off: u64, buf: &mut [u8]) -> io::Result<usize> {
    let mut filled = 0_usize;
    while filled < buf.len() {
        let rest = buf
            .get_mut(filled..)
            .ok_or_else(|| invalid("read cursor"))?;
        let n = file.read_at(rest, off + filled as u64)?;
        if n == 0 {
            break;
        }
        filled += n;
    }
    Ok(filled)
}

/// Verzeichnis des Pfads; ein leerer Elternteil bedeutet das aktuelle Verzeichnis.
fn parent_of(p: &Path) -> &Path {
    match p.parent() {
        Some(d) if !d.as_os_str().is_empty() => d,
        _ => Path::new("."),
    }
}

/// `fsync` auf das Elternverzeichnis, damit Namensänderungen dauerhaft sind (WP-5).
fn sync_parent_dir(p: &Path) -> io::Result<()> {
    File::open(parent_of(p))?.sync_all()
}

/// `O_DIRECT`-Wert der Architektur. Nur für x86_64 und aarch64 bekannt; bei anderen
/// Architekturen liefert `None` (fail-closed).
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
fn direct_flag() -> Option<i32> {
    Some(0o40000)
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
fn direct_flag() -> Option<i32> {
    None
}

fn invalid(what: &'static str) -> io::Error {
    io::Error::new(ErrorKind::InvalidInput, what)
}
