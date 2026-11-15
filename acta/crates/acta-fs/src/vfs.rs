//! Dateisystem-Ports `Vfs` und `VFile` (Spec 3.7, WP-5 bis WP-7, 5.6).
//!
//! Alle Dateizugriffe des Speichers laufen über diese Ports (G-FS). Die Signaturen
//! folgen Kap. 3.7 normativ. Die Ports liegen in diesem Crate, weil ihr Erbringer
//! `DurableVfs` hier liegt (HR-2).

use crate::error::FsyncFailed;
use std::io;
use std::path::{Path, PathBuf};

/// Dateisystem-Port: legt Dateien an, öffnet sie, ersetzt und entfernt sie.
pub trait Vfs: Send + Sync {
    /// Legt eine neue Datei an. Existiert der Pfad bereits, scheitert der Aufruf
    /// mit `ErrorKind::AlreadyExists`.
    ///
    /// # Errors
    /// `io::Error` bei Anlage, Berechtigungs- oder Platzmangel.
    fn create_new(&self, p: &Path) -> io::Result<Box<dyn VFile>>;

    /// Öffnet eine Datei lesend.
    ///
    /// # Errors
    /// `io::Error`, wenn die Datei fehlt oder nicht lesbar ist.
    fn open_read(&self, p: &Path) -> io::Result<Box<dyn VFile>>;

    /// Öffnet eine Datei lesend mit `O_DIRECT`. Lesevorgänge umgehen den Seiten-Cache
    /// und arbeiten intern auf 4096 Byte ausgerichteten Puffern.
    ///
    /// # Errors
    /// `ErrorKind::Unsupported` auf Architekturen ohne bekannten `O_DIRECT`-Wert.
    /// Sonst `io::Error`, auch wenn das Dateisystem `O_DIRECT` ablehnt (fail-closed).
    fn open_direct_read(&self, p: &Path) -> io::Result<Box<dyn VFile>>;

    /// Ersetzt `to` durch `from` atomar (`rename`) und synchronisiert die betroffenen
    /// Verzeichnisse (`fsync` auf das Verzeichnis).
    ///
    /// # Errors
    /// `io::Error` bei Umbenennung oder Verzeichnis-Synchronisation.
    fn rename_atomic(&self, from: &Path, to: &Path) -> io::Result<()>;

    /// Überschreibt den Inhalt mit Nullen, synchronisiert ihn und entfernt die Datei.
    /// Flash-Reste sind davon nicht erfasst (A-FLASH).
    ///
    /// # Errors
    /// `io::Error` bei Überschreiben, Synchronisation oder Entfernen.
    fn remove_scrub(&self, p: &Path) -> io::Result<()>;

    /// Listet die Einträge eines Verzeichnisses in aufsteigender Reihenfolge.
    ///
    /// # Errors
    /// `io::Error`, wenn das Verzeichnis nicht lesbar ist.
    fn list(&self, dir: &Path) -> io::Result<Vec<PathBuf>>;
}

/// Dateihandle. Nach einem fehlgeschlagenen `sync_data` ist die Datei für diesen
/// Prozess endgültig entwertet (WP-7).
///
/// Ein `is_empty` fehlt absichtlich: `len` ist fehlbar, und ein bool ohne Fehlerpfad
/// wäre nicht ehrlich.
#[allow(clippy::len_without_is_empty)]
pub trait VFile: Send + Sync {
    /// Schreibt `buf` vollständig ab Byte-Offset `off`.
    ///
    /// # Errors
    /// `io::Error`, auch wenn die Datei nach einem Fsync-Versagen nicht mehr
    /// beschrieben werden darf.
    fn write_all_at(&self, off: u64, buf: &[u8]) -> io::Result<()>;

    /// Füllt `buf` vollständig ab Byte-Offset `off`.
    ///
    /// # Errors
    /// `ErrorKind::UnexpectedEof`, wenn die Datei zu kurz ist. Sonst `io::Error`.
    fn read_exact_at(&self, off: u64, buf: &mut [u8]) -> io::Result<()>;

    /// Synchronisiert die Daten der Datei (`fdatasync`).
    ///
    /// # Errors
    /// [`FsyncFailed`] bei jedem Fehler. Ein weiterer Aufruf liefert ebenfalls
    /// [`FsyncFailed`], auch wenn das Betriebssystem nun Erfolg meldet.
    fn sync_data(&self) -> Result<(), FsyncFailed>;

    /// Aktuelle Länge der Datei in Byte.
    ///
    /// # Errors
    /// `io::Error`, wenn die Metadaten nicht lesbar sind.
    fn len(&self) -> io::Result<u64>;
}
