// FILE-CONTEXT
// STAND: 2026-11-02T00:00:00Z
// ZWECK: Platform-aware low-level durability primitives for file and directory flushes (Ring 1 / SPEC 03-contextra-sys.md)
// INVARIANTEN: Zero panic; SyncFailure error propagation; fail-closed on unsupported targets; EINTR handling
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7 (INV-DURABILITY-PRIM-1)

//! # Modul: Dauerhaftigkeit und physische Synchronisation (`durability`)
//!
//! Dieses Modul stellt Grundfunktionen (Primitiven) zur Verfügung,
//! um sicherzustellen, dass geschriebene Daten tatsächlich auf dem physischen Speichermedium
//! (SSD/HDD) dauerhaft persistiert werden (Durability gemäß ACID-Kriterien).
//!
//! ## Ring-Zugehörigkeit und Zweck
//! - **Ring 1 (Kern):** Kapselt direkte Betriebssystem-Aufrufe (`libc::fsync`, `libc::fdatasync`,
//!   `libc::fcntl(F_FULLFSYNC)`).
//! - **Verwendung im System:** Wird von `contextra-store` (Write-Ahead-Log WAL und LSM-SSTables)
//!   sowie `contextra-durable-fs` genutzt.
//! - **Persistenz & Fehlergarantie:** Fehler bei `fsync`-Aufrufen werden niemals verschluckt, sondern
s//!   strikt als [`std::io::Result`] bzw. [`SyncFailure`] an den Aufrufer propagiert (Fail-Closed-Prinzip).
//!
//! ## Plattformunterschiede
//! - **Linux:** Nutzt `fdatasync` (für Daten) oder `fsync` (für alle Metadaten).
//! - **macOS:** Standardmäßiges `fsync` flasht auf Apple-Dateisystemen (APFS) oft nur den Betriebssystem-Cache.
//!   Daher wird zwingend `fcntl(fd, F_FULLFSYNC, 0)` verwendet (SY-06).
//! - **Signalbehandlung:** Aufrufe werden bei Unterbrechung durch Unix-Signale (`EINTR`) automatisch wiederholt.

#[path = "error.rs"]
mod error;

pub use error::SyncFailure;
use std::fs::File;
use std::io::{Error, ErrorKind, Result, Write};
use std::os::unix::io::AsRawFd;
use std::path::Path;

use crate::error::SyncFailure;

/// Specification of the flush scope for file handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlushKind {
    /// Flush only data without forcing metadata updates to disk (Linux `fdatasync`).
    Data,
    /// Flush both data and all filesystem metadata (Linux `fsync`, macOS `F_FULLFSYNC`).
    All,
    /// Abwaertskompatibilitaet: alias for Data.
    DataOnly,
    /// Abwaertskompatibilitaet: alias for All.
    DataAndMetadata,
}

// ANCHOR[contextra-sys:DUR-FLUSH-FILE-001]
/// Flasht Dateiinhalt und optional Metadaten auf den physischen Datenträger gemäß den Garantien der Zielplattform.
///
/// # Ablauf
/// 1. Extrahiert den rohen Dateideskriptor (`fd` unter Unix).
/// 2. Führt die plattformspezifische Synchronisation aus:
///    - **Linux:** Ruft in einer Schleife `libc::fdatasync` (bei [`FlushKind::Data`]) oder `libc::fsync` (bei [`FlushKind::All`]) auf.
///    - **macOS:** Ruft `libc::fcntl(fd, libc::F_FULLFSYNC, 0)` auf, um den Hardware-Plattencache zu leeren (SY-06).
///    - **Anderes POSIX:** Ruft `libc::fsync` auf.
///    - **Nicht unterstützte Plattform:** Gibt sofort einen `Unsupported`-Fehler zurück (Fail-Closed).
/// 3. Fängt den Fehler `EINTR` (Interrupted system call) ab und wiederholt die Operation automatisch.
/// 4. Gibt sonstige Fehler unverändert nach oben zurück (SY-05).
///
/// # Vorbedingungen
/// - `file` muss eine gültige, geöffnete Datei referenzieren.
///
/// # Nachbedingungen
/// - Bei Rückgabe von `Ok(())` befinden sich die Daten nachweislich auf dem persistenten Speichermedium.
pub fn flush_file(file: &File, kind: FlushKind) -> Result<()> {
    flush_file_sync(file, kind).map_err(Into::into)
}

/// Spezifikationsgemaeße Variante mit `SyncFailure` als Fehlertyp (D-55).
pub fn flush_file_sync(file: &File, kind: FlushKind) -> std::result::Result<(), SyncFailure> {
    let fd = file.as_raw_fd();
    if fd < 0 {
        return Err(SyncFailure::PermissionDenied);
    }

    #[cfg(target_os = "linux")]
    {
        loop {
            // SAFETY: `fd` ist ein gültiger, geöffneter Dateideskriptor aus `file.as_raw_fd()`.
            // Die Deskriptorlebensdauer wird durch die unveränderliche Referenz `&File` garantiert.
            let res = unsafe {
                match kind {
                    FlushKind::Data | FlushKind::DataOnly => libc::fdatasync(fd),
                    FlushKind::All | FlushKind::DataAndMetadata => libc::fsync(fd),
                }
            };
            if res == 0 {
                return Ok(());
            }
            let err = Error::last_os_error();
            if err.kind() == ErrorKind::Interrupted {
                continue;
            }
            return Err(SyncFailure::from(err));
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = kind;
        loop {
            // SAFETY: `fd` ist ein geöffneter, gültiger Dateideskriptor aus `file.as_raw_fd()`.
            // `F_FULLFSYNC` verlangt das dritte Argument als `0` und leert den Hardware-Speichercache.
            let res = unsafe { libc::fcntl(fd, libc::F_FULLFSYNC, 0) };
            if res == 0 {
                return Ok(());
            }
            let err = Error::last_os_error();
            if err.kind() == ErrorKind::Interrupted {
                continue;
            }
            return Err(SyncFailure::from(err));
        }
    }

    #[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
    {
        let _ = kind;
        loop {
            // SAFETY: `fd` ist ein gültiger offener Dateideskriptor aus `file.as_raw_fd()`.
            let res = unsafe { libc::fsync(fd) };
            if res == 0 {
                return Ok(());
            }
            let err = Error::last_os_error();
            if err.kind() == ErrorKind::Interrupted {
                continue;
            }
            return Err(SyncFailure::from(err));
        }
    }

    #[cfg(not(unix))]
    {
        let _ = fd;
        let _ = kind;
        Err(SyncFailure::Unsupported)
    }
}

// ANCHOR[contextra-sys:DUR-FLUSH-DIR-001]
/// Flasht ein Verzeichnis persistent auf den Datenträger.
///
/// # Ablauf
/// 1. Öffnet das Verzeichnis schreibgeschützt als [`File`].
/// 2. Ruft [`flush_file`] mit [`FlushKind::All`] auf.
///
/// # Fehlerfälle
/// Gibt einen [`std::io::Error`] zurück, wenn das Verzeichnis nicht geöffnet oder nicht synchronisiert werden kann.
pub fn flush_dir(path: &Path) -> Result<()> {
    flush_dir_sync(path).map_err(Into::into)
}

/// Spezifikationsgemaeße Variante mit `SyncFailure` als Fehlertyp (D-55).
pub fn flush_dir_sync(path: &Path) -> std::result::Result<(), SyncFailure> {
    let dir = File::open(path).map_err(SyncFailure::from)?;
    flush_file_sync(&dir, FlushKind::All)
}

/// Durably renames `from` to `to` and flushes the parent directory.
///
/// # Ablauf
/// 1. Benennt `from` atomar in `to` um mittels [`std::fs::rename`].
/// 2. Ermittelt das übergeordnete Verzeichnis von `to` und flasht dieses via [`flush_dir`].
///
/// # Fehlerbehandlung
/// Fehler bei der Verzeichnissynchronisation werden nicht verschluckt, sondern sofort als
/// Fehler zurückgegeben (D-55, SY-05).
pub fn rename_durable(from: &Path, to: &Path) -> Result<()> {
    rename_durable_sync(from, to).map_err(Into::into)
}

/// Spezifikationsgemaeße Variante mit `SyncFailure` als Fehlertyp (D-55).
pub fn rename_durable_sync(from: &Path, to: &Path) -> std::result::Result<(), SyncFailure> {
    std::fs::rename(from, to).map_err(SyncFailure::from)?;
    if let Some(parent) = to.parent() {
        flush_dir_sync(parent)?;
    }
    Ok(())
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

// ============================================================================
// Kompatibilitaets-Wrapper fuer bestehende Konsumenten (z. B. contextra-durable-fs)
// ============================================================================

/// Abwaertskompatible Signatur fuer `flush_file`.
pub fn durable_flush_file(file: &File, metadata_changed: bool) -> Result<()> {
    let kind = if metadata_changed {
        FlushKind::All
    } else {
        FlushKind::Data
    };
    flush_file(file, kind)
}

/// Abwaertskompatible Signatur fuer `flush_dir`.
pub fn durable_flush_dir(path: &Path) -> Result<()> {
    flush_dir(path)
}
