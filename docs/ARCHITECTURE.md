# Contextra — Systemarchitektur & Ring-Modell

Dieses Dokument ist die maßgebliche technische Architekturbeschreibung des Contextra Kerns. Es übersetzt die **[Finale Produktspezifikation (Synthese)](spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md)** in eine vertiefte Systembeschreibung, dokumentiert den Crate-Bestand, das Ring-0–4-Modell, die drei Produkt-Ringe (`fast`, `sovereign`, `compliance`), die Vier-Schichten-Interface-Architektur sowie die Systeminvarianten und Locking-Disziplin.

---

## Inhaltsverzeichnis
1. [Ring-Modell (Ring 0–4) & Produkt-Ringe](#1-ring-modell)
2. [Vier-Schichten-Interface-Modell](#2-vier-schichten-modell)
3. [Crate-Inventar & Abhängigkeitsmatrix](#3-crate-inventar)
4. [DAG-Topologie (Automatisch generiert)](#4-dag-topologie)
5. [Systeminvarianten P1–P30 (Automatisch generiert)](#5-systeminvarianten)
6. [Unsafe-Inseln & Locking-Disziplin](#6-unsafe-inseln)
7. [Referenzen & Weiterführende Dokumente](#7-referenzen)

---

<a id="1-ring-modell"></a>
## 1. Ring-Modell (Ring 0–4) & Produkt-Ringe

Contextra gliedert seine Funktionalität architektonisch in ein Fünf-Ring-Schichtenmodell (Ring 0 bis Ring 4) sowie kommerziell/kryptographisch in drei Produkt-Ringe (`fast`, `sovereign`, `compliance`).

### 1.1 Das Fünf-Ring-Schichtenmodell

Abhängigkeiten dürfen ausschließlich **von höheren Ringen auf tiefere Ringe** verlaufen. Aufwärtskanten (z. B. ein Ring-0-Crate, das von einem Ring-3-Crate abhängt) sind streng verboten (Principle P5 / Ring Layering).

* **Ring 0 — Foundation & Pure Numerics / Types (Sync-Kern)**:
  Kerneigene Datenstrukturen, Typdefinitionen, Traits, Kryptographie, Mathematik, Vektor- und Graph-Primitive. Ring 0 enthält **kein `tokio`** (P26: *Sync-Kern, async-Schale*).
  *Crates:* `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-wire`, `contextra-sys`, `contextra-simd`, `contextra-crypto`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-rank`, `contextra-adapt`.
* **Ring 1 — Persistence & Cache**:
  Persistenz-Engines (LSM-Tree, WAL mit Group-Commit, HMAC-Integritätskette) und Caching-Infrastruktur (KV-Cache, Checkpoint-Registry). Async-I/O an Grenzen erlaubt.
  *Crates:* `contextra-store`, `contextra-kvcache`, `contextra-checkpoint`.
* **Ring 2 — External Adapters & Sandboxing (Leaf Engines)**:
  Isoliert flüchtige oder externe Schwergewicht-Abhängigkeiten (`candle`, `ollama`, `ort`/ONNX, `wasmtime`). Ring-2-Crates sind Blätter im Graphen und hängen nie von Ring 1 oder 3 ab.
  *Crates:* `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-sandbox`.
* **Ring 3 — Orchestration & Cognition**:
  Geschäfts- und Orchestrierungslogik (`Collection`, Multi-Index-Synthese, PII-Vault/Privacy Gateway, Profil-Routing, Agent-Workflow).
  *Crates:* `contextra-engine`, `contextra-cognition`, `contextra-privacy`, `contextra-router`, `contextra-agent`.
* **Ring 4 — Boundary & Composition Roots**:
  Öffentliche Fassade (`contextra`), MCP-Server (`contextra-mcp`) und Language-Bindings (`contextra-py`). Bildet die einzige Composition Root.

### 1.2 Die drei Produkt-Ringe (Open vs. Closed)

- **Ring `fast` (MIT/Apache-2.0, Open Source):** Für jeden Nutzer frei zugänglich. Enthält Vektor-, Text- und Graph-Retrieval, Bandit-Routing sowie lokale Inferenz. Kein Krypto-Overhead im Hot-Path.
- **Ring `sovereign` (Quelloffen, Opt-in Aktivierung):** Quellcode bleibt offen (Löschbeweis `DeletionProof`, Privacy Gateway, Zero-Net-Traffic). Aktivierung ist wegen Latenzkosten opt-in.
- **Ring `compliance` (Closed-Source, Commercial):** Quellcode verlässt das Haus nie. Vertrieb als signiertes Binary/Appliance. Beinhaltet Lizenzschicht (`contextra-license`), BSI TR-02102-1 Mapping und Mandanten-Scoping. <!-- crate-ref-ignore -->

---

<a id="2-vier-schichten-modell"></a>
## 2. Vier-Schichten-Interface-Modell

Jede Schnittstelle in Contextra gehört zu genau einer der vier Schichten gemäß §5 der Produktspezifikation:

1. **Schicht 1 — Domänentypen (`contextra-types`):** Reine, serialisierbare Typen (`DocId`, `TxId`, `TenantId`, `FilterAST`, Budgets).
2. **Schicht 2 — Ports (`contextra-ports`):** Trait-Definitionen (`StorageEngine`, `VectorIndex`, `TextIndex`, `GraphEngine`, `Embedder`).
3. **Schicht 3 — Fassade (`contextra-engine`, `contextra`):** Das vereinigende High-Level API für Rust.
4. **Schicht 4 — Produktgrenze (`contextra-mcp`, `contextra-py`, `contextra-wire`):** Stdio JSON-RPC MCP Server, PyO3 Bindings, FlatBuffers Schema.

---

<a id="3-crate-inventar"></a>
## 3. Crate-Inventar & Abhängigkeitsmatrix

Tabelle aller Kern-Crates unter `crates/`:

| Ring | Crate-Name | Inhalt & Verantwortlichkeit |
|---|---|---|
| **0** | `contextra-types` | Identifikatoren (`DocId`, `TxId`, `TenantId`), Filter-AST, Kernel-Budgets |
| **0** | `contextra-ports` | Trait-Verträge für Storage, Indizes, Embedder & Uhren |
| **0** | `contextra-mvcc` | `SeqLog`, `SnapshotRegistry` und transaktionaler `TxBuffer` |
| **0** | `contextra-wire` | FlatBuffers-IPC Generat & Zero-Copy Adapter (**Unsafe-Insel**) |
| **0** | `contextra-sys` | System-Abstraktionen für `mmap`, `mlock` & Win32-ACLs (**Unsafe-Insel**) |
| **0** | `contextra-simd` | AVX2/AVX-512/NEON SIMD-Distanzberechnungskerne (**Unsafe-Insel**) |
| **0** | `contextra-crypto` | AES-256-GCM-SIV, WAL-HMAC-Kette, `DeletionProof`, Zeroize |
| **0** | `contextra-vector` | HNSW, DiskANN, SQ8 / RaBitQ Quantisierung |
| **0** | `contextra-text` | BM25 / BM25F Volltextindexierung & deutsche Morphologie |
| **0** | `contextra-graph` | CSR Graph, Forward-Push PPR, Leiden Community-Detection, Hyperkanten |
| **0** | `contextra-rank` | 4-Signal-Fusion, Isotonic- / Platt-Kalibrierung & Drift |
| **0** | `contextra-adapt` | LinUCB Bandit, Sherman-Morrison, FC-TS, Lyapunov-Drift & PID |
| **1** | `contextra-store` | LSM-Tree Storage Engine mit WAL Group-Commit & HMAC-Check |
| **1** | `contextra-kvcache` | Verschlüsselter Prefix-Radix-Baum & KV-Cache Segmentverwaltung |
| **1** | `contextra-checkpoint` | Time-Travel-Registry & Checkpoint-Verwaltung ohne globalen Zustand |
| **2** | `contextra-infer-candle` | GGUF-Modell-Inferenz via Candle |
| **2** | `contextra-infer-ollama` | HTTP-Inferenz-Client für Ollama |
| **2** | `contextra-infer-onnx` | ONNX Embeddings & Cross-Encoder Reranking via `ort` |
| **2** | `contextra-sandbox` | WASM-Ausführungsisolation mit Fuel- & Wall-Clock-Limits via Wasmtime |
| **3** | `contextra-engine` | `Collection`, Multi-Index-Pläne & Schreibtransaktionen |
| **3** | `contextra-cognition` | Dreistufige Kognitionspipeline, Synthese & Kompaktierung |
| **3** | `contextra-privacy` | Cloud-Egress Gateway, PII-Vault, Surrogat-Tokenisierung & DLP |
| **3** | `contextra-router` | SLM-Profil-Routing & MCP-Dispatch |
| **3** | `contextra-agent` | Agenten-Workflow-Engine mit auditierbarer State-Machine |
| **4** | `contextra` | Haupt-Fassade & Composition Root |
| **4** | `contextra-mcp` | Stdio-JSON-RPC MCP Server |
| **4** | `contextra-py` | PyO3 Python-FFI Bindings |

---

<a id="4-dag-topologie"></a>
## 4. DAG-Topologie (Automatisch generiert)

<!-- AUTOGENERATED:START:DAG_TOPOLOGY -->
```
Layer 0:  contextra-adapt — Anpassungs- und Transformations-Layer für Datenmodelle (PID Latency Controller)
          contextra-privacy — Egress Filtering, Anonymisierung und Privacy Compliance Rules
          contextra-sandbox — WASM Execution Boundary und Isolations-Sandbox
          contextra-sys — Systemnahe Bindings, Low-Level I/O und Memory-Mapping
          contextra-types — Grundlegende Typdefinitionen (DocId, TxId, TenantId, Error-Typen)
          contextra-wire — FlatBuffers Wire-Protokolle und Serialisierungs-Formate
Layer 1:  contextra-crypto — Kryptographische Vaults, Zeroize, Egress-Verschlüsselung und HMAC (deps: contextra-types)
          contextra-mvcc — Multi-Version Concurrency Control und Isolation-Mechanismen (deps: contextra-types)
          contextra-ports — Abstrakte Port-Schnittstellen und Trait-Definitionen für Entkopplung (deps: contextra-types)
Layer 2:  contextra-audit-export — GDPR Article 30 Processing Register export generator for Contextra (deps: contextra-ports, contextra-types)
          contextra-checkpoint — Persistent Checkpoint Engine und Blake3 Manifest-Verifikation (deps: contextra-ports, contextra-types)
          contextra-core — Kern-Datenstrukturen, Tombstones, Invarianten und Memory-Buffer (deps: contextra-mvcc, contextra-ports, contextra-types, contextra-wire)
          contextra-graph — CSR Graph-Engine, Path Graphing und GraphRAG Community Detection (deps: contextra-ports, contextra-types)
          contextra-kvcache — Stufenmodell KV-Cache Storage & Offloading (deps: contextra-crypto, contextra-ports, contextra-types)
          contextra-rank — Ranking-, Fusion- und Score-Kalibrierungs-Algorithmen (RRF, Isotonische/Platt Kalibrierung, Bonus Scorers) (deps: contextra-ports, contextra-types)
          contextra-router — Lyapunov Drift Control und Multi-Candidate Search Routing Engine (deps: contextra-adapt, contextra-ports, contextra-privacy, contextra-types, contextra-wire)
          contextra-testkit — Test Fixtures, Mock Engines und Chaos Matrix Harness (deps: contextra-ports, contextra-types)
          contextra-text — Volltextsuche und Inverted-Index (BM25 Engine) (deps: contextra-ports, contextra-types)
Layer 3:  contextra-infer-ollama — Ollama External Provider Client und Embeddings Integration (deps: contextra-ports, contextra-rank, contextra-types)
          contextra-simd — SIMD-beschleunigte Distanzberechnungen und Vektor-Operationen (deps: contextra-core)
          contextra-store — LSM-Tree Storage Engine, WAL Durability und Compaction Engine (deps: contextra-core, contextra-crypto, contextra-sys)
Layer 4:  contextra-infer-candle — Candle LLM Inference Client und KV-Bridge Adapter (deps: contextra-crypto, contextra-ports, contextra-rank, contextra-store, contextra-types)
          contextra-vector — Vektorsuche und Indexierung (HNSW, DiskANN, Quantisierung) (deps: contextra-core, contextra-crypto, contextra-simd, contextra-sys)
Layer 5:  contextra-engine — Compute Pool und Asynchrone Storage-Execution Layer (ADR-N02) (deps: contextra-adapt, contextra-checkpoint, contextra-crypto, contextra-graph, contextra-mvcc, contextra-ports, contextra-rank, contextra-store, contextra-sys, contextra-text, contextra-types, contextra-vector)
          contextra-infer-onnx — ONNX Embeddings und Cross-Encoder Reranking Execution Provider (deps: contextra-infer-candle, contextra-ports, contextra-rank, contextra-types)
Layer 6:  contextra-cognition — Konsolidierungs- und Background Sleep Passes (deps: contextra-engine, contextra-graph, contextra-ports, contextra-store, contextra-types, contextra-vector)
Layer 7:  contextra-db — Contextra Main Database Abstraction, Hybrid Search & Collections (deps: contextra-adapt, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-engine, contextra-graph, contextra-ports, contextra-rank, contextra-store, contextra-sys, contextra-text, contextra-types, contextra-vector)
Layer 8:  contextra — Haupt-Library Facade für Endanwender (deps: contextra-core, contextra-crypto, contextra-db, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-privacy, contextra-rank, contextra-router)
          contextra-agent — Audit Engine, InMemoryStorageEngine und Agent Execution Pipeline (deps: contextra-checkpoint, contextra-db, contextra-graph, contextra-ports, contextra-router, contextra-store, contextra-types)
          contextra-bench — Contextra — Reproducible Benchmark Harness for Retrieval Accuracy (deps: contextra-core, contextra-db, contextra-graph, contextra-infer-onnx, contextra-store, contextra-text, contextra-vector)
          contextra-py — PyO3 Python Bindings (deps: contextra-adapt, contextra-core, contextra-db, contextra-rank, contextra-router, contextra-store)
Layer 9:  contextra-mcp — Model Context Protocol Server & Cloud Egress Gateway (deps: contextra-adapt, contextra-agent, contextra-crypto, contextra-infer-candle, contextra-infer-onnx, contextra-ports, contextra-privacy, contextra-rank, contextra-types, contextra-wire)
```

**Aktiver Workspace-Build**: 32 Kern-Crates.
<!-- AUTOGENERATED:END:DAG_TOPOLOGY -->

---

<a id="5-systeminvarianten"></a>
## 5. Systeminvarianten P1–P30 (Automatisch generiert)

<!-- AUTOGENERATED:START:INVARIANTS_TABLE -->
| Invariante | Status | Befund |
|---|---|---|
| **Souveränität** (Zero-C-Deps im Core) | ✅ Erfüllt | Core-Schichten laufen in Pure Rust. Ollama übernimmt LLM/Embeddings via HTTP. |
| **Zero-Panic** | 🟢 Vollständig | Alle offenen `.expect()`-Stellen im Prod-Code wurden umgestellt. |
| **Determinismus** (SIMD) | ✅ Erfüllt | Cross-Check SIMD vs. Skalar via Proptest. |
| **WAL-Crash-Consistency** | ✅ Erfüllt | Fault-Injection im WAL, HMAC-Chaining. |
| **Graph-Persistenz** | ✅ Erfüllt | Persistierung im LSM-Tree unter den Präfixen `__graph:entity:` und `__graph:edge:`. |
| **DAG Integrity** | ✅ Erfüllt | Unidirektionale Schichten-Abhängigkeiten von Layer 0 bis Layer 4. |
| **Disk-I/O Isolation** | ✅ Erfüllt | tokio::fs für Metadaten/Lifecycle, std::fs::File ausschließlich innerhalb spawn_blocking für Block-Level Random-Access (ADR-012). |
| **Snapshot Isolation** | 🟢 Vollständig | Für Storage (LSM), Text (BM25) & Vektor (HNSW) vollständig implementiert. Graph-Suche operiert auf dem aktuellen In-Memory-Zustand (ADR-024). |
<!-- AUTOGENERATED:END:INVARIANTS_TABLE -->

---

<a id="6-unsafe-inseln"></a>
## 6. Unsafe-Inseln & Locking-Disziplin

### 6.1 Unsafe-Inseln

Das Repository erzwingt `#![forbid(unsafe_code)]` in allen Crates mit exakt **drei zulässigen Unsafe-Inseln**:

1. **`contextra-sys`**: OS-Level Operationen (`mmap`, `mlock`, Win32 ACLs).
2. **`contextra-simd`**: Handoptimierte SIMD-Kerne (AVX2, AVX-512, NEON).
3. **`contextra-wire`**: FlatBuffers-Zero-Copy-Adapter.

Jeder `unsafe`-Block erfordert zwingend eine `// SAFETY:`-Begründung.

### 6.2 Locking-Disziplin

```
collections (RwLock) → kv_locks (schlüssel-granular via KvKeyLocks) → embedder (RwLock)
```

- Mehrfach-Locks werden ausnahmslos in fester Reihenfolge von links nach rechts akquiriert.
- `KvKeyLocks` akquiriert Shard-Indizes zwingend in aufsteigend sortierter Reihenfolge (`acquire_multi_sorted`), um Deadlocks auszuschließen.
- Kein `.await` unter gehaltenen synchronen Mutex/RwLock-Guards.

---

<a id="7-referenzen"></a>
## 7. Referenzen & Weiterführende Dokumente

* **Normative Spezifikation (Synthese):** [`docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`](spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md)
* **Agenten-Betriebsanleitung:** [`AGENTS.md`](../AGENTS.md)
* **Compliance BSI TR-02102 Mapping:** [`docs/compliance/BSI_TR02102_MAPPING.md`](compliance/BSI_TR02102_MAPPING.md)
