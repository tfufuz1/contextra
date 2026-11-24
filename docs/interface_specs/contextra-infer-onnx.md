# Schnittstellenspezifikation: `contextra-infer-onnx`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-infer-onnx` |
| Version | workspace |
| Pfad | `crates/contextra-infer-onnx` |
| Rolle | Inferenz ONNX |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> `contextra-infer-onnx` — Autarke In-Process Inferenz für Embeddings und Reranking (Klasse A, Ring 2).
>
> Stellt isolierte lokale Inferenz-Funktionalität hinter Feature-Flags bereit.
> Standard-Build ist vollständig Pure-Rust ohne externe Shared-Libraries oder C-FFI.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `blake3` | normal | workspace | nein |  |
| `contextra-ports` | normal | path | ja | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-types` | normal | path | ja | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `criterion` | dev | workspace | nein |  |
| `ndarray` | normal | registry | ja |  |
| `ort` | normal | registry | ja |  |
| `parking_lot` | normal | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `reqwest` | normal | registry | ja |  |
| `serde` | normal | workspace | ja |  |
| `serde_json` | normal | workspace | ja |  |
| `sha2` | normal | registry | ja |  |
| `tempfile` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokenizers` | normal | registry | ja |  |
| `tokio` | dev | workspace | nein |  |
| `tracing` | normal | workspace | ja |  |

**Verwendet von (Crate-Ebene):** `contextra-db`, `contextra-mcp`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | – |
| `download` | `dep:reqwest` |
| `onnx` | `dep:ort`, `dep:tokenizers`, `dep:ndarray`, `dep:sha2` |
| `reranking` | `onnx` |
| `serde` | `dep:serde`, `dep:serde_json` |
| `std` | – |
| `test-utils` | – |
| `tracing` | `dep:tracing` |
| `workspace` | `dep:contextra-ports`, `dep:contextra-types` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 9 |
| Quellzeilen | 1016 |
| Öffentliche Items (Modul-Ebene) | 23 |
| davon `fn` | 2 |
| davon `struct` | 6 |
| davon `enum` | 4 |
| davon `trait` | 2 |
| davon `type` | 1 |
| davon `mod` | 8 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 6 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 0 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 20 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 36 / 36 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `compat::embedding_provider` | `crates/contextra-infer-onnx/src/compat/embedding_provider.rs` | 56 |
| `compat` | `crates/contextra-infer-onnx/src/compat/mod.rs` | 5 |
| `config` | `crates/contextra-infer-onnx/src/config.rs` | 216 |
| `embedder` | `crates/contextra-infer-onnx/src/embedder.rs` | 251 |
| `error` | `crates/contextra-infer-onnx/src/error.rs` | 62 |
| `fakes` | `crates/contextra-infer-onnx/src/fakes.rs` | 148 |
| `crate` | `crates/contextra-infer-onnx/src/lib.rs` | 34 |
| `reranker` | `crates/contextra-infer-onnx/src/reranker/mod.rs` | 200 |
| `traits` | `crates/contextra-infer-onnx/src/traits.rs` | 44 |

## 6. Re-Exports (`pub use`)

- `pub use config::` — `crates/contextra-infer-onnx/src/lib.rs:20` (Modul `crate`)
- `pub use embedder::OnnxEmbedder` — `crates/contextra-infer-onnx/src/lib.rs:23` (Modul `crate`)
- `pub use error::` — `crates/contextra-infer-onnx/src/lib.rs:24` (Modul `crate`)
- `pub use fakes::` — `crates/contextra-infer-onnx/src/lib.rs:26` (Modul `crate`)
- `pub use reranker::` — `crates/contextra-infer-onnx/src/lib.rs:27` (Modul `crate`)
- `pub use traits::` — `crates/contextra-infer-onnx/src/lib.rs:28` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `compat`

#### `mod embedding_provider`
- Ort: `crates/contextra-infer-onnx/src/compat/mod.rs:5`
```rust
pub mod embedding_provider
```

### Modul `config`

#### `struct EmbedderConfig`
- Ort: `crates/contextra-infer-onnx/src/config.rs:102`
```rust
pub struct EmbedderConfig
```
Dokumentation: Konfiguration für einen ONNX-Embedder.
- Impl-Block `crates/contextra-infer-onnx/src/config.rs:123`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `model_path` | `PathBuf` | `crates/contextra-infer-onnx/src/config.rs:104` |
| `tokenizer_path` | `PathBuf` | `crates/contextra-infer-onnx/src/config.rs:106` |
| `expected_dimension` | `usize` | `crates/contextra-infer-onnx/src/config.rs:108` |
| `max_sequence_length` | `usize` | `crates/contextra-infer-onnx/src/config.rs:110` |
| `pooling` | `PoolingMode` | `crates/contextra-infer-onnx/src/config.rs:112` |
| `normalize_l2` | `bool` | `crates/contextra-infer-onnx/src/config.rs:114` |
| `num_threads` | `usize` | `crates/contextra-infer-onnx/src/config.rs:116` |
| `model_hash` | `Option<ModelHash>` | `crates/contextra-infer-onnx/src/config.rs:118` |
| `tokenizer_hash` | `Option<ModelHash>` | `crates/contextra-infer-onnx/src/config.rs:120` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(model_path: PathBuf, tokenizer_path: PathBuf, expected_dimension: usize) -> Self` | `crates/contextra-infer-onnx/src/config.rs:126` | Erstellt eine Basiskonfiguration mit sicheren Standardwerten. |
| `pub fn validate(&self) -> Result<()>` | `crates/contextra-infer-onnx/src/config.rs:144` | Validiert die Konfiguration und prüft ggf. Dateiprüfsummen. # Errors Gibt Fehler bei ungültigen Dimensionen, Thread-Zahlen oder Hash-Abweichungen zurück. |

#### `struct RerankerConfig`
- Ort: `crates/contextra-infer-onnx/src/config.rs:168`
```rust
pub struct RerankerConfig
```
Dokumentation: Konfiguration für einen ONNX Cross-Encoder Reranker.
- Impl-Block `crates/contextra-infer-onnx/src/config.rs:183`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `model_path` | `PathBuf` | `crates/contextra-infer-onnx/src/config.rs:170` |
| `tokenizer_path` | `PathBuf` | `crates/contextra-infer-onnx/src/config.rs:172` |
| `max_length` | `usize` | `crates/contextra-infer-onnx/src/config.rs:174` |
| `num_threads` | `usize` | `crates/contextra-infer-onnx/src/config.rs:176` |
| `model_hash` | `Option<ModelHash>` | `crates/contextra-infer-onnx/src/config.rs:178` |
| `tokenizer_hash` | `Option<ModelHash>` | `crates/contextra-infer-onnx/src/config.rs:180` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(model_path: PathBuf, tokenizer_path: PathBuf) -> Self` | `crates/contextra-infer-onnx/src/config.rs:186` | Erstellt eine Basiskonfiguration für den Reranker. |
| `pub fn validate(&self) -> Result<()>` | `crates/contextra-infer-onnx/src/config.rs:201` | Validiert die Reranker-Konfiguration. # Errors Meldet Konfigurationsfehler oder Hash-Mismatches. |

#### `enum ModelHash`
- Ort: `crates/contextra-infer-onnx/src/config.rs:11`
```rust
pub enum ModelHash
```
Dokumentation: Unterstützte kryptografische Hash-Verfahren zur Modellverifikation.
- Impl-Block `crates/contextra-infer-onnx/src/config.rs:18`

Varianten:

- `Blake3(String)` — `crates/contextra-infer-onnx/src/config.rs:13`
- `Sha256(String)` — `crates/contextra-infer-onnx/src/config.rs:15`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn verify_file(&self, path: &Path) -> Result<()>` | `crates/contextra-infer-onnx/src/config.rs:24` | Überprüft den Hash einer Datei am angegebenen Pfad. # Errors Gibt `Error::HashMismatch` zurück, wenn die Prüfsumme nicht übereinstimmt, oder `Error::Io` bei Les |

#### `enum PoolingMode`
- Ort: `crates/contextra-infer-onnx/src/config.rs:92`
```rust
pub enum PoolingMode
```
Dokumentation: Pooling-Modus für Embeddings aus Transformer-Hidden-States.

Varianten:

- `Mean` — `crates/contextra-infer-onnx/src/config.rs:95`
- `Cls` — `crates/contextra-infer-onnx/src/config.rs:97`

#### `fn verify_path_safety`
- Ort: `crates/contextra-infer-onnx/src/config.rs:71`
```rust
pub fn verify_path_safety(base: &Path, user_path: &Path) -> Result<PathBuf>
```
Dokumentation: Prüft, ob ein übergebener Zielpfad sicher innerhalb eines Basisverzeichnisses liegt. Schützt vor Path-Traversal (`..`, absolute Pfade außerhalb des Root). # Errors Gibt `Error::SecurityViolation` zurück, falls der Pfad das Basisverzeichnis verlässt.

### Modul `crate`

#### `mod compat`
- Ort: `crates/contextra-infer-onnx/src/lib.rs:10`
```rust
pub mod compat
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod config`
- Ort: `crates/contextra-infer-onnx/src/lib.rs:11`
```rust
pub mod config
```

#### `mod embedder`
- Ort: `crates/contextra-infer-onnx/src/lib.rs:12`
```rust
pub mod embedder
```

#### `mod error`
- Ort: `crates/contextra-infer-onnx/src/lib.rs:13`
```rust
pub mod error
```

#### `mod fakes`
- Ort: `crates/contextra-infer-onnx/src/lib.rs:15`
```rust
pub mod fakes
```

#### `mod reranker`
- Ort: `crates/contextra-infer-onnx/src/lib.rs:16`
```rust
pub mod reranker
```

#### `mod traits`
- Ort: `crates/contextra-infer-onnx/src/lib.rs:17`
```rust
pub mod traits
```

#### `fn default_model_cache_dir`
- Ort: `crates/contextra-infer-onnx/src/lib.rs:32`
```rust
pub fn default_model_cache_dir() -> std::path::PathBuf
```
Dokumentation: Standard-Cacheverzeichnis für heruntergeladene Modelle.

### Modul `embedder`

#### `struct OnnxEmbedder`
- Ort: `crates/contextra-infer-onnx/src/embedder.rs:22`
```rust
pub struct OnnxEmbedder
```
Dokumentation: ONNX-basierter Embedding-Provider mit Session-Pool und Tokenizer.
- Impl-Block (Trait `EmbeddingProvider`) `crates/contextra-infer-onnx/src/compat/embedding_provider.rs:7`
- Impl-Block `crates/contextra-infer-onnx/src/embedder.rs:30`
- Impl-Block (Trait `Embedder`) `crates/contextra-infer-onnx/src/embedder.rs:72`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(config: EmbedderConfig) -> Result<Self>` | `crates/contextra-infer-onnx/src/embedder.rs:35` | Initialisiert einen neuen `OnnxEmbedder` anhand der übergebenen Konfiguration. # Errors Gibt `Error` zurück, wenn Validierung, Hash-Verifikation, Tokenizer oder |
| `pub fn config(&self) -> &EmbedderConfig` | `crates/contextra-infer-onnx/src/embedder.rs:67` | Gibt eine Referenz auf die Konfiguration zurück. |

### Modul `error`

#### `enum Error`
- Ort: `crates/contextra-infer-onnx/src/error.rs:11`
```rust
pub enum Error
```
Dokumentation: Primärer Fehlertyp für Inferenz-, Modelllade- und Validierungsoperationen.

Varianten:

- `ModelNotFound(String)` — `crates/contextra-infer-onnx/src/error.rs:14`
- `SecurityViolation(String)` — `crates/contextra-infer-onnx/src/error.rs:18`
- `HashMismatch{` — `crates/contextra-infer-onnx/src/error.rs:22`
- `InvalidDimension{` — `crates/contextra-infer-onnx/src/error.rs:33`
- `Tokenization(String)` — `crates/contextra-infer-onnx/src/error.rs:42`
- `Inference(String)` — `crates/contextra-infer-onnx/src/error.rs:46`
- `Io(#[from] std::io::Error)` — `crates/contextra-infer-onnx/src/error.rs:50`
- `InvalidConfig(String)` — `crates/contextra-infer-onnx/src/error.rs:54`
- `Unsupported(String)` — `crates/contextra-infer-onnx/src/error.rs:58`

#### `type Result`
- Ort: `crates/contextra-infer-onnx/src/error.rs:62`
```rust
pub type Result<T> = std::result::Result<T, Error>
```
Dokumentation: Kanonischer Result-Typ für `contextra-infer-onnx`.

### Modul `fakes`

#### `struct FakeEmbedder`
- Ort: `crates/contextra-infer-onnx/src/fakes.rs:11`
```rust
pub struct FakeEmbedder
```
Dokumentation: Deterministischer Fake-Embedder für Tests und isolierte Umgebungen.
- Impl-Block (Trait `EmbeddingProvider`) `crates/contextra-infer-onnx/src/compat/embedding_provider.rs:33`
- Impl-Block `crates/contextra-infer-onnx/src/fakes.rs:15`
- Impl-Block (Trait `Embedder`) `crates/contextra-infer-onnx/src/fakes.rs:31`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(dimension: usize) -> Result<Self>` | `crates/contextra-infer-onnx/src/fakes.rs:20` | Erstellt einen neuen `FakeEmbedder` mit der gewünschten Vektordimension. # Errors Gibt `Error::InvalidDimension` zurück, wenn dimension == 0 ist. |

#### `struct FakeReranker`
- Ort: `crates/contextra-infer-onnx/src/fakes.rs:72`
```rust
pub struct FakeReranker
```
Dokumentation: Deterministischer Fake-Reranker für Tests.
- Impl-Block `crates/contextra-infer-onnx/src/fakes.rs:74`
- Impl-Block (Trait `Reranker`) `crates/contextra-infer-onnx/src/fakes.rs:82`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-infer-onnx/src/fakes.rs:77` | Erstellt eine neue Instanz des Fake-Rerankers. |

### Modul `reranker`

#### `struct OnnxReranker`
- Ort: `crates/contextra-infer-onnx/src/reranker/mod.rs:19`
```rust
pub struct OnnxReranker
```
Dokumentation: ONNX-basierter Cross-Encoder Reranker.
- Impl-Block `crates/contextra-infer-onnx/src/reranker/mod.rs:27`
- Impl-Block (Trait `Reranker`) `crates/contextra-infer-onnx/src/reranker/mod.rs:69`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(config: RerankerConfig) -> Result<Self>` | `crates/contextra-infer-onnx/src/reranker/mod.rs:32` | Erstellt eine neue Instanz des `OnnxReranker`. # Errors Gibt `Error` zurück, falls Konfigurationsvalidierung, Hash-Prüfung oder Modellinitialisierung fehlschlag |
| `pub fn config(&self) -> &RerankerConfig` | `crates/contextra-infer-onnx/src/reranker/mod.rs:64` | Gibt eine Referenz auf die Konfiguration zurück. |

#### `enum CrossEncoderReranker`
- Ort: `crates/contextra-infer-onnx/src/reranker/mod.rs:167`
```rust
pub enum CrossEncoderReranker
```
Dokumentation: Universeller Cross-Encoder Wrapper, der entweder ONNX Runtime oder eine Fake/Passthrough-Engine nutzt.
- Impl-Block `crates/contextra-infer-onnx/src/reranker/mod.rs:175`
- Impl-Block (Trait `Reranker`) `crates/contextra-infer-onnx/src/reranker/mod.rs:192`

Varianten:

- `Onnx(OnnxReranker)` — `crates/contextra-infer-onnx/src/reranker/mod.rs:169`
- `Fake(crate::fakes::FakeReranker)` — `crates/contextra-infer-onnx/src/reranker/mod.rs:172`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn passthrough() -> Self` | `crates/contextra-infer-onnx/src/reranker/mod.rs:179` | Erstellt einen Passthrough- / Fake-Reranker ohne externe Modell-Abhängigkeiten. |
| `pub fn from_config(config: RerankerConfig) -> Result<Self>` | `crates/contextra-infer-onnx/src/reranker/mod.rs:187` | Erstellt einen ONNX-gestützten Reranker. # Errors Gibt `Error` zurück, falls die Initialisierung scheitert. |

### Modul `traits`

#### `trait Embedder`
- Ort: `crates/contextra-infer-onnx/src/traits.rs:9`
```rust
pub trait Embedder: Send + Sync
```
Dokumentation: Synchroner Trait für Vektor-Embeddings.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn embed(&self, text: &str) -> Result<Vec<f32>>` | `crates/contextra-infer-onnx/src/traits.rs:14` | Erzeugt ein Embedding für einen einzelnen Text. # Errors Gibt einen Fehler zurück, falls Tokenisierung oder Inferenz fehlschlagen. |
| `fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>` | `crates/contextra-infer-onnx/src/traits.rs:23` | Erzeugt Embeddings für einen Batch von Texten. Das Ergebnis muss bit- oder innerhalb enger numerischer Toleranz äquivalent zu sequentiellen Einzelaufrufen von ` |
| `fn dimension(&self) -> usize` | `crates/contextra-infer-onnx/src/traits.rs:32` | Gibt die feste Ausgabedimension des Modells zurück. |

#### `trait Reranker`
- Ort: `crates/contextra-infer-onnx/src/traits.rs:36`
```rust
pub trait Reranker: Send + Sync
```
Dokumentation: Synchroner Trait für Cross-Encoder Reranking.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn score(&self, query: &str, docs: &[&str]) -> Result<Vec<f32>>` | `crates/contextra-infer-onnx/src/traits.rs:43` | Berechnet Relevanz-Scores für eine Query und eine Liste von Dokumenten. Gibt einen Vektor von Scores gleicher Länge wie `docs` zurück. # Errors Gibt einen Fehle |
