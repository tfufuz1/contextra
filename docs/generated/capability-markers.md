# Contextra — Capability & Maturity Markers

> **Hinweis**: Diese Datei ist autogeneriert aus `capabilities.toml` durch `cargo xtask generate-markers` (P12).
> Alle Reifegrad-Marker werden maschinell aus dem Manifest abgeleitet.

| Crate | Ring | Maturity Marker | Capabilities | Beschreibung |
| :--- | :--- | :--- | :--- | :--- |
| `contextra` | Ring 4 | 🟢 stable | `public-api`, `facade` | Haupt-Library Facade für Endanwender |
| `contextra-adapt` | Ring 0 | 🟡 experimental | `model-adaption`, `pid-latency-controller` | Anpassungs- und Transformations-Layer für Datenmodelle (PID Latency Controller) |
| `contextra-agent` | Ring 3 | 🟢 stable | `in-memory-audit`, `agent-pipeline` | Audit Engine, InMemoryStorageEngine und Agent Execution Pipeline |
| `contextra-checkpoint` | Ring 1 | 🟢 stable | `blake3-checkpoints`, `orphan-recovery` | Persistent Checkpoint Engine und Blake3 Manifest-Verifikation |
| `contextra-cognition` | Ring 3 | 🟢 stable | `sleep-cycle-consolidation`, `session-grouping` | Konsolidierungs- und Background Sleep Passes |
| `contextra-core` | Ring 0 | 🟢 stable | `tombstone-semantics`, `tx-buffer` | Kern-Datenstrukturen, Tombstones, Invarianten und Memory-Buffer |
| `contextra-crypto` | Ring 0 | 🟢 stable | `aes-256-gcm`, `hmac-sha256`, `zeroize-on-drop` | Kryptographische Vaults, Zeroize, Egress-Verschlüsselung und HMAC |
| `contextra-db` | Ring 3 | 🟢 stable | `hybrid-search`, `collection-manager`, `volatile-vault` | Contextra Main Database Abstraction, Hybrid Search & Collections |
| `contextra-engine` | Ring 3 | 🟢 stable | `bounded-compute-pool`, `ring0-async-wrapper` | Compute Pool und Asynchrone Storage-Execution Layer (ADR-N02) |
| `contextra-graph` | Ring 0 | 🟢 stable | `csr-traversal`, `label-propagation`, `deadlock-free-dag` | CSR Graph-Engine, Path Graphing und GraphRAG Community Detection |
| `contextra-infer-candle` | Ring 2 | 🟢 stable | `candle-llm`, `kv-cache-bridge` | Candle LLM Inference Client und KV-Bridge Adapter |
| `contextra-infer-ollama` | Ring 2 | 🟢 stable | `ollama-client`, `remote-embeddings` | Ollama External Provider Client und Embeddings Integration |
| `contextra-infer-onnx` | Ring 2 | 🟡 experimental | `onnx-embeddings`, `cross-encoder-reranking` | ONNX Embeddings und Cross-Encoder Reranking Execution Provider |
| `contextra-kvcache` | Ring 1 | 🟢 stable | `kv-tiered-cache`, `kv-bridge-storage` | Stufenmodell KV-Cache Storage & Offloading |
| `contextra-mcp` | Ring 4 | 🟢 stable | `mcp-protocol`, `cloud-egress-gateway` | Model Context Protocol Server & Cloud Egress Gateway |
| `contextra-mvcc` | Ring 0 | 🟢 stable | `snapshot-isolation`, `lockless-read` | Multi-Version Concurrency Control und Isolation-Mechanismen |
| `contextra-ports` | Ring 0 | 🟢 stable | `trait-interfaces`, `ring0-decoupling` | Abstrakte Port-Schnittstellen und Trait-Definitionen für Entkopplung |
| `contextra-privacy` | Ring 3 | 🟢 stable | `egress-vault`, `regex-classification` | Egress Filtering, Anonymisierung und Privacy Compliance Rules |
| `contextra-py` | Ring 4 | 🟢 stable | `python-bindings`, `ffi-safe` | PyO3 Python Bindings |
| `contextra-rank` | Ring 0 | 🟢 stable | `rrf-fusion`, `coherence-bonus`, `platt-scaler`, `isotonic-calibrator` | Ranking-, Fusion- und Score-Kalibrierungs-Algorithmen (RRF, Isotonische/Platt Kalibrierung, Bonus Scorers) |
| `contextra-router` | Ring 3 | 🟢 stable | `lyapunov-drift-watcher`, `cascade-routing` | Lyapunov Drift Control und Multi-Candidate Search Routing Engine |
| `contextra-sandbox` | Ring 2 | 🟢 stable | `wasmtime-isolation`, `cpu-fuel-budget`, `wall-clock-timeout` | WASM Execution Boundary und Isolations-Sandbox |
| `contextra-simd` | Ring 0 | 🟢 stable | `simd-distance`, `zero-panic-simd` | SIMD-beschleunigte Distanzberechnungen und Vektor-Operationen |
| `contextra-store` | Ring 1 | 🟢 stable | `wal-v3-hmac`, `lsm-compaction`, `crash-consistency` | LSM-Tree Storage Engine, WAL Durability und Compaction Engine |
| `contextra-sys` | Ring 0 | 🟢 stable | `mmap-io`, `direct-file-access` | Systemnahe Bindings, Low-Level I/O und Memory-Mapping |
| `contextra-testkit` | Tooling | 🟢 stable | `chaos-testing`, `mock-storage` | Test Fixtures, Mock Engines und Chaos Matrix Harness |
| `contextra-text` | Ring 0 | 🟢 stable | `bm25-search`, `inverted-index` | Volltextsuche und Inverted-Index (BM25 Engine) |
| `contextra-types` | Ring 0 | 🟢 stable | `zero-copy-types`, `type-safety` | Grundlegende Typdefinitionen (DocId, TxId, TenantId, Error-Typen) |
| `contextra-vector` | Ring 0 | 🟢 stable | `hnsw-index`, `diskann-index`, `scalar-quantization` | Vektorsuche und Indexierung (HNSW, DiskANN, Quantisierung) |
| `contextra-wire` | Ring 0 | 🟢 stable | `flatbuffers-wire`, `zero-copy-deserialization` | FlatBuffers Wire-Protokolle und Serialisierungs-Formate |
