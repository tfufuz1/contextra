// FILE-CONTEXT
// STAND: 2026-11-02T00:00:00Z
// ZWECK: Platform-aware low-level durability primitives for file and directory flushes (Ring 0 / SPEC §8.3, SY-05, SY-06, SY-10, D-55, D-56)
// INVARIANTEN: Zero panic; typed io::Error/SyncFailure propagation; fail-closed on unsupported targets; EINTR handling

//! # Modul: Dauerhaftigkeit und physische Synchronisation (`durability`)
//!
//! Dieses Modul stellt plattformspezifische Grundfunktionen (Primitiven) zur Verfügung,
//! um sicherzustellen, dass geschriebene Daten tatsächlich auf dem physischen Speichermedium
//! (SSD/HDD) dauerhaft persistiert werden (Durability gemäß ACID-Kriterien).
//!
//! ## Ring-Zugehörigkeit und Zweck
//! - **Ring 0 (Kern):** Kapselt direkte Betriebssystem-Aufrufe (`libc::fsync`, `libc::fdatasync`,
//!   `libc::fcntl(F_FULLFSYNC)`).
//! - **Verwendung im System:** Wird von `contextra-store` (Write-Ahead-Log WAL und LSM-SSTables)
//!   sowie `contextra-durable-fs` genutzt.
//! - **Persistenz & Fehlergarantie:** Fehler bei `fsync`-Aufrufen werden niemals verschluckt, sondern
//!   strikte als [`SyncFailure`] an den Aufrufer propagiert (Fail-Closed-Prinzip: Bricht der Flush
//!   ab, darf keine Transaktion als erfolgreich bestätigt werden).
//!
//! ## Plattformunterschiede (SY-06)
//! - **Linux:** Nutzt `fdatasync` (wenn keine Metadaten wie Dateigröße geändert wurden) oder `fsync`.
//! - **macOS:** Standardmäßiges `fsync` flasht auf Apple-Dateisystemen (APFS) oft nur den Betriebssystem-Cache,
//!   nicht jedoch den Festplattencache. Daher wird zwingend `fcntl(fd, F_FULLFSYNC, 0)` verwendet.
//! - **Signalbehandlung:** Aufrufe werden bei Unterbrechung durch Unix-Signale (`EINTR`) automatisch wiederholt.

#[path = "error.rs"]
mod error;

pub use error::SyncFailure;
use std::fs::File;
use std::io::{Error, ErrorKind, Result, Write};
use std::path::Path;

// ANCHOR[contextra-sys:DUR-FLUSH-FILE-001]
/// Flasht Dateiinhalt und optional Metadaten auf den physischen Datenträger gemäß den Garantien der Zielplattform.
///
/// # Ablauf
/// 1. Extrahiert den rohen Dateideskriptor (`fd` unter Unix).
/// 2. Führt die plattformspezifische Synchronisation aus:
///    - **Linux:** Ruft in einer Schleife `libc::fdatasync` (falls `metadata_changed == false`) oder `libc::fsync` auf.
///    - **macOS:** Ruft `libc::fcntl(fd, libc::F_FULLFSYNC, 0)` auf, um den Hardware-Plattencache zu leeren.
///    - **Anderes POSIX:** Ruft `libc::fsync` auf.
/// 3. Fängt den Fehler `EINTR` (Interrupted system call) ab und wiederholt die Operation automatisch.
/// 4. Gibt sonstige Fehler unverändert nach oben zurück (SY-05).
///
/// # Vorbedingungen
/// - `file` muss eine gültige, geöffnete Datei referenzieren.
///
/// # Nachbedingungen
/// - Bei Rückgabe von `Ok(())` befinden sich die Daten nachweislich auf dem persistenten Speichermedium.
///
/// # Fehlerfälle
/// - Gibt bei I/O-Fehlern, Dateisystemfehlern oder Plattenfehlern einen typisierten [`std::io::Error`] zurück.
pub fn durable_flush_file(file: &File, metadata_changed: bool) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        loop {
            // SAFETY: `fd` ist ein unveränderlicher, offener Dateideskriptor aus der langlebigen Referenz `&file`.
            // `libc::fsync` und `libc::fdatasync` sind standardisierte POSIX-Systemaufrufe, die keine Pointer
            // dereferenzieren und den Deskriptor nicht schließen. Fehler (inklusive Signalunterbrechung `EINTR`)
            // werden über den Rückgabewert und `errno` vollständig erfasst.
            let res = unsafe {
                if metadata_changed {
                    libc::fsync(fd)
                } else {
                    libc::fdatasync(fd)
                }
            };
            if res == 0 {
                return Ok(());
            }
            let err = Error::last_os_error();
            if err.kind() != ErrorKind::Interrupted {
                return Err(err);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        let _ = metadata_changed;
        loop {
            // SAFETY: `fd` ist ein offener POSIX-Dateideskriptor aus `&file`. `libc::fcntl` mit `F_FULLFSYNC`
            // erzwingt auf macOS / APFS das Leeren aller physischen Platten- und Controller-Caches.
            // Der Aufruf modifiziert keine Speicherbereiche und schließt den Deskriptor nicht.
            let res = unsafe { libc::fcntl(fd, libc::F_FULLFSYNC, 0) };
            if res != -1 {
                return Ok(());
            }
            let err = Error::last_os_error();
            if err.kind() != ErrorKind::Interrupted {
                return Err(err);
            }
        }
    }

    #[cfg(all(unix, not(target_os = "linux"), not(target_os = "macos")))]
    {
        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        let _ = metadata_changed;
        loop {
            // SAFETY: `fd` ist ein unveränderlicher, offener Dateideskriptor aus `&file`. `libc::fsync`
            // flasht Daten und Metadaten auf den persistenten Datenträger ohne Speicherzugriffe oder Deskriptormodifikationen.
            let res = unsafe { libc::fsync(fd) };
            if res == 0 {
                return Ok(());
            }
            let err = Error::last_os_error();
            if err.kind() != ErrorKind::Interrupted {
                return Err(err);
            }
        }
    }

    #[cfg(not(unix))]
    {
        let _ = (file, metadata_changed);
        Err(Error::new(
            ErrorKind::Unsupported,
            "durable_flush_file is not supported on this target architecture (fail-closed)",
        ))
    }
}

// ANCHOR[contextra-sys:DUR-FLUSH-DIR-001]
/// Flasht den Verzeichniseintrag (übergeordnetes Verzeichnis) einer neu erstellten oder umbenannten Datei (SY-06).
///
/// # Warum ein Directory Flush notwendig ist
/// Unter Unix-Dateisystemen (wie ext4 oder XFS) reicht ein `fsync` auf eine neu erstellte Datei nicht aus,
/// um zu garantieren, dass die Datei nach einem Stromausfall auffindbar ist: Der Verzeichniseintrag (Inode-Link)
/// liegt im übergeordneten Verzeichnis und muss separat geflasht werden.
///
/// # Ablauf
/// 1. Öffnet das Verzeichnis schreibgeschützt als [`File`] und ruft [`durable_flush_file`] mit `metadata_changed = true` auf.
/// 2. Bei nicht unterstützter Plattform: Gibt `ErrorKind::Unsupported` zurück (Fail-Closed).
///
/// # Fehlerfälle
/// Gibt einen [`std::io::Error`] zurück, wenn das Verzeichnis nicht geöffnet oder nicht synchronisiert werden kann.
pub fn durable_flush_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let dir = File::open(path)?;
        durable_flush_file(&dir, true)
    }

    #[cfg(not(unix))]
    {
        let _ = path;
        Err(Error::new(
            ErrorKind::Unsupported,
            "durable_flush_dir is not supported on this target architecture (fail-closed)",
        ))
    }
}

// ANCHOR[contextra-sys:DUR-WRITE-ALL-001]
/// Schreibt die gesamten Bytes aus `buf` zuverlässig in `file`, unter robuster Behandlung partieller Schreibvorgänge und Signal-Unterbrechungen.
///
/// # Ablauf
/// 1. Führt eine Schleife aus, solange noch ungelesene Bytes in `buf` verbleiben.
/// 2. Ruft `file.write(buf)` auf.
/// 3. Behandelt die Rückgabewerte:
///    - `Ok(0)`: Unerwartetes Dateiende / keine Bytes geschrieben -> Abbruch mit `ErrorKind::WriteZero`.
///    - `Ok(n)`: Rückt den Pufferzeiger um `n` geschriebene Bytes vor.
///    - `Err(EINTR)`: Wiederholt den Schreibaufruf ohne Datenverlust.
///    - Sonstiger Fehler: Gibt den I/O-Fehler sofort an den Aufrufer zurück.
///
/// # Fehlerfälle
/// Gibt einen [`std::io::Error`] zurück, wenn das Zielmedium voll ist (`WriteZero`) oder ein Hardware-Fehler auftritt.
pub fn write_all_durable(file: &mut File, mut buf: &[u8]) -> Result<()> {
    while !buf.is_empty() {
        match file.write(buf) {
            Ok(0) => {
                return Err(Error::new(
                    ErrorKind::WriteZero,
                    "failed to write whole buffer: write returned 0 bytes",
                ));
            }
            Ok(n) => {
                buf = match buf.get(n..) {
                    Some(b) => b,
                    None => &[],
                };
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Specification of the flush scope for file handles (SY-06).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlushKind {
    /// Flush only file data to stable storage without metadata.
    Data,
    /// Flush both file data and metadata to stable storage.
    All,
    /// Compatibility alias: flush file data without metadata.
    DataOnly,
    /// Compatibility alias: flush file data and metadata.
    DataAndMetadata,
}

/// Filesystem classification for durability and locking guarantees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FsClass {
    /// Local filesystem with supported POSIX durability guarantees.
    LocalSupported,
    /// Network filesystem.
    Network,
    /// User-space filesystem.
    Fuse,
    /// Cloud-synchronized directory.
    CloudSync,
    /// Unknown filesystem.
    Unknown,
}

impl FsClass {
    /// Returns true if the filesystem supports full local persistence and locking guarantees.
    #[must_use]
    pub const fn is_supported_local(&self) -> bool {
        matches!(self, Self::LocalSupported)
    }
}

/// Flushes an open file to disk using platform durability primitives (SY-05, SY-06).
pub fn flush_file(file: &File, kind: FlushKind) -> std::result::Result<(), SyncFailure> {
    let metadata_changed = match kind {
        FlushKind::Data | FlushKind::DataOnly => false,
        FlushKind::All | FlushKind::DataAndMetadata => true,
    };
    durable_flush_file(file, metadata_changed).map_err(SyncFailure::new)
}

/// Flushes directory metadata entries to stable storage (SY-06).
pub fn flush_dir(dir: &Path) -> std::result::Result<(), SyncFailure> {
    durable_flush_dir(dir).map_err(SyncFailure::new)
}

/// Classifies the filesystem host of `path`.
pub fn fs_class(path: &Path) -> Result<FsClass> {
    let path_str = path.to_string_lossy().to_lowercase();
    if path_str.contains("onedrive")
        || path_str.contains("dropbox")
        || path_str.contains("icloud")
        || path_str.contains("google drive")
    {
        Ok(FsClass::CloudSync)
    } else {
        Ok(FsClass::LocalSupported)
    }
}

/// Durably renames `from` to `to` and flushes the parent directory (SY-05, SY-06).
///
/// # Ablauf
/// 1. Benennt `from` atomar in `to` um mittels [`std::fs::rename`].
/// 2. Ermittelt das übergeordnete Verzeichnis von `to` (bzw. `.` falls relativer Dateiname) und flasht dieses via [`flush_dir`].
///
/// # Fehlerbehandlung
/// Fehler bei der Verzeichnissynchronisation werden nicht verschluckt, sondern sofort als
/// [`SyncFailure`] zurückgegeben, um Datenverlust bei Stromausfall nachweisbar zu verhindern.
pub fn rename_durable(from: &Path, to: &Path) -> std::result::Result<(), SyncFailure> {
    std::fs::rename(from, to).map_err(SyncFailure::new)?;
    let parent = match to.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    flush_dir(parent)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::Read;

    #[test]
    fn test_write_all_durable_and_flush_file() -> std::io::Result<()> {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "contextra_sys_durability_test_{}.bin",
            std::process::id()
        ));

        {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&file_path)?;

            write_all_durable(&mut file, b"contextra durability payload")?;
            durable_flush_file(&file, false)?;
            durable_flush_file(&file, true)?;
        }

        {
            let mut file = File::open(&file_path)?;
            let mut content = Vec::new();
            file.read_to_end(&mut content)?;
            assert_eq!(&content[..], b"contextra durability payload");
        }

        let _ = std::fs::remove_file(file_path);
        Ok(())
    }

    #[test]
    fn test_durable_flush_file_metadata_variations() -> std::io::Result<()> {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "contextra_sys_durability_meta_{}.bin",
            std::process::id()
        ));

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)?;

        write_all_durable(&mut file, b"metadata variation test")?;
        durable_flush_file(&file, false)?;
        durable_flush_file(&file, true)?;

        let _ = std::fs::remove_file(file_path);
        Ok(())
    }

    #[test]
    fn test_durable_flush_dir() -> std::io::Result<()> {
        let temp_dir = std::env::temp_dir();
        let test_subdir = temp_dir.join(format!(
            "contextra_sys_durability_dir_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&test_subdir)?;

        let file_path = test_subdir.join("data.txt");
        {
            let mut file = File::create(&file_path)?;
            write_all_durable(&mut file, b"test payload")?;
            durable_flush_file(&file, true)?;
        }

        durable_flush_dir(&test_subdir)?;

        let _ = std::fs::remove_file(file_path);
        let _ = std::fs::remove_dir(test_subdir);
        Ok(())
    }

    #[test]
    fn test_write_all_durable_empty_buffer() -> std::io::Result<()> {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "contextra_sys_empty_test_{}.bin",
            std::process::id()
        ));

        let mut file = File::create(&file_path)?;
        write_all_durable(&mut file, &[])?;
        durable_flush_file(&file, false)?;

        let _ = std::fs::remove_file(file_path);
        Ok(())
    }

    #[test]
    fn test_write_all_durable_chunked() -> std::io::Result<()> {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join(format!(
            "contextra_sys_chunk_test_{}.bin",
            std::process::id()
        ));

        let mut file = File::create(&file_path)?;
        let data = vec![0x37u8; 64 * 1024];
        write_all_durable(&mut file, &data)?;
        durable_flush_file(&file, true)?;

        let _ = std::fs::remove_file(file_path);
        Ok(())
    }

    #[test]
    fn test_durable_flush_dir_nonexistent_path() {
        let nonexistent = Path::new("/nonexistent_path_contextra_sys_test_12345");
        let res = durable_flush_dir(nonexistent);
        assert!(res.is_err());
        if let Err(err) = res {
            assert_eq!(err.kind(), ErrorKind::NotFound);
        }
    }
}
