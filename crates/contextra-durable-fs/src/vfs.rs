//! # Reales Virtuelles Dateisystem (`RealVfs`)
//!
//! Zweck: Bereitstellung von `RealVfs` als primäre Produktionsimplementierung des kanonischen
//! [`contextra_ports::vfs::Vfs`]-Traits.
//!
//! Ring-Zugehörigkeit: Ring 1 (Dateisystem-Durability).
//!
//! Invarianten:
//! - Atomares Schreiben (`write_atomic`): Temp-Datei schreiben -> `sync_all` -> atomares `rename` -> `sync_dir`.
//! - Alle Fehler von `sync_*`, `write`, `rename` werden unverändert propagiert (kein `let _ =` auf `Result`).
//! - Dateirechte: Verzeichnisse erzwingen Modus `0700`, Datendateien erzwingen Modus `0600`.
//! - Startprobe (`probe_platform_durability`): Verifiziert Plattform-Durability und Verzeichnis-fsync (T-PLAT-1, SPEC §8.5 D-56).

// FILE-CONTEXT
// STAND: 2026-11-02
// ZWECK: Kanonische RealVfs-Implementierung fuer das einheitliche contextra_ports::Vfs-Trait.
// INVARIANTEN: Zero-Panic, Fail-Closed, Zweistufige Haltbarkeit, Rechte 0600/0700.

pub use contextra_ports::vfs::Vfs;
use contextra_ports::{ContextraError, Result};
use contextra_sys::{flush_dir, flush_file, fs_class, rename_durable, FlushKind};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Atomarer Zähler zur Generierung kollisionsfreier temporärer Dateinamen für `RealVfs`.
static VFS_TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

// ANCHOR[contextra-durable-fs:REAL-VFS-001]
/// Produktionsimplementierung von [`contextra_ports::vfs::Vfs`] für reale Dateisysteme.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RealVfs;

/// Alias für Rückwärtskompatibilität.
pub type RealFs = RealVfs;

impl RealVfs {
    /// Erstellt eine neue Instanz von [`RealVfs`].
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Vfs for RealVfs {
    /// Liest den gesamten Dateiinhalt an `path`.
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        if !path.exists() {
            return Err(ContextraError::NotFound(path.display().to_string()));
        }
        fs::read(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ContextraError::NotFound(path.display().to_string())
            } else {
                ContextraError::Io(e)
            }
        })
    }

    /// Schreibt Daten atomar und crash-sicher an `path`.
    ///
    /// Ablauf (SPEC FI-F01):
    /// 1. Elternverzeichnis ermitteln und sicherstellen (erstellt mit Rechten 0700).
    /// 2. Temporäre Datei im selben Verzeichnis anlegen (Rechte 0600).
    /// 3. Daten vollständig schreiben.
    /// 4. `flush_file(FlushKind::DataAndMetadata)` (`fsync`) aufrufen.
    /// 5. Datei schließen und atomar via `rename_durable` auf Zielpfad verschieben.
    /// 6. Elternverzeichnis via `flush_dir` synchronisieren.
    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        let parent = path.parent().ok_or_else(|| {
            ContextraError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "target path has no parent directory",
            ))
        })?;

        if !parent.exists() {
            self.create_dir_all(parent)?;
        }

        let file_stem = path.file_name().and_then(|s| s.to_str()).unwrap_or("tmp");
        let count = VFS_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
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

        let mut tmp_file = options.open(&tmp_path).map_err(ContextraError::Io)?;
        let guard = TmpFileCleanupGuard(&tmp_path);

        // Daten schreiben
        tmp_file.write_all(data).map_err(ContextraError::Io)?;

        // Datei synchronisieren (fsync)
        flush_file(&tmp_file, FlushKind::DataAndMetadata).map_err(ContextraError::Io)?;
        drop(tmp_file);

        // Rechte sicherstellen (0600)
        contextra_sys::set_restrictive_file_acl(&tmp_path).map_err(ContextraError::Io)?;

        // Atomar umbenennen
        rename_durable(&tmp_path, path).map_err(ContextraError::Io)?;

        // Verzeichnis synchronisieren
        flush_dir(parent).map_err(ContextraError::Io)?;

        std::mem::forget(guard);
        Ok(())
    }

    /// Erstellt ein Verzeichnis rekursiv mit restriktiven Rechten (0700 unter Unix).
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path).map_err(ContextraError::Io)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o700))
                .map_err(ContextraError::Io)?;
        }
        Ok(())
    }

    /// Entfernt eine Datei dauerhaft und flusht das Elternverzeichnis.
    fn remove_file(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(ContextraError::NotFound(path.display().to_string()));
        }
        let parent = path.parent();
        fs::remove_file(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ContextraError::NotFound(path.display().to_string())
            } else {
                ContextraError::Io(e)
            }
        })?;

        if let Some(dir) = parent {
            if dir.exists() {
                flush_dir(dir).map_err(ContextraError::Io)?;
            }
        }
        Ok(())
    }

    /// Prüft, ob `path` existiert.
    fn exists(&self, path: &Path) -> Result<bool> {
        Ok(path.exists())
    }

    /// Gibt die Dateilänge in Bytes zurück.
    fn len(&self, path: &Path) -> Result<u64> {
        if !path.exists() {
            return Err(ContextraError::NotFound(path.display().to_string()));
        }
        fs::metadata(path).map(|m| m.len()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ContextraError::NotFound(path.display().to_string())
            } else {
                ContextraError::Io(e)
            }
        })
    }

    /// Listet Dateinamen innerhalb des Verzeichnisses auf.
    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        if !path.exists() {
            return Err(ContextraError::NotFound(path.display().to_string()));
        }
        let entries = fs::read_dir(path).map_err(ContextraError::Io)?;
        let mut names = Vec::new();
        for entry_res in entries {
            let entry = entry_res.map_err(ContextraError::Io)?;
            let file_name = entry.file_name();
            names.push(file_name.to_string_lossy().to_string());
        }
        names.sort();
        Ok(names)
    }

    /// Synchronisiert die Verzeichnismetadaten dauerhaft auf Festplatte.
    fn sync_dir(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(ContextraError::NotFound(path.display().to_string()));
        }
        flush_dir(path).map_err(ContextraError::Io)
    }
}

/// RAII-Guard zum automatischen Löschen temporärer Zwischendateien bei Fehlern.
struct TmpFileCleanupGuard<'a>(&'a Path);

impl<'a> Drop for TmpFileCleanupGuard<'a> {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_file(self.0) {
            if e.kind() != std::io::ErrorKind::NotFound {
                // Best-effort cleanup inside drop without panic or swallowed unexamined result
                drop(e);
            }
        }
    }
}

// ANCHOR[contextra-durable-fs:PROBE-PLATFORM-DURABILITY-001]
/// Startprobe für Plattform-Durability und Verzeichnis-fsync (T-PLAT-1, SPEC §8.5 D-56).
///
/// Ablauf:
/// 1. Dateisystem-Klasse via [`fs_class`] ermitteln. Nicht unterstützte (Netzwerk, FUSE, CloudSync) abweisen.
/// 2. Testverzeichnis mit Rechten 0700 erstellen.
/// 3. Testdatei mit Rechten 0600 schreiben und `fsync` ausführen.
/// 4. Atomares Rename durchführen.
/// 5. Verzeichnis-`fsync` ausführen.
/// 6. Berechtigungen verifizieren.
/// 7. Testartefakte rückstandslos bereinigen.
pub fn probe_platform_durability(base_dir: &Path) -> Result<()> {
    let class = fs_class(base_dir).map_err(ContextraError::Io)?;
    if !class.is_supported_local() {
        return Err(ContextraError::Storage(format!(
            "unsupported filesystem class for durability: {:?}",
            class
        )));
    }

    let probe_dir = base_dir.join(format!(
        ".contextra_durability_probe_{}",
        std::process::id()
    ));
    fs::create_dir_all(&probe_dir).map_err(ContextraError::Io)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&probe_dir, fs::Permissions::from_mode(0o700))
            .map_err(ContextraError::Io)?;
    }

    let probe_file = probe_dir.join("probe_init.dat");
    let target_file = probe_dir.join("probe_target.dat");

    let vfs = RealVfs::new();
    vfs.write_atomic(&probe_file, b"DURABILITY_PROBE_PAYLOAD")?;

    // Rename-Test
    rename_durable(&probe_file, &target_file).map_err(ContextraError::Io)?;
    flush_dir(&probe_dir).map_err(ContextraError::Io)?;

    // Verifikation der Leseoperation
    let read_back = vfs.read(&target_file)?;
    if read_back != b"DURABILITY_PROBE_PAYLOAD" {
        fs::remove_dir_all(&probe_dir).map_err(ContextraError::Io)?;
        return Err(ContextraError::Storage(
            "durability probe readback mismatch".to_string(),
        ));
    }

    // Berechtigungen verifizieren (0600 für Datei, 0700 für Verzeichnis)
    contextra_sys::verify_file_acl_owner_only(&target_file).map_err(ContextraError::Io)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let dir_meta = fs::metadata(&probe_dir).map_err(ContextraError::Io)?;
        let mode = dir_meta.mode() & 0o777;
        if mode != 0o700 {
            fs::remove_dir_all(&probe_dir).map_err(ContextraError::Io)?;
            return Err(ContextraError::Storage(format!(
                "durability probe directory permissions mismatch: expected 0700, got 0{:o}",
                mode
            )));
        }
    }

    // Bereinigung
    fs::remove_dir_all(&probe_dir).map_err(ContextraError::Io)?;
    Ok(())
}
