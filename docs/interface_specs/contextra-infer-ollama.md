# Schnittstellenspezifikation: `contextra-infer-ollama`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-infer-ollama` |
| Version | workspace |
| Pfad | `crates/contextra-infer-ollama` |
| Rolle | Inferenz Ollama |
| Bibliothekspfad | src/lib.rs (Standard) |

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `contextra-ports` | dev | path | nein | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-types` | dev | path | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `criterion` | dev | workspace | nein |  |
| `futures` | normal | registry | ja |  |
| `futures-util` | normal | registry | ja |  |
| `parking_lot` | normal | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `quick-xml` | dev | registry | nein |  |
| `regex` | normal | registry | nein |  |
| `reqwest` | normal | registry | ja |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |
| `tracing-test` | dev | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-mcp`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `client` | `dep:reqwest`, `dep:tokio`, `dep:futures`, `dep:futures-util` |
| `default` | – |
| `http` | `client` |
| `ports-compat` | `workspace` |
| `types-compat` | `workspace` |
| `workspace` | `dep:contextra-types`, `dep:contextra-ports` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 25 |
| Quellzeilen | 6782 |
| Öffentliche Items (Modul-Ebene) | 80 |
| davon `fn` | 23 |
| davon `struct` | 18 |
| davon `enum` | 3 |
| davon `trait` | 8 |
| davon `type` | 3 |
| davon `const` | 6 |
| davon `mod` | 19 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 16 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 69 |
| `.expect(` (Produktionscode) | 5 |
| `panic!` (Produktionscode) | 5 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 16 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 63 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 141 / 141 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `client::circuit_breaker` | `crates/contextra-infer-ollama/src/client/circuit_breaker.rs` | 193 |
| `client::config` | `crates/contextra-infer-ollama/src/client/config.rs` | 64 |
| `client::core` | `crates/contextra-infer-ollama/src/client/core.rs` | 1251 |
| `client::errors` | `crates/contextra-infer-ollama/src/client/errors.rs` | 38 |
| `client::policy` | `crates/contextra-infer-ollama/src/client/policy.rs` | 409 |
| `client::validation` | `crates/contextra-infer-ollama/src/client/validation.rs` | 62 |
| `client::wire` | `crates/contextra-infer-ollama/src/client/wire.rs` | 62 |
| `client` | `crates/contextra-infer-ollama/src/client.rs` | 47 |
| `compat::client` | `crates/contextra-infer-ollama/src/compat/client.rs` | 19 |
| `compat::client_tests` | `crates/contextra-infer-ollama/src/compat/client_tests.rs` | 1739 |
| `compat::embedding` | `crates/contextra-infer-ollama/src/compat/embedding.rs` | 290 |
| `compat::error` | `crates/contextra-infer-ollama/src/compat/error.rs` | 184 |
| `compat` | `crates/contextra-infer-ollama/src/compat/mod.rs` | 17 |
| `compat::query_rewriter` | `crates/contextra-infer-ollama/src/compat/query_rewriter.rs` | 56 |
| `compat::trait_impls` | `crates/contextra-infer-ollama/src/compat/trait_impls.rs` | 66 |
| `compat::types` | `crates/contextra-infer-ollama/src/compat/types.rs` | 48 |
| `context_prefixer` | `crates/contextra-infer-ollama/src/context_prefixer.rs` | 461 |
| `embedding` | `crates/contextra-infer-ollama/src/embedding.rs` | 155 |
| `error` | `crates/contextra-infer-ollama/src/error.rs` | 189 |
| `importance` | `crates/contextra-infer-ollama/src/importance.rs` | 685 |
| `crate` | `crates/contextra-infer-ollama/src/lib.rs` | 43 |
| `model_info` | `crates/contextra-infer-ollama/src/model_info.rs` | 339 |
| `query_rewriter` | `crates/contextra-infer-ollama/src/query_rewriter.rs` | 302 |
| `traits` | `crates/contextra-infer-ollama/src/traits.rs` | 44 |
| `xml_escape` | `crates/contextra-infer-ollama/src/xml_escape.rs` | 19 |

## 6. Re-Exports (`pub use`)

- `pub use circuit_breaker::CircuitBreaker` — `crates/contextra-infer-ollama/src/client.rs:35` (Modul `client`)
- `pub use config::*` — `crates/contextra-infer-ollama/src/client.rs:36` (Modul `client`)
- `pub use core::` — `crates/contextra-infer-ollama/src/client.rs:40` (Modul `client`)
- `pub use errors::*` — `crates/contextra-infer-ollama/src/client.rs:42` (Modul `client`)
- `pub use policy::` — `crates/contextra-infer-ollama/src/client.rs:44` (Modul `client`)
- `pub use validation::*` — `crates/contextra-infer-ollama/src/client.rs:47` (Modul `client`)
- `pub use client::` — `crates/contextra-infer-ollama/src/lib.rs:20` (Modul `crate`)
- `pub use client::` — `crates/contextra-infer-ollama/src/lib.rs:22` (Modul `crate`)
- `pub use context_prefixer::` — `crates/contextra-infer-ollama/src/lib.rs:28` (Modul `crate`)
- `pub use embedding::OllamaEmbedder` — `crates/contextra-infer-ollama/src/lib.rs:30` (Modul `crate`)
- `pub use error::` — `crates/contextra-infer-ollama/src/lib.rs:31` (Modul `crate`)
- `pub use importance::` — `crates/contextra-infer-ollama/src/lib.rs:35` (Modul `crate`)
- `pub use importance::` — `crates/contextra-infer-ollama/src/lib.rs:39` (Modul `crate`)
- `pub use model_info::` — `crates/contextra-infer-ollama/src/lib.rs:40` (Modul `crate`)
- `pub use query_rewriter::` — `crates/contextra-infer-ollama/src/lib.rs:42` (Modul `crate`)
- `pub use traits::` — `crates/contextra-infer-ollama/src/lib.rs:43` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `client`

#### `mod config`
- Ort: `crates/contextra-infer-ollama/src/client.rs:10`
```rust
pub mod config
```

#### `mod policy`
- Ort: `crates/contextra-infer-ollama/src/client.rs:16`
```rust
pub mod policy
```
- Gate: `#[cfg(feature = "client")]`

#### `mod validation`
- Ort: `crates/contextra-infer-ollama/src/client.rs:17`
```rust
pub mod validation
```

### Modul `client::circuit_breaker`

#### `struct CircuitBreaker`
- Ort: `crates/contextra-infer-ollama/src/client/circuit_breaker.rs:29`
```rust
pub struct CircuitBreaker
```
Dokumentation: Thread-safe circuit breaker backed by the `Clock` port.
- Impl-Block `crates/contextra-infer-ollama/src/client/circuit_breaker.rs:35`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self` | `crates/contextra-infer-ollama/src/client/circuit_breaker.rs:37` | Erstellt eine neue Instanz des CircuitBreakers mit konfigurierbarem Schwellwert und Reset-Timeout. |
| `pub fn status(&self, clock: &dyn LocalClock) -> BreakerStatus` | `crates/contextra-infer-ollama/src/client/circuit_breaker.rs:50` | Liefert den aktuellen Status des Circuit-Breakers unter Auswertung der Zeit. |
| `pub fn check(&self, clock: &dyn LocalClock) -> Result<(), String>` | `crates/contextra-infer-ollama/src/client/circuit_breaker.rs:58` | Checks if the circuit breaker allows a request. Returns `Ok(())` if closed/half-open, or `Err(msg)` if open. |
| `pub fn record_success(&self)` | `crates/contextra-infer-ollama/src/client/circuit_breaker.rs:82` | Records a successful request, resetting consecutive failure counts and transitioning to Closed. |
| `pub fn record_failure(&self, clock: &dyn LocalClock)` | `crates/contextra-infer-ollama/src/client/circuit_breaker.rs:90` | Records a failed request (transient/network/5xx). |

#### `enum BreakerStatus`
- Ort: `crates/contextra-infer-ollama/src/client/circuit_breaker.rs:10`
```rust
pub enum BreakerStatus
```
Dokumentation: Expliziter Zustand des Circuit-Breakers.

Varianten:

- `Closed` — `crates/contextra-infer-ollama/src/client/circuit_breaker.rs:12`
- `Open` — `crates/contextra-infer-ollama/src/client/circuit_breaker.rs:14`
- `HalfOpen` — `crates/contextra-infer-ollama/src/client/circuit_breaker.rs:16`

### Modul `client::config`

#### `struct EndpointPolicy`
- Ort: `crates/contextra-infer-ollama/src/client/config.rs:5`
```rust
pub struct EndpointPolicy
```
- Impl-Block `crates/contextra-infer-ollama/src/client/policy.rs:58`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `allow_remote` | `bool` | `crates/contextra-infer-ollama/src/client/config.rs:6` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn loopback_only() -> Self` | `crates/contextra-infer-ollama/src/client/policy.rs:60` | Creates a default loopback-only policy. |
| `pub fn check_endpoint(&self, endpoint: &str, audit_sink: &dyn EgressAuditSink) -> Result<()>` | `crates/contextra-infer-ollama/src/client/policy.rs:70` | Checks whether the given endpoint URL is permitted under this policy. Loopback endpoints (`127.0.0.0/8`, `::1`, `localhost`) and permitted Unix sockets are allo |

#### `struct OllamaConfig`
- Ort: `crates/contextra-infer-ollama/src/client/config.rs:26`
```rust
pub struct OllamaConfig
```
Dokumentation: Configuration options for the Ollama HTTP client.
- Impl-Block (Trait `Default`) `crates/contextra-infer-ollama/src/client/config.rs:49`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `base_url` | `String` | `crates/contextra-infer-ollama/src/client/config.rs:28` |
| `model` | `String` | `crates/contextra-infer-ollama/src/client/config.rs:30` |
| `request_timeout` | `Duration` | `crates/contextra-infer-ollama/src/client/config.rs:32` |
| `connect_timeout` | `Duration` | `crates/contextra-infer-ollama/src/client/config.rs:34` |
| `read_timeout` | `Duration` | `crates/contextra-infer-ollama/src/client/config.rs:36` |
| `max_response_bytes` | `usize` | `crates/contextra-infer-ollama/src/client/config.rs:38` |
| `max_retries` | `u32` | `crates/contextra-infer-ollama/src/client/config.rs:40` |
| `policy` | `EndpointPolicy` | `crates/contextra-infer-ollama/src/client/config.rs:42` |
| `failure_threshold` | `u32` | `crates/contextra-infer-ollama/src/client/config.rs:44` |
| `circuit_reset_timeout` | `Duration` | `crates/contextra-infer-ollama/src/client/config.rs:46` |

#### `const DEFAULT_BASE_URL`
- Ort: `crates/contextra-infer-ollama/src/client/config.rs:12`
```rust
pub const DEFAULT_BASE_URL: &str = ""
```
Dokumentation: Standard Ollama base URL

#### `const DEFAULT_EMBED_MODEL`
- Ort: `crates/contextra-infer-ollama/src/client/config.rs:14`
```rust
pub const DEFAULT_EMBED_MODEL: &str = ""
```
Dokumentation: Standard embedding model for SME usage (multilingual support for German)

#### `const DEFAULT_MAX_RESPONSE_BYTES`
- Ort: `crates/contextra-infer-ollama/src/client/config.rs:22`
```rust
pub const DEFAULT_MAX_RESPONSE_BYTES: usize = 10_000_000
```
Dokumentation: Default maximum response size limit in bytes (10 MB)

#### `const MAX_BATCH_SIZE`
- Ort: `crates/contextra-infer-ollama/src/client/config.rs:18`
```rust
pub const MAX_BATCH_SIZE: usize = 512
```
Dokumentation: Maximum allowed batch size for batch operations to prevent request timeouts and memory exhaustion.

#### `const MAX_RETRIES`
- Ort: `crates/contextra-infer-ollama/src/client/config.rs:16`
```rust
pub const MAX_RETRIES: u32 = 3
```
Dokumentation: Default maximum retry attempts for transient errors

#### `const MAX_TEXT_BYTES`
- Ort: `crates/contextra-infer-ollama/src/client/config.rs:20`
```rust
pub const MAX_TEXT_BYTES: usize = 10_000_000
```
Dokumentation: Maximum allowed text length in bytes (10 MB) to prevent OOM or DoS

### Modul `client::core`

#### `struct OllamaClient`
- Ort: `crates/contextra-infer-ollama/src/client/core.rs:63`
```rust
pub struct OllamaClient
```
Dokumentation: HTTP client for interacting with a local or configured Ollama instance.
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-infer-ollama/src/client/core.rs:74`
- Impl-Block `crates/contextra-infer-ollama/src/client/core.rs:89`
- Impl-Block `crates/contextra-infer-ollama/src/compat/client.rs:7`
- Impl-Block (Trait `contextra_ports::LlmTextGenerator`) `crates/contextra-infer-ollama/src/compat/trait_impls.rs:3`
- Impl-Block (Trait `contextra_ports::LlmTextGeneratorStreaming`) `crates/contextra-infer-ollama/src/compat/trait_impls.rs:16`
- Impl-Block (Trait `contextra_ports::SegmentSynthesizer`) `crates/contextra-infer-ollama/src/compat/trait_impls.rs:45`
- Impl-Block `crates/contextra-infer-ollama/src/model_info.rs:24`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `config` | `OllamaConfig` | `crates/contextra-infer-ollama/src/client/core.rs:64` |
| `client` | `reqwest::Client` | `crates/contextra-infer-ollama/src/client/core.rs:65` |
| `client_build_error` | `Option<String>` | `crates/contextra-infer-ollama/src/client/core.rs:66` |
| `rng` | `Arc<dyn LocalRng>` | `crates/contextra-infer-ollama/src/client/core.rs:67` |
| `clock` | `Arc<dyn LocalClock>` | `crates/contextra-infer-ollama/src/client/core.rs:68` |
| `audit_sink` | `Arc<dyn EgressAuditSink>` | `crates/contextra-infer-ollama/src/client/core.rs:69` |
| `payload_gate` | `Arc<dyn PayloadGate>` | `crates/contextra-infer-ollama/src/client/core.rs:70` |
| `circuit_breaker` | `Arc<CircuitBreaker>` | `crates/contextra-infer-ollama/src/client/core.rs:71` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(base_url: impl Into<String>) -> Self` | `crates/contextra-infer-ollama/src/client/core.rs:94` | Creates a new `OllamaClient` with the specified base URL and default timeout config. Note: Intended for local development and testing. For production remote egr |
| `pub fn with_config(config: OllamaConfig) -> Self` | `crates/contextra-infer-ollama/src/client/core.rs:106` | Creates a new `OllamaClient` with custom configuration parameters. Note: Intended for local development and testing. If building the underlying HTTP client fail |
| `pub fn try_with_config( config: OllamaConfig, audit_sink: Arc<dyn EgressAuditSink>, ) -> Result<Self>` | `crates/contextra-infer-ollama/src/client/core.rs:151` | Creates a fallible `OllamaClient` with custom configuration parameters and a mandatory audit sink. Returns an error immediately if building the HTTP client fail |
| `pub fn with_rng(mut self, rng: Arc<dyn LocalRng>) -> Self` | `crates/contextra-infer-ollama/src/client/core.rs:189` | Sets a custom random number generator for retry jitter computation. |
| `pub fn with_clock(mut self, clock: Arc<dyn LocalClock>) -> Self` | `crates/contextra-infer-ollama/src/client/core.rs:195` | Sets a custom clock port. |
| `pub fn with_audit_sink(mut self, audit_sink: Arc<dyn EgressAuditSink>) -> Self` | `crates/contextra-infer-ollama/src/client/core.rs:201` | Sets a custom egress audit sink. |
| `pub fn with_payload_gate(mut self, payload_gate: Arc<dyn PayloadGate>) -> Self` | `crates/contextra-infer-ollama/src/client/core.rs:207` | Sets a custom payload gate. |
| `pub async fn is_available(&self) -> bool` | `crates/contextra-infer-ollama/src/client/core.rs:229` | Health check verifying Ollama availability via GET /api/tags |
| `pub async fn check_availability(&self) -> Result<()>` | `crates/contextra-infer-ollama/src/client/core.rs:234` | Health check verifying Ollama availability via GET /api/tags |
| `pub fn with_defaults() -> Self` | `crates/contextra-infer-ollama/src/client/core.rs:278` |  |
| `pub fn base_url(&self) -> &str` | `crates/contextra-infer-ollama/src/client/core.rs:282` |  |
| `pub fn config(&self) -> &OllamaConfig` | `crates/contextra-infer-ollama/src/client/core.rs:286` |  |
| `pub async fn embed_batch(&self, model: &str, texts: &[&str]) -> Result<Vec<Vec<f32>>>` | `crates/contextra-infer-ollama/src/client/core.rs:291` | Generates vector embeddings for multiple texts. Idempotent retry operation. |
| `pub async fn try_embed_batch(&self, model: &str, texts: &[&str]) -> Result<Vec<Vec<f32>>>` | `crates/contextra-infer-ollama/src/client/core.rs:336` |  |
| `pub async fn list_models(&self) -> Result<Vec<String>>` | `crates/contextra-infer-ollama/src/client/core.rs:400` | List available models in Ollama via GET /api/tags |
| `pub async fn is_model_available(&self, model: &str) -> bool` | `crates/contextra-infer-ollama/src/client/core.rs:442` | Verifies if a specific model is available in the Ollama instance. |
| `pub async fn generate_text(&self, model: &str, prompt: &str) -> Result<String>` | `crates/contextra-infer-ollama/src/client/core.rs:460` | Idempotent text generation with retries for transient failures. |
| `pub async fn generate_text_stream<F, Fut>( &self, model: &str, prompt: &str, mut on_token: F, ) -> Result<String> where F: FnMut(String) -> Fut + Send, Fut: std::future::Future<Output = bool> + Send,` | `crates/contextra-infer-ollama/src/client/core.rs:497` | Streams text completion token by token via POST /api/chat. Non-idempotent (no automatic retries). |
| `pub async fn try_generate_text(&self, model: &str, prompt: &str) -> Result<String>` | `crates/contextra-infer-ollama/src/client/core.rs:672` | Single generate_text attempt via POST /api/chat. |
| `pub async fn generate(&self, model: &str, prompt: &str) -> Result<String>` | `crates/contextra-infer-ollama/src/client/core.rs:746` | Generates non-streaming text completion via POST /api/generate with retry logic. |
| `pub async fn try_generate(&self, model: &str, prompt: &str) -> Result<String>` | `crates/contextra-infer-ollama/src/client/core.rs:783` | Single generate attempt via POST /api/generate (no retry). |
| `pub async fn ensure_model_available(&self, model: &str) -> Result<()>` | `crates/contextra-infer-ollama/src/client/core.rs:842` | Ensures that the specified model exists; returns `OllamaError::ModelNotFound` with helpful instruction if missing. |
| `pub async fn embed(&self, model: &str, text: &str) -> Result<Vec<f32>>` | `crates/contextra-infer-ollama/src/client/core.rs:853` | Generates vector embedding with retry logic for transient failures. |
| `pub async fn try_embed(&self, model: &str, text: &str) -> Result<Vec<f32>>` | `crates/contextra-infer-ollama/src/client/core.rs:892` | Single embed attempt via POST /api/embeddings (no retry). |
| `pub async fn chat_with_rag_streaming( &self, model: &str, user_query: &str, context: &str, mut on_token: impl FnMut(String) + Send, ) -> Result<String>` | `crates/contextra-infer-ollama/src/client/core.rs:951` | Streams RAG chat response token by token via POST /api/chat |
| `pub fn with_ports_rng(mut self, rng: Arc<dyn contextra_ports::Rng>) -> Self` | `crates/contextra-infer-ollama/src/compat/client.rs:9` | Sets a custom random number generator from contextra_ports. |
| `pub async fn validate_model_available(&self, model: &str) -> Result<()>` | `crates/contextra-infer-ollama/src/model_info.rs:26` | Validates that the model exists in Ollama via GET /api/tags before invoking embeddings/chat. |
| `pub async fn show_model(&self, model: &str) -> Result<ModelInfo>` | `crates/contextra-infer-ollama/src/model_info.rs:38` | Retrieves model info via POST /api/show |

#### `fn read_response_bytes`
- Ort: `crates/contextra-infer-ollama/src/client/core.rs:20`
```rust
pub async fn read_response_bytes(response: reqwest::Response, max_bytes: usize) -> Result<Vec<u8>>
```
Dokumentation: Helper to read HTTP response bytes while strictly enforcing `max_bytes` limit.

#### `fn read_response_text`
- Ort: `crates/contextra-infer-ollama/src/client/core.rs:49`
```rust
pub async fn read_response_text(response: reqwest::Response, max_bytes: usize) -> Result<String>
```
Dokumentation: Helper to read HTTP response text while strictly enforcing `max_bytes` limit.

### Modul `client::errors`

#### `fn classify_reqwest_error`
- Ort: `crates/contextra-infer-ollama/src/client/errors.rs:9`
```rust
pub fn classify_reqwest_error(e: reqwest::Error, base_url: &str, context: &str) -> OllamaError
```
Dokumentation: Classifies a `reqwest::Error` into structured `OllamaError` variants.

#### `fn is_transient_error`
- Ort: `crates/contextra-infer-ollama/src/client/errors.rs:22`
```rust
pub fn is_transient_error(e: &OllamaError) -> bool
```
Dokumentation: Helper to classify transient network errors for retry.

#### `fn is_transient_network_error`
- Ort: `crates/contextra-infer-ollama/src/client/errors.rs:4`
```rust
pub fn is_transient_network_error(err: &reqwest::Error) -> bool
```
Dokumentation: Helper to check if a reqwest error is a transient network error (timeout or connection error).

### Modul `client::policy`

#### `struct EndpointPolicy`
- Ort: `crates/contextra-infer-ollama/src/client/policy.rs:49`
```rust
pub struct EndpointPolicy
```
Dokumentation: Endpoint policy governing allowed egress destinations for Ollama requests.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `allow_remote` | `bool` | `crates/contextra-infer-ollama/src/client/policy.rs:51` |
| `allowed_remote_hosts` | `Vec<String>` | `crates/contextra-infer-ollama/src/client/policy.rs:53` |
| `allowed_unix_sockets` | `Vec<PathBuf>` | `crates/contextra-infer-ollama/src/client/policy.rs:55` |

#### `struct NoopEgressAuditSink`
- Ort: `crates/contextra-infer-ollama/src/client/policy.rs:37`
```rust
pub struct NoopEgressAuditSink
```
Dokumentation: Default no-op implementation of `EgressAuditSink`.
- Impl-Block (Trait `EgressAuditSink`) `crates/contextra-infer-ollama/src/client/policy.rs:39`

#### `struct PermissivePayloadGate`
- Ort: `crates/contextra-infer-ollama/src/client/policy.rs:27`
```rust
pub struct PermissivePayloadGate
```
Dokumentation: Standard-Implementierung von `PayloadGate`, die alle Nutzlasten passieren lässt.
- Impl-Block (Trait `PayloadGate`) `crates/contextra-infer-ollama/src/client/policy.rs:29`

#### `trait EgressAuditSink`
- Ort: `crates/contextra-infer-ollama/src/client/policy.rs:9`
```rust
pub trait EgressAuditSink: Send + Sync + std::fmt::Debug
```
Dokumentation: Trait for auditing remote egress access attempts.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn audit_egress(&self, endpoint: &str, allowed: bool, reason: &str)` | `crates/contextra-infer-ollama/src/client/policy.rs:11` | Called when an egress check is performed for a remote (non-loopback) endpoint. |
| `fn is_noop(&self) -> bool` | `crates/contextra-infer-ollama/src/client/policy.rs:14` | Returns `true` if this audit sink is a no-op sink (default). |

#### `trait PayloadGate`
- Ort: `crates/contextra-infer-ollama/src/client/policy.rs:20`
```rust
pub trait PayloadGate: Send + Sync + std::fmt::Debug
```
Dokumentation: Trait zur Vorab-Prüfung von Nutzlasten vor dem Versand (z.B. Privacy/Egress-Filter).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn check_payload(&self, payload: &str) -> Result<()>` | `crates/contextra-infer-ollama/src/client/policy.rs:22` | Prüft den Payload-Inhalt vor dem Senden. Gibt `Ok(())` zurück oder bricht fail-closed ab. |

### Modul `client::validation`

#### `fn build_rag_prompt`
- Ort: `crates/contextra-infer-ollama/src/client/validation.rs:19`
```rust
pub fn build_rag_prompt(system_context: &str, rag_context: &str, user_query: &str) -> String
```
Dokumentation: Halluzinationsprävention bei lokalen 7B-Modellen ohne Guard: Lokale Sprachmodelle neigen ohne explizite Verhaltensanweisungen dazu, bei fehlendem Kontext Fakten frei zu erfinden (Halluzination). Der Instruktionsblock wird VOR dem `<context>`-Block platziert, damit das Modell die Regeln zur Kontexttreue, Fallback-Antwort und Quellenangabe vor der Verarbeitung der Dokumentdaten liest. Constructs a structurally isolated prompt encapsulating system instructions, RAG instructions, RAG context, and user query.

#### `fn validate_batch_size`
- Ort: `crates/contextra-infer-ollama/src/client/validation.rs:40`
```rust
pub fn validate_batch_size(count: usize) -> Result<()>
```
Dokumentation: Validates that batch size does not exceed `MAX_BATCH_SIZE`.

#### `fn validate_model_name`
- Ort: `crates/contextra-infer-ollama/src/client/validation.rs:50`
```rust
pub fn validate_model_name(name: &str) -> Result<()>
```
Dokumentation: Validates that model name does not contain invalid path traversal or whitespace control characters.

#### `fn validate_text_length`
- Ort: `crates/contextra-infer-ollama/src/client/validation.rs:29`
```rust
pub fn validate_text_length(text: &str, field_name: &str) -> Result<()>
```
Dokumentation: Validates that input text length does not exceed `MAX_TEXT_BYTES`.

#### `fn xml_escape`
- Ort: `crates/contextra-infer-ollama/src/client/validation.rs:5`
```rust
pub fn xml_escape(input: &str) -> String
```
Dokumentation: Escapes XML special characters in string inputs to prevent tag injection.

### Modul `compat`

#### `mod client`
- Ort: `crates/contextra-infer-ollama/src/compat/mod.rs:6`
```rust
pub mod client
```
- Gate: `#[cfg(feature = "client")]`

#### `mod embedding`
- Ort: `crates/contextra-infer-ollama/src/compat/mod.rs:8`
```rust
pub mod embedding
```
- Gate: `#[cfg(feature = "client")]`

#### `mod error`
- Ort: `crates/contextra-infer-ollama/src/compat/mod.rs:9`
```rust
pub mod error
```

#### `mod query_rewriter`
- Ort: `crates/contextra-infer-ollama/src/compat/mod.rs:11`
```rust
pub mod query_rewriter
```
- Gate: `#[cfg(feature = "client")]`

#### `mod trait_impls`
- Ort: `crates/contextra-infer-ollama/src/compat/mod.rs:13`
```rust
pub mod trait_impls
```
- Gate: `#[cfg(feature = "client")]`

#### `mod types`
- Ort: `crates/contextra-infer-ollama/src/compat/mod.rs:14`
```rust
pub mod types
```

### Modul `context_prefixer`

#### `struct ContextPrefixConfig`
- Ort: `crates/contextra-infer-ollama/src/context_prefixer.rs:25`
```rust
pub struct ContextPrefixConfig
```
Dokumentation: Konfiguration für Kontext-Präfix-Generierung.
- Impl-Block (Trait `Default`) `crates/contextra-infer-ollama/src/context_prefixer.rs:38`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `model` | `String` | `crates/contextra-infer-ollama/src/context_prefixer.rs:28` |
| `max_document_chars` | `usize` | `crates/contextra-infer-ollama/src/context_prefixer.rs:32` |
| `max_prefix_tokens` | `usize` | `crates/contextra-infer-ollama/src/context_prefixer.rs:35` |

#### `struct ContextPrefixEngine`
- Ort: `crates/contextra-infer-ollama/src/context_prefixer.rs:59`
```rust
pub struct ContextPrefixEngine
```
Dokumentation: Generiert LLM-basierte Kontext-Präfixe für Dokument-Chunks. # Verwendung ```ignore # async fn example() -> Result<(), contextra_infer_ollama::OllamaError> { use contextra_infer_ollama::{OllamaClient, ContextPrefixEngine, ContextPrefixConfig}; let client = OllamaClient::new("http://localhost:11434"); let engine = ContextPrefixEngine::new(client, ContextPrefixConfig::default()); let prefix = engine.generate_prefix("Volltext des Dokuments...", "Chunk-Inhalt...").await?; # Ok(()) } ```
- Impl-Block `crates/contextra-infer-ollama/src/context_prefixer.rs:68`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `config` | `ContextPrefixConfig` | `crates/contextra-infer-ollama/src/context_prefixer.rs:62` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn config(&self) -> &ContextPrefixConfig` | `crates/contextra-infer-ollama/src/context_prefixer.rs:69` |  |
| `pub fn new(client: OllamaClient, config: ContextPrefixConfig) -> Self` | `crates/contextra-infer-ollama/src/context_prefixer.rs:74` |  |
| `pub fn new(config: ContextPrefixConfig) -> Self` | `crates/contextra-infer-ollama/src/context_prefixer.rs:79` |  |
| `pub async fn generate_prefix( &self, full_document: &str, chunk_content: &str, ) -> Result<String>` | `crates/contextra-infer-ollama/src/context_prefixer.rs:92` | Generiert ein Kontext-Präfix für einen einzelnen Chunk. Der Prompt folgt dem Contextra-Contextual-Chunk-Prefixing-Schema: "Describe this chunk in context of the |
| `pub async fn generate_prefix_batch( &self, full_document: &str, chunks: &[&str], ) -> Vec<Result<String>>` | `crates/contextra-infer-ollama/src/context_prefixer.rs:140` | Generiert Präfixe für einen Batch von Chunks desselben Dokuments. Nutzt sequentielle Verarbeitung (Ollama ist single-threaded per default). Für Parallel-Verarbe |

#### `fn compute_max_prefix_chars`
- Ort: `crates/contextra-infer-ollama/src/context_prefixer.rs:157`
```rust
pub fn compute_max_prefix_chars(max_prefix_tokens: usize) -> usize
```

#### `fn truncate_chars`
- Ort: `crates/contextra-infer-ollama/src/context_prefixer.rs:161`
```rust
pub fn truncate_chars(s: &str, max_chars: usize) -> String
```

#### `fn truncate_prefix`
- Ort: `crates/contextra-infer-ollama/src/context_prefixer.rs:172`
```rust
pub fn truncate_prefix(s: &str, max_tokens: usize, max_chars: usize) -> String
```
Dokumentation: Truncates context prefix text while preserving word boundaries up to `max_tokens` (words) and hard character limit `max_chars`.

#### `type ContextPrefixer`
- Ort: `crates/contextra-infer-ollama/src/context_prefixer.rs:66`
```rust
pub type ContextPrefixer = ContextPrefixEngine
```
Dokumentation: Compatibility alias for ContextPrefixEngine.

### Modul `crate`

#### `mod client`
- Ort: `crates/contextra-infer-ollama/src/lib.rs:7`
```rust
pub mod client
```

#### `mod compat`
- Ort: `crates/contextra-infer-ollama/src/lib.rs:5`
```rust
pub mod compat
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod compat`
- Ort: `crates/contextra-infer-ollama/src/lib.rs:9`
```rust
pub mod compat
```
- Gate: `#[cfg(feature = "ports-compat")]`

#### `mod context_prefixer`
- Ort: `crates/contextra-infer-ollama/src/lib.rs:10`
```rust
pub mod context_prefixer
```

#### `mod embedding`
- Ort: `crates/contextra-infer-ollama/src/lib.rs:12`
```rust
pub mod embedding
```
- Gate: `#[cfg(feature = "client")]`

#### `mod error`
- Ort: `crates/contextra-infer-ollama/src/lib.rs:13`
```rust
pub mod error
```

#### `mod importance`
- Ort: `crates/contextra-infer-ollama/src/lib.rs:14`
```rust
pub mod importance
```

#### `mod model_info`
- Ort: `crates/contextra-infer-ollama/src/lib.rs:15`
```rust
pub mod model_info
```

#### `mod query_rewriter`
- Ort: `crates/contextra-infer-ollama/src/lib.rs:17`
```rust
pub mod query_rewriter
```
- Gate: `#[cfg(feature = "client")]`

#### `mod traits`
- Ort: `crates/contextra-infer-ollama/src/lib.rs:18`
```rust
pub mod traits
```

### Modul `embedding`

#### `struct OllamaEmbedder`
- Ort: `crates/contextra-infer-ollama/src/embedding.rs:17`
```rust
pub struct OllamaEmbedder
```
Dokumentation: Implementation of `TextEmbeddingEngine` using Ollama's HTTP API.
- Impl-Block (Trait `EmbeddingProvider`) `crates/contextra-infer-ollama/src/compat/embedding.rs:7`
- Impl-Block `crates/contextra-infer-ollama/src/embedding.rs:25`
- Impl-Block (Trait `Embedder`) `crates/contextra-infer-ollama/src/embedding.rs:122`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self` | `crates/contextra-infer-ollama/src/embedding.rs:26` |  |
| `pub fn with_config(config: OllamaConfig) -> Self` | `crates/contextra-infer-ollama/src/embedding.rs:44` |  |
| `pub fn with_defaults() -> Self` | `crates/contextra-infer-ollama/src/embedding.rs:56` |  |
| `pub fn config(&self) -> &OllamaConfig` | `crates/contextra-infer-ollama/src/embedding.rs:60` |  |
| `pub fn client(&self) -> &OllamaClient` | `crates/contextra-infer-ollama/src/embedding.rs:64` |  |
| `pub fn with_concurrency(mut self, concurrency: usize) -> Self` | `crates/contextra-infer-ollama/src/embedding.rs:68` |  |
| `pub fn with_expected_dimension(mut self, dim: usize) -> Self` | `crates/contextra-infer-ollama/src/embedding.rs:73` |  |
| `pub fn model(&self) -> &str` | `crates/contextra-infer-ollama/src/embedding.rs:78` |  |
| `pub fn embedding_dimension(&self) -> usize` | `crates/contextra-infer-ollama/src/embedding.rs:82` |  |

### Modul `error`

#### `struct LocalImportanceScore`
- Ort: `crates/contextra-infer-ollama/src/error.rs:182`
```rust
pub struct LocalImportanceScore(pub f32)
```
Dokumentation: Lokale Bewertung von Wichtigkeit (entkoppelt von `contextra-types::ImportanceScore`). # Skala Der Wert liegt im kontinuierlichen Intervall `[0.0, 1.0]`: - `0.0`: Völlig unwichtig / Rauschen / Verwerfbar. - `0.5`: Standard-Fallback bei unvollständigem oder unparsebarem LLM-Ergebnis. - `1.0`: Höchste Priorität / Essentielle Information / Dauerhaftes Behalten im Kontext.
- Impl-Block (Trait `From<contextra_types::ImportanceScore>`) `crates/contextra-infer-ollama/src/compat/error.rs:163`
- Impl-Block (Trait `From<contextra_types::ImportanceScore>`) `crates/contextra-infer-ollama/src/compat/types.rs:27`
- Impl-Block `crates/contextra-infer-ollama/src/error.rs:184`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn as_f32(&self) -> f32` | `crates/contextra-infer-ollama/src/error.rs:186` | Liefert den Rohwert als f32. |

#### `struct LocalQueryRewriteOutput`
- Ort: `crates/contextra-infer-ollama/src/error.rs:141`
```rust
pub struct LocalQueryRewriteOutput
```
Dokumentation: Lokale Datenstruktur für Re-Write-Ausgaben.
- Impl-Block (Trait `From<contextra_types::QueryRewriteOutput>`) `crates/contextra-infer-ollama/src/compat/error.rs:151`
- Impl-Block (Trait `From<contextra_types::QueryRewriteOutput>`) `crates/contextra-infer-ollama/src/compat/types.rs:15`
- Impl-Block `crates/contextra-infer-ollama/src/error.rs:150`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `text_query` | `Option<String>` | `crates/contextra-infer-ollama/src/error.rs:143` |
| `semantic_query` | `Option<String>` | `crates/contextra-infer-ollama/src/error.rs:145` |
| `anchor_entities` | `Vec<String>` | `crates/contextra-infer-ollama/src/error.rs:147` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( text_query: Option<String>, semantic_query: Option<String>, anchor_entities: Vec<String>, ) -> Self` | `crates/contextra-infer-ollama/src/error.rs:152` | Erstellt eine neue Instanz. |
| `pub fn text_only(text: impl Into<String>) -> Self` | `crates/contextra-infer-ollama/src/error.rs:165` | Erstellt eine rein textuelle Umformulierung. |

#### `struct LocalSeededRng`
- Ort: `crates/contextra-infer-ollama/src/error.rs:95`
```rust
pub struct LocalSeededRng
```
Dokumentation: Deterministischer atomarer Zufallsgenerator für Wiederholbarkeit ohne thread_rng.
- Impl-Block `crates/contextra-infer-ollama/src/error.rs:99`
- Impl-Block (Trait `Default`) `crates/contextra-infer-ollama/src/error.rs:112`
- Impl-Block (Trait `LocalRng`) `crates/contextra-infer-ollama/src/error.rs:118`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(seed: u64) -> Self` | `crates/contextra-infer-ollama/src/error.rs:101` | Erstellt einen Generator mit dem angegebenen Seed. |

#### `struct SystemClock`
- Ort: `crates/contextra-infer-ollama/src/error.rs:67`
```rust
pub struct SystemClock
```
Dokumentation: Standard-Implementierung von `LocalClock` mit monotonic process epoch baseline.
- Impl-Block (Trait `crate::error::LocalClock`) `crates/contextra-infer-ollama/src/compat/error.rs:169`
- Impl-Block (Trait `LocalClock`) `crates/contextra-infer-ollama/src/compat/types.rs:33`
- Impl-Block (Trait `LocalClock`) `crates/contextra-infer-ollama/src/error.rs:70`

#### `enum OllamaError`
- Ort: `crates/contextra-infer-ollama/src/error.rs:8`
```rust
pub enum OllamaError
```
Dokumentation: Fehlerzustände von `contextra-infer-ollama`.
- Impl-Block (Trait `From<contextra_types::ContextraError>`) `crates/contextra-infer-ollama/src/compat/error.rs:108`

Varianten:

- `InvalidInput(String)` — `crates/contextra-infer-ollama/src/error.rs:11`
- `ModelNotFound(String)` — `crates/contextra-infer-ollama/src/error.rs:15`
- `Network(String)` — `crates/contextra-infer-ollama/src/error.rs:19`
- `Timeout(String)` — `crates/contextra-infer-ollama/src/error.rs:23`
- `Server(String)` — `crates/contextra-infer-ollama/src/error.rs:27`
- `Deserialization(String)` — `crates/contextra-infer-ollama/src/error.rs:31`
- `EgressViolation(String)` — `crates/contextra-infer-ollama/src/error.rs:35`
- `CircuitBreakerOpen` — `crates/contextra-infer-ollama/src/error.rs:39`
- `Internal(String)` — `crates/contextra-infer-ollama/src/error.rs:43`

#### `trait LocalClock`
- Ort: `crates/contextra-infer-ollama/src/error.rs:50`
```rust
pub trait LocalClock: Send + Sync
```
Dokumentation: Lokaler Trait für Zeitgeber (entkoppelt von `contextra-ports::Clock`).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn now(&self) -> std::time::SystemTime` | `crates/contextra-infer-ollama/src/error.rs:52` | Liefert die aktuelle Systemzeit. |
| `fn monotonic_nanos(&self) -> u64` | `crates/contextra-infer-ollama/src/error.rs:55` | Liefert eine monotone Zeit in Nanosekunden. |

#### `trait LocalRng`
- Ort: `crates/contextra-infer-ollama/src/error.rs:83`
```rust
pub trait LocalRng: Send + Sync
```
Dokumentation: Lokaler Trait für Zufallszahlengeneratoren (entkoppelt von `contextra-ports::Rng`).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn next_u64(&self) -> u64` | `crates/contextra-infer-ollama/src/error.rs:85` | Generiert einen u64-Zufallswert. |
| `fn next_f64(&self) -> f64` | `crates/contextra-infer-ollama/src/error.rs:88` | Generiert eine f64-Fließkommazahl im Bereich [0.0, 1.0). |

#### `type Result`
- Ort: `crates/contextra-infer-ollama/src/error.rs:47`
```rust
pub type Result<T> = core::result::Result<T, OllamaError>
```
Dokumentation: Standard-Result-Typ für `contextra-infer-ollama`.

### Modul `importance`

#### `struct ImportanceAssessment`
- Ort: `crates/contextra-infer-ollama/src/importance.rs:106`
```rust
pub struct ImportanceAssessment
```
Dokumentation: Result of an LLM importance evaluation containing the score, parse confidence status, model provenance, and optional calibrated confidence.
- Impl-Block `crates/contextra-infer-ollama/src/importance.rs:120`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `score` | `LocalImportanceScore` | `crates/contextra-infer-ollama/src/importance.rs:108` |
| `confidence` | `Confidence` | `crates/contextra-infer-ollama/src/importance.rs:110` |
| `model_id` | `String` | `crates/contextra-infer-ollama/src/importance.rs:113` |
| `calibrated_confidence` | `Option<f32>` | `crates/contextra-infer-ollama/src/importance.rs:117` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(score: LocalImportanceScore, confidence: Confidence) -> Self` | `crates/contextra-infer-ollama/src/importance.rs:122` | Creates a new `ImportanceAssessment` with default empty model ID and no calibration. |
| `pub fn with_provenance( score: LocalImportanceScore, confidence: Confidence, model_id: impl Into<String>, calibrated_confidence: Option<f32>, ) -> Self` | `crates/contextra-infer-ollama/src/importance.rs:132` | Creates a new `ImportanceAssessment` with model provenance and optional calibrated confidence. |
| `pub fn value(&self) -> f32` | `crates/contextra-infer-ollama/src/importance.rs:147` | Helper returning the raw `f32` importance score value. |
| `pub fn is_parsed(&self) -> bool` | `crates/contextra-infer-ollama/src/importance.rs:152` | Returns `true` if the score was parsed successfully from the LLM output. |

#### `enum Confidence`
- Ort: `crates/contextra-infer-ollama/src/importance.rs:97`
```rust
pub enum Confidence
```
Dokumentation: Confidence or parse status of an LLM importance rating.

Varianten:

- `Parsed` — `crates/contextra-infer-ollama/src/importance.rs:99`
- `Unparseable` — `crates/contextra-infer-ollama/src/importance.rs:101`

#### `trait ImportanceCalibrator`
- Ort: `crates/contextra-infer-ollama/src/importance.rs:22`
```rust
pub trait ImportanceCalibrator: Send + Sync
```
Dokumentation: Abstract calibrator trait for post-hoc confidence calibration and outcome recording. Allows decoupling of `contextra-infer-ollama` from concrete rankers or calibration crates.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn calibrated_probability(&self, raw_score: f32) -> Option<f32>` | `crates/contextra-infer-ollama/src/importance.rs:24` | Computes calibrated probability for a raw score in [0.0, 1.0], or returns `None` if warming up. |
| `fn record_outcome(&mut self, raw_score: f32, was_useful: bool)` | `crates/contextra-infer-ollama/src/importance.rs:27` | Records outcome feedback for a raw score. |

#### `fn parse_importance_score_response`
- Ort: `crates/contextra-infer-ollama/src/importance.rs:253`
```rust
pub fn parse_importance_score_response(raw_response: &str) -> ImportanceAssessment
```
Dokumentation: Parses an `ImportanceAssessment` from raw LLM output. Returns `Confidence::Parsed` on successful float extraction, or `Confidence::Unparseable` with default score (0.5) and a warning log if unparseable.

#### `fn record_importance_outcome`
- Ort: `crates/contextra-infer-ollama/src/importance.rs:242`
```rust
pub fn record_importance_outcome( calibrator: &Arc<Mutex<dyn ImportanceCalibrator>>, raw_score: f32, was_useful: bool, )
```
Dokumentation: Records outcome feedback for an importance score into an `ImportanceCalibrator`. Intended for downstream retrieval feedback loops evaluating whether a memory chunk assigned a given `raw_score` was actually useful during agent operations.

#### `fn score_importance`
- Ort: `crates/contextra-infer-ollama/src/importance.rs:179`
```rust
pub async fn score_importance( client: &OllamaClient, chunk_text: &str, ) -> Result<ImportanceAssessment>
```
- Gate: `#[cfg(feature = "client")]`

#### `fn score_importance_batch`
- Ort: `crates/contextra-infer-ollama/src/importance.rs:40`
```rust
pub async fn score_importance_batch( client: &Arc<OllamaClient>, chunks: &[&str], calibrator: Option<&Arc<Mutex<dyn ImportanceCalibrator>>>, max_concurrent: usize, ) -> Vec<ImportanceAssessment>
```
Dokumentation: Evaluates importance for multiple text chunks in parallel using tokio::spawn with an optional dynamic calibrator. Returns a `Vec<ImportanceAssessment>` in the same order as `chunks`. Individual chunk errors return `ImportanceAssessment` with `Confidence::Unparseable` and default score 0.5 — never propagate a single chunk error to the entire batch. # Performance Spawns one tokio task per chunk up to `max_concurrent` (default: 8). Uses `futures::stream::buffer_unordered` to limit concurrency without blocking.
- Gate: `#[cfg(feature = "client")]`

#### `fn score_importance_with_calibrator`
- Ort: `crates/contextra-infer-ollama/src/importance.rs:191`
```rust
pub async fn score_importance_with_calibrator( client: &OllamaClient, chunk_text: &str, calibrator: Option<&Arc<Mutex<dyn ImportanceCalibrator>>>, ) -> Result<ImportanceAssessment>
```
Dokumentation: Evaluates importance of a text chunk with an optional shared `ImportanceCalibrator`. Populates `model_id` provenance from `client.config().model` and enriches `calibrated_confidence` when the calibrator has completed warmup.
- Gate: `#[cfg(feature = "client")]`

### Modul `model_info`

#### `struct ModelInfo`
- Ort: `crates/contextra-infer-ollama/src/model_info.rs:17`
```rust
pub struct ModelInfo
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `modelfile` | `Option<String>` | `crates/contextra-infer-ollama/src/model_info.rs:18` |
| `parameter_size` | `Option<String>` | `crates/contextra-infer-ollama/src/model_info.rs:19` |
| `quantization_level` | `Option<String>` | `crates/contextra-infer-ollama/src/model_info.rs:20` |

#### `fn known_dimension`
- Ort: `crates/contextra-infer-ollama/src/model_info.rs:107`
```rust
pub fn known_dimension(model: &str) -> Option<usize>
```
Dokumentation: Gibt die Embedding-Dimension für bekannte Modelle zurück. Gibt `None` zurück wenn das Modell unbekannt ist.

### Modul `query_rewriter`

#### `struct OllamaQueryRewriter`
- Ort: `crates/contextra-infer-ollama/src/query_rewriter.rs:47`
```rust
pub struct OllamaQueryRewriter
```
Dokumentation: LLM-basierter Multi-Signal Query Rewriter über Ollama.
- Impl-Block (Trait `QueryRewriter`) `crates/contextra-infer-ollama/src/compat/query_rewriter.rs:7`
- Impl-Block `crates/contextra-infer-ollama/src/compat/query_rewriter.rs:31`
- Impl-Block `crates/contextra-infer-ollama/src/query_rewriter.rs:53`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn rewrite_structured<'a>( &'a self, original_query: &'a str, current_results: &'a [ScoredEntry], ) -> BoxFuture<'a, contextra_types::Result<Vec<QueryRewriteOutput>>>` | `crates/contextra-infer-ollama/src/compat/query_rewriter.rs:33` | Rewrites a query structured according to contextra-ports QueryRewriter contract. |
| `pub fn new(client: OllamaClient, config: OllamaQueryRewriterConfig) -> Self` | `crates/contextra-infer-ollama/src/query_rewriter.rs:56` | Erstellt einen neuen `OllamaQueryRewriter` mit dem angegebenen Client und der Konfiguration. |
| `pub fn config(&self) -> &OllamaQueryRewriterConfig` | `crates/contextra-infer-ollama/src/query_rewriter.rs:61` | Gibt eine Referenz auf die Konfiguration zurück. |
| `pub async fn rewrite( &self, original_query: &str, context_snippets: &[&str], ) -> Result<Vec<LocalQueryRewriteOutput>>` | `crates/contextra-infer-ollama/src/query_rewriter.rs:67` | Rewrites a query using plain text context snippets. |

#### `struct OllamaQueryRewriterConfig`
- Ort: `crates/contextra-infer-ollama/src/query_rewriter.rs:24`
```rust
pub struct OllamaQueryRewriterConfig
```
Dokumentation: Konfiguration für den `OllamaQueryRewriter`.
- Impl-Block (Trait `Default`) `crates/contextra-infer-ollama/src/query_rewriter.rs:35`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `model` | `String` | `crates/contextra-infer-ollama/src/query_rewriter.rs:26` |
| `max_context_results` | `usize` | `crates/contextra-infer-ollama/src/query_rewriter.rs:28` |
| `max_snippet_chars` | `usize` | `crates/contextra-infer-ollama/src/query_rewriter.rs:30` |
| `max_query_chars` | `usize` | `crates/contextra-infer-ollama/src/query_rewriter.rs:32` |

#### `fn build_rewrite_prompt_from_snippets`
- Ort: `crates/contextra-infer-ollama/src/query_rewriter.rs:108`
```rust
pub fn build_rewrite_prompt_from_snippets( original_query: &str, snippets: &[&str], max_context_results: usize, max_snippet_chars: usize, ) -> String
```

#### `fn parse_rewrite_response`
- Ort: `crates/contextra-infer-ollama/src/query_rewriter.rs:149`
```rust
pub fn parse_rewrite_response(response: &str, original_query: &str) -> LocalQueryRewriteOutput
```

#### `fn truncate_chars`
- Ort: `crates/contextra-infer-ollama/src/query_rewriter.rs:101`
```rust
pub fn truncate_chars(s: &str, max_chars: usize) -> &str
```

### Modul `traits`

#### `trait AsyncEmbedder`
- Ort: `crates/contextra-infer-ollama/src/traits.rs:26`
```rust
pub trait AsyncEmbedder: Send + Sync
```
Dokumentation: Lokaler Trait für asynchrone Embedding-Provider.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn model_name(&self) -> &str` | `crates/contextra-infer-ollama/src/traits.rs:28` | Liefert den Modellnamen des Providers. |
| `fn embed_async<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>>>` | `crates/contextra-infer-ollama/src/traits.rs:31` | Erzeugt ein Embedding asynchron. |
| `fn embed_batch_async<'a>( &'a self, texts: &'a [&'a str], ) -> BoxFuture<'a, Result<Vec<Vec<f32>>>>` | `crates/contextra-infer-ollama/src/traits.rs:34` | Erzeugt Embeddings für einen Batch asynchron. |

#### `trait Embedder`
- Ort: `crates/contextra-infer-ollama/src/traits.rs:11`
```rust
pub trait Embedder: Send + Sync
```
Dokumentation: Lokaler Trait für Embedding-Provider.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn provider_name(&self) -> &str` | `crates/contextra-infer-ollama/src/traits.rs:13` | Liefert den Namen des Providers. |
| `fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>>>` | `crates/contextra-infer-ollama/src/traits.rs:16` | Erzeugt ein Embedding für den übergebenen Text. |
| `fn embedding_dim(&self) -> usize` | `crates/contextra-infer-ollama/src/traits.rs:19` | Liefert die Embedding-Dimension des konfigurierten Modells. |
| `fn embed_batch<'a>(&'a self, texts: &'a [&'a str]) -> BoxFuture<'a, Result<Vec<Vec<f32>>>>` | `crates/contextra-infer-ollama/src/traits.rs:22` | Erzeugt Embeddings für einen Batch von Texten. |

#### `trait TextGenerator`
- Ort: `crates/contextra-infer-ollama/src/traits.rs:41`
```rust
pub trait TextGenerator: Send + Sync
```
Dokumentation: Lokaler Trait für LLM-Textgeneratoren.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn generate<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, Result<String>>` | `crates/contextra-infer-ollama/src/traits.rs:43` | Generiert Text für den übergebenen Prompt. |

#### `type BoxFuture`
- Ort: `crates/contextra-infer-ollama/src/traits.rs:8`
```rust
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>
```
Dokumentation: Typalias für gepinnte ge-boxte Futures ohne Fremdtypen.

### Modul `xml_escape`

#### `fn xml_escape`
- Ort: `crates/contextra-infer-ollama/src/xml_escape.rs:2`
```rust
pub fn xml_escape(input: &str) -> String
```
Dokumentation: Escapes XML special characters in string inputs to prevent tag injection.

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl From<OllamaError> for contextra_types::ContextraError` — `crates/contextra-infer-ollama/src/compat/error.rs:5` (0 pub Methoden)
- `impl From<OllamaError> for contextra_ports::EmbeddingError` — `crates/contextra-infer-ollama/src/compat/error.rs:130` (0 pub Methoden)
- `impl From<LocalQueryRewriteOutput> for contextra_types::QueryRewriteOutput` — `crates/contextra-infer-ollama/src/compat/error.rs:141` (0 pub Methoden)
- `impl From<LocalImportanceScore> for contextra_types::ImportanceScore` — `crates/contextra-infer-ollama/src/compat/error.rs:157` (0 pub Methoden)
- `impl crate::error::LocalRng for contextra_ports::SeededRng` — `crates/contextra-infer-ollama/src/compat/error.rs:180` (0 pub Methoden)
- `impl From<LocalQueryRewriteOutput> for contextra_types::QueryRewriteOutput` — `crates/contextra-infer-ollama/src/compat/types.rs:5` (0 pub Methoden)
- `impl From<LocalImportanceScore> for contextra_types::ImportanceScore` — `crates/contextra-infer-ollama/src/compat/types.rs:21` (0 pub Methoden)
- `impl LocalRng for contextra_ports::SeededRng` — `crates/contextra-infer-ollama/src/compat/types.rs:44` (0 pub Methoden)
