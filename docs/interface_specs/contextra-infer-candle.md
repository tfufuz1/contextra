# Schnittstellenspezifikation: `contextra-infer-candle`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-infer-candle` |
| Version | workspace |
| Pfad | `crates/contextra-infer-candle` |
| Rolle | Inferenz Candle |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> `contextra-infer-candle`: Native Candle GGUF ML Inferenz-Baustein für Contextra (Klasse A).
>
> # Architektur
> - **Pure Rust Standard**: Ohne aktiviertes Feature `candle` baut dieser Crate autark
> mit lokalen Traits (`Embedder`, `TextGenerator`) und Fake-Backends.
> - **GGUF-Laden**: Robuster, defensiver GGUF-Header- und Metadaten-Parser mit SHA-256 Validierung.
> - **Optionale Hardwarebeschleunigung**: Unterstützt CUDA und Metal via Feature-Flags.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `bincode` | normal | workspace | ja |  |
| `bytes` | normal | workspace | nein |  |
| `candle-core` | normal | registry | ja |  |
| `candle-nn` | normal | registry | ja |  |
| `candle-transformers` | normal | registry | ja |  |
| `contextra-crypto` | dev | workspace | nein | contextra-crypto, contextra-crypto-fuzz, contextra-infer-candle, contextra-kvcache, contextra-mcp, contextra-store |
| `contextra-env` | normal | path | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-infer-candle, contextra-kvcache, contextra-mvcc, contextra-ports, contextra-store, contextra-text, contextra-vector |
| `contextra-ports` | dev | path | nein | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-rank` | dev | path | nein | contextra-bench, contextra-cognition, contextra-db, contextra-engine, contextra-infer-candle, contextra-mcp |
| `contextra-types` | dev | path | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `criterion` | dev | workspace | nein |  |
| `futures-util` | normal | registry | nein |  |
| `half` | normal | registry | ja |  |
| `parking_lot` | normal | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `sha2` | normal | registry | nein |  |
| `tempfile` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokenizers` | normal | registry | ja |  |
| `tokio` | normal | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |

**Verwendet von (Crate-Ebene):** `contextra`, `contextra-mcp`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `candle` | `dep:candle-core`, `dep:candle-nn`, `dep:candle-transformers`, `dep:tokenizers`, `dep:half` |
| `cuda` | `candle-core?/cuda`, `candle-nn?/cuda`, `candle-transformers?/cuda` |
| `default` | `candle` |
| `exp-gasp` | – |
| `exp-kv-stage-b` | `kv-stage-b` |
| `kv-bridge` | `workspace`, `dep:contextra-crypto`, `contextra-crypto/kv-encryption`, `dep:bincode` |
| `kv-stage-b` | `dep:bincode` |
| `ports-compat` | `workspace` |
| `test-utils` | – |
| `types-compat` | `workspace` |
| `workspace` | `dep:contextra-types`, `dep:contextra-ports`, `dep:contextra-rank` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 30 |
| Quellzeilen | 6808 |
| Öffentliche Items (Modul-Ebene) | 85 |
| davon `fn` | 6 |
| davon `struct` | 35 |
| davon `enum` | 2 |
| davon `trait` | 8 |
| davon `type` | 2 |
| davon `const` | 11 |
| davon `mod` | 21 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 22 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 43 |
| `.expect(` (Produktionscode) | 2 |
| `panic!` (Produktionscode) | 8 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 16 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 52 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 169 / 169 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `attention_exporter` | `crates/contextra-infer-candle/src/attention_exporter.rs` | 103 |
| `calibrator` | `crates/contextra-infer-candle/src/calibrator.rs` | 80 |
| `compat::attention` | `crates/contextra-infer-candle/src/compat/attention.rs` | 42 |
| `compat::calibrator` | `crates/contextra-infer-candle/src/compat/calibrator.rs` | 49 |
| `compat::embedding` | `crates/contextra-infer-candle/src/compat/embedding.rs` | 289 |
| `compat::error` | `crates/contextra-infer-candle/src/compat/error.rs` | 141 |
| `compat::fingerprint` | `crates/contextra-infer-candle/src/compat/fingerprint.rs` | 23 |
| `compat::grounding` | `crates/contextra-infer-candle/src/compat/grounding.rs` | 63 |
| `compat::inference_tests` | `crates/contextra-infer-candle/src/compat/inference_tests.rs` | 825 |
| `compat::kv` | `crates/contextra-infer-candle/src/compat/kv.rs` | 136 |
| `compat::kv_bridge` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs` | 694 |
| `compat::llm` | `crates/contextra-infer-candle/src/compat/llm.rs` | 278 |
| `compat` | `crates/contextra-infer-candle/src/compat/mod.rs` | 7 |
| `embedding` | `crates/contextra-infer-candle/src/embedding.rs` | 500 |
| `embedding_provider` | `crates/contextra-infer-candle/src/embedding_provider.rs` | 9 |
| `error` | `crates/contextra-infer-candle/src/error.rs` | 42 |
| `fake` | `crates/contextra-infer-candle/src/fake.rs` | 101 |
| `gasp` | `crates/contextra-infer-candle/src/gasp.rs` | 512 |
| `gguf_loader` | `crates/contextra-infer-candle/src/gguf_loader.rs` | 402 |
| `inference::config` | `crates/contextra-infer-candle/src/inference/config.rs` | 85 |
| `inference::core` | `crates/contextra-infer-candle/src/inference/core.rs` | 704 |
| `inference::prefix_reuse` | `crates/contextra-infer-candle/src/inference/prefix_reuse.rs` | 29 |
| `inference::quantized_llama` | `crates/contextra-infer-candle/src/inference/quantized_llama.rs` | 380 |
| `inference` | `crates/contextra-infer-candle/src/inference.rs` | 25 |
| `kv_state` | `crates/contextra-infer-candle/src/kv_state.rs` | 330 |
| `crate` | `crates/contextra-infer-candle/src/lib.rs` | 77 |
| `model` | `crates/contextra-infer-candle/src/model/mod.rs` | 8 |
| `model::quantized_llama` | `crates/contextra-infer-candle/src/model/quantized_llama.rs` | 590 |
| `model_registry` | `crates/contextra-infer-candle/src/model_registry.rs` | 173 |
| `traits` | `crates/contextra-infer-candle/src/traits.rs` | 111 |

## 6. Re-Exports (`pub use`)

- `pub use crate::compat::embedding::verify_embedding_provider` — `crates/contextra-infer-candle/src/embedding_provider.rs:9` (Modul `embedding_provider`)
- `pub use core::*` — `crates/contextra-infer-candle/src/inference.rs:19` (Modul `inference`)
- `pub use prefix_reuse::*` — `crates/contextra-infer-candle/src/inference.rs:22` (Modul `inference`)
- `pub use crate::compat::kv::` — `crates/contextra-infer-candle/src/inference.rs:25` (Modul `inference`)
- `pub use calibrator::ConfidenceCalibrator` — `crates/contextra-infer-candle/src/lib.rs:29` (Modul `crate`)
- `pub use error::` — `crates/contextra-infer-candle/src/lib.rs:30` (Modul `crate`)
- `pub use fake::` — `crates/contextra-infer-candle/src/lib.rs:32` (Modul `crate`)
- `pub use gguf_loader::` — `crates/contextra-infer-candle/src/lib.rs:34` (Modul `crate`)
- `pub use traits::` — `crates/contextra-infer-candle/src/lib.rs:38` (Modul `crate`)
- `pub use compat::kv_bridge` — `crates/contextra-infer-candle/src/lib.rs:52` (Modul `crate`)
- `pub use attention_exporter::*` — `crates/contextra-infer-candle/src/lib.rs:59` (Modul `crate`)
- `pub use embedding::CandleEmbedClient` — `crates/contextra-infer-candle/src/lib.rs:61` (Modul `crate`)
- `pub use embedding_provider::verify_embedding_provider` — `crates/contextra-infer-candle/src/lib.rs:63` (Modul `crate`)
- `pub use embedding_provider::MAX_CANDLE_EMBED_BATCH_SIZE` — `crates/contextra-infer-candle/src/lib.rs:65` (Modul `crate`)
- `pub use gasp::` — `crates/contextra-infer-candle/src/lib.rs:66` (Modul `crate`)
- `pub use inference::QuantizedLlamaModel` — `crates/contextra-infer-candle/src/lib.rs:68` (Modul `crate`)
- `pub use inference::*` — `crates/contextra-infer-candle/src/lib.rs:70` (Modul `crate`)
- `pub use compat::kv_bridge::KvBridgeAdapter` — `crates/contextra-infer-candle/src/lib.rs:72` (Modul `crate`)
- `pub use kv_state::` — `crates/contextra-infer-candle/src/lib.rs:74` (Modul `crate`)
- `pub use model::quantized_llama` — `crates/contextra-infer-candle/src/lib.rs:76` (Modul `crate`)
- `pub use model_registry::` — `crates/contextra-infer-candle/src/lib.rs:77` (Modul `crate`)
- `pub use quantized_llama::` — `crates/contextra-infer-candle/src/model/mod.rs:8` (Modul `model`)

## 8. Öffentliche Schnittstelle je Item

### Modul `attention_exporter`

#### `struct CandleAttentionExporter`
- Ort: `crates/contextra-infer-candle/src/attention_exporter.rs:11`
```rust
pub struct CandleAttentionExporter
```
- Impl-Block (Trait `Default`) `crates/contextra-infer-candle/src/attention_exporter.rs:21`
- Impl-Block `crates/contextra-infer-candle/src/attention_exporter.rs:27`
- Impl-Block (Trait `AttentionExporter`) `crates/contextra-infer-candle/src/compat/attention.rs:6`
- Impl-Block `crates/contextra-infer-candle/src/compat/attention.rs:13`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-infer-candle/src/attention_exporter.rs:28` |  |
| `pub fn record_attention_weights_local(&self, request_id: u64, weights: Vec<f32>)` | `crates/contextra-infer-candle/src/attention_exporter.rs:34` |  |
| `pub fn record_layer_head_scores_local(&self, request_id: u64, scores: &[Vec<f32>])` | `crates/contextra-infer-candle/src/attention_exporter.rs:53` |  |
| `pub fn export_weights(&self, request_id: u64) -> Option<Vec<f32>>` | `crates/contextra-infer-candle/src/attention_exporter.rs:75` |  |
| `pub fn tracked_request_count(&self) -> usize` | `crates/contextra-infer-candle/src/attention_exporter.rs:79` |  |
| `pub fn clear(&self)` | `crates/contextra-infer-candle/src/attention_exporter.rs:83` |  |
| `pub fn record_attention_weights(&self, request_id: impl Into<u64_request_id::IntoU64>, weights: Vec<f32>)` | `crates/contextra-infer-candle/src/compat/attention.rs:15` | Fassade nach H3: Akzeptiert `RequestId` für Abwärtskompatibilität. |
| `pub fn record_layer_head_scores(&self, request_id: impl Into<u64_request_id::IntoU64>, scores: &[Vec<f32>])` | `crates/contextra-infer-candle/src/compat/attention.rs:20` | Fassade nach H3: Akzeptiert `RequestId` für Abwärtskompatibilität. |

#### `const MAX_TRACKED_REQUESTS`
- Ort: `crates/contextra-infer-candle/src/attention_exporter.rs:8`
```rust
pub const MAX_TRACKED_REQUESTS: usize = 256
```

### Modul `calibrator`

#### `struct ConfigFingerprint`
- Ort: `crates/contextra-infer-candle/src/calibrator.rs:6`
```rust
pub struct ConfigFingerprint
```
- Impl-Block `crates/contextra-infer-candle/src/calibrator.rs:15`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `model_id` | `String` | `crates/contextra-infer-candle/src/calibrator.rs:7` |
| `quantization` | `String` | `crates/contextra-infer-candle/src/calibrator.rs:8` |
| `prompt_template_hash` | `[u8; 32]` | `crates/contextra-infer-candle/src/calibrator.rs:9` |
| `temperature_bits` | `u32` | `crates/contextra-infer-candle/src/calibrator.rs:10` |
| `threshold_bits` | `u32` | `crates/contextra-infer-candle/src/calibrator.rs:12` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( model_id: impl Into<String>, quantization: impl Into<String>, prompt_template: &str, temperature: f32, ) -> Self` | `crates/contextra-infer-candle/src/calibrator.rs:16` |  |
| `pub fn with_threshold(mut self, threshold: f32) -> Self` | `crates/contextra-infer-candle/src/calibrator.rs:35` |  |
| `pub fn threshold(&self) -> f32` | `crates/contextra-infer-candle/src/calibrator.rs:41` |  |
| `pub fn temperature(&self) -> f32` | `crates/contextra-infer-candle/src/calibrator.rs:46` |  |

#### `struct IdentityCalibrator`
- Ort: `crates/contextra-infer-candle/src/calibrator.rs:59`
```rust
pub struct IdentityCalibrator
```
- Impl-Block (Trait `ConfidenceCalibrator`) `crates/contextra-infer-candle/src/calibrator.rs:64`
- Impl-Block (Trait `ConfidenceCalibrator`) `crates/contextra-infer-candle/src/traits.rs:95`

#### `trait ConfidenceCalibrator`
- Ort: `crates/contextra-infer-candle/src/calibrator.rs:51`
```rust
pub trait ConfidenceCalibrator: Send + Sync
```

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn record_outcome(&mut self, raw_score: f32, ground_truth: bool)` | `crates/contextra-infer-candle/src/calibrator.rs:52` |  |
| `fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32>` | `crates/contextra-infer-candle/src/calibrator.rs:53` |  |
| `fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint)` | `crates/contextra-infer-candle/src/calibrator.rs:54` |  |
| `fn observation_count(&self) -> usize` | `crates/contextra-infer-candle/src/calibrator.rs:55` |  |

### Modul `compat`

#### `mod attention`
- Ort: `crates/contextra-infer-candle/src/compat/mod.rs:3`
```rust
pub mod attention
```

#### `mod calibrator`
- Ort: `crates/contextra-infer-candle/src/compat/mod.rs:4`
```rust
pub mod calibrator
```

#### `mod embedding`
- Ort: `crates/contextra-infer-candle/src/compat/mod.rs:6`
```rust
pub mod embedding
```
- Gate: `#[cfg(feature = "candle")]`

#### `mod error`
- Ort: `crates/contextra-infer-candle/src/compat/mod.rs:7`
```rust
pub mod error
```

### Modul `compat::attention`

#### `mod u64_request_id`
- Ort: `crates/contextra-infer-candle/src/compat/attention.rs:26`
```rust
pub mod u64_request_id
```

### Modul `compat::attention::u64_request_id`

#### `struct IntoU64`
- Ort: `crates/contextra-infer-candle/src/compat/attention.rs:29`
```rust
pub struct IntoU64(pub u64)
```
- Impl-Block (Trait `From<u64>`) `crates/contextra-infer-candle/src/compat/attention.rs:31`
- Impl-Block (Trait `From<RequestId>`) `crates/contextra-infer-candle/src/compat/attention.rs:37`

### Modul `compat::calibrator`

#### `struct RankIsotonicCalibratorAdapter`
- Ort: `crates/contextra-infer-candle/src/compat/calibrator.rs:7`
```rust
pub struct RankIsotonicCalibratorAdapter
```
Dokumentation: Adapter um `IsotonicCalibrator` als `ConfidenceCalibrator` zu verwenden.
- Impl-Block `crates/contextra-infer-candle/src/compat/calibrator.rs:12`
- Impl-Block (Trait `ConfidenceCalibrator`) `crates/contextra-infer-candle/src/compat/calibrator.rs:30`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( warmup_required: u32, max_observations: usize, fingerprint: ConfigFingerprint, ) -> Self` | `crates/contextra-infer-candle/src/compat/calibrator.rs:14` | Erstellt einen neuen Adapter. |

### Modul `compat::embedding`

#### `fn verify_embedding_provider`
- Ort: `crates/contextra-infer-candle/src/compat/embedding.rs:102`
```rust
pub fn verify_embedding_provider( provider: &dyn EmbeddingProvider, expected_dim: Option<usize>, ) -> Result<()>
```
Dokumentation: Sichere Startup-Probe zur Überprüfung der Funktionalität und Dimensionsplausibilität eines `EmbeddingProvider`s vor Inbetriebnahme.

#### `const MAX_CANDLE_EMBED_BATCH_SIZE`
- Ort: `crates/contextra-infer-candle/src/compat/embedding.rs:17`
```rust
pub const MAX_CANDLE_EMBED_BATCH_SIZE: usize = 256
```
Dokumentation: Maximum batch size for Candle GGUF embeddings. Set to 256 (lower than ONNX's 512) because GGUF tensor activations and intermediate FP32/FP16 buffers consume higher peak VRAM/RAM per batch item during Candle model forward passes.

### Modul `compat::kv`

#### `struct KvPrefixContext`
- Ort: `crates/contextra-infer-candle/src/compat/kv.rs:78`
```rust
pub struct KvPrefixContext
```
Dokumentation: Wrapper context holding a thread-safe KV prefix store reference.
- Impl-Block (Trait `fmt::Debug`) `crates/contextra-infer-candle/src/compat/kv.rs:82`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `store` | `Arc<dyn KvPrefixStore>` | `crates/contextra-infer-candle/src/compat/kv.rs:79` |

#### `fn build_prefix_key`
- Ort: `crates/contextra-infer-candle/src/compat/kv.rs:91`
```rust
pub fn build_prefix_key( fingerprint: &ModelFingerprint, tokenizer: &tokenizers::Tokenizer, layout: impl Into<PortsKvLayout>, rope: impl Into<PortsRopeConfig>, ) -> Result<PrefixKey>
```
Dokumentation: Constructs a `PrefixKey` for the given model fingerprint, tokenizer, layout, and RoPE config.

#### `fn seed_from_hit`
- Ort: `crates/contextra-infer-candle/src/compat/kv.rs:115`
```rust
pub fn seed_from_hit(hit: &KvPrefixHit, tokens_len: usize) -> Result<PrefixSeed>
```
Dokumentation: Reconstructs a `PrefixSeed` from a `KvPrefixHit` and total prompt token count.

### Modul `compat::kv_bridge`

#### `struct KvBridgeAdapter`
- Ort: `crates/contextra-infer-candle/src/compat/kv_bridge.rs:54`
```rust
pub struct KvBridgeAdapter
```
Dokumentation: Verbindet Retrieval-Chunks mit dem mandantenisolierten, verschlüsselten KV-Cache. # Fail-Safe-Garantie Jede Methode fällt bei Fehler oder Cache-Miss transparent auf den normalen Prefill zurück. Kein Fehler aus dem KV-Store oder der Krypto-Schicht darf eine Anfrage abbrechen.
- Impl-Block `crates/contextra-infer-candle/src/compat/kv_bridge.rs:61`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-infer-candle/src/compat/kv_bridge.rs:292`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `store` | `Arc<TenantIsolatedKvStore>` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:55` |
| `lsm_store` | `Option<Arc<dyn KvBridgeStorage>>` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:56` |
| `cipher` | `Arc<KvSegmentCipher>` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:57` |
| `consultations` | `Arc<std::sync::atomic::AtomicU64>` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:58` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(store: Arc<TenantIsolatedKvStore>, cipher: Arc<KvSegmentCipher>) -> Self` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:63` | Erstellt einen neuen Adapter mit gegebenem Store und Cipher. |
| `pub fn with_lsm_fallback( store: Arc<TenantIsolatedKvStore>, lsm_store: Arc<dyn KvBridgeStorage>, cipher: Arc<KvSegmentCipher>, ) -> Self` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:73` | Erstellt einen neuen Adapter mit Zwei-Tier-Hierarchie (RAM Fast-Path + LSM-Spill Slow-Path). |
| `pub fn consult_segment<'a>(&self, segment: &ContextSegment<'a>)` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:119` | Consults segment metadata and KV cache bridge state for a context segment. |
| `pub fn consultation_count(&self) -> u64` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:131` | Returns the number of segment consultations recorded. |
| `pub fn try_get_cached_segment(&self, tenant: TenantId, key: &KvCacheKey) -> Option<Vec<u8>>` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:174` | Versucht, ein gecachetes KV-Segment zu laden. Gibt `None` zurück bei Cache-Miss, Decrypt-Fehler, Fingerprint-Mismatch oder jedem anderen Fehler. NIEMALS wird ei |
| `pub async fn try_get_cached_segment_async( &self, tenant: TenantId, key: &KvCacheKey, ) -> Option<Vec<u8>>` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:197` | Versucht asynchron, ein gecachetes KV-Segment erst im RAM (Tier 1) und bei Miss im LSM-Store (Tier 2) zu laden. |
| `pub async fn try_get_cached_segment_async_bytes( &self, tenant: TenantId, key: &KvCacheKey, ) -> Option<Bytes>` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:245` | Versucht asynchron, ein gecachetes KV-Segment zu laden und als `Bytes` Slice (Zero-Copy) zurückzugeben. |
| `pub fn store_segment(&self, tenant: TenantId, key: KvCacheKey, plaintext_kv_bytes: Vec<u8>)` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:256` | Speichert ein KV-Segment im Cache. Fehler werden geloggt, nie propagiert. |

#### `struct KvCacheKey`
- Ort: `crates/contextra-infer-candle/src/compat/kv_bridge.rs:32`
```rust
pub struct KvCacheKey
```
Dokumentation: Cache-Lookup-Schlüssel: eindeutige Kombination aus Chunk-ID, Modell und optionalem RoPE-Offset.
- Impl-Block `crates/contextra-infer-candle/src/compat/kv_bridge.rs:38`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `chunk_id` | `u64` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:33` |
| `fingerprint` | `ModelFingerprint` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:34` |
| `rope_offset` | `Option<usize>` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:35` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(chunk_id: u64, fingerprint: ModelFingerprint, rope_offset: Option<usize>) -> Self` | `crates/contextra-infer-candle/src/compat/kv_bridge.rs:39` |  |

### Modul `crate`

#### `mod attention_exporter`
- Ort: `crates/contextra-infer-candle/src/lib.rs:40`
```rust
pub mod attention_exporter
```

#### `mod calibrator`
- Ort: `crates/contextra-infer-candle/src/lib.rs:17`
```rust
pub mod calibrator
```

#### `mod compat`
- Ort: `crates/contextra-infer-candle/src/lib.rs:19`
```rust
pub mod compat
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod embedding`
- Ort: `crates/contextra-infer-candle/src/lib.rs:42`
```rust
pub mod embedding
```
- Gate: `#[cfg(feature = "candle")]`

#### `mod embedding_provider`
- Ort: `crates/contextra-infer-candle/src/lib.rs:44`
```rust
pub mod embedding_provider
```
- Gate: `#[cfg(feature = "candle")]`

#### `mod error`
- Ort: `crates/contextra-infer-candle/src/lib.rs:20`
```rust
pub mod error
```

#### `mod fake`
- Ort: `crates/contextra-infer-candle/src/lib.rs:24`
```rust
pub mod fake
```
- Gate: `#[cfg(feature = "test-utils")]`

#### `mod gasp`
- Ort: `crates/contextra-infer-candle/src/lib.rs:49`
```rust
pub mod gasp
```

#### `mod gguf_loader`
- Ort: `crates/contextra-infer-candle/src/lib.rs:27`
```rust
pub mod gguf_loader
```
- Gate: `#[cfg(feature = "candle")]`

#### `mod inference`
- Ort: `crates/contextra-infer-candle/src/lib.rs:46`
```rust
pub mod inference
```
- Gate: `#[cfg(feature = "candle")]`

#### `mod kv_state`
- Ort: `crates/contextra-infer-candle/src/lib.rs:55`
```rust
pub mod kv_state
```

#### `mod model`
- Ort: `crates/contextra-infer-candle/src/lib.rs:57`
```rust
pub mod model
```

#### `mod model_registry`
- Ort: `crates/contextra-infer-candle/src/lib.rs:47`
```rust
pub mod model_registry
```

#### `mod traits`
- Ort: `crates/contextra-infer-candle/src/lib.rs:21`
```rust
pub mod traits
```

### Modul `embedding`

#### `struct BertEmbedModel`
- Ort: `crates/contextra-infer-candle/src/embedding.rs:229`
```rust
pub struct BertEmbedModel
```
Dokumentation: Real BERT transformer text embedding model wrapper.
- Impl-Block `crates/contextra-infer-candle/src/embedding.rs:234`
- Impl-Block (Trait `CandleEmbedInner`) `crates/contextra-infer-candle/src/embedding.rs:266`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn load(weights_path: &Path, _config_path: &Path, device: &Device) -> Result<Self>` | `crates/contextra-infer-candle/src/embedding.rs:236` | Loads BERT model weights from a `.safetensors` file and configuration from `config.json`. |

#### `struct CandleEmbedClient`
- Ort: `crates/contextra-infer-candle/src/embedding.rs:36`
```rust
pub struct CandleEmbedClient
```
Dokumentation: Text embedding client powered by Candle ML backend.
- Impl-Block (Trait `EmbeddingProvider`) `crates/contextra-infer-candle/src/compat/embedding.rs:19`
- Impl-Block `crates/contextra-infer-candle/src/embedding.rs:53`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `device` | `Device` | `crates/contextra-infer-candle/src/embedding.rs:38` |
| `model` | `Arc<tokio::sync::Mutex<Box<dyn CandleEmbedInner + Send>>>` | `crates/contextra-infer-candle/src/embedding.rs:40` |
| `fingerprint` | `ModelFingerprint` | `crates/contextra-infer-candle/src/embedding.rs:42` |
| `tokenizer` | `tokenizers::Tokenizer` | `crates/contextra-infer-candle/src/embedding.rs:44` |
| `dim` | `usize` | `crates/contextra-infer-candle/src/embedding.rs:46` |
| `max_concurrent_embeddings` | `usize` | `crates/contextra-infer-candle/src/embedding.rs:48` |
| `semaphore` | `Arc<tokio::sync::Semaphore>` | `crates/contextra-infer-candle/src/embedding.rs:50` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( device: Device, model: Box<dyn CandleEmbedInner + Send>, fingerprint: ModelFingerprint, tokenizer: tokenizers::Tokenizer, ) -> Self` | `crates/contextra-infer-candle/src/embedding.rs:55` | Creates a new `CandleEmbedClient`. |
| `pub fn with_max_concurrent_embeddings(mut self, limit: usize) -> Self` | `crates/contextra-infer-candle/src/embedding.rs:75` | Configures maximum concurrent embedding operations for backpressure control. |
| `pub fn fingerprint(&self) -> &ModelFingerprint` | `crates/contextra-infer-candle/src/embedding.rs:83` | Returns a reference to the model's fingerprint. |
| `pub fn embedding_dim(&self) -> usize` | `crates/contextra-infer-candle/src/embedding.rs:88` | Returns the dimensionality of embeddings. |
| `pub async fn embed(&self, text: &str) -> Result<Vec<f32>>` | `crates/contextra-infer-candle/src/embedding.rs:93` | Generates an embedding vector for the provided text. |
| `pub async fn embed_batch<S: AsRef<str>>(&self, texts: &[S]) -> Result<Vec<Vec<f32>>>` | `crates/contextra-infer-candle/src/embedding.rs:114` | Generates embedding vectors for a batch of input texts. |
| `pub fn from_dir( model_dir: &std::path::Path, quantization: crate::model_registry::CandleQuantization, ) -> Result<Self>` | `crates/contextra-infer-candle/src/embedding.rs:140` | Loads a `CandleEmbedClient` from a model directory. |

#### `struct DefaultCandleEmbedModel`
- Ort: `crates/contextra-infer-candle/src/embedding.rs:340`
```rust
pub struct DefaultCandleEmbedModel
```
Dokumentation: Default inner Candle embedding mock model for unit testing when weight files are missing.
- Impl-Block (Trait `CandleEmbedInner`) `crates/contextra-infer-candle/src/embedding.rs:345`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `dim` | `usize` | `crates/contextra-infer-candle/src/embedding.rs:342` |

#### `trait CandleEmbedInner`
- Ort: `crates/contextra-infer-candle/src/embedding.rs:22`
```rust
pub trait CandleEmbedInner: Send
```
Dokumentation: Inner trait abstracting low-level Candle forward execution for vector embeddings. Enables mock-based unit testing without loading full ONNX or GGUF files in CI.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn embed( &mut self, text: &str, tokenizer: &tokenizers::Tokenizer, device: &Device, ) -> Result<Vec<f32>>` | `crates/contextra-infer-candle/src/embedding.rs:24` | Generates an embedding vector for the provided input text. |
| `fn dim(&self) -> usize` | `crates/contextra-infer-candle/src/embedding.rs:32` | Returns the vector dimension produced by this embedding model. |

#### `const DEFAULT_MAX_CONCURRENT_EMBEDDINGS`
- Ort: `crates/contextra-infer-candle/src/embedding.rs:17`
```rust
pub const DEFAULT_MAX_CONCURRENT_EMBEDDINGS: usize = 8
```
Dokumentation: Default maximum concurrent embedding operations for Candle vector embedding.

### Modul `embedding_provider`

#### `const MAX_CANDLE_EMBED_BATCH_SIZE`
- Ort: `crates/contextra-infer-candle/src/embedding_provider.rs:6`
```rust
pub const MAX_CANDLE_EMBED_BATCH_SIZE: usize = 256
```
Dokumentation: Maximum batch size for Candle GGUF embeddings.

### Modul `error`

#### `enum CandleError`
- Ort: `crates/contextra-infer-candle/src/error.rs:9`
```rust
pub enum CandleError
```
- Impl-Block (Trait `From<contextra_types::ContextraError>`) `crates/contextra-infer-candle/src/compat/error.rs:40`

Varianten:

- `Io(#[from] std::io::Error)` — `crates/contextra-infer-candle/src/error.rs:12`
- `ModelLoad{` — `crates/contextra-infer-candle/src/error.rs:15`
- `Tokenizer(String)` — `crates/contextra-infer-candle/src/error.rs:20`
- `Inference(String)` — `crates/contextra-infer-candle/src/error.rs:23`
- `InvalidInput(String)` — `crates/contextra-infer-candle/src/error.rs:26`
- `ResourceExceeded(String)` — `crates/contextra-infer-candle/src/error.rs:29`
- `PolicyViolation(String)` — `crates/contextra-infer-candle/src/error.rs:32`
- `BackendUnavailable(String)` — `crates/contextra-infer-candle/src/error.rs:35`

#### `type Error`
- Ort: `crates/contextra-infer-candle/src/error.rs:39`
```rust
pub type Error = CandleError
```
Dokumentation: Typ-Alias zur Abwärtskompatibilität innerhalb des Crates und für externe Aufrufer.

#### `type Result`
- Ort: `crates/contextra-infer-candle/src/error.rs:42`
```rust
pub type Result<T> = std::result::Result<T, CandleError>
```
Dokumentation: Kanonischer Result-Typ für `contextra-infer-candle`.

### Modul `fake`

#### `struct FakeEmbedder`
- Ort: `crates/contextra-infer-candle/src/fake.rs:15`
```rust
pub struct FakeEmbedder
```
Dokumentation: Deterministischer Fake-Embedder für Tests und Standalone-Nutzung ohne Candle.
- Impl-Block `crates/contextra-infer-candle/src/fake.rs:19`
- Impl-Block (Trait `Default`) `crates/contextra-infer-candle/src/fake.rs:26`
- Impl-Block (Trait `Embedder`) `crates/contextra-infer-candle/src/fake.rs:32`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(dimension: usize) -> Self` | `crates/contextra-infer-candle/src/fake.rs:21` | Erstellt eine neue Instanz mit der gewünschten Vektordimension. |

#### `struct FakeTextGenerator`
- Ort: `crates/contextra-infer-candle/src/fake.rs:72`
```rust
pub struct FakeTextGenerator
```
Dokumentation: Deterministischer Fake-Text-Generator für Tests und Standalone-Nutzung.
- Impl-Block `crates/contextra-infer-candle/src/fake.rs:76`
- Impl-Block (Trait `TextGenerator`) `crates/contextra-infer-candle/src/fake.rs:85`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(prefix: impl Into<String>) -> Self` | `crates/contextra-infer-candle/src/fake.rs:78` | Erstellt eine neue Instanz mit optionalem Präfix für generierte Antworten. |

### Modul `gasp`

#### `struct ContextChunk`
- Ort: `crates/contextra-infer-candle/src/gasp.rs:41`
```rust
pub struct ContextChunk
```
- Impl-Block `crates/contextra-infer-candle/src/gasp.rs:48`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `u64` | `crates/contextra-infer-candle/src/gasp.rs:42` |
| `content` | `String` | `crates/contextra-infer-candle/src/gasp.rs:43` |
| `relevance` | `f32` | `crates/contextra-infer-candle/src/gasp.rs:44` |
| `contextual_prefix` | `Option<String>` | `crates/contextra-infer-candle/src/gasp.rs:45` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn combined_text_owned(&self) -> String` | `crates/contextra-infer-candle/src/gasp.rs:49` |  |

#### `struct GaspConfig`
- Ort: `crates/contextra-infer-candle/src/gasp.rs:16`
```rust
pub struct GaspConfig
```
- Impl-Block (Trait `Default`) `crates/contextra-infer-candle/src/gasp.rs:23`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `threshold` | `f32` | `crates/contextra-infer-candle/src/gasp.rs:17` |
| `warmup_required` | `u32` | `crates/contextra-infer-candle/src/gasp.rs:18` |
| `max_observations` | `usize` | `crates/contextra-infer-candle/src/gasp.rs:19` |
| `fingerprint` | `ConfigFingerprint` | `crates/contextra-infer-candle/src/gasp.rs:20` |

#### `struct GaspValidator`
- Ort: `crates/contextra-infer-candle/src/gasp.rs:89`
```rust
pub struct GaspValidator
```
- Impl-Block (Trait `ResponseGroundingValidator`) `crates/contextra-infer-candle/src/compat/grounding.rs:20`
- Impl-Block (Trait `GroundingValidator`) `crates/contextra-infer-candle/src/compat/grounding.rs:27`
- Impl-Block `crates/contextra-infer-candle/src/compat/grounding.rs:52`
- Impl-Block `crates/contextra-infer-candle/src/gasp.rs:94`
- Impl-Block (Trait `Default`) `crates/contextra-infer-candle/src/gasp.rs:293`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn compute_raw_grounding_score( &self, response: &str, context_chunks: &[ContextChunk], ) -> Result<f32>` | `crates/contextra-infer-candle/src/compat/grounding.rs:54` | Fassade nach H3: Akzeptiert `&[contextra_types::ContextChunk]` für Abwärtskompatibilität. |
| `pub fn new() -> Self` | `crates/contextra-infer-candle/src/gasp.rs:95` |  |
| `pub fn with_config(config: GaspConfig) -> Self` | `crates/contextra-infer-candle/src/gasp.rs:99` |  |
| `pub fn with_calibrator( config: GaspConfig, mut calibrator: Box<dyn ConfidenceCalibrator>, ) -> Self` | `crates/contextra-infer-candle/src/gasp.rs:123` |  |
| `pub fn set_threshold(&mut self, threshold: f32)` | `crates/contextra-infer-candle/src/gasp.rs:134` |  |
| `pub fn record_external_feedback(&self, raw_score: f32, ground_truth_grounded: bool)` | `crates/contextra-infer-candle/src/gasp.rs:141` |  |
| `pub fn threshold(&self) -> f32` | `crates/contextra-infer-candle/src/gasp.rs:147` |  |
| `pub fn config(&self) -> &GaspConfig` | `crates/contextra-infer-candle/src/gasp.rs:151` |  |
| `pub fn refresh_config(&mut self, config: GaspConfig)` | `crates/contextra-infer-candle/src/gasp.rs:155` |  |
| `pub fn observation_count(&self) -> usize` | `crates/contextra-infer-candle/src/gasp.rs:162` |  |
| `pub fn compute_raw_grounding_score_local( &self, response: &str, context_chunks: &[ContextChunk], ) -> Result<f32>` | `crates/contextra-infer-candle/src/gasp.rs:170` |  |
| `pub fn score_grounding_slices(&self, response: &str, sources: &[&str]) -> Result<f32>` | `crates/contextra-infer-candle/src/gasp.rs:255` |  |
| `pub async fn validate_grounding_local( &self, response: &str, context_chunks: &[ContextChunk], ) -> Result<f32>` | `crates/contextra-infer-candle/src/gasp.rs:269` |  |

#### `const DEFAULT_GROUNDING_THRESHOLD`
- Ort: `crates/contextra-infer-candle/src/gasp.rs:13`
```rust
pub const DEFAULT_GROUNDING_THRESHOLD: f32 = 0.70
```

### Modul `gguf_loader`

#### `struct GgufMetadata`
- Ort: `crates/contextra-infer-candle/src/gguf_loader.rs:34`
```rust
pub struct GgufMetadata
```
Dokumentation: Metadata extracted from a GGUF model container.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `architecture` | `String` | `crates/contextra-infer-candle/src/gguf_loader.rs:36` |
| `tensor_count` | `usize` | `crates/contextra-infer-candle/src/gguf_loader.rs:38` |
| `metadata_keys` | `Vec<String>` | `crates/contextra-infer-candle/src/gguf_loader.rs:40` |

#### `fn parse_gguf_metadata`
- Ort: `crates/contextra-infer-candle/src/gguf_loader.rs:45`
```rust
pub fn parse_gguf_metadata(model_path: &Path) -> Result<GgufMetadata>
```
Dokumentation: Inspects a GGUF model file and extracts its architecture and metadata header keys after performing strict header validation and optional SHA-256 verification.

#### `fn parse_gguf_metadata_with_checksum`
- Ort: `crates/contextra-infer-candle/src/gguf_loader.rs:54`
```rust
pub fn parse_gguf_metadata_with_checksum( model_path: &Path, expected_sha256: Option<&[u8
```
Dokumentation: Inspects a GGUF model file with optional SHA-256 verification.

#### `const MAX_GGUF_METADATA_DEPTH`
- Ort: `crates/contextra-infer-candle/src/gguf_loader.rs:30`
```rust
pub const MAX_GGUF_METADATA_DEPTH: usize = 2
```
Dokumentation: Maximum permitted depth for nested GGUF metadata array parsing.

#### `const MAX_GGUF_METADATA_KVS`
- Ort: `crates/contextra-infer-candle/src/gguf_loader.rs:24`
```rust
pub const MAX_GGUF_METADATA_KVS: u64 = 65_536
```
Dokumentation: Maximum number of metadata key-value pairs permitted in a valid GGUF header.

#### `const MAX_GGUF_STRING_LEN`
- Ort: `crates/contextra-infer-candle/src/gguf_loader.rs:27`
```rust
pub const MAX_GGUF_STRING_LEN: u64 = 16_777_216
```
Dokumentation: Maximum length of a string in bytes permitted in a GGUF metadata string field.

#### `const MAX_GGUF_TENSORS`
- Ort: `crates/contextra-infer-candle/src/gguf_loader.rs:21`
```rust
pub const MAX_GGUF_TENSORS: u64 = 65_536
```
Dokumentation: Maximum number of tensors permitted in a valid GGUF header.

### Modul `inference`

#### `mod core`
- Ort: `crates/contextra-infer-candle/src/inference.rs:13`
```rust
pub mod core
```

### Modul `inference::config`

#### `struct GenerationConfig`
- Ort: `crates/contextra-infer-candle/src/inference/config.rs:9`
```rust
pub struct GenerationConfig
```
Dokumentation: Sampling and resource bounds configuration for Candle LLM text generation.
- Impl-Block (Trait `Default`) `crates/contextra-infer-candle/src/inference/config.rs:22`
- Impl-Block `crates/contextra-infer-candle/src/inference/config.rs:34`
- Impl-Block (Trait `Default`) `crates/contextra-infer-candle/src/traits.rs:44`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `seed` | `u64` | `crates/contextra-infer-candle/src/inference/config.rs:11` |
| `temperature` | `f64` | `crates/contextra-infer-candle/src/inference/config.rs:13` |
| `top_p` | `Option<f64>` | `crates/contextra-infer-candle/src/inference/config.rs:15` |
| `max_context_length` | `usize` | `crates/contextra-infer-candle/src/inference/config.rs:17` |
| `max_token_budget` | `usize` | `crates/contextra-infer-candle/src/inference/config.rs:19` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn deterministic(seed: u64) -> Self` | `crates/contextra-infer-candle/src/inference/config.rs:36` | Creates a deterministic generation configuration (temperature = 0.0). |
| `pub fn with_max_context_length(mut self, limit: usize) -> Self` | `crates/contextra-infer-candle/src/inference/config.rs:47` | Sets the maximum context length in tokens. |
| `pub fn with_max_token_budget(mut self, budget: usize) -> Self` | `crates/contextra-infer-candle/src/inference/config.rs:53` | Sets the maximum token budget for generation. |
| `pub fn validate_prompt_len(&self, prompt_token_count: usize) -> Result<(), ContextraError>` | `crates/contextra-infer-candle/src/inference/config.rs:59` | Validates input prompt token length against `max_context_length`. |
| `pub fn validate_token_budget(&self, requested_tokens: usize) -> Result<(), ContextraError>` | `crates/contextra-infer-candle/src/inference/config.rs:73` | Validates target sample length against `max_token_budget`. |

### Modul `inference::core`

#### `struct CandleLlmClient`
- Ort: `crates/contextra-infer-candle/src/inference/core.rs:95`
```rust
pub struct CandleLlmClient
```
Dokumentation: LLM Text Generator implementation powered by Candle inference engine.
- Impl-Block (Trait `LlmTextGenerator`) `crates/contextra-infer-candle/src/compat/llm.rs:10`
- Impl-Block (Trait `LlmTextGeneratorStreaming`) `crates/contextra-infer-candle/src/compat/llm.rs:112`
- Impl-Block `crates/contextra-infer-candle/src/compat/llm.rs:172`
- Impl-Block `crates/contextra-infer-candle/src/inference/core.rs:123`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `device` | `Device` | `crates/contextra-infer-candle/src/inference/core.rs:97` |
| `model` | `Arc<tokio::sync::Mutex<Box<dyn CandleModelInner + Send>>>` | `crates/contextra-infer-candle/src/inference/core.rs:99` |
| `fingerprint` | `ModelFingerprint` | `crates/contextra-infer-candle/src/inference/core.rs:101` |
| `tokenizer` | `tokenizers::Tokenizer` | `crates/contextra-infer-candle/src/inference/core.rs:103` |
| `max_concurrent_inferences` | `usize` | `crates/contextra-infer-candle/src/inference/core.rs:105` |
| `semaphore` | `Arc<tokio::sync::Semaphore>` | `crates/contextra-infer-candle/src/inference/core.rs:107` |
| `kv_bridge` | `Option<crate::KvBridgeAdapter>` | `crates/contextra-infer-candle/src/inference/core.rs:110` |
| `prefill_count` | `Arc<std::sync::atomic::AtomicU64>` | `crates/contextra-infer-candle/src/inference/core.rs:112` |
| `prefill_skip_count` | `Arc<std::sync::atomic::AtomicU64>` | `crates/contextra-infer-candle/src/inference/core.rs:114` |
| `prefix_store` | `Option<crate::inference::KvPrefixContext>` | `crates/contextra-infer-candle/src/inference/core.rs:117` |
| `prefill_skipped_tokens` | `Arc<std::sync::atomic::AtomicU64>` | `crates/contextra-infer-candle/src/inference/core.rs:120` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn with_prefix_store(mut self, store: Arc<dyn contextra_ports::kv::KvPrefixStore>) -> Self` | `crates/contextra-infer-candle/src/compat/llm.rs:174` | Configures optional `KvPrefixStore` context for stage B prefix reuse. |
| `pub fn generate_with_prefix_store<'a>( &'a self, tenant: TenantId, segments: &'a [ContextSegment<'a>], ) -> BoxFuture<'a, Result<String>>` | `crates/contextra-infer-candle/src/compat/llm.rs:180` | Generates completion with stage B KV prefix reuse and tenant isolation. |
| `pub fn new( device: Device, model: Box<dyn CandleModelInner + Send>, fingerprint: ModelFingerprint, tokenizer: tokenizers::Tokenizer, ) -> Self` | `crates/contextra-infer-candle/src/inference/core.rs:125` | Creates a new `CandleLlmClient` with default concurrency limits. |
| `pub fn prefill_count(&self) -> u64` | `crates/contextra-infer-candle/src/inference/core.rs:151` | Returns the total count of segment full prefill calculations performed. |
| `pub fn prefill_skip_count(&self) -> u64` | `crates/contextra-infer-candle/src/inference/core.rs:156` | Returns the total count of segment prefill skips via KV cache hits. |
| `pub fn prefill_skipped_tokens(&self) -> u64` | `crates/contextra-infer-candle/src/inference/core.rs:163` | Returns the total count of prompt tokens skipped via KV prefix reuse. |
| `pub fn with_max_concurrent_inferences(mut self, limit: usize) -> Self` | `crates/contextra-infer-candle/src/inference/core.rs:169` | Configures maximum concurrent inference operations for backpressure control. |
| `pub fn with_kv_bridge(mut self, adapter: crate::KvBridgeAdapter) -> Self` | `crates/contextra-infer-candle/src/inference/core.rs:178` | Configures optional `KvBridgeAdapter` for KV cache segment retrieval and injection. |
| `pub fn fingerprint(&self) -> &ModelFingerprint` | `crates/contextra-infer-candle/src/inference/core.rs:184` | Returns a reference to the model's fingerprint. |
| `pub fn swap_model( &mut self, new_model: Box<dyn CandleModelInner + Send>, new_fingerprint: ModelFingerprint, validator: Option<&mut GaspValidator>, )` | `crates/contextra-infer-candle/src/inference/core.rs:191` | Tauscht das zugrundeliegende Modell und dessen Fingerprint zur Laufzeit aus. Falls ein optionaler `GaspValidator` übergeben wird, wird dessen Kalibrierungsstatu |
| `pub fn from_dir( model_dir: &std::path::Path, quantization: crate::model_registry::CandleQuantization, ) -> Result<Self>` | `crates/contextra-infer-candle/src/inference/core.rs:220` | Loads a `CandleLlmClient` from a model directory. |

#### `struct DefaultCandleLlmModel`
- Ort: `crates/contextra-infer-candle/src/inference/core.rs:669`
```rust
pub struct DefaultCandleLlmModel
```
Dokumentation: Default inner Candle LLM mock model for unit tests when binary weights are absent.
- Impl-Block (Trait `CandleModelInner`) `crates/contextra-infer-candle/src/inference/core.rs:672`

#### `struct QuantizedLlamaModel`
- Ort: `crates/contextra-infer-candle/src/inference/core.rs:315`
```rust
pub struct QuantizedLlamaModel
```
Dokumentation: Real quantized Llama / GGUF model execution wrapper.
- Impl-Block `crates/contextra-infer-candle/src/inference/core.rs:320`
- Impl-Block (Trait `CandleModelInner`) `crates/contextra-infer-candle/src/inference/core.rs:355`
- Impl-Block `crates/contextra-infer-candle/src/inference/quantized_llama.rs:24`
- Impl-Block (Trait `CandleModelInner`) `crates/contextra-infer-candle/src/inference/quantized_llama.rs:81`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn load(model_path: &Path, device: &Device) -> Result<Self>` | `crates/contextra-infer-candle/src/inference/core.rs:322` | Loads GGUF quantized model weights from the specified file path. |
| `pub fn load(model_path: &Path, device: &Device) -> Result<Self>` | `crates/contextra-infer-candle/src/inference/quantized_llama.rs:26` | Loads GGUF quantized model weights from the specified file path with default generation config. |
| `pub fn load_with_config( model_path: &Path, device: &Device, config: GenerationConfig, ) -> Result<Self>` | `crates/contextra-infer-candle/src/inference/quantized_llama.rs:31` | Loads GGUF quantized model weights from file path with custom `GenerationConfig`. |
| `pub fn with_generation_config(mut self, config: GenerationConfig) -> Self` | `crates/contextra-infer-candle/src/inference/quantized_llama.rs:62` | Configures generation config for sampling parameters and context bounds. |
| `pub fn truncate_kv_cache(&mut self, pos: usize) -> Result<()>` | `crates/contextra-infer-candle/src/inference/quantized_llama.rs:70` | Truncates the underlying layer-wise KV cache tensors to `pos` tokens. Used when context window overflow occurs or prefix reuse requires resetting tail cache. |
| `pub fn clear_kv_cache(&mut self)` | `crates/contextra-infer-candle/src/inference/quantized_llama.rs:76` | Clears the underlying layer-wise KV cache tensors completely. |

#### `trait CandleModelInner`
- Ort: `crates/contextra-infer-candle/src/inference/core.rs:32`
```rust
pub trait CandleModelInner: Send
```
Dokumentation: Inner trait abstracting low-level Candle forward/text-generation execution. This trait allows dependency injection for unit testing with mock models without requiring full GGUF binary weights in CI environments.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn generate( &mut self, prompt: &str, tokenizer: &tokenizers::Tokenizer, device: &Device, ) -> Result<String>` | `crates/contextra-infer-candle/src/inference/core.rs:34` | Generates text completion for a given prompt using tokenizer and device settings. |
| `fn generate_stream( &mut self, prompt: &str, tokenizer: &tokenizers::Tokenizer, device: &Device, on_token: &mut dyn FnMut(String) -> bool, ) -> Result<String>` | `crates/contextra-infer-candle/src/inference/core.rs:43` | Generates text stream for a given prompt, invoking `on_token` for each generated token or chunk. Returns early if `on_token` returns `false`. |
| `fn kv_layout( &self, ) -> Option<( crate::kv_state::KvLayout, crate::kv_state::RopeConfig, )>` | `crates/contextra-infer-candle/src/inference/core.rs:57` | Returns the KV layout and RoPE configuration of the underlying model, if supported. |
| `fn generate_stream_with_prefix( &mut self, prompt: &str, tokenizer: &tokenizers::Tokenizer, device: &Device, seed: Option<crate::inference::PrefixSeed>, on_token: &mut dyn FnMut(String) -> bool, ) -> Result<crate::inference::PrefixRun>` | `crates/contextra-infer-candle/src/inference/core.rs:68` | Generates text stream using an optional KV prefix seed state. |

#### `const DEFAULT_MAX_CONCURRENT_INFERENCES`
- Ort: `crates/contextra-infer-candle/src/inference/core.rs:92`
```rust
pub const DEFAULT_MAX_CONCURRENT_INFERENCES: usize = 4
```
Dokumentation: Default maximum concurrent inference operations for Candle LLM text generation.

### Modul `inference::prefix_reuse`

#### `struct PrefixRun`
- Ort: `crates/contextra-infer-candle/src/inference/prefix_reuse.rs:20`
```rust
pub struct PrefixRun
```
Dokumentation: Execution result of a prefix-assisted generation run.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `output` | `String` | `crates/contextra-infer-candle/src/inference/prefix_reuse.rs:22` |
| `prompt_tokens` | `Vec<u32>` | `crates/contextra-infer-candle/src/inference/prefix_reuse.rs:24` |
| `reused_tokens` | `usize` | `crates/contextra-infer-candle/src/inference/prefix_reuse.rs:26` |
| `exported_prefix` | `Option<KvBlock>` | `crates/contextra-infer-candle/src/inference/prefix_reuse.rs:28` |

#### `struct PrefixSeed`
- Ort: `crates/contextra-infer-candle/src/inference/prefix_reuse.rs:11`
```rust
pub struct PrefixSeed
```
Dokumentation: Seed state for executing prompt generation with an imported KV cache prefix.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `matched_len` | `usize` | `crates/contextra-infer-candle/src/inference/prefix_reuse.rs:13` |
| `state` | `KvState` | `crates/contextra-infer-candle/src/inference/prefix_reuse.rs:15` |

### Modul `inference::quantized_llama`

#### `struct QuantizedLlamaModel`
- Ort: `crates/contextra-infer-candle/src/inference/quantized_llama.rs:19`
```rust
pub struct QuantizedLlamaModel
```
Dokumentation: Real quantized Llama / GGUF model execution wrapper.

### Modul `kv_state`

#### `struct KvBlock`
- Ort: `crates/contextra-infer-candle/src/kv_state.rs:14`
```rust
pub struct KvBlock
```
Dokumentation: Repräsentiert ein einzelnes exportiertes KV-Cache-Block-Segment.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `block_id` | `u64` | `crates/contextra-infer-candle/src/kv_state.rs:15` |
| `data` | `Bytes` | `crates/contextra-infer-candle/src/kv_state.rs:16` |

#### `struct KvLayout`
- Ort: `crates/contextra-infer-candle/src/kv_state.rs:21`
```rust
pub struct KvLayout
```
Dokumentation: Konfiguration für KV-Cache-Layout und Dimensionen.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `n_layer` | `u32` | `crates/contextra-infer-candle/src/kv_state.rs:22` |
| `n_kv_head` | `u32` | `crates/contextra-infer-candle/src/kv_state.rs:23` |
| `head_dim` | `u32` | `crates/contextra-infer-candle/src/kv_state.rs:24` |
| `dtype` | `String` | `crates/contextra-infer-candle/src/kv_state.rs:25` |

#### `struct KvState`
- Ort: `crates/contextra-infer-candle/src/kv_state.rs:61`
```rust
pub struct KvState
```
Dokumentation: Explicit KV-Cache state holding layer-wise tensors and current sequence length position.
- Impl-Block `crates/contextra-infer-candle/src/kv_state.rs:166`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `layers` | `Vec<LayerKv>` | `crates/contextra-infer-candle/src/kv_state.rs:62` |
| `pos` | `usize` | `crates/contextra-infer-candle/src/kv_state.rs:63` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(layers: Vec<LayerKv>, pos: usize) -> Self` | `crates/contextra-infer-candle/src/kv_state.rs:167` |  |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-infer-candle/src/kv_state.rs:171` |  |
| `pub fn pos(&self) -> usize` | `crates/contextra-infer-candle/src/kv_state.rs:175` |  |
| `pub fn layer_count(&self) -> usize` | `crates/contextra-infer-candle/src/kv_state.rs:179` |  |
| `pub fn truncate(&mut self, len: usize) -> Result<()>` | `crates/contextra-infer-candle/src/kv_state.rs:200` |  |
| `pub fn export_block(&self, range: Range<usize>) -> Result<KvBlock>` | `crates/contextra-infer-candle/src/kv_state.rs:223` |  |
| `pub fn import_block(&mut self, block: &KvBlock) -> Result<()>` | `crates/contextra-infer-candle/src/kv_state.rs:269` |  |
| `pub fn import_block_at(&mut self, block: &KvBlock, at: usize) -> Result<()>` | `crates/contextra-infer-candle/src/kv_state.rs:317` |  |

#### `struct LayerKv`
- Ort: `crates/contextra-infer-candle/src/kv_state.rs:48`
```rust
pub struct LayerKv
```
Dokumentation: Key and Value tensors for a single transformer layer.
- Impl-Block `crates/contextra-infer-candle/src/kv_state.rs:53`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `k` | `Tensor` | `crates/contextra-infer-candle/src/kv_state.rs:49` |
| `v` | `Tensor` | `crates/contextra-infer-candle/src/kv_state.rs:50` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(k: Tensor, v: Tensor) -> Self` | `crates/contextra-infer-candle/src/kv_state.rs:54` |  |

#### `struct RopeConfig`
- Ort: `crates/contextra-infer-candle/src/kv_state.rs:30`
```rust
pub struct RopeConfig
```
Dokumentation: Konfiguration für RoPE (Rotary Position Embeddings).
- Impl-Block (Trait `Eq`) `crates/contextra-infer-candle/src/kv_state.rs:35`
- Impl-Block (Trait `std::hash::Hash`) `crates/contextra-infer-candle/src/kv_state.rs:37`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `base` | `f32` | `crates/contextra-infer-candle/src/kv_state.rs:31` |
| `scaling` | `Option<f32>` | `crates/contextra-infer-candle/src/kv_state.rs:32` |

### Modul `model`

#### `mod quantized_llama`
- Ort: `crates/contextra-infer-candle/src/model/mod.rs:6`
```rust
pub mod quantized_llama
```

### Modul `model::quantized_llama`

#### `struct LayerWeights`
- Ort: `crates/contextra-infer-candle/src/model/quantized_llama.rs:150`
```rust
pub struct LayerWeights
```
- Impl-Block `crates/contextra-infer-candle/src/model/quantized_llama.rs:171`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `attention_wq` | `QuantizedMatMul` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:151` |
| `attention_wk` | `QuantizedMatMul` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:152` |
| `attention_wv` | `QuantizedMatMul` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:153` |
| `attention_wo` | `QuantizedMatMul` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:154` |
| `attention_norm` | `RmsNorm` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:155` |
| `ffn_norm` | `RmsNorm` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:157` |
| `n_head` | `usize` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:158` |
| `n_kv_head` | `usize` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:159` |
| `head_dim` | `usize` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:160` |
| `rope_is_neox` | `bool` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:161` |
| `cos` | `Tensor` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:162` |
| `sin` | `Tensor` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:163` |
| `neg_inf` | `Tensor` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:164` |

#### `struct ModelWeights`
- Ort: `crates/contextra-infer-candle/src/model/quantized_llama.rs:294`
```rust
pub struct ModelWeights
```
- Impl-Block `crates/contextra-infer-candle/src/model/quantized_llama.rs:309`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tok_embeddings` | `Embedding` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:296` |
| `layers` | `Vec<LayerWeights>` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:298` |
| `norm` | `RmsNorm` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:300` |
| `output` | `QuantizedMatMul` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:302` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_gguf<R: std::io::Seek + std::io::Read>( ct: gguf_file::Content, reader: &mut R, device: &Device, ) -> Result<Self, ContextraError>` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:312` |  |
| `pub fn export_kv_state(&self) -> Result<KvState, ContextraError>` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:464` |  |
| `pub fn import_kv_state(&mut self, state: &KvState) -> Result<(), ContextraError>` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:487` |  |
| `pub fn truncate_kv_cache(&mut self, pos: usize) -> Result<(), ContextraError>` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:506` |  |
| `pub fn clear_kv_cache(&mut self)` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:533` |  |
| `pub fn forward(&mut self, x: &Tensor, index_pos: usize) -> Result<Tensor, ContextraError>` | `crates/contextra-infer-candle/src/model/quantized_llama.rs:542` |  |

#### `struct QuantizedMatMul`
- Ort: `crates/contextra-infer-candle/src/model/quantized_llama.rs:94`
```rust
pub struct QuantizedMatMul
```
- Impl-Block `crates/contextra-infer-candle/src/model/quantized_llama.rs:99`

#### `const MAX_SEQ_LEN`
- Ort: `crates/contextra-infer-candle/src/model/quantized_llama.rs:25`
```rust
pub const MAX_SEQ_LEN: usize = 4096
```

### Modul `model_registry`

#### `struct ModelFingerprint`
- Ort: `crates/contextra-infer-candle/src/model_registry.rs:52`
```rust
pub struct ModelFingerprint
```
Dokumentation: Eigener Newtype für Modell-Fingerabdrücke in contextra-infer-candle.
- Impl-Block (Trait `From<ModelFingerprint>`) `crates/contextra-infer-candle/src/compat/error.rs:63`
- Impl-Block (Trait `From<contextra_types::ModelFingerprint>`) `crates/contextra-infer-candle/src/compat/error.rs:73`
- Impl-Block (Trait `From<LocalModelFingerprint>`) `crates/contextra-infer-candle/src/compat/fingerprint.rs:5`
- Impl-Block `crates/contextra-infer-candle/src/model_registry.rs:61`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `hash` | `[u8; 32]` | `crates/contextra-infer-candle/src/model_registry.rs:54` |
| `model_id` | `String` | `crates/contextra-infer-candle/src/model_registry.rs:56` |
| `quantization` | `String` | `crates/contextra-infer-candle/src/model_registry.rs:58` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( hash: [u8` | `crates/contextra-infer-candle/src/model_registry.rs:63` | Erstellt einen neuen ModelFingerprint. |

#### `enum CandleQuantization`
- Ort: `crates/contextra-infer-candle/src/model_registry.rs:16`
```rust
pub enum CandleQuantization
```
Dokumentation: Supported Candle quantization formats.
- Impl-Block (Trait `fmt::Display`) `crates/contextra-infer-candle/src/model_registry.rs:25`
- Impl-Block (Trait `FromStr`) `crates/contextra-infer-candle/src/model_registry.rs:35`

Varianten:

- `Q4KM` — `crates/contextra-infer-candle/src/model_registry.rs:18`
- `Q8_0` — `crates/contextra-infer-candle/src/model_registry.rs:20`
- `F16` — `crates/contextra-infer-candle/src/model_registry.rs:22`

#### `fn compute_fingerprint`
- Ort: `crates/contextra-infer-candle/src/model_registry.rs:77`
```rust
pub fn compute_fingerprint( model_path: &Path, quantization: &CandleQuantization, ) -> Result<ModelFingerprint>
```
Dokumentation: Computes a unique fingerprint for a model file and quantization variant.

### Modul `traits`

#### `struct GenerationConfig`
- Ort: `crates/contextra-infer-candle/src/traits.rs:31`
```rust
pub struct GenerationConfig
```
Dokumentation: Optionen für die Textgenerierung.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_new_tokens` | `usize` | `crates/contextra-infer-candle/src/traits.rs:33` |
| `temperature` | `f64` | `crates/contextra-infer-candle/src/traits.rs:35` |
| `top_p` | `Option<f64>` | `crates/contextra-infer-candle/src/traits.rs:37` |
| `seed` | `Option<u64>` | `crates/contextra-infer-candle/src/traits.rs:39` |
| `stop_tokens` | `Vec<String>` | `crates/contextra-infer-candle/src/traits.rs:41` |

#### `struct IdentityCalibrator`
- Ort: `crates/contextra-infer-candle/src/traits.rs:91`
```rust
pub struct IdentityCalibrator
```
Dokumentation: Deterministische Null-Kalibrierung für autarken Betrieb ohne rank-Crate.

#### `trait ConfidenceCalibrator`
- Ort: `crates/contextra-infer-candle/src/traits.rs:75`
```rust
pub trait ConfidenceCalibrator: Send + Sync
```
Dokumentation: Lokaler Trait zur Konfidenz-Kalibrierung (entkoppelt von `contextra-rank`).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32>` | `crates/contextra-infer-candle/src/traits.rs:77` | Liefert die kalibrierte Wahrscheinlichkeit für einen Roh-Score in [0.0, 1.0]. |
| `fn record_outcome(&mut self, raw_score: f32, ground_truth: bool)` | `crates/contextra-infer-candle/src/traits.rs:80` | Zeichnet eine Beobachtung für die Kalibrierung auf. |
| `fn observation_count(&self) -> usize` | `crates/contextra-infer-candle/src/traits.rs:83` | Liefert die Anzahl gesammelter Beobachtungen. |
| `fn reset(&mut self)` | `crates/contextra-infer-candle/src/traits.rs:86` | Setzt die Kalibrierung zurück (z.B. bei Schwellenwert- oder Modelländerung). |

#### `trait Embedder`
- Ort: `crates/contextra-infer-candle/src/traits.rs:12`
```rust
pub trait Embedder: Send + Sync
```
Dokumentation: Lokaler Trait für Vektor-Embeddings (z.B. BERT, MiniLM).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn embed(&self, text: &str) -> Result<Vec<f32>>` | `crates/contextra-infer-candle/src/traits.rs:14` | Erzeugt einen Embedding-Vektor für den gegebenen Text. |
| `fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>` | `crates/contextra-infer-candle/src/traits.rs:17` | Erzeugt Embedding-Vektoren für einen Batch von Texten. |
| `fn dimension(&self) -> usize` | `crates/contextra-infer-candle/src/traits.rs:26` | Gibt die Dimension der erzeugten Einbettungen zurück. |

#### `trait KvStateSink`
- Ort: `crates/contextra-infer-candle/src/traits.rs:63`
```rust
pub trait KvStateSink: Send + Sync
```
Dokumentation: Lokaler Trait für KV-State-Senke (Export von Attention- oder KV-Zuständen).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn store_segment(&mut self, segment_id: &[u8], payload: &[u8]) -> Result<()>` | `crates/contextra-infer-candle/src/traits.rs:65` | Speichert ein KV-Zustandssegment unter einem opaken Schlüssel. |

#### `trait KvStateSource`
- Ort: `crates/contextra-infer-candle/src/traits.rs:69`
```rust
pub trait KvStateSource: Send + Sync
```
Dokumentation: Lokaler Trait für KV-State-Quelle (Wiederherstellung von Attention- oder KV-Zuständen).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn load_segment(&self, segment_id: &[u8]) -> Result<Option<Vec<u8>>>` | `crates/contextra-infer-candle/src/traits.rs:71` | Liest ein KV-Zustandssegment anhand des opaken Schlüssels. |

#### `trait TextGenerator`
- Ort: `crates/contextra-infer-candle/src/traits.rs:57`
```rust
pub trait TextGenerator: Send + Sync
```
Dokumentation: Lokaler Trait für synchrone Textgenerierung.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn generate(&self, prompt: &str, config: &GenerationConfig) -> Result<String>` | `crates/contextra-infer-candle/src/traits.rs:59` | Generiert Text auf Basis des angegebenen Prompts und Konfiguration. |

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl From<CandleError> for contextra_types::ContextraError` — `crates/contextra-infer-candle/src/compat/error.rs:6` (0 pub Methoden)
- `impl From<contextra_types::ModelFingerprint> for LocalModelFingerprint` — `crates/contextra-infer-candle/src/compat/fingerprint.rs:15` (0 pub Methoden)
- `impl From<&ContextChunk> for LocalContextChunk` — `crates/contextra-infer-candle/src/compat/grounding.rs:9` (0 pub Methoden)
- `impl From<LocalKvLayout> for PortsKvLayout` — `crates/contextra-infer-candle/src/compat/kv.rs:18` (0 pub Methoden)
- `impl From<PortsKvLayout> for LocalKvLayout` — `crates/contextra-infer-candle/src/compat/kv.rs:29` (0 pub Methoden)
- `impl From<LocalRopeConfig> for PortsRopeConfig` — `crates/contextra-infer-candle/src/compat/kv.rs:40` (0 pub Methoden)
- `impl From<PortsRopeConfig> for LocalRopeConfig` — `crates/contextra-infer-candle/src/compat/kv.rs:49` (0 pub Methoden)
- `impl From<LocalKvBlock> for PortsKvBlock` — `crates/contextra-infer-candle/src/compat/kv.rs:58` (0 pub Methoden)
- `impl From<PortsKvBlock> for LocalKvBlock` — `crates/contextra-infer-candle/src/compat/kv.rs:67` (0 pub Methoden)
