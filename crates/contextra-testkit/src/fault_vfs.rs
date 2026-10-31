//! # Virtuelles Dateisystem mit deterministischer Fehlerinjektion (`fault_vfs`)
//!
//! Dieses Modul stellt mit [`FaultVfs`] ein In-Memory-Dateisystem bereit, das I/O-Fehler,
//! Abstürze und `fsync`-Verweigerungen gezielt nach einer konfigurierbaren Anzahl von
//! Operationen injizieren kann.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Regel 5 (SPEC §3.2):** I/O nur über Vfs; `fsync`-Fehler müssen strikt propagiert werden.
//! - **Regel 10 (SPEC §3.2):** Fail-Closed: Schlägt ein I/O- oder sync-Aufruf fehl, darf das
//!   System keinen Teilerfolg vortäuschen oder in einen inkonsistenten Zustand verfallen.
//!
//! ## Funktionsweise der Fehlerinjektion
//! Über [`FaultConfig`] wird konfiguriert, nach wie vielen erfolgreichen Schreib-,
//! Lese- oder Sync-Operationen ein synthetischer I/O-Fehler ausgelöst wird:
//! - `fail_writes_after`: Gibt nach N erfolgreichen Schreibvorgängen einen Fehler zurück.
//! - `fail_syncs_after`: Simuliert fehlschlagende `fsync`-Befehle des Betriebssystems.
//! - `fail_reads_after`: Simuliert Lesefehler von defekten Blöcken.
//! - `crashed`: Simuliert einen sofortigen Systemabsturz (Power-Cut).

// FILE-CONTEXT
// STAND: 2026-09-19T20:12:00Z (SESSION: 01c5be8b)
// ZWECK: Deterministic I/O error injector and Virtual File System test utility.
// INVARIANTEN: Zero real data corruption, deterministic fault triggers based on operation count.

use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

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
/// - Die Operationszähler (`read_count`, `write_count`, `sync_count`) sind atomare [`AtomicUsize`],
///   sodass auch nebenläufige Schreibversuche präzise gezählt werden.
#[derive(Debug, Clone, Default)]
pub struct FaultVfs {
    /// In-Memory Dateispeicher: Pfad -> Dateibytes.
    files: Arc<RwLock<BTreeMap<PathBuf, Vec<u8>>>>,
    /// Anzahl der versuchten Leseoperationen.
    read_count: Arc<AtomicUsize>,
    /// Anzahl der versuchten Schreiboperationen.
    write_count: Arc<AtomicUsize>,
    /// Anzahl der versuchten Sync-Operationen.
    sync_count: Arc<AtomicUsize>,
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
    ///
    /// **Parameter:**
    /// - `config`: Die neue [`FaultConfig`].
    pub fn set_config(&self, config: FaultConfig) {
        *self.config.write() = config;
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-TRIGGER-CRASH-001]
    /// Simuliert einen abrupten Systemabsturz (Power-Cut).
    ///
    /// Nach diesem Aufruf schlagen alle folgenden Schreib-, Lese- und Sync-Operationen
    /// mit `ErrorKind::BrokenPipe` fehl, bis [`Self::reset_counters`] aufgerufen wird.
    pub fn trigger_crash(&self) {
        self.crashed.store(true, Ordering::SeqCst);
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-RESET-COUNTERS-001]
    /// Setzt die Operationszähler und den Crash-Status zurück.
    ///
    /// Die bestehenden Dateiinhalte in `files` bleiben dabei unverändert erhalten.
    pub fn reset_counters(&self) {
        self.read_count.store(0, Ordering::SeqCst);
        self.write_count.store(0, Ordering::SeqCst);
        self.sync_count.store(0, Ordering::SeqCst);
        self.crashed.store(false, Ordering::SeqCst);
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-CHECK-WRITE-001]
    /// Prüft vor einem Schreibvorgang, ob eine Fehlerbedingung erfüllt ist.
    ///
    /// **Ablauf:**
    /// 1. Crash-Zustand prüfen -> Fehler `ErrorKind::BrokenPipe`.
    /// 2. `fail_all`-Konfiguration prüfen -> Fehler `io::Error::other`.
    /// 3. Schreibzähler atomar inkrementieren.
    /// 4. Prüfen, ob das Limit `fail_writes_after` erreicht oder überschritten wurde.
    /// 5. Bei Überschreitung Rückgabe von `ErrorKind::WriteZero`.
    fn check_write_fault(&self) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "VFS simulated crash"));
        }
        let cfg = self.config.read();
        if cfg.fail_all {
            return Err(io::Error::other("Simulated injected failure (fail_all)"));
        }
        let current = self.write_count.fetch_add(1, Ordering::SeqCst);
        if let Some(limit) = cfg.fail_writes_after {
            if current >= limit {
                return Err(io::Error::new(
                    ErrorKind::WriteZero,
                    "Simulated disk write failure",
                ));
            }
        }
        Ok(())
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-CHECK-READ-001]
    /// Prüft vor einem Lesevorgang, ob eine Fehlerbedingung erfüllt ist.
    ///
    /// **Ablauf:**
    /// 1. Crash-Zustand prüfen -> Fehler `ErrorKind::BrokenPipe`.
    /// 2. `fail_all`-Konfiguration prüfen -> Fehler `io::Error::other`.
    /// 3. Lesezähler atomar inkrementieren.
    /// 4. Prüfen, ob das Limit `fail_reads_after` erreicht oder überschritten wurde.
    /// 5. Bei Überschreitung Rückgabe von `ErrorKind::UnexpectedEof`.
    fn check_read_fault(&self) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "VFS simulated crash"));
        }
        let cfg = self.config.read();
        if cfg.fail_all {
            return Err(io::Error::other("Simulated injected failure (fail_all)"));
        }
        let current = self.read_count.fetch_add(1, Ordering::SeqCst);
        if let Some(limit) = cfg.fail_reads_after {
            if current >= limit {
                return Err(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    "Simulated disk read failure",
                ));
            }
        }
        Ok(())
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-WRITE-FILE-001]
    /// Schreibt Bytes an den angegebenen Pfad unter Berücksichtigung der Fehlerrichtlinien.
    ///
    /// **Fehlerbehandlung:**
    /// Schlägt fehl, falls ein Crash aktiv ist oder das Schreiblimit erreicht wurde.
    /// Im Fehlerfall werden keine Daten im In-Memory-Speicher abgelegt.
    pub fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        self.check_write_fault()?;
        self.files.write().insert(path.to_path_buf(), data.to_vec());
        Ok(())
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-READ-FILE-001]
    /// Liest Bytes vom angegebenen Pfad unter Berücksichtigung der Fehlerrichtlinien.
    ///
    /// **Fehlerbehandlung:**
    /// Liefert einen Lesefehler bei injiziertem Fault oder `ErrorKind::NotFound`,
    /// falls die Datei nicht existiert.
    pub fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.check_read_fault()?;
        self.files
            .read()
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "File not found in FaultVfs"))
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-SYNC-001]
    /// Simuliert einen `fsync`-Aufruf zur dauerhaften Bestätigung auf dem Speichermedium.
    ///
    /// **Ablauf:**
    /// 1. Crash-Zustand prüfen.
    /// 2. `fail_all` prüfen.
    /// 3. Sync-Zähler inkrementieren.
    /// 4. Falls `fail_syncs_after` erreicht ist, Rückgabe eines `io::Error`.
    pub fn sync(&self) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "VFS simulated crash"));
        }
        let cfg = self.config.read();
        if cfg.fail_all {
            return Err(io::Error::other("Simulated injected failure (fail_all)"));
        }
        let current = self.sync_count.fetch_add(1, Ordering::SeqCst);
        if let Some(limit) = cfg.fail_syncs_after {
            if current >= limit {
                return Err(io::Error::other("Simulated fsync I/O failure"));
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
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
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
        assert!(res3.is_err());
        assert_eq!(res3.unwrap_err().kind(), ErrorKind::WriteZero);
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
}
