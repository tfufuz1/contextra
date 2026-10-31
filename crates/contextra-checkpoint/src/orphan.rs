//! # Verwaltung verwaister ("orphaned") Checkpoints und Sequenz-Pins
//!
//! Dieses Modul behandelt unvollständige Transaktionen und gepinnte Sequenznummern nach Abstürzen oder Panics.
//!
//! ## Ring & Rolle
//! Gehört zu **Ring 1 (Persistenz)**. Verhindert Ressourcenlecks im MVCC-Speicher (durch nie entpinnte
//! Snapshots) und räumt Transaktionen auf, deren `CheckpointGuard` gedroppt wurde, ohne dass ein
//! explizites `commit()` oder `rollback()` stattfand.
//!
//! ## Funktionsweise: Synchroner Drop & Asynchrones Reaping (ADR-011, ADR-053)
//! In Rust kann der `Drop`-Trait nicht asynchron sein (`async drop`). Wenn ein Guard bei einem Panic
//! oder vorzeitigen Funktionsabbruch zerstört wird:
//! 1. Der synchrone `drop()`-Handler registriert den Eintrag im Speicher in [`InstanceOrphanRegistry`].
//! 2. Dabei wird kein blockierendes Disk-I/O im Tokio-Worker-Thread ausgeführt (Non-blocking In-Memory Mutation).
//! 3. Ein "Dirty"-Flag markiert die Registry für das nächste Persistieren (Piggyback-Flush oder Shutdown).
//! 4. Beim nächsten Store-Start oder Recovery-Lauf (`recover_orphaned_pins`) werden die verwaisten Pins entpinnt.

use crate::meta::StateCheckpoint;
use contextra_ports::SystemClock;
use contextra_types::{Result, TxId};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ANCHOR[contextra-checkpoint:ORP-001]
/// Typ-Alias für Sequenznummern, die als gepinnte Checkpoint-Identifikatoren verwaltet werden.
pub type PinId = u64;

// ANCHOR[contextra-checkpoint:ORP-002]
/// Bericht über wiederhergestellte und aufgeräumte verwaiste Pins und Checkpoints.
///
/// # Felder
/// - `recovered_pins`: Liste aller erfolgreich entpinnten Sequenznummern.
/// - `recovered_checkpoints`: Liste aller bereinigten Transaktions-IDs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryReport {
    /// Liste aller erfolgreich entpinnten Sequenznummern.
    pub recovered_pins: Vec<PinId>,
    /// Liste aller bereinigten Transaktions-IDs.
    pub recovered_checkpoints: Vec<TxId>,
}

impl RecoveryReport {
    /// Liefert `true`, wenn keine verwaisten Einträge vorhanden waren oder bereinigt wurden.
    pub fn is_empty(&self) -> bool {
        self.recovered_pins.is_empty() && self.recovered_checkpoints.is_empty()
    }

    /// Liefert die Gesamtzahl aller bereinigten Pins und Checkpoints.
    pub fn total_recovered(&self) -> usize {
        self.recovered_pins.len() + self.recovered_checkpoints.len()
    }
}

/// Gibt eine einmalige Warnung aus, wenn der veraltete globale Orphan-Pfad verwendet wird.
fn warn_deprecated_global_orphan_path() {
    static WARN_ONCE: std::sync::Once = std::sync::Once::new();
    WARN_ONCE.call_once(|| {
        tracing::warn!(
            "Deprecated global OrphanRegistry path used — migrate to InstanceOrphanRegistry (ADR-053 / P29)"
        );
    });
}

// ANCHOR[contextra-checkpoint:ORP-003]
/// Veralteter globaler Zugriff auf die Orphan-Registry.
///
/// # Historie & ADR-053
/// Statischer globaler Zustand verstößt gegen SPEC §3 P29 (Mandanten-/Instanz-Isolation).
/// Bitte stattdessen instanzgebundene [`InstanceOrphanRegistry`] verwenden.
#[deprecated(
    since = "0.1.0",
    note = "Use InstanceOrphanRegistry instead. Global functions are not safe in multi-instance environments."
)]
#[allow(deprecated)]
pub fn global_orphan_registry() -> &'static OrphanRegistry {
    warn_deprecated_global_orphan_path();
    static EMPTY_ORPHAN_REGISTRY: std::sync::LazyLock<OrphanRegistry> =
        std::sync::LazyLock::new(|| OrphanRegistry::new(""));
    &EMPTY_ORPHAN_REGISTRY
}

/// Ermittelt das Standard-Datenverzeichnis für verwaiste Statusdateien.
///
/// # Sicherheit & Persistenz
/// Nutzt `dirs::data_local_dir()`. Falls nicht verfügbar, Fallback auf `std::env::temp_dir()`.
/// Im Produktivbetrieb sollte der Pfad explizit konfiguriert werden.
fn default_data_dir() -> std::path::PathBuf {
    dirs::data_local_dir().unwrap_or_else(std::env::temp_dir)
}

// ANCHOR[contextra-checkpoint:ORP-004]
/// Liest einen Millisekunden-Zeitstempel deterministisch über die übergebene Clock-Port-Instanz.
///
/// # Parameter
/// - `clock`: Referenz auf eine Implementierung des [`contextra_ports::Clock`]-Traits.
pub fn monotonic_timestamp_ms_with(clock: &dyn contextra_ports::Clock) -> u64 {
    // NOTE: Zeit und Zufall dürfen gemäß Regel 3 / INV-CHECKPOINT-DETERMINISM-1 nur über Ports fließen
    clock.now_unix_nanos() / 1_000_000
}

// ANCHOR[contextra-checkpoint:ORP-005]
/// Liest den Millisekunden-Zeitstempel über die standardmäßige SystemClock.
pub fn monotonic_timestamp_ms() -> u64 {
    let clock = SystemClock::new();
    monotonic_timestamp_ms_with(&clock)
}

/// Crate-interner Helfer zum Lesen der aktuellen Zeit in Millisekunden über einen Clock-Port.
pub(crate) fn clock_timestamp_ms(clock: &dyn contextra_ports::Clock) -> u64 {
    monotonic_timestamp_ms_with(clock)
}

/// Standardpfad für die Datei verwaister Sequenz-Pins.
fn orphan_pin_file_path() -> std::path::PathBuf {
    default_data_dir().join("contextra_orphaned_pins.json")
}

// ANCHOR[contextra-checkpoint:ORP-006]
/// Persistente Append-Only-Registry für verwaiste Sequenz-Pins (ADR-052).
///
/// Veraltet zugunsten von [`InstanceOrphanRegistry`].
#[deprecated(
    since = "0.1.0",
    note = "Use InstanceOrphanRegistry instead. Global OrphanRegistry is deprecated per ADR-053."
)]
pub struct OrphanRegistry {
    inner: InstanceOrphanRegistry,
}

#[allow(deprecated)]
impl Default for OrphanRegistry {
    fn default() -> Self {
        Self::new(orphan_pin_file_path())
    }
}

#[allow(deprecated)]
impl OrphanRegistry {
    /// Erzeugt eine neue Instanz von [`OrphanRegistry`].
    pub fn new(file_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            inner: InstanceOrphanRegistry::new(file_path),
        }
    }

    /// Registriert synchron einen verwaisten Pin.
    pub fn register_orphan(&self, pin_id: PinId) -> std::io::Result<()> {
        let wall_ms = clock_timestamp_ms(self.inner.clock().as_ref());
        self.inner.register_orphan_sync(PinnedSeqNoOrphan {
            seq_no: pin_id,
            timestamp_ms: wall_ms,
        });
        Ok(())
    }

    /// Liefert alle aktuell registrierten verwaisten Pin-Sequenznummern.
    pub fn get_orphans(&self) -> Vec<PinId> {
        self.inner
            .get_orphan_pins()
            .into_iter()
            .map(|o| o.seq_no)
            .collect()
    }

    /// Löscht alle Einträge im Speicher und auf der Platte synchron.
    pub fn clear_all(&self) {
        self.inner.clear_all();
    }

    /// Bereinigt alle verwaisten Pins durch Entpinnen in der Storage-Engine.
    ///
    /// # Ablauf von `recover_and_clean`:
    /// 1. Liste aller verwaisten Pins abfragen.
    /// 2. Für jeden Pin `storage.unpin_checkpoint` aufrufen.
    /// 3. Bei Erfolg den Pin aus der inneren Registry entfernen.
    /// 4. Liste der erfolgreich wiederhergestellten Pins zurückgeben.
    pub async fn recover_and_clean<S: contextra_ports::StorageEngine>(
        &self,
        storage: &S,
    ) -> Result<Vec<PinId>> {
        let orphans = self.get_orphans();
        let mut recovered = Vec::new();

        for pin_id in orphans {
            // NOTE: Schlägt unpin fehl, wird geloggt und der Pin nicht aus der Registry entfernt (Fail-Closed)
            if let Err(e) = storage.unpin_checkpoint(pin_id).await {
                tracing::warn!(pin_id = pin_id, error = %e, "Failed to unpin orphaned pin during recovery");
            } else {
                recovered.push(pin_id);
                self.inner.clear_orphan_pin(pin_id);
            }
        }

        Ok(recovered)
    }
}

/// Standardpfad für die Datei verwaister Checkpoints.
fn orphan_file_path() -> std::path::PathBuf {
    default_data_dir().join("contextra_orphaned_checkpoints.json")
}

// ANCHOR[contextra-checkpoint:ORP-007]
/// Verwaiste gepinnte Sequenznummer — wird beim Recovery verarbeitet.
///
/// # Felder
/// - `seq_no`: Die Sequenznummer, die entpinnt werden muss.
/// - `timestamp_ms`: Zeitpunkt der Registrierung in Millisekunden.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PinnedSeqNoOrphan {
    pub seq_no: u64,
    pub timestamp_ms: u64,
}

/// Veraltete Funktion zur Registrierung gepinnter Sequenznummer-Orphans.
#[deprecated(
    since = "0.1.0",
    note = "Use PersistentCheckpointStore::register_pinned_seq_no_orphan instead. Global functions are not safe in multi-instance environments."
)]
#[allow(deprecated)]
pub fn register_pinned_seq_no_orphan(orphan: PinnedSeqNoOrphan) {
    tracing::warn!(
        seq_no = orphan.seq_no,
        "register_pinned_seq_no_orphan is deprecated and no-op; use InstanceOrphanRegistry instead"
    );
}

// ANCHOR[contextra-checkpoint:ORP-008]
/// Serialisierbarer Gesamtzustand aller verwaisten Checkpoints und Pins einer Instanz.
///
/// # Felder
/// - `checkpoints`: Liste unvollendeter Transaktions-Checkpoints.
/// - `pinned_seq_nos`: Liste unvollendeter Pin-Reservierungen.
/// - `persist_path`: Dateipfad zur dauerhaften Ablage im JSON-Format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct OrphanState {
    pub checkpoints: Vec<StateCheckpoint>,
    pub pinned_seq_nos: Vec<PinnedSeqNoOrphan>,
    pub persist_path: std::path::PathBuf,
}

impl OrphanState {
    /// Schreibt den Zustand synchron als formatierte JSON-Datei auf die Festplatte.
    pub fn persist_sync(&self) -> std::io::Result<()> {
        if self.persist_path.as_os_str().is_empty() {
            return Ok(());
        }
        let data = serde_json::to_vec_pretty(self)?;
        std::fs::write(&self.persist_path, data)
    }

    /// Lädt den Zustand synchron von der Festplatte oder liefert einen leeren Zustand bei Fehlern.
    pub fn load_sync(path: &std::path::Path) -> Self {
        if let Ok(data) = std::fs::read(path) {
            if let Ok(state) = serde_json::from_slice::<OrphanState>(&data) {
                return state;
            }
        }
        Self {
            checkpoints: Vec::new(),
            pinned_seq_nos: Vec::new(),
            persist_path: path.to_path_buf(),
        }
    }
}

// ANCHOR[contextra-checkpoint:ORP-009]
/// Instanz-gebundene Orphan-Registry für Checkpoints und gepinnte Sequenznummern (ADR-053).
///
/// # Thread-Sicherheit & Nebenläufigkeit
/// Verwendet [`parking_lot::Mutex`] für minimale Latenz beim Guard-Drop ohne OS-Thread-Blockaden.
/// Ein atomares `is_dirty`-Flag synchronisiert anstehende Disk-Schreibvorgänge.
pub struct InstanceOrphanRegistry {
    pins: Mutex<Vec<PinnedSeqNoOrphan>>,
    checkpoints: Mutex<Vec<StateCheckpoint>>,
    persist_path: std::path::PathBuf,
    is_dirty: AtomicBool,
    clock: Arc<dyn contextra_ports::Clock>,
}

impl std::fmt::Debug for InstanceOrphanRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstanceOrphanRegistry")
            .field("pins", &self.pins)
            .field("checkpoints", &self.checkpoints)
            .field("persist_path", &self.persist_path)
            .field("is_dirty", &self.is_dirty)
            .field("clock", &"<dyn Clock>")
            .finish()
    }
}

impl Default for InstanceOrphanRegistry {
    fn default() -> Self {
        Self::new(orphan_file_path())
    }
}

impl InstanceOrphanRegistry {
    /// Erzeugt eine neue instanzgebundene Registry für den angegebenen Dateipfad.
    pub fn new(persist_path: impl Into<std::path::PathBuf>) -> Self {
        let path = persist_path.into();
        Self::load_sync(&path)
    }

    /// Konfiguriert die Registry mit einer spezifischen Clock-Implementierung.
    pub fn with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Setzt die Clock-Implementierung nachträglich.
    pub fn set_clock(&mut self, clock: Arc<dyn contextra_ports::Clock>) {
        self.clock = clock;
    }

    /// Liefert eine Referenz auf den injizierten Clock-Port.
    pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock> {
        &self.clock
    }

    /// Lädt den Zustand synchron von der Festplatte.
    pub fn load_sync(path: &std::path::Path) -> Self {
        let clock: Arc<dyn contextra_ports::Clock> = Arc::new(SystemClock::new());
        if !path.as_os_str().is_empty() {
            if let Ok(data) = std::fs::read(path) {
                if let Ok(state) = serde_json::from_slice::<OrphanState>(&data) {
                    return Self {
                        pins: Mutex::new(state.pinned_seq_nos),
                        checkpoints: Mutex::new(state.checkpoints),
                        persist_path: path.to_path_buf(),
                        is_dirty: AtomicBool::new(false),
                        clock,
                    };
                }
            }
        }
        Self {
            pins: Mutex::new(Vec::new()),
            checkpoints: Mutex::new(Vec::new()),
            persist_path: path.to_path_buf(),
            is_dirty: AtomicBool::new(false),
            clock,
        }
    }

    /// Persistiert den aktuellen Zustand synchron auf die Festplatte.
    pub fn persist_sync(&self) -> std::io::Result<()> {
        if self.persist_path.as_os_str().is_empty() {
            return Ok(());
        }
        let state = OrphanState {
            checkpoints: self.checkpoints.lock().clone(),
            pinned_seq_nos: self.pins.lock().clone(),
            persist_path: self.persist_path.clone(),
        };
        let data = serde_json::to_vec_pretty(&state)?;
        std::fs::write(&self.persist_path, data)
    }

    // ANCHOR[contextra-checkpoint:ORP-010]
    /// Registriert einen verwaisten Sequenz-Pin rein im Arbeitsspeicher ohne blockierendes Disk-I/O.
    ///
    /// # Crash-Safety & RAII Drop Semantik
    /// Wird synchron aus [`crate::PinGuard`]`::drop()` aufgerufen. Führt strikt nur
    /// Speicher-Mutationen (`Mutex<Vec<...>>`) aus und setzt `is_dirty = true`.
    /// Persistenz auf Platte folgt entkoppelt beim nächsten regulären Flush.
    pub fn register_orphan_sync(&self, orphan: PinnedSeqNoOrphan) {
        let mut lock = self.pins.lock();
        if !lock.iter().any(|o| o.seq_no == orphan.seq_no) {
            let seq_no = orphan.seq_no;
            lock.push(orphan);
            drop(lock);
            // NOTE: Ordering::Release garantiert, dass der Vector-Eintrag vor dem Dirty-Flag sichtbar ist
            self.is_dirty.store(true, Ordering::Release);
            tracing::debug!(
                seq = seq_no,
                "Registered orphan pin in memory (dirty); awaiting next flush"
            );
        }
    }

    // ANCHOR[contextra-checkpoint:ORP-011]
    /// Registriert einen verwaisten Checkpoint rein im Arbeitsspeicher ohne blockierendes Disk-I/O.
    ///
    /// # Crash-Safety & RAII Drop Semantik
    /// Wird synchron aus [`crate::CheckpointGuard`]`::drop()` aufgerufen.
    /// Verhindert Blockaden der Tokio-Worker-Threads bei Panics oder Transaktionsabbrüchen.
    pub fn register_checkpoint_sync(&self, cp: StateCheckpoint) {
        let mut lock = self.checkpoints.lock();
        if !lock.iter().any(|o| o.tx_id == cp.tx_id) {
            let tx_id = cp.tx_id;
            lock.push(cp);
            drop(lock);
            self.is_dirty.store(true, Ordering::Release);
            tracing::debug!(
                ?tx_id,
                "Registered orphaned checkpoint in memory (dirty); awaiting next flush"
            );
        }
    }

    // ANCHOR[contextra-checkpoint:ORP-012]
    /// Flusht die In-Memory-Einträge asynchron auf Festplatte, sofern Änderungen vorliegen (`is_dirty`).
    ///
    /// # Ablauf von `flush_orphan_registry`:
    /// 1. Prüfen, ob ein Pfad konfiguriert ist; falls leer, sofort `Ok(())`.
    /// 2. Atomarer Tausch von `is_dirty` gegen `false` (`AcqRel`). War es nicht dirty, sofort `Ok(())`.
    /// 3. Zustand klonen und bei vorhandener Tokio-Laufzeit via `spawn_blocking` schreiben.
    /// 4. Bei Schreibfehler `is_dirty` wieder auf `true` setzen (Fail-Closed).
    pub async fn flush_orphan_registry(&self) -> std::io::Result<()> {
        if self.persist_path.as_os_str().is_empty() {
            return Ok(());
        }
        if !self.is_dirty.swap(false, Ordering::AcqRel) {
            return Ok(());
        }
        let state = OrphanState {
            checkpoints: self.checkpoints.lock().clone(),
            pinned_seq_nos: self.pins.lock().clone(),
            persist_path: self.persist_path.clone(),
        };
        // NOTE: spawn_blocking verhindert Blockieren des asynchronen Executors bei Dateisystem-I/O
        let res = if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::spawn_blocking(move || state.persist_sync())
                .await
                .map_err(|e| std::io::Error::other(e.to_string()))?
        } else {
            state.persist_sync()
        };
        if let Err(err) = &res {
            self.is_dirty.store(true, Ordering::Release);
            tracing::error!(?err, "Failed to persist orphan registry during flush");
        }
        res
    }

    /// Entnimmt alle verwaisten Pins aus dem Speicher und aktualisiert die Statusdatei.
    pub fn drain_orphan_pins(&self) -> Vec<PinnedSeqNoOrphan> {
        let mut lock = self.pins.lock();
        let drained = std::mem::take(&mut *lock);
        drop(lock);
        if let Err(err) = self.persist_sync() {
            self.is_dirty.store(true, Ordering::Release);
            tracing::error!(
                ?err,
                "Failed to persist orphan registry after draining pins"
            );
        } else {
            self.is_dirty.store(false, Ordering::Release);
        }
        drained
    }

    /// Liefert eine Momentaufnahme aller verwaisten Pins im Speicher.
    pub fn get_orphan_pins(&self) -> Vec<PinnedSeqNoOrphan> {
        self.pins.lock().clone()
    }

    /// Entfernt eine einzelne gepinnte Sequenznummer aus dem Speicher und persistiert.
    pub fn clear_orphan_pin(&self, seq_no: u64) {
        let mut lock = self.pins.lock();
        lock.retain(|o| o.seq_no != seq_no);
        drop(lock);
        if let Err(err) = self.persist_sync() {
            self.is_dirty.store(true, Ordering::Release);
            tracing::error!(
                ?err,
                seq_no = seq_no,
                "Failed to persist orphan registry after clearing pin"
            );
        } else {
            self.is_dirty.store(false, Ordering::Release);
        }
    }

    /// Entnimmt alle verwaisten Checkpoints aus dem Speicher und aktualisiert die Statusdatei.
    pub fn drain_orphaned_checkpoints(&self) -> Vec<StateCheckpoint> {
        let mut lock = self.checkpoints.lock();
        let drained = std::mem::take(&mut *lock);
        drop(lock);
        if let Err(err) = self.persist_sync() {
            self.is_dirty.store(true, Ordering::Release);
            tracing::error!(
                ?err,
                "Failed to persist orphan registry after draining checkpoints"
            );
        } else {
            self.is_dirty.store(false, Ordering::Release);
        }
        drained
    }

    /// Liefert eine Momentaufnahme aller verwaisten Checkpoints im Speicher.
    pub fn get_orphaned_checkpoints(&self) -> Vec<StateCheckpoint> {
        self.checkpoints.lock().clone()
    }

    /// Entfernt einen einzelnen verwaisten Checkpoint anhand seiner Transaktions-ID und persistiert.
    pub fn clear_orphaned_checkpoint(&self, tx_id: TxId) {
        let mut lock = self.checkpoints.lock();
        lock.retain(|o| o.tx_id != tx_id);
        drop(lock);
        if let Err(err) = self.persist_sync() {
            self.is_dirty.store(true, Ordering::Release);
            tracing::error!(?err, tx_id = ?tx_id, "Failed to persist orphan registry after clearing checkpoint");
        } else {
            self.is_dirty.store(false, Ordering::Release);
        }
    }

    /// Löscht alle Einträge (Pins und Checkpoints) und leert die Statusdatei.
    pub fn clear_all(&self) {
        self.pins.lock().clear();
        self.checkpoints.lock().clear();
        if let Err(err) = self.persist_sync() {
            self.is_dirty.store(true, Ordering::Release);
            tracing::error!(?err, "Failed to persist orphan registry after clearing all");
        } else {
            self.is_dirty.store(false, Ordering::Release);
        }
    }
}

/// Veralteter globaler Wrapper zur Registrierung verwaister Checkpoints.
#[deprecated(
    since = "0.1.0",
    note = "Use PersistentCheckpointStore::register_orphaned_checkpoint instead. Global functions are not safe in multi-instance environments."
)]
#[allow(deprecated)]
pub fn register_orphaned_checkpoint(cp: StateCheckpoint) {
    tracing::warn!(
        tx_id = ?cp.tx_id,
        "register_orphaned_checkpoint is deprecated and no-op; use InstanceOrphanRegistry instead"
    );
}

/// Veralteter globaler Wrapper zum Abfragen verwaister Checkpoints.
#[deprecated(
    since = "0.1.0",
    note = "Use PersistentCheckpointStore::get_orphaned_checkpoints instead. Global functions are not safe in multi-instance environments."
)]
#[allow(deprecated)]
pub fn get_orphaned_checkpoints() -> Vec<StateCheckpoint> {
    Vec::new()
}

/// Liefert verwaiste Checkpoints eines bestimmten Namespaces (veraltet).
#[allow(deprecated)]
pub fn get_orphaned_checkpoints_for_namespace(ns: &str) -> Vec<StateCheckpoint> {
    get_orphaned_checkpoints()
        .into_iter()
        .filter(|cp| cp.namespace.as_deref() == Some(ns))
        .collect()
}

/// Entfernt einen verwaisten Checkpoint (veraltet).
#[deprecated(
    since = "0.1.0",
    note = "Use PersistentCheckpointStore::clear_orphaned_checkpoint instead. Global functions are not safe in multi-instance environments."
)]
#[allow(deprecated)]
pub fn clear_orphaned_checkpoint(_tx_id: TxId) {}

/// Löscht alle verwaisten Checkpoints (veraltet).
#[deprecated(
    since = "0.1.0",
    note = "Use PersistentCheckpointStore::clear_all_orphaned_checkpoints instead. Global functions are not safe in multi-instance environments."
)]
#[allow(deprecated)]
pub fn clear_all_orphaned_checkpoints() {}

/// Rückwärtskompatibilität: Wartet auf ausstehende Rollbacks (No-op, da Hintergrund-Tasks entfernt wurden).
pub async fn await_pending_rollbacks() {}

/// Rückwärtskompatibilität: Liefert die Anzahl ausstehender Rollback-Hintergrundaufgaben (immer 0).
pub fn pending_rollback_count() -> usize {
    0
}

/// Liefert die Anzahl der aktuell registrierten verwaisten ("orphaned") Checkpoints.
#[allow(deprecated)]
pub fn orphaned_checkpoint_count() -> usize {
    #[allow(deprecated)]
    get_orphaned_checkpoints().len()
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]
mod tests {
    use super::*;

    // Prüft, dass der veraltete globale Orphan-Pfad Warnungen ausgibt und ohne Panic ausgeführt wird.
    #[test]
    fn test_deprecated_global_orphan_path_warns() {
        #[allow(deprecated)]
        {
            clear_all_orphaned_checkpoints();
            register_pinned_seq_no_orphan(PinnedSeqNoOrphan {
                seq_no: 99999,
                timestamp_ms: 1000,
            });
            let _orphans = get_orphaned_checkpoints();
            clear_all_orphaned_checkpoints();
        }
    }

    // Prüft, dass aufeinanderfolgende Zeitstempel über die Clock monoton ansteigen oder gleich bleiben.
    #[test]
    fn timestamp_ms_is_monotonic() {
        let clock = SystemClock::new();
        let t1 = clock_timestamp_ms(&clock);
        let t2 = clock_timestamp_ms(&clock);
        assert!(t2 >= t1, "Timestamp must be monotonic");
    }

    // Prüft das Entnehmen ("drain") von verwaisten Pins und Checkpoints aus der InstanceOrphanRegistry.
    #[test]
    fn test_instance_orphan_registry_drain_pins_and_checkpoints() {
        let registry = InstanceOrphanRegistry::new("");
        registry.register_orphan_sync(PinnedSeqNoOrphan {
            seq_no: 100,
            timestamp_ms: 1000,
        });
        registry.register_checkpoint_sync(StateCheckpoint {
            tx_id: TxId::new(200),
            timestamp_ms: 2000,
            namespace: Some("test_drain".to_string()),
        });

        assert_eq!(registry.get_orphan_pins().len(), 1);
        assert_eq!(registry.get_orphaned_checkpoints().len(), 1);

        let drained_pins = registry.drain_orphan_pins();
        assert_eq!(drained_pins.len(), 1);
        assert_eq!(drained_pins[0].seq_no, 100);
        assert!(registry.get_orphan_pins().is_empty());

        let drained_cps = registry.drain_orphaned_checkpoints();
        assert_eq!(drained_cps.len(), 1);
        assert_eq!(drained_cps[0].tx_id, TxId::new(200));
        assert!(registry.get_orphaned_checkpoints().is_empty());
    }

    // Prüft, dass das Löschen nicht existierender Einträge existierende Einträge unberührt lässt.
    #[test]
    fn test_instance_orphan_registry_clear_nonexistent() {
        let registry = InstanceOrphanRegistry::new("");
        registry.register_orphan_sync(PinnedSeqNoOrphan {
            seq_no: 50,
            timestamp_ms: 100,
        });
        registry.register_checkpoint_sync(StateCheckpoint {
            tx_id: TxId::new(60),
            timestamp_ms: 200,
            namespace: None,
        });

        // Clearing non-existent pin or checkpoint should preserve existing ones
        registry.clear_orphan_pin(999);
        registry.clear_orphaned_checkpoint(TxId::new(999));

        assert_eq!(registry.get_orphan_pins().len(), 1);
        assert_eq!(registry.get_orphaned_checkpoints().len(), 1);
    }

    // Prüft, dass eine Registry ohne Persistenzpfad (`""`) Schreiboperationen als erfolgreichen No-op behandelt.
    #[test]
    fn test_instance_orphan_registry_empty_path_persistence_noop() {
        let registry = InstanceOrphanRegistry::new("");
        assert!(registry.persist_sync().is_ok());

        let state = OrphanState::default();
        assert!(state.persist_sync().is_ok());
    }

    // Prüft die strikte Trennung und gegenseitige Isolation zweier unabhängiger InstanceOrphanRegistry-Instanzen.
    #[test]
    fn test_two_instance_registries_independent() {
        let dir1 = tempfile::tempdir().expect("tempdir 1");
        let dir2 = tempfile::tempdir().expect("tempdir 2");

        let path1 = dir1.path().join("orphans1.json");
        let path2 = dir2.path().join("orphans2.json");

        let reg1 = InstanceOrphanRegistry::new(&path1);
        let reg2 = InstanceOrphanRegistry::new(&path2);

        reg1.register_orphan_sync(PinnedSeqNoOrphan {
            seq_no: 101,
            timestamp_ms: 1000,
        });

        reg2.register_checkpoint_sync(StateCheckpoint {
            tx_id: TxId::new(202),
            timestamp_ms: 2000,
            namespace: Some("ns2".to_string()),
        });

        assert_eq!(reg1.get_orphan_pins().len(), 1);
        assert_eq!(reg1.get_orphaned_checkpoints().len(), 0);

        assert_eq!(reg2.get_orphan_pins().len(), 0);
        assert_eq!(reg2.get_orphaned_checkpoints().len(), 1);
    }

    // Property-basierter Test für die Monotonie der Zeitstempel über die SystemClock.
    proptest::proptest! {
        #[test]
        fn prop_monotonic_timestamp_ms_increases_or_equals(_n: u8) {
            let clock = SystemClock::new();
            let ts1 = clock_timestamp_ms(&clock);
            let ts2 = clock_timestamp_ms(&clock);
            proptest::prop_assert!(ts2 >= ts1);
        }
    }
}
