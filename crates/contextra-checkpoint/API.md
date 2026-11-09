# Öffentliche API: `contextra-checkpoint`

Vollständige Übersicht aller öffentlichen Typen, Traits, Structs und Funktionen in `contextra-checkpoint`.

## `src/guard.rs`

| Signatur | Beschreibung |
|---|---|
| `pub struct PinGuard<S: contextra_ports::StorageEngine> ` |  |
| `pub async fn pin(` | Pinnt eine Sequenznummer in der Storage-Engine unter Verwendung der Clock aus der Orphan-Registry. |
| `pub async fn pin_with_clock(` | Pinnt eine Sequenznummer mit explizit übergebenem Clock-Port. |
| `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock> ` | Liefert eine Referenz auf den injizierten Clock-Port. |
| `pub fn defuse(mut self) ` | Entschärft den Guard nach erfolgreicher Persistierung, sodass die Sequenznummer gepinnt bleibt. |
| `pub async fn unpin(mut self) -> Result<()> ` | Entpinnt die Sequenznummer explizit und asynchron bei einem abgefangenen Fehler. |
| `pub struct CheckpointGuard<S: contextra_ports::StorageEngine> ` |  |
| `pub fn new(checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>) -> Self ` | Erzeugt einen neuen `CheckpointGuard` mit Standard-SystemClock. |
| `pub fn with_clock(` | Erzeugt einen neuen `CheckpointGuard` mit injiziertem Clock-Port. |
| `pub fn with_registry(` | Erzeugt einen neuen `CheckpointGuard` mit vorhandener Orphan-Registry. |
| `pub fn with_registry_and_counter(` | Erzeugt einen neuen `CheckpointGuard` mit vorhandener Orphan-Registry und Zähler für übersprungene Rollbacks. |
| `pub fn with_registry_counter_and_clock(` | Vollständiger interner Konstruktor mit allen Parametern. |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self ` | Ergänzt den Guard um eine Metrik-Senke zur Erfassung von Ausführungsdauern. |
| `pub fn with_metrics_sink_opt(` | Ergänzt den Guard optional um eine Metrik-Senke. |
| `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock> ` | Liefert den verwendeten Clock-Port. |
| `pub async fn for_agent_step(storage: Arc<S>, tx: TxId) -> Result<Self> ` | Erstellt einen neuen CheckpointGuard für einen Agenten-Schritt mit SystemClock. |
| `pub async fn for_agent_step_with_registry(` | Erstellt einen CheckpointGuard für einen Agenten-Schritt mit spezifischer Orphan-Registry. |
| `pub async fn for_agent_step_with_clock(` | Erstellt einen CheckpointGuard für einen Agenten-Schritt mit speziellem Clock-Port. |
| `pub fn checkpoint(&self) -> Result<&StateCheckpoint> ` | Liefert eine Referenz auf den enthaltenen Checkpoint, sofern noch nicht konsumiert. |
| `pub fn commit(mut self) -> Result<StateCheckpoint> ` | Überführt den Guard in einen dauerhaften Zustand (Commit) und konsumiert ihn. |
| `pub async fn rollback(mut self) -> Result<()> ` | Führt ein manuelles, asynchrones Rollback des Checkpoints aus. |
| `pub fn rollback_blocking(mut self) -> Result<()> ` | Führt ein synchrones, blockierendes Rollback des Checkpoints aus. |

## `src/hardlink_cloner.rs`

| Signatur | Beschreibung |
|---|---|
| `pub trait SnapshotPinner: Send + Sync ` | Trait zur Abstraktion von Snapshot-Pinning-Mechanismen während des SSTable-Klonens. |
| `pub async fn create_dir_all<P: AsRef<Path>>(_path: P) -> std::io::Result<()> ` |  |
| `pub async fn read_dir<P: AsRef<Path>>(_path: P) -> std::io::Result<LoomReadDir> ` |  |
| `pub async fn metadata<P: AsRef<Path>>(_path: P) -> std::io::Result<LoomMetadata> ` |  |
| `pub async fn remove_file<P: AsRef<Path>>(_path: P) -> std::io::Result<()> ` |  |
| `pub async fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>(` |  |
| `pub async fn copy<P: AsRef<Path>, Q: AsRef<Path>>(_from: P, _to: Q) -> std::io::Result<u64> ` |  |
| `pub struct LoomReadDir` |  |
| `pub async fn next_entry(&mut self) -> std::io::Result<Option<LoomDirEntry>> ` |  |
| `pub struct LoomDirEntry` |  |
| `pub fn path(&self) -> std::path::PathBuf ` |  |
| `pub struct LoomMetadata` |  |
| `pub struct HardlinkCloneResult ` |  |
| `pub source_seq_no: u64,` | Die gepinnte und geklonte Quell-Sequenznummer. |
| `pub linked_files: Vec<PathBuf>,` | Liste der Zielpfade, die als Hardlinks oder Kopien erzeugt wurden. |
| `pub wal_tail_offset: Option<u64>,` | WAL-Tail-Offset, an dem der Klon beginnt, neue Mutationen anzunehmen. |
| `pub trait CheckpointHardlinkCloner: Send + Sync ` | Trait für die Schnittstelle zum physischen SSTable-Klonen via Hardlinks. |
| `pub struct DefaultHardlinkCloner` |  |
| `pub fn new() -> Self ` | Erzeugt eine neue Instanz von [`DefaultHardlinkCloner`]. |

## `src/lib.rs`

| Signatur | Beschreibung |
|---|---|
| `pub use guard::{CheckpointGuard, PinGuard}` |  |
| `pub use hardlink_cloner::` |  |
| `pub use manifest::CheckpointManifest` |  |
| `pub use meta::{CheckpointMeta, StateCheckpoint}` |  |
| `pub use orphan::` |  |
| `pub use store::{CheckpointRegistry, PersistentCheckpointStore}` |  |

## `src/manifest.rs`

| Signatur | Beschreibung |
|---|---|
| `pub struct CheckpointManifest ` |  |
| `pub meta: CheckpointMeta,` |  |
| `pub components: Vec<String>,` |  |
| `pub checksum: String,` |  |
| `pub fn new(meta: CheckpointMeta, components: Vec<String>) -> Result<Self> ` | Erzeugt ein neues [`CheckpointManifest`] und berechnet die BLAKE3-Prüfsumme. |
| `pub fn verify(&self) -> Result<()> ` | Verifiziert die Integrität des Manifests anhand der gespeicherten Prüfsumme. |

## `src/meta.rs`

| Signatur | Beschreibung |
|---|---|
| `pub struct CheckpointMeta ` |  |
| `pub name: String,` |  |
| `pub collection_id: String,` |  |
| `pub seq_no: u64,` |  |
| `pub tx_id: TxId,` |  |
| `pub metadata: serde_json::Value,` |  |
| `pub created_at: u64,` |  |
| `pub fn into_workflow_state(&self) -> WorkflowState ` |  |
| `pub fn to_workflow_state(&self) -> WorkflowState ` | Konvertiert die Metadaten in einen [`WorkflowState`]. |
| `pub struct StateCheckpoint ` |  |
| `pub tx_id: TxId,` |  |
| `pub timestamp_ms: u64,` |  |
| `pub namespace: Option<String>,` |  |

## `src/orphan.rs`

| Signatur | Beschreibung |
|---|---|
| `pub type PinId = u64` | Typ-Alias für Sequenznummern, die als gepinnte Checkpoint-Identifikatoren verwaltet werden. |
| `pub struct RecoveryReport ` |  |
| `pub recovered_pins: Vec<PinId>,` | Liste aller erfolgreich entpinnten Sequenznummern. |
| `pub recovered_checkpoints: Vec<TxId>,` | Liste aller bereinigten Transaktions-IDs. |
| `pub fn is_empty(&self) -> bool ` | Liefert `true`, wenn keine verwaisten Einträge vorhanden waren oder bereinigt wurden. |
| `pub fn total_recovered(&self) -> usize ` | Liefert die Gesamtzahl aller bereinigten Pins und Checkpoints. |
| `pub fn global_orphan_registry() -> &'static OrphanRegistry ` |  |
| `pub fn monotonic_timestamp_ms_with(clock: &dyn contextra_ports::Clock) -> u64 ` | Liest einen Millisekunden-Zeitstempel deterministisch über die übergebene Clock-Port-Instanz. |
| `pub fn monotonic_timestamp_ms() -> u64 ` | Liest den Millisekunden-Zeitstempel über die standardmäßige SystemClock. |
| `pub struct OrphanRegistry ` |  |
| `pub fn new(file_path: impl Into<std::path::PathBuf>) -> Self ` | Erzeugt eine neue Instanz von [`OrphanRegistry`]. |
| `pub fn register_orphan(&self, pin_id: PinId) -> std::io::Result<()> ` | Registriert synchron einen verwaisten Pin. |
| `pub fn get_orphans(&self) -> Vec<PinId> ` | Liefert alle aktuell registrierten verwaisten Pin-Sequenznummern. |
| `pub fn clear_all(&self) ` | Löscht alle Einträge im Speicher und auf der Platte synchron. |
| `pub async fn recover_and_clean<S: contextra_ports::StorageEngine>(` | Bereinigt alle verwaisten Pins durch Entpinnen in der Storage-Engine. |
| `pub struct PinnedSeqNoOrphan ` |  |
| `pub seq_no: u64,` |  |
| `pub timestamp_ms: u64,` |  |
| `pub fn register_pinned_seq_no_orphan(orphan: PinnedSeqNoOrphan) ` |  |
| `pub struct OrphanState ` |  |
| `pub checkpoints: Vec<StateCheckpoint>,` |  |
| `pub pinned_seq_nos: Vec<PinnedSeqNoOrphan>,` |  |
| `pub persist_path: std::path::PathBuf,` |  |
| `pub fn persist_sync(&self) -> std::io::Result<()> ` | Schreibt den Zustand synchron als formatierte JSON-Datei auf die Festplatte (ggf. über Vfs). |
| `pub fn persist_vfs(&self, vfs: &dyn contextra_ports::Vfs) -> std::io::Result<()> ` | Schreibt den Zustand über die Vfs-Schnittstelle crash-konsistent (`write_atomic`). |
| `pub fn load_sync(path: &std::path::Path) -> Self ` | Lädt den Zustand synchron von der Festplatte oder liefert einen leeren Zustand bei Fehlern. |
| `pub fn load_vfs(vfs: &dyn contextra_ports::Vfs, path: &std::path::Path) -> Self ` | Lädt den Zustand über die Vfs-Schnittstelle. |
| `pub struct InstanceOrphanRegistry ` | Instanz-gebundene Orphan-Registry für Checkpoints und gepinnte Sequenznummern (ADR-053). |
| `pub fn new(persist_path: impl Into<std::path::PathBuf>) -> Self ` | Erzeugt eine neue instanzgebundene Registry für den angegebenen Dateipfad. |
| `pub fn new_with_vfs(` | Erzeugt eine neue instanzgebundene Registry über den VFS-Port. |
| `pub fn with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self ` | Konfiguriert die Registry mit einer spezifischen Clock-Implementierung. |
| `pub fn with_vfs(mut self, vfs: Arc<dyn contextra_ports::Vfs>) -> Self ` | Konfiguriert die Registry mit einer VFS-Implementierung. |
| `pub fn set_clock(&mut self, clock: Arc<dyn contextra_ports::Clock>) ` | Setzt die Clock-Implementierung nachträglich. |
| `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock> ` | Liefert eine Referenz auf den injizierten Clock-Port. |
| `pub fn vfs(&self) -> Option<&Arc<dyn contextra_ports::Vfs>> ` | Liefert eine Referenz auf das injizierte VFS (falls vorhanden). |
| `pub fn load_sync(path: &std::path::Path) -> Self ` | Lädt den Zustand synchron von der Festplatte. |
| `pub fn persist_sync(&self) -> std::io::Result<()> ` | Persistiert den aktuellen Zustand synchron auf die Festplatte (bevorzugt über Vfs). |
| `pub fn register_orphan_sync(&self, orphan: PinnedSeqNoOrphan) ` | Registriert einen verwaisten Sequenz-Pin rein im Arbeitsspeicher ohne blockierendes Disk-I/O. |
| `pub fn register_checkpoint_sync(&self, cp: StateCheckpoint) ` | Registriert einen verwaisten Checkpoint rein im Arbeitsspeicher ohne blockierendes Disk-I/O. |
| `pub async fn flush_orphan_registry(&self) -> std::io::Result<()> ` | Flusht die In-Memory-Einträge asynchron auf Festplatte, sofern Änderungen vorliegen (`is_dirty`). |
| `pub fn drain_orphan_pins(&self) -> Vec<PinnedSeqNoOrphan> ` | Entnimmt alle verwaisten Pins aus dem Speicher und aktualisiert die Statusdatei. |
| `pub fn get_orphan_pins(&self) -> Vec<PinnedSeqNoOrphan> ` | Liefert eine Momentaufnahme aller verwaisten Pins im Speicher. |
| `pub fn clear_orphan_pin(&self, seq_no: u64) ` | Entfernt eine einzelne gepinnte Sequenznummer aus dem Speicher und persistiert. |
| `pub fn drain_orphaned_checkpoints(&self) -> Vec<StateCheckpoint> ` | Entnimmt alle verwaisten Checkpoints aus dem Speicher und aktualisiert die Statusdatei. |
| `pub fn get_orphaned_checkpoints(&self) -> Vec<StateCheckpoint> ` | Liefert eine Momentaufnahme aller verwaisten Checkpoints im Speicher. |
| `pub fn clear_orphaned_checkpoint(&self, tx_id: TxId) ` | Entfernt einen einzelnen verwaisten Checkpoint anhand seiner Transaktions-ID und persistiert. |
| `pub fn clear_all(&self) ` | Löscht alle Einträge (Pins und Checkpoints) und leert die Statusdatei. |
| `pub fn register_orphaned_checkpoint(cp: StateCheckpoint) ` |  |
| `pub fn get_orphaned_checkpoints() -> Vec<StateCheckpoint> ` |  |
| `pub fn get_orphaned_checkpoints_for_namespace(ns: &str) -> Vec<StateCheckpoint> ` |  |
| `pub fn clear_orphaned_checkpoint(_tx_id: TxId) {}` |  |
| `pub fn clear_all_orphaned_checkpoints() {}` |  |
| `pub async fn await_pending_rollbacks() {}` | Rückwärtskompatibilität: Wartet auf ausstehende Rollbacks (No-op, da Hintergrund-Tasks entfernt wurden). |
| `pub fn pending_rollback_count() -> usize ` | Rückwärtskompatibilität: Liefert die Anzahl ausstehender Rollback-Hintergrundaufgaben (immer 0). |
| `pub fn orphaned_checkpoint_count() -> usize ` |  |

## `src/store.rs`

| Signatur | Beschreibung |
|---|---|
| `pub trait CheckpointRegistry: contextra_ports::Checkpoint + Send + Sync ` | Trait für die erweiterte Checkpoint-Verwaltung inklusive Orphan-Recovery. |
| `pub high_water_mark: u64,` |  |
| `pub struct PersistentCheckpointStore<S: contextra_ports::StorageEngine> ` | Registry für gespeicherte Checkpoints mit Thread-sicherem Zustand. |
| `pub async fn open_and_recover(` | Öffnet einen PersistentCheckpointStore und führt eine vollständige Orphan-Recovery für Pins und Checkpoints durch. |
| `pub async fn open_and_recover_with_orphan_registry(` | Öffnet einen PersistentCheckpointStore mit einer spezifischen instanzgebundenen Orphan Registry und führt Orphan-Recovery durch. |
| `pub async fn open(storage: Arc<S>, namespace: impl Into<String>) -> Result<Self> ` | Öffnet einen PersistentCheckpointStore asynchron mit Rekonstruktion und Monotonie-Garantie. |
| `pub async fn open_with_orphan_registry(` | Öffnet einen PersistentCheckpointStore mit einer spezifischen instanzgebundenen Orphan Registry. |
| `pub fn new(storage: Arc<S>, namespace: impl Into<String>) -> Result<Self> ` | Synchroner Konstruktor für Umgebungen ohne asynchronen Tokio-Einstiegspunkt. |
| `pub fn new_with_orphan_registry(` | Synchroner Konstruktor mit injizierter Orphan Registry. |
| `pub async fn allocate_tx(&self) -> Result<TxId> ` | Reserviert eine neue Transaktions-ID aus dem reservierten internen Bereich (`INTERNAL_BASE`). |
| `pub fn skipped_rollback_count(&self) -> u64 ` | Liefert die Anzahl übersprungener Rollbacks. |
| `pub fn checkpoint_guard_skipped_rollback_count(&self) -> u64 ` | Alias für `skipped_rollback_count`. |
| `pub fn checkpoint_counter(&self) -> u64 ` | Liefert den Zählerstand für Checkpoint-Zeitstempel. |
| `pub fn with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self ` | Setzt den Zeitgeber für den `PersistentCheckpointStore`. |
| `pub fn clock(&self) -> &Arc<dyn contextra_ports::Clock> ` | Liefert den aktuellen Clock-Port. |
| `pub fn set_metrics_sink(&mut self, sink: Arc<dyn contextra_ports::MetricsSink>) ` | Setzt die Metrik-Senke. |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self ` | Konfiguriert den Store mit einer Metrik-Senke. |
| `pub fn monotonic_timestamp_ms(&self) -> u64 ` | Liefert einen monoton steigenden Zeitstempel in Millisekunden. |
| `pub fn create_guard(&self, tx_id: TxId) -> Result<CheckpointGuard<S>> ` | Creates an ephemeral transactional checkpoint RAII guard. |
| `pub async fn create_checkpoint(` | Creates a new persistent checkpoint. |
| `pub async fn drop_checkpoint(&self, name: &str) -> Result<()> ` | Deletes a persistent checkpoint by name. |
| `pub async fn list_checkpoints(&self) -> Result<Vec<CheckpointMeta>> ` | Listet alle persistent gespeicherten Checkpoints im konfigurierten Namespace auf. |
| `pub async fn get_checkpoint(&self, name: &str) -> Result<Option<CheckpointMeta>> ` | Ruft einen Checkpoint nach Namen ab. |
| `pub async fn restore_checkpoint(&self, name: &str) -> Result<CheckpointMeta> ` | Restores the system to a specific checkpoint by name. |
| `pub fn orphan_registry(&self) -> &Arc<InstanceOrphanRegistry> ` | Liefert eine Referenz auf die Orphan-Registry. |
| `pub async fn flush_orphan_registry(&self) -> Result<()> ` | Flusht die Orphan-Registry asynchron auf Platte. |
| `pub async fn shutdown(&self) -> Result<()> ` | Führt einen expliziten Flush der Orphan-Registry beim Herunterfahren durch. |
| `pub async fn close(&self) -> Result<()> ` | Alias für `shutdown()`. Führt einen kontrollierten Flush beim Schließen aus. |
| `pub fn register_pinned_seq_no_orphan(&self, orphan: PinnedSeqNoOrphan) ` | Registriert einen verwaisten Sequenz-Pin. |
| `pub fn register_orphaned_checkpoint(&self, cp: StateCheckpoint) ` | Registriert einen verwaisten Checkpoint. |
| `pub fn get_orphaned_checkpoints(&self) -> Vec<StateCheckpoint> ` | Liefert alle verwaisten Checkpoints. |
| `pub fn clear_orphaned_checkpoint(&self, tx_id: TxId) ` | Löscht einen verwaisten Checkpoint anhand seiner Transaktions-ID. |
| `pub fn clear_all_orphaned_checkpoints(&self) ` | Löscht alle verwaisten Checkpoints. |
| `pub async fn recover_orphaned_pins(&self) -> Result<Vec<PinId>> ` | Recovers all registered/persisted orphaned sequence pins (ADR-052). |
| `pub async fn recover_orphaned_checkpoints(&self) -> Result<Vec<TxId>> ` | Recovers all registered/persisted orphaned checkpoints during controlled startup or recovery. |
| `pub async fn create_hardlink_clone(` | Creates a physical hardlink clone of all SSTable files visible for `seq_no` into `target_dir`. |
