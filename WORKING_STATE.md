# Contextra — Globaler Arbeitsstand (WORKING_STATE.md)

> **Stand:** 2026-10-15 | **Format:** v2 (Automatisierte Generierung via `cargo xtask sync-docs`)

## 1. Systemstatus & Architektur
Contextra ist eine hochperformante, einbettbare Vektor- und Graph-Engine für Rust.

## 2. Session-Kontinuität
- **Standard-Session-ID:** `16874557851389833557` (automatische Kontinuität, falls kein Override per Flag)
- **Offene contextra_tags (DEBT/ANCHOR/AI-TAG):** 7 (siehe Tabelle unten)
- **Aktive Task-Claims (.jules/claims.toml):** 0

## 3. Crate-Inventar & Ringe
| Crate | Pfad | Layer | LOC | Status | Ring | Reife | Beschreibung |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `contextra-wire` | `crates/contextra-wire` | 0 | 1.5k | 🟢 stable | `Ring 0` | `stable` | FlatBuffers Wire-Protokolle und Serialisierungs-Formate |
| `contextra-sys` | `crates/contextra-sys` | 0 | 2.5k | 🟢 stable | `Ring 0` | `stable` | Systemnahe Bindings, Low-Level I/O und Memory-Mapping |
| `contextra-durable-fs` | `crates/contextra-durable-fs` | 1 | 1.0k | 🟢 stable | `Ring 0` | `stable` | Durable Filesystem Operations und synchrone Disk-Flushes |
| `contextra-simd` | `crates/contextra-simd` | 4 | 2.6k | 🟢 stable | `Ring 0` | `stable` | SIMD-beschleunigte Distanzberechnungen und Vektor-Operationen |
| `contextra-core` | `crates/contextra-core` | 3 | 123 | 🟢 stable | `Ring 0` | `stable` | Kern-Datenstrukturen, Tombstones, Invarianten und Memory-Buffer |
| `contextra-store` | `crates/contextra-store` | 4 | 26.8k | 🟢 stable | `Ring 1` | `stable` | LSM-Tree Storage Engine, WAL Durability und Compaction Engine |
| `contextra-vector` | `crates/contextra-vector` | 5 | 15.2k | 🟢 stable | `Ring 0` | `stable` | Vektorsuche und Indexierung (HNSW, DiskANN, Quantisierung) |
| `contextra-db` | `crates/contextra-db` | 7 | 2.8k | 🟢 stable | `Ring 3` | `stable` | Contextra Main Database Abstraction, Hybrid Search & Collections |
| `contextra-text` | `crates/contextra-text` | 2 | 4.9k | 🟢 stable | `Ring 0` | `stable` | Volltextsuche und Inverted-Index (BM25 Engine) |
| `contextra-checkpoint` | `crates/contextra-checkpoint` | 4 | 4.6k | 🟢 stable | `Ring 1` | `stable` | Persistent Checkpoint Engine und Blake3 Manifest-Verifikation |
| `contextra-crypto` | `crates/contextra-crypto` | 2 | 14.4k | 🟢 stable | `Ring 0` | `stable` | Kryptographische Vaults, Zeroize, Egress-Verschlüsselung und HMAC |
| `contextra-privacy` | `crates/contextra-privacy` | 2 | 3.4k | 🟢 stable | `Ring 3` | `stable` | Egress Filtering, Anonymisierung und Privacy Compliance Rules |
| `contextra-graph` | `crates/contextra-graph` | 3 | 14.5k | 🟢 stable | `Ring 0` | `stable` | CSR Graph-Engine, Path Graphing und GraphRAG Community Detection |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx` | 4 | 1.9k | 🟡 experimental | `Ring 2` | `experimental` | ONNX Embeddings und Cross-Encoder Reranking Execution Provider |
| `contextra-mcp` | `crates/contextra-mcp` | 9 | 9.2k | 🟢 stable | `Ring 4` | `stable` | Model Context Protocol Server & Cloud Egress Gateway |
| `contextra-agent` | `crates/contextra-agent` | 8 | 4.7k | 🟡 experimental | `Ring 3` | `experimental` | Audit Engine, InMemoryStorageEngine und Agent Execution Pipeline |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama` | 3 | 4.5k | 🟢 stable | `Ring 2` | `stable` | Ollama External Provider Client und Embeddings Integration |
| `contextra-router` | `crates/contextra-router` | 3 | 3.3k | 🟡 experimental | `Ring 3` | `experimental` | Lyapunov Drift Control und Multi-Candidate Search Routing Engine |
| `contextra-bench` | `benchmarks/contextra-bench` | 8 | 6.6k | 🟢 stable | `Tooling` | `stable` | Contextra Reproducible Benchmark Harness for Retrieval Accuracy |
| `contextra-infer-candle` | `crates/contextra-infer-candle` | 3 | 4.9k | 🟡 experimental | `Ring 2` | `experimental` | Candle LLM Inference Client und KV-Bridge Adapter |
| `contextra-sandbox` | `crates/contextra-sandbox` | 0 | 2.7k | 🟢 stable | `Ring 2` | `stable` | WASM Execution Boundary und Isolations-Sandbox |
| `contextra-py` | `crates/contextra-py` | 8 | 2.5k | 🟢 stable | `Ring 4` | `stable` | PyO3 Python Bindings |
| `contextra-testkit` | `crates/contextra-testkit` | 2 | 3.3k | 🟢 stable | `Tooling` | `stable` | Test Fixtures, Mock Engines und Chaos Matrix Harness |
| `contextra` | `crates/contextra` | 9 | 2.1k | 🟢 stable | `Ring 4` | `stable` | Haupt-Library Facade für Endanwender |
| `contextra-types` | `crates/contextra-types` | 0 | 5.6k | 🟢 stable | `Ring 0` | `stable` | Grundlegende Typdefinitionen (DocId, TxId, TenantId, Error-Typen) |
| `contextra-ports` | `crates/contextra-ports` | 1 | 4.2k | 🟢 stable | `Ring 0` | `stable` | Abstrakte Port-Schnittstellen und Trait-Definitionen für Entkopplung |
| `contextra-mvcc` | `crates/contextra-mvcc` | 2 | 3.6k | 🟢 stable | `Ring 0` | `stable` | Multi-Version Concurrency Control und Isolation-Mechanismen |
| `contextra-adapt` | `crates/contextra-adapt` | 2 | 4.5k | 🟡 experimental | `Ring 0` | `experimental` | Anpassungs- und Transformations-Layer für Datenmodelle (PID Latency Controller) |
| `contextra-kvcache` | `crates/contextra-kvcache` | 3 | 4.4k | 🟡 experimental | `Ring 1` | `experimental` | Stufenmodell KV-Cache Storage & Offloading |
| `contextra-engine` | `crates/contextra-engine` | 6 | 18.5k | 🟢 stable | `Ring 3` | `stable` | Compute Pool und Asynchrone Storage-Execution Layer (ADR-N02) |
| `contextra-cognition` | `crates/contextra-cognition` | 7 | 5.8k | 🟡 experimental | `Ring 3` | `experimental` | Konsolidierungs- und Background Sleep Passes |
| `contextra-rank` | `crates/contextra-rank` | 2 | 3.9k | 🟢 stable | `Ring 0` | `stable` | Ranking-, Fusion- und Score-Kalibrierungs-Algorithmen (RRF, Isotonische/Platt Kalibrierung, Bonus Scorers) |
| `contextra-audit-export` | `crates/contextra-audit-export` | 3 | 716 | 🟡 experimental | `Ring 0` | `experimental` | Audit Log Export und DSGVO Art. 30 Verarbeitungsverzeichnis-Export |
| `contextra-avv-generator` | `crates/contextra-avv-generator` | 2 | 390 | 🟡 experimental | `Ring 0` | `experimental` | AVV (Auftragsverarbeitungsvertrag) Template Generator gemäß Art. 28 DSGVO |
| `contextra-license` | `crates/contextra-license` | 2 | 669 | 🟡 experimental | `Ring 4` | `experimental` | Lizenz- und Aktivierungsprüfung für Contextra Feature-Ringe |
| `xtask-heavy` | `xtask-heavy` | 9 | 250 | 🟡 experimental | `Tooling` | `experimental` | Heavy Xtask Integration Benchmark und Long-Running Heavy Verification Tasks |

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
- **`contextra-durable-fs`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-sys
- **`contextra-ports`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-types

### Layer 2 (Rang 2)
- **`contextra-text`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-crypto`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-durable-fs, contextra-ports, contextra-types
- **`contextra-privacy`** (Ring: `Ring 3`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-testkit`** (Ring: `Tooling`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-mvcc`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-adapt`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-rank`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-avv-generator`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-ports, contextra-types
- **`contextra-license`** (Ring: `Ring 4`) -> Abhaengigkeiten: contextra-ports, contextra-types

### Layer 3 (Rang 3)
- **`contextra-core`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-mvcc, contextra-ports, contextra-types, contextra-wire
- **`contextra-graph`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-adapt, contextra-ports, contextra-types
- **`contextra-infer-ollama`** (Ring: `Ring 2`) -> Abhaengigkeiten: contextra-ports, contextra-rank, contextra-types
- **`contextra-router`** (Ring: `Ring 3`) -> Abhaengigkeiten: contextra-adapt, contextra-ports, contextra-privacy, contextra-types, contextra-wire
- **`contextra-infer-candle`** (Ring: `Ring 2`) -> Abhaengigkeiten: contextra-crypto, contextra-ports, contextra-rank, contextra-types
- **`contextra-kvcache`** (Ring: `Ring 1`) -> Abhaengigkeiten: contextra-crypto, contextra-ports, contextra-types
- **`contextra-audit-export`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-crypto, contextra-ports, contextra-types

### Layer 4 (Rang 4)
- **`contextra-simd`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-core
- **`contextra-store`** (Ring: `Ring 1`) -> Abhaengigkeiten: contextra-core, contextra-crypto, contextra-durable-fs, contextra-mvcc, contextra-ports, contextra-sys, contextra-types
- **`contextra-checkpoint`** (Ring: `Ring 1`) -> Abhaengigkeiten: contextra-core, contextra-ports, contextra-types
- **`contextra-infer-onnx`** (Ring: `Ring 2`) -> Abhaengigkeiten: contextra-infer-candle, contextra-ports, contextra-rank, contextra-types

### Layer 5 (Rang 5)
- **`contextra-vector`** (Ring: `Ring 0`) -> Abhaengigkeiten: contextra-core, contextra-crypto, contextra-durable-fs, contextra-ports, contextra-simd, contextra-sys, contextra-types

### Layer 6 (Rang 6)
- **`contextra-engine`** (Ring: `Ring 3`) -> Abhaengigkeiten: contextra-adapt, contextra-checkpoint, contextra-crypto, contextra-graph, contextra-infer-candle, contextra-kvcache, contextra-license, contextra-mvcc, contextra-ports, contextra-rank, contextra-sandbox, contextra-store, contextra-sys, contextra-text, contextra-types, contextra-vector

### Layer 7 (Rang 7)
- **`contextra-db`** (Ring: `Ring 3`) -> Abhaengigkeiten: contextra-adapt, contextra-checkpoint, contextra-crypto, contextra-engine, contextra-graph, contextra-ports, contextra-rank, contextra-store, contextra-sys, contextra-text, contextra-types, contextra-vector
- **`contextra-cognition`** (Ring: `Ring 3`) -> Abhaengigkeiten: contextra-engine, contextra-graph, contextra-ports, contextra-store, contextra-types, contextra-vector

### Layer 8 (Rang 8)
- **`contextra-agent`** (Ring: `Ring 3`) -> Abhaengigkeiten: contextra-checkpoint, contextra-db, contextra-graph, contextra-ports, contextra-router, contextra-store, contextra-types
- **`contextra-bench`** (Ring: `Tooling`) -> Abhaengigkeiten: contextra-core, contextra-crypto, contextra-db, contextra-graph, contextra-infer-candle, contextra-infer-onnx, contextra-kvcache, contextra-ports, contextra-store, contextra-text, contextra-types, contextra-vector
- **`contextra-py`** (Ring: `Ring 4`) -> Abhaengigkeiten: contextra-adapt, contextra-core, contextra-db, contextra-rank, contextra-router, contextra-store, contextra-types

### Layer 9 (Rang 9)
- **`contextra-mcp`** (Ring: `Ring 4`) -> Abhaengigkeiten: contextra-adapt, contextra-agent, contextra-crypto, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-license, contextra-ports, contextra-privacy, contextra-rank, contextra-types, contextra-wire
- **`contextra`** (Ring: `Ring 4`) -> Abhaengigkeiten: contextra-adapt, contextra-agent, contextra-cognition, contextra-core, contextra-crypto, contextra-db, contextra-engine, contextra-graph, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-license, contextra-ports, contextra-privacy, contextra-rank, contextra-router, contextra-sandbox, contextra-store, contextra-text, contextra-types
- **`xtask-heavy`** (Ring: `Tooling`) -> Abhaengigkeiten: contextra-bench, contextra-router


## 6. KI-Tags & Schulden-Register
| Datei | Zeile | Typ | Schweregrad | Beschreibung | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `xtask/src/check_stale_tags.rs` | 376 | `AI-TAG` | **MINOR** | TODO(audit-C-1): Fix startup flush. (ID: AGT-STORE-bae66245) (TS: 2026-09-10T19:14:58Z) (SESSION: 21a8d3e8) | OPEN |
| `xtask/src/check_stale_tags.rs` | 378 | `AI-TAG` | **MINOR** | Non-audit tag (TS: 2026-09-10T19:14:58Z) | OPEN |
| `xtask/src/check_stale_tags.rs` | 409 | `AI-TAG` | **MINOR** | TODO(audit-1.1): Old stale audit tag (ID: AGT-TEST-00000001) (TS: 2026-01-01T00:00:00Z) (SESSION: 00000001) | OPEN |
| `xtask/src/check_stale_tags.rs` | 410 | `AI-TAG` | **MINOR** | TODO(audit-1.2): Recent audit tag (ID: AGT-TEST-00000002) (TS: 2026-09-10T00:00:00Z) (SESSION: 00000002) | OPEN |
| `crates/contextra-db/tests/split_brain_vector_orphan_test.rs` | 2 | `ANCHOR` | **INFO** | STATUS:IN_PROGRESS (TS:2026-09-09T22:00:00Z) (SESSION: 504d02fc) | IN_PROGRESS |
| `crates/contextra-store/src/compaction/engine.rs` | 717 | `AI-TAG` | **MINOR** | BOUND_UNIFICATION: Standardize boundary check (ID: AGT-STORE-c7e14a2b) (TS: 2026-10-12T17:20:27Z) (SESSION: a832a6e7) | OPEN |
| `benches/migration_benchmarks.rs` | 1 | `ANCHOR` | **INFO** | STATUS:PARTIAL (TS:2026-08-29T00:00:00Z) — Benchmark Suite für LangGraph Migration | PARTIAL |

## 7. Chronik & Modifikations-Historie
| Datum | Komponente | Beschreibung | Tag-ID | Session |
| :--- | :--- | :--- | :--- | :--- |
| TS:2026-10 | `crates/contextra-kvcache/src/wiring.rs` | // STAND: TS:2026-10-13T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-kvcache/tests/two_tenant_isolation_and_segment_validation.rs` | // STAND: TS:2026-10-13T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-agent/src/wiring.rs` | // STAND: TS:2026-10-13T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-engine/src/kv_cache_integration/wiring.rs` | // STAND: TS:2026-10-13T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-store/tests/block_cache_invalidation.rs` | // STAND: TS:2026-10-12T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-crypto/src/secure_buf.rs` | // STAND: TS:2026-10-12T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-crypto/tests/secure_buf.rs` | // STAND: TS:2026-10-12T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-kvcache/tests/cache_scope_scan.rs` | // STAND: TS:2026-10-12T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-vector/src/diskann/persistence.rs` | // STAND: TS:2026-10-06T12:00:00Z (SESSION: fix/p03-d-diskann-pending-wal) | `-` | `-` |
| TS:2026-10 | `crates/contextra-vector/src/quantize/mod.rs` | // STAND: TS:2026-10-06T12:00:00Z (SESSION: P03-F03) | `-` | `P03-F03` |
| TS:2026-10 | `crates/contextra-kvcache/tests/e2e_lifecycle_with_engine.rs` | // STAND: TS:2026-10-05T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-agent/tests/j01closure-contextra-agent_wiring_test.rs` | // STAND: TS:2026-10-04T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-crypto/tests/campaign_j25.rs` | // STAND: TS:2026-10-04 (Kampagne J-25) | `-` | `-` |
| TS:2026-10 | `crates/contextra-kvcache/src/scratchpad_invalidation.rs` | // STAND: TS:2026-10-01T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-kvcache/src/prefix_store.rs` | // STAND: TS:2026-10-01T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-kvcache/tests/j06closure-contextra-kvcache_wiring.rs` | // STAND: TS:2026-10-01T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-kvcache/tests/j06_contextra_kvcache_wiring.rs` | // STAND: TS:2026-10-01T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-agent/src/clm_scratchpad.rs` | // STAND: TS:2026-10-01T00:00:00Z | `-` | `-` |
| TS:2026-10 | `crates/contextra-agent/src/graph.rs` | // STAND: TS:2026-10-01T00:00:00Z (SESSION: 5a38054a) | `-` | `5a38054a` |
| TS:2026-10 | `crates/contextra-agent/src/goal_condition.rs` | // STAND: TS:2026-10-01T00:00:00Z (SESSION: 088b4a44) | `-` | `088b4a44` |
