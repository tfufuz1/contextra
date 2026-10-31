//! # Virtuelles Dateisystem (VFS - Virtual File System)
//!
//! Zweck: Bereitstellung einer Schnittstellenabstraktion für Dateisystemoperationen,
//! um positionsbasiertes I/O (`read_at`, `write_at`), Dateisynchronisation (`sync_data`, `sync_all`)
//! und Verzeichnisverwaltung testbar und mockbar zu machen.
//!
//! Ring-Zugehörigkeit: Ring 0 (I/O- und Speicherschnittstellen).
//! Wird genutzt von: Index- und Speicher-Engines (z. B. HNSW-Vektorindex, Blockmanager).
//! Wichtigste Typen:
//! - [`Vfs`]: Der abstrakte Trait für Dateisystemaktionen (Öffnen, Löschen, Umbenennen).
//! - [`VfsFile`]: Der Trait für positionsabhängige Lese- und Schreibzugriffe auf offene Dateien.
//! - [`VfsOpenOptions`]: Parameter zum Öffnen von Dateien (Lesen, Schreiben, Erstellen, Kürzen).
//! - [`RealFs`]: Standardimplementierung, die Aufrufe an das reale Host-Dateisystem weiterleitet.
//! - [`RealFsFile`]: Dateihandle-Wrapper für [`std::fs::File`] mit plattformspezifischen Erweiterungen.
//!
//! Invarianten:
//! - Schnittstellenvertrag: Deterministische I/O-Fehlerweiterleitung ohne `panic!`.
//! - Plattformunabhängigkeit: Verwendet auf Unix `FileExt::read_at`/`write_at` und auf Windows `FileExt::seek_read`/`seek_write`.

// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Virtual File System (VFS) Abstraktion fuer deterministische I/O-Injektion und Testbarkeit.
// INVARIANTEN: Zero-Panic, Fail-Closed, SCHNITTSTELLENVERTRAG exakt eingehalten.

use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::FileExt;

#[cfg(windows)]
use std::os::windows::fs::FileExt;

// ANCHOR[contextra-durable-fs:VFS-OPTIONS-001]
/// Optionen zum Öffnen von Dateien über das virtuelle Dateisystem ([`Vfs::open`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VfsOpenOptions {
    /// Öffnet die Datei mit Leserechten.
    pub read: bool,
    /// Öffnet die Datei mit Schreibrechten.
    pub write: bool,
    /// Erstellt eine neue Datei, falls sie noch nicht existiert.
    pub create: bool,
    /// Erstellt eine neue Datei und schlägt fehl, falls sie bereits existiert (`create_new`).
    pub create_new: bool,
    /// Setzt die Dateilänge beim Öffnen sofort auf 0 zurück (`truncate`).
    pub truncate: bool,
    /// Hängt neue Schreiboperationen immer an das Dateiende an (`append`).
    pub append: bool,
}

// ANCHOR[contextra-durable-fs:VFS-TRAIT-001]
/// Abstrakte Schnittstelle für Dateisysteme zur Entkopplung von Speicherkomponenten.
pub trait Vfs: Send + Sync + 'static {
    // ANCHOR[contextra-durable-fs:VFS-OPEN]
    /// Öffnet eine Datei gemäß den übergebenen Optionen.
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> std::io::Result<Box<dyn VfsFile>>;
    // ANCHOR[contextra-durable-fs:VFS-CREATE-DIR-ALL]
    /// Erstellt das angegebene Verzeichnis inklusive aller übergeordneten Ordner.
    fn create_dir_all(&self, path: &Path) -> std::io::Result<()>;
    // ANCHOR[contextra-durable-fs:VFS-REMOVE-FILE]
    /// Entfernt eine Datei dauerhaft aus dem Dateisystem.
    fn remove_file(&self, path: &Path) -> std::io::Result<()>;
    // ANCHOR[contextra-durable-fs:VFS-RENAME]
    /// Benennt eine Datei atomar von `from` nach `to` um.
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()>;
    // ANCHOR[contextra-durable-fs:VFS-READ-DIR]
    /// Liest alle Einträge eines Verzeichnisses aus.
    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<PathBuf>>;
    // ANCHOR[contextra-durable-fs:VFS-EXISTS]
    /// Prüft, ob ein Pfad (Datei oder Ordner) im Dateisystem existiert.
    fn exists(&self, path: &Path) -> std::io::Result<bool>;
    // ANCHOR[contextra-durable-fs:VFS-SYNC-DIR]
    /// Synchronisiert die Verzeichnismetadaten auf stabile Speichermedien.
    fn sync_dir(&self, path: &Path) -> std::io::Result<()>;
}

// ANCHOR[contextra-durable-fs:VFS-FILE-TRAIT-001]
/// Abstraktes Dateihandle mit Unterstützung für positionsbasiertes I/O (Offset-Zugriff).
pub trait VfsFile: Send + Sync {
    // ANCHOR[contextra-durable-fs:VFS-FILE-READ-AT]
    /// Liest Daten ab einem festen Byte-Offset `offset` in den Puffer `buf`, ohne den Dateizeiger zu verändern.
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;
    // ANCHOR[contextra-durable-fs:VFS-FILE-WRITE-AT]
    /// Schreibt Daten aus `buf` an den festen Byte-Offset `offset`, ohne den Dateizeiger zu verändern.
    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize>;
    // ANCHOR[contextra-durable-fs:VFS-FILE-SET-LEN]
    /// Ändert die physische Größe der Datei (Kürzen oder Erweitern).
    fn set_len(&self, len: u64) -> std::io::Result<()>;
    // ANCHOR[contextra-durable-fs:VFS-FILE-LEN]
    /// Fragt die aktuelle Dateigröße in Bytes ab.
    fn len(&self) -> std::io::Result<u64>;
    // ANCHOR[contextra-durable-fs:VFS-FILE-IS-EMPTY]
    /// Prüft, ob die Datei 0 Bytes groß ist.
    fn is_empty(&self) -> std::io::Result<bool> {
        Ok(self.len()? == 0)
    }
    // ANCHOR[contextra-durable-fs:VFS-FILE-SYNC-DATA]
    /// Synchronisiert die geschriebenen Nutzdatenblöcke auf die Festplatte (`fdatasync`).
    fn sync_data(&self) -> std::io::Result<()>;
    // ANCHOR[contextra-durable-fs:VFS-FILE-SYNC-ALL]
    /// Synchronisiert Nutzdaten und alle Dateimetadaten auf die Festplatte (`fsync`).
    fn sync_all(&self) -> std::io::Result<()>;
}

// ANCHOR[contextra-durable-fs:REAL-FS-001]
/// Standard-Implementierung des [`Vfs`], die direkt mit dem Host-Betriebssystem interagiert.
#[derive(Debug, Clone, Copy, Default)]
pub struct RealFs;

impl Vfs for RealFs {
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> std::io::Result<Box<dyn VfsFile>> {
        let mut std_opts = OpenOptions::new();
        std_opts
            .read(opts.read)
            .write(opts.write)
            .create(opts.create)
            .create_new(opts.create_new)
            .truncate(opts.truncate)
            .append(opts.append);

        let file = std_opts.open(path)?;
        Ok(Box::new(RealFsFile { file }))
    }

    fn create_dir_all(&self, path: &Path) -> std::io::Result<()> {
        fs::create_dir_all(path)
    }

    fn remove_file(&self, path: &Path) -> std::io::Result<()> {
        crate::durable_remove(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        // NOTE: Verwendet rename_durable aus contextra-sys zur dauerhaften Umbenennung.
        contextra_sys::rename_durable(from, to)?;
        if let Some(parent) = to.parent() {
            let _ = crate::sync_dir(parent);
        }
        Ok(())
    }

    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
        let entries = fs::read_dir(path)?;
        let mut res = Vec::new();
        for entry in entries {
            res.push(entry?.path());
        }
        Ok(res)
    }

    fn exists(&self, path: &Path) -> std::io::Result<bool> {
        Ok(path.exists())
    }

    fn sync_dir(&self, path: &Path) -> std::io::Result<()> {
        crate::sync_dir(path)
    }
}

// ANCHOR[contextra-durable-fs:REAL-FS-FILE-001]
/// Implementierung von [`VfsFile`] auf Basis von [`std::fs::File`].
#[derive(Debug)]
pub struct RealFsFile {
    file: File,
}

impl VfsFile for RealFsFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        #[cfg(unix)]
        {
            FileExt::read_at(&self.file, buf, offset)
        }
        #[cfg(windows)]
        {
            FileExt::seek_read(&self.file, buf, offset)
        }
        #[cfg(not(any(unix, windows)))]
        {
            compile_error!("Unsupported target OS for positional VfsFile");
        }
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
        #[cfg(unix)]
        {
            FileExt::write_at(&self.file, buf, offset)
        }
        #[cfg(windows)]
        {
            FileExt::seek_write(&self.file, buf, offset)
        }
        #[cfg(not(any(unix, windows)))]
        {
            compile_error!("Unsupported target OS for positional VfsFile");
        }
    }

    fn set_len(&self, len: u64) -> std::io::Result<()> {
        self.file.set_len(len)
    }

    fn len(&self) -> std::io::Result<u64> {
        self.file.metadata().map(|m| m.len())
    }

    fn sync_data(&self) -> std::io::Result<()> {
        self.file.sync_data()
    }

    fn sync_all(&self) -> std::io::Result<()> {
        self.file.sync_all()
    }
}
