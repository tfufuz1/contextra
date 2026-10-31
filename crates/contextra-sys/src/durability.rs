// FILE-CONTEXT
// STAND: 2026-10-28T00:00:00Z
// ZWECK: Platform-aware low-level durability primitives for file and directory flushes (Ring 0 / SPEC 7.7)
// INVARIANTEN: Zero panic; typed io::Error propagation; fail-closed on unsupported targets; EINTR handling
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7 (INV-DURABILITY-PRIM-1)

//! # Modul: Dauerhaftigkeit und physische Synchronisation (`durability`)
//!
//! Dieses Modul stellt plattformspezifische Grundfunktionen (Primitiven) zur Verfügung,
//! um sicherzustellen, dass geschriebene Daten tatsächlich auf dem physischen Speichermedium
//! (SSD/HDD) dauerhaft persistiert werden (Durability gemäß ACID-Kriterien).
//!
//! ## Ring-Zugehörigkeit und Zweck
//! - **Ring 0 (Kern):** Kapselt direkte Betriebssystem-Aufrufe (`libc::fsync`, `libc::fdatasync`,
//!   `libc::fcntl(F_FULLFSYNC)`, `FlushFileBuffers`).
//! - **Verwendung im System:** Wird von `contextra-store` (Write-Ahead-Log WAL und LSM-SSTables)
//!   sowie `contextra-durable-fs` genutzt.
//! - **Persistenz & Fehlergarantie:** Fehler bei `fsync`-Aufrufen werden niemals verschluckt, sondern
//!   strikte als [`std::io::Error`] an den Aufrufer propagiert (Fail-Closed-Prinzip: Bricht der Flush
//!   ab, darf keine Transaktion als erfolgreich bestätigt werden).
//!
//! ## Plattformunterschiede
//! - **Linux:** Nutzt `fdatasync` (wenn keine Metadaten wie Dateigröße geändert wurden) oder `fsync`.
//! - **macOS:** Standardmäßiges `fsync` flasht auf Apple-Dateisystemen (APFS) oft nur den Betriebssystem-Cache,
//!   nicht jedoch den Festplattencache. Daher wird zwingend `fcntl(fd, F_FULLFSYNC, 0)` verwendet.
//! - **Windows:** Verwendet `FlushFileBuffers`.
//! - **Signalbehandlung:** Aufrufe werden bei Unterbrechung durch Unix-Signale (`EINTR`) automatisch wiederholt.

use std::fs::File;
use std::io::{Error, ErrorKind, Result, Write};
use std::path::Path;

// ANCHOR[contextra-sys:DUR-FLUSH-FILE-001]
/// Flasht Dateiinhalt und optional Metadaten auf den physischen Datenträger gemäß den Garantien der Zielplattform.
///
/// # Ablauf
/// 1. Extrahiert den rohen Dateideskriptor bzw. das Datei-Handle (`fd` unter Unix, `HANDLE` unter Windows).
/// 2. Führt die plattformspezifische Synchronisation aus:
///    - **Linux:** Ruft in einer Schleife `libc::fdatasync` (falls `metadata_changed == false`) oder `libc::fsync` auf.
///    - **macOS:** Ruft `libc::fcntl(fd, libc::F_FULLFSYNC, 0)` auf, um den Hardware-Plattencache zu leeren.
///    - **Anderes POSIX:** Ruft `libc::fsync` auf.
///    - **Windows:** Ruft `FlushFileBuffers` auf.
///    - **Nicht unterstützte Plattform:** Gibt sofort einen `Unsupported`-Fehler zurück (Fail-Closed).
/// 3. Fängt den Fehler `EINTR` (Interrupted system call) ab und wiederholt die Operation automatisch.
///
/// # Vorbedingungen
/// - `file` muss eine gültige, geöffnete Datei referenzieren.
///
/// # Nachbedingungen
/// - Bei Rückgabe von `Ok(())` befinden sich die Daten nachweislich auf dem persistenten Speichermedium.
///
/// # Fehlerfälle
/// - Gibt bei I/O-Fehlern, Dateisystemfehlern oder Plattenfehlern einen typisierten [`std::io::Error`] zurück.
///
/// # Beispiel
/// ```no_run
/// use std::fs::File;
/// use contextra_sys::durable_flush_file;
///
/// # fn run() -> std::io::Result<()> {
/// let file = File::open("wal_segment.log")?;
/// durable_flush_file(&file, false)?;
/// # Ok(())
/// # }
/// ```
pub fn durable_flush_file(file: &File, metadata_changed: bool) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        loop {
            // SAFETY: `fd` ist ein gültiger, offener Dateideskriptor, der direkt aus `std::fs::File::as_raw_fd()` bezogen wurde.
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
            // SAFETY: `fd` ist ein gültiger Dateideskriptor aus `std::fs::File`. `F_FULLFSYNC` erzwingt auf macOS das Leeren aller physischen Caches.
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
            // SAFETY: `fd` ist ein gültiger offener Dateideskriptor aus `std::fs::File`.
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

    #[cfg(windows)]
    {
        use std::os::windows::io::AsRawHandle;
        let handle = file.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
        let _ = metadata_changed;
        // SAFETY: `handle` ist ein gültiges OS-Datei-Handle aus `std::fs::File::as_raw_handle()`.
        let res = unsafe { windows_sys::Win32::Storage::FileSystem::FlushFileBuffers(handle) };
        if res != 0 {
            Ok(())
        } else {
            Err(Error::last_os_error())
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = (file, metadata_changed);
        Err(Error::new(
            ErrorKind::Unsupported,
            "durable_flush_file is not supported on this target architecture (fail-closed)",
        ))
    }
}

// ANCHOR[contextra-sys:DUR-FLUSH-DIR-001]
/// Flasht den Verzeichniseintrag (übergeordnetes Verzeichnis) einer neu erstellten oder umbenannten Datei.
///
/// # Warum ein Directory Flush notwendig ist
/// Unter Unix-Dateisystemen (wie ext4 oder XFS) reicht ein `fsync` auf eine neu erstellte Datei nicht aus,
/// um zu garantieren, dass die Datei nach einem Stromausfall auffindbar ist: Der Verzeichniseintrag (Inode-Link)
/// liegt im übergeordneten Verzeichnis und muss separat geflasht werden.
///
/// # Ablauf
/// 1. **Unix:** Öffnet das Verzeichnis schreibgeschützt als [`File`] und ruft [`durable_flush_file`] mit `metadata_changed = true` auf.
/// 2. **Windows:** Verzeichnis-Handles unterstützen kein `FlushFileBuffers`; Änderungen werden über Dateisystem-Semantiken verwaltet (Best Effort, `Ok(())`).
/// 3. **Nicht unterstützt:** Gibt `ErrorKind::Unsupported` zurück (Fail-Closed).
///
/// # Fehlerfälle
/// Gibt einen [`std::io::Error`] zurück, wenn das Verzeichnis nicht geöffnet oder nicht synchronisiert werden kann.
pub fn durable_flush_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let dir = File::open(path)?;
        durable_flush_file(&dir, true)
    }

    #[cfg(windows)]
    {
        let _ = path;
        // Windows directory handles do not support FlushFileBuffers; changes are managed via filesystem semantics (Best Effort).
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
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
                buf = buf.get(n..).unwrap_or(&[]);
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

// ANCHOR[contextra-sys:DUR-TESTS-001]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::Read;

    // ANCHOR[contextra-sys:DUR-TEST-WRITE-FLUSH-001]
    // Prüft das Zusammenspiel von `write_all_durable` und nachfolgendem `durable_flush_file` mit Daten- und Metadaten-Flush.
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

    // ANCHOR[contextra-sys:DUR-TEST-META-VAR-001]
    // Prüft das Flashen von Dateien bei Variation des `metadata_changed`-Flags (fdatasync vs. fsync).
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

    // ANCHOR[contextra-sys:DUR-TEST-DIR-001]
    // Prüft die Verzeichnissynchronisation (`durable_flush_dir`) nach dem Erstellen einer Datei in einem Unterverzeichnis.
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

    // ANCHOR[contextra-sys:DUR-TEST-EMPTY-001]
    // Prüft, dass das haltbare Schreiben eines leeren Puffers ohne Fehler erfolgreich beendet wird.
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

    // ANCHOR[contextra-sys:DUR-TEST-CHUNKED-001]
    // Prüft das Schreiben größerer Datenmengen (64 KiB), um blockweises Schreiben zu testen.
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

    // ANCHOR[contextra-sys:DUR-TEST-NONEXISTENT-DIR-001]
    // Prüft das Fehlerverhalten beim Flashen eines nicht existierenden Verzeichnisses.
    #[test]
    fn test_durable_flush_dir_nonexistent_path() {
        let nonexistent = std::path::Path::new("/nonexistent_path_contextra_sys_test_12345");
        #[cfg(unix)]
        {
            assert_eq!(
                durable_flush_dir(nonexistent).err().map(|e| e.kind()),
                Some(std::io::ErrorKind::NotFound)
            );
        }
        #[cfg(windows)]
        {
            let res = durable_flush_dir(nonexistent);
            assert!(res.is_ok());
        }
    }
}
