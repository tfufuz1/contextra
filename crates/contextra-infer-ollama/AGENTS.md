# AGENTS.md — `contextra-infer-ollama`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-infer-ollama` · **Ring:** 2 · **Version:** 0.1.0
- **Beschreibung:** Autonomous REST/HTTP adapter for Ollama inference servers (Klasse A, Ring 2)
- **src:** 25 Dateien, 5633 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 2):** `contextra-ports`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (2):** `contextra-ports`, `contextra-types`
- **Externe Abhängigkeiten (normal, 10):** `futures`, `futures-util`, `parking_lot`, `regex`, `reqwest`, `serde`, `serde_json`, `thiserror`, `tokio`, `tracing`
- **Reverse-Abhängigkeiten (1):** `contextra-mcp`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend –
- **Features (6):** `client`, `default`, `http`, `ports-compat`, `types-compat`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (17): `CircuitBreaker`, `ContextPrefixConfig`, `ContextPrefixEngine`, `EndpointPolicy`, `ImportanceAssessment`, `LocalImportanceScore`, `LocalQueryRewriteOutput`, `LocalSeededRng`, `ModelInfo`, `NoopEgressAuditSink`, `OllamaClient`, `OllamaConfig`, `OllamaEmbedder`, `OllamaQueryRewriter`, `OllamaQueryRewriterConfig` … (+2)
- enum (3): `BreakerStatus`, `Confidence`, `OllamaError`
- trait (8): `AsyncEmbedder`, `EgressAuditSink`, `Embedder`, `ImportanceCalibrator`, `LocalClock`, `LocalRng`, `PayloadGate`, `TextGenerator`
- fn (69): `as_f32`, `base_url`, `build_rag_prompt`, `build_rewrite_prompt_from_snippets`, `chat_with_rag_streaming`, `check`, `check_availability`, `check_endpoint`, `classify_reqwest_error`, `client`, `compute_max_prefix_chars`, `config`, `embed`, `embed_batch`, `embedding_dimension` … (+54)
- type (3): `BoxFuture`, `ContextPrefixer`, `Result`
- const (6): `DEFAULT_BASE_URL`, `DEFAULT_EMBED_MODEL`, `DEFAULT_MAX_RESPONSE_BYTES`, `MAX_BATCH_SIZE`, `MAX_RETRIES`, `MAX_TEXT_BYTES`
- mod (14): `client`, `compat`, `config`, `context_prefixer`, `embedding`, `error`, `importance`, `model_info`, `policy`, `query_rewriter`, `trait_impls`, `traits`, `types`, `validation`
- pub use: `circuit_breaker::CircuitBreaker`, `client::{ read_response_bytes, read_response_text, CircuitBreaker, EgressAuditSink, EndpointPolicy, NoopEgressAuditSink, OllamaClient, OllamaConfig, PayloadGate, PermissivePayloadGate, DEFAULT_BASE_URL, DEFAULT_EMBED_MODEL, }`, `client::{build_rag_prompt, xml_escape}`, `config::*`, `context_prefixer::{ContextPrefixConfig, ContextPrefixEngine, ContextPrefixer}`, `core::{read_response_bytes, read_response_text, OllamaClient}`, `embedding::OllamaEmbedder`, `error::{ LocalClock, LocalImportanceScore, LocalQueryRewriteOutput, LocalRng, LocalSeededRng, OllamaError, Result, SystemClock, }`, `errors::*`, `importance::{ parse_importance_score_response, Confidence, ImportanceAssessment, ImportanceCalibrator, }` … (+6)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
