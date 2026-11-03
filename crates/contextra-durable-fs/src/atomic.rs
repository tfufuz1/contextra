//! # Atomare Dateioperationen und temporäre Bereinigung (`atomic`)
//!
//! Zweck: Implementierung crash-sicherer Datei-Austauschoperationen (`write_atomic`)
//! und Bereinigung verwaister temporärer Dateien (`clean_tmp`) gemäß SPEC DF-02, DF-08, DF-12.
//!
//! Ring-Zugehörigkeit: Ring 1 (Dateisystem-Durability).
//! Wichtigste Typen und Funktionen:
//! - [`write_atomic`]: Schreibt Zieldateien atomar über eine temporäre Datei im selben Verzeichnis.
//! - [`write_atomic_with_injector`]: Atomares Schreiben mit Injektionspunkten für Crash-Matrix-Prüfungen.
//! - [`clean_tmp`]: Leert `tmp/`-Ordner und entfernt temporäre Waisendateien beim Start.
//!
//! Invarianten:
//! - `INV-ATOMIC-1`: Zieldatei ist niemals im halb geschriebenen Zustand sichtbar.
//! - `INV-PERM-1`: Neu angelegte Dateien erhalten Modus `0600`, Verzeichnisse Modus `0700`.
//! - `INV-CLEAN-1`: Fehler in Teilschritten werden unverändert propagiert; temporäre Dateien werden abgeräumt.
//! - Zero-Panic: Feste Fehlermeldungen ohne sensible Pfade im Display.

use contextra_sys::{flush_dir, flush_file, rename_durable, set_restrictive_file_acl, FlushKind};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Globaler atomarer Zähler zur Generierung kollisionsfreier temporärer Dateinamen.
static ATOMIC_TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Schnittstelle für Fehlerinjektionen bei atomaren Schreib- und Austauschvorgängen.
///
/// Ermöglicht Unterbrechungen an jedem einzelnen Schritt des atomaren Austauschs:
/// Vorbereitung, Schreiben, Daten-Sync (`sync_data`), Umbenennen (`rename`), Verzeichnis-Sync (`sync_dir`).
pub trait AtomicFaultInjector: Send + Sync {
    /// Hook vor dem Schreiben von Datenblöcken.
    fn on_write(&self, _path: &Path, _bytes: usize) -> std::io::Result<()> {
        Ok(())
    }

    /// Hook vor der synchronen Datenspeicherung (`sync_data` / `fdatasync`).
    fn on_sync_data(&self, _path: &Path) -> std::io::Result<()> {
        Ok(())
    }

    /// Hook vor der atomaren Dateiumbenennung (`rename`).
    fn on_rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
        Ok(())
    }

    /// Hook vor der Synchronisation des Elternverzeichnisses (`sync_dir`).
    fn on_sync_dir(&self, _dir: &Path) -> std::io::Result<()> {
        Ok(())
    }

    /// Hook vor dem Entfernen einer Datei (`unlink`).
    fn on_unlink(&self, _path: &Path) -> std::io::Result<()> {
        Ok(())
    }
}

/// Standard-Injektor ohne Fehlerinjektion für den Produktivbetrieb.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoFault;

impl AtomicFaultInjector for NoFault {
    #[inline(always)]
    fn on_write(&self, _path: &Path, _bytes: usize) -> std::io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn on_sync_data(&self, _path: &Path) -> std::io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn on_rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn on_sync_dir(&self, _dir: &Path) -> std::io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn on_unlink(&self, _path: &Path) -> std::io::Result<()> {
        Ok(())
    }
}

impl<T: AtomicFaultInjector + ?Sized> AtomicFaultInjector for &T {
    #[inline]
    fn on_write(&self, path: &Path, bytes: usize) -> std::io::Result<()> {
        (**self).on_write(path, bytes)
    }

    #[inline]
    fn on_sync_data(&self, path: &Path) -> std::io::Result<()> {
        (**self).on_sync_data(path)
    }

    #[inline]
    fn on_rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        (**self).on_rename(from, to)
    }

    #[inline]
    fn on_sync_dir(&self, dir: &Path) -> std::io::Result<()> {
        (**self).on_sync_dir(dir)
    }

    #[inline]
    fn on_unlink(&self, path: &Path) -> std::io::Result<()> {
        (**self).on_unlink(path)
    }
}

/// RAII-Guard zum automatischen Entfernen temporärer Zwischendateien bei Fehlern oder Abbrüchen.
struct TmpFileGuard<'a> {
    path: &'a Path,
    active: bool,
}

impl<'a> TmpFileGuard<'a> {
    fn new(path: &'a Path) -> Self {
        Self { path, active: true }
    }

    fn disarm(&mut self) {
        self.active = false;
    }
}

impl<'a> Drop for TmpFileGuard<'a> {
    fn drop(&mut self) {
        if self.active {
            let _ = fs::remove_file(self.path);
        }
    }
}

/// Erstellt ein Verzeichnis rekursiv mit restriktivem Berechtigungsmodus `0700` (DF-08).
pub fn create_dir_all_0700(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

/// Schreibt Daten atomar und crash-sicher an den Zielpfad `path` (DF-02, DF-08, DF-12).
///
/// Ablauf:
/// 1. Elternverzeichnis ermitteln. Schlägt fehl mit `InvalidInput`, falls kein Elternverzeichnis existiert.
/// 2. Temporäre Zwischendatei im selben Verzeichnis im Modus `0600` anlegen.
/// 3. Daten schreiben und via `sync_data` (`FlushKind::DataOnly`) synchronisieren.
/// 4. Datei schließen und atomar via `rename_durable` auf den Zielpfad umbenennen.
/// 5. Elternverzeichnis via `flush_dir` synchronisieren.
/// 6. Jeder Einzelschritt meldet Fehler unverändert zurück.
pub fn write_atomic(path: &Path, data: &[u8]) -> std::io::Result<()> {
    write_atomic_with_injector(path, data, &NoFault)
}

/// Schreibt Daten atomar und crash-sicher unter Verwendung eines [`AtomicFaultInjector`].
///
/// Ermöglicht deterministische Fehlerinjektion an allen Phasen des Austauschs (T-CRASH-2).
pub fn write_atomic_with_injector<I: AtomicFaultInjector>(
    path: &Path,
    data: &[u8],
    injector: &I,
) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "target path has no parent directory",
        )
    })?;

    if !parent.exists() {
        create_dir_all_0700(parent)?;
    }

    let file_stem = path.file_name().and_then(|s| s.to_str()).unwrap_or("tmp");
    let count = ATOMIC_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let tmp_filename = format!(".{}.tmp.{}.{}", file_stem, pid, count);
    let tmp_path = parent.join(tmp_filename);

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_access(7);
    }

    let mut tmp_file = options.open(&tmp_path)?;
    let mut guard = TmpFileGuard::new(&tmp_path);

    // Rechte verifizieren bzw. setzen (0600 für Datei)
    set_restrictive_file_acl(&tmp_path)?;

    // Schritt 1: Schreiben
    injector.on_write(&tmp_path, data.len())?;
    tmp_file.write_all(data)?;

    // Schritt 2: sync_data
    injector.on_sync_data(&tmp_path)?;
    flush_file(&tmp_file, FlushKind::DataOnly)?;
    drop(tmp_file);

    // Schritt 3: atomarer Tausch
    injector.on_rename(&tmp_path, path)?;
    rename_durable(&tmp_path, path)?;

    // Schritt 4: Verzeichnis synchronisieren
    injector.on_sync_dir(parent)?;
    flush_dir(parent)?;

    guard.disarm();
    Ok(())
}

/// Bereinigt das `tmp/`-Unterverzeichnis sowie temporäre Waisendateien beim Start (DF-08, T-CRASH-2, T-CRASH-6).
///
/// Ablauf:
/// 1. Prüft, ob `dir.join("tmp")` existiert. Wenn ja, werden alle darin liegenden Einträge dauerhaft gelöscht.
/// 2. Durchsucht das Verzeichnis `dir` nach verwaisten temporären Dateien (Dateinamen beginnend mit `.` und enthaltend `.tmp.`).
/// 3. Nach dem Löschen verwaister Dateien wird das Verzeichnis synchronisiert (`flush_dir`).
/// 4. Liefert die Anzahl bereinigter Waiseneinträge zurück.
pub fn clean_tmp(dir: &Path) -> std::io::Result<usize> {
    clean_tmp_with_injector(dir, &NoFault)
}

/// Bereinigt temporäre Waisendateien mit Unterstützung eines [`AtomicFaultInjector`].
pub fn clean_tmp_with_injector<I: AtomicFaultInjector>(
    dir: &Path,
    injector: &I,
) -> std::io::Result<usize> {
    if !dir.exists() {
        return Ok(0);
    }

    let mut removed_count = 0usize;

    // 1. Spezielles Unterverzeichnis "tmp" bereinigen, falls vorhanden
    let tmp_subdir = dir.join("tmp");
    if tmp_subdir.is_dir() {
        let read_dir = fs::read_dir(&tmp_subdir)?;
        for entry_res in read_dir {
            let entry = entry_res?;
            let entry_path = entry.path();
            injector.on_unlink(&entry_path)?;
            if entry_path.is_dir() {
                fs::remove_dir_all(&entry_path)?;
            } else {
                fs::remove_file(&entry_path)?;
            }
            removed_count = removed_count.saturating_add(1);
        }
        injector.on_sync_dir(&tmp_subdir)?;
        flush_dir(&tmp_subdir)?;
    }

    // 2. Temporäre Waisendateien im Basisverzeichnis selbst suchen (Muster .*.tmp.*)
    let read_dir = fs::read_dir(dir)?;
    for entry_res in read_dir {
        let entry = entry_res?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();

        let is_orphan_tmp = (name_str.starts_with('.') && name_str.contains(".tmp."))
            || (name_str.starts_with("tmp.") && !entry.path().is_dir());

        if is_orphan_tmp {
            let entry_path = entry.path();
            injector.on_unlink(&entry_path)?;
            fs::remove_file(&entry_path)?;
            removed_count = removed_count.saturating_add(1);
        }
    }

    if removed_count > 0 {
        injector.on_sync_dir(dir)?;
        flush_dir(dir)?;
    }

    Ok(removed_count)
}
