# Schnittstellenspezifikation: `contextra-mcp`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-mcp` |
| Version | workspace |
| Pfad | `crates/contextra-mcp` |
| Rolle | MCP-Server |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> Layer-7-Rand-Crate ohne jegliche unsafe-Toleranz — verarbeitet direkt untrusted stdio-Input, siehe ADR-010.
>
> ## MCP Tool Category Mapping
>
> | Tool Name | ToolCategory | Description |
> |---|---|---|
> | `contextra_search` | `DatabaseRead` | Hybrid semantic search over stored documents |
> | `contextra_get` | `DatabaseRead` | Retrieve a document by ID |
> | `contextra_collections` | `DatabaseRead` | List all collections |
> | `contextra_explain` | `DatabaseRead` | Provenance breakdown & retrieval explanation |
> | `contextra_insert` | `DatabaseWrite` | Store a document with auto-chunking & auto-embedding |
> | `contextra_upsert` | `DatabaseWrite` | Insert/update document idempotently by key |

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `contextra` | normal | workspace | nein | contextra-mcp, contextra-py |
| `contextra-crypto` | normal | workspace | nein | contextra-crypto, contextra-crypto-fuzz, contextra-infer-candle, contextra-kvcache, contextra-mcp, contextra-store |
| `contextra-infer-candle` | normal | workspace | ja | contextra, contextra-mcp |
| `contextra-infer-ollama` | normal | workspace | ja | contextra-mcp |
| `contextra-infer-onnx` | normal | workspace | ja | contextra-db, contextra-mcp |
| `contextra-ports` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-privacy` | normal | workspace | nein | contextra-mcp |
| `contextra-prompt-guard` | normal | path | nein | contextra-mcp, contextra-prompt-guard-fuzz |
| `contextra-rank` | normal | workspace | nein | contextra-bench, contextra-cognition, contextra-db, contextra-engine, contextra-infer-candle, contextra-mcp |
| `contextra-router` | normal | workspace | nein | contextra-agent, contextra-mcp |
| `contextra-types` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `contextra-wire` | normal | workspace | nein | contextra-mcp, contextra-py, contextra-router, contextra-wire-fuzz |
| `futures-util` | normal | registry | nein |  |
| `parking_lot` | normal | workspace | nein |  |
| `rand` | normal | workspace | nein |  |
| `scc` | normal | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `tempfile` | dev | registry | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | normal | workspace | nein |  |
| `tokio-util` | normal | registry | nein |  |
| `tracing` | normal | workspace | nein |  |
| `tracing-subscriber` | normal | workspace | nein |  |
| `unicode-normalization` | normal | registry | nein |  |
| `zeroize` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-mcp-fuzz`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `candle` | `dep:contextra-infer-candle` |
| `default` | – |
| `ollama` | `dep:contextra-infer-ollama` |
| `onnx` | `dep:contextra-infer-onnx` |
| `test-utils` | – |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 18 |
| Quellzeilen | 4813 |
| Öffentliche Items (Modul-Ebene) | 55 |
| davon `fn` | 12 |
| davon `struct` | 15 |
| davon `enum` | 2 |
| davon `const` | 8 |
| davon `mod` | 18 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 18 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 36 |
| `.expect(` (Produktionscode) | 19 |
| `panic!` (Produktionscode) | 9 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 7 |
| `#[deprecated]` | 2 |
| `#[cfg(feature…)]`-Gates | 7 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 92 / 92 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `bin:contextra-mcp-server` | `crates/contextra-mcp/src/bin/contextra-mcp-server.rs` | 94 |
| `bulk_exfiltration_detector` | `crates/contextra-mcp/src/bulk_exfiltration_detector.rs` | 10 |
| `config` | `crates/contextra-mcp/src/config.rs` | 913 |
| `egress_gateway` | `crates/contextra-mcp/src/egress_gateway.rs` | 41 |
| `egress_guard` | `crates/contextra-mcp/src/egress_guard.rs` | 60 |
| `explain` | `crates/contextra-mcp/src/explain.rs` | 274 |
| `io` | `crates/contextra-mcp/src/io.rs` | 88 |
| `crate` | `crates/contextra-mcp/src/lib.rs` | 82 |
| `prompt_injection` | `crates/contextra-mcp/src/prompt_injection.rs` | 76 |
| `protocol` | `crates/contextra-mcp/src/protocol.rs` | 155 |
| `routing` | `crates/contextra-mcp/src/routing.rs` | 125 |
| `sandbox` | `crates/contextra-mcp/src/sandbox.rs` | 763 |
| `server` | `crates/contextra-mcp/src/server.rs` | 174 |
| `server_dispatch` | `crates/contextra-mcp/src/server_dispatch.rs` | 364 |
| `server_tools` | `crates/contextra-mcp/src/server_tools.rs` | 382 |
| `tests` | `crates/contextra-mcp/src/tests.rs` | 980 |
| `tools_crud` | `crates/contextra-mcp/src/tools_crud.rs` | 197 |
| `validation` | `crates/contextra-mcp/src/validation.rs` | 35 |

## 6. Re-Exports (`pub use`)

- `pub use contextra_privacy::bulk_exfiltration_detector::*` — `crates/contextra-mcp/src/bulk_exfiltration_detector.rs:8` (Modul `bulk_exfiltration_detector`)
- `pub use contextra_privacy::egress_gateway::*` — `crates/contextra-mcp/src/egress_gateway.rs:8` (Modul `egress_gateway`)
- `pub use contextra_privacy::egress_vault::` — `crates/contextra-mcp/src/egress_gateway.rs:9` (Modul `egress_gateway`)
- `pub use contextra_privacy::egress_guard::*` — `crates/contextra-mcp/src/egress_guard.rs:8` (Modul `egress_guard`)
- `pub use bulk_exfiltration_detector::` — `crates/contextra-mcp/src/lib.rs:51` (Modul `crate`)
- `pub use config::*` — `crates/contextra-mcp/src/lib.rs:54` (Modul `crate`)
- `pub use egress_gateway::` — `crates/contextra-mcp/src/lib.rs:56` (Modul `crate`)
- `pub use explain::` — `crates/contextra-mcp/src/lib.rs:61` (Modul `crate`)
- `pub use io::` — `crates/contextra-mcp/src/lib.rs:63` (Modul `crate`)
- `pub use prompt_injection::` — `crates/contextra-mcp/src/lib.rs:65` (Modul `crate`)
- `pub use routing::` — `crates/contextra-mcp/src/lib.rs:70` (Modul `crate`)
- `pub use server::McpServer` — `crates/contextra-mcp/src/lib.rs:72` (Modul `crate`)
- `pub use validation::` — `crates/contextra-mcp/src/lib.rs:73` (Modul `crate`)
- `pub use contextra_prompt_guard::` — `crates/contextra-mcp/src/prompt_injection.rs:54` (Modul `prompt_injection`)
- `pub use contextra_prompt_guard::audit::` — `crates/contextra-mcp/src/prompt_injection.rs:63` (Modul `prompt_injection::audit`)
- `pub use contextra_prompt_guard::guard::` — `crates/contextra-mcp/src/prompt_injection.rs:67` (Modul `prompt_injection::guard`)
- `pub use contextra_prompt_guard::policy::` — `crates/contextra-mcp/src/prompt_injection.rs:71` (Modul `prompt_injection::policy`)
- `pub use contextra_wire::` — `crates/contextra-mcp/src/protocol.rs:87` (Modul `protocol`)

## 8. Öffentliche Schnittstelle je Item

### Modul `config`

#### `struct EmbeddingConfig`
- Ort: `crates/contextra-mcp/src/config.rs:178`
```rust
pub struct EmbeddingConfig
```
Dokumentation: Embedding provider configuration settings.
- Impl-Block (Trait `Default`) `crates/contextra-mcp/src/config.rs:193`
- Impl-Block `crates/contextra-mcp/src/config.rs:206`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `provider` | `String` | `crates/contextra-mcp/src/config.rs:180` |
| `ollama_url` | `String` | `crates/contextra-mcp/src/config.rs:182` |
| `embed_model` | `String` | `crates/contextra-mcp/src/config.rs:184` |
| `onnx_model_path` | `Option<PathBuf>` | `crates/contextra-mcp/src/config.rs:186` |
| `candle_model_dir` | `Option<PathBuf>` | `crates/contextra-mcp/src/config.rs:188` |
| `expected_dim` | `Option<usize>` | `crates/contextra-mcp/src/config.rs:190` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_env() -> Self` | `crates/contextra-mcp/src/config.rs:208` | Loads configuration from environment variables with fallbacks. |
| `pub fn with_expected_dim(mut self, dim: usize) -> Self` | `crates/contextra-mcp/src/config.rs:247` |  |
| `pub fn provider_opt(&self) -> Option<&str>` | `crates/contextra-mcp/src/config.rs:252` |  |
| `pub fn build_provider(&self) -> Result<Arc<dyn EmbeddingProvider>, ContextraError>` | `crates/contextra-mcp/src/config.rs:262` | Instantiates the configured `EmbeddingProvider` as an `Arc<dyn EmbeddingProvider>` and runs startup probe. |

#### `struct LlmConfig`
- Ort: `crates/contextra-mcp/src/config.rs:348`
```rust
pub struct LlmConfig
```
Dokumentation: LLM provider configuration settings.
- Impl-Block (Trait `Default`) `crates/contextra-mcp/src/config.rs:359`
- Impl-Block `crates/contextra-mcp/src/config.rs:370`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `provider` | `String` | `crates/contextra-mcp/src/config.rs:350` |
| `ollama_url` | `String` | `crates/contextra-mcp/src/config.rs:352` |
| `llm_model` | `String` | `crates/contextra-mcp/src/config.rs:354` |
| `candle_model_dir` | `Option<PathBuf>` | `crates/contextra-mcp/src/config.rs:356` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_env() -> Self` | `crates/contextra-mcp/src/config.rs:372` | Loads configuration from environment variables with fallbacks. |
| `pub fn provider_opt(&self) -> Option<&str>` | `crates/contextra-mcp/src/config.rs:405` |  |
| `pub fn build_generator(&self) -> Result<Arc<dyn LlmTextGenerator>, ContextraError>` | `crates/contextra-mcp/src/config.rs:415` | Instantiates the configured `LlmTextGenerator` as an `Arc<dyn LlmTextGenerator>`. |

#### `struct MockEmbedder`
- Ort: `crates/contextra-mcp/src/config.rs:536`
```rust
pub struct MockEmbedder
```
Dokumentation: Fallback Mock Embedder (Test & dev utilities only).
- Impl-Block `crates/contextra-mcp/src/config.rs:542`
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-mcp/src/config.rs:559`
- Impl-Block (Trait `EmbeddingProvider`) `crates/contextra-mcp/src/config.rs:566`
- Impl-Block (Trait `contextra_ports::EmbeddingProvider`) `crates/contextra-mcp/src/explain.rs:135`
- Impl-Block (Trait `contextra_ports::EmbeddingProvider`) `crates/contextra-mcp/src/tests.rs:21`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(dimension: usize) -> Self` | `crates/contextra-mcp/src/config.rs:543` |  |
| `pub fn with_zero_vectors(dimension: usize) -> Self` | `crates/contextra-mcp/src/config.rs:550` |  |

#### `struct MockLlmGenerator`
- Ort: `crates/contextra-mcp/src/config.rs:596`
```rust
pub struct MockLlmGenerator
```
Dokumentation: Fallback Mock LLM Text Generator (Test & dev utilities only).
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-mcp/src/config.rs:599`
- Impl-Block (Trait `LlmTextGenerator`) `crates/contextra-mcp/src/config.rs:606`

#### `struct RouterConfig`
- Ort: `crates/contextra-mcp/src/config.rs:489`
```rust
pub struct RouterConfig
```
Dokumentation: Router configuration settings.
- Impl-Block `crates/contextra-mcp/src/config.rs:498`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `profiles` | `Vec<contextra_router::SlmProfile>` | `crates/contextra-mcp/src/config.rs:491` |
| `profiles_path` | `Option<PathBuf>` | `crates/contextra-mcp/src/config.rs:493` |
| `calibration_store_path` | `Option<PathBuf>` | `crates/contextra-mcp/src/config.rs:495` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_env() -> Self` | `crates/contextra-mcp/src/config.rs:500` | Loads configuration from environment variables with fallbacks. |

#### `fn available_compiled_providers`
- Ort: `crates/contextra-mcp/src/config.rs:21`
```rust
pub fn available_compiled_providers() -> Vec<&'static str>
```
Dokumentation: Returns the list of currently compiled provider identifiers.

#### `fn create_embedding_provider`
- Ort: `crates/contextra-mcp/src/config.rs:276`
```rust
pub fn create_embedding_provider( provider_type: Option<&str>, ollama_url: &str, embed_model: &str, onnx_model_path: Option<&Path>, candle_model_dir: Option<&Path>, ) -> Result<Arc<dyn EmbeddingProvider>, ContextraError>
```
Dokumentation: Dynamically constructs an `EmbeddingProvider` implementation based on provider identifier.

#### `fn create_llm_text_generator`
- Ort: `crates/contextra-mcp/src/config.rs:426`
```rust
pub fn create_llm_text_generator( provider_type: Option<&str>, ollama_url: &str, llm_model: &str, candle_model_dir: Option<&Path>, ) -> Result<Arc<dyn LlmTextGenerator>, ContextraError>
```
Dokumentation: Dynamically constructs an `LlmTextGenerator` implementation based on provider identifier.

#### `fn verify_embedding_provider`
- Ort: `crates/contextra-mcp/src/config.rs:82`
```rust
pub fn verify_embedding_provider( provider: &dyn EmbeddingProvider, expected_dim: Option<usize>, ) -> Result<(), ContextraError>
```
Dokumentation: Startup embedding self-test (probe) verifying provider functionality.

### Modul `crate`

#### `mod bulk_exfiltration_detector`
- Ort: `crates/contextra-mcp/src/lib.rs:32`
```rust
pub mod bulk_exfiltration_detector
```

#### `mod config`
- Ort: `crates/contextra-mcp/src/lib.rs:33`
```rust
pub mod config
```

#### `mod egress_gateway`
- Ort: `crates/contextra-mcp/src/lib.rs:34`
```rust
pub mod egress_gateway
```

#### `mod egress_guard`
- Ort: `crates/contextra-mcp/src/lib.rs:35`
```rust
pub mod egress_guard
```

#### `mod explain`
- Ort: `crates/contextra-mcp/src/lib.rs:36`
```rust
pub mod explain
```

#### `mod io`
- Ort: `crates/contextra-mcp/src/lib.rs:37`
```rust
pub mod io
```

#### `mod prompt_injection`
- Ort: `crates/contextra-mcp/src/lib.rs:38`
```rust
pub mod prompt_injection
```

#### `mod protocol`
- Ort: `crates/contextra-mcp/src/lib.rs:39`
```rust
pub mod protocol
```

#### `mod routing`
- Ort: `crates/contextra-mcp/src/lib.rs:40`
```rust
pub mod routing
```

#### `mod sandbox`
- Ort: `crates/contextra-mcp/src/lib.rs:41`
```rust
pub mod sandbox
```

#### `mod server`
- Ort: `crates/contextra-mcp/src/lib.rs:42`
```rust
pub mod server
```

#### `mod server_dispatch`
- Ort: `crates/contextra-mcp/src/lib.rs:43`
```rust
pub mod server_dispatch
```

#### `mod server_tools`
- Ort: `crates/contextra-mcp/src/lib.rs:44`
```rust
pub mod server_tools
```

#### `mod tools_crud`
- Ort: `crates/contextra-mcp/src/lib.rs:45`
```rust
pub mod tools_crud
```

#### `mod validation`
- Ort: `crates/contextra-mcp/src/lib.rs:46`
```rust
pub mod validation
```

### Modul `egress_gateway`

#### `struct PromptGuardDetector`
- Ort: `crates/contextra-mcp/src/egress_gateway.rs:33`
```rust
pub struct PromptGuardDetector<'a>(pub &'a PromptInjectionGuard)
```
Dokumentation: Adapter, der einen [`PromptInjectionGuard`] als [`contextra_privacy::InjectionDetector`] nutzbar macht. Die Orphan-Regel verbietet die direkte Implementierung, weil weder Trait noch Typ in diesem Crate definiert sind. Der Newtype ist lokal und löst das auf.
- Impl-Block (Trait `contextra_privacy::InjectionDetector`) `crates/contextra-mcp/src/egress_gateway.rs:35`

### Modul `egress_guard`

#### `struct CollectionSearchEngine`
- Ort: `crates/contextra-mcp/src/egress_guard.rs:15`
```rust
pub struct CollectionSearchEngine(pub Arc<Collection>)
```
Dokumentation: Adapter implementing `TextSearchEngine` for `contextra::Collection`.
- Impl-Block (Trait `TextSearchEngine`) `crates/contextra-mcp/src/egress_guard.rs:17`

### Modul `explain`

#### `struct ExplainRequest`
- Ort: `crates/contextra-mcp/src/explain.rs:14`
```rust
pub struct ExplainRequest
```
Dokumentation: Request payload for `contextra_explain`.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-mcp/src/explain.rs:16` |
| `collection` | `String` | `crates/contextra-mcp/src/explain.rs:19` |

#### `struct ExplainResponse`
- Ort: `crates/contextra-mcp/src/explain.rs:28`
```rust
pub struct ExplainResponse
```
Dokumentation: Response payload for `contextra_explain`.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `id` | `String` | `crates/contextra-mcp/src/explain.rs:30` |
| `collection` | `String` | `crates/contextra-mcp/src/explain.rs:32` |
| `provenance` | `Option<ProvenanceRecord>` | `crates/contextra-mcp/src/explain.rs:35` |
| `explanation` | `String` | `crates/contextra-mcp/src/explain.rs:37` |
| `content_provenance` | `String` | `crates/contextra-mcp/src/explain.rs:39` |

### Modul `io`

#### `fn read_line_bounded`
- Ort: `crates/contextra-mcp/src/io.rs:19`
```rust
pub async fn read_line_bounded<R: tokio::io::AsyncBufRead + Unpin>( reader: &mut R, buf: &mut String, max_bytes: usize, ) -> std::io::Result<usize>
```
Dokumentation: Reads a single line from an async reader into `buf` up to `max_bytes`. If the line exceeds `max_bytes`, consumes and discards the remainder of the line without allocating memory and returns `InvalidData`.

#### `const MAX_RPC_BYTES`
- Ort: `crates/contextra-mcp/src/io.rs:8`
```rust
pub const MAX_RPC_BYTES: usize = 4 * 1024 * 1024
```
Dokumentation: Maximum allowed single JSON-RPC message size via stdio (4 MB).

#### `const MAX_SEARCH_QUERY_BYTES`
- Ort: `crates/contextra-mcp/src/io.rs:10`
```rust
pub const MAX_SEARCH_QUERY_BYTES: usize = 64 * 1024
```
Dokumentation: Maximum allowed search query length in bytes (64 KB).

### Modul `prompt_injection`

#### `mod audit`
- Ort: `crates/contextra-mcp/src/prompt_injection.rs:62`
```rust
pub mod audit
```

#### `mod guard`
- Ort: `crates/contextra-mcp/src/prompt_injection.rs:66`
```rust
pub mod guard
```

#### `mod policy`
- Ort: `crates/contextra-mcp/src/prompt_injection.rs:70`
```rust
pub mod policy
```

### Modul `protocol`

#### `enum McpError`
- Ort: `crates/contextra-mcp/src/protocol.rs:20`
```rust
pub enum McpError
```
Dokumentation: MCP Error representation matching standard JSON-RPC 2.0 error codes.
- Impl-Block (Trait `From<EgressError>`) `crates/contextra-mcp/src/egress_gateway.rs:15`
- Impl-Block `crates/contextra-mcp/src/protocol.rs:33`
- Impl-Block (Trait `From<contextra_types::ContextraError>`) `crates/contextra-mcp/src/protocol.rs:65`
- Impl-Block (Trait `From<String>`) `crates/contextra-mcp/src/protocol.rs:75`
- Impl-Block (Trait `From<&str>`) `crates/contextra-mcp/src/protocol.rs:81`

Varianten:

- `ParseError(String)` — `crates/contextra-mcp/src/protocol.rs:22`
- `InvalidRequest(String)` — `crates/contextra-mcp/src/protocol.rs:24`
- `MethodNotFound(String)` — `crates/contextra-mcp/src/protocol.rs:26`
- `InvalidParams(String)` — `crates/contextra-mcp/src/protocol.rs:28`
- `InternalError(String)` — `crates/contextra-mcp/src/protocol.rs:30`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn parse_error(msg: impl Into<String>) -> Self` | `crates/contextra-mcp/src/protocol.rs:34` |  |
| `pub fn invalid_request(msg: impl Into<String>) -> Self` | `crates/contextra-mcp/src/protocol.rs:38` |  |
| `pub fn method_not_found(msg: impl Into<String>) -> Self` | `crates/contextra-mcp/src/protocol.rs:42` |  |
| `pub fn invalid_params(msg: impl Into<String>) -> Self` | `crates/contextra-mcp/src/protocol.rs:46` |  |
| `pub fn internal_error(msg: impl Into<String>) -> Self` | `crates/contextra-mcp/src/protocol.rs:50` |  |
| `pub fn code(&self) -> i32` | `crates/contextra-mcp/src/protocol.rs:54` |  |

#### `fn measure_json_depth`
- Ort: `crates/contextra-mcp/src/protocol.rs:90`
```rust
pub fn measure_json_depth(val: &Value) -> usize
```
Dokumentation: Ermittelt die Schachtelungstiefe eines JSON-Value.

#### `fn response_from_error`
- Ort: `crates/contextra-mcp/src/protocol.rs:125`
```rust
pub fn response_from_error(id: Option<Value>, err: McpError) -> JsonRpcResponse
```
Dokumentation: Helper function to convert an `McpError` directly into a `JsonRpcResponse`.

#### `fn validate_json_depth`
- Ort: `crates/contextra-mcp/src/protocol.rs:114`
```rust
pub fn validate_json_depth(val: &Value, max_depth: usize) -> Result<(), McpError>
```
Dokumentation: Validiert, dass ein JSON-Value die maximale Tiefe nicht überschreitet.

#### `const MAX_BATCH_SIZE`
- Ort: `crates/contextra-mcp/src/protocol.rs:16`
```rust
pub const MAX_BATCH_SIZE: usize = 100
```
Dokumentation: Maximale Anzahl von Elementen in einem JSON-RPC Batch-Request (Schutz gegen Batch-DoS).

#### `const MAX_JSON_DEPTH`
- Ort: `crates/contextra-mcp/src/protocol.rs:13`
```rust
pub const MAX_JSON_DEPTH: usize = 32
```
Dokumentation: Maximale Rekursionstiefe für eingehende JSON-RPC Payloads (Schutz gegen Stack-Overflow).

### Modul `routing`

#### `struct RoutingHandle`
- Ort: `crates/contextra-mcp/src/routing.rs:80`
```rust
pub struct RoutingHandle
```
Dokumentation: Holds strong Arc references to routing and calibration components to maintain live Weak references in `Contextra`.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `router` | `Arc<contextra_router::DefaultRouterEngine>` | `crates/contextra-mcp/src/routing.rs:81` |
| `calibrator` | `Arc<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>` | `crates/contextra-mcp/src/routing.rs:82` |
| `_drift_adapter` | `Arc<dyn contextra_ports::DriftStatusProvider>` | `crates/contextra-mcp/src/routing.rs:83` |

#### `fn setup_routing`
- Ort: `crates/contextra-mcp/src/routing.rs:86`
```rust
pub async fn setup_routing( db: &Arc<Contextra>, config: &RouterConfig, ) -> Result<Option<Arc<RoutingHandle>>, ContextraError>
```

### Modul `sandbox`

#### `struct McpSandbox`
- Ort: `crates/contextra-mcp/src/sandbox.rs:268`
```rust
pub struct McpSandbox
```
Dokumentation: MCP Sandbox: validiert Tool-Calls und verwaltet volatile Ergebnisse.
- Impl-Block `crates/contextra-mcp/src/sandbox.rs:276`
- Impl-Block (Trait `Drop`) `crates/contextra-mcp/src/sandbox.rs:433`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(policy: SandboxPolicy) -> Result<Self>` | `crates/contextra-mcp/src/sandbox.rs:278` | Erstellt eine neue Sandbox-Instanz mit frischem Sitzungsschlüssel. |
| `pub fn policy(&self) -> &SandboxPolicy` | `crates/contextra-mcp/src/sandbox.rs:295` | Gibt die aktuell konfigurierte SandboxPolicy zurück. |
| `pub fn validate_tool_call(&self, method: &str, _params: &Value) -> Result<()>` | `crates/contextra-mcp/src/sandbox.rs:309` | Validiert ob ein Tool-Call erlaubt ist. HINWEIS: Unbekannte Tool-Namen (die nicht in `TOOL_REGISTRY` gelistet sind) werden sofort per `try_classify_method` abge |
| `pub fn try_classify_method(method: &str) -> Result<ToolCategory>` | `crates/contextra-mcp/src/sandbox.rs:353` | Klassifiziert den MCP-Tool-Namen strikt anhand von `TOOL_REGISTRY`. Gibt `Err(ContextraError::InvalidInput("unknown tool"))` zurück, falls der Name nicht regist |
| `pub fn classify_method(method: &str) -> ToolCategory` | `crates/contextra-mcp/src/sandbox.rs:371` |  |
| `pub async fn execute_with_timeout<F, T, E>( &self, tool_name: &str, fut: F, ) -> std::result::Result<T, E> where F: std::future::Future<Output = std::result::Result<T, E>>, E: From<ContextraError>,` | `crates/contextra-mcp/src/sandbox.rs:376` | Führt eine Asynchrone Tool-Future mit Timeout gemäß SandboxPolicy aus. |
| `pub fn store_volatile(&self, key: &str, output: &[u8]) -> Result<()>` | `crates/contextra-mcp/src/sandbox.rs:395` | Speichert einen Tool-Output verschlüsselt in der Session. |
| `pub fn get_volatile(&self, key: &str) -> Result<Option<zeroize::Zeroizing<Vec<u8>>>>` | `crates/contextra-mcp/src/sandbox.rs:422` | Ruft einen verschlüsselten Tool-Output ab und entschlüsselt ihn. |

#### `struct SandboxPolicy`
- Ort: `crates/contextra-mcp/src/sandbox.rs:203`
```rust
pub struct SandboxPolicy
```
Dokumentation: Konfiguration der Sandbox-Policy.
- Impl-Block (Trait `Default`) `crates/contextra-mcp/src/sandbox.rs:213`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `allow_db_reads` | `bool` | `crates/contextra-mcp/src/sandbox.rs:204` |
| `allow_db_writes` | `bool` | `crates/contextra-mcp/src/sandbox.rs:205` |
| `allow_code_execution` | `bool` | `crates/contextra-mcp/src/sandbox.rs:206` |
| `allow_cloud_egress` | `bool` | `crates/contextra-mcp/src/sandbox.rs:208` |
| `max_execution_ms` | `u64` | `crates/contextra-mcp/src/sandbox.rs:210` |

#### `struct ToolDefinition`
- Ort: `crates/contextra-mcp/src/sandbox.rs:30`
```rust
pub struct ToolDefinition
```
Dokumentation: Definition eines MCP-Tools mit Name, Kategorie, Beschreibung und JSON-Schema.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `name` | `&'static str` | `crates/contextra-mcp/src/sandbox.rs:31` |
| `category` | `ToolCategory` | `crates/contextra-mcp/src/sandbox.rs:32` |
| `description` | `&'static str` | `crates/contextra-mcp/src/sandbox.rs:33` |
| `input_schema` | `fn() -> Value` | `crates/contextra-mcp/src/sandbox.rs:34` |

#### `struct VolatileToolResult`
- Ort: `crates/contextra-mcp/src/sandbox.rs:229`
```rust
pub struct VolatileToolResult
```
Dokumentation: Volatiler Tool-Output-Speicher mit Zeroize-Garantie bei Drop. Tool-Ergebnisse werden verschlüsselt im Arbeitsspeicher gehalten. Beim Drop wird der Speicher via Zeroize bereinigt.
- Impl-Block `crates/contextra-mcp/src/sandbox.rs:236`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn encrypt(plaintext: &[u8], key: &contextra_crypto::CryptoKey) -> Result<Self>` | `crates/contextra-mcp/src/sandbox.rs:238` | Speichert einen Tool-Output verschlüsselt. |
| `pub fn decrypt( &self, key: &contextra_crypto::CryptoKey, ) -> Result<zeroize::Zeroizing<Vec<u8>>>` | `crates/contextra-mcp/src/sandbox.rs:249` | Entschlüsselt und gibt den Klartext zeroized zurück. |

#### `enum ToolCategory`
- Ort: `crates/contextra-mcp/src/sandbox.rs:190`
```rust
pub enum ToolCategory
```
Dokumentation: Erlaubte MCP-Tool-Kategorien (Whitelist-Prinzip).

Varianten:

- `DatabaseRead` — `crates/contextra-mcp/src/sandbox.rs:192`
- `DatabaseWrite` — `crates/contextra-mcp/src/sandbox.rs:194`
- `CodeExecution` — `crates/contextra-mcp/src/sandbox.rs:196`
- `CloudEgress` — `crates/contextra-mcp/src/sandbox.rs:198`

#### `const MAX_VOLATILE_KEY_BYTES`
- Ort: `crates/contextra-mcp/src/sandbox.rs:24`
```rust
pub const MAX_VOLATILE_KEY_BYTES: usize = 256
```
Dokumentation: Maximale Länge eines Volatile-Result-Schlüssels in Bytes.

#### `const MAX_VOLATILE_OUTPUT_BYTES`
- Ort: `crates/contextra-mcp/src/sandbox.rs:26`
```rust
pub const MAX_VOLATILE_OUTPUT_BYTES: usize = 16 * 1024 * 1024
```
Dokumentation: Maximale Größe einer volatilen Tool-Ausgabe in Bytes (16 MB).

#### `const MAX_VOLATILE_RESULTS`
- Ort: `crates/contextra-mcp/src/sandbox.rs:22`
```rust
pub const MAX_VOLATILE_RESULTS: usize = 1_000
```
Dokumentation: Maximale Anzahl von volatilen Ergebnissen pro Sandbox-Session.

#### `const TOOL_REGISTRY`
- Ort: `crates/contextra-mcp/src/sandbox.rs:53`
```rust
pub const TOOL_REGISTRY: &[ToolDefinition] = &[ ToolDefinition
```
Dokumentation: Single Source of Truth für MCP-Tool-Registrierung, Discovery (`tools/list`) und -Klassifizierung. ## Tool-Kategorie-Tabelle | Werkzeug | Kategorie | Beschreibung | |---|---|---| | `contextra_search` | `DatabaseRead` | Hybrid semantic search over stored documents | | `contextra_get` | `DatabaseRead` | Retrieve a document by ID | | `contextra_collections` | `DatabaseRead` | List all collections | | `contextra_explain` | `DatabaseRead` | Retrieval explanation and provenance breakdown | | `contextra_insert` | `DatabaseWrite` | Store document (auto-embedding, auto-chunking) | | `contextra_upsert` |

### Modul `sandbox::hex`

#### `fn encode`
- Ort: `crates/contextra-mcp/src/sandbox.rs:456`
```rust
pub fn encode(bytes: [u8
```

### Modul `server`

#### `struct McpServer`
- Ort: `crates/contextra-mcp/src/server.rs:19`
```rust
pub struct McpServer
```
- Impl-Block `crates/contextra-mcp/src/explain.rs:42`
- Impl-Block `crates/contextra-mcp/src/server.rs:30`
- Impl-Block `crates/contextra-mcp/src/server_dispatch.rs:9`
- Impl-Block `crates/contextra-mcp/src/server_tools.rs:14`
- Impl-Block `crates/contextra-mcp/src/tools_crud.rs:12`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `db` | `Arc<Contextra>` | `crates/contextra-mcp/src/server.rs:20` |
| `embedder` | `Arc<dyn EmbeddingProvider>` | `crates/contextra-mcp/src/server.rs:21` |
| `sandbox` | `Arc<McpSandbox>` | `crates/contextra-mcp/src/server.rs:22` |
| `injection_guard` | `Arc<PromptInjectionGuard>` | `crates/contextra-mcp/src/server.rs:23` |
| `egress_classifier` | `Arc<dyn EgressClassifier>` | `crates/contextra-mcp/src/server.rs:24` |
| `routing` | `Option<Arc<RoutingHandle>>` | `crates/contextra-mcp/src/server.rs:25` |
| `plugin_registry` | `Option<Arc<contextra_ports::plugin::PluginRegistry>>` | `crates/contextra-mcp/src/server.rs:26` |
| `tenant_id` | `Option<TenantId>` | `crates/contextra-mcp/src/server.rs:27` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( db: Arc<Contextra>, embedder: Arc<dyn EmbeddingProvider>, ) -> Result<Self, ContextraError>` | `crates/contextra-mcp/src/server.rs:31` |  |
| `pub fn with_write_permission( db: Arc<Contextra>, embedder: Arc<dyn EmbeddingProvider>, allow_db_writes: bool, ) -> Result<Self, ContextraError>` | `crates/contextra-mcp/src/server.rs:38` |  |
| `pub fn with_sandbox( db: Arc<Contextra>, embedder: Arc<dyn EmbeddingProvider>, sandbox: Arc<McpSandbox>, ) -> Self` | `crates/contextra-mcp/src/server.rs:55` |  |
| `pub fn with_tenant_id(mut self, tenant_id: Option<TenantId>) -> Self` | `crates/contextra-mcp/src/server.rs:73` |  |
| `pub fn with_injection_guard(mut self, injection_guard: Arc<PromptInjectionGuard>) -> Self` | `crates/contextra-mcp/src/server.rs:78` |  |
| `pub fn with_egress_classifier(mut self, classifier: Arc<dyn EgressClassifier>) -> Self` | `crates/contextra-mcp/src/server.rs:83` |  |
| `pub fn with_routing(mut self, routing: Option<Arc<RoutingHandle>>) -> Self` | `crates/contextra-mcp/src/server.rs:88` |  |
| `pub fn with_plugin_registry( mut self, registry: Option<Arc<contextra_ports::plugin::PluginRegistry>>, ) -> Self` | `crates/contextra-mcp/src/server.rs:93` |  |
| `pub async fn run_stdio(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>>` | `crates/contextra-mcp/src/server.rs:106` | Startet den MCP stdio-Loop. Liest zeilenweise von stdin, dispatcht JSON-RPC-Requests und schreibt Antworten als einzelne JSON-Zeile nach stdout. Der Loop endet, |
| `pub async fn handle_value(&self, val: Value) -> Option<Value>` | `crates/contextra-mcp/src/server_dispatch.rs:13` | Verarbeitet einen Parsed JSON-RPC 2.0 Request (Einzelanfrage, Notification oder Batch Array). Gibt `Some(Value)` zurück, falls eine Antwort nach stdout gesendet |
| `pub async fn handle(&self, req: JsonRpcRequest) -> JsonRpcResponse` | `crates/contextra-mcp/src/server_dispatch.rs:125` |  |

### Modul `validation`

#### `fn is_write_allowed_by_env`
- Ort: `crates/contextra-mcp/src/validation.rs:8`
```rust
pub fn is_write_allowed_by_env() -> bool
```
Dokumentation: Checks whether database write access is explicitly enabled via environment variable `CONTEXTRA_MCP_ALLOW_WRITE`.

#### `fn validate_collection_name`
- Ort: `crates/contextra-mcp/src/validation.rs:20`
```rust
pub fn validate_collection_name(name: &str) -> Result<(), McpError>
```
Dokumentation: MCP-Server mit stdio-Transport (JSON-RPC 2.0). stdout ist dem Protokoll vorbehalten — Logs gehen ausschließlich nach stderr.
