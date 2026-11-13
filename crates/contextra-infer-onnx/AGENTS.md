# AGENTS.md — `contextra-infer-onnx`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-infer-onnx` · **Ring:** 2 · **Version:** 0.1.0
- **Beschreibung:** Autonomous in-process ONNX inference engine for embeddings and cross-encoder reranking (Klasse A, Ring 2)
- **src:** 9 Dateien, 744 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 2):** `contextra-ports`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 11):** `blake3`, `ndarray`, `ort`, `parking_lot`, `reqwest`, `serde`, `serde_json`, `sha2`, `thiserror`, `tokenizers`, `tracing`
- **Reverse-Abhängigkeiten (2):** `contextra-db`, `contextra-mcp`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend `contextra-db`
- **Features (9):** `default`, `download`, `onnx`, `reranking`, `serde`, `std`, `test-utils`, `tracing`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (6): `EmbedderConfig`, `FakeEmbedder`, `FakeReranker`, `OnnxEmbedder`, `OnnxReranker`, `RerankerConfig`
- enum (4): `CrossEncoderReranker`, `Error`, `ModelHash`, `PoolingMode`
- trait (2): `Embedder`, `Reranker`
- fn (8): `config`, `default_model_cache_dir`, `from_config`, `new`, `passthrough`, `validate`, `verify_file`, `verify_path_safety`
- type (1): `Result`
- mod (8): `compat`, `config`, `embedder`, `embedding_provider`, `error`, `fakes`, `reranker`, `traits`
- pub use: `config::{ verify_path_safety, EmbedderConfig, ModelHash, PoolingMode, RerankerConfig, }`, `embedder::OnnxEmbedder`, `error::{Error, Result}`, `fakes::{FakeEmbedder, FakeReranker}`, `reranker::{CrossEncoderReranker, OnnxReranker}`, `traits::{Embedder, Reranker}`

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->`.

<!-- END GENERATED: crate_inventory.py -->
