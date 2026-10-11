# Contextra — Globaler Arbeitsstand (WORKING_STATE.md)

> **Stand:** 2026-10-06 | **Format:** v2 (Automatisierte Generierung via `cargo xtask sync-docs`)

## 1. Systemstatus & Architektur
Contextra ist eine hochperformante, einbettbare Vektor- und Graph-Engine für Rust.

## 2. Session-Kontinuität
- **Standard-Session-ID:** `12861055699283359246` (automatische Kontinuität, falls kein Override per Flag)
- **Offene contextra_tags (DEBT/ANCHOR/AI-TAG):** 95 (siehe Tabelle unten)
- **Aktive Task-Claims (.jules/claims.toml):** 0

## 3. Crate-Inventar & Ringe
| Crate | Pfad | Layer | LOC | Status | Ring | Reife | Beschreibung |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `contextra-wire` | `crates/contextra-wire` | 0 | 1.5k | 🟢 stable | `Ring 0` | `stable` | FlatBuffers Wire-Protokolle und Serialisierungs-Formate |
| `contextra-sys` | `crates/contextra-sys` | 0 | 632 | 🟢 stable | `Ring 0` | `stable` | Systemnahe Bindings, Low-Level I/O und Memory-Mapping |
| `contextra-simd` | `crates/contextra-simd` | 4 | 1.2k | 🟢 stable | `Ring 0` | `stable` | SIMD-beschleunigte Distanzberechnungen und Vektor-Operationen |
| `contextra-core` | `crates/contextra-core` | 3 | 122 | 🟢 stable | `Ring 0` | `stable` | Kern-Datenstrukturen, Tombstones, Invarianten und Memory-Buffer |
| `contextra-store` | `crates/contextra-store` | 4 | 18.6k | 🟢 stable | `Ring 1` | `stable` | LSM-Tree Storage Engine, WAL Durability und Compaction Engine |
| `contextra-vector` | `crates/contextra-vector` | 5 | 13.6k | 🟢 stable | `Ring 0` | `stable` | Vektorsuche und Indexierung (HNSW, DiskANN, Quantisierung) |
| `contextra-db` | `crates/contextra-db` | 8 | 2.4k | 🟢 stable | `Ring 3` | `stable` | Contextra Main Database Abstraction, Hybrid Search & Collections |
| `contextra-text` | `crates/contextra-text` | 2 | 4.5k | 🟢 stable | `Ring 0` | `stable` | Volltextsuche und Inverted-Index (BM25 Engine) |
| `contextra-checkpoint` | `crates/contextra-checkpoint` | 4 | 4.0k | 🟢 stable | `Ring 1` | `stable` | Persistent Checkpoint Engine und Blake3 Manifest-Verifikation |
| `contextra-crypto` | `crates/contextra-crypto` | 2 | 8.1k | 🟢 stable | `Ring 0` | `stable` | Kryptographische Vaults, Zeroize, Egress-Verschlüsselung und HMAC |
| `contextra-privacy` | `crates/contextra-privacy` | 2 | 3.3k | 🟢 stable | `Ring 3` | `stable` | Egress Filtering, Anonymisierung und Privacy Compliance Rules |
| `contextra-graph` | `crates/contextra-graph` | 3 | 14.3k | 🟢 stable | `Ring 0` | `stable` | CSR Graph-Engine, Path Graphing und GraphRAG Community Detection |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx` | 4 | 1.8k | 🟡 experimental | `Ring 2` | `experimental` | ONNX Embeddings und Cross-Encoder Reranking Execution Provider |
| `contextra-mcp` | `crates/contextra-mcp` | 10 | 6.8k | 🟢 stable | `Ring 4` | `stable` | Model Context Protocol Server & Cloud Egress Gateway |
| `contextra-agent` | `crates/contextra-agent` | 9 | 4.5k | 🟢 stable | `Ring 3` | `stable` | Audit Engine, InMemoryStorageEngine und Agent Execution Pipeline |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama` | 3 | 4.5k | 🟢 stable | `Ring 2` | `stable` | Ollama External Provider Client und Embeddings Integration |
| `contextra-router` | `crates/contextra-router` | 3 | 3.2k | 🟢 stable | `Ring 3` | `stable` | Lyapunov Drift Control und Multi-Candidate Search Routing Engine |
| `contextra-bench` | `benchmarks/contextra-bench` | 9 | 6.4k | 🔴 unklassifiziert | `unklassifiziert` | `unklassifiziert` | Contextra — Reproducible Benchmark Harness for Retrieval Accuracy |
| `contextra-infer-candle` | `crates/contextra-infer-candle` | 3 | 4.6k | 🟢 stable | `Ring 2` | `stable` | Candle LLM Inference Client und KV-Bridge Adapter |
| `contextra-sandbox` | `crates/contextra-sandbox` | 0 | 1.9k | 🟢 stable | `Ring 2` | `stable` | WASM Execution Boundary und Isolations-Sandbox |
| `contextra-py` | `crates/contextra-py` | 9 | 2.2k | 🟢 stable | `Ring 4` | `stable` | PyO3 Python Bindings |
| `contextra-testkit` | `crates/contextra-testkit` | 2 | 675 | 🟢 stable | `Tooling` | `stable` | Test Fixtures, Mock Engines und Chaos Matrix Harness |
| `contextra` | `crates/contextra` | 9 | 1.1k | 🟢 stable | `Ring 4` | `stable` | Haupt-Library Facade für Endanwender |
| `contextra-types` | `crates/contextra-types` | 0 | 5.5k | 🟢 stable | `Ring 0` | `stable` | Grundlegende Typdefinitionen (DocId, TxId, TenantId, Error-Typen) |
| `contextra-ports` | `crates/contextra-ports` | 1 | 3.2k | 🟢 stable | `Ring 0` | `stable` | Abstrakte Port-Schnittstellen und Trait-Definitionen für Entkopplung |
| `contextra-mvcc` | `crates/contextra-mvcc` | 2 | 3.2k | 🟢 stable | `Ring 0` | `stable` | Multi-Version Concurrency Control und Isolation-Mechanismen |
| `contextra-adapt` | `crates/contextra-adapt` | 2 | 4.3k | 🟡 experimental | `Ring 0` | `experimental` | Anpassungs- und Transformations-Layer für Datenmodelle (PID Latency Controller) |
| `contextra-kvcache` | `crates/contextra-kvcache` | 3 | 4.1k | 🟢 stable | `Ring 1` | `stable` | Stufenmodell KV-Cache Storage & Offloading |
| `contextra-engine` | `crates/contextra-engine` | 6 | 14.2k | 🟢 stable | `Ring 3` | `stable` | Compute Pool und Asynchrone Storage-Execution Layer (ADR-N02) |
| `contextra-cognition` | `crates/contextra-cognition` | 7 | 5.6k | 🟢 stable | `Ring 3` | `stable` | Konsolidierungs- und Background Sleep Passes |
| `contextra-rank` | `crates/contextra-rank` | 2 | 3.6k | 🟢 stable | `Ring 0` | `stable` | Ranking-, Fusion- und Score-Kalibrierungs-Algorithmen (RRF, Isotonische/Platt Kalibrierung, Bonus Scorers) |
| `contextra-audit-export` | `crates/contextra-audit-export` | 2 | 473 | 🟡 experimental | `Ring 0` | `experimental` | Audit Log Export und DSGVO Art. 30 Verarbeitungsverzeichnis-Export |
| `contextra-avv-generator` | `crates/contextra-avv-generator` | 1 | 315 | 🟡 experimental | `Ring 0` | `experimental` | AVV (Auftragsverarbeitungsvertrag) Template Generator gemäß Art. 28 DSGVO |
| `contextra-license` | `crates/contextra-license` | 2 | 406 | 🟡 experimental | `Ring 4` | `experimental` | Lizenz- und Aktivierungsprüfung für Contextra Feature-Ringe |
| `xtask-heavy` | `xtask-heavy` | 10 | 244 | 🔴 unklassifiziert | `unklassifiziert` | `unklassifiziert` | unklassifiziert |

## 4. Invarianten & Qualitäts-Gates
| Invariante / Gate | Beschreibung | Ziel-Ebene | status |
| :--- | :--- | :--- | :--- |
| **Zero-Panic (P7)** | Kein unwrap/expect in Produktionscode | Ring 0, Ring 1 | 🟢 Aktiv |
| **Ring-Layering** | Strikter gerichteter Azyklischer Graph | Alle Crates | 🟢 Aktiv |
| **Async Purity** | Keine Async-Blockaden in Ring 0 | Ring 0 | 🟢 Aktiv |
| **Determinismus** | Reproduzierbarkeit via Port-Injektion | Ring 0 | 🟢 Aktiv |

## 5. DAG & Topologie
### Layer 0 (Rang 0)
- **`contextra-wire`** (Ring: `Ring 0`) -> Abhaengigkeiten: keine
- **`contextra-sys`** (Ring: `Ring 0`) -> Abhaengigkeiten: keine
- **`contextra-sandbox`** (Ring: `Ring 2`) -> Abhaengigkeiten: keine
- **`contextra-types`** (Ring: `Ring 0`) -> Abhaengigkeiten: keine

### Layer 1 (Rang 1)
- **`contextra-ports`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-types
- **`contextra-avv-generator`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-types

### Layer 2 (Rang 2)
- **`contextra-text`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-crypto`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-privacy`** (Ring: `Ring 3`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-testkit`** (Ring: `Tooling`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-mvcc`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-adapt`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-rank`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-audit-export`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-license`** (Ring: `Ring 4`) -> Abhaengigkeiten: contextra-ports, contextra-types

### Layer 3 (Rang 3)
- **`contextra-core`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-mvcc, contextra-ports, contextra-types, contextra-wire
- **`contextra-graph`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-adapt, contextra-ports, contextra-types
- **`contextra-infer-ollama`** (Ring: `Ring 2`) -> Abhaengigkeiten: contextra-ports, contextra-rank, contextra-types
- **`contextra-router`** (Ring: `Ring 3`) -> Abhaengigkeiten: contextra-adapt, contextra-ports, contextra-privacy, contextra-types, contextra-wire
- **`contextra-infer-candle`** (Ring: `Ring 2`) -> Abhaengigkeiten: contextra-crypto, contextra-ports, contextra-rank, contextra-types
- **`contextra-kvcache`** (Ring: `Ring 1`) -> Abhaengigkeiten: contextra-crypto, contextra-ports, contextra-types

### Layer 4 (Rang 4)
- **`contextra-simd`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-core
- **`contextra-store`** (Ring: `Ring 1`) -> Abhaengigkeiten: contextra-core, contextra-crypto, contextra-mvcc, contextra-ports, contextra-sys, contextra-types
- **`contextra-checkpoint`** (Ring: `Ring 1`) -> Abhaengigkeiten: contextra-core, contextra-ports, contextra-types
- **`contextra-infer-onnx`** (Ring: `Ring 2`) -> Abhaengigkeiten: contextra-infer-candle, contextra-ports, contextra-rank, contextra-types

### Layer 5 (Rang 5)
- **`contextra-vector`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-core, contextra-crypto, contextra-ports, contextra-simd, contextra-sys, contextra-types

### Layer 6 (Rang 6)
- **`contextra-engine`** (Ring: `Ring 3`) -> Abhaengigkeiten: contextra-adapt, contextra-checkpoint, contextra-crypto, contextra-graph, contextra-infer-candle, contextra-kvcache, contextra-mvcc, contextra-ports, contextra-rank, contextra-sandbox, contextra-store, contextra-sys, contextra-text, contextra-types, contextra-vector

### Layer 7 (Rang 7)
- **`contextra-cognition`** (Ring: `Ring 3`) -> Abhaengigkeiten: contextra-engine, contextra-graph, contextra-ports, contextra-store, contextra-types, contextra-vector

### Layer 8 (Rang 8)
- **`contextra-db`** (Ring: `Ring 3`) -> Abhaengigkeiten: contextra-adapt, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-engine, contextra-graph, contextra-ports, contextra-rank, contextra-store, contextra-sys, contextra-text, contextra-types, contextra-vector

### Layer 9 (Rang 9)
- **`contextra-agent`** (Ring: `Ring 3`) -> Abhaengigkeiten: contextra-checkpoint, contextra-db, contextra-graph, contextra-ports, contextra-router, contextra-store, contextra-types
- **`contextra-bench`** (Ring: `unklassifiziert`) -> Abhaengigkeiten: contextra-core, contextra-crypto, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-onnx, contextra-kvcache, contextra-ports, contextra-store, contextra-text, contextra-types, contextra-vector
- **`contextra-py`** (Ring: `Ring 4`) -> Abhaengigkeiten: contextra-adapt, contextra-core, contextra-db, contextra-rank, contextra-router, contextra-store, contextra-types
- **`contextra`** (Ring: `Ring 4`) -> Abhaengigkeiten: contextra-audit-export, contextra-avv-generator, contextra-core, contextra-crypto, contextra-db, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-license, contextra-ports, contextra-privacy, contextra-rank, contextra-router, contextra-store, contextra-types

### Layer 10 (Rang 10)
- **`contextra-mcp`** (Ring: `Ring 4`) -> Abhaengigkeiten: contextra-adapt, contextra-agent, contextra-crypto, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-license, contextra-ports, contextra-privacy, contextra-rank, contextra-types, contextra-wire
- **`xtask-heavy`** (Ring: `unklassifiziert`) -> Abhaengigkeiten: contextra-bench, contextra-router


## 6. KI-Tags & Schulden-Register
| Datei | Zeile | Typ | Schweregrad | Beschreibung | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `docs/CHANGELOG.md` | 313 | `AI-TAG` | **CRITICAL** | RESOLVED: Validates non-empty input parameters for agent workflow context initialization. (TS:2026-08-30T15:00:19Z) (SESSION: 283abf0f) | | OPEN |
| `docs/CHANGELOG.md` | 314 | `AI-TAG` | **CRITICAL** | RESOLVED: Validates non-empty event source to prevent silent telemetry attribution loss. (TS:2026-08-30T15:00:19Z) (SESSION: 283abf0f) | | OPEN |
| `docs/CHANGELOG.md` | 315 | `AI-TAG` | **CRITICAL** | RESOLVED: Enforces bounded event queue capacity to guard against unbounded memory growth. (TS:2026-08-30T15:00:19Z) (SESSION: 283abf0f) | | OPEN |
| `docs/CHANGELOG.md` | 316 | `AI-TAG` | **CRITICAL** | RESOLVED: Validates non-empty Node ID and description for graph nodes. (TS:2026-08-30T15:00:19Z) (SESSION: 283abf0f) | | OPEN |
| `docs/CHANGELOG.md` | 317 | `AI-TAG` | **CRITICAL** | RESOLVED: Validates non-empty from/to endpoints for workflow edges. (TS:2026-08-30T15:00:19Z) (SESSION: 283abf0f) | | OPEN |
| `docs/CHANGELOG.md` | 325 | `AI-TAG` | **CRITICAL** | RESOLVED: AGT-DB-005 — relate() rollback race behoben, siehe ADR-023 (TS:2026-08-28T00:00:00Z) | | OPEN |
| `.jules/COMMON_LLM_ERRORS.md` | 147 | `AI-TAG` | **CRITICAL** | Problem in dieser Funktion | OPEN |
| `.jules/COMMON_LLM_ERRORS.md` | 150 | `AI-TAG` | **CRITICAL** | Problem in dieser Funktion | OPEN |
| `crates/contextra-agent/src/event_source.rs` | 34 | `AI-TAG` | **CRITICAL** | RESOLVED: Validates non-empty event source to prevent silent telemetry attribution loss. | OPEN |
| `crates/contextra-agent/src/event_source.rs` | 138 | `AI-TAG` | **CRITICAL** | RESOLVED: Enforces bounded event queue capacity to guard against unbounded memory growth. | OPEN |
| `crates/contextra-agent/src/context.rs` | 109 | `AI-TAG` | **CRITICAL** | RESOLVED: Validates non-empty input parameters for agent workflow context initialization. | OPEN |
| `crates/contextra-agent/src/graph.rs` | 113 | `AI-TAG` | **CRITICAL** | RESOLVED: Validates non-empty Node ID and description for graph nodes. | OPEN |
| `crates/contextra-agent/src/graph.rs` | 188 | `AI-TAG` | **CRITICAL** | RESOLVED: Validates non-empty from/to endpoints for workflow edges. | OPEN |
| `crates/contextra-engine/src/collection/relate.rs` | 6 | `AI-TAG` | **CRITICAL** | RESOLVED: AGT-DB-005 — relate() rollback race behoben, siehe ADR-023 | OPEN |
| `benchmarks/contextra-bench/src/path_rag_sweep.rs` | 1 | `FILE-CONTEXT` | **INFO** | // FILE-CONTEXT | OPEN |
| `benchmarks/contextra-bench/src/path_rag_sweep.rs` | 2 | `STAND` | **INFO** | // STAND: 2026-09-07 | OPEN |
| `benchmarks/contextra-bench/src/path_rag_sweep.rs` | 3 | `ZWECK` | **INFO** | // ZWECK: Parameter-Sweep für PathRAG sufficiency_threshold über LongMemEval und LoCoMo Datensätze. | OPEN |
| `benchmarks/contextra-bench/src/ann_benchmarks.rs` | 1 | `FILE-CONTEXT` | **INFO** | // FILE-CONTEXT | OPEN |
| `benchmarks/contextra-bench/src/ann_benchmarks.rs` | 2 | `STAND` | **INFO** | // STAND: 2026-09-15 | OPEN |
| `benchmarks/contextra-bench/src/ann_benchmarks.rs` | 3 | `ZWECK` | **INFO** | // ZWECK: ANN-Benchmarks kompatible Evaluation für Contextra HNSW-Index. | OPEN |
| `benchmarks/contextra-bench/src/compare.rs` | 1 | `FILE-CONTEXT` | **INFO** | // FILE-CONTEXT | OPEN |
| `benchmarks/contextra-bench/src/compare.rs` | 2 | `STAND` | **INFO** | // STAND: 2026-09-07 | OPEN |
| `benchmarks/contextra-bench/src/compare.rs` | 3 | `ZWECK` | **INFO** | // ZWECK: Baseline-Vergleichs-Engine für Retrieval-Benchmark-Metriken (LongMemEval & LoCoMo) | OPEN |
| `benchmarks/contextra-bench/src/main.rs` | 1 | `FILE-CONTEXT` | **INFO** | // FILE-CONTEXT | OPEN |
| `benchmarks/contextra-bench/src/main.rs` | 2 | `STAND` | **INFO** | // STAND: 2026-10-02 | OPEN |
| `benchmarks/contextra-bench/src/main.rs` | 3 | `ZWECK` | **INFO** | // ZWECK: Reproduzierbarer Benchmark-Harness für Retrieval-Qualität & LongMemEval / LoCoMo Regressions-Suite mit echten Embeddings | OPEN |
| `benchmarks/contextra-bench/src/locomo.rs` | 1 | `FILE-CONTEXT` | **INFO** | // FILE-CONTEXT | OPEN |
| `benchmarks/contextra-bench/src/locomo.rs` | 2 | `STAND` | **INFO** | // STAND: 2026-10-02 | OPEN |
| `benchmarks/contextra-bench/src/locomo.rs` | 3 | `ZWECK` | **INFO** | // ZWECK: Loader und Evaluation Harness für das LoCoMo Benchmark (SNAP Research / Long Conversational Memory) mit echten Embeddings und Turn-Level Evidence Resolution | OPEN |
| `benchmarks/contextra-bench/src/beir_eval.rs` | 1 | `FILE-CONTEXT` | **INFO** | // FILE-CONTEXT | OPEN |

## 7. Chronik & Modifikations-Historie
| Datum | Komponente | Beschreibung | Tag-ID | Session |
| :--- | :--- | :--- | :--- | :--- |
