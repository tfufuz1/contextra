# AGENTS.md — `contextra-infer-candle`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-infer-candle` · **Ring:** 2 · **Version:** 0.1.0
- **Beschreibung:** Native Candle GGUF ML inference backend for Contextra
- **src:** 30 Dateien, 5595 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 5):** `contextra-crypto`, `contextra-env`, `contextra-ports`, `contextra-rank`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (4):** `contextra-crypto`, `contextra-ports`, `contextra-rank`, `contextra-types`
- **Externe Abhängigkeiten (normal, 14):** `bincode`, `bytes`, `candle-core`, `candle-nn`, `candle-transformers`, `futures-util`, `half`, `parking_lot`, `serde`, `sha2`, `thiserror`, `tokenizers`, `tokio`, `tracing`
- **Reverse-Abhängigkeiten (2):** `contextra`, `contextra-mcp`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend `contextra`
- **Features (8):** `candle`, `cuda`, `default`, `exp-gasp`, `exp-kv-stage-b`, `kv-bridge`, `kv-stage-b`, `test-utils`

### Öffentliche API (grob, aus src/)
- struct (32): `BertEmbedModel`, `CandleAttentionExporter`, `CandleEmbedClient`, `CandleLlmClient`, `ConfigFingerprint`, `ContextChunk`, `DefaultCandleEmbedModel`, `DefaultCandleLlmModel`, `FakeEmbedder`, `FakeTextGenerator`, `GaspConfig`, `GaspValidator`, `GenerationConfig`, `GgufMetadata`, `IdentityCalibrator` … (+17)
- enum (2): `CandleError`, `CandleQuantization`
- trait (7): `CandleEmbedInner`, `CandleModelInner`, `ConfidenceCalibrator`, `Embedder`, `KvStateSink`, `KvStateSource`, `TextGenerator`
- fn (70): `build_prefix_key`, `clear`, `clear_kv_cache`, `combined_text_owned`, `compute_fingerprint`, `compute_raw_grounding_score`, `compute_raw_grounding_score_local`, `config`, `consult_segment`, `consultation_count`, `deterministic`, `embed`, `embed_batch`, `embedding_dim`, `export_block` … (+55)
- type (2): `Error`, `Result`
- const (10): `DEFAULT_GROUNDING_THRESHOLD`, `DEFAULT_MAX_CONCURRENT_EMBEDDINGS`, `DEFAULT_MAX_CONCURRENT_INFERENCES`, `MAX_CANDLE_EMBED_BATCH_SIZE`, `MAX_GGUF_METADATA_DEPTH`, `MAX_GGUF_METADATA_KVS`, `MAX_GGUF_STRING_LEN`, `MAX_GGUF_TENSORS`, `MAX_SEQ_LEN`, `MAX_TRACKED_REQUESTS`
- mod (18): `attention`, `attention_exporter`, `calibrator`, `compat`, `core`, `embedding`, `embedding_provider`, `error`, `fake`, `gasp`, `gguf_loader`, `inference`, `kv_state`, `model`, `model_registry` … (+3)
- pub use: `attention_exporter::*`, `compat::kv_bridge`, `compat::kv_bridge::KvBridgeAdapter`, `core::*`, `crate::compat::embedding::verify_embedding_provider`, `crate::compat::kv::{build_prefix_key, seed_from_hit, KvPrefixContext}`, `embedding::CandleEmbedClient`, `embedding_provider::{verify_embedding_provider, MAX_CANDLE_EMBED_BATCH_SIZE}`, `error::{CandleError, Error, Result}`, `fake::{FakeEmbedder, FakeTextGenerator}` … (+10)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
