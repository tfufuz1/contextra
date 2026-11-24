# Schnittstellenspezifikation: `contextra-checkpoint`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-checkpoint` |
| Version | workspace |
| Pfad | `crates/contextra-checkpoint` |
| Rolle | Checkpoints |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # Checkpoint-Registry für Time-Travel und MVCC-basiertes Snapshotting
>
> Dieses Modul bildet die Wurzel der Crate `contextra-checkpoint`.
>
> ## Zweck & Rolle im System
> Die Crate `contextra-checkpoint` gehört zu **Ring 1 (Persistenz)** der Contextra-Architektur.
> Sie koordiniert konsistente Snapshots über heterogene Storage-Komponenten hinweg (LSM-Tree,
> Vektor-Index, Graph-Speicher) und stellt deterministische Rollbacks für Agenten-Workflows bereit.
>
> ## Öffentliche Checkpoint-Subsystem Architecture (gemäß ADR-011)
> `contextra-checkpoint` ist der **einzige öffentlich sichtbare Einstiegspunkt** für das Checkpoint-Konzept.
> Es stellt den Trait `CheckpointCoordinator` (unter `compat`), die Registry [`PersistentCheckpointStore`]

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `blake3` | normal | workspace | nein |  |
| `bytes` | normal | workspace | nein |  |
| `contextra-env` | normal | path | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-infer-candle, contextra-kvcache, contextra-mvcc, contextra-ports, contextra-store, contextra-text, contextra-vector |
| `contextra-ids` | normal | path | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-mvcc, contextra-store, contextra-text, contextra-types, contextra-vector |
| `contextra-ports` | dev | path | nein | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-testkit` | dev | path | nein | contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-store |
| `contextra-types` | dev | path | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `criterion` | dev | registry | nein |  |
| `parking_lot` | normal | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | dev | workspace | nein |  |
| `tempfile` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | normal | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-agent`, `contextra-db`, `contextra-engine`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | – |
| `docid-128` | `contextra-ids/docid-128` |
| `ports-compat` | `workspace` |
| `types-compat` | `workspace` |
| `workspace` | `dep:contextra-types`, `dep:contextra-ports` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 20 |
| Quellzeilen | 5593 |
| Öffentliche Items (Modul-Ebene) | 55 |
| davon `fn` | 18 |
| davon `struct` | 19 |
| davon `enum` | 1 |
| davon `trait` | 6 |
| davon `type` | 5 |
| davon `const` | 1 |
| davon `mod` | 5 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 15 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 83 |
| `.expect(` (Produktionscode) | 39 |
| `panic!` (Produktionscode) | 4 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 11 |
| `#[deprecated]` | 21 |
| `#[cfg(feature…)]`-Gates | 3 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 153 / 153 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `compat::error` | `crates/contextra-checkpoint/src/compat/error.rs` | 70 |
| `compat` | `crates/contextra-checkpoint/src/compat/mod.rs` | 7 |
| `compat::ports_impl` | `crates/contextra-checkpoint/src/compat/ports_impl.rs` | 62 |
| `compat::snapshot_bridge` | `crates/contextra-checkpoint/src/compat/snapshot_bridge.rs` | 108 |
| `contract` | `crates/contextra-checkpoint/src/contract.rs` | 162 |
| `error` | `crates/contextra-checkpoint/src/error.rs` | 82 |
| `fs_seam` | `crates/contextra-checkpoint/src/fs_seam.rs` | 69 |
| `guard::tests` | `crates/contextra-checkpoint/src/guard/tests.rs` | 300 |
| `guard` | `crates/contextra-checkpoint/src/guard.rs` | 855 |
| `hardlink_cloner` | `crates/contextra-checkpoint/src/hardlink_cloner.rs` | 294 |
| `crate` | `crates/contextra-checkpoint/src/lib.rs` | 72 |
| `manifest` | `crates/contextra-checkpoint/src/manifest.rs` | 259 |
| `meta::tests` | `crates/contextra-checkpoint/src/meta/tests.rs` | 97 |
| `meta` | `crates/contextra-checkpoint/src/meta.rs` | 270 |
| `orphan::tests` | `crates/contextra-checkpoint/src/orphan/tests.rs` | 134 |
| `orphan` | `crates/contextra-checkpoint/src/orphan.rs` | 836 |
| `snapshot_source` | `crates/contextra-checkpoint/src/snapshot_source.rs` | 55 |
| `store::tests` | `crates/contextra-checkpoint/src/store/tests.rs` | 886 |
| `store` | `crates/contextra-checkpoint/src/store.rs` | 960 |
| `vfs` | `crates/contextra-checkpoint/src/vfs.rs` | 15 |

## 6. Re-Exports (`pub use`)

- `pub use crate::error::` — `crates/contextra-checkpoint/src/contract.rs:12` (Modul `contract`)
- `pub use crate::fs_seam::` — `crates/contextra-checkpoint/src/contract.rs:13` (Modul `contract`)
- `pub use crate::meta::WorkflowState` — `crates/contextra-checkpoint/src/contract.rs:14` (Modul `contract`)
- `pub use crate::snapshot_source::` — `crates/contextra-checkpoint/src/contract.rs:15` (Modul `contract`)
- `pub use crate::vfs::CheckpointVfs` — `crates/contextra-checkpoint/src/contract.rs:16` (Modul `contract`)
- `pub use vfs::CheckpointVfs` — `crates/contextra-checkpoint/src/lib.rs:52` (Modul `crate`)
- `pub use error::` — `crates/contextra-checkpoint/src/lib.rs:55` (Modul `crate`)
- `pub use fs_seam::` — `crates/contextra-checkpoint/src/lib.rs:56` (Modul `crate`)
- `pub use guard::` — `crates/contextra-checkpoint/src/lib.rs:57` (Modul `crate`)
- `pub use hardlink_cloner::` — `crates/contextra-checkpoint/src/lib.rs:58` (Modul `crate`)
- `pub use manifest::CheckpointManifest` — `crates/contextra-checkpoint/src/lib.rs:61` (Modul `crate`)
- `pub use meta::` — `crates/contextra-checkpoint/src/lib.rs:62` (Modul `crate`)
- `pub use orphan::` — `crates/contextra-checkpoint/src/lib.rs:64` (Modul `crate`)
- `pub use snapshot_source::` — `crates/contextra-checkpoint/src/lib.rs:71` (Modul `crate`)
- `pub use store::` — `crates/contextra-checkpoint/src/lib.rs:72` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `compat`

#### `mod error`
- Ort: `crates/contextra-checkpoint/src/compat/mod.rs:5`
```rust
pub mod error
```

#### `mod ports_impl`
- Ort: `crates/contextra-checkpoint/src/compat/mod.rs:6`
```rust
pub mod ports_impl
```

#### `mod snapshot_bridge`
- Ort: `crates/contextra-checkpoint/src/compat/mod.rs:7`
```rust
pub mod snapshot_bridge
```

### Modul `contract`

#### `struct InMemorySnapshotSource`
- Ort: `crates/contextra-checkpoint/src/contract.rs:28`
```rust
pub struct InMemorySnapshotSource
```
Dokumentation: Vollständige, I/O-freie Referenzimplementierung einer `SnapshotSource`. Geeignet für Standalone-Einsätze, Tests und In-Memory-Verwendung ohne Speicher-Backend.
- Impl-Block `crates/contextra-checkpoint/src/contract.rs:35`
- Impl-Block (Trait `SnapshotSource`) `crates/contextra-checkpoint/src/contract.rs:52`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-checkpoint/src/contract.rs:37` | Erzeugt eine neue, leere In-Memory-Quelle. |
| `pub fn get_sync(&self, key: &[u8]) -> Option<Vec<u8>>` | `crates/contextra-checkpoint/src/contract.rs:42` | Synchroner Datenzugriff für Tests oder Debugging. |

#### `type VersionedKeyMap`
- Ort: `crates/contextra-checkpoint/src/contract.rs:22`
```rust
pub type VersionedKeyMap = BTreeMap<Vec<u8>, BTreeMap<u64, Option<Vec<u8>>>>
```
Dokumentation: Vollständige, I/O-freie Referenzimplementierung einer `SnapshotSource`. Geeignet für Standalone-Einsätze, Tests und In-Memory-Verwendung ohne Speicher-Backend. Typalias für versionierte Byte-Einträge in der In-Memory-Quelle.

### Modul `crate`

#### `mod compat`
- Ort: `crates/contextra-checkpoint/src/lib.rs:39`
```rust
pub mod compat
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod contract`
- Ort: `crates/contextra-checkpoint/src/lib.rs:40`
```rust
pub mod contract
```

### Modul `error`

#### `enum CheckpointError`
- Ort: `crates/contextra-checkpoint/src/error.rs:17`
```rust
pub enum CheckpointError
```
Dokumentation: Fehlerzustände im Checkpoint-Subsystem.
- Impl-Block `crates/contextra-checkpoint/src/error.rs:61`

Varianten:

- `InvalidInput(String)` — `crates/contextra-checkpoint/src/error.rs:20`
- `Serialization(String)` — `crates/contextra-checkpoint/src/error.rs:24`
- `Transaction(String)` — `crates/contextra-checkpoint/src/error.rs:28`
- `CheckpointNotFound` — `crates/contextra-checkpoint/src/error.rs:32`
- `CrossDeviceLink{` — `crates/contextra-checkpoint/src/error.rs:36`
- `Io(#[from] std::io::Error)` — `crates/contextra-checkpoint/src/error.rs:45`
- `IdWidthMismatch{` — `crates/contextra-checkpoint/src/error.rs:49`
- `Internal(String)` — `crates/contextra-checkpoint/src/error.rs:58`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn invalid_input(msg: impl Into<String>) -> Self` | `crates/contextra-checkpoint/src/error.rs:63` | Erzeugt einen `InvalidInput`-Fehler. |
| `pub fn internal(msg: impl Into<String>) -> Self` | `crates/contextra-checkpoint/src/error.rs:68` | Erzeugt einen `Internal`-Fehler. |
| `pub fn cross_device_link( source_path: impl Into<String>, target_path: impl Into<String>, ) -> Self` | `crates/contextra-checkpoint/src/error.rs:73` | Erzeugt einen `CrossDeviceLink`-Fehler. |

#### `type CheckpointResult`
- Ort: `crates/contextra-checkpoint/src/error.rs:9`
```rust
pub type CheckpointResult<T> = std::result::Result<T, CheckpointError>
```
Dokumentation: Typalias für Resultate mit [`CheckpointError`].

#### `type Result`
- Ort: `crates/contextra-checkpoint/src/error.rs:12`
```rust
pub type Result<T> = CheckpointResult<T>
```
Dokumentation: Kurzform für interne Module, identisch zu [`CheckpointResult`].

### Modul `fs_seam`

#### `struct StdFsSeam`
- Ort: `crates/contextra-checkpoint/src/fs_seam.rs:30`
```rust
pub struct StdFsSeam
```
Dokumentation: Standard-Implementierung basierend auf `std::fs`.
- Impl-Block `crates/contextra-checkpoint/src/fs_seam.rs:32`
- Impl-Block (Trait `FsSeam`) `crates/contextra-checkpoint/src/fs_seam.rs:39`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-checkpoint/src/fs_seam.rs:34` | Erzeugt eine neue Instanz von [`StdFsSeam`]. |

#### `trait FsSeam`
- Ort: `crates/contextra-checkpoint/src/fs_seam.rs:11`
```rust
pub trait FsSeam: Send + Sync
```
Dokumentation: Minimale Schnittstelle für Dateisystemzugriffe.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn write_file(&self, path: &Path, content: &[u8]) -> io::Result<()>` | `crates/contextra-checkpoint/src/fs_seam.rs:13` | Schreibt Daten atomar oder direkt in eine Datei. |
| `fn read_file(&self, path: &Path) -> io::Result<Vec<u8>>` | `crates/contextra-checkpoint/src/fs_seam.rs:16` | Liest den gesamten Inhalt einer Datei. |
| `fn exists(&self, path: &Path) -> bool` | `crates/contextra-checkpoint/src/fs_seam.rs:19` | Prüft, ob ein Pfad existiert. |
| `fn remove_file(&self, path: &Path) -> io::Result<()>` | `crates/contextra-checkpoint/src/fs_seam.rs:22` | Löscht eine Datei, falls sie existiert. |
| `fn create_dir_all(&self, path: &Path) -> io::Result<()>` | `crates/contextra-checkpoint/src/fs_seam.rs:25` | Erzeugt ein Verzeichnis rekursiv. |

### Modul `guard`

#### `struct CheckpointGuard`
- Ort: `crates/contextra-checkpoint/src/guard.rs:154`
```rust
pub struct CheckpointGuard<S: SnapshotSource>
```
Dokumentation: RAII Guard, der explizit über [`commit`](Self::commit) oder [`rollback`](Self::rollback) finalisiert werden MUSS. # RAII-Kontrakt und Betriebshinweise Ein `CheckpointGuard` muss explizit über `commit()` oder `rollback().await` finalisiert werden. Wird der Guard ohne expliziten Commit/Rollback gedroppt (z. B. bei Panik oder Programmierfehler), wird KEIN asynchroner Hintergrund-Rollback gespawnt, um Kollisionen mit späteren Transaktionen zu verhindern. Stattdessen wird der Checkpoint als "orphaned" registriert/persistiert und beim nächsten kontrollierten Recovery-Zyklus verarbeitet.
- Impl-Block `crates/contextra-checkpoint/src/guard.rs:165`
- Impl-Block (Trait `Drop`) `crates/contextra-checkpoint/src/guard.rs:466`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `checkpoint` | `Option<StateCheckpoint>` | `crates/contextra-checkpoint/src/guard.rs:155` |
| `storage` | `Arc<S>` | `crates/contextra-checkpoint/src/guard.rs:156` |
| `namespace` | `String` | `crates/contextra-checkpoint/src/guard.rs:157` |
| `orphan_registry` | `Arc<InstanceOrphanRegistry>` | `crates/contextra-checkpoint/src/guard.rs:158` |
| `skipped_rollbacks` | `Arc<AtomicU64>` | `crates/contextra-checkpoint/src/guard.rs:159` |
| `clock` | `Arc<dyn Clock>` | `crates/contextra-checkpoint/src/guard.rs:160` |
| `metrics_sink` | `Option<Arc<dyn MetricsSink>>` | `crates/contextra-checkpoint/src/guard.rs:161` |
| `created_at_nanos` | `u64` | `crates/contextra-checkpoint/src/guard.rs:162` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>) -> Self` | `crates/contextra-checkpoint/src/guard.rs:167` | Erzeugt einen neuen `CheckpointGuard` mit Standard-SystemClock. |
| `pub fn with_clock( checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>, clock: Arc<dyn Clock>, ) -> Self` | `crates/contextra-checkpoint/src/guard.rs:173` | Erzeugt einen neuen `CheckpointGuard` mit injiziertem Clock-Port. |
| `pub fn with_registry( checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Self` | `crates/contextra-checkpoint/src/guard.rs:194` | Erzeugt einen neuen `CheckpointGuard` mit vorhandener Orphan-Registry. |
| `pub fn with_registry_and_counter( checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, skipped_rollbacks: Arc<AtomicU64>, ) -> Self` | `crates/contextra-checkpoint/src/guard.rs:213` | Erzeugt einen neuen `CheckpointGuard` mit vorhandener Orphan-Registry und Zähler für übersprungene Rollbacks. |
| `pub fn with_registry_counter_and_clock( checkpoint: StateCheckpoint, storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, skipped_rollbacks: Arc<AtomicU64>, clock: Arc<dyn Clock>, ) -> Self` | `crates/contextra-checkpoint/src/guard.rs:232` | Vollständiger interner Konstruktor mit allen Parametern. |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn MetricsSink>) -> Self` | `crates/contextra-checkpoint/src/guard.rs:254` | Ergänzt den Guard um eine Metrik-Senke zur Erfassung von Ausführungsdauern. |
| `pub fn with_metrics_sink_opt(mut self, sink: Option<Arc<dyn MetricsSink>>) -> Self` | `crates/contextra-checkpoint/src/guard.rs:260` | Ergänzt den Guard optional um eine Metrik-Senke. |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | `crates/contextra-checkpoint/src/guard.rs:266` | Liefert den verwendeten Clock-Port. |
| `pub async fn for_agent_step(storage: Arc<S>, tx: TxId) -> Result<Self>` | `crates/contextra-checkpoint/src/guard.rs:272` | Erstellt einen neuen CheckpointGuard für einen Agenten-Schritt mit SystemClock. |
| `pub async fn for_agent_step_with_registry( storage: Arc<S>, tx: TxId, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Result<Self>` | `crates/contextra-checkpoint/src/guard.rs:278` | Erstellt einen CheckpointGuard für einen Agenten-Schritt mit spezifischer Orphan-Registry. |
| `pub async fn for_agent_step_with_clock( storage: Arc<S>, tx: TxId, clock: Arc<dyn Clock>, ) -> Result<Self>` | `crates/contextra-checkpoint/src/guard.rs:298` | Erstellt einen CheckpointGuard für einen Agenten-Schritt mit speziellem Clock-Port. |
| `pub fn checkpoint(&self) -> Result<&StateCheckpoint>` | `crates/contextra-checkpoint/src/guard.rs:316` | Liefert eine Referenz auf den enthaltenen Checkpoint, sofern noch nicht konsumiert. |
| `pub fn commit(mut self) -> Result<StateCheckpoint>` | `crates/contextra-checkpoint/src/guard.rs:330` | Überführt den Guard in einen dauerhaften Zustand (Commit) und konsumiert ihn. # Ablauf von `commit`: 1. Checkpoint aus Option entnehmen (bei vorherigem Konsum - |
| `pub async fn rollback(mut self) -> Result<()>` | `crates/contextra-checkpoint/src/guard.rs:373` | Führt ein manuelles, asynchrones Rollback des Checkpoints aus. Nach Aufruf von `rollback()` ist der Guard konsumiert, sodass beim Drop kein erneutes Rollback au |
| `pub fn rollback_blocking(mut self) -> Result<()>` | `crates/contextra-checkpoint/src/guard.rs:416` | Führt ein synchrones, blockierendes Rollback des Checkpoints aus. # Einschränkung Diese Methode darf **nicht** aus einem laufenden asynchronen Tokio-Kontext her |

#### `struct PinGuard`
- Ort: `crates/contextra-checkpoint/src/guard.rs:43`
```rust
pub struct PinGuard<S: SnapshotSource>
```
Dokumentation: RAII-Guard für gepinnte Checkpoint-Sequenznummern (gemäß ADR-015). # Zweck Stellt sicher, dass eine gepinnte Sequenznummer bei einem Fehler oder Panic während des Schreibens automatisch entpinnt oder als verwaist registriert wird, um dauerhafte GC-Blockaden (LSM-Compaction) zu verhindern. # Felder - `storage`: Referenz auf die Speicherengine zum Entpinnen. - `seq_no`: Die gepinnte Sequenznummer (wird bei `defuse()` oder `unpin()` geleert). - `orphan_registry`: Instanzgebundene Registry für verwaiste Einträge bei unkontrolliertem Drop. - `clock`: Injizierter Uhrzeit-Port für deterministische Ze
- Impl-Block `crates/contextra-checkpoint/src/guard.rs:50`
- Impl-Block (Trait `Drop`) `crates/contextra-checkpoint/src/guard.rs:121`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub async fn pin( storage: Arc<S>, seq_no: u64, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Result<Self>` | `crates/contextra-checkpoint/src/guard.rs:58` | Pinnt eine Sequenznummer in der Storage-Engine unter Verwendung der Clock aus der Orphan-Registry. # Parameter - `storage`: Die zugrundeliegende Speicherengine. |
| `pub async fn pin_with_clock( storage: Arc<S>, seq_no: u64, orphan_registry: Arc<InstanceOrphanRegistry>, clock: Arc<dyn Clock>, ) -> Result<Self>` | `crates/contextra-checkpoint/src/guard.rs:76` | Pinnt eine Sequenznummer mit explizit übergebenem Clock-Port. # Ablauf von `pin_with_clock`: 1. `storage.pin_checkpoint(seq_no)` asynchron ausführen. 2. Bei Erf |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | `crates/contextra-checkpoint/src/guard.rs:93` | Liefert eine Referenz auf den injizierten Clock-Port. |
| `pub fn defuse(mut self)` | `crates/contextra-checkpoint/src/guard.rs:98` | Entschärft den Guard nach erfolgreicher Persistierung, sodass die Sequenznummer gepinnt bleibt. |
| `pub async fn unpin(mut self) -> Result<()>` | `crates/contextra-checkpoint/src/guard.rs:110` | Entpinnt die Sequenznummer explizit und asynchron bei einem abgefangenen Fehler. # Ablauf von `unpin`: 1. `seq_no` entnehmen. 2. `storage.unpin_checkpoint(seq_n |

### Modul `hardlink_cloner`

#### `struct DefaultHardlinkCloner`
- Ort: `crates/contextra-checkpoint/src/hardlink_cloner.rs:178`
```rust
pub struct DefaultHardlinkCloner
```
Dokumentation: Produktive Implementierung von [`CheckpointHardlinkCloner`] basierend auf `tokio::fs`. Nutzt asynchrone Dateisystem-Operationen ohne den Haupt-Thread zu blockieren.
- Impl-Block `crates/contextra-checkpoint/src/hardlink_cloner.rs:180`
- Impl-Block (Trait `CheckpointHardlinkCloner`) `crates/contextra-checkpoint/src/hardlink_cloner.rs:187`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-checkpoint/src/hardlink_cloner.rs:182` | Erzeugt eine neue Instanz von [`DefaultHardlinkCloner`]. |

#### `struct HardlinkCloneResult`
- Ort: `crates/contextra-checkpoint/src/hardlink_cloner.rs:128`
```rust
pub struct HardlinkCloneResult
```
Dokumentation: Ergebnisstruktur einer durchgeführten Hardlink-Klon-Operation für SSTables. # Zweck Liefert dem Aufrufer die Sequenznummer sowie die Liste aller erfolgreich verknüpften oder kopierten Zieldateien. # Felder - `source_seq_no`: Die gepinnte und geklonte Quell-Sequenznummer. - `linked_files`: Liste der im Zielverzeichnis erzeugten Pfade. - `wal_tail_offset`: Offset im Write-Ahead-Log, ab dem der geklonte Zustand neue Mutationen akzeptiert.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `source_seq_no` | `u64` | `crates/contextra-checkpoint/src/hardlink_cloner.rs:130` |
| `linked_files` | `Vec<PathBuf>` | `crates/contextra-checkpoint/src/hardlink_cloner.rs:132` |
| `wal_tail_offset` | `Option<u64>` | `crates/contextra-checkpoint/src/hardlink_cloner.rs:139` |

#### `trait CheckpointHardlinkCloner`
- Ort: `crates/contextra-checkpoint/src/hardlink_cloner.rs:151`
```rust
pub trait CheckpointHardlinkCloner: Send + Sync
```
Dokumentation: Trait für die Schnittstelle zum physischen SSTable-Klonen via Hardlinks. # Zweck Abstrahiert das Dateisystem-Klonen für Tests und verschiedene Plattformen. # Windows NTFS Limit Unter Windows ist `CreateHardLink` NTFS-exklusiv und auf maximal **1023 Links** pro Datei limitiert. Bei Überschreitung oder dateisystemübergreifenden Links wird [`CheckpointError::CrossDeviceLink`] gemeldet.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn clone_sstable_hardlinks<'a>( &'a self, seq_no: u64, source_dir: &'a Path, target_dir: &'a Path, snapshot_pinner: &'a (dyn SnapshotPinner + 'a), ) -> BoxFuture<'a, Result<HardlinkCloneResult>>` | `crates/contextra-checkpoint/src/hardlink_cloner.rs:164` | Klont alle für `seq_no` sichtbaren SSTable-Dateien im Quellverzeichnis ins Zielverzeichnis. Hält während der Operation einen aktiven `SnapshotRegistry`-Pin auf  |

#### `trait SnapshotPinner`
- Ort: `crates/contextra-checkpoint/src/hardlink_cloner.rs:30`
```rust
pub trait SnapshotPinner: Send + Sync
```
Dokumentation: Trait zur Abstraktion von Snapshot-Pinning-Mechanismen während des SSTable-Klonens.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn pin_seq_no(&self, seq_no: u64) -> Box<dyn std::any::Any + Send + Sync>` | `crates/contextra-checkpoint/src/hardlink_cloner.rs:32` | Pinnt eine Sequenznummer und liefert einen RAII-Guard zurück. |

### Modul `hardlink_cloner::fs`

#### `struct LoomDirEntry`
- Ort: `crates/contextra-checkpoint/src/hardlink_cloner.rs:106`
```rust
pub struct LoomDirEntry
```
- Impl-Block `crates/contextra-checkpoint/src/hardlink_cloner.rs:107`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn path(&self) -> std::path::PathBuf` | `crates/contextra-checkpoint/src/hardlink_cloner.rs:108` |  |

#### `struct LoomMetadata`
- Ort: `crates/contextra-checkpoint/src/hardlink_cloner.rs:113`
```rust
pub struct LoomMetadata
```

#### `struct LoomReadDir`
- Ort: `crates/contextra-checkpoint/src/hardlink_cloner.rs:99`
```rust
pub struct LoomReadDir
```
- Impl-Block `crates/contextra-checkpoint/src/hardlink_cloner.rs:100`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub async fn next_entry(&mut self) -> std::io::Result<Option<LoomDirEntry>>` | `crates/contextra-checkpoint/src/hardlink_cloner.rs:101` |  |

#### `fn copy`
- Ort: `crates/contextra-checkpoint/src/hardlink_cloner.rs:95`
```rust
pub async fn copy<P: AsRef<Path>, Q: AsRef<Path>>(_from: P, _to: Q) -> std::io::Result<u64>
```

#### `fn create_dir_all`
- Ort: `crates/contextra-checkpoint/src/hardlink_cloner.rs:69`
```rust
pub async fn create_dir_all<P: AsRef<Path>>(_path: P) -> std::io::Result<()>
```

#### `fn hard_link`
- Ort: `crates/contextra-checkpoint/src/hardlink_cloner.rs:88`
```rust
pub async fn hard_link<P: AsRef<Path>, Q: AsRef<Path>>( _from: P, _to: Q, ) -> std::io::Result<()>
```

#### `fn metadata`
- Ort: `crates/contextra-checkpoint/src/hardlink_cloner.rs:77`
```rust
pub async fn metadata<P: AsRef<Path>>(_path: P) -> std::io::Result<LoomMetadata>
```

#### `fn read_dir`
- Ort: `crates/contextra-checkpoint/src/hardlink_cloner.rs:73`
```rust
pub async fn read_dir<P: AsRef<Path>>(_path: P) -> std::io::Result<LoomReadDir>
```

#### `fn remove_file`
- Ort: `crates/contextra-checkpoint/src/hardlink_cloner.rs:84`
```rust
pub async fn remove_file<P: AsRef<Path>>(_path: P) -> std::io::Result<()>
```

### Modul `manifest`

#### `struct CheckpointManifest`
- Ort: `crates/contextra-checkpoint/src/manifest.rs:43`
```rust
pub struct CheckpointManifest
```
Dokumentation: Kryptografisch gesichertes Manifest eines gespeicherten Checkpoints. # Zweck Bindet Metadaten (`CheckpointMeta`) und zugehörige Komponenten-Dateien (z. B. SSTable-Listen) an eine BLAKE3-Prüfsumme, um Teil-Schreibfehler oder Manipulationen sofort zu erkennen. # Felder - `meta`: Die logischen Metadaten des Checkpoints. - `components`: Liste der enthaltenen Komponenten-/Dateinamen. - `checksum`: Hexadezimaler BLAKE3-Hash über die serialisierte Kombination aus `meta` und `components`.
- Impl-Block `crates/contextra-checkpoint/src/manifest.rs:52`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `meta` | `CheckpointMeta` | `crates/contextra-checkpoint/src/manifest.rs:45` |
| `components` | `Vec<String>` | `crates/contextra-checkpoint/src/manifest.rs:47` |
| `checksum` | `String` | `crates/contextra-checkpoint/src/manifest.rs:49` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(meta: CheckpointMeta, components: Vec<String>) -> Result<Self>` | `crates/contextra-checkpoint/src/manifest.rs:70` | Erzeugt ein neues [`CheckpointManifest`] und berechnet die BLAKE3-Prüfsumme. # Vorbedingungen - `meta.name` und `meta.collection_id` müssen gültige Bezeichner s |
| `pub fn verify(&self) -> Result<()>` | `crates/contextra-checkpoint/src/manifest.rs:102` | Verifiziert die Integrität des Manifests anhand der gespeicherten Prüfsumme. # Ablauf von `verify`: 1. Tupel `(&self.meta, &self.components)` neu nach JSON seri |

### Modul `meta`

#### `struct CheckpointMeta`
- Ort: `crates/contextra-checkpoint/src/meta.rs:74`
```rust
pub struct CheckpointMeta
```
Dokumentation: Metadaten für einen persistenten, benannten Checkpoint. # Zweck Speichert alle Identifikations-, Versions- und Audit-Informationen zu einem Snapshot im LSM-Tree und Vektor-Index. # Felder - `name`: Eindeutiger Name des Checkpoints (z. B. "pre_migration"). - `collection_id`: Eindeutige Kennung der Collection bzw. des Mandanten. - `seq_no`: LSM-Sequenznummer (MVCC-Schnittlinie für Lese-Isolation). - `tx_id`: Transaktions-ID, die den Zustand erzeugt hat. - `metadata`: JSON-Objekt für anwendungsspezifische Kontextdaten (z. B. Modell-Version, Prompt-ID). - `created_at`: Unix-Zeitstempel der Erstell
- Impl-Block `crates/contextra-checkpoint/src/meta.rs:99`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `name` | `String` | `crates/contextra-checkpoint/src/meta.rs:76` |
| `collection_id` | `String` | `crates/contextra-checkpoint/src/meta.rs:78` |
| `seq_no` | `u64` | `crates/contextra-checkpoint/src/meta.rs:80` |
| `tx_id` | `TxId` | `crates/contextra-checkpoint/src/meta.rs:82` |
| `metadata` | `serde_json::Value` | `crates/contextra-checkpoint/src/meta.rs:84` |
| `created_at` | `u64` | `crates/contextra-checkpoint/src/meta.rs:86` |
| `id_width_bits` | `u16` | `crates/contextra-checkpoint/src/meta.rs:89` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate_id_width(&self) -> Result<()>` | `crates/contextra-checkpoint/src/meta.rs:101` | Validiert die ID-Breite gegen die aktuelle Build-Konfiguration (SPEC §5.2). |
| `pub fn into_workflow_state(&self) -> WorkflowState` | `crates/contextra-checkpoint/src/meta.rs:117` | Konvertiert die Metadaten in einen [`WorkflowState`]. # Kompatibilität Behält `&self` für Rückwärtskompatibilität mit externen Aufrufern bei (ADR-011). |
| `pub fn to_workflow_state(&self) -> WorkflowState` | `crates/contextra-checkpoint/src/meta.rs:128` | Konvertiert die Metadaten in einen [`WorkflowState`]. # Ablauf: 1. Transaktions-ID aus `tx_id` übernehmen. 2. Deterministic Hash aus der Sequenznummer mit BLAKE |

#### `struct StateCheckpoint`
- Ort: `crates/contextra-checkpoint/src/meta.rs:149`
```rust
pub struct StateCheckpoint
```
Dokumentation: Point-in-Time Checkpoint für einen Agenten-Schritt oder eine Transaktionsgrenze. # Zweck Repräsentiert einen leichten Transaktions-Zwischenstand zur Laufzeit, z. B. während mehrstufiger Agenten-Schleifen. # Felder - `tx_id`: Transaktions-ID des Zwischenschritts. - `timestamp_ms`: Monotoner Zeitstempel in Millisekunden (über injected Clock-Port generiert). - `namespace`: Optionaler logischer Namensraum zur Isolation von Workflows.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tx_id` | `TxId` | `crates/contextra-checkpoint/src/meta.rs:151` |
| `timestamp_ms` | `u64` | `crates/contextra-checkpoint/src/meta.rs:153` |
| `namespace` | `Option<String>` | `crates/contextra-checkpoint/src/meta.rs:156` |

#### `struct WorkflowState`
- Ort: `crates/contextra-checkpoint/src/meta.rs:18`
```rust
pub struct WorkflowState
```
Dokumentation: Defines a frozen workflow state acting as a savepoint.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tx` | `TxId` | `crates/contextra-checkpoint/src/meta.rs:20` |
| `graph_hash` | `[u8; 32]` | `crates/contextra-checkpoint/src/meta.rs:22` |

#### `const CURRENT_ID_WIDTH_BITS`
- Ort: `crates/contextra-checkpoint/src/meta.rs:93`
```rust
pub const CURRENT_ID_WIDTH_BITS: u16 = if cfg!(feature = "")
```
Dokumentation: Aktuelle ID-Breite in Bits (SPEC §5.2).

### Modul `orphan`

#### `struct InstanceOrphanRegistry`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:306`
```rust
pub struct InstanceOrphanRegistry
```
Dokumentation: Instanz-gebundene Orphan-Registry für Checkpoints und gepinnte Sequenznummern (ADR-053). # Thread-Sicherheit & Nebenläufigkeit Verwendet [`parking_lot::Mutex`] für minimale Latenz beim Guard-Drop ohne OS-Thread-Blockaden. Ein atomares `is_dirty`-Flag synchronisiert anstehende Disk-Schreibvorgänge.
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-checkpoint/src/orphan.rs:315`
- Impl-Block (Trait `Default`) `crates/contextra-checkpoint/src/orphan.rs:327`
- Impl-Block `crates/contextra-checkpoint/src/orphan.rs:333`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(persist_path: impl Into<std::path::PathBuf>) -> Self` | `crates/contextra-checkpoint/src/orphan.rs:335` | Erzeugt eine neue instanzgebundene Registry für den angegebenen Dateipfad. |
| `pub fn new_with_vfs( persist_path: impl Into<std::path::PathBuf>, vfs: Arc<dyn crate::vfs::CheckpointVfs>, ) -> Self` | `crates/contextra-checkpoint/src/orphan.rs:341` | Erzeugt eine neue instanzgebundene Registry über den VFS-Port. |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-checkpoint/src/orphan.rs:350` | Konfiguriert die Registry mit einer spezifischen Clock-Implementierung. |
| `pub fn with_vfs(mut self, vfs: Arc<dyn crate::vfs::CheckpointVfs>) -> Self` | `crates/contextra-checkpoint/src/orphan.rs:356` | Konfiguriert die Registry mit einer VFS-Implementierung. |
| `pub fn set_clock(&mut self, clock: Arc<dyn Clock>)` | `crates/contextra-checkpoint/src/orphan.rs:362` | Setzt die Clock-Implementierung nachträglich. |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | `crates/contextra-checkpoint/src/orphan.rs:367` | Liefert eine Referenz auf den injizierten Clock-Port. |
| `pub fn vfs(&self) -> Option<&Arc<dyn crate::vfs::CheckpointVfs>>` | `crates/contextra-checkpoint/src/orphan.rs:372` | Liefert eine Referenz auf das injizierte VFS (falls vorhanden). |
| `pub fn load_sync(path: &std::path::Path) -> Self` | `crates/contextra-checkpoint/src/orphan.rs:377` | Lädt den Zustand synchron von der Festplatte. |
| `pub fn persist_sync(&self) -> std::io::Result<()>` | `crates/contextra-checkpoint/src/orphan.rs:417` | Persistiert den aktuellen Zustand synchron auf die Festplatte (bevorzugt über Vfs). |
| `pub fn register_orphan_sync(&self, orphan: PinnedSeqNoOrphan)` | `crates/contextra-checkpoint/src/orphan.rs:441` | Registriert einen verwaisten Sequenz-Pin rein im Arbeitsspeicher ohne blockierendes Disk-I/O. # Crash-Safety & RAII Drop Semantik Wird synchron aus [`crate::Pin |
| `pub fn register_checkpoint_sync(&self, cp: StateCheckpoint)` | `crates/contextra-checkpoint/src/orphan.rs:462` | Registriert einen verwaisten Checkpoint rein im Arbeitsspeicher ohne blockierendes Disk-I/O. # Crash-Safety & RAII Drop Semantik Wird synchron aus [`crate::Chec |
| `pub async fn flush_orphan_registry(&self) -> std::io::Result<()>` | `crates/contextra-checkpoint/src/orphan.rs:484` | Flusht die In-Memory-Einträge asynchron auf Festplatte, sofern Änderungen vorliegen (`is_dirty`). # Ablauf von `flush_orphan_registry`: 1. Prüfen, ob ein Pfad k |
| `pub fn drain_orphan_pins(&self) -> Vec<PinnedSeqNoOrphan>` | `crates/contextra-checkpoint/src/orphan.rs:512` | Entnimmt alle verwaisten Pins aus dem Speicher und aktualisiert die Statusdatei. |
| `pub fn get_orphan_pins(&self) -> Vec<PinnedSeqNoOrphan>` | `crates/contextra-checkpoint/src/orphan.rs:529` | Liefert eine Momentaufnahme aller verwaisten Pins im Speicher. |
| `pub fn clear_orphan_pin(&self, seq_no: u64)` | `crates/contextra-checkpoint/src/orphan.rs:534` | Entfernt eine einzelne gepinnte Sequenznummer aus dem Speicher und persistiert. |
| `pub fn drain_orphaned_checkpoints(&self) -> Vec<StateCheckpoint>` | `crates/contextra-checkpoint/src/orphan.rs:551` | Entnimmt alle verwaisten Checkpoints aus dem Speicher und aktualisiert die Statusdatei. |
| `pub fn get_orphaned_checkpoints(&self) -> Vec<StateCheckpoint>` | `crates/contextra-checkpoint/src/orphan.rs:568` | Liefert eine Momentaufnahme aller verwaisten Checkpoints im Speicher. |
| `pub fn clear_orphaned_checkpoint(&self, tx_id: TxId)` | `crates/contextra-checkpoint/src/orphan.rs:573` | Entfernt einen einzelnen verwaisten Checkpoint anhand seiner Transaktions-ID und persistiert. |
| `pub fn clear_all(&self)` | `crates/contextra-checkpoint/src/orphan.rs:586` | Löscht alle Einträge (Pins und Checkpoints) und leert die Statusdatei. |

#### `struct OrphanRegistry`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:129`
```rust
pub struct OrphanRegistry
```
- Impl-Block (Trait `Default`) `crates/contextra-checkpoint/src/orphan.rs:134`
- Impl-Block `crates/contextra-checkpoint/src/orphan.rs:141`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(file_path: impl Into<std::path::PathBuf>) -> Self` | `crates/contextra-checkpoint/src/orphan.rs:143` | Erzeugt eine neue Instanz von [`OrphanRegistry`]. |
| `pub fn register_orphan(&self, pin_id: PinId) -> std::io::Result<()>` | `crates/contextra-checkpoint/src/orphan.rs:150` | Registriert synchron einen verwaisten Pin. |
| `pub fn get_orphans(&self) -> Vec<PinId>` | `crates/contextra-checkpoint/src/orphan.rs:160` | Liefert alle aktuell registrierten verwaisten Pin-Sequenznummern. |
| `pub fn clear_all(&self)` | `crates/contextra-checkpoint/src/orphan.rs:169` | Löscht alle Einträge im Speicher und auf der Platte synchron. |
| `pub async fn recover_and_clean<S: crate::snapshot_source::SnapshotSource>( &self, storage: &S, ) -> Result<Vec<PinId>>` | `crates/contextra-checkpoint/src/orphan.rs:180` | Bereinigt alle verwaisten Pins durch Entpinnen in der Storage-Engine. # Ablauf von `recover_and_clean`: 1. Liste aller verwaisten Pins abfragen. 2. Für jeden Pi |

#### `struct OrphanState`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:241`
```rust
pub struct OrphanState
```
Dokumentation: Serialisierbarer Gesamtzustand aller verwaisten Checkpoints und Pins einer Instanz. # Felder - `checkpoints`: Liste unvollendeter Transaktions-Checkpoints. - `pinned_seq_nos`: Liste unvollendeter Pin-Reservierungen. - `persist_path`: Dateipfad zur dauerhaften Ablage im JSON-Format.
- Impl-Block `crates/contextra-checkpoint/src/orphan.rs:250`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `checkpoints` | `Vec<StateCheckpoint>` | `crates/contextra-checkpoint/src/orphan.rs:243` |
| `pinned_seq_nos` | `Vec<PinnedSeqNoOrphan>` | `crates/contextra-checkpoint/src/orphan.rs:245` |
| `persist_path` | `std::path::PathBuf` | `crates/contextra-checkpoint/src/orphan.rs:247` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn persist_sync(&self) -> std::io::Result<()>` | `crates/contextra-checkpoint/src/orphan.rs:252` | Schreibt den Zustand synchron als formatierte JSON-Datei auf die Festplatte (ggf. über Vfs). |
| `pub fn persist_vfs(&self, vfs: &dyn crate::vfs::CheckpointVfs) -> std::io::Result<()>` | `crates/contextra-checkpoint/src/orphan.rs:261` | Schreibt den Zustand über die Vfs-Schnittstelle crash-konsistent (`write_atomic`). |
| `pub fn load_sync(path: &std::path::Path) -> Self` | `crates/contextra-checkpoint/src/orphan.rs:270` | Lädt den Zustand synchron von der Festplatte oder liefert einen leeren Zustand bei Fehlern. |
| `pub fn load_vfs(vfs: &dyn crate::vfs::CheckpointVfs, path: &std::path::Path) -> Self` | `crates/contextra-checkpoint/src/orphan.rs:284` | Lädt den Zustand über die Vfs-Schnittstelle. |

#### `struct PinnedSeqNoOrphan`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:213`
```rust
pub struct PinnedSeqNoOrphan
```
Dokumentation: Verwaiste gepinnte Sequenznummer — wird beim Recovery verarbeitet. # Felder - `seq_no`: Die Sequenznummer, die entpinnt werden muss. - `timestamp_ms`: Zeitpunkt der Registrierung in Millisekunden.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `seq_no` | `u64` | `crates/contextra-checkpoint/src/orphan.rs:215` |
| `timestamp_ms` | `u64` | `crates/contextra-checkpoint/src/orphan.rs:217` |

#### `struct RecoveryReport`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:38`
```rust
pub struct RecoveryReport
```
Dokumentation: Bericht über wiederhergestellte und aufgeräumte verwaiste Pins und Checkpoints. # Felder - `recovered_pins`: Liste aller erfolgreich entpinnten Sequenznummern. - `recovered_checkpoints`: Liste aller bereinigten Transaktions-IDs.
- Impl-Block `crates/contextra-checkpoint/src/orphan.rs:45`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `recovered_pins` | `Vec<PinId>` | `crates/contextra-checkpoint/src/orphan.rs:40` |
| `recovered_checkpoints` | `Vec<TxId>` | `crates/contextra-checkpoint/src/orphan.rs:42` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn is_empty(&self) -> bool` | `crates/contextra-checkpoint/src/orphan.rs:47` | Liefert `true`, wenn keine verwaisten Einträge vorhanden waren oder bereinigt wurden. |
| `pub fn total_recovered(&self) -> usize` | `crates/contextra-checkpoint/src/orphan.rs:52` | Liefert die Gesamtzahl aller bereinigten Pins und Checkpoints. |

#### `fn await_pending_rollbacks`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:647`
```rust
pub async fn await_pending_rollbacks()
```
Dokumentation: Rückwärtskompatibilität: Wartet auf ausstehende Rollbacks (No-op, da Hintergrund-Tasks entfernt wurden).

#### `fn clear_all_orphaned_checkpoints`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:644`
```rust
pub fn clear_all_orphaned_checkpoints()
```
- **deprecated**

#### `fn clear_orphaned_checkpoint`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:636`
```rust
pub fn clear_orphaned_checkpoint(_tx_id: TxId)
```
- **deprecated**

#### `fn get_orphaned_checkpoints`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:617`
```rust
pub fn get_orphaned_checkpoints() -> Vec<StateCheckpoint>
```
- **deprecated**

#### `fn get_orphaned_checkpoints_for_namespace`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:623`
```rust
pub fn get_orphaned_checkpoints_for_namespace(ns: &str) -> Vec<StateCheckpoint>
```
Dokumentation: Liefert verwaiste Checkpoints eines bestimmten Namespaces (veraltet).
- **deprecated**

#### `fn global_orphan_registry`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:78`
```rust
pub fn global_orphan_registry() -> &'static OrphanRegistry
```
- **deprecated**

#### `fn monotonic_timestamp_ms`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:106`
```rust
pub fn monotonic_timestamp_ms() -> u64
```
Dokumentation: Liest den Millisekunden-Zeitstempel über die standardmäßige SystemClock.

#### `fn monotonic_timestamp_ms_with`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:99`
```rust
pub fn monotonic_timestamp_ms_with(clock: &dyn Clock) -> u64
```
Dokumentation: Liest einen Millisekunden-Zeitstempel deterministisch über die übergebene Clock-Port-Instanz. # Parameter - `clock`: Referenz auf eine Implementierung des [`Clock`]-Traits.

#### `fn orphaned_checkpoint_count`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:656`
```rust
pub fn orphaned_checkpoint_count() -> usize
```
Dokumentation: Liefert die Anzahl der aktuell registrierten verwaisten ("orphaned") Checkpoints.
- **deprecated**

#### `fn pending_rollback_count`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:650`
```rust
pub fn pending_rollback_count() -> usize
```
Dokumentation: Rückwärtskompatibilität: Liefert die Anzahl ausstehender Rollback-Hintergrundaufgaben (immer 0).

#### `fn register_orphaned_checkpoint`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:604`
```rust
pub fn register_orphaned_checkpoint(cp: StateCheckpoint)
```
- **deprecated**

#### `fn register_pinned_seq_no_orphan`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:226`
```rust
pub fn register_pinned_seq_no_orphan(orphan: PinnedSeqNoOrphan)
```
- **deprecated**

#### `type PinId`
- Ort: `crates/contextra-checkpoint/src/orphan.rs:29`
```rust
pub type PinId = u64
```
Dokumentation: Typ-Alias für Sequenznummern, die als gepinnte Checkpoint-Identifikatoren verwaltet werden.

### Modul `snapshot_source`

#### `trait SnapshotSource`
- Ort: `crates/contextra-checkpoint/src/snapshot_source.rs:15`
```rust
pub trait SnapshotSource: Send + Sync + 'static
```
Dokumentation: Lokale Schnittstelle zur Speicher-Engine für Snapshot-, Checkpoint- und Rollback-Mechanik.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>` | `crates/contextra-checkpoint/src/snapshot_source.rs:17` | Liest einen Wert zu einem Schlüssel. |
| `fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>` | `crates/contextra-checkpoint/src/snapshot_source.rs:20` | Schreibt ein Schlüssel-Wert-Paar im Transaktionskontext. |
| `fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>>` | `crates/contextra-checkpoint/src/snapshot_source.rs:23` | Löscht einen Schlüssel im Transaktionskontext. |
| `fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>` | `crates/contextra-checkpoint/src/snapshot_source.rs:26` | Bestätigt eine Transaktion. |
| `fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>` | `crates/contextra-checkpoint/src/snapshot_source.rs:29` | Rollt eine Transaktion zurück. |
| `fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>` | `crates/contextra-checkpoint/src/snapshot_source.rs:32` | Setzt die Engine auf den Zustand einer früheren Transaktion zurück. |
| `fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>>` | `crates/contextra-checkpoint/src/snapshot_source.rs:35` | Flusht anstehende Daten auf die Festplatte. |
| `fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>>` | `crates/contextra-checkpoint/src/snapshot_source.rs:38` | Liefert die höchste vergebene Sequenznummer. |
| `fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>>` | `crates/contextra-checkpoint/src/snapshot_source.rs:41` | Liefert die ID der letzten Transaktion. |
| `fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>` | `crates/contextra-checkpoint/src/snapshot_source.rs:44` | Pinnt einen Snapshot bei der angegebenen Sequenznummer gegen GC. |
| `fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>>` | `crates/contextra-checkpoint/src/snapshot_source.rs:47` | Gibt den gepinnten Snapshot wieder frei. |
| `fn scan_prefix<'a>( &'a self, prefix: &'a [u8], ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>>` | `crates/contextra-checkpoint/src/snapshot_source.rs:51` | Scannt Schlüssel mit dem angegebenen Präfix. |

#### `type BoxFuture`
- Ort: `crates/contextra-checkpoint/src/snapshot_source.rs:12`
```rust
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>
```
Dokumentation: Lokaler Alias für pinning-boxed Futures ohne externe Abhängigkeit.

### Modul `store`

#### `struct PersistentCheckpointStore`
- Ort: `crates/contextra-checkpoint/src/store.rs:235`
```rust
pub struct PersistentCheckpointStore<S: SnapshotSource>
```
Dokumentation: Registry für gespeicherte Checkpoints mit Thread-sicherem Zustand. # Zweck Verwaltet Checkpoint-Metadaten, koordiniert MVCC-Sequenznummern und stellt sicher, dass keine Ressourcenlecks oder TxId-Kollisionen auftreten. # Invarianten - Alle Methoden sind durch `RwLock` thread-sicher - `StorageEngine`-Zugriffe nutzen atomare Transaktionen via `TxId` - Keine Panics (Zero-Panic Doctrine)
- Impl-Block `crates/contextra-checkpoint/src/store.rs:261`
- Impl-Block (Trait `CheckpointRegistry`) `crates/contextra-checkpoint/src/store.rs:919`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `storage` | `Arc<S>` | `crates/contextra-checkpoint/src/store.rs:236` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub async fn open_and_recover( storage: Arc<S>, namespace: impl Into<String>, ) -> Result<(Self, RecoveryReport)>` | `crates/contextra-checkpoint/src/store.rs:271` | Öffnet einen PersistentCheckpointStore und führt eine vollständige Orphan-Recovery für Pins und Checkpoints durch. # Ablauf von `open_and_recover`: 1. Pfad zur  |
| `pub async fn open_and_recover_with_orphan_registry( storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Result<(Self, RecoveryReport)>` | `crates/contextra-checkpoint/src/store.rs:282` | Öffnet einen PersistentCheckpointStore mit einer spezifischen instanzgebundenen Orphan Registry und führt Orphan-Recovery durch. |
| `pub async fn open(storage: Arc<S>, namespace: impl Into<String>) -> Result<Self>` | `crates/contextra-checkpoint/src/store.rs:298` | Öffnet einen PersistentCheckpointStore asynchron mit Rekonstruktion und Monotonie-Garantie. |
| `pub async fn open_with_orphan_registry( storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Result<Self>` | `crates/contextra-checkpoint/src/store.rs:306` | Öffnet einen PersistentCheckpointStore mit einer spezifischen instanzgebundenen Orphan Registry. |
| `pub fn new(storage: Arc<S>, namespace: impl Into<String>) -> Result<Self>` | `crates/contextra-checkpoint/src/store.rs:335` | Synchroner Konstruktor für Umgebungen ohne asynchronen Tokio-Einstiegspunkt. |
| `pub fn new_with_orphan_registry( storage: Arc<S>, namespace: impl Into<String>, orphan_registry: Arc<InstanceOrphanRegistry>, ) -> Result<Self>` | `crates/contextra-checkpoint/src/store.rs:343` | Synchroner Konstruktor mit injizierter Orphan Registry. |
| `pub async fn allocate_tx(&self) -> Result<TxId>` | `crates/contextra-checkpoint/src/store.rs:414` | Reserviert eine neue Transaktions-ID aus dem reservierten internen Bereich (`INTERNAL_BASE`). # Ablauf von `allocate_tx`: 1. Atomaren Zähler `tx_counter` inkrem |
| `pub fn skipped_rollback_count(&self) -> u64` | `crates/contextra-checkpoint/src/store.rs:437` | Liefert die Anzahl übersprungener Rollbacks. |
| `pub fn checkpoint_guard_skipped_rollback_count(&self) -> u64` | `crates/contextra-checkpoint/src/store.rs:442` | Alias für `skipped_rollback_count`. |
| `pub fn checkpoint_counter(&self) -> u64` | `crates/contextra-checkpoint/src/store.rs:447` | Liefert den Zählerstand für Checkpoint-Zeitstempel. |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-checkpoint/src/store.rs:455` | Setzt den Zeitgeber für den `PersistentCheckpointStore`. # Composition Root Der Composition Root setzt per `with_clock` eine injizierte Uhr (Default ist `System |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | `crates/contextra-checkpoint/src/store.rs:464` | Liefert den aktuellen Clock-Port. |
| `pub fn set_metrics_sink(&mut self, sink: Arc<dyn MetricsSink>)` | `crates/contextra-checkpoint/src/store.rs:469` | Setzt die Metrik-Senke. |
| `pub fn with_metrics_sink(mut self, sink: Arc<dyn MetricsSink>) -> Self` | `crates/contextra-checkpoint/src/store.rs:474` | Konfiguriert den Store mit einer Metrik-Senke. |
| `pub fn monotonic_timestamp_ms(&self) -> u64` | `crates/contextra-checkpoint/src/store.rs:480` | Liefert einen monoton steigenden Zeitstempel in Millisekunden. |
| `pub fn create_guard(&self, tx_id: TxId) -> Result<CheckpointGuard<S>>` | `crates/contextra-checkpoint/src/store.rs:491` | Creates an ephemeral transactional checkpoint RAII guard. If the returned guard is dropped without calling `.commit()`, the underlying storage is automatically  |
| `pub async fn create_checkpoint( &self, name: &str, collection_id: &str, seq_no: u64, tx_id: TxId, metadata: serde_json::Value, ) -> Result<CheckpointMeta>` | `crates/contextra-checkpoint/src/store.rs:518` | Creates a new persistent checkpoint. # Ablauf von `create_checkpoint`: 1. Bezeichner `name` und `collection_id` validieren. 2. `write_lock` akquirieren für exkl |
| `pub async fn drop_checkpoint(&self, name: &str) -> Result<()>` | `crates/contextra-checkpoint/src/store.rs:615` | Deletes a persistent checkpoint by name. # Ablauf von `drop_checkpoint`: 1. Bezeichner validieren und `write_lock` holen. 2. Checkpoint laden. 3. Eindeutige int |
| `pub async fn list_checkpoints(&self) -> Result<Vec<CheckpointMeta>>` | `crates/contextra-checkpoint/src/store.rs:714` | Listet alle persistent gespeicherten Checkpoints im konfigurierten Namespace auf. # Ablauf von `list_checkpoints`: 1. Präfix-Scan auf der Storage-Engine für `{n |
| `pub async fn get_checkpoint(&self, name: &str) -> Result<Option<CheckpointMeta>>` | `crates/contextra-checkpoint/src/store.rs:748` | Ruft einen Checkpoint nach Namen ab. |
| `pub async fn restore_checkpoint(&self, name: &str) -> Result<CheckpointMeta>` | `crates/contextra-checkpoint/src/store.rs:760` | Restores the system to a specific checkpoint by name. This will rollback the underlying storage to the transaction ID of the checkpoint. # Serialisierungsbarrie |
| `pub fn storage(&self) -> &Arc<S>` | `crates/contextra-checkpoint/src/store.rs:790` | Liefert eine Referenz auf die Orphan-Registry. Liefert eine Referenz auf die zugrundeliegende Speicher-Engine. |
| `pub fn orphan_registry(&self) -> &Arc<InstanceOrphanRegistry>` | `crates/contextra-checkpoint/src/store.rs:795` | Liefert eine Referenz auf die Orphan-Registry. |
| `pub async fn flush_orphan_registry(&self) -> Result<()>` | `crates/contextra-checkpoint/src/store.rs:800` | Flusht die Orphan-Registry asynchron auf Platte. |
| `pub async fn shutdown(&self) -> Result<()>` | `crates/contextra-checkpoint/src/store.rs:808` | Führt einen expliziten Flush der Orphan-Registry beim Herunterfahren durch. |
| `pub async fn close(&self) -> Result<()>` | `crates/contextra-checkpoint/src/store.rs:813` | Alias für `shutdown()`. Führt einen kontrollierten Flush beim Schließen aus. |
| `pub fn register_pinned_seq_no_orphan(&self, orphan: PinnedSeqNoOrphan)` | `crates/contextra-checkpoint/src/store.rs:818` | Registriert einen verwaisten Sequenz-Pin. |
| `pub fn register_orphaned_checkpoint(&self, cp: StateCheckpoint)` | `crates/contextra-checkpoint/src/store.rs:823` | Registriert einen verwaisten Checkpoint. |
| `pub fn get_orphaned_checkpoints(&self) -> Vec<StateCheckpoint>` | `crates/contextra-checkpoint/src/store.rs:828` | Liefert alle verwaisten Checkpoints. |
| `pub fn clear_orphaned_checkpoint(&self, tx_id: TxId)` | `crates/contextra-checkpoint/src/store.rs:833` | Löscht einen verwaisten Checkpoint anhand seiner Transaktions-ID. |
| `pub fn clear_all_orphaned_checkpoints(&self)` | `crates/contextra-checkpoint/src/store.rs:838` | Löscht alle verwaisten Checkpoints. |
| `pub async fn recover_orphaned_pins(&self) -> Result<Vec<PinId>>` | `crates/contextra-checkpoint/src/store.rs:850` | Recovers all registered/persisted orphaned sequence pins (ADR-052). # Ablauf von `recover_orphaned_pins`: 1. Liste verwaister Pins aus der Orphan-Registry abruf |
| `pub async fn recover_orphaned_checkpoints(&self) -> Result<Vec<TxId>>` | `crates/contextra-checkpoint/src/store.rs:874` | Recovers all registered/persisted orphaned checkpoints during controlled startup or recovery. Checks the serialization barrier (`last_tx <= cp.tx_id`) before ex |
| `pub async fn create_hardlink_clone( &self, seq_no: u64, source_dir: &Path, target_dir: &Path, snapshot_pinner: &(dyn SnapshotPinner + '_), ) -> Result<HardlinkCloneResult>` | `crates/contextra-checkpoint/src/store.rs:905` | Creates a physical hardlink clone of all SSTable files visible for `seq_no` into `target_dir`. Delegates to [`DefaultHardlinkCloner`] while holding a snapshot p |

#### `trait CheckpointRegistry`
- Ort: `crates/contextra-checkpoint/src/store.rs:47`
```rust
pub trait CheckpointRegistry: Send + Sync
```
Dokumentation: Trait für die erweiterte Checkpoint-Verwaltung inklusive Orphan-Recovery. Definiert die Schnittstelle zum Speichern, Laden und Bereinigen von Checkpoints.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn save_checkpoint<'a>(&'a self, meta: CheckpointMeta) -> BoxFuture<'a, Result<()>>` | `crates/contextra-checkpoint/src/store.rs:49` | Speichert die Metadaten eines Checkpoints asynchron ab. |
| `fn load_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<Option<CheckpointMeta>>>` | `crates/contextra-checkpoint/src/store.rs:51` | Lädt die Metadaten eines Checkpoints anhand der Sequenznummer. |
| `fn list_checkpoints<'a>(&'a self) -> BoxFuture<'a, Result<Vec<CheckpointMeta>>>` | `crates/contextra-checkpoint/src/store.rs:53` | Listet alle im Store registrierten Checkpoints auf. |
| `fn orphan_registry(&self) -> Option<&Arc<InstanceOrphanRegistry>>` | `crates/contextra-checkpoint/src/store.rs:56` | Liefert eine Referenz auf die zugeordnete Orphan-Registry, falls vorhanden. |
| `fn recover_orphaned_pins<'a>(&'a self) -> BoxFuture<'a, Result<Vec<PinId>>>` | `crates/contextra-checkpoint/src/store.rs:61` | Stellt verwaiste Pins wieder her und unpinnt sie in der Speicherengine. |
| `fn recover_orphaned_checkpoints<'a>(&'a self) -> BoxFuture<'a, Result<Vec<TxId>>>` | `crates/contextra-checkpoint/src/store.rs:66` | Führt für alle verwaisten Checkpoints ein kontrolliertes Rollback aus. |
| `fn flush_orphan_registry<'a>(&'a self) -> BoxFuture<'a, Result<()>>` | `crates/contextra-checkpoint/src/store.rs:71` | Schreibt ungespeicherte Orphan-Einträge auf die Festplatte. |

### Modul `vfs`

#### `trait CheckpointVfs`
- Ort: `crates/contextra-checkpoint/src/vfs.rs:9`
```rust
pub trait CheckpointVfs: Send + Sync
```
Dokumentation: Lokale minimale VFS-Schnittstelle für `contextra-checkpoint`.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn write_atomic(&self, path: &Path, data: &[u8]) -> std::io::Result<()>` | `crates/contextra-checkpoint/src/vfs.rs:11` | Schreibt Daten atomar an den angegebenen Pfad. |
| `fn read(&self, path: &Path) -> std::io::Result<Vec<u8>>` | `crates/contextra-checkpoint/src/vfs.rs:14` | Liest den gesamten Dateiinhalt an dem angegebenen Pfad. |

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl From<CheckpointError> for ContextraError` — `crates/contextra-checkpoint/src/compat/error.rs:8` (0 pub Methoden)
- `impl<T: StorageEngine + ?Sized> SnapshotSource for T` — `crates/contextra-checkpoint/src/compat/snapshot_bridge.rs:28` (0 pub Methoden)
- `impl<F> SnapshotPinner for F where F: Fn(u64) -> Box<dyn std::any::Any + Send + Sync> + Send + Sync,` — `crates/contextra-checkpoint/src/hardlink_cloner.rs:35` (0 pub Methoden)
- `impl SnapshotPinner for ()` — `crates/contextra-checkpoint/src/hardlink_cloner.rs:44` (0 pub Methoden)
- `impl SnapshotPinner for Arc<contextra_mvcc::SnapshotRegistry>` — `crates/contextra-checkpoint/src/hardlink_cloner.rs:51` (0 pub Methoden)
