# Contextra Prompter Restrukturierungsplan (v33)

> **Verzeichnis**: `.jules/`
> **Ziel-Prompter**: `.jules/contextra-Prompter.html`
> **Stand**: 2026-09-24
> **Fokus**: Umstellung auf die neue 31-Crate Ring-0..4-Architektur, Stabilisierung, Wartung, Tests, Audits & Pre-Commit-Disziplin.

---

## 1. Kontext & Ausgangslage

Das Contextra-Projekt wurde massiv umstrukturiert. Die frühere 18-Crate-Struktur wurde auf **31 Workspace-Crates** in einer geschichteten Ring-Architektur (Ring 0 Unsafe Island, Ring 0 Core, Ring 1 Infrastructure, Ring 2 Orchestration, Ring 3 Inference/Interfaces, Ring 4 Composition) erweitert.

Inkompatibilitäten der vorherigen Prompter-Version (v32):
- **Crate-Namen & Rebrandings**: Veraltete Bezeichnungen wie `contextra-vector` (nun `contextra-vector`), `contextra-infer-onnx` (nun `contextra-infer-onnx`), `contextra-infer-candle` (nun `contextra-infer-candle`), `contextra-infer-ollama` (nun `contextra-infer-ollama`), `contextra-crypto` (Cargo-Package `contextra-privacy`).
- **Aufgelöste Crates**: `contextra-rank` wurde aufgelöst — Score-Kalibrierung (`IsotonicCalibrator`, `PlattScaler`) wanderte nach `contextra-rank`, die PID-Pool-Regelung nach `contextra-adapt`.
- **Neue Ring 0 & Ring 1 Crates**: Fehlen von `contextra-wire`, `contextra-sys`, `contextra-simd`, `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-privacy`, `contextra-kvcache`, `contextra-testkit`, `contextra-sandbox`, `contextra-cognition`, `contextra-engine`, `contextra-rank`, `contextra-adapt`.
- **Prozess- & Bau-Befehle**: `xtask` läuft isoliert außerhalb des Haupt-Workspace (`exclude = ["xtask"]`), Befehle müssen `--manifest-path xtask/Cargo.toml` nutzen.

---

## 2. Neue 31-Crate-Topologie & Ring-Schichten

| Ring / Layer | Crate Name | Cargo Package | Rolle & Hauptaufgabe | Tier | Risk |
|---|---|---|---|---|---|
| **Ring 0 / L0** | `contextra-wire` | `contextra-wire` | Unsafe Island, FlatBuffers IPC DTO Code Generator (`schemas/memfuse.fbs`) | Tier 2 | `gen` |
| **Ring 0 / L0** | `contextra-sys` | `contextra-sys` | C/FFI Bindings & Sys-Wrapper | Tier 2 | `ffi` |
| **Ring 0 / L3** | `contextra-simd` | `contextra-simd` | SIMD Vektordistanz-Routinen (AVX2/NEON/Highway) | Tier 2 | `simd` |
| **Ring 0 / L0** | `contextra-types` | `contextra-types` | Kanonische Domain-Typen: `DocId`, `TxId`, `TenantId`, `Embedding` | Tier 2 | `none` |
| **Ring 0 / L1** | `contextra-ports` | `contextra-ports` | Non-Determinism Ports (`Clock`, `Rng`, `IdGen`) | Tier 2 | `none` |
| **Ring 0 / L1** | `contextra-mvcc` | `contextra-mvcc` | MVCC Visibility, `SeqLog` (24-Byte Entries), `SnapshotRegistry` & `TxBuffer` | Tier 2 | `concurrency` |
| **Ring 0 / L0** | `contextra-privacy` | `contextra-privacy` | Anonymisierung & PII-Redaktion | Tier 2 | `sec` |
| **Ring 0 / L0** | `contextra-adapt` | `contextra-adapt` | LinUCB Bandit, Lyapunov Drift, PID Pool Sizing | Tier 2 | `calib` |
| **Ring 1 / L1** | `contextra-privacy` | `contextra-privacy` | AES-256-GCM-SIV, HKDF, HMAC, DSGVO Art. 17 `DeletionProof` | Tier 1 | `sec` |
| **Ring 1 / L2** | `contextra-checkpoint` | `contextra-checkpoint` | RAII `CheckpointGuard`, Time-Travel Rollback, BLAKE3 Manifest | Tier 2 | `none` |
| **Ring 1 / L2** | `contextra-kvcache` | `contextra-kvcache` | Tenant-isoliertes KV-Cache & Block Management | Tier 2 | `sec` |
| **Ring 1 / L3** | `contextra-store` | `contextra-store` | LSM-Tree (`LsmStorage`), WAL (`wal.rs`), MemTable, SSTable, Compaction | Tier 1 | `crash` |
| **Ring 1 / L4** | `contextra-vector` | `contextra-vector` | HNSW Graph, DiskANN, SQ8 Quantizer, `VectorCandidateStream` | Tier 1 | `simd` |
| **Ring 1 / L2** | `contextra-graph` | `contextra-graph` | CSR Graph, PPR, Community Detection, Bi-Temporal Axes | Tier 2 | `none` |
| **Ring 1 / L2** | `contextra-text` | `contextra-text` | BM25 Inverted Index, German Compound Splitter | Tier 2 | `utf8` |
| **Ring 2 / L2** | `contextra-rank` | `contextra-rank` | 4-Signal Search Fusion, `IsotonicCalibrator`, `PlattScaler`, `DriftDetector` | Tier 2 | `fusion` |
| **Ring 2 / L7** | `contextra-db` | `contextra-db` | Collection CRUD, 2PC Transactions, Hybrid Search, Post-RRF Superseding | Tier 1 | `fusion` |
| **Ring 2 / L5** | `contextra-engine` | `contextra-engine` | Re-Export Facade für Transaktions-Pipeline (`transaction/mod.rs`) | Tier 2 | `concurrency` |
| **Ring 2 / L6** | `contextra-cognition` | `contextra-cognition` | Structural Consolidation Pass (LLM-frei, P1) & Generative Synthesis Pass | Tier 2 | `none` |
| **Ring 3 / L3** | `contextra-infer-onnx` | `contextra-infer-onnx` | ONNX CrossEncoder Reranking, Feature-gated `onnx` | Tier 3 | `gate` |
| **Ring 3 / L4** | `contextra-infer-candle` | `contextra-infer-candle` | Pure Rust ML Backend, GGUF Loader, `KvBridgeAdapter` | Tier 3 | `candle` |
| **Ring 3 / L3** | `contextra-infer-ollama` | `contextra-infer-ollama` | Ollama HTTP Client, XML Escaping, Prompt Prefixing | Tier 3 | `inj` |
| **Ring 3 / L2** | `contextra-router` | `contextra-router` | SLM Router Engine, `recalibrate_conformal`, Conformal Calibration | Tier 3 | `nan` |
| **Ring 3 / L8** | `contextra-agent` | `contextra-agent` | State Machine, Exactly-Once Execution, AuditLog, TokenBudget | Tier 2 | `budget` |
| **Ring 3 / L0** | `contextra-sandbox` | `contextra-sandbox` | Process Isolation Sandbox & Containment Environment | Tier 2 | `sec` |
| **Ring 3 / L9** | `contextra-mcp` | `contextra-mcp` | stdio JSON-RPC 2.0 Server, `McpSandbox`, Prompt Injection Guard | Tier 1 | `proto` |
| **Ring 3 / L8** | `contextra-py` | `contextra-py` | PyO3 Bindings (Standalone Workspace, `catch_unwind` Boundary) | Tier 1 | `ffi` |
| **Ring 0 / L2 (Test)** | `contextra-testkit` | `contextra-testkit` | `ManualClock`, `FaultVfs` für deterministische Tests | Tier 2 | `none` |
| **Ring 4 / L8** | `contextra` | `contextra` | Top-Level Facade Crate & `ContextraBuilder` | Tier 2 | `none` |
| **Ring 4 / L8** | `contextra-bench` | `contextra-bench` | LOCOMO / Long-Mem-Eval Benchmark Harness | Tier 2 | `none` |

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
   - Aktualisierung aller 31 Crates in `[crate_overrides]`, `[target_architecture]` und `[component_focus]`.
2. **Automatische Datengenerierung (`cargo run --manifest-path xtask/Cargo.toml -- gen-prompter-data`)**:
   - Generierung von `.jules/prompter-data.json` mit Live-LOC, Test-Count, Risk & Target-Architektur.
3. **HTML Baukasten Aktualisierung (`.jules/contextra-Prompter.html`)**:
   - Standard-Fallback-Daten in `DEFAULT_CRATES` und `DEFAULT_COMPONENTS` für alle 31 Crates spiegeln.
   - `PKG()` Package-Name Mapping anpassen.
   - Subsysteme, Eigenbau-Komponenten & Prozess-Objekte aktualisieren.
4. **Qualitätssicherung & Validation**:
   - Testen der JS-Syntax via Node.js (`node -c`).
   - Verifizierung aller pre-commit und CI-Gates.
