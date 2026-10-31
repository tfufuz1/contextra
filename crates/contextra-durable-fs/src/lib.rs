#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

//! # Dauerhafte Dateisystem-Operationen und synchrone Disk-Flushes (`contextra-durable-fs`)
//!
//! Zweck: Bereitstellung von betriebssystemübergreifenden Hilfsfunktionen für atomares Ersetzen,
//! dauerhaftes Löschen und synchrone Verzeichnis- und Datei-Flushes (`fsync`/`fdatasync`).
//!
//! Ring-Zugehörigkeit: Ring 0 (Basisinfrastruktur / OS-Abstraktion; in `Cargo.toml` als `ring = "0"` annotiert).
//! Wird eingebunden von: Derzeit keinen anderen Crates direkt (Stand SPEC 4.1 Ist-Zustand).
//! Abhängigkeiten: [`contextra-sys`] (Plattform-Durability-Primitive und Dateisystem-Klassifikation).
//!
//! Wichtigste bereitgestellte Komponenten:
//! - [`sync_dir`]: Flusht Verzeichnis-Metadaten auf die physische Festplatte.
//! - [`atomic_replace`]: Ersetzt eine Datei atomar über temporäre Zwischendatei, `fsync` und Verzeichnis-Sync.
//! - [`durable_remove`]: Löscht eine Datei und flusht das Elternverzeichnis dauerhaft.
//! - [`scrub_and_remove`]: Überschreibt Dateien vor dem Löschen mit Nullen.
//! - [`DurableHandle`]: Kapselt [`File`] und vergiftet sich dauerhaft bei I/O- oder `fsync`-Fehlern (`INV-FSYNC-1`).
//! - [`DirLock`]: Verzeichnis-Sperre mit Dateisystem-Validierung (verbietet Locking auf Netzlaufwerken / FUSE).
//! - [`FaultInjector`]: Trait für deterministische Fehlerinjektion in Persistenzpfaden.
//!
//! ## Invarianten
//!
//! - `INV-FSYNC-1`: Dateihandles ([`DurableHandle`]) werden bei jeglichem Schreib- oder Flush-Fehler sofort
//!   vergiftet (`is_poisoned = true`), um nachfolgende fehlerhafte Schreiboperationen strikt abzuweisen (Fail-Closed).
//! - `INV-ENOSPC-1`: Speichermangel (`ENOSPC`) sowie sonstige I/O-Fehler werden deterministisch als [`std::io::Error`]
//!   propagiert; keine `panic!`, `unwrap()` oder `expect()` in Produktionspfaden.

/// Fail-Closed-Zustandsverfolgung für Speichersysteme (§18.4).
pub mod health;

/// Fehlerinjektions-Schnittstelle zur Simulation von Hardware- und Dateisystemausfällen.
pub mod fault_injector;

/// Crash-sicheres Journal für ausstehende Datei-Löschaufträge.
pub mod journal;

/// Sicheres Überschreiben (Scrubbing) und Schutzmechanismen vor dem Löschen.
pub mod scrub;

/// Abstraktion eines virtuellen Dateisystems (VFS) für entkoppeltes I/O und Testbarkeit.
pub mod vfs;

pub use contextra_sys::FlushKind;
pub use fault_injector::{FaultInjector, NoFault};
pub use health::{AtomicHealthState, HealthEvent, HealthState};
pub use journal::PendingDeleteJournal;
pub use scrub::{
    scrub_if_safe, scrub_if_safe_with_injector, ScrubOutcome, ScrubPolicy, ScrubRefusal,
};
pub use vfs::{RealFs, RealFsFile, Vfs, VfsFile, VfsOpenOptions};

use contextra_sys::{flush_dir, flush_file, fs_class, rename_durable, FsClass};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Atomarer Zähler zur Generierung eindeutiger temporärer Dateinamen für [`atomic_replace`].
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

// ANCHOR[contextra-durable-fs:POISONED-HANDLE-001]
/// Markierungstyp zur Kennzeichnung eines durch `fsync`- oder Schreibfehler vergifteten Handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoisonedHandle;

// ANCHOR[contextra-durable-fs:DURABLE-HANDLE-001]
/// Ein Wrapper um [`File`], der das Handle bei einem `fsync`- oder Schreibfehler dauerhaft vergiftet (`INV-FSYNC-1`).
///
/// Verhindert, dass nach einem fehlgeschlagenen Hardware-Sync weitere Schreiboperationen auf
/// einem potenziell inkonsistenten Dateideskriptor ausgeführt werden.
///
/// Vorbedingungen:
/// - Die übergebene Datei muss ein gültiges, offenes Dateihandle sein.
///
/// Nachbedingungen:
/// - Sobald ein Flush fehlschlägt, wechselt `is_poisoned` auf `true` und alle Folgeaufrufe scheitern sofort.
#[derive(Debug)]
pub struct DurableHandle<F: FaultInjector = NoFault> {
    file: Option<File>,
    is_poisoned: bool,
    injector: F,
    path: Option<PathBuf>,
}

impl DurableHandle<NoFault> {
    // ANCHOR[contextra-durable-fs:DURABLE-HANDLE-NEW]
    /// Kapselt eine geöffnete [`File`] in ein neues [`DurableHandle`] mit standardmäßigem Null-Kosten-Injektor ([`NoFault`]).
    pub fn new(file: File) -> Self {
        Self::new_with_injector(file, NoFault)
    }
}

impl<F: FaultInjector> DurableHandle<F> {
    // ANCHOR[contextra-durable-fs:DURABLE-HANDLE-NEW-WITH-INJECTOR]
    /// Kapselt eine geöffnete [`File`] mit einem benutzerdefinierten [`FaultInjector`].
    pub fn new_with_injector(file: File, injector: F) -> Self {
        Self {
            file: Some(file),
            is_poisoned: false,
            injector,
            path: None,
        }
    }

    // ANCHOR[contextra-durable-fs:DURABLE-HANDLE-NEW-WITH-PATH-AND-INJECTOR]
    /// Kapselt eine geöffnete [`File`] mit zugehörigem Pfad und benutzerdefiniertem [`FaultInjector`].
    pub fn new_with_path_and_injector(file: File, path: PathBuf, injector: F) -> Self {
        Self {
            file: Some(file),
            is_poisoned: false,
            injector,
            path: Some(path),
        }
    }

    // ANCHOR[contextra-durable-fs:DURABLE-HANDLE-IS-POISONED]
    /// Prüft, ob dieses Handle aufgrund eines vorangegangenen Flush-Fehlers vergiftet ist.
    #[must_use]
    pub fn is_poisoned(&self) -> bool {
        self.is_poisoned
    }

    // ANCHOR[contextra-durable-fs:DURABLE-HANDLE-FLUSH]
    /// Flusht Dateiinhalte mittels Plattform-Primitiven auf die Festplatte und führt Fehlerinjektions-Hooks aus.
    ///
    /// Ablauf von `flush`:
    /// 1. Prüfen, ob das Handle bereits vergiftet ist (`is_poisoned == true`). Falls ja, sofortiger Fehlerabbruch.
    /// 2. Prüfen, ob die zugrundeliegende Datei vorhanden ist.
    /// 3. Fehlerinjektor-Hook `on_fsync` aufrufen. Scheitert dieser, Handle vergiften und Fehler melden.
    /// 4. Betriebssystem-Flush via [`flush_file`] ausführen. Scheitert dieser, Handle vergiften (`INV-FSYNC-1`).
    ///
    /// Fehlerfälle:
    /// - Liefert `Err(std::io::Error)`, wenn das Handle vergiftet ist oder die Synchronisation fehlschlägt.
    pub fn flush(&mut self, kind: FlushKind) -> std::io::Result<()> {
        if self.is_poisoned {
            return Err(std::io::Error::other(
                "handle is poisoned due to a previous fsync failure",
            ));
        }

        let Some(file) = self.file.as_ref() else {
            return Err(std::io::Error::other("file handle has been closed"));
        };

        if let Err(err) = self.injector.on_fsync(self.path.as_deref()) {
            self.is_poisoned = true;
            return Err(err);
        }

        if let Err(err) = flush_file(file, kind) {
            self.is_poisoned = true;
            Err(err)
        } else {
            Ok(())
        }
    }

    // ANCHOR[contextra-durable-fs:DURABLE-HANDLE-GET-REF]
    /// Liefert eine unveränderliche Referenz auf die zugrundeliegende Datei, oder `None`, falls das Handle vergiftet ist.
    #[must_use]
    pub fn get_ref(&self) -> Option<&File> {
        if self.is_poisoned {
            None
        } else {
            self.file.as_ref()
        }
    }

    // ANCHOR[contextra-durable-fs:DURABLE-HANDLE-GET-MUT]
    /// Liefert eine veränderliche Referenz auf die zugrundeliegende Datei, oder `None`, falls das Handle vergiftet ist.
    pub fn get_mut(&mut self) -> Option<&mut File> {
        if self.is_poisoned {
            None
        } else {
            self.file.as_mut()
        }
    }

    // ANCHOR[contextra-durable-fs:DURABLE-HANDLE-POISON]
    /// Vergiftet dieses Handle explizit manuell (z. B. bei erkannten Prüfsummenfehlern).
    pub fn poison(&mut self) {
        self.is_poisoned = true;
    }
}

impl<F: FaultInjector> Write for DurableHandle<F> {
    // ANCHOR[contextra-durable-fs:DURABLE-HANDLE-WRITE]
    /// Schreibt Daten in die Datei, sofern das Handle nicht vergiftet ist.
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.is_poisoned {
            return Err(std::io::Error::other(
                "handle is poisoned due to a previous fsync failure",
            ));
        }

        let Some(file) = self.file.as_mut() else {
            return Err(std::io::Error::other("file handle has been closed"));
        };

        self.injector.on_write(self.path.as_deref(), buf.len())?;

        file.write(buf)
    }

    // ANCHOR[contextra-durable-fs:DURABLE-HANDLE-WRITE-FLUSH]
    /// Führt einen vollständigen Daten- und Metadaten-Flush durch.
    fn flush(&mut self) -> std::io::Result<()> {
        DurableHandle::flush(self, FlushKind::DataAndMetadata)
    }
}

// ANCHOR[contextra-durable-fs:LOCK-MODE-001]
/// Sperrmodus für Verzeichnissperren gemäß Spezifikation §7.3.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LockMode {
    /// Standard-Sperre: Erfordert ein unterstütztes lokales Dateisystem (verhindert unsichere Netzlaufwerke).
    #[default]
    Normal,
    /// Reiner In-Memory-Modus: Umgeht Dateisystemprüfungen (z. B. für reine Speichertests).
    MemoryOnly,
}

// ANCHOR[contextra-durable-fs:DIR-LOCK-001]
/// RAII-Guard (Resource Acquisition Is Initialization) für eine Verzeichnissperre (`.dirlock`).
///
/// Hält die Sperrdatei im Zielordner geöffnet und entfernt sie automatisch bei Verlassen des Gültigkeitsbereichs (`Drop`).
#[derive(Debug)]
pub struct DirLock {
    lock_path: PathBuf,
    _file: Option<File>,
    mode: LockMode,
}

impl DirLock {
    // ANCHOR[contextra-durable-fs:DIR-LOCK-ACQUIRE]
    /// Erwirbt eine Sperre auf `dir` im angegebenen [`LockMode`].
    ///
    /// Ablauf:
    /// 1. Dateisystemklasse über [`fs_class`] ermitteln.
    /// 2. Im Modus [`LockMode::Normal`]: Prüfen, ob es sich um ein unterstütztes lokales Dateisystem handelt.
    ///    Netzwerklaufwerke (`FsClass::Network`), FUSE (`FsClass::Fuse`) und Cloud-Sync-Ordner werden abgewiesen.
    /// 3. Lock-Datei `.dirlock` im Zielordner anlegen/öffnen.
    /// 4. [`DirLock`]-Instanz zurückgeben.
    ///
    /// Fehlerfälle:
    /// - Liefert `Err(ErrorKind::Unsupported)`, wenn das Dateisystem nicht unterstützt wird (INV-LOG-1: Pfade werden in Fehlermeldungen weggelassen).
    pub fn acquire(dir: &Path, mode: LockMode) -> std::io::Result<Self> {
        let class = fs_class(dir)?;

        if mode == LockMode::Normal && !class.is_supported_local() {
            let class_str = match class {
                FsClass::Network => "network",
                FsClass::Fuse => "fuse",
                FsClass::CloudSync => "cloud_sync",
                _ => "unsupported_filesystem",
            };
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!(
                    "directory locking rejected for filesystem class: {}",
                    class_str
                ),
            ));
        }

        let lock_path = dir.join(".dirlock");
        let file = if mode == LockMode::Normal {
            let mut options = OpenOptions::new();
            options.write(true).create(true);
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                // FILE_SHARE_READ (1) | FILE_SHARE_WRITE (2) | FILE_SHARE_DELETE (4)
                options.share_access(7);
            }
            Some(options.open(&lock_path)?)
        } else {
            None
        };

        Ok(Self {
            lock_path,
            _file: file,
            mode,
        })
    }

    // ANCHOR[contextra-durable-fs:DIR-LOCK-MODE]
    /// Liefert den aktiven [`LockMode`] dieser Verzeichnissperre.
    #[must_use]
    pub fn mode(&self) -> LockMode {
        self.mode
    }
}

impl Drop for DirLock {
    // NOTE: Beim Verlassen des Bereichs wird die Sperrdatei im Normal-Modus geschlossen und gelöscht.
    fn drop(&mut self) {
        if self.mode == LockMode::Normal {
            let _file = self._file.take();
            drop(_file);
            let _ = std::fs::remove_file(&self.lock_path);
        }
    }
}

// ANCHOR[contextra-durable-fs:SYNC-DIR-001]
/// Synchronisiert die Verzeichnismetadaten eines Ordners mit Standard-Injektor [`NoFault`] auf die Festplatte.
pub fn sync_dir(dir: &Path) -> std::io::Result<()> {
    sync_dir_with_injector(dir, &NoFault)
}

// ANCHOR[contextra-durable-fs:SYNC-DIR-WITH-INJECTOR]
/// Synchronisiert die Verzeichnismetadaten eines Ordners mit benutzerdefiniertem [`FaultInjector`].
///
/// Ablauf:
/// 1. `injector.on_dir_fsync(dir)` aufrufen.
/// 2. Verzeichnis per [`flush_dir`] synchronisieren.
pub fn sync_dir_with_injector<F: FaultInjector>(dir: &Path, injector: &F) -> std::io::Result<()> {
    injector.on_dir_fsync(dir)?;
    flush_dir(dir)
}

/// RAII-Guard zum automatischen Löschen temporärer Zwischendateien bei Fehlern oder Panics.
struct TmpFileGuard<'a>(&'a Path);

impl<'a> Drop for TmpFileGuard<'a> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0);
    }
}

// ANCHOR[contextra-durable-fs:ATOMIC-REPLACE-001]
/// Ersetzt den Inhalt der Datei unter `path` atomar mit `bytes` unter Verwendung des Standardinjektors [`NoFault`].
pub fn atomic_replace(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    atomic_replace_with_injector(path, bytes, &NoFault)
}

// ANCHOR[contextra-durable-fs:ATOMIC-REPLACE-WITH-INJECTOR]
/// Ersetzt den Inhalt der Datei unter `path` atomar mit `bytes` unter Verwendung eines [`FaultInjector`].
///
/// Ablauf von `atomic_replace`:
/// 1. Elternverzeichnis von `path` ermitteln. Schlägt fehl mit `InvalidInput`, falls kein Elternordner existiert.
/// 2. Eindeutigen temporären Dateinamen im selben Ordner erzeugen (`.<stem>.tmp.<pid>.<count>`).
/// 3. Temporäre Datei exklusiv anlegen (`create_new(true)`) und mit `TmpFileGuard` absichern.
/// 4. Datenblöcke schreiben (`injector.on_write` und `write_all`).
/// 5. Datei synchronisieren (`injector.on_fsync` und [`flush_file`]).
/// 6. Datei schließen und atomar auf Zielpfad umbenennen (`injector.on_rename` und [`rename_durable`]).
/// 7. Elternverzeichnis synchronisieren (`injector.on_dir_fsync`), damit der Verzeichniseintrag stabil ist.
/// 8. `TmpFileGuard` vergessen (`mem::forget`), da die Datei erfolgreich finalisiert wurde.
///
/// Fehlerfälle:
/// - Bricht bei Schreib-, Sync- oder Umbenennungsfehlern sofort ab; die temporäre Datei wird via Drop gelöscht.
pub fn atomic_replace_with_injector<F: FaultInjector>(
    path: &Path,
    bytes: &[u8],
    injector: &F,
) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "target path has no parent directory",
        )
    })?;

    let file_stem = path.file_name().and_then(|s| s.to_str()).unwrap_or("tmp");

    let count = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();

    let tmp_filename = format!(".{}.tmp.{}.{}", file_stem, pid, count);
    let tmp_path = parent.join(tmp_filename);

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_access(7);
    }

    let mut tmp_file = options.open(&tmp_path)?;
    let guard = TmpFileGuard(&tmp_path);

    injector.on_write(Some(&tmp_path), bytes.len())?;
    tmp_file.write_all(bytes)?;

    injector.on_fsync(Some(&tmp_path))?;
    flush_file(&tmp_file, FlushKind::DataAndMetadata)?;
    drop(tmp_file);

    injector.on_rename(&tmp_path, path)?;
    rename_durable(&tmp_path, path)?;

    injector.on_dir_fsync(parent)?;

    std::mem::forget(guard);

    Ok(())
}

// ANCHOR[contextra-durable-fs:DURABLE-REMOVE-001]
/// Entfernt eine Datei dauerhaft und flusht das Elternverzeichnis mit Standardinjektor [`NoFault`].
pub fn durable_remove(path: &Path) -> std::io::Result<()> {
    durable_remove_with_injector(path, &NoFault)
}

// ANCHOR[contextra-durable-fs:DURABLE-REMOVE-WITH-INJECTOR]
/// Entfernt eine Datei dauerhaft und flusht das Elternverzeichnis mit benutzerdefiniertem [`FaultInjector`].
///
/// Ablauf:
/// 1. Elternverzeichnis ermitteln.
/// 2. Datei aus dem Dateisystem löschen (`remove_file`).
/// 3. Verzeichnis-Sync-Hook aufrufen (`injector.on_dir_fsync`).
/// 4. Elternverzeichnis mit [`flush_dir`] auf Festplatte synchronisieren.
pub fn durable_remove_with_injector<F: FaultInjector>(
    path: &Path,
    injector: &F,
) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "target path has no parent directory",
        )
    })?;

    std::fs::remove_file(path)?;
    injector.on_dir_fsync(parent)?;
    flush_dir(parent)?;
    Ok(())
}

// ANCHOR[contextra-durable-fs:SCRUB-RECEIPT-001]
/// Quittung nach erfolgreichem Überschreiben und dauerhaftem Entfernen einer Datei.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrubReceipt {
    /// Gesamtanzahl der vor dem Löschen mit Nullen überschriebenen Bytes.
    pub bytes_scrubbed: u64,
}

// ANCHOR[contextra-durable-fs:SCRUB-AND-REMOVE-001]
/// Überschreibt eine Datei vollständig mit Nullen und entfernt sie dauerhaft (mit [`NoFault`]).
pub fn scrub_and_remove(path: &Path) -> std::io::Result<ScrubReceipt> {
    scrub_and_remove_with_injector(path, &NoFault)
}

// ANCHOR[contextra-durable-fs:SCRUB-AND-REMOVE-WITH-INJECTOR]
/// Überschreibt eine Datei mit Nullen und entfernt sie dauerhaft unter Verwendung eines [`FaultInjector`].
///
/// Ablauf:
/// 1. Metadaten prüfen: Symlinks und Verzeichnisse werden abgewiesen.
/// 2. Datei öffnen und in 64-KiB-Schritten mit Nullen überschreiben (`injector.on_write`).
/// 3. Datei per `injector.on_fsync` und [`flush_file`] auf Festplatte synchronisieren.
/// 4. Datei dauerhaft entfernen und Elternverzeichnis flushen ([`durable_remove_with_injector`]).
/// 5. [`ScrubReceipt`] mit der Anzahl der überschriebenen Bytes zurückgeben.
pub fn scrub_and_remove_with_injector<F: FaultInjector>(
    path: &Path,
    injector: &F,
) -> std::io::Result<ScrubReceipt> {
    let metadata = path.symlink_metadata()?;
    let file_type = metadata.file_type();

    if file_type.is_symlink() || !file_type.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path is a symlink or not a regular file",
        ));
    }

    let len = metadata.len();
    if len > 0 {
        let mut options = OpenOptions::new();
        options.write(true);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_access(7);
        }

        let mut file = options.open(path)?;

        let zeros = [0u8; 64 * 1024];
        let mut remaining = len;

        while remaining > 0 {
            let chunk_size = usize::try_from(remaining)
                .unwrap_or(zeros.len())
                .min(zeros.len());
            injector.on_write(Some(path), chunk_size)?;
            file.write_all(&zeros[..chunk_size])?;
            remaining -= chunk_size as u64;
        }

        injector.on_fsync(Some(path))?;
        flush_file(&file, FlushKind::DataAndMetadata)?;
    }

    durable_remove_with_injector(path, injector)?;

    Ok(ScrubReceipt {
        bytes_scrubbed: len,
    })
}
