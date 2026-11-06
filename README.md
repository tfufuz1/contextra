# Contextra

> **Das modulare Rust-Baukastensystem (Crate-Ökosystem) für lokale KI-, Vektor-, Volltext-, Graph- und Speicher-Workflows.**

Contextra ist ein strikt modularisiertes Ökosystem aus autarken Rust-Crates ("Lego-Bausteine"). Jede Komponente ist so konzipiert, dass sie unabhängig verstanden, isoliert getestet und ohne unnötige transitive Abhängigkeiten in eigenen Projekten verwendet werden kann.

---

## 🧱 Architektur: Das Lego-Baukastensystem

Das Gesamtsystem transformiert sich von einem historisch gewachsenen Monolithen zu einer klaren, geschichteten Baustein-Architektur (**Ring-Modell**).

```
                      ┌──────────────────────────────────────────────┐
                      │    Ring 4: Endanwender-Fassaden & Gateways   │
                      │  contextra, contextra-mcp, contextra-py,     │
                      │  contextra-ingest, contextra-license         │
                      └──────────────────────┬───────────────────────┘
                                             │
                      ┌──────────────────────▼───────────────────────┐
                      │    Ring 3: Orchestrierung, Domäne & Löschung │
                      │  contextra-db, contextra-agent,              │
                      │  contextra-erasure, contextra-privacy        │
                      └──────────────────────┬───────────────────────┘
                                             │
                      ┌──────────────────────▼───────────────────────┐
                      │    Ring 2: Inferenz & Erweiterungen          │
                      │  contextra-infer-onnx, contextra-candle,     │
                      │  contextra-infer-ollama, contextra-sandbox   │
                      └──────────────────────┬───────────────────────┘
                                             │
                      ┌──────────────────────▼───────────────────────┐
                      │    Ring 1: Speicherkerne & Durability        │
                      │  contextra-store, contextra-checkpoint,      │
                      │  contextra-kvcache, contextra-durable-fs     │
                      └──────────────────────┬───────────────────────┘
                                             │
                      ┌──────────────────────▼───────────────────────┐
                      │    Ring 0: Reine Fundamente & Schnittstellen │
                      │  contextra-types, contextra-ports,           │
                      │  contextra-mvcc, contextra-crypto,           │
                      │  contextra-vector, contextra-text,           │
                      │  contextra-graph, contextra-simd,            │
                      │  contextra-sys, contextra-wire               │
                      └──────────────────────────────────────────────┘
```

### Die Kernprinzipien des Baukastens:
1. **Modulare Autarkie:** Jeder Crate kann als eigenständiger Baustein in Fremdprojekten genutzt werden (z. B. `contextra-vector` als HNSW-Index, `contextra-text` als BM25-Engine oder `contextra-store` als LSM-Tree-Speicher).
2. **Schnittstellen via Ports/Traits:** Engines kommunizieren über standardisierte Schnittstellen in `contextra-ports`. Konkrete Speicherkerne hängen nicht voneinander ab.
3. **Modularisierung der Löschkette in `contextra-erasure`:** Das bisherige Alleinstellungsmerkmal der revisionssicheren Löschung wurde aus den Kern-Engines herausgelöst und als eigener Baustein im Crate `contextra-erasure` isoliert. Datenhaltende Kerne implementieren lediglich schlanke Bereinigungs-Ports (`Purger`, `LayerScanner`).
4. **Offizielle Deklaration von Altlasten:** Noch im Kern verbliebene monolithische Verknüpfungen gelten als technische Schuld und werden in [`DEPRECATED_DEBT.md`](DEPRECATED_DEBT.md) nachverfolgt.

---

## 📦 Crate-Katalog: Alle 36 Bausteine im Überblick

### 🔹 Ring 0: Fundamente, Primitiven & Unsafe-Inseln
| Crate | Rolle / Funktion | Status |
|---|---|---|
| [`contextra-types`](crates/contextra-types/) | Zentrale Domänentypen (`DocId`, `TxId`, `TenantId`), fehler- und I/O-frei | 🟢 Autark |
| [`contextra-ports`](crates/contextra-ports/) | Abstrakte Port- und Trait-Definitionen für VFS, Storage, Indizes und Purger | 🟢 Autark |
| [`contextra-mvcc`](crates/contextra-mvcc/) | Multi-Version Concurrency Control und Transaktions-Isolation | 🟢 Autark |
| [`contextra-crypto`](crates/contextra-crypto/) | Kryptografische Primitiven, Hashketten, AEAD und KDF | 🟢 Autark |
| [`contextra-vector`](crates/contextra-vector/) | HNSW-Vektorindexierung, SQ8-Quantisierung und Cosine/Dot-Distanz | 🟡 Refactoring |
| [`contextra-text`](crates/contextra-text/) | BM25-Volltextsuche, Inverted Index und Block-Max WAND | 🟡 Refactoring |
| [`contextra-graph`](crates/contextra-graph/) | CSR-Graph-Engine, Traversierung, PPR und Community Detection für GraphRAG | 🟢 Autark |
| [`contextra-simd`](crates/contextra-simd/) | SIMD-beschleunigte Vektordistanzberechnungen (Unsafe-Insel 1) | 🟢 Autark |
| [`contextra-sys`](crates/contextra-sys/) | POSIX Low-Level I/O, mmap und Prozess-Härtung (Unsafe-Insel 2) | 🟢 Autark |
| [`contextra-wire`](crates/contextra-wire/) | FlatBuffers Zero-Copy IPC-Protokolle (Unsafe-Insel 3) | 🟢 Autark |
| [`contextra-rank`](crates/contextra-rank/) | Score-Kalibrierung (Platt, Isotonisch) und Fusion-Algorithmen | 🟢 Autark |
| [`contextra-core`](crates/contextra-core/) | Strangler-Fassade für historische Typen (wird schrittweise ersetzt) | 🟡 Fassade |

### 🔹 Ring 1: Speicherkerne & Persistenz
| Crate | Rolle / Funktion | Status |
|---|---|---|
| [`contextra-store`](crates/contextra-store/) | LSM-Tree Key-Value-Speicher mit WAL, MemTable, SSTable und Compaction | 🟡 Refactoring |
| [`contextra-checkpoint`](crates/contextra-checkpoint/) | Ring-1 Snapshots, CheckpointGuard und MVCC-Sequenzpins | 🟢 Autark |
| [`contextra-kvcache`](crates/contextra-kvcache/) | Segmentierter KV-Cache mit Tenant-Isolation und LRU-Eviction | 🟡 Refactoring |
| [`contextra-durable-fs`](crates/contextra-durable-fs/) | Crash-sichere atomare Dateisystem-Operationen und VFS-Implementierung | 🟢 Autark |

### 🔹 Ring 2: Inferenz & Ausführung
| Crate | Rolle / Funktion | Status |
|---|---|---|
| [`contextra-infer-onnx`](crates/contextra-infer-onnx/) | In-Process ONNX Embedding Engine und Cross-Encoder Reranker | 🟢 Autark |
| [`contextra-infer-candle`](crates/contextra-infer-candle/) | Lokale Candle-basierte Tensor-Inferenz | 🟢 Autark |
| [`contextra-infer-ollama`](crates/contextra-infer-ollama/) | Ollama API-Client für Embeddings und LLM-Inferenz | 🟢 Autark |
| [`contextra-sandbox`](crates/contextra-sandbox/) | WASM-Isolationssandbox für sichere Tool-Ausführung | 🟢 Autark |

### 🔹 Ring 3: Orchestrierung, Domänendienste & Revisionssichere Löschung
| Crate | Rolle / Funktion | Status |
|---|---|---|
| [`contextra-erasure`](crates/contextra-erasure/) | **Isolierter Löschbaustein:** Mehrstufige Lösch-Saga, Gate-Log, Tresor, Witness & signierter Bericht | 🟢 Autark |
| [`contextra-db`](crates/contextra-db/) | Collection-Verwaltung, Hybridsuche und Speicher-Orchestrierung | 🟡 Refactoring |
| [`contextra-agent`](crates/contextra-agent/) | Autonome Agenten-Workflow-Engine (`checkpoint → execute → commit → audit`) | 🟢 Autark |
| [`contextra-engine`](crates/contextra-engine/) | Transaktions- und Multi-Index-Ausführungskern | 🟡 Refactoring |
| [`contextra-cognition`](crates/contextra-cognition/) | Semantische Gedächtnis-Konsolidierung und Abstraktionsebenen | 🟢 Autark |
| [`contextra-privacy`](crates/contextra-privacy/) | Cloud Egress DLP, Pseudonymisierung und Surrogat-Vaulting | 🟡 Refactoring |
| [`contextra-router`](crates/contextra-router/) | Multi-Candidate Search Routing und SLM-Dispatching | 🟡 Refactoring |
| [`contextra-adapt`](crates/contextra-adapt/) | Adaptive HNSW-Candidate-Pool-Optimierung und Feedback-Loops | 🟢 Autark |

### 🔹 Ring 4: Fassaden, Werkzeuge & Schnittstellen
| Crate | Rolle / Funktion | Status |
|---|---|---|
| [`contextra`](crates/contextra/) | High-Level Fassade und Composition Root (`ContextraBuilder`) | 🟡 Fassade |
| [`contextra-mcp`](crates/contextra-mcp/) | Model Context Protocol (MCP) Server über stdio (JSON-RPC 2.0) | 🟡 Refactoring |
| [`contextra-py`](crates/contextra-py/) | PyO3 Python-Bindings für schnellen FFI- und NumPy-Austausch | 🟢 Autark |
| [`contextra-ingest`](crates/contextra-ingest/) | Multi-Format Dokumentextraktion (PDF, DOCX, XLSX, HTML) und Chunking | 🟢 Autark |
| [`contextra-audit-export`](crates/contextra-audit-export/) | Export-Werkzeuge für kryptografisch signierte Audit-Spuren | 🟢 Autark |
| [`contextra-avv-generator`](crates/contextra-avv-generator/) | Generierung von AVV-Vertragsvorlagen basierend auf technischen Garantien | 🟢 Autark |
| [`contextra-license`](crates/contextra-license/) | Kryptografische Lizenz- und Feature-Ring-Aktivierung | 🟢 Autark |
| [`contextra-testkit`](crates/contextra-testkit/) | In-Memory Referenzmodelle, Port-Konformitätstests und Auditharness | 🟢 Tooling |

---

## 🛠️ Bausteine kombinieren (Beispiele)

### Beispiel 1: Reiner Vektorspeicher mit HNSW (`contextra-vector`)
```rust
use contextra_vector::{HnswIndex, VectorConfig, MetricKind};
use contextra_types::TenantId;

// Vektorbaustein völlig ohne Gesamtsystem nutzen
let config = VectorConfig::new(128, MetricKind::Cosine);
let index = HnswIndex::new(config)?;
```

### Beispiel 2: Revisionssichere Lösch-Saga (`contextra-erasure`)
```rust
use contextra_erasure::saga::ErasureCore;
use contextra_ports::storage::StorageEngine;

// Der Löschbaustein orchestriert die Bereinigung über abstrakte Ports
// ohne direkte Abhängigkeit von konkreten Speicher-Engines.
```

### Beispiel 3: High-Level Client (`contextra`)
```rust
use contextra::ContextraBuilder;

let client = ContextraBuilder::new()
    .with_data_dir("./contextra_data")
    .build()
    .await?;
```

---

## 🧭 Dokumentation & Richtlinien

- [`AGENTS.md`](AGENTS.md): Leitfaden für Coding-Agenten und Architekten.
- [`PLAN.md`](PLAN.md): Detaillierter Refaktorisierungsplan zur modularen Entkopplung.
- [`DEPRECATED_DEBT.md`](DEPRECATED_DEBT.md): Bestandsaufnahme historischer Altlasten und Migrationsstatus.
- [`docs/SPEC.md`](docs/SPEC.md): Normative Systemspezifikation.
- [`docs/VETOES.md`](docs/VETOES.md): Architektonische Veto-Kriterien.
- [`docs/MODULAR_DECOUPLING_AUDIT.md`](docs/MODULAR_DECOUPLING_AUDIT.md): Automatisierter Entkopplungs- und Schnittstellenbericht.

---

## ⚖️ Lizenz

Lizenziert unter Apache-2.0 oder MIT (siehe `LICENSE-APACHE` und `LICENSE-MIT`).
