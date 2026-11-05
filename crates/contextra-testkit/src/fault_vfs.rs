//! # Virtuelles Dateisystem mit deterministischer Fehlerinjektion (`fault_vfs`)
//!
//! Dieses Modul stellt mit [`FaultVfs`] ein zweischichtiges In-Memory-Dateisystem (Cache und Medium) bereit,
//! das I/O-Fehler, Abstürze, umsortierte Sektorpersistierung und `fsync`-Verweigerungen gezielt nach
//! einer konfigurierbaren Anzahl von Operationen oder anhand eines deterministischen Plans injizieren kann.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Port:** Implementiert [`contextra_ports::Vfs`].
//! - **Regel 5 (SPEC §3.2):** I/O nur über Vfs; `fsync`-Fehler müssen strikt propagiert werden.
//! - **Regel 10 (SPEC §3.2):** Fail-Closed: Schlägt ein I/O- oder sync-Aufruf fehl, darf das
//!   System keinen Teilerfolg vortäuschen oder in einen inkonsistenten Zustand verfallen.
//! - **T-FSYNC-1 / DF-03:** Ein `fsync`-Fehler auf einem Handle macht dieses dauerhaft fatal
//!   (`Poisoned`). Weitere Aufrufe werden abgewiesen.
//! - **D-113 Abs. 5:** Modellierung von Cache und Medium; `read` liest Cache, `read_verified`
//!   liest Medium; `fsync` überträgt Cache auf Medium; Crash verwirft Cache und liefert Medium;
//!   umsortierte Sektorpersistierung mit konfigurierbaren Sektorgrößen (512 B und 4 KiB).

// FILE-CONTEXT
// STAND: 2026-11-04
// ZWECK: Deterministic I/O error injector and two-layer (Cache & Medium) Virtual File System test utility.
// INVARIANTEN: Zero real data corruption, deterministic fault triggers based on operation count, T-FSYNC-1 handle poisoning.

use crate::manual_clock::ManualClock;
use contextra_ports::rng::{Rng, SeededRng};
use contextra_ports::vfs::{VerifiedVia, Vfs, VfsFile, VfsOpenOptions};
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
    /// Lesezugriff auf Dateiinhalte (`read`, `read_at`).
    Read,
    /// Schreibzugriff (`write_atomic`, `write_file`, `write_at`).
    Write,
    /// Synchronisationsaufruf (`sync`, `sync_dir`, `sync_data`, `sync_all`).
    Sync,
    /// Umbenennen (`rename`).
    Rename,
    /// Löschen einer Datei (`remove_file`, `unlink`).
    Remove,
    /// Verzeichnisanlage (`create_dir_all`).
    CreateDir,
    /// Längenänderung (`set_len`).
    SetLen,
}

/// Zeitpunkt des Abbruchs / Fehlerinjektion relativ zur Ausführung der Operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CrashTiming {
    /// Abbruch unmittelbar vor dem Ausführen der eigentlichen Operation.
    Before,
    /// Abbruch unmittelbar nach dem Ausführen der Operation (Effekt im Cache/RAM vorhanden).
    After,
}

/// Ein deterministischer Crash-Punkt in der Crash-Matrix (SPEC §15.1, T-CRASH-*).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CrashPoint {
    /// Laufende 0-basierte globale Operationsnummer.
    pub op_index: usize,
    /// Operationsart (`Write`, `Sync`, `Rename`, `Remove`, etc.).
    pub operation: FaultOperation,
    /// Zeitpunkt relativ zur Operation (vorher oder nachher).
    pub timing: CrashTiming,
    /// Betroffener Dateipfad (falls bekannt).
    pub path: Option<PathBuf>,
}

/// Spezifischer Fehlerinjektions-Plan für den n-ten Aufruf einer Operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaultPlan {
    /// Betroffene Operationsart.
    pub operation: FaultOperation,
    /// 0-basierter Index des Aufrufs, der fehlschlagen soll.
    pub nth_call: usize,
    /// Art des zurückzugebenden I/O-Fehlers.
    pub error_kind: ErrorKind,
}

impl FaultPlan {
    /// Erstellt einen exakten Plan für den n-ten Aufruf einer Operation mit dem angegebenen Fehler.
    pub fn exact(operation: FaultOperation, nth_call: usize, error_kind: ErrorKind) -> Self {
        Self {
            operation,
            nth_call,
            error_kind,
        }
    }
}

/// Konfiguration für den Modus „umsortierte Sektorpersistierung“ (SPEC D-113 Abs. 5).
#[derive(Debug, Clone, PartialEq)]
pub struct SectorReorderConfig {
    /// Sektorgröße in Bytes (typisch 512 oder 4096).
    pub sector_size: usize,
    /// Deterministischer Seed für Pseudozufallsauswahl.
    pub seed: u64,
    /// Anteil der veränderten Sektoren, die beim Crash persistent werden (0.0 bis 1.0).
    pub persisted_fraction: f64,
}

impl Default for SectorReorderConfig {
    fn default() -> Self {
        Self {
            sector_size: 512,
            seed: 0,
            persisted_fraction: 0.5,
        }
    }
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
    /// Löschoperationen (`remove_file`/`unlink`) nach dieser Anzahl erfolgreicher Aufrufe fehlschlagen lassen.
    pub fail_removes_after: Option<usize>,
    /// Verzeichniserstellungen (`create_dir_all`) nach dieser Anzahl erfolgreicher Aufrufe fehlschlagen lassen.
    pub fail_createdirs_after: Option<usize>,
    /// Gezielt bei einer bestimmten Operation und einem bestimmten Null-basierten Index fehlschlagen.
    pub fail_at_operation: Option<(FaultOperation, usize)>,
    /// Expliziter Einzelplan oder Liste von Plänen.
    pub plan: Option<FaultPlan>,
    /// Globaler Notschalter: Alle Operationen schlagen sofort fehl.
    pub fail_all: bool,
    /// Simuliere Platte voll (ENOSPC / StorageFull) für Schreibvorgänge.
    pub fail_with_enospc: bool,
    /// Crash-Punkt (nach N Operationen crashen und alle nachfolgenden ungeschriebenen Daten verwerfen).
    pub crash_after_ops: Option<usize>,
    /// Replay-Limit: Nach n Operationen anhalten / abbrechen.
    pub replay_limit: Option<usize>,
    /// Spezifischer CrashPoint für deterministischen Abbruch.
    pub target_crash_point: Option<CrashPoint>,
    /// Konfiguration für umsortierte Sektorpersistierung bei Crash (D-113 Abs. 5).
    pub sector_reorder: Option<SectorReorderConfig>,
    /// Optionaler deterministischer Seed zur Ausgabe im Fehlertext (SPEC P7, T-CRASH-*).
    pub seed: Option<u64>,
}

/// In-Memory Repräsentation einer geöffneten Datei in [`FaultVfs`].
#[derive(Debug)]
pub struct FaultFile {
    path: PathBuf,
    vfs: FaultVfs,
    options: VfsOpenOptions,
    poisoned: AtomicBool,
}

impl FaultFile {
    fn check_poisoned(&self) -> io::Result<()> {
        if self.poisoned.load(Ordering::SeqCst) {
            return Err(io::Error::other(
                "File handle is poisoned due to previous fsync error",
            ));
        }
        Ok(())
    }

    fn poison(&self) {
        self.poisoned.store(true, Ordering::SeqCst);
    }
}

impl VfsFile for FaultFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        self.check_poisoned()?;
        if !self.options.read {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "File not open for reading",
            ));
        }
        self.vfs
            .check_fault(FaultOperation::Read, &self.vfs.read_count)?;
        let cache = self.vfs.cache.read();
        let data = cache
            .get(&self.path)
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "File not found in cache"))?;

        let off = usize::try_from(offset)
            .map_err(|_| io::Error::new(ErrorKind::InvalidInput, "Offset out of range"))?;

        if off >= data.len() {
            return Ok(0);
        }

        let available = data.get(off..).unwrap_or(&[]);
        let to_copy = std::cmp::min(buf.len(), available.len());
        if let (Some(dst), Some(src)) = (buf.get_mut(..to_copy), available.get(..to_copy)) {
            dst.copy_from_slice(src);
        }
        Ok(to_copy)
    }

    fn read_verified(&self, offset: u64, len: usize) -> io::Result<(Vec<u8>, VerifiedVia)> {
        self.check_poisoned()?;
        if !self.options.read {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "File not open for reading",
            ));
        }
        self.vfs
            .check_fault(FaultOperation::Read, &self.vfs.read_count)?;
        // D-113: read_verified liest strikt von der Mediumschicht, nicht aus dem Cache!
        let medium = self.vfs.medium.read();
        let data = medium
            .get(&self.path)
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "File not found on medium"))?;

        let off = usize::try_from(offset)
            .map_err(|_| io::Error::new(ErrorKind::InvalidInput, "Offset out of range"))?;

        if off >= data.len() {
            return Ok((Vec::new(), VerifiedVia::Direct));
        }

        let available = data.get(off..).unwrap_or(&[]);
        let to_copy = std::cmp::min(len, available.len());
        let res = available.get(..to_copy).unwrap_or(&[]).to_vec();
        Ok((res, VerifiedVia::Direct))
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        self.check_poisoned()?;
        if !self.options.write {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "File not open for writing",
            ));
        }
        self.vfs.record_and_check_crash_point(
            FaultOperation::Write,
            CrashTiming::Before,
            Some(&self.path),
        )?;
        self.vfs
            .check_fault(FaultOperation::Write, &self.vfs.write_count)?;

        let off = usize::try_from(offset)
            .map_err(|_| io::Error::new(ErrorKind::InvalidInput, "Offset out of range"))?;

        let mut cache = self.vfs.cache.write();
        let data = cache.entry(self.path.clone()).or_insert_with(|| {
            self.vfs
                .medium
                .read()
                .get(&self.path)
                .cloned()
                .unwrap_or_default()
        });

        if off + buf.len() > data.len() {
            data.resize(off + buf.len(), 0);
        }

        if let Some(target_slice) = data.get_mut(off..off + buf.len()) {
            target_slice.copy_from_slice(buf);
        }
        drop(cache);
        self.vfs.record_and_check_crash_point(
            FaultOperation::Write,
            CrashTiming::After,
            Some(&self.path),
        )?;
        Ok(buf.len())
    }

    fn set_len(&self, len: u64) -> io::Result<()> {
        self.check_poisoned()?;
        if !self.options.write {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "File not open for writing",
            ));
        }
        self.vfs
            .check_fault(FaultOperation::SetLen, &self.vfs.set_len_count)?;

        let target_len = usize::try_from(len)
            .map_err(|_| io::Error::new(ErrorKind::InvalidInput, "Length out of range"))?;

        let mut cache = self.vfs.cache.write();
        let data = cache.entry(self.path.clone()).or_insert_with(|| {
            self.vfs
                .medium
                .read()
                .get(&self.path)
                .cloned()
                .unwrap_or_default()
        });
        data.resize(target_len, 0);
        Ok(())
    }

    fn len(&self) -> io::Result<u64> {
        self.check_poisoned()?;
        self.vfs
            .check_fault(FaultOperation::Read, &self.vfs.read_count)?;
        let cache = self.vfs.cache.read();
        let data = cache
            .get(&self.path)
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "File not found"))?;
        Ok(data.len() as u64)
    }

    fn sync_data(&self) -> io::Result<()> {
        self.check_poisoned()?;
        if let Err(e) = self.vfs.record_and_check_crash_point(
            FaultOperation::Sync,
            CrashTiming::Before,
            Some(&self.path),
        ) {
            self.poison();
            return Err(e);
        }
        match self
            .vfs
            .check_fault(FaultOperation::Sync, &self.vfs.sync_count)
        {
            Ok(()) => {
                let cache = self.vfs.cache.read();
                if let Some(content) = cache.get(&self.path) {
                    self.vfs
                        .medium
                        .write()
                        .insert(self.path.clone(), content.clone());
                }
                drop(cache);
                if let Err(e) = self.vfs.record_and_check_crash_point(
                    FaultOperation::Sync,
                    CrashTiming::After,
                    Some(&self.path),
                ) {
                    self.poison();
                    return Err(e);
                }
                Ok(())
            }
            Err(e) => {
                // T-FSYNC-1 / DF-03: Handle wird fatal vergiftet
                self.poison();
                Err(e)
            }
        }
    }

    fn sync_all(&self) -> io::Result<()> {
        self.check_poisoned()?;
        if let Err(e) = self.vfs.record_and_check_crash_point(
            FaultOperation::Sync,
            CrashTiming::Before,
            Some(&self.path),
        ) {
            self.poison();
            return Err(e);
        }
        match self
            .vfs
            .check_fault(FaultOperation::Sync, &self.vfs.sync_count)
        {
            Ok(()) => {
                let cache = self.vfs.cache.read();
                if let Some(content) = cache.get(&self.path) {
                    self.vfs
                        .medium
                        .write()
                        .insert(self.path.clone(), content.clone());
                }
                drop(cache);
                if let Err(e) = self.vfs.record_and_check_crash_point(
                    FaultOperation::Sync,
                    CrashTiming::After,
                    Some(&self.path),
                ) {
                    self.poison();
                    return Err(e);
                }
                Ok(())
            }
            Err(e) => {
                // T-FSYNC-1 / DF-03: Handle wird fatal vergiftet
                self.poison();
                Err(e)
            }
        }
    }
}

// ANCHOR[contextra-testkit:FAULT-VFS-STRUCT-001]
/// Simuliertes zweischichtiges In-Memory-Dateisystem mit deterministischer Fehlerinjektion.
///
/// **Zweck:** Dient als I/O-Testunterlage für Speichermodule (z. B. WAL und LSM-Storage),
/// um Fehlertoleranz und Absturzsicherheit ohne reale Festplattenfehler zu testen.
/// Bildet Cache-Schicht und Medium-Schicht getrennt ab (D-113).
#[derive(Debug, Clone, Default)]
pub struct FaultVfs {
    /// In-Memory Seitencache (flüchtige Schreiboperationen): Pfad -> Dateibytes.
    cache: Arc<RwLock<BTreeMap<PathBuf, Vec<u8>>>>,
    /// In-Memory Medium (dauerhafter Speicherbestand): Pfad -> Dateibytes.
    medium: Arc<RwLock<BTreeMap<PathBuf, Vec<u8>>>>,
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
    /// Anzahl der versuchten SetLen-Operationen.
    set_len_count: Arc<AtomicUsize>,
    /// Aktuelle Injektionskonfiguration.
    config: Arc<RwLock<FaultConfig>>,
    /// Flag für simulierten Totalabsturz.
    crashed: Arc<AtomicBool>,
    /// Aufgezeichnete Crash-Punkte aller ausgeführten I/O-Operationen.
    recorded_points: Arc<RwLock<Vec<CrashPoint>>>,
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

    /// Setzt einen gezielten Fehlerinjektions-Plan.
    pub fn set_plan(&self, plan: FaultPlan) {
        let mut cfg = self.config.write();
        cfg.plan = Some(plan);
    }

    /// Konfiguriert das Dateisystem so, dass es nach genau `n` Operationen abbricht (`replay_until`).
    pub fn replay_until(&self, n: usize) {
        let mut cfg = self.config.write();
        cfg.replay_limit = Some(n);
    }

    /// Liefert die Liste aller bisher durchlaufenen potenziellen Crash-Punkte (`before` und `after`).
    pub fn crash_points(&self) -> Vec<CrashPoint> {
        self.recorded_points.read().clone()
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-TRIGGER-CRASH-001]
    /// Simuliert einen abrupten Systemabsturz (Power-Cut).
    ///
    /// Verwirft alle nicht mit `sync`/`sync_data`/`sync_all` gesicherten Cache-Inhalte
    /// und setzt den Cache auf den exakten Zustand des Mediums zurück.
    /// Falls `sector_reorder` konfiguriert ist, werden veränderte Sektoren in deterministisch
    /// pseudozufälliger Reihenfolge teilweise vorab auf das Medium übertragen (Torn Writes auf Sektorebene, D-113 Abs. 5).
    pub fn trigger_crash(&self) {
        self.crashed.store(true, Ordering::SeqCst);
        self.apply_crash_state();
    }

    /// Simuliert einen Crash und sofortigen Neustart (Recovery), wodurch Operationen wieder zulässig sind,
    /// jedoch alle ungesyncten Cache-Daten verloren sind (Medium-Zustand wird geladen).
    pub fn crash_and_recover(&self) {
        self.apply_crash_state();
        self.crashed.store(false, Ordering::SeqCst);
    }

    fn apply_crash_state(&self) {
        let cfg = self.config.read().clone();
        if let Some(ref reorder) = cfg.sector_reorder {
            let sector_size = if reorder.sector_size == 0 {
                512
            } else {
                reorder.sector_size
            };
            let rng = SeededRng::new(reorder.seed);

            let cache = self.cache.read();
            let mut medium = self.medium.write();

            for (path, cache_bytes) in cache.iter() {
                let medium_bytes = medium.entry(path.clone()).or_default();
                let max_len = std::cmp::max(cache_bytes.len(), medium_bytes.len());
                let total_sectors = max_len.div_ceil(sector_size);

                let mut changed_sectors = Vec::new();
                for sec_idx in 0..total_sectors {
                    let start = sec_idx * sector_size;
                    let end = std::cmp::min(start + sector_size, max_len);

                    let c_slice = if start < cache_bytes.len() {
                        cache_bytes
                            .get(start..std::cmp::min(end, cache_bytes.len()))
                            .unwrap_or(&[])
                    } else {
                        &[]
                    };
                    let m_slice = if start < medium_bytes.len() {
                        medium_bytes
                            .get(start..std::cmp::min(end, medium_bytes.len()))
                            .unwrap_or(&[])
                    } else {
                        &[]
                    };

                    if c_slice != m_slice {
                        changed_sectors.push(sec_idx);
                    }
                }

                if !changed_sectors.is_empty() {
                    for i in (1..changed_sectors.len()).rev() {
                        #[allow(clippy::cast_possible_truncation)]
                        let j = (rng.next_u64() as usize) % (i + 1);
                        changed_sectors.swap(i, j);
                    }

                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let persist_count = ((changed_sectors.len() as f64)
                        * reorder.persisted_fraction)
                        .round() as usize;
                    let num_to_persist = std::cmp::min(persist_count, changed_sectors.len());
                    let sectors_to_persist = changed_sectors.get(..num_to_persist).unwrap_or(&[]);

                    if medium_bytes.len() < cache_bytes.len() {
                        medium_bytes.resize(cache_bytes.len(), 0);
                    }

                    for &sec_idx in sectors_to_persist {
                        let start = sec_idx * sector_size;
                        let end = std::cmp::min(start + sector_size, cache_bytes.len());
                        if start < cache_bytes.len() {
                            if let Some(src) = cache_bytes.get(start..end) {
                                if medium_bytes.len() < end {
                                    medium_bytes.resize(end, 0);
                                }
                                if let Some(dst) = medium_bytes.get_mut(start..end) {
                                    dst.copy_from_slice(src);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Cache auf Stand des Mediums zurücksetzen (ungesyncte Cache-Daten werden verworfen)
        let medium = self.medium.read();
        let mut cache = self.cache.write();
        *cache = medium.clone();
    }

    /// Alias für `trigger_crash`.
    pub fn crash(&self) {
        self.trigger_crash();
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-RESET-COUNTERS-001]
    /// Setzt die Operationszähler und den Crash-Status zurück.
    ///
    /// Die bestehenden Dateiinhalte auf dem Medium und im Cache bleiben erhalten.
    pub fn reset_counters(&self) {
        self.read_count.store(0, Ordering::SeqCst);
        self.write_count.store(0, Ordering::SeqCst);
        self.sync_count.store(0, Ordering::SeqCst);
        self.rename_count.store(0, Ordering::SeqCst);
        self.remove_count.store(0, Ordering::SeqCst);
        self.create_dir_count.store(0, Ordering::SeqCst);
        self.set_len_count.store(0, Ordering::SeqCst);
        self.crashed.store(false, Ordering::SeqCst);
        self.recorded_points.write().clear();
    }

    fn record_and_check_crash_point(
        &self,
        op: FaultOperation,
        timing: CrashTiming,
        path: Option<&Path>,
    ) -> io::Result<()> {
        let op_idx = self.total_operations();
        let pt = CrashPoint {
            op_index: op_idx,
            operation: op,
            timing,
            path: path.map(Path::to_path_buf),
        };
        self.recorded_points.write().push(pt.clone());

        let cfg = self.config.read();
        if let Some(ref target) = cfg.target_crash_point {
            if target == &pt {
                drop(cfg);
                self.trigger_crash();
                return Err(io::Error::new(
                    ErrorKind::BrokenPipe,
                    format!("Crash injected at target point {pt:?}"),
                ));
            }
        }
        Ok(())
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-CHECK-FAULT-001]
    /// Interne Überprüfung, ob für die gegebene Operation ein Fehler injiziert werden soll.
    fn check_fault(&self, op: FaultOperation, counter: &AtomicUsize) -> io::Result<()> {
        let cfg = self.config.read();
        let seed_suffix = match cfg.seed {
            Some(seed) => format!(" (seed={seed})"),
            None => String::new(),
        };

        if self.crashed.load(Ordering::SeqCst) {
            return Err(io::Error::new(
                ErrorKind::BrokenPipe,
                format!("VFS simulated crash{seed_suffix}"),
            ));
        }
        let total = self.total_operations();

        if let Some(limit) = cfg.crash_after_ops {
            if total >= limit {
                drop(cfg);
                self.trigger_crash();
                return Err(io::Error::new(
                    ErrorKind::BrokenPipe,
                    format!("VFS simulated crash at op {total}{seed_suffix}"),
                ));
            }
        }

        if let Some(limit) = cfg.replay_limit {
            if total >= limit {
                drop(cfg);
                self.trigger_crash();
                return Err(io::Error::new(
                    ErrorKind::BrokenPipe,
                    format!("VFS replay_until limit {limit} reached at op {total}{seed_suffix}"),
                ));
            }
        }

        if cfg.fail_all {
            return Err(io::Error::other(format!(
                "Simulated injected failure [fail_all]{seed_suffix}"
            )));
        }

        let idx = counter.fetch_add(1, Ordering::SeqCst);

        if let Some(ref plan) = cfg.plan {
            if plan.operation == op && plan.nth_call == idx {
                return Err(io::Error::new(
                    plan.error_kind,
                    format!(
                        "Simulated failure from FaultPlan for {op:?} at call {idx}{seed_suffix}"
                    ),
                ));
            }
        }

        if let Some((target_op, target_idx)) = cfg.fail_at_operation {
            if target_op == op && target_idx == idx {
                return Err(io::Error::other(format!(
                    "Simulated failure at operation {op:?} at call {idx}{seed_suffix}"
                )));
            }
        }

        match op {
            FaultOperation::Write => {
                if let Some(limit) = cfg.fail_writes_after {
                    if idx >= limit {
                        let kind = if cfg.fail_with_enospc {
                            ErrorKind::StorageFull
                        } else {
                            ErrorKind::WriteZero
                        };
                        let msg = if cfg.fail_with_enospc {
                            format!("Simulated disk full ENOSPC{seed_suffix}")
                        } else {
                            format!("Simulated disk write failure{seed_suffix}")
                        };
                        return Err(io::Error::new(kind, msg));
                    }
                }
            }
            FaultOperation::Read => {
                if let Some(limit) = cfg.fail_reads_after {
                    if idx >= limit {
                        return Err(io::Error::new(
                            ErrorKind::UnexpectedEof,
                            format!("Simulated disk read failure{seed_suffix}"),
                        ));
                    }
                }
            }
            FaultOperation::Sync => {
                if let Some(limit) = cfg.fail_syncs_after {
                    if idx >= limit {
                        return Err(io::Error::other(format!(
                            "Simulated fsync I/O failure{seed_suffix}"
                        )));
                    }
                }
            }
            FaultOperation::Rename => {
                if let Some(limit) = cfg.fail_renames_after {
                    if idx >= limit {
                        return Err(io::Error::other(format!(
                            "Simulated rename failure{seed_suffix}"
                        )));
                    }
                }
            }
            FaultOperation::Remove => {
                if let Some(limit) = cfg.fail_removes_after {
                    if idx >= limit {
                        return Err(io::Error::other(format!(
                            "Simulated remove/unlink failure{seed_suffix}"
                        )));
                    }
                }
            }
            FaultOperation::CreateDir => {
                if let Some(limit) = cfg.fail_createdirs_after {
                    if idx >= limit {
                        return Err(io::Error::other(format!(
                            "Simulated create_dir failure{seed_suffix}"
                        )));
                    }
                }
            }
            FaultOperation::SetLen => {}
        }

        Ok(())
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-WRITE-FILE-001]
    /// Schreibt Bytes in den Cache unter Berücksichtigung der Fehlerrichtlinien.
    pub fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        self.check_fault(FaultOperation::Write, &self.write_count)?;
        if let Some(parent) = path.parent() {
            self.create_dir_all_internal(parent)?;
        }
        self.cache.write().insert(path.to_path_buf(), data.to_vec());
        Ok(())
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-READ-FILE-001]
    /// Liest Bytes aus dem Cache unter Berücksichtigung der Fehlerrichtlinien.
    pub fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.check_fault(FaultOperation::Read, &self.read_count)?;
        self.cache
            .read()
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "File not found"))
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-SYNC-001]
    /// Simuliert einen `fsync`-Aufruf zur dauerhaften Bestätigung aller Cache-Dateien auf dem Speichermedium.
    pub fn sync(&self) -> io::Result<()> {
        self.check_fault(FaultOperation::Sync, &self.sync_count)?;
        let cache = self.cache.read();
        let mut medium = self.medium.write();
        *medium = cache.clone();
        Ok(())
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
            + self.set_len_count.load(Ordering::SeqCst)
    }

    // ANCHOR[contextra-testkit:FAULT-VFS-CRASH-MATRIX-001]
    /// Führt einen Testblock für jeden sequentiellen Crash-Punkt (Absturz nach $N$ Operationen) aus.
    ///
    /// **Ablauf:**
    /// 1. Führt den Workload zuerst unbeeinflusst aus, um die Gesamtzahl an Operationen $M$ zu ermitteln.
    /// 2. Iteriert für jeden Punkt $i \in [0, M]$ durch:
    ///    - Erstellt eine frische `FaultVfs`-Instanz mit Crash-Punkt nach $i$ Operationen.
    ///    - Führt die Test-Closure `setup_and_run` aus.
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
            if crash_at < total_ops {
                vfs.set_config(FaultConfig {
                    crash_after_ops: Some(crash_at),
                    ..Default::default()
                });
            }
            let outcome = setup_and_run(&vfs, Some(crash_at));
            results.push((crash_at, outcome));
        }

        results
    }
}

// ANCHOR[contextra-testkit:CRASH-MATRIX-DRIVER-001]
/// Ergebnis eines einzelnen Schritts in der Crash-Matrix (SPEC §15.1, T-CRASH-*).
#[derive(Debug, Clone)]
pub struct CrashMatrixStepResult<T> {
    /// Crash-Punkt (nach wie vielen Operationen der Crash ausgelöst wurde).
    pub crash_at_op: usize,
    /// Anzahl der tatsächlich vor dem Crash ausgeführten Operationen.
    pub executed_ops: usize,
    /// Ob der Crash ausgelöst wurde.
    pub did_crash: bool,
    /// Ergebnis der Verifikation bzw. des Recovery-Checks.
    pub recovery_outcome: T,
}

/// Strukturierter Treiber zur systematischen Aufzählung aller Crash-Punkte eines Workloads (SPEC §15.1).
///
/// Simuliert für jeden Schritt $n \in [0, M]$:
/// 1. Ausführung bis zum $n$-ten I/O-Syscall
/// 2. Simulierte Stromunterbrechung / Crash (Verwerfen ungesyncter Cache-Daten)
/// 3. Neustart / Recovery auf dem verbleibenden persistenten Zustand
/// 4. Prüfung der Konsistenzinvariante
#[derive(Debug, Clone, Default)]
pub struct CrashMatrixDriver {
    /// Optionaler Seed für deterministische Sektorumsortierung beim Crash.
    pub seed: Option<u64>,
}

impl CrashMatrixDriver {
    /// Erstellt einen neuen `CrashMatrixDriver`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Setzt einen deterministischen Seed für Crash-Sektor-Reordering.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Führt die vollständige Crash-Matrix für einen gegebenen Workload und Recovery-Check aus.
    ///
    /// - `workload`: Führt die mutierenden Operationen auf `FaultVfs` aus (z. B. Saga, Commit).
    /// - `verifier`: Führt nach dem simulierten Crash und Recovery die Integritätsprüfung aus.
    pub fn run<W, V, R>(&self, mut workload: W, mut verifier: V) -> Vec<CrashMatrixStepResult<R>>
    where
        W: FnMut(&FaultVfs) -> Result<()>,
        V: FnMut(&FaultVfs) -> R,
    {
        // 1. Baseline ermitteln
        let probe = FaultVfs::new();
        let _ = workload(&probe);
        let max_ops = probe.total_operations();

        let mut reports = Vec::with_capacity(max_ops + 1);

        // 2. Iteration über alle Schritte 0..=max_ops
        for crash_at in 0..=max_ops {
            let vfs = FaultVfs::new();
            let mut cfg = FaultConfig {
                crash_after_ops: if crash_at < max_ops {
                    Some(crash_at)
                } else {
                    None
                },
                seed: self.seed,
                ..Default::default()
            };
            if let Some(seed) = self.seed {
                cfg.sector_reorder = Some(SectorReorderConfig {
                    sector_size: 512,
                    seed: seed.wrapping_add(crash_at as u64),
                    persisted_fraction: 0.5,
                });
            }
            vfs.set_config(cfg);

            let res = workload(&vfs);
            let did_crash = res.is_err() || vfs.crashed.load(Ordering::SeqCst);
            let executed_ops = vfs.total_operations();

            // Simuliere Crash & Recovery (ungesyncte Cache-Daten verwerfen)
            vfs.crash_and_recover();

            // Führe Verifikation / Recovery auf persistentem Stand aus
            let recovery_outcome = verifier(&vfs);

            reports.push(CrashMatrixStepResult {
                crash_at_op: crash_at,
                executed_ops,
                did_crash,
                recovery_outcome,
            });
        }

        reports
    }
}

// ANCHOR[contextra-testkit:SOAK-DRIVER-001]
/// Bericht über einen Soak-Lauf (SPEC §15.1, D-109, T-SOAK-1).
#[derive(Debug, Clone)]
pub struct SoakReport {
    /// Gesamtzahl der durchlaufenen Iterationen.
    pub iterations: usize,
    /// Anzahl der injizierten Crashes.
    pub crashes_injected: usize,
    /// Anzahl der erfolgreichen Recovery-Durchläufe.
    pub recoveries_successful: usize,
    /// Verwendeter Start-Seed.
    pub seed: u64,
    /// Gesamtdauer des Laufs.
    pub elapsed: std::time::Duration,
}

/// Soak-Treiber für Dauer- und Robustheitsprüfungen (SPEC D-109, T-SOAK-1).
///
/// Ermöglicht konfigurierbare Laufzeit (z. B. Kurz-Smoketest in CI, stundenlange
/// Schleifen im Dauertest) mit deterministischem Pseudozufalls-Crash nach jedem Schritt.
#[derive(Debug, Clone)]
pub struct SoakDriver {
    /// Deterministischer Seed.
    pub seed: u64,
    /// Maximale Zeitdauer.
    pub max_duration: std::time::Duration,
    /// Maximale Anzahl von Iterationen.
    pub max_iterations: usize,
}

impl SoakDriver {
    /// Erstellt einen neuen `SoakDriver` mit vorgegebenem Seed und Grenzen.
    pub fn new(seed: u64, max_duration: std::time::Duration, max_iterations: usize) -> Self {
        Self {
            seed,
            max_duration,
            max_iterations,
        }
    }

    /// Führt die Soak-Schleife aus.
    ///
    /// In jeder Iteration:
    /// - Mutierende Operation mit Pseudozufalls-Crashwahrscheinlichkeit ausführen.
    /// - Nach Crash `crash_and_recover()` aufrufen.
    /// - `verify_fn` prüft, dass der Zustand konsistent ist (keine Korruption, kein unvollendeter Teilzustand).
    pub fn run<O, V>(&self, mut op_fn: O, mut verify_fn: V) -> Result<SoakReport>
    where
        O: FnMut(&FaultVfs, usize) -> Result<()>,
        V: FnMut(&FaultVfs, usize) -> Result<bool>,
    {
        let clock = ManualClock::new(1_700_000_000_000_000_000);
        let start_nanos = clock.now_nanos();
        let rng = SeededRng::new(self.seed);
        let vfs = FaultVfs::new();

        let mut iterations = 0;
        let mut crashes_injected = 0;
        let mut recoveries_successful = 0;

        while iterations < self.max_iterations {
            iterations += 1;
            clock.advance(std::time::Duration::from_millis(1));
            let elapsed_nanos = clock.now_nanos().saturating_sub(start_nanos);
            if std::time::Duration::from_nanos(elapsed_nanos) >= self.max_duration {
                break;
            }

            // Alle ~5 Operationen deterministisch einen Crash injizieren
            let will_crash = rng.next_u64().is_multiple_of(5);
            if will_crash {
                #[allow(clippy::cast_possible_truncation)]
                let ops = (rng.next_u64() as usize % 4) + 1;
                vfs.set_config(FaultConfig {
                    crash_after_ops: Some(ops),
                    seed: Some(self.seed.wrapping_add(iterations as u64)),
                    sector_reorder: Some(SectorReorderConfig {
                        sector_size: 512,
                        seed: self.seed.wrapping_add(iterations as u64),
                        persisted_fraction: 0.5,
                    }),
                    ..Default::default()
                });
            } else {
                vfs.set_config(FaultConfig {
                    seed: Some(self.seed.wrapping_add(iterations as u64)),
                    ..Default::default()
                });
            }

            let _ = op_fn(&vfs, iterations);

            if will_crash || vfs.crashed.load(Ordering::SeqCst) {
                crashes_injected += 1;
                vfs.crash_and_recover();
            }

            let consistent = verify_fn(&vfs, iterations)?;
            if consistent {
                recoveries_successful += 1;
            } else {
                return Err(ContextraError::Internal(format!(
                    "Soak-Test Konsistenzverletzung bei Iteration {iterations} (seed={})",
                    self.seed
                )));
            }
        }

        let elapsed_nanos = clock.now_nanos().saturating_sub(start_nanos);
        Ok(SoakReport {
            iterations,
            crashes_injected,
            recoveries_successful,
            seed: self.seed,
            elapsed: std::time::Duration::from_nanos(elapsed_nanos),
        })
    }
}

// ANCHOR[contextra-testkit:FAULT-VFS-TRAIT-IMPL-001]
impl Vfs for FaultVfs {
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        if opts.create_new && self.exists(path)? {
            return Err(ContextraError::Io(io::Error::new(
                ErrorKind::AlreadyExists,
                "File already exists with create_new",
            )));
        }

        if !self.exists(path)? {
            if opts.create || opts.create_new {
                if let Some(parent) = path.parent() {
                    self.create_dir_all(parent)?;
                }
                self.cache.write().insert(path.to_path_buf(), Vec::new());
            } else {
                return Err(ContextraError::NotFound(String::new()));
            }
        } else if opts.truncate {
            self.cache.write().insert(path.to_path_buf(), Vec::new());
        } else {
            // Falls in Medium, aber noch nicht im Cache: in den Cache laden
            let mut cache = self.cache.write();
            if !cache.contains_key(path) {
                if let Some(data) = self.medium.read().get(path) {
                    cache.insert(path.to_path_buf(), data.clone());
                }
            }
        }

        Ok(Box::new(FaultFile {
            path: path.to_path_buf(),
            vfs: self.clone(),
            options: opts.clone(),
            poisoned: AtomicBool::new(false),
        }))
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        self.check_fault(FaultOperation::Read, &self.read_count)
            .map_err(ContextraError::Io)?;
        let cache = self.cache.read();
        if let Some(data) = cache.get(path) {
            return Ok(data.clone());
        }
        let medium = self.medium.read();
        medium
            .get(path)
            .cloned()
            .ok_or_else(|| ContextraError::NotFound(String::new()))
    }

    fn read_at(&self, path: &Path, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let file = self.open(path, &VfsOpenOptions::read_only())?;
        file.read_at(buf, offset).map_err(ContextraError::Io)
    }

    fn read_verified(
        &self,
        path: &Path,
        offset: u64,
        len: usize,
    ) -> Result<(Vec<u8>, VerifiedVia)> {
        let file = self.open(path, &VfsOpenOptions::read_only())?;
        file.read_verified(offset, len).map_err(ContextraError::Io)
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

        // Cache und Medium aktualisieren (da write_atomic voll synchronisiert ist)
        self.cache.write().insert(path.to_path_buf(), data.to_vec());
        self.medium
            .write()
            .insert(path.to_path_buf(), data.to_vec());

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
        let mut cache = self.cache.write();
        let mut medium = self.medium.write();
        let c_removed = cache.remove(path).is_some();
        let m_removed = medium.remove(path).is_some();
        if c_removed || m_removed {
            Ok(())
        } else {
            Err(ContextraError::NotFound(String::new()))
        }
    }

    fn unlink(&self, path: &Path) -> Result<()> {
        self.remove_file(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        self.record_and_check_crash_point(FaultOperation::Rename, CrashTiming::Before, Some(from))
            .map_err(ContextraError::Io)?;
        self.check_fault(FaultOperation::Rename, &self.rename_count)
            .map_err(ContextraError::Io)?;

        let mut cache = self.cache.write();
        let data = cache
            .remove(from)
            .or_else(|| self.medium.read().get(from).cloned())
            .ok_or_else(|| ContextraError::NotFound(String::new()))?;

        if let Some(parent) = to.parent() {
            self.create_dir_all_internal(parent)
                .map_err(ContextraError::Io)?;
        }

        cache.insert(to.to_path_buf(), data.clone());

        let mut medium = self.medium.write();
        if medium.remove(from).is_some() {
            medium.insert(to.to_path_buf(), data);
        }
        drop(cache);
        drop(medium);

        self.record_and_check_crash_point(FaultOperation::Rename, CrashTiming::After, Some(to))
            .map_err(ContextraError::Io)?;

        Ok(())
    }

    fn exists(&self, path: &Path) -> Result<bool> {
        let cache = self.cache.read();
        if cache.contains_key(path) {
            return Ok(true);
        }
        let medium = self.medium.read();
        if medium.contains_key(path) {
            return Ok(true);
        }
        let dirs = self.dirs.read();
        Ok(dirs.contains(path))
    }

    fn len(&self, path: &Path) -> Result<u64> {
        self.check_fault(FaultOperation::Read, &self.read_count)
            .map_err(ContextraError::Io)?;
        let cache = self.cache.read();
        if let Some(data) = cache.get(path) {
            return Ok(data.len() as u64);
        }
        let medium = self.medium.read();
        medium
            .get(path)
            .map(|data| data.len() as u64)
            .ok_or_else(|| ContextraError::NotFound(String::new()))
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        let dirs = self.dirs.read();
        if !dirs.contains(path) && !path.as_os_str().is_empty() {
            return Err(ContextraError::NotFound(String::new()));
        }

        let cache = self.cache.read();
        let medium = self.medium.read();
        let mut entries = Vec::new();

        for file_path in cache.keys().chain(medium.keys()) {
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

    fn list(&self, path: &Path) -> Result<Vec<String>> {
        self.list_dir(path)
    }

    fn sync_dir(&self, path: &Path) -> Result<()> {
        self.record_and_check_crash_point(FaultOperation::Sync, CrashTiming::Before, Some(path))
            .map_err(ContextraError::Io)?;
        self.check_fault(FaultOperation::Sync, &self.sync_count)
            .map_err(ContextraError::Io)?;
        let dirs = self.dirs.read();
        let res = if dirs.contains(path) || path.as_os_str().is_empty() {
            Ok(())
        } else {
            Err(ContextraError::NotFound(String::new()))
        };
        drop(dirs);
        if res.is_ok() {
            self.record_and_check_crash_point(FaultOperation::Sync, CrashTiming::After, Some(path))
                .map_err(ContextraError::Io)?;
        }
        res
    }
}
