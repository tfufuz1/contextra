# Schnittstellenspezifikation: `contextra-py`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-py` |
| Version | workspace |
| Pfad | `crates/contextra-py` |
| Rolle | Python-Bindings |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # Contextra Python Bindings (`contextra-py`)
>
> Dieses Crate stellt die offizielle Python-Brücke (FFI = Foreign Function Interface)
> für die eingebettete 4-Signal Hybrid-Search Vektordatenbank **Contextra** bereit.
> Es verbindet das Python-Ökosystem via PyO3 mit der Fassade (`contextra`).
>
> ## Ring-Zugehörigkeit & Architektur
>
> - **Ring / Klasse:** Ring 4 (Ränder / FFI-Bindings).
> - **Abhängigkeiten (eingehend):** Keine Workspace-Crates hängen von `contextra-py` ab.
> Die Nutzung erfolgt ausschließlich durch Python-Clients / User-Space.
> - **Abhängigkeiten (ausgehend):** `contextra-types`, `contextra`, `contextra-wire`

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `contextra` | normal | workspace | nein | contextra-mcp, contextra-py |
| `contextra-types` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `contextra-wire` | normal | workspace | nein | contextra-mcp, contextra-py, contextra-router, contextra-wire-fuzz |
| `flatbuffers` | normal | workspace | nein |  |
| `numpy` | normal | registry | ja |  |
| `parking_lot` | normal | workspace | nein |  |
| `pyo3` | normal | registry | ja |  |
| `pythonize` | normal | registry | ja |  |
| `serde_json` | normal | workspace | nein |  |
| `tempfile` | dev | workspace | nein |  |
| `tokio` | normal | workspace | nein |  |

**Verwendet von (Crate-Ebene):** –

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | – |
| `docid-128` | `contextra-types/docid-128` |
| `pyo3` | `dep:pyo3`, `dep:numpy`, `dep:pythonize` |
| `test-hooks` | – |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 16 |
| Quellzeilen | 3529 |
| Öffentliche Items (Modul-Ebene) | 51 |
| davon `fn` | 22 |
| davon `struct` | 8 |
| davon `const` | 7 |
| davon `mod` | 14 |
| Öffentliche Makros (`#[macro_export]`) | 2 |
| Öffentliche Re-Exports (`pub use`) | 11 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 0 |
| `#[deprecated]` | 1 |
| `#[cfg(feature…)]`-Gates | 9 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 75 / 89 = 84 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `bindings::collection` | `crates/contextra-py/src/bindings/collection.rs` | 128 |
| `bindings::common` | `crates/contextra-py/src/bindings/common.rs` | 488 |
| `bindings::crud_macros` | `crates/contextra-py/src/bindings/crud_macros.rs` | 494 |
| `bindings::db` | `crates/contextra-py/src/bindings/db.rs` | 236 |
| `bindings::db_stats` | `crates/contextra-py/src/bindings/db_stats.rs` | 64 |
| `bindings::document` | `crates/contextra-py/src/bindings/document.rs` | 35 |
| `bindings::functions` | `crates/contextra-py/src/bindings/functions.rs` | 135 |
| `bindings::hyperedge` | `crates/contextra-py/src/bindings/hyperedge.rs` | 197 |
| `bindings` | `crates/contextra-py/src/bindings/mod.rs` | 49 |
| `bindings::runtime_state` | `crates/contextra-py/src/bindings/runtime_state.rs` | 116 |
| `bindings::search_result` | `crates/contextra-py/src/bindings/search_result.rs` | 38 |
| `bindings::storage_stats` | `crates/contextra-py/src/bindings/storage_stats.rs` | 43 |
| `bindings::tests` | `crates/contextra-py/src/bindings/tests.rs` | 1026 |
| `bindings::vector_index_stats` | `crates/contextra-py/src/bindings/vector_index_stats.rs` | 43 |
| `kv_links` | `crates/contextra-py/src/kv_links.rs` | 296 |
| `crate` | `crates/contextra-py/src/lib.rs` | 141 |

## 6. Re-Exports (`pub use`)

- `pub use collection::PyCollection` — `crates/contextra-py/src/bindings/mod.rs:40` (Modul `bindings`)
- `pub use common::*` — `crates/contextra-py/src/bindings/mod.rs:41` (Modul `bindings`)
- `pub use db::PyContextra` — `crates/contextra-py/src/bindings/mod.rs:42` (Modul `bindings`)
- `pub use db_stats::PyDbStats` — `crates/contextra-py/src/bindings/mod.rs:43` (Modul `bindings`)
- `pub use document::PyDocument` — `crates/contextra-py/src/bindings/mod.rs:44` (Modul `bindings`)
- `pub use functions::*` — `crates/contextra-py/src/bindings/mod.rs:45` (Modul `bindings`)
- `pub use runtime_state::*` — `crates/contextra-py/src/bindings/mod.rs:46` (Modul `bindings`)
- `pub use search_result::PySearchResult` — `crates/contextra-py/src/bindings/mod.rs:47` (Modul `bindings`)
- `pub use storage_stats::PyStorageStats` — `crates/contextra-py/src/bindings/mod.rs:48` (Modul `bindings`)
- `pub use vector_index_stats::PyVectorIndexStats` — `crates/contextra-py/src/bindings/mod.rs:49` (Modul `bindings`)
- `pub use bindings::*` — `crates/contextra-py/src/lib.rs:51` (Modul `crate`)

## 7. Öffentliche Makros

- `macro_rules! contextra_crud_methods` — `crates/contextra-py/src/bindings/crud_macros.rs:39`
  - Generiert synchrone Python-CRUD-, Such- und Scan-Methoden für den angegebenen Struct-Typ.
  - # Implementierte Methoden
- `macro_rules! contextra_batch_methods` — `crates/contextra-py/src/bindings/crud_macros.rs:427`
  - Generiert synchrone Python-Massenoperationen (`insert_many`, `upsert_many`) für Batch-Transaktionen.
  - # Implementierte Methoden

## 8. Öffentliche Schnittstelle je Item

### Modul `bindings`

#### `mod collection`
- Ort: `crates/contextra-py/src/bindings/mod.rs:24`
```rust
pub mod collection
```

#### `mod common`
- Ort: `crates/contextra-py/src/bindings/mod.rs:25`
```rust
pub mod common
```

#### `mod crud_macros`
- Ort: `crates/contextra-py/src/bindings/mod.rs:22`
```rust
pub mod crud_macros
```

#### `mod db`
- Ort: `crates/contextra-py/src/bindings/mod.rs:26`
```rust
pub mod db
```

#### `mod db_stats`
- Ort: `crates/contextra-py/src/bindings/mod.rs:27`
```rust
pub mod db_stats
```

#### `mod document`
- Ort: `crates/contextra-py/src/bindings/mod.rs:28`
```rust
pub mod document
```

#### `mod functions`
- Ort: `crates/contextra-py/src/bindings/mod.rs:29`
```rust
pub mod functions
```

#### `mod hyperedge`
- Ort: `crates/contextra-py/src/bindings/mod.rs:30`
```rust
pub mod hyperedge
```

#### `mod runtime_state`
- Ort: `crates/contextra-py/src/bindings/mod.rs:31`
```rust
pub mod runtime_state
```

#### `mod search_result`
- Ort: `crates/contextra-py/src/bindings/mod.rs:32`
```rust
pub mod search_result
```

#### `mod storage_stats`
- Ort: `crates/contextra-py/src/bindings/mod.rs:33`
```rust
pub mod storage_stats
```

#### `mod vector_index_stats`
- Ort: `crates/contextra-py/src/bindings/mod.rs:38`
```rust
pub mod vector_index_stats
```

### Modul `bindings::collection`

#### `struct PyCollection`
- Ort: `crates/contextra-py/src/bindings/collection.rs:44`
```rust
pub struct PyCollection
```
Dokumentation: Kapselt eine benannte Contextra-Dokumentensammlung für Python. # Felder - `inner`: Thread-sicherer Zeiger auf die interne `ContextraCollection`. - `runtime`: Asynchrone Tokio-Laufzeitumgebung. - `poisoned`: Globales Vergiftungsflag bei Panics.
- Impl-Block `crates/contextra-py/src/bindings/collection.rs:50`
- Impl-Block `crates/contextra-py/src/bindings/collection.rs:66`
- Impl-Block `crates/contextra-py/src/bindings/hyperedge.rs:66`
- Impl-Block `crates/contextra-py/src/kv_links.rs:73`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `inner` | `Arc<ContextraCollection>` | `crates/contextra-py/src/bindings/collection.rs:45` |
| `runtime` | `Arc<Runtime>` | `crates/contextra-py/src/bindings/collection.rs:46` |
| `poisoned` | `Arc<AtomicBool>` | `crates/contextra-py/src/bindings/collection.rs:47` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( inner: Arc<ContextraCollection>, runtime: Arc<Runtime>, poisoned: Arc<AtomicBool>, ) -> Self` | `crates/contextra-py/src/bindings/collection.rs:52` | Erzeugt eine neue `PyCollection`-Instanz. |
| `pub fn is_poisoned(&self) -> bool` | `crates/contextra-py/src/bindings/collection.rs:73` | Gibt `True` zurück, wenn die Collection aufgrund eines vorherigen Rust-Panics vergiftet wurde. # Invariante (Fail-Closed) Sobald `is_poisoned` wahr ist, schlage |
| `pub fn _trigger_panic_for_test(&self, py: Python<'_>, message: Option<String>) -> PyResult<()>` | `crates/contextra-py/src/bindings/collection.rs:84` | Interner Testhook zur Verifikation von FFI-Panic-Isolation und Engine-Poisoning. # Panics Bewusstes Auslösen eines Panics zum Testen der Barriere. Wird durch `r |
| `pub fn stats(&self, py: Python<'_>) -> PyResult<PyVectorIndexStats>` | `crates/contextra-py/src/bindings/collection.rs:94` | Liefert detaillierte Statistiken über den HNSW-Vektorindex dieser Collection. # Ablauf 1. Asynchron `inner.stats()` unter GIL-Release ausführen. 2. Rust-Statist |
| `pub fn len(&self, py: Python<'_>) -> PyResult<usize>` | `crates/contextra-py/src/bindings/collection.rs:109` | Gibt die Gesamtzahl der Dokumente in dieser Collection zurück. |
| `pub fn is_empty(&self, py: Python<'_>) -> PyResult<bool>` | `crates/contextra-py/src/bindings/collection.rs:116` | Prüft, ob die Collection leer ist (`True` falls keine Dokumente enthalten sind). |
| `pub fn relate_n_ary<'py>( &self, py: Python<'py>, predicate: &str, participants: Vec<(Bound<'py, PyAny>, String)>, source_doc_id: Option<Bound<'py, PyAny>>, ) -> PyResult<u64>` | `crates/contextra-py/src/bindings/hyperedge.rs:84` | Legt eine n-äre Hyperkante an. # Parameter - `predicate`: Der Name des Prädikats als String. - `participants`: Liste von Tupeln `[(doc_id, role), ...]`, mindest |
| `pub fn put_kv<'py>( &self, py: Python<'py>, id: &Bound<'py, PyAny>, value: &Bound<'py, PyDict>, ) -> PyResult<()>` | `crates/contextra-py/src/kv_links.rs:91` | Speichert ein reines Key-Value-Paar direkt im Speicher ab, ohne Vektor- oder Volltextindizes zu verändern. # Parameter - `py`: Das Python GIL-Token. - `id`: Der |
| `pub fn put_kv_if_absent<'py>( &self, py: Python<'py>, id: &Bound<'py, PyAny>, value: &Bound<'py, PyDict>, ) -> PyResult<()>` | `crates/contextra-py/src/kv_links.rs:120` | Speichert ein Key-Value-Paar nur dann, wenn der Schlüssel noch nicht im Storage existiert. # Zweck Verhindert versehentliches Überschreiben existierender Einträ |
| `pub fn get_kv<'py>( &self, py: Python<'py>, id: &Bound<'py, PyAny>, ) -> PyResult<Option<PyObject>>` | `crates/contextra-py/src/kv_links.rs:146` | Liest ein Key-Value-Paar direkt anhand des Schlüssels aus dem Speicher. # Rückgabe - `Some(dict)` falls der Eintrag existiert (als Python-Dictionary deserialisi |
| `pub fn relate_bidirectional<'py>( &self, py: Python<'py>, from: &Bound<'py, PyAny>, to: &Bound<'py, PyAny>, label: &str, ) -> PyResult<()>` | `crates/contextra-py/src/kv_links.rs:174` | Erstellt atomar eine ungerichtete bzw. beidseitige Relation zwischen zwei Dokumenten. # Parameter - `from`: ID des Ursprungsdokuments. - `to`: ID des Zieldokume |
| `pub fn link_memories<'py>( &self, py: Python<'py>, from: &Bound<'py, PyAny>, to: &Bound<'py, PyAny>, relation: &str, ) -> PyResult<()>` | `crates/contextra-py/src/kv_links.rs:207` | Verknüpft zwei Memory-Knoten mit einer typisierten Zettelkasten-Relation (A-MEM). # Parameter - `from`: ID des Ausgangsdokuments. - `to`: ID des Zieldokuments.  |
| `pub fn get_links<'py>( &self, py: Python<'py>, id: &Bound<'py, PyAny>, ) -> PyResult<Vec<PyObject>>` | `crates/contextra-py/src/kv_links.rs:236` | Ruft alle ausgehenden Memory-Links für ein bestimmtes Dokument ab. # Rückgabe Eine Python-Liste von Dictionaries mit den Schlüsseln `target`, `relation` und `cr |

### Modul `bindings::common`

#### `fn check_subinterpreter_guard`
- Ort: `crates/contextra-py/src/bindings/common.rs:290`
```rust
pub fn check_subinterpreter_guard(py: Python<'_>) -> PyResult<()>
```
Dokumentation: Prüft explizit, ob das Crate innerhalb eines CPython Sub-Interpreters geladen wird. # Zweck Verhindert fehlerhafte Ausführung in isolierten Sub-Interpretern (Interpreter-ID != 0), wenn keine vollständige Threadpool-Isolation garantiert werden kann. # Fehlerfälle Liefert `PyImportError`, falls `interpreter_id != 0`.

#### `fn contextra_err`
- Ort: `crates/contextra-py/src/bindings/common.rs:436`
```rust
pub fn contextra_err(e: contextra_types::ContextraError) -> PyErr
```
Dokumentation: Übersetzt ein typisiertes Rust `ContextraError` in eine strukturierte Python-Exception (`PyErr`). # Ablauf 1. Wandelt `ContextraError` in ein Data Transfer Object (`ContextraErrorDto`) um. 2. Mappt die Fehlerkategorie (`kind`) auf die entsprechende Python-Exception-Klasse: - `"NotFound"` -> `NotFoundError` - `"Conflict"`, `"Transaction"` -> `IntegrityError` - `"TenantMismatch"`, `"NamespaceViolation"` -> `TenantError` - `"Storage"`, `"Io"`, `"WalCorruption"` -> `ContextraIOError` - `"Index"`, `"Text"` -> `ContextraIndexError` - `"Crypto"` -> `ContextraCryptoError` - `"InvalidInput"` -> `Contex

#### `fn dict_to_json`
- Ort: `crates/contextra-py/src/bindings/common.rs:66`
```rust
pub fn dict_to_json(d: &pyo3::Bound<'_, pyo3::types::PyDict>) -> PyResult<serde_json::Value>
```
Dokumentation: Konvertiert ein Python-Dictionary (`PyDict`) in einen `serde_json::Value`. # Zweck Wandelt dynamische Metadaten aus Python in das interne JSON-Format von Contextra um. # Fehlerfälle Liefert `PyValueError`, falls das Dictionary nicht JSON-serialisierbare Objekte enthält.

#### `fn doc_to_py`
- Ort: `crates/contextra-py/src/bindings/common.rs:386`
```rust
pub fn doc_to_py(py: Python<'_>, d: contextra::Document) -> PyResult<PyDocument>
```
Dokumentation: Konvertiert ein `contextra::Document` in ein `PyDocument`. # Ablauf 1. Metadaten-JSON (falls vorhanden) in ein natives Python-Dictionary überführen. 2. In eine `PyDocument`-Instanz kapseln.

#### `fn json_to_py`
- Ort: `crates/contextra-py/src/bindings/common.rs:374`
```rust
pub fn json_to_py(py: Python<'_>, val: &serde_json::Value) -> PyResult<PyObject>
```
Dokumentation: Konvertiert einen `serde_json::Value` zurück in ein natives Python-Objekt (`PyObject`). # Fehlerfälle Liefert `PyRuntimeError`, falls die Konvertierung scheitert.

#### `fn opt_dict_to_json`
- Ort: `crates/contextra-py/src/bindings/common.rs:76`
```rust
pub fn opt_dict_to_json( metadata: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>, ) -> PyResult<Option<serde_json::Value>>
```
Dokumentation: Konvertiert ein optionales Python-Dictionary in ein optionales `serde_json::Value`. # Rückgabe - `Ok(Some(Value))` wenn ein Dictionary übergeben wurde. - `Ok(None)` wenn `None` übergeben wurde.

#### `fn results_to_py`
- Ort: `crates/contextra-py/src/bindings/common.rs:403`
```rust
pub fn results_to_py( py: Python<'_>, results: Vec<contextra::SearchResult>, ) -> PyResult<Vec<PySearchResult>>
```
Dokumentation: Konvertiert einen Vektor von internen `contextra::SearchResult` in Python-taugliche `PySearchResult`. # Ablauf 1. Allokiert einen Vektor mit passender Kapazität. 2. Iteriert über die Treffer, wandelt Metadaten in Python-Objekte um und erzeugt `PySearchResult`.

#### `fn run_blocking_ffi`
- Ort: `crates/contextra-py/src/bindings/common.rs:334`
```rust
pub fn run_blocking_ffi<F, R>(py: Python<'_>, poisoned: &AtomicBool, f: F) -> PyResult<R> where F: FnOnce() -> PyResult<R> + Send, R: Send,
```
Dokumentation: Führt einen blockierenden Aufruf sicher über die FFI-Grenze hinweg aus (mit Panic-Catch und Poisoning). # Zweck & Invariante (ADR-N04 / APM-PY-A): Verhindert, dass unkontrollierte Rust-Panics in den Python-Prozess durchschlagen. # Ablauf in Schritten: 1. **Poisoning-Prüfung:** Prüft das atomare `poisoned`-Flag. Falls zuvor ein Panic aufgetreten ist, wird sofort mit `PyRuntimeError` abgebrochen (Fail-Closed). 2. **GIL-Release & Panic Catch:** Gibt mit `py.allow_threads` den Python-GIL frei und führt die Closure `f` gekapselt in `std::panic::catch_unwind` aus. 3. **Ergebnis-Auswertung:** - Bei E

#### `fn validate_batch_size`
- Ort: `crates/contextra-py/src/bindings/common.rs:213`
```rust
pub fn validate_batch_size(size: usize) -> PyResult<()>
```
Dokumentation: Prüft, dass eine Batch-Größe größer 0 ist und das Limit von 10.000 Elementen nicht überschreitet. # Fehlerfälle Liefert `ContextraValueError` bei leerer Batch oder Überschreitung des Limits.

#### `fn validate_collection_name`
- Ort: `crates/contextra-py/src/bindings/common.rs:142`
```rust
pub fn validate_collection_name(name: &str) -> PyResult<()>
```
Dokumentation: Validiert, dass ein Collection-Name nicht leer ist, keine Null-Bytes enthält und maximal 64 Zeichen lang ist. # Fehlerfälle Liefert `ContextraValueError` bei ungültigem Namen.

#### `fn validate_db_path`
- Ort: `crates/contextra-py/src/bindings/common.rs:168`
```rust
pub fn validate_db_path(path: &str) -> PyResult<()>
```
Dokumentation: Validiert, dass ein Datenbank-Dateipfad nicht leer ist und keine Null-Bytes enthält. # Fehlerfälle Liefert `ContextraValueError` bei fehlerhaftem Pfadstring.

#### `fn validate_id`
- Ort: `crates/contextra-py/src/bindings/common.rs:116`
```rust
pub fn validate_id(id: &str) -> PyResult<()>
```
Dokumentation: Validiert, dass eine Dokumenten-ID nicht leer ist, keine Null-Bytes enthält und maximal 1024 Zeichen lang ist. # Fehlerfälle Liefert `ContextraValueError` bei ungültiger ID.

#### `fn validate_id_obj`
- Ort: `crates/contextra-py/src/bindings/common.rs:253`
```rust
pub fn validate_id_obj(id_obj: &pyo3::Bound<'_, pyo3::types::PyAny>) -> PyResult<String>
```
Dokumentation: Validiert und extrahiert ein Dokument-ID-Objekt aus Python (erlaubt String oder nicht-negativen Integer). # Ablauf 1. Prüft, ob das Objekt ein String ist -> Validierung via `validate_id`. 2. Prüft, ob das Objekt ein Integer ist -> Sicherstellung, dass `>= 0` und `< u64::MAX`. 3. Gibt die ID als String zurück. # Fehlerfälle Liefert `ContextraValueError`, falls der Typ weder String noch Integer ist oder Schranken verletzt werden.

#### `fn validate_label`
- Ort: `crates/contextra-py/src/bindings/common.rs:90`
```rust
pub fn validate_label(label: &str) -> PyResult<()>
```
Dokumentation: Validiert, dass ein Beziehungs-Label nicht leer ist, keine Null-Bytes enthält und die Maximallänge nicht überschreitet. # Fehlerfälle Liefert `ContextraValueError` bei ungültigem Label.

#### `fn validate_query_text`
- Ort: `crates/contextra-py/src/bindings/common.rs:187`
```rust
pub fn validate_query_text(text: &str) -> PyResult<()>
```
Dokumentation: Validiert, dass ein Volltext-Suchquery nicht leer ist und keine Null-Bytes enthält. # Fehlerfälle Liefert `ContextraValueError` bei ungültigem Suchtext.

#### `fn validate_vector`
- Ort: `crates/contextra-py/src/bindings/common.rs:231`
```rust
pub fn validate_vector(vector: &[f32]) -> PyResult<()>
```
Dokumentation: Validiert, dass ein Vektor nicht leer ist und weder `NaN` noch unendliche Werte (`+/-Inf`) enthält. # Fehlerfälle Liefert `ContextraValueError` bei leeren oder unendlichen Gleitkommazahlen.

#### `const MAX_BATCH_SIZE`
- Ort: `crates/contextra-py/src/bindings/common.rs:54`
```rust
pub const MAX_BATCH_SIZE: usize = 10_000
```
Dokumentation: Maximale Batch-Größe für Massenoperationen (10.000 Einträge).

#### `const MAX_COLLECTION_NAME_LENGTH`
- Ort: `crates/contextra-py/src/bindings/common.rs:50`
```rust
pub const MAX_COLLECTION_NAME_LENGTH: usize = 64
```
Dokumentation: Maximale Länge für Collection-Namen (64 Zeichen).

#### `const MAX_ID_LENGTH`
- Ort: `crates/contextra-py/src/bindings/common.rs:48`
```rust
pub const MAX_ID_LENGTH: usize = 1024
```
Dokumentation: Maximale Länge für Dokumenten-IDs (1024 Zeichen bzw. Bytes).

#### `const MAX_LABEL_LENGTH`
- Ort: `crates/contextra-py/src/bindings/common.rs:52`
```rust
pub const MAX_LABEL_LENGTH: usize = 256
```
Dokumentation: Maximale Länge für Relations-Labels (256 Zeichen).

#### `const MAX_WORKER_THREADS`
- Ort: `crates/contextra-py/src/bindings/common.rs:45`
```rust
pub const MAX_WORKER_THREADS: usize = 256
```
Dokumentation: Maximale Anzahl an Worker-Threads für die Tokio-Laufzeitumgebung.

#### `const MIN_WORKER_THREADS`
- Ort: `crates/contextra-py/src/bindings/common.rs:43`
```rust
pub const MIN_WORKER_THREADS: usize = 1
```
Dokumentation: Minimale Anzahl an Worker-Threads für die Tokio-Laufzeitumgebung.

### Modul `bindings::db`

#### `struct PyContextra`
- Ort: `crates/contextra-py/src/bindings/db.rs:46`
```rust
pub struct PyContextra
```
Dokumentation: Zentrales Python-Handle für eine geöffnete Contextra-Datenbankinstanz (`Db`). # Felder - `inner`: Thread-sichere Referenz auf den Rust-Datenbankkern (`Contextra`). - `runtime`: Asynchrone Tokio-Multi-Thread-Laufzeitumgebung. - `worker_threads`: Konfigurierte Thread-Anzahl im Tokio-Pool. - `poisoned`: Vergiftungsindikator zur Verhinderung von Folgeaufrufen nach Panics.
- Impl-Block `crates/contextra-py/src/bindings/db.rs:53`
- Impl-Block `crates/contextra-py/src/bindings/db.rs:71`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `inner` | `Arc<Contextra>` | `crates/contextra-py/src/bindings/db.rs:47` |
| `runtime` | `Arc<Runtime>` | `crates/contextra-py/src/bindings/db.rs:48` |
| `worker_threads` | `usize` | `crates/contextra-py/src/bindings/db.rs:49` |
| `poisoned` | `Arc<AtomicBool>` | `crates/contextra-py/src/bindings/db.rs:50` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( inner: Arc<Contextra>, runtime: Arc<Runtime>, worker_threads: usize, poisoned: Arc<AtomicBool>, ) -> Self` | `crates/contextra-py/src/bindings/db.rs:55` | Konstruiert ein neues `PyContextra`-Handle. |
| `pub fn worker_threads(&self) -> usize` | `crates/contextra-py/src/bindings/db.rs:75` | Gibt die konfigurierte Anzahl an Tokio-Worker-Threads für diese Instanz zurück. |
| `pub fn is_poisoned(&self) -> bool` | `crates/contextra-py/src/bindings/db.rs:82` | Gibt `True` zurück, wenn die Datenbankinstanz nach einem vorangegangenen Rust-Panic vergiftet ist. |
| `pub fn _trigger_panic_for_test(&self, py: Python<'_>, message: Option<String>) -> PyResult<()>` | `crates/contextra-py/src/bindings/db.rs:92` | Interner Testhook zur Verifikation der FFI-Panik-Isolation. # Panics Test-Panic wird ausgelöst und durch `run_blocking_ffi` kontrolliert in `PyErr` gewandelt. |
| `pub fn collection(&self, name: &str, py: Python<'_>) -> PyResult<PyCollection>` | `crates/contextra-py/src/bindings/db.rs:134` | Öffnet oder erstellt eine spezifische Collection (Namespace) innerhalb der Datenbank. # Parameter - `name`: Name der Collection (maximal 64 Zeichen). # Rückgabe |
| `pub fn list_collections(&self, py: Python<'_>) -> PyResult<Vec<String>>` | `crates/contextra-py/src/bindings/db.rs:151` | Liefert eine Liste der Namen aller vorhandenen Collections in der Datenbank. |
| `pub fn drop_collection(&self, name: &str, py: Python<'_>) -> PyResult<()>` | `crates/contextra-py/src/bindings/db.rs:164` | Löscht eine Collection und entfernt alle darin enthaltenen Daten unwiderruflich aus dem Storage. # Parameter - `name`: Name der zu entfernenden Collection. |
| `pub fn flush(&self, py: Python<'_>) -> PyResult<()>` | `crates/contextra-py/src/bindings/db.rs:181` | Schreibt alle im Speicher gepufferten Daten (MemTable/WAL) persistent auf die Festplatte (`fsync`). |
| `pub fn stats(&self, py: Python<'_>) -> PyResult<PyDbStats>` | `crates/contextra-py/src/bindings/db.rs:190` | Ruft kombinierte Statistiken für Vektorindex, LSM-Tree, Kalibrierung und Drift ab. |
| `pub fn len(&self, py: Python<'_>) -> PyResult<usize>` | `crates/contextra-py/src/bindings/db.rs:217` | Gibt die Anzahl der Dokumente in der Standard-Collection zurück. |
| `pub fn is_empty(&self, py: Python<'_>) -> PyResult<bool>` | `crates/contextra-py/src/bindings/db.rs:226` | Prüft, ob die Standard-Collection leer ist. |

### Modul `bindings::db_stats`

#### `struct PyDbStats`
- Ort: `crates/contextra-py/src/bindings/db_stats.rs:32`
```rust
pub struct PyDbStats
```
Dokumentation: Ganzheitliche Datenbank- und Systembeobachtungs-Statistiken (`DbStats`). # Felder - `drift_status`: Zustand der Vektor-Driftanalyse ("stabil", "warnung", "kritisch", "unbekannt"). - `calibration_ece`: Erwarteter Kalibrierungsfehler (ECE = Expected Calibration Error, 0.0 bis 1.0). - `last_calibration_at`: UNIX-Zeitstempel der letzten Kalibrierungsmodell-Berechnung. - `active_memory_count`: Anzahl aktiver Dokumente/Memories in der Default-Collection. - `pid_pool_size`: Aktuelle Kandidaten-Pool-Größe des PID-Reglers. - `index_stats`: Detaillierte Statistiken über den HNSW-Vektorindex. - `storage_
- Impl-Block `crates/contextra-py/src/bindings/db_stats.rs:50`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `drift_status` | `String` | `crates/contextra-py/src/bindings/db_stats.rs:34` |
| `calibration_ece` | `Option<f32>` | `crates/contextra-py/src/bindings/db_stats.rs:36` |
| `last_calibration_at` | `Option<u64>` | `crates/contextra-py/src/bindings/db_stats.rs:38` |
| `active_memory_count` | `usize` | `crates/contextra-py/src/bindings/db_stats.rs:40` |
| `pid_pool_size` | `Option<usize>` | `crates/contextra-py/src/bindings/db_stats.rs:42` |
| `index_stats` | `PyVectorIndexStats` | `crates/contextra-py/src/bindings/db_stats.rs:44` |
| `storage_stats` | `PyStorageStats` | `crates/contextra-py/src/bindings/db_stats.rs:46` |

### Modul `bindings::document`

#### `struct PyDocument`
- Ort: `crates/contextra-py/src/bindings/document.rs:22`
```rust
pub struct PyDocument
```
Dokumentation: Ein aus Contextra abgerufener Datensatz (Dokument). # Felder - `id`: Der Primärschlüssel des Dokuments als String. - `metadata`: Assoziierte Metadaten des Dokuments (z.B. Python-Dict mit Freitextfeldern).
- Impl-Block `crates/contextra-py/src/bindings/document.rs:30`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-py/src/bindings/document.rs:24` |
| `metadata` | `Option<PyObject>` | `crates/contextra-py/src/bindings/document.rs:26` |

### Modul `bindings::functions`

#### `fn _trigger_panic_for_test`
- Ort: `crates/contextra-py/src/bindings/functions.rs:132`
```rust
pub fn _trigger_panic_for_test(py: Python<'_>, message: Option<String>) -> PyResult<()>
```
Dokumentation: Python-Funktion zum Testen der Panik-Isolation (nur bei Feature `test-hooks`). # Zweck Wird von der Python-Testsuite aufgerufen, um zu verifizieren, dass ein Rust-Panic sauber als `PyRuntimeError` im Python-Interpreter ankommt.
- Gate: `#[cfg(feature = "test-hooks")]`

#### `fn open`
- Ort: `crates/contextra-py/src/bindings/functions.rs:46`
```rust
pub fn open( py: Python<'_>, path: &str, dimension: usize, max_elements: Option<usize>, encryption_passphrase: Option<String>, distance_metric: Option<String>, ) -> PyResult<PyContextra>
```
Dokumentation: Öffnet oder erstellt eine Contextra-Datenbank am angegebenen Dateipfad. # Parameter - `path`: Dateipfad zum Datenbankverzeichnis (wird validiert, keine Null-Bytes). - `dimension`: Vektordimension (Standard: 768, passend für Standardmodelle wie `nomic-embed-text`; Schranke [1, 10000]). - `max_elements`: Optionale Obergrenze für Elemente im HNSW-Index (> 0). - `encryption_passphrase`: Optionale Passphrase zur transparenten Verschlüsselung der Datenbank. - `distance_metric`: Distanzmetrik für Vektoren ("cosine", "euclidean"/"l2", "dot"/"dotproduct"). # Ablauf in Schritten: 1. Pfad und Vektordimen

### Modul `bindings::hyperedge`

#### `fn validate_hyperedge_args`
- Ort: `crates/contextra-py/src/bindings/hyperedge.rs:33`
```rust
pub fn validate_hyperedge_args( predicate: &str, participants: &[(&str, &str)], source_doc_id: Option<&str>, ) -> PyResult<()>
```
Dokumentation: Validiert die Eingabeparameter für das Erstellen einer n-ären Hyperkante. # Parameter - `predicate`: Der Beziehungsname (z. B. "verhandelt", "gekauft"). - `participants`: Eine Slice von Tupeln bestehend aus `(doc_id, role)`. - `source_doc_id`: Optionales Referenzdokument, aus dem die Beziehung hervorging. # Invarianten & Grenzwerte: - Teilnehmeranzahl: Mindestens 2, maximal 64 Knoten. - Gültigkeit: Weder Prädikat noch Rollen noch Dokument-IDs dürfen leer sein oder ungültige Zeichen enthalten. # Fehlerfälle - `ContextraValueError`, falls die Teilnehmergrenzen verletzt sind oder Strings leer sin

### Modul `bindings::runtime_state`

#### `struct PyRuntimeState`
- Ort: `crates/contextra-py/src/bindings/runtime_state.rs:57`
```rust
pub struct PyRuntimeState
```
Dokumentation: Kapselt die Tokio-Laufzeitumgebung und die Thread-Konfiguration pro Python-Modul. # Felder - `runtime`: Referenzgezählter Handle (`Arc<Runtime>`) auf den Tokio-Multi-Thread-Pool. - `worker_threads`: Anzahl konfigurierter Worker-Threads in diesem Pool.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `runtime` | `Arc<Runtime>` | `crates/contextra-py/src/bindings/runtime_state.rs:58` |
| `worker_threads` | `usize` | `crates/contextra-py/src/bindings/runtime_state.rs:59` |

#### `fn get_runtime`
- Ort: `crates/contextra-py/src/bindings/runtime_state.rs:77`
```rust
pub fn get_runtime(py: Python<'_>) -> PyResult<Arc<Runtime>>
```
Dokumentation: Ruft die Tokio-Runtime für den aktuellen Python-Interpreter ab oder initialisiert sie bei Bedarf. # Zweck Stellt sicher, dass asynchrone Rust-Operationen in Python synchron ausgeführt werden können. # Ablauf in Schritten: 1. Sucht nach dem bereits geladenen Modul `contextra._contextra` oder `_contextra`. 2. Prüft, ob das Modulattribut `_runtime_state` bereits existiert und extrahiert den Handle. 3. Falls noch nicht vorhanden: Ermittelt Worker-Threads via `parse_worker_threads_env()`. 4. Erzeugt eine neue Multi-Thread-Tokio-Runtime mit dem Thread-Präfix `contextra-py-worker`. 5. Bindet die neue

#### `fn parse_worker_threads_env`
- Ort: `crates/contextra-py/src/bindings/runtime_state.rs:35`
```rust
pub fn parse_worker_threads_env() -> usize
```
Dokumentation: Liest und begrenzt die Umgebungsvariable `CONTEXTRA_WORKER_THREADS` auf den Bereich `[MIN_WORKER_THREADS, MAX_WORKER_THREADS]`. # Zweck Bestimmt die Anzahl der Hintergrund-Worker-Threads für die Tokio-Laufzeitumgebung. # Ablauf 1. Standardwert ermitteln: Halbe Anzahl verfügbarer CPU-Kerne (`available_parallelism`), mindestens jedoch auf `[MIN_WORKER_THREADS, MAX_WORKER_THREADS]` begrenzt (z. B. 2 bis 32). 2. Liest die Umgebungsvariable `CONTEXTRA_WORKER_THREADS`. 3. Parst den String zu `usize` und klemmt den Wert in die Schranken `MIN_WORKER_THREADS..=MAX_WORKER_THREADS`. 4. Liefert bei Fehler

### Modul `bindings::search_result`

#### `struct PySearchResult`
- Ort: `crates/contextra-py/src/bindings/search_result.rs:23`
```rust
pub struct PySearchResult
```
Dokumentation: Ein einzelnes Suchergebnis aus einer Vektor- oder Hybrid-Suche. # Felder - `id`: Der Primärschlüssel des gefundenen Dokuments. - `score`: Der berechnete Ähnlichkeitswert (höherer Wert bedeutet höhere Relevanz/Ähnlichkeit). - `metadata`: Assoziierte Metadaten des Treffers (falls bei der Suche angefordert).
- Impl-Block `crates/contextra-py/src/bindings/search_result.rs:33`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-py/src/bindings/search_result.rs:25` |
| `score` | `f32` | `crates/contextra-py/src/bindings/search_result.rs:27` |
| `metadata` | `Option<PyObject>` | `crates/contextra-py/src/bindings/search_result.rs:29` |

### Modul `bindings::storage_stats`

#### `struct PyStorageStats`
- Ort: `crates/contextra-py/src/bindings/storage_stats.rs:25`
```rust
pub struct PyStorageStats
```
Dokumentation: Statistiken der persistenten LSM-Speicherengine (`StorageStats`). # Felder - `num_segments`: Anzahl der auf der Festplatte persistierten SSTable-Segmente. - `total_size_bytes`: Gesamter belegter Festplattenplatz aller SSTable-Dateien in Bytes. - `memtable_size_bytes`: Aktuelle Belegung der In-Memory-MemTables im Arbeitsspeicher in Bytes.
- Impl-Block `crates/contextra-py/src/bindings/storage_stats.rs:35`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `num_segments` | `usize` | `crates/contextra-py/src/bindings/storage_stats.rs:27` |
| `total_size_bytes` | `u64` | `crates/contextra-py/src/bindings/storage_stats.rs:29` |
| `memtable_size_bytes` | `u64` | `crates/contextra-py/src/bindings/storage_stats.rs:31` |

### Modul `bindings::vector_index_stats`

#### `struct PyVectorIndexStats`
- Ort: `crates/contextra-py/src/bindings/vector_index_stats.rs:25`
```rust
pub struct PyVectorIndexStats
```
Dokumentation: Statistiken über den HNSW-Vektorindex (`VectorIndexStats`). # Felder - `num_vectors`: Anzahl der aktuell aktiven (nicht als gelöscht markierten) Vektoren im Index. - `memory_usage_bytes`: Geschätzter Arbeitsspeicherverbrauch des Vektorgraphen in Bytes. - `num_layers`: Anzahl der Schichten (Layer) im hierarchischen HNSW-Graph.
- Impl-Block `crates/contextra-py/src/bindings/vector_index_stats.rs:35`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `num_vectors` | `usize` | `crates/contextra-py/src/bindings/vector_index_stats.rs:27` |
| `memory_usage_bytes` | `usize` | `crates/contextra-py/src/bindings/vector_index_stats.rs:29` |
| `num_layers` | `usize` | `crates/contextra-py/src/bindings/vector_index_stats.rs:31` |

### Modul `crate`

#### `mod bindings`
- Ort: `crates/contextra-py/src/lib.rs:46`
```rust
pub mod bindings
```
- Gate: `#[cfg(feature = "pyo3")]`

#### `mod kv_links`
- Ort: `crates/contextra-py/src/lib.rs:48`
```rust
pub mod kv_links
```
- Gate: `#[cfg(feature = "pyo3")]`

#### `const VERSION`
- Ort: `crates/contextra-py/src/lib.rs:141`
```rust
pub const VERSION: &str = ""
```
Dokumentation: Fallback / Library-Information wenn `contextra-py` ohne `pyo3`-Feature gebaut wird.

### Modul `kv_links`

#### `fn parse_link_relation`
- Ort: `crates/contextra-py/src/kv_links.rs:45`
```rust
pub fn parse_link_relation(s: &str) -> PyResult<LinkRelation>
```
Dokumentation: Parst einen String ohne Allokation und unabhängig von Groß-/Kleinschreibung in eine `LinkRelation`. # Zweck Konvertiert nutzerdefinierte Relationsbezeichnungen aus Python in das typsichere Rust-Enum `LinkRelation` aus `contextra_types`. # Unterstützte Relationen: - `"elaborates"`: Vertieft, erklärt oder ergänzt den Zielknoten. - `"contradicts"`: Widerspricht der Aussage des Zielknotens. - `"supersedes"`: Löst den Zielknoten ab (neuere Version / Korrektur). - `"references"`: Zitiert oder verweist auf den Zielknoten. # Fehlerfälle Gibt einen `PyValueError` zurück, falls der übergebene String kei
