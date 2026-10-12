# Contextra Prompter Restrukturierungsplan (v34)

> **Verzeichnis**: `.jules/`
> **Ziel-Prompter**: `.jules/contextra-Prompter.html`
> **Stand**: 2026-10-12
> **Fokus**: Umstellung auf die tatsächliche 35-Package (34 Crates + 1 Bench) Ring-0..4-Architektur, Ground-Truth-Klarstellung, Stabilisierung, Wartung, Tests, Audits & Pre-Commit-Disziplin.

---

## 0. Ground Truth & Crate-Analyse

Aus der automatischen Analyse via `cargo metadata --no-deps --format-version 1` und den Manifest-Dateien im Workspace ergeben sich **35 Workspace-Packages** (34 Haupt-Crates unter `crates/` und 1 Benchmark-Package `contextra-bench` unter `benchmarks/contextra-bench`):

| Package Name | Verzeichnis | Ring | Hauptaufgabe |
|---|---|---|---|
| `context` | `crates/contextra` | Ring 4 | Top-Level Facade Crate & `ContextraBuilder` |
| `contextra-adapt` | `crates/contextra-adapt` | Ring 0 | LinUCB Bandit, Lyapunov Drift, PID Pool Sizing |
| `contextra-agent` | `crates/contextra-agent` | Ring 3 | State Machine, Exactly-Once Execution, AuditLog, TokenBudget |
| `contextra-audit-export` | `crates/contextra-audit-export` | Ring 0 | DSGVO Art. 30 Verarbeitungsverzeichnis Generator |
| `contextra-avv-generator` | `crates/contextra-avv-generator` | Ring 0 | AVV (Auftragsverarbeitungsvertrag) Generator |
| `contextra-bench` | `benchmarks/contextra-bench` | Tooling | LOCOMO / Long-Mem-Eval Benchmark Harness |
| `contextra-checkpoint` | `crates/contextra-checkpoint` | Ring 1 | RAII CheckpointGuard, Time-Travel Rollback |
| `contextra-cognition` | `crates/contextra-cognition` | Ring 3 | Structural Consolidation Pass & Synthesis |
| `contextra-core` | `crates/contextra-core` | Ring 0 | Strangler-Facade (Re-Export für Types/Ports/MVCC) |
| `contextra-crypto` | `crates/contextra-crypto` | Ring 0 | Encryption at Rest, DeletionProof & KV-Cache Security |
| `contextra-db` | `crates/contextra-db` | Ring 3 | Collection CRUD, 2PC Transactions, Hybrid Search |
| `contextra-durable-fs` | `crates/contextra-durable-fs` | Ring 0 | Durable Filesystem Operations & synchrone Disk-Flushes |
| `contextra-engine` | `crates/contextra-engine` | Ring 3 | Core storage, index & transaction orchestrator engine |
| `contextra-graph` | `crates/contextra-graph` | Ring 0 | CSR Graph, PPR, Community Detection |
| `contextra-infer-candle` | `crates/contextra-infer-candle` | Ring 2 | Pure Rust ML Backend, GGUF Loader, KvBridgeAdapter |
| `contextra-infer-ollama` | `crates/contextra-infer-ollama` | Ring 2 | Ollama HTTP Client, XML Escaping |
| `contextra-infer-onnx` | `crates/contextra-infer-onnx` | Ring 2 | ONNX CrossEncoder Reranking Execution Provider |
| `contextra-kvcache` | `crates/contextra-kvcache` | Ring 1 | Tenant-isoliertes KV-Cache & Block Management |
| `contextra-license` | `crates/contextra-license` | Ring 4 | Lizenz- und Aktivierungsprüfung für Ringe |
| `contextra-mcp` | `crates/contextra-mcp` | Ring 4 | stdio JSON-RPC 2.0 Server, McpSandbox |
| `contextra-mvcc` | `crates/contextra-mvcc` | Ring 0 | MVCC Visibility, SeqLog, SnapshotRegistry |
| `contextra-ports` | `crates/contextra-ports` | Ring 0 | Non-Determinism Ports (Clock, Rng, IdGen) |
| `contextra-privacy` | `crates/contextra-privacy` | Ring 3 | Cloud Egress Security, DLP & PII Protection |
| `contextra-py` | `crates/contextra-py` | Ring 4 | PyO3 Python Bindings (Standalone Workspace) |
| `contextra-rank` | `crates/contextra-rank` | Ring 0 | 4-Signal Search Fusion, Score-Kalibrierung |
| `contextra-router` | `crates/contextra-router` | Ring 3 | SLM Router Engine, Conformal Calibration |
| `contextra-sandbox` | `crates/contextra-sandbox` | Ring 2 | WASM Execution Boundary Sandbox |
| `contextra-simd` | `crates/contextra-simd` | Ring 0 | SIMD Vektordistanz-Routinen (AVX2/NEON) |
| `contextra-store` | `crates/contextra-store` | Ring 1 | LSM-Tree Storage Engine, WAL, SSTable |
| `contextra-sys` | `crates/contextra-sys` | Ring 0 | Systemnahe Bindings, Low-Level I/O & Memory-Mapping |
| `contextra-testkit` | `crates/contextra-testkit` | Tooling | ManualClock, FaultVfs, Mock-Storage |
| `contextra-text` | `crates/contextra-text` | Ring 0 | BM25 Inverted Index, German Compound Splitter |
| `contextra-types` | `crates/contextra-types` | Ring 0 | Kanonische Domain-Typen (DocId, TxId, TenantId) |
| `contextra-vector` | `crates/contextra-vector` | Ring 0 | HNSW Graph, DiskANN, SQ8 Quantizer |
| `contextra-wire` | `crates/contextra-wire` | Ring 0 | FlatBuffers Wire-Protokoll & IPC Generator |

### Statusentscheidung `contextra-crypto` vs. `contextra-privacy`:
Beide Packages existieren eigenständig und koexistieren im Workspace:
1. `contextra-crypto` (Package-Name: `contextra-crypto` in `crates/contextra-crypto/Cargo.toml`): Ring 0 Crate für Encryption-at-Rest, KV-Cache-Verschlüsselung, WAL-HMAC-Kette und DSGVO Art. 17 DeletionProof.
2. `contextra-privacy` (Package-Name: `contextra-privacy` in `crates/contextra-privacy/Cargo.toml`): Ring 3 Crate für Cloud Egress Security, DLP-Regeln, PII-Erkennung und Regex-Validierung (Luhn/IBAN).

---

## 1. Kontext & Ausgangslage

Das Contextra-Projekt umfasst **35 Workspace-Packages** (34 Crates unter `crates/` und 1 Benchmark-Harness unter `benchmarks/contextra-bench/`) in einer geschichteten Ring-Architektur (Ring 0 Unsafe Island & Core, Ring 1 Infrastructure, Ring 2 Leaf Engines, Ring 3 Orchestration & Cognition, Ring 4 Composition & Boundaries).

Inkompatibilitäten und Korrekturen gegenüber veralteten Plänen:
- **Bereinigung von Selbstverweisen**: Bezeichnungen wie "X (nun X)" wurden durch die tatsächlichen Package-Namen ersetzt.
- **Klarstellung Crypto vs. Privacy**: `contextra-crypto` (Ring 0) und `contextra-privacy` (Ring 3) existieren beide als eigenständige Packages.
- **Vollständigkeit**: Alle 35 Packages (inkl. `contextra-durable-fs`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-license` und `contextra-bench`) sind vollständig im Schema erfasst.
- **Prozess- & Bau-Befehle**: `xtask` läuft isoliert außerhalb des Haupt-Workspace (`exclude = ["xtask"]`), Befehle nutzen `--manifest-path xtask/Cargo.toml`.

---

## 2. Workspace-Topologie & Ring-Schichten (35 Packages)

| Ring / Layer | Crate Name | Cargo Package | Rolle & Hauptaufgabe | Tier | Risk |
|---|---|---|---|---|---|
| **Ring 0 / L0** | `contextra-wire` | `contextra-wire` | Unsafe Island, FlatBuffers IPC DTO Code Generator | Tier 2 | `gen` |
| **Ring 0 / L0** | `contextra-sys` | `contextra-sys` | C/FFI Bindings & Sys-Wrapper (`mmap`, `mlock`) | Tier 2 | `ffi` |
| **Ring 0 / L0** | `contextra-durable-fs` | `contextra-durable-fs` | Durable Filesystem Operations & synchrone Disk-Flushes | Tier 2 | `none` |
| **Ring 0 / L0** | `contextra-simd` | `contextra-simd` | SIMD Vektordistanz-Routinen (AVX2/NEON) | Tier 2 | `simd` |
| **Ring 0 / L0** | `contextra-types` | `contextra-types` | Kanonische Domain-Typen: `DocId`, `TxId`, `TenantId`, `Embedding` | Tier 2 | `none` |
| **Ring 0 / L0** | `contextra-ports` | `contextra-ports` | Non-Determinism Ports (`Clock`, `Rng`, `IdGen`) | Tier 2 | `none` |
| **Ring 0 / L0** | `contextra-mvcc` | `contextra-mvcc` | MVCC Visibility, `SeqLog` (24-Byte Entries), `SnapshotRegistry` | Tier 2 | `concurrency` |
| **Ring 0 / L0** | `contextra-crypto` | `contextra-crypto` | AES-256-GCM-SIV, HKDF, HMAC, DSGVO Art. 17 `DeletionProof` | Tier 1 | `sec` |
| **Ring 0 / L0** | `contextra-adapt` | `contextra-adapt` | LinUCB Bandit, Lyapunov Drift, PID Pool Sizing | Tier 2 | `calib` |
| **Ring 0 / L0** | `contextra-rank` | `contextra-rank` | 4-Signal Search Fusion, `IsotonicCalibrator`, `PlattScaler` | Tier 2 | `fusion` |
| **Ring 0 / L0** | `contextra-text` | `contextra-text` | BM25 Inverted Index, German Compound Splitter | Tier 2 | `utf8` |
| **Ring 0 / L0** | `contextra-graph` | `contextra-graph` | CSR Graph, PPR, Community Detection, Bi-Temporal Axes | Tier 2 | `none` |
| **Ring 0 / L0** | `contextra-vector` | `contextra-vector` | HNSW Graph, DiskANN, SQ8 Quantizer, `VectorCandidateStream` | Tier 1 | `simd` |
| **Ring 0 / L0** | `contextra-audit-export` | `contextra-audit-export` | DSGVO Art. 30 Verarbeitungsverzeichnis Export | Tier 2 | `none` |
| **Ring 0 / L0** | `contextra-avv-generator` | `contextra-avv-generator` | AVV Vertragsmuster Generator | Tier 2 | `none` |
| **Ring 0 / L0** | `contextra-core` | `contextra-core` | Strangler-Facade (Re-Export für Types/Ports/MVCC) | Tier 1 | `none` |
| **Ring 1 / L1** | `contextra-checkpoint` | `contextra-checkpoint` | RAII `CheckpointGuard`, Time-Travel Rollback, BLAKE3 Manifest | Tier 2 | `none` |
| **Ring 1 / L1** | `contextra-kvcache` | `contextra-kvcache` | Tenant-isoliertes KV-Cache & Block Management | Tier 2 | `sec` |
| **Ring 1 / L1** | `contextra-store` | `contextra-store` | LSM-Tree (`LsmStorage`), WAL (`wal.rs`), MemTable, SSTable | Tier 1 | `crash` |
| **Ring 2 / L2** | `contextra-infer-onnx` | `contextra-infer-onnx` | ONNX CrossEncoder Reranking, Feature-gated `onnx` | Tier 3 | `gate` |
| **Ring 2 / L2** | `contextra-infer-candle` | `contextra-infer-candle` | Pure Rust ML Backend, GGUF Loader, `KvBridgeAdapter` | Tier 3 | `candle` |
| **Ring 2 / L2** | `contextra-infer-ollama` | `contextra-infer-ollama` | Ollama HTTP Client, XML Escaping, Prompt Prefixing | Tier 3 | `inj` |
| **Ring 2 / L2** | `contextra-sandbox` | `contextra-sandbox` | Process Isolation Sandbox & Containment Environment | Tier 2 | `sec` |
| **Ring 3 / L3** | `contextra-db` | `contextra-db` | Collection CRUD, 2PC Transactions, Hybrid Search | Tier 1 | `fusion` |
| **Ring 3 / L3** | `contextra-engine` | `contextra-engine` | Core storage, index & transaction orchestrator engine | Tier 2 | `concurrency` |
| **Ring 3 / L3** | `contextra-cognition` | `contextra-cognition` | Structural Consolidation Pass (LLM-frei, P1) & Synthesis Pass | Tier 2 | `none` |
| **Ring 3 / L3** | `contextra-router` | `contextra-router` | SLM Router Engine, `recalibrate_conformal` | Tier 3 | `nan` |
| **Ring 3 / L3** | `contextra-agent` | `contextra-agent` | State Machine, Exactly-Once Execution, AuditLog, TokenBudget | Tier 2 | `budget` |
| **Ring 3 / L3** | `contextra-privacy` | `contextra-privacy` | Cloud Egress Security, DLP, PII Vaulting | Tier 2 | `sec` |
| **Ring 4 / L4** | `contextra-mcp` | `contextra-mcp` | stdio JSON-RPC 2.0 Server, `McpSandbox`, Injection Guard | Tier 1 | `proto` |
| **Ring 4 / L4** | `contextra-py` | `contextra-py` | PyO3 Bindings (Standalone Workspace) | Tier 1 | `ffi` |
| **Ring 4 / L4** | `contextra-license` | `contextra-license` | Lizenz- und Aktivierungsprüfung für Ringe | Tier 2 | `none` |
| **Ring 4 / L4** | `contextra` | `contextra` | Top-Level Facade Crate & `ContextraBuilder` | Tier 2 | `none` |
| **Tooling** | `contextra-testkit` | `contextra-testkit` | `ManualClock`, `FaultVfs` für deterministische Tests | Tier 2 | `none` |
| **Tooling** | `contextra-bench` | `contextra-bench` | LOCOMO / Long-Mem-Eval Benchmark Harness | Tier 2 | `none` |

---

## 3. Architektur-Invarianten im Prompter

1. **Pass Separation Invariant (P1)**:
   - `memory_consolidation.rs` (`contextra-cognition`) implementiert den determistischen, LLM-freien Structural Consolidation Pass (`run_structural_synthesis_pass`).
   - Darf strikt ZERO Abhängigkeiten zu `contextra-graph` und ZERO LLM-API-Aufrufe enthalten.
2. **Ring 0 Tokio-Purity**:
   - Ring 0 Kern-Crates (`contextra-text`, `contextra-graph`, `contextra-adapt`, `contextra-crypto`) sind synchron und frei von `tokio`.
3. **Non-Determinism Ports**:
   - Synchronous, dyn-kompatible Trait-Ports (`Clock`, `Rng`, `IdGen`) in `contextra-ports`, standardisierte Test-Implementierung `ManualClock` in `contextra-testkit`.
4. **Group-Commit Leader Scope Reduction (`contextra-store`)**:
   - `commit_mutex` wird vor physical WAL disk I/O (`wal.append_batch`) freigegeben und danach für MemTable Updates re-equiriert.
5. **128-Bit DocId Rollout**:
   - Feature `docid-128` schaltet 16-Byte BLAKE3 DocId Derivation frei.
6. **Unwrap Baseline Trend Governance (Gate 2b)**:
   - Erfordert Net-Zero-Wachstum von `.unwrap()` in Tier-1 Crates.

---

## 4. Umsetzungsstrategie & Datenfluss

1. **Prompter Config Update (`.jules/prompter-tiers.toml`)**:
   - Aktualisierung aller 35 Packages in `[crate_overrides]`, `[target_architecture]` und `[component_focus]`.
2. **Automatische Datengenerierung (`cargo run --manifest-path xtask/Cargo.toml -- gen-prompter-data`)**:
   - Generierung von `.jules/prompter-data.json` mit Live-LOC, Test-Count, Risk & Target-Architektur.
3. **Qualitätssicherung & Validation**:
   - Verifizierung aller tests via `cargo test --manifest-path xtask/Cargo.toml prompter`.
