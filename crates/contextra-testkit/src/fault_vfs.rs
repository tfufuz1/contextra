//! # Virtuelles Dateisystem mit deterministischer Fehlerinjektion (`fault_vfs`)
//!
//! Dieses Modul stellt mit [`FaultVfs`] ein In-Memory-Dateisystem bereit, das I/O-Fehler,
//! Abstürze und `fsync`-Verweigerungen gezielt nach einer konfigurierbaren Anzahl von
//! Operationen injizieren kann.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Port:** Implementiert [`contextra_ports::Vfs`].
//! - **Regel 5 (SPEC §3.2):** I/O nur über Vfs; `fsync`-Fehler müssen strikt propagiert werden.
//! - **Regel 10 (SPEC §3.2):** Fail-Closed: Schlägt ein I/O- oder sync-Aufruf fehl, darf das
//!   System keinen Teilerfolg vortäuschen oder in einen inkonsistenten Zustand verfallen.
//!
//! ## Funktionsweise der Fehlerinjektion
//! Über [`FaultConfig`] wird konfiguriert, nach wie vielen erfolgreichen Schreib-,
//! Lese-, Sync- oder Rename-Operationen ein synthetischer I/O-Fehler ausgelöst wird:
//! - `fail_writes_after`: Gibt nach N erfolgreichen Schreibvorgängen einen Fehler zurück.
//! - `fail_syncs_after`: Simuliert fehlschlagende `fsync`-Befehle des Betriebssystems.
//! - `fail_reads_after`: Simuliert Lesefehler von defekten Blöcken.
//! - `fail_renames_after`: Simuliert Fehler beim atomaren Umbenennen.
//! - `fail_at_operation`: Gezielt bei `(FaultOperation, index)` fehlschlagen.
//! - `crashed`: Simuliert einen sofortigen Systemabsturz (Power-Cut).

// FILE-CONTEXT
// STAND: 2026-11-01
// ZWECK: Deterministic I/O error injector and Virtual File System test utility.
// INVARIANTEN: Zero real data corruption, deterministic fault triggers based on operation count.

use contextra_ports::Vfs;
use contextra_types::{ContextraError, Result};
use parking_lot::RwLock;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

// ANCHOR[contextra-testkit:FAULT-OPERATION-ENUM-001]
/// Klassifikation der I/O-Operationsarten für gezielte Fehlerinjektion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FaultOperation {
    /// Lesezugriff auf Dateiinhalte (`read`).
    Read,
    /// Schreibzugriff (`write_atomic` / `write_file`).
    Write,
    /// Synchronisationsaufruf (`sync`, `sync_dir`).
    Sync,
    /// Umbenennen (`rename`).
    Rename,
    /// Löschen einer Datei (`remove_file`).
    Remove,
    /// Verzeichnisanlage (`create_dir_all`).
    CreateDir,
}

// ANCHOR[contextra-testkit:FAULT-CONFIG-STRUCT-001]
/// Konfiguration für deterministische Fehlerinjektion in [`FaultVfs`].
///
/// **Zweck:** Ermöglicht Test-Cases die exakte Vorgabe von Fehlerbedingungen,
/// beispielsweise um Crash-Recovery nach dem N-ten WAL-Schreibvorgang zu validieren.
#[derive(Debug, Clone, Default)]
pub struct FaultConfig {
    /// Schreiboperationen nach dieser Anzahl erfolgreicher Schreibvorgänge fehlschlagen lassen.
    pub fail_writes_after: Option<usize>,
    /// Sync- bzw. Flush-Operationen (`fsync`) nach dieser Anzahl erfolgreicher Aufrufe fehlschlagen lassen.
    pub fail_syncs_after: Option<usize>,
    /// Leseoperationen nach dieser Anzahl erfolgreicher Lesevorgänge fehlschlagen lassen.
    pub fail_reads_after: Option<usize>,
    /// Umbenennungen nach dieser Anzahl erfolgreicher Aufrufe fehlschlagen lassen.
    pub fail_renames_after: Option<usize>,
    /// Gezielt bei einer bestimmten Operation und einem bestimmten Null-basierten Index fehlschlagen.
    pub fail_at_operation: Option<(FaultOperation, usize)>,
    /// Globaler Notschalter: Alle Operationen schlagen sofort fehl.
    pub fail_all: bool,
}

// ANCHOR[contextra-testkit:FAULT-VFS-STRUCT-001]
/// Simuliertes In-Memory-Dateisystem mit deterministischer Fehlerinjektion.
///
/// **Zweck:** Dient als I/O-Testunterlage für Speichermodule (z. B. WAL und LSM-Storage),
/// um Fehlertoleranz und Absturzsicherheit ohne reale Festplattenfehler zu testen.
///
/// **Nebenläufigkeit:**
/// - Dateiinhalte liegen geschützt in einem [`RwLock<BTreeMap<PathBuf, Vec<u8>>>`].
/// - Verzeichnisstruktur liegt in einem [`RwLock<BTreeSet<PathBuf>>`].
/// - Die Operationszähler sind atomare [`AtomicUsize`].
#[derive(Debug, Clone, Default)]
pub struct FaultVfs {
    /// In-Memory Dateispeicher: Pfad -> Dateibytes.
    files: Arc<RwLock<BTreeMap<PathBuf, Vec<u8>>>>,
    /// In-Memory Verzeichnisbaum.
    dirs: Arc<RwLock<BTreeSet<PathBuf>>>,
    /// Anzahl der versuchten Leseoperationen.
    read_count: Arc<AtomicUsize>,
    /// Anzahl der versuchten Schreiboperationen.
    write_count: Arc<AtomicUsize>,
    /// Anzahl der versuchten Sync-Operationen.
    sync_count: Arc<AtomicUsize>,
    /// Anzahl der versuchten Rename-Operationen.
    rename_count: Arc<AtomicUsize>,
    /// Anzahl der versuchten Remove-Operationen.
    remove_count: Arc<AtomicUsize>,
    /// Anzahl der versuchten CreateDir-Operationen.
    create_dir_count: Arc<AtomicUsize>,
    /// Aktuelle Injektionskonfiguration.
    config: Arc<RwLock<FaultConfig>>,
    /// Flag für simulierten Totalabsturz.
    crashed: Arc<AtomicBool>,
}

impl FaultVfs {
    // ANCHOR[contextra-testkit:FAULT-VFS-NEW-001]
    /// Erstellt eine neue, leere [`FaultVfs`]-Instanz ohne aktive Fehlerregeln.
    pub fn new() -> Self {
        Self::default()
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-SET-CONFIG-001]
    /// Aktualisiert die aktive Konfiguration für Fehlerinjektion.
    pub fn set_config(&self, config: FaultConfig) {
        *self.config.write() = config;
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-TRIGGER-CRASH-001]
    /// Simuliert einen abrupten Systemabsturz (Power-Cut).
    pub fn trigger_crash(&self) {
        self.crashed.store(true, Ordering::SeqCst);
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-RESET-COUNTERS-001]
    /// Setzt die Operationszähler und den Crash-Status zurück.
    ///
    /// Die bestehenden Dateiinhalte bleiben unverändert erhalten.
    pub fn reset_counters(&self) {
        self.read_count.store(0, Ordering::SeqCst);
        self.write_count.store(0, Ordering::SeqCst);
        self.sync_count.store(0, Ordering::SeqCst);
        self.rename_count.store(0, Ordering::SeqCst);
        self.remove_count.store(0, Ordering::SeqCst);
        self.create_dir_count.store(0, Ordering::SeqCst);
        self.crashed.store(false, Ordering::SeqCst);
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-CHECK-FAULT-001]
    /// Interne Überprüfung, ob für die gegebene Operation ein Fehler injiziert werden soll.
    fn check_fault(&self, op: FaultOperation, counter: &AtomicUsize) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "VFS simulated crash"));
        }
        let cfg = self.config.read();
        if cfg.fail_all {
            return Err(io::Error::other("Simulated injected failure (fail_all)"));
        }

        let idx = counter.fetch_add(1, Ordering::SeqCst);

        if let Some((target_op, target_idx)) = cfg.fail_at_operation {
            if target_op == op && target_idx == idx {
                return Err(io::Error::other(format!(
                    "Simulated failure at {op:?} operation index {idx}"
                )));
            }
        }

        match op {
            FaultOperation::Write => {
                if let Some(limit) = cfg.fail_writes_after {
                    if idx >= limit {
                        return Err(io::Error::new(
                            ErrorKind::WriteZero,
                            "Simulated disk write failure",
                        ));
                    }
                }
            }
            FaultOperation::Read => {
                if let Some(limit) = cfg.fail_reads_after {
                    if idx >= limit {
                        return Err(io::Error::new(
                            ErrorKind::UnexpectedEof,
                            "Simulated disk read failure",
                        ));
                    }
                }
            }
            FaultOperation::Sync => {
                if let Some(limit) = cfg.fail_syncs_after {
                    if idx >= limit {
                        return Err(io::Error::other("Simulated fsync I/O failure"));
                    }
                }
            }
            FaultOperation::Rename => {
                if let Some(limit) = cfg.fail_renames_after {
                    if idx >= limit {
                        return Err(io::Error::other("Simulated rename failure"));
                    }
                }
            }
            FaultOperation::Remove | FaultOperation::CreateDir => {}
        }

        Ok(())
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-WRITE-FILE-001]
    /// Schreibt Bytes an den angegebenen Pfad unter Berücksichtigung der Fehlerrichtlinien.
    pub fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        self.check_fault(FaultOperation::Write, &self.write_count)?;
        if let Some(parent) = path.parent() {
            self.create_dir_all_internal(parent)?;
        }
        self.files.write().insert(path.to_path_buf(), data.to_vec());
        Ok(())
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-READ-FILE-001]
    /// Liest Bytes vom angegebenen Pfad unter Berücksichtigung der Fehlerrichtlinien.
    pub fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.check_fault(FaultOperation::Read, &self.read_count)?;
        self.files
            .read()
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "File not found in FaultVfs"))
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-SYNC-001]
    /// Simuliert einen `fsync`-Aufruf zur dauerhaften Bestätigung auf dem Speichermedium.
    pub fn sync(&self) -> io::Result<()> {
        self.check_fault(FaultOperation::Sync, &self.sync_count)
    }

    fn create_dir_all_internal(&self, path: &Path) -> io::Result<()> {
        let mut dirs = self.dirs.write();
        let mut cur = path.to_path_buf();
        while !cur.as_os_str().is_empty() {
            dirs.insert(cur.clone());
            if !cur.pop() {
                break;
            }
        }
        Ok(())
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-COUNTS-001]
    /// Liefert die Anzahl der bisher versuchten Schreiboperationen.
    pub fn write_count(&self) -> usize {
        self.write_count.load(Ordering::SeqCst)
    }

    /// Liefert die Anzahl der bisher versuchten Leseoperationen.
    pub fn read_count(&self) -> usize {
        self.read_count.load(Ordering::SeqCst)
    }

    /// Liefert die Anzahl der bisher versuchten Sync-Operationen.
    pub fn sync_count(&self) -> usize {
        self.sync_count.load(Ordering::SeqCst)
    }

    /// Liefert die Gesamtanzahl aller versuchten I/O-Operationen.
    pub fn total_operations(&self) -> usize {
        self.read_count.load(Ordering::SeqCst)
            + self.write_count.load(Ordering::SeqCst)
            + self.sync_count.load(Ordering::SeqCst)
            + self.rename_count.load(Ordering::SeqCst)
            + self.remove_count.load(Ordering::SeqCst)
            + self.create_dir_count.load(Ordering::SeqCst)
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-CRASH-MATRIX-001]
    /// Führt einen Testblock für jeden sequentiellen Crash-Punkt (Absturz nach $N$ Operationen) aus.
    ///
    /// **Ablauf:**
    /// 1. Führt den Workload zuerst unbeeinflusst aus, um die Gesamtzahl an Operationen $M$ zu ermitteln.
    /// 2. Iteriert für jeden Punkt $i \in [0, M]$ durch:
    ///    - Erstellt eine frische `FaultVfs`-Instanz mit Fehler bei Operation $i$.
    ///    - Führt die Test-Closure `test_fn` aus.
    pub fn for_each_crash_point<F, R>(mut setup_and_run: F) -> Vec<(usize, R)>
    where
        F: FnMut(&FaultVfs, Option<usize>) -> R,
    {
        // 1. Baseline unconstrained run to determine max operations
        let probe_vfs = FaultVfs::new();
        let _ = setup_and_run(&probe_vfs, None);
        let total_ops = probe_vfs.total_operations();

        let mut results = Vec::with_capacity(total_ops + 1);

        // 2. Iterate each crash point
        for crash_at in 0..=total_ops {
            let vfs = FaultVfs::new();
            vfs.set_config(FaultConfig {
                fail_writes_after: Some(crash_at),
                fail_syncs_after: Some(crash_at),
                fail_renames_after: Some(crash_at),
                ..Default::default()
            });
            let outcome = setup_and_run(&vfs, Some(crash_at));
            results.push((crash_at, outcome));
        }

        results
    }
}

// ANCHOR[contextra-testkit:FAULT-VFS-TRAIT-IMPL-001]
impl Vfs for FaultVfs {
    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        self.check_fault(FaultOperation::Read, &self.read_count)
            .map_err(ContextraError::Io)?;
        self.files
            .read()
            .get(path)
            .cloned()
            .ok_or_else(|| ContextraError::NotFound(path.display().to_string()))
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        // Atomic write contract: write to temporary, sync, rename, sync_dir
        self.check_fault(FaultOperation::Write, &self.write_count)
            .map_err(ContextraError::Io)?;

        if let Some(parent) = path.parent() {
            self.create_dir_all(parent)?;
        }

        self.check_fault(FaultOperation::Sync, &self.sync_count)
            .map_err(ContextraError::Io)?;

        self.check_fault(FaultOperation::Rename, &self.rename_count)
            .map_err(ContextraError::Io)?;

        self.files.write().insert(path.to_path_buf(), data.to_vec());

        if let Some(parent) = path.parent() {
            self.sync_dir(parent)?;
        }

        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        self.check_fault(FaultOperation::CreateDir, &self.create_dir_count)
            .map_err(ContextraError::Io)?;
        self.create_dir_all_internal(path)
            .map_err(ContextraError::Io)
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        self.check_fault(FaultOperation::Remove, &self.remove_count)
            .map_err(ContextraError::Io)?;
        let mut files = self.files.write();
        if files.remove(path).is_some() {
            Ok(())
        } else {
            Err(ContextraError::NotFound(path.display().to_string()))
        }
    }

    fn exists(&self, path: &Path) -> Result<bool> {
        let files = self.files.read();
        if files.contains_key(path) {
            return Ok(true);
        }
        let dirs = self.dirs.read();
        Ok(dirs.contains(path))
    }

    fn len(&self, path: &Path) -> Result<u64> {
        self.check_fault(FaultOperation::Read, &self.read_count)
            .map_err(ContextraError::Io)?;
        let files = self.files.read();
        files
            .get(path)
            .map(|data| data.len() as u64)
            .ok_or_else(|| ContextraError::NotFound(path.display().to_string()))
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        let dirs = self.dirs.read();
        if !dirs.contains(path) && !path.as_os_str().is_empty() {
            return Err(ContextraError::NotFound(path.display().to_string()));
        }

        let files = self.files.read();
        let mut entries = Vec::new();

        for file_path in files.keys() {
            if file_path.parent() == Some(path) {
                if let Some(name) = file_path.file_name() {
                    entries.push(name.to_string_lossy().to_string());
                }
            }
        }
        for dir_path in dirs.iter() {
            if dir_path != path && dir_path.parent() == Some(path) {
                if let Some(name) = dir_path.file_name() {
                    entries.push(name.to_string_lossy().to_string());
                }
            }
        }

        entries.sort();
        entries.dedup();
        Ok(entries)
    }

    fn sync_dir(&self, path: &Path) -> Result<()> {
        self.check_fault(FaultOperation::Sync, &self.sync_count)
            .map_err(ContextraError::Io)?;
        let dirs = self.dirs.read();
        if dirs.contains(path) || path.as_os_str().is_empty() {
            Ok(())
        } else {
            Err(ContextraError::NotFound(path.display().to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR[contextra-testkit:TEST-FAULT-VFS-WRITE-LIMIT-001]
    // Prüft, dass nach Erreichen des konfigurierten Schreiblimits folgende Schreibvorgänge abgewiesen werden.
    #[test]
    fn test_fault_vfs_write_limit() {
        let vfs = FaultVfs::new();
        vfs.set_config(FaultConfig {
            fail_writes_after: Some(2),
            ..Default::default()
        });

        let p1 = Path::new("file1.txt");
        let p2 = Path::new("file2.txt");
        let p3 = Path::new("file3.txt");

        assert!(vfs.write_file(p1, b"hello").is_ok());
        assert!(vfs.write_file(p2, b"world").is_ok());
        let res3 = vfs.write_file(p3, b"fail");
        assert!(matches!(res3, Err(e) if e.kind() == ErrorKind::WriteZero));
    }

    // ANCHOR[contextra-testkit:TEST-FAULT-VFS-SYNC-FAILURE-001]
    // Prüft, dass Sync-Aufrufe nach dem Schwellenwert fehlschlagen (fsync-Fehlersimulation).
    #[test]
    fn test_fault_vfs_sync_failure() {
        let vfs = FaultVfs::new();
        vfs.set_config(FaultConfig {
            fail_syncs_after: Some(1),
            ..Default::default()
        });

        assert!(vfs.sync().is_ok());
        assert!(vfs.sync().is_err());
    }

    // ANCHOR[contextra-testkit:TEST-FAULT-VFS-VFS-CONTRACT-001]
    // Prüft die Implementierung des contextra_ports::Vfs-Traits.
    #[test]
    fn test_fault_vfs_vfs_trait_contract() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let vfs = FaultVfs::new();
        let base_dir = Path::new("/var/data");
        vfs.create_dir_all(base_dir)?;

        let file_path = base_dir.join("test.wal");
        vfs.write_atomic(&file_path, b"test-payload")?;
        assert!(vfs.exists(&file_path)?);
        assert_eq!(vfs.len(&file_path)?, 12);

        let data = vfs.read(&file_path)?;
        assert_eq!(data, b"test-payload");

        let entries = vfs.list_dir(base_dir)?;
        assert_eq!(entries, vec!["test.wal".to_string()]);

        vfs.sync_dir(base_dir)?;
        vfs.remove_file(&file_path)?;
        assert!(!vfs.exists(&file_path)?);
        Ok(())
    }

    // ANCHOR[contextra-testkit:TEST-FAULT-VFS-CRASH-MATRIX-001]
    // Prüft den Crash-Matrix-Iterator for_each_crash_point.
    #[test]
    fn test_fault_vfs_crash_matrix_iteration() {
        let outcomes = FaultVfs::for_each_crash_point(|vfs, crash_idx| {
            let path = Path::new("/data/matrix.log");
            let res = vfs.write_atomic(path, b"matrix-entry");
            (crash_idx, res.is_ok())
        });

        // First iteration is 0 (immediate failure), last iteration succeeds
        assert!(!outcomes.is_empty());
        let first = outcomes.first();
        assert!(matches!(first, Some((0, (_, false)))));

        let last = outcomes.last();
        assert!(matches!(last, Some((_, (_, true)))));
    }
}
