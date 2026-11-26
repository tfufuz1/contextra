# Öffentliche API: `contextra-testkit`

Vollständige Übersicht aller öffentlichen Typen, Traits, Structs und Funktionen in `contextra-testkit`.

## `src/conformance/clock.rs`

| Signatur | Beschreibung |
|---|---|
| `pub fn run_clock_suite<C: Clock>(clock: &C) -> ConformanceResult ` | Führt die vollständige Konformitätsprüfung für eine [`Clock`]-Implementierung durch. |

## `src/conformance/failure.rs`

| Signatur | Beschreibung |
|---|---|
| `pub struct ConformanceFailure ` |  |
| `pub suite_name: &'static str,` | Name der ausgeführten Suite (z. B. "StorageEngine", "VectorIndex", "Clock"). |
| `pub test_case: &'static str,` | Name des spezifischen fehlgeschlagenen Testfalls (z. B. "delete_removes_key"). |
| `pub details: String,` | Detaillierte Beschreibung der festgestellten Vertragsverletzung. |
| `pub fn new(` | Erstellt eine neue [`ConformanceFailure`]-Instanz. |
| `pub type ConformanceResult = Result<(), ConformanceFailure>` | Spezialisierter Result-Typ für Konformitätsläufe: `Ok(())` oder `Err(ConformanceFailure)`. |

## `src/conformance/graph.rs`

| Signatur | Beschreibung |
|---|---|
| `pub async fn run_graph_index_suite<F, Fut, G>(factory: F) -> ConformanceResult` | Führt die vollständige Konformitätsprüfung für eine [`GraphIndex`]-Implementierung durch. |

## `src/conformance/metrics.rs`

| Signatur | Beschreibung |
|---|---|
| `pub fn run_metrics_sink_suite<M: MetricsSink>(sink: &M) -> ConformanceResult ` | Führt die vollständige Konformitätsprüfung für eine [`MetricsSink`]-Implementierung durch. |

## `src/conformance/mod.rs`

| Signatur | Beschreibung |
|---|---|
| `pub mod clock` | Konformitätssuite für den `Clock`-Port. |
| `pub mod failure` | Typen zur detaillierten Erfassung von Konformitätsfehlschlägen. |
| `pub mod graph` | Konformitätssuite für den `GraphIndex`-Port. |
| `pub mod macros` | Hilfsmakros für Konformitätsprüfungen (`assert_conformance!`). |
| `pub mod metrics` | Konformitätssuite für den `MetricsSink`-Port. |
| `pub mod rng` | Konformitätssuite für den Zufallszahlen-Port (`Rng`). |
| `pub mod storage` | Konformitätssuite für den `StorageEngine`-Port. |
| `pub mod text` | Konformitätssuite für den `TextIndex`-Port. |
| `pub mod vector` | Konformitätssuite für den `VectorIndex`-Port. |
| `pub mod vfs` | Konformitätssuite für den `Vfs`-Port. |
| `pub use clock::run_clock_suite` |  |
| `pub use failure::{ConformanceFailure, ConformanceResult}` |  |
| `pub use graph::run_graph_index_suite` |  |
| `pub use metrics::run_metrics_sink_suite` |  |
| `pub use rng::run_rng_suite` |  |
| `pub use storage::run_storage_engine_suite` |  |
| `pub use text::run_text_index_suite` |  |
| `pub use vector::run_vector_index_suite` |  |
| `pub use vfs::run_vfs_suite` |  |

## `src/conformance/rng.rs`

| Signatur | Beschreibung |
|---|---|
| `pub fn run_rng_suite<R: Rng>(rng: &R) -> ConformanceResult ` | Führt die vollständige Konformitätsprüfung für einen [`Rng`]-Port durch. |

## `src/conformance/storage.rs`

| Signatur | Beschreibung |
|---|---|
| `pub async fn run_storage_engine_suite<F, Fut, S>(factory: F) -> ConformanceResult` | Führt die vollständige Konformitätsprüfung für eine [`StorageEngine`]-Implementierung durch. |

## `src/conformance/text.rs`

| Signatur | Beschreibung |
|---|---|
| `pub async fn run_text_index_suite<F, Fut, T>(factory: F) -> ConformanceResult` | Führt die vollständige Konformitätsprüfung für eine [`TextIndex`]-Implementierung durch. |

## `src/conformance/vector.rs`

| Signatur | Beschreibung |
|---|---|
| `pub async fn run_vector_index_suite<F, Fut, V>(factory: F) -> ConformanceResult` | Führt die vollständige Konformitätsprüfung für eine [`VectorIndex`]-Implementierung durch. |

## `src/conformance/vfs.rs`

| Signatur | Beschreibung |
|---|---|
| `pub fn run_vfs_suite<V: Vfs>(vfs: &V) -> ConformanceResult ` | Führt die umfassende Konformitätsprüfsuite gegen eine gegebene [`Vfs`]-Instanz aus. |

## `src/fault_vfs.rs`

| Signatur | Beschreibung |
|---|---|
| `pub struct FaultConfig ` |  |
| `pub fail_writes_after: Option<usize>,` | Schreiboperationen nach dieser Anzahl erfolgreicher Schreibvorgänge fehlschlagen lassen. |
| `pub fail_syncs_after: Option<usize>,` | Sync- bzw. Flush-Operationen (`fsync`) nach dieser Anzahl erfolgreicher Aufrufe fehlschlagen lassen. |
| `pub fail_reads_after: Option<usize>,` | Leseoperationen nach dieser Anzahl erfolgreicher Lesevorgänge fehlschlagen lassen. |
| `pub fail_all: bool,` | Globaler Notschalter: Alle Operationen schlagen sofort fehl. |
| `pub struct FaultVfs ` |  |
| `pub fn new() -> Self ` | Erstellt eine neue, leere [`FaultVfs`]-Instanz ohne aktive Fehlerregeln. |
| `pub fn set_config(&self, config: FaultConfig) ` | Aktualisiert die aktive Konfiguration für Fehlerinjektion. |
| `pub fn trigger_crash(&self) ` | Simuliert einen abrupten Systemabsturz (Power-Cut). |
| `pub fn reset_counters(&self) ` | Setzt die Operationszähler und den Crash-Status zurück. |
| `pub fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()> ` | Schreibt Bytes an den angegebenen Pfad unter Berücksichtigung der Fehlerrichtlinien. |
| `pub fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> ` | Liest Bytes vom angegebenen Pfad unter Berücksichtigung der Fehlerrichtlinien. |
| `pub fn sync(&self) -> io::Result<()> ` | Simuliert einen `fsync`-Aufruf zur dauerhaften Bestätigung auf dem Speichermedium. |
| `pub fn write_count(&self) -> usize ` | Liefert die Anzahl der bisher versuchten Schreiboperationen. |
| `pub fn read_count(&self) -> usize ` | Liefert die Anzahl der bisher versuchten Leseoperationen. |
| `pub fn sync_count(&self) -> usize ` | Liefert die Anzahl der bisher versuchten Sync-Operationen. |

## `src/in_memory_graph.rs`

| Signatur | Beschreibung |
|---|---|
| `pub struct InMemoryGraphIndex ` |  |
| `pub fn new() -> Self ` | Erstellt einen neuen, leeren `InMemoryGraphIndex`. |

## `src/in_memory_store.rs`

| Signatur | Beschreibung |
|---|---|
| `pub struct InMemoryStorageEngine ` |  |
| `pub fn new() -> Self ` | Erstellt eine neue, leere In-Memory-Speicher-Engine. |

## `src/in_memory_text.rs`

| Signatur | Beschreibung |
|---|---|
| `pub struct InMemoryTextIndex ` |  |
| `pub fn new() -> Self ` | Erstellt einen neuen, leeren `InMemoryTextIndex`. |

## `src/in_memory_vector.rs`

| Signatur | Beschreibung |
|---|---|
| `pub struct InMemoryVectorIndex ` |  |
| `pub fn new(dim: usize) -> Self ` | Erstellt einen neuen `InMemoryVectorIndex` mit der erwarteten Vektordimension `dim`. |

## `src/lib.rs`

| Signatur | Beschreibung |
|---|---|
| `pub mod conformance` | Modul mit standardisierten Konformitätsprüfsuiten für Contextra-Port-Traits. |
| `pub mod fault_vfs` | Virtuelles Dateisystem mit deterministischer Fehlerinjektion (I/O- und fsync-Failures). |
| `pub mod in_memory_graph` | Deterministische In-Memory-Referenzimplementierung des `GraphIndex`-Ports. |
| `pub mod in_memory_store` | Transaktionale In-Memory-Referenzimplementierung des `StorageEngine`-Ports. |
| `pub mod in_memory_text` | Deterministische In-Memory-Referenzimplementierung des `TextIndex`-Ports. |
| `pub mod in_memory_vector` | Deterministische In-Memory-Referenzimplementierung des `VectorIndex`-Ports. |
| `pub mod manual_clock` | Manuell steuerbare Uhr zur reproduzierbaren Zeitmessung im `Clock`-Port. |
| `pub mod mocks` | Mocks für externe Provider wie Embedding-Generatoren und Metrik-Sinks. |
| `pub mod reference_model` | Formales Referenzmodell zur Prüfung von MVCC- und Transaktionsisolationsgarantien. |
| `pub mod doubles ` | Baukasten-Zielmodul `doubles`: Test-Doubles, In-Memory-Referenzen und steuerbare Uhren. |
| `pub use crate::in_memory_graph::InMemoryGraphIndex` |  |
| `pub use crate::in_memory_store::InMemoryStorageEngine` |  |
| `pub use crate::in_memory_text::InMemoryTextIndex` |  |
| `pub use crate::in_memory_vector::InMemoryVectorIndex` |  |
| `pub use crate::manual_clock::ManualClock` |  |
| `pub use crate::mocks::*` |  |
| `pub mod fault ` | Baukasten-Zielmodul `fault`: Fehlerinjektion und Fehlersimulation (VFS, I/O, Crashes). |
| `pub use crate::fault_vfs::{FaultConfig, FaultVfs}` |  |
| `pub mod model ` | Baukasten-Zielmodul `model`: Formale Orakel und Referenzmodelle für MVCC und KV-Semantik. |
| `pub use crate::reference_model::{RefOp, ReferenceModel}` |  |
| `pub use fault_vfs::{FaultConfig, FaultVfs}` |  |
| `pub use in_memory_graph::InMemoryGraphIndex` |  |
| `pub use in_memory_store::InMemoryStorageEngine` |  |
| `pub use in_memory_text::InMemoryTextIndex` |  |
| `pub use in_memory_vector::InMemoryVectorIndex` |  |
| `pub use manual_clock::ManualClock` |  |
| `pub use mocks::*` |  |
| `pub use reference_model::{RefOp, ReferenceModel}` |  |
| `pub fn mint_witness_for_tests(` |  |

## `src/manual_clock.rs`

| Signatur | Beschreibung |
|---|---|
| `pub struct ManualClock ` |  |
| `pub fn new(initial_nanos: u64) -> Self ` | Erstellt eine neue `ManualClock` mit einem gegebenen Nanosekundenwert. |
| `pub fn advance(&self, duration: Duration) ` | Stellt die Uhr um eine bestimmte Dauer vor. |
| `pub fn set_nanos(&self, nanos: u64) ` | Setzt die Uhr absolut auf einen bestimmten Nanosekunden-Zeitstempel. |
| `pub fn now_nanos(&self) -> u64 ` | Liest den aktuellen Zeitstempel in Nanosekunden seit UNIX_EPOCH aus. |
| `pub fn now_secs_f64(&self) -> f64 ` | Gibt den aktuellen Zeitstempel als Fließkommazahl in Sekunden zurück. |
| `pub fn now_system_time(&self) -> SystemTime ` | Konvertiert den aktuellen Zustand in einen Standard-Rust-[`SystemTime`]-Zeitpunkt. |

## `src/mocks.rs`

| Signatur | Beschreibung |
|---|---|
| `pub struct MockEmbedder ` | Deterministischer Mock-Embedding-Provider für Testläufe. |
| `pub dim: usize,` | Ziel-Dimensionalität der generierten Embeddings. |
| `pub fixed_output: Option<Vec<f32>>,` | Fester Ausgabevektor für reproduzierbare Assertions in Tests. |
| `pub fn new(dim: usize) -> Self ` | Erzeugt einen `MockEmbedder` mit der vorgegebenen Dimension. |
| `pub fn with_fixed_output(fixed_output: Vec<f32>) -> Self ` | Erzeugt einen `MockEmbedder` mit einem festen Vektor. |
| `pub enum MetricEvent ` |  |
| `pub struct TestMetricsSink ` |  |
| `pub fn new() -> Self ` | Erstellt eine leere [`TestMetricsSink`]. |
| `pub fn events(&self) -> Vec<MetricEvent> ` | Erstellt einen Schnappschuss (Snapshot) aller bisher aufgezeichneten Ereignisse. |
| `pub fn clear(&self) ` | Leert die Liste aller aufgezeichneten Metrik-Ereignisse. |

## `src/reference_model.rs`

| Signatur | Beschreibung |
|---|---|
| `pub enum RefOp ` |  |
| `pub struct ReferenceModel ` |  |
| `pub fn new() -> Self ` | Creates a new empty reference model initialized at sequence number 0. |
| `pub fn put(&mut self, key: &[u8], value: &[u8]) ` | Stages a `Put` operation in uncommitted pending buffer. |
| `pub fn delete(&mut self, key: &[u8]) ` | Stages a `Delete` operation in uncommitted pending buffer. |
| `pub fn rollback(&mut self) ` | Clears any uncommitted pending writes without advancing sequence number. |
| `pub fn commit(&mut self) -> u64 ` | Commits all pending operations under a new sequence number. |
| `pub fn commit_batch(&mut self, ops: Vec<RefOp>) -> u64 ` | Appends `ops` to pending buffer and commits them under a new sequence number. |
| `pub fn snapshot_seq(&self) -> u64 ` | Returns the current high water mark sequence number. |
| `pub fn get_at(&self, key: &[u8], seq: u64) -> Option<Vec<u8>> ` | Retrieves the value of `key` as visible at `seq`. |
| `pub fn scan_prefix_at(&self, prefix: &[u8], seq: u64) -> Vec<(Vec<u8>, Vec<u8>)> ` | Scans keys starting with `prefix` and returns key-value pairs visible at `seq`. |
| `pub fn scan_range_at(` | Scans key range `[start, end]` (using `Bound`) and returns visible pairs at `seq`. |
| `pub fn get_latest(&self, key: &[u8]) -> Option<Vec<u8>> ` | Gets the latest committed value for `key`. |
| `pub fn scan_prefix_latest(&self, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> ` | Scans all entries starting with `prefix` at latest sequence number. |
| `pub fn snapshot_map_at(&self, seq: u64) -> BTreeMap<Vec<u8>, Vec<u8>> ` | Returns a full snapshot map of latest visible key-values at `seq`. |
