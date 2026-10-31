//! # Hardlink-Cloner für physische SSTables mit MVCC-Snapshot-Pinning
//!
//! Dieses Modul ermöglicht schnelles Klonen von SSTables auf Dateisystem-Ebene via Hardlinks.
//!
//! ## Ring & Rolle
//! Gehört zu **Ring 1 (Persistenz)**. SSTables (Sorted String Tables) sind die unveränderlichen
//! Datendateien der LSM-Speicherengine. Dieses Modul erlaubt O(1) Copy-on-Write Klone beim
//! Forking von Agenten-Zuständen.
//!
//! ## Architektur & Copy-on-Write
//! Erzeugt einen physischen Hardlink-Klon aller sichtbaren SSTables einer gegebenen Sequenznummer
//! in ein Zielverzeichnis. Während des gesamten Klonvorgangs wird ein aktiver Pin in der
//! [`SnapshotRegistry`] gehalten, damit die LSM-Compaction oder Garbage Collection (GC) keine
//! SSTables löscht.
//!
//! ## Copy-on-Write und Dateikopie-Fallback
//! Hardlinks werden für blitzschnelle Klone bevorzugt. Schlagen Hardlinks fehl (z. B. über
//! Partitionen/Mount-Points hinweg, Erreichen des NTFS-Hardlink-Limits unter Windows oder
//! nicht-unterstützte Dateisysteme), schlägt der Vorgang automatisch auf eine vollständige
//! Byte-für-Byte-Kopie (`fs::copy`) zurück.

#![allow(unexpected_cfgs)]

// FILE-CONTEXT
// STAND: 2026-09-28T00:00:00Z
// ZWECK: Hardlink-Cloner für Agent-Forking und SSTable CoW-Klone.
// INVARIANTEN: Zero-Panic, Async-I/O via tokio::fs, SnapshotRegistry-Pinning während Klonen.

use contextra_mvcc::SnapshotRegistry;
use contextra_types::{ContextraError, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(not(loom))]
use tokio::fs;

#[cfg(loom)]
mod fs {
    use std::path::Path;

    pub async fn create_dir_all<P: AsRef<Path>>(_path: P) -> std::io::Result<()> {
        Ok(())
    }

    pub async fn read_dir<P: AsRef<Path>>(_path: P) -> std::io::Result<LoomReadDir> {
        Ok(LoomReadDir)
    }

    pub async fn metadata<P: AsRef<Path>>(_path: P) -> std::io::Result<LoomMetadata> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "loom mock",
        ))
    }

    pub async fn remove_file<P: AsRef<Path>>(_path: P) -> std::io::Result<()> {
        Ok(())
    }

    pub async fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(
        _from: P,
        _to: Q,
    ) -> std::io::Result<()> {
        Ok(())
    }

    pub async fn copy<P: AsRef<Path>, Q: AsRef<Path>>(_from: P, _to: Q) -> std::io::Result<u64> {
        Ok(0)
    }

    pub struct LoomReadDir;
    impl LoomReadDir {
        pub async fn next_entry(&mut self) -> std::io::Result<Option<LoomDirEntry>> {
            Ok(None)
        }
    }

    pub struct LoomDirEntry;
    impl LoomDirEntry {
        pub fn path(&self) -> std::path::PathBuf {
            std::path::PathBuf::new()
        }
    }

    pub struct LoomMetadata;
}

// ANCHOR[contextra-checkpoint:CLONE-001]
/// Ergebnisstruktur einer durchgeführten Hardlink-Klon-Operation für SSTables.
///
/// # Zweck
/// Liefert dem Aufrufer die Sequenznummer sowie die Liste aller erfolgreich verknüpften
/// oder kopierten Zieldateien.
///
/// # Felder
/// - `source_seq_no`: Die gepinnte und geklonte Quell-Sequenznummer.
/// - `linked_files`: Liste der im Zielverzeichnis erzeugten Pfade.
/// - `wal_tail_offset`: Offset im Write-Ahead-Log, ab dem der geklonte Zustand neue Mutationen akzeptiert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardlinkCloneResult {
    /// Die gepinnte und geklonte Quell-Sequenznummer.
    pub source_seq_no: u64,
    /// Liste der Zielpfade, die als Hardlinks oder Kopien erzeugt wurden.
    pub linked_files: Vec<PathBuf>,
    /// WAL-Tail-Offset, an dem der Klon beginnt, neue Mutationen anzunehmen.
    ///
    /// **Known Gap / Backlog Note:**
    /// Real-time WAL tail offset extraction requires active coordination with the WAL manager
    /// in `contextra-store`. Currently set to `None` as a best-effort explicit placeholder
    /// to avoid returning speculative or unverified offset values.
    pub wal_tail_offset: Option<u64>,
}

// ANCHOR[contextra-checkpoint:CLONE-002]
/// Trait für die Schnittstelle zum physischen SSTable-Klonen via Hardlinks.
///
/// # Zweck
/// Abstrahiert das Dateisystem-Klonen für Tests und verschiedene Plattformen.
///
/// # Windows NTFS Limit
/// Unter Windows ist `CreateHardLink` NTFS-exklusiv und auf maximal **1023 Links** pro Datei limitiert.
/// Bei Überschreitung oder dateisystemübergreifenden Links wird [`ContextraError::CrossDeviceLink`] gemeldet.
pub trait CheckpointHardlinkCloner: Send + Sync {
    /// Klont alle für `seq_no` sichtbaren SSTable-Dateien im Quellverzeichnis ins Zielverzeichnis.
    ///
    /// Hält während der Operation einen aktiven `SnapshotRegistry`-Pin auf `seq_no`.
    ///
    /// # Parameter
    /// - `seq_no`: Sequenznummer des MVCC-Snapshots.
    /// - `source_dir`: Quellverzeichnis mit bestehenden `.sst`-Dateien.
    /// - `target_dir`: Zielverzeichnis für die Klone.
    /// - `snapshot_registry`: Referenz auf die MVCC-Snapshot-Registry zum Pinning.
    ///
    /// # Fehler
    /// Gibt einen Fehler zurück bei Dateisystemproblemen oder unauflösbaren Link-Konflikten.
    fn clone_sstable_hardlinks<'a>(
        &'a self,
        seq_no: u64,
        source_dir: &'a Path,
        target_dir: &'a Path,
        snapshot_registry: &'a Arc<SnapshotRegistry>,
    ) -> contextra_ports::BoxFuture<'a, Result<HardlinkCloneResult>>;
}

// ANCHOR[contextra-checkpoint:CLONE-003]
/// Produktive Implementierung von [`CheckpointHardlinkCloner`] basierend auf `tokio::fs`.
///
/// Nutzt asynchrone Dateisystem-Operationen ohne den Haupt-Thread zu blockieren.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultHardlinkCloner;

impl DefaultHardlinkCloner {
    /// Erzeugt eine neue Instanz von [`DefaultHardlinkCloner`].
    pub fn new() -> Self {
        Self
    }
}

impl CheckpointHardlinkCloner for DefaultHardlinkCloner {
    // ANCHOR[contextra-checkpoint:CLONE-004]
    // Ablauf von `clone_sstable_hardlinks`:
    // 1. Snapshot in der `SnapshotRegistry` pinnen (_guard verhindert Compactor-Löschung).
    // 2. Zielverzeichnis anlegen falls noch nicht vorhanden.
    // 3. Quellverzeichnis nach `.sst`-Dateien durchsuchen.
    // 4. Für jede Datei bestehende Zieldatei entfernen und Hardlink setzen.
    // 5. Schlägt der Hardlink fehl, Fallback auf `fs::copy`.
    // 6. Liegt ein Cross-Device-Fehler vor, fehlerhafte Operation als `cross_device_link` melden.
    // 7. Zieldateien deterministisch sortieren und Ergebnis zurückgeben.
    fn clone_sstable_hardlinks<'a>(
        &'a self,
        seq_no: u64,
        source_dir: &'a Path,
        target_dir: &'a Path,
        snapshot_registry: &'a Arc<SnapshotRegistry>,
    ) -> contextra_ports::BoxFuture<'a, Result<HardlinkCloneResult>> {
        Box::pin(async move {
            // 1. Pin snapshot in SnapshotRegistry for the duration of the clone operation
            // NOTE: Der Guard wird über die gesamte Future gehalten, damit kein GC-Lauf SSTables löscht
            let _guard = snapshot_registry.register(seq_no);

            // 2. Ensure target directory exists
            fs::create_dir_all(target_dir)
                .await
                .map_err(ContextraError::Io)?;

            // 3. Scan source directory for SSTable files (.sst)
            let mut read_dir = fs::read_dir(source_dir).await.map_err(ContextraError::Io)?;

            let mut linked_files = Vec::new();

            while let Some(entry) = read_dir.next_entry().await.map_err(ContextraError::Io)? {
                let path = entry.path();
                if path.is_file() {
                    let is_sst = path
                        .extension()
                        .is_some_and(|ext| ext == "sst" || ext == "tmp");
                    if is_sst
                        || path
                            .file_name()
                            .is_some_and(|f| f.to_string_lossy().contains(".sst"))
                    {
                        let file_name = path.file_name().ok_or_else(|| {
                            ContextraError::invalid_input("Invalid file name in SST directory")
                        })?;
                        let target_path = target_dir.join(file_name);

                        // Remove existing link if present
                        if fs::metadata(&target_path).await.is_ok() {
                            let _ = fs::remove_file(&target_path).await;
                        }

                        // NOTE: Hardlink ist O(1). Bei Fehlschlag (z. B. anderes Volume) greift Fallback-Kopie.
                        if let Err(io_err) = fs::hard_link(&path, &target_path).await {
                            tracing::warn!(
                                source = %path.display(),
                                target = %target_path.display(),
                                error = %io_err,
                                "Hardlink creation failed; falling back to byte-for-byte file copy"
                            );
                            if let Err(copy_err) = fs::copy(&path, &target_path).await {
                                if is_cross_device_error(&io_err) {
                                    return Err(ContextraError::cross_device_link(
                                        path.to_string_lossy(),
                                        target_path.to_string_lossy(),
                                    ));
                                } else {
                                    return Err(ContextraError::Io(copy_err));
                                }
                            }
                        }

                        linked_files.push(target_path);
                    }
                }
            }

            // Sort linked files for deterministic output ordering
            // NOTE: Sortierung sichert plattformunabhängige, reproduzierbare Ergebnislisten
            linked_files.sort();

            Ok(HardlinkCloneResult {
                source_seq_no: seq_no,
                linked_files,
                wal_tail_offset: None,
            })
        })
    }
}

// ANCHOR[contextra-checkpoint:CLONE-005]
/// Hilfsfunktion zur Erkennung plattformspezifischer Cross-Device-Link-Fehler (z. B. EXDEV).
///
/// # Ablauf:
/// Prüft sowohl `ErrorKind::CrossesDevices` als auch die systemeigenen Fehlercodes (Unix: 18, Windows: 17).
fn is_cross_device_error(err: &std::io::Error) -> bool {
    if err.kind() == std::io::ErrorKind::CrossesDevices {
        return true;
    }
    if let Some(code) = err.raw_os_error() {
        // NOTE: Unix EXDEV (18), Windows ERROR_NOT_SAME_DEVICE (17)
        return code == 18 || code == 17;
    }
    false
}
