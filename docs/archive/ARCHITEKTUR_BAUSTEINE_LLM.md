> Nicht normativ, widerlegt durch SPEC v4.0 §1

# Architektur & Transformationsplan: Modulare Baustein-Bibliothek für Rust-LLM- & Enterprise-Systeme

> **Dokumentenstatus:** Umfassendes Architektur-, Analyse- und Migrationskonzept
> **Geltungsbereich:** Workspace-weite Restrukturierung zur autarken Baustein-Bibliothek (Toolbox)

---

## 1. Phase 1: Analysebericht (Ist-Zustand, Kopplung & Risiken)

### 1.1 Ist-Zustand des Repositories
Das Repository umfasst derzeit **37 Crates** mit insgesamt **252.448 Zeilen Rust-Code** (`src/`). Es entstand über den Verlauf eines Jahres intensiver Entwicklung mit dem Ziel, ein vollumfängliches, vertikal integriertes RAG- und Speichersystem für Large Language Models bereitzustellen.

Die Codebasis zeichnet sich durch hohe algorithmische Qualität aus (robuste HNSW-Vektorindizes, SIMD-Abstandsberechnungen, BM25-Volltextsuche, GGUF-/ONNX-Inferenz, deterministische KV-Caches, POSIX-Fsync-Durability und WASM-Sandboxing). Allerdings wurde diese Funktionalität in ein monolithisches Schichtenmodell gezwungen, das eine flexible Entnahme einzelner Komponenten drastisch erschwert.

#### Crate-Bestandsübersicht (sortiert nach Codeumfang)
| Crate | LOC (`src/`) | Quelldateien | Interne Workspace-Abhängigkeiten | Primäre Fachdomäne |
|---|:---:|:---:|---|---|
| `contextra-store` | 28.294 | 77 | `types`, `ports`, `sys`, `mvcc`, `crypto` | LSM-Tree Storage Engine, WAL, Compaction |
| `contextra-engine` | 22.246 | 84 | `types`, `ports`, `mvcc`, `crypto`, `sandbox`, `infer-onnx`, `adapt`, `store`, `vector`, `text`, `checkpoint`, `rank`, `graph` | Zentraler Monolith-Kleber & Orchestrierung |
| `contextra-graph` | 21.504 | 73 | `types`, `ports` | CSR-Graph, Entity-Relationen, GraphRAG |
| `contextra-ingest` | 20.445 | 77 | `types` | Universal-Parser: PDF, DOCX, XLSX, HTML, CSV |
| `contextra-vector` | 14.587 | 45 | `types`, `ports`, `mvcc`, `simd`, `sys` | HNSW-Vektorindex, Graph-Repair |
| `contextra-erasure` | 11.870 | 61 | `types`, `ports`, `crypto` | DSGVO-Saga, Revisionssicherheit, Vault |
| `contextra-crypto` | 8.217 | 26 | `types`, `ports` | AES-256-GCM-SIV, HKDF, BLAKE3-Ketten |
| `contextra-types` | 8.199 | 25 | *keine (autark)* | Kanonische Datentypen, Newtypes, IDs |
| `contextra-text` | 8.016 | 26 | `types`, `ports` | BM25-Index, Unicode-Tokenizer, Stemmer |
| `contextra-router` | 7.775 | 28 | `types`, `wire`, `ports`, `adapt` | Intent-Routing, Model-Cascading |
| `contextra-cognition` | 7.330 | 24 | `engine`, `graph`, `ports`, `types` | Memory-Consolidation, Kurz-/Langzeitgedächtnis |
| `contextra-ports` | 6.694 | 41 | `types` | Trait-Definitionen (`dyn`-kompatibel) |
| `contextra-mcp` | 6.377 | 24 | `contextra`, `crypto`, `infer-candle`, `infer-ollama`, `infer-onnx`, `ports`, `privacy`, `rank`, `router`, `types`, `wire` | Model Context Protocol Server & Tools |
| `contextra-infer-candle` | 6.135 | 17 | `crypto`, `ports`, `rank`, `types` | Lokale GGUF-Inferenz via HuggingFace Candle |
| `contextra-kvcache` | 5.995 | 11 | `crypto`, `ports`, `types` | Prefix-Radix-Tree, Attention-State-Cache |
| `contextra-infer-ollama` | 5.942 | 16 | `ports`, `types` | Async Client für Ollama REST/UDS-Endpunkte |
| `contextra-agent` | 5.555 | 18 | `checkpoint`, `db`, `graph`, `ports`, `router`, `types` | StateGraph Workflow-Engine, ReAct-Loop |
| `contextra-privacy` | 5.147 | 11 | `ports`, `types` | PII-Erkennung (IBAN, E-Mail), Egress-Vault |
| `contextra-adapt` | 5.069 | 13 | `ports`, `types` | PID-Regler, Banditen, Adaptive Dynamic Batching |
| `contextra-checkpoint` | 4.957 | 11 | `ports`, `types` | Snapshot-Management, RAII Transaction Guards |
| `contextra-rank` | 4.548 | 21 | `ports`, `types` | Reciprocal Rank Fusion (RRF), Platt-Scaling |
| `contextra-mvcc` | 4.139 | 5 | `ports`, `types` | MVCC Snapshot Registry, Sequence-Log |
| `contextra-testkit` | 3.967 | 20 | `ports`, `types` | Trait-Konformitätstests, Fault-Injection |
| `contextra-bench` | 3.591 | 9 | `db`, `store`, `text`, `vector` | LoCoMo & LongMemEval Benchmark-Suiten |
| `contextra-py` | 3.529 | 16 | `contextra`, `types`, `wire` | PyO3 Python-Bindings |
| `contextra-simd` | 3.189 | 7 | `types` | AVX2/AVX-512/NEON Distanzmetriken |
| `contextra-wire` | 3.079 | 6 | *keine (autark)* | FlatBuffers Framing, gehärtetes JSON-RPC 2.0 |
| `contextra-infer-onnx` | 2.955 | 6 | `ports`, `rank`, `types` | ONNX Runtime Embeddings & Cross-Encoder |
| `contextra-sandbox` | 2.376 | 13 | *keine (autark)* | Wasmtime WASM-Isolation mit Fuel & Limits |
| `contextra-db` | 2.123 | 5 | `engine`, `ports`, `types` | Monolithische Datenbank-Fassade |
| `contextra` | 2.082 | 7 | `db`, `infer-candle`, `license`, `ports`, `rank`, `types` | Top-Level Composition Root |
| `contextra-sys` | 1.970 | 7 | *keine (autark)* | Safe OS Facades (`mem`, `durability`, `fsmeta`, `mmap`) |
| `contextra-durable-fs` | 1.699 | 6 | `ports`, `sys`, `types` | `DirLock`, atomare Schreiboperationen |
| `contextra-avv-generator` | 1.022 | 9 | `types` | Generator für AVV-Verträge (Markdown) |
| `contextra-license` | 840 | 2 | `ports`, `types` | Ed25519 Offline-Lizenzprüfung |
| `contextra-audit-export` | 729 | 5 | `types` | DSGVO Art. 30 Verarbeitungsverzeichnis |
| `contextra-core` | 256 | 1 | `mvcc`, `ports`, `types`, `wire` | *Veraltete Strangler Facade (nicht in Workspace)* |

---

### 1.2 Der Kopplungsgraph & Die 3 zentralen Flaschenhälse

```
              ┌──────────────────────────────────────────────┐
              │               contextra-types                │
              │  (Zentraler Flaschenhals 1: 32 Konsumenten)  │
              └──────────────────────┬───────────────────────┘
                                     │
              ┌──────────────────────▼───────────────────────┐
              │               contextra-ports                │
              │  (Zentraler Flaschenhals 2: 26 Konsumenten)  │
              └──────────────────────┬───────────────────────┘
                                     │
     ┌───────────────────────────────┼──────────────────────────────┐
     │                               │                              │
┌────▼─────────────┐     ┌───────────▼────────────┐     ┌───────────▼─────────────┐
│ contextra-vector │     │    contextra-store     │     │     contextra-text      │
│ (Vektorsuche)    │     │    (LSM-Tree On-Disk)  │     │     (BM25 Volltext)     │
└────┬─────────────┘     └───────────┬────────────┘     └───────────┬─────────────┘
     │                               │                              │
     └───────────────────────────────┼──────────────────────────────┘
                                     │
              ┌──────────────────────▼───────────────────────┐
              │              contextra-engine                │
              │ (Zentraler Flaschenhals 3: Monolith-Kleber)  │
              │           13 interne Abhängigkeiten          │
              └──────────────────────────────────────────────┘
```

#### Flaschenhals 1: Monolithischer Datentyp-Container (`contextra-types`)
- **Befund:** 32 von 37 Crates binden `contextra-types` ein. In diesem Crate liegen sowohl fundamentale Primitiven (`DocId`, `DistanceMetric`), Token-Budgets, als auch hochspezifische Filterbäume, Modell-Fingerabdrücke und veraltete Löschtypen nebeneinander.
- **Problem für den Baustein-Nutzer:** Wer lediglich den HNSW-Index oder den SIMD-Kernel in ein neues Projekt kopieren möchte, muss 8.199 Zeilen Fremdcode und Typstrukturen mitschleppen.

#### Flaschenhals 2: Übergreifende Trait-Indirektion (`contextra-ports`)
- **Befund:** 26 Crates konsumieren `contextra-ports`. Dieses Crate definiert über 40 Traits (`StorageEngine`, `VectorIndex`, `TextIndex`, `Clock`, `Rng`, `Vfs`, `ErasureLayer`, etc.) mit dynamischer `BoxFuture`-Dispatch-Kompatibilität.
- **Problem für den Baustein-Nutzer:** Module können nicht unabhängig voneinander instanziiert werden. Um einen Inferenz-Client zu nutzen, muss das `EmbeddingProvider`-Trait aus `contextra-ports` implementiert werden, anstatt einfache Rust-Typen (`async fn embed(&self, text: &str) -> Vec<f32>`) bereitzustellen.

#### Flaschenhals 3: Monolithische Integrationsschicht (`contextra-engine` & `contextra`)
- **Befund:** `contextra-engine` verknüpft 13 interne Crates fest miteinander. Es existiert kein Mechanismus, um z. B. nur Textsuche + Vektorsuche ohne die gesamte LSM-Tree-Engine (`contextra-store`) oder ohne das WASM-Sandbox-Subsystem zu betreiben.

---

### 1.3 Problemliste nach Priorität

| Priorität | Problem | Betroffene Komponenten | Risiko / Auswirkung |
|:---:|---|---|---|
| **P1** | **Typen-Monolith:** Fehlende Autarkie der Kernkomponenten | `contextra-types`, `contextra-simd`, `contextra-ingest` | Hoher Aufwand beim Herauslösen; Copy-Paste erfordert Kaskade von Workspace-Dateien. |
| **P1** | **Trait-Korsett:** Zwanghafte dynamische Abstraktion (`dyn Port`) | `contextra-ports`, Inferenz-Crates, Storage | Überkomplexe Schnittstellen (`Pin<Box<dyn Future>>`), Inkompatibilität mit einfachen Drittprojekten. |
| **P2** | **Kompilierfehler in veralteten Modulen:** `contextra-store` Importfehler | `contextra-store/src/lsm/engine.rs` | Workspace-weites `cargo check` scheitert an ungelösten `ResourceTracker`-Imports. |
| **P2** | **Verwaiste Crates im Dateisystem:** `contextra-core` | `crates/contextra-core` | Verwirrung über kanonische Typenquellen; Altlast ohne Workspace-Zugehörigkeit. |
| **P3** | **Produkt-Spezifika in Infrastruktur:** Deletion-Proofs & DSGVO | `contextra-vector`, `contextra-text`, `contextra-crypto` | Verunreinigung reiner RAG- und Mathematik-Algorithmen mit bürokratischen Audit-Anforderungen. |
| **P3** | **Schwere optionale Abhängigkeiten ohne Feature-Gating:** ONNX, Candle | `contextra-infer-*` | Hohe Build-Zeiten und C-Bibliotheksabhängigkeiten, selbst wenn nur einfache Abfragen gebraucht werden. |

---

## 2. Phase 2: Zielarchitektur (Die private Rust-Toolbox)

### 2.1 Architektur-Prinzipien der Toolbox
1. **Zero-Knowledge-Kompilierbarkeit:** Jeder primäre Baustein muss als eigenständiges Crate mit reinen Standard-Rust-Typen (`std::vec::Vec`, `std::string::String`, `&[f32]`, `std::time::Instant`) kompilieren.
2. **Lokale Fehlerdefinition:** Keine globalen Sammel-Fehlertypen (`ContextraError`). Jedes Modul deklariert sein eigenes, stark typisiertes `enum Error` via `thiserror`.
3. **Flache Hierarchie & Standard-Traits:** Direkte Rückgabetypen statt `BoxFuture`. Lokale Traits für Inversion of Control.
4. **Feingranulare Feature-Flags:** Schwere Backends (ONNX, Wasmtime, SIMD, RocksDB/LSM) müssen opt-in sein.

---

### 2.2 Modul-Cluster & Ziel-Bausteine

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                        DIE PRIVATE RUST-BAUSTEIN-TOOLBOX                               │
├──────────────────────────┬──────────────────────────┬──────────────────────────────────┤
│ 1. SYSTEM & HARDWARE     │ 2. VEKTOR & RAG          │ 3. INFERENZ & MODELLE            │
│  - sys_primitives        │  - simd_metrics          │  - infer_onnx (Feature: onnx)    │
│  - durable_fs (atomic)   │  - hnsw_vector_index     │  - infer_candle (Feature: gguf)  │
│  - wasm_sandbox          │  - bm25_text_index       │  - infer_ollama (async REST/UDS) │
│  - wire_buffer (IPC/RPC) │  - rank_fusion (RRF)     │  - kvcache_radix                 │
├──────────────────────────┼──────────────────────────┼──────────────────────────────────┤
│ 4. AGENTEN & ROBOTIK     │ 5. DATA & INGESTION      │ 6. SICHERHEIT & DATENSCHUTZ      │
│  - adapt_controllers     │  - ingest_universal      │  - crypto_envelope               │
│  - state_graph_workflow  │  - csr_graph             │  - privacy_guard (DLP/PII)       │
│  - intent_router         │  - mvcc_registry         │  - audit_log                     │
└──────────────────────────┴──────────────────────────┴──────────────────────────────────┘
```

---

### 2.3 Würdigung bisheriger Architekturentscheidungen (Trade-off-Analyse)

| Bisherige Entscheidung | Vorteile im alten Monolithen | Nachteile für die Baukasten-Nutzung | Neue Entscheidung |
|---|---|---|---|
| **Zentrales `contextra-types`** | Einheitliche Definition aller Typen, garantiert typgleiche Interoperabilität über 36 Crates. | Enorme Kopplung. Jede Änderung erfordert Rebuild des gesamten Repositories. Nicht herauskopierbar. | **Auflösen in lokale Typen.** Wo Interoperabilität nötig ist, genügen Standardtypen (`f32`, `&str`, `u64`). |
| **Globales `contextra-ports` mit Dyn-Traits** | Ermöglichte Mocking und Austauschbarkeit in End-to-End-Tests über Ringgrenzen hinweg. | Runtime-Overhead durch dynamischen Dispatch (`BoxFuture`), unlesbare Signaturen, Trait-Verflechtung. | **Lokale Inversion-of-Control.** Module definieren nur Schnittstellen, die sie konsumieren. |
| **Feste Kopplung von Vector/Text an Deletion-Proofs** | Garantierte mathematische Nachweisbarkeit DSGVO-konformer Löschung im Enterprise-Produkt. | Zerstört Wiederverwendbarkeit in Robotik oder RAG, wo Append-Only oder einfaches Truncate ausreicht. | **Kryptografische Löschung isolieren.** `contextra-erasure` bleibt spezialisierter Satellit. |
| **All-in-One Engine-Orchestrierung** | Ein zentraler Aufruf initialisierte Storage, Vektoren, Volltext und Inferenz. | Erzwang die Einbindung aller Subsysteme, selbst wenn man nur einen In-Memory-Vektorindex wollte. | **Beispiel-basierte Komposition.** Bereitstellung von schlanken Beispielen (`examples/`) statt starrem Kleber. |

---

### 2.4 Qualitätsgarantien (Enterprise & Robotik)

#### A. Speichersicherheit (`unsafe`)
- **Prinzip:** Null `unsafe` außerhalb von `contextra-sys` (OS-Abstraktionen) und `contextra-simd` (Hardware-Intrinsics).
- **Vorgabe:** Jedes verbleibende `unsafe` besitzt eine explizite `// SAFETY:`-Begründung, die Pointer-Gültigkeit, Alignment und Aliasing-Regeln formal belegt.

#### B. Concurrency & Send/Sync
- **Prinzip:** Bevorzuge unveränderliche Zustände und atomare Übergänge (`ArcSwap`, `AtomicU64`) gegenüber grobgranularen Locks (`Mutex`).
- **Deadlock-Prävention:** In `mvcc` und `kvcache` gilt strikte Lock-Reihenfolge: Read-Locks dürfen niemals innerhalb gehaltener Write-Locks angefordert werden.

#### C. Durability & Fsync-Verhalten
- **Prinzip:** Im Persistenz-Baustein (`durable-fs` / `store`) muss jeder Schreibpfad atomar sein: Tempfile schreiben -> `fsync` des Inhalts -> Rename -> `fsync` des Elternverzeichnisses.
- **Fail-Closed:** Persistenzfehler (z. B. fehlschlagendes `fsync`) vergiften den Dateideskriptor permanent, um "Silent Corruption" im Crash-Fall auszuschließen.

#### D. Determinismus
- **Prinzip:** Gleiche Eingabe = Bitgenau gleicher Zustand. In Re-Ranking (`rank`), KV-Caching (`kvcache`) und SIMD-Kerneln werden keine nicht-deterministischen Hashes (`RandomState`) auf kritischen Pfaden verwendet. Stattdessen kommen BLAKE3 oder `BTreeMap` zum Einsatz.

---

## 3. Phase 3: Migrationsplan pro Crate (Maßnahmenmatrix)

| Crate | Geplante Maßnahme | Begründung | Risiko | Aufwand |
|---|---|---|:---:|:---:|
| `contextra-sys` | **Beibehalten (Autark)** | Bereits 100% unabhängig, sichere OS-Abstraktionen für Memory-Locking und Fsync. | Sehr gering | Minimal (Dokumentation) |
| `contextra-simd` | **Entkoppeln von `types`** | Entfernen der `contextra-types`-Abhängigkeit. Einbetten eines lokalen `DistanceMetric`-Enums. | Gering | 0.5 Tage |
| `contextra-sandbox` | **Beibehalten (Autark)** | Perfekt autarke WASM-Sandbox mit Fuel-Metering. Ideal für sichere Tool-Ausführung. | Sehr gering | Minimal |
| `contextra-wire` | **Beibehalten (Autark)** | Schnelle Zero-Copy FlatBuffers und gehärtetes JSON-RPC 2.0. | Sehr gering | Minimal |
| `contextra-durable-fs`| **Entkoppeln von `ports`**| Ersetzen der Port-Traits durch direkte POSIX-Aufrufe via `sys`. | Gering | 1 Tag |
| `contextra-vector` | **Bereinigen & Isolieren**| Entfernen alter Deletion-Proofs. HNSW autark mit In-Memory- und Disk-Persistenz anbieten. | Mittel | 2 Tage |
| `contextra-text` | **Bereinigen & Isolieren**| BM25-Volltextsuche und Tokenizer als autarken Volltextbaustein isolieren. | Mittel | 1.5 Tage |
| `contextra-rank` | **Isolieren (Standalone)** | Entfernen von `ports`. RRF- und Platt-Scaling als mathematische Lib anbieten. | Sehr gering | 0.5 Tage |
| `contextra-infer-onnx`| **Feature-Gating** | ONNX Runtime hinter Feature `onnx` kapseln. Unabhängiger Embedding-/Cross-Encoder-Worker. | Gering | 1 Tag |
| `contextra-infer-candle`| **Feature-Gating** | Candle-Inferenz hinter Feature `candle` kapseln. Autarker GGUF-Runner. | Gering | 1 Tag |
| `contextra-infer-ollama`| **Entkoppeln von `ports`**| Leichtgewichtiger async REST-Client für Ollama mit eigenem Fehler-Enum. | Sehr gering | 0.5 Tage |
| `contextra-kvcache` | **Krypto entflechten** | Radix-Tree-Caching als primäre In-Memory-Datenstruktur bereitstellen. Krypto optional. | Gering | 1 Tag |
| `contextra-adapt` | **Isolieren für Robotik** | PID-Regler, Banditen und dynamische Regelkreise vollkommen entkoppeln. | Gering | 0.5 Tage |
| `contextra-graph` | **Ports generifizieren** | CSR-Graph-Datenstrukturen generisch machen (`Graph<NodeId, EdgeData>`). | Mittel | 2 Tage |
| `contextra-ingest` | **Lokales Model-Modul** | Typen aus `contextra-types` lokal in `ingest` einbetten. Universeller Parser-Baustein. | Gering | 1 Tag |
| `contextra-privacy` | **Stand-Alone Filter** | PII-Erkennung und Egress-Sanitizer als universelle Middleware für Prompts bereitstellen. | Gering | 0.5 Tage |
| `contextra-agent` | **Workflow entflechten** | StateGraph von spezifischen DB- und Checkpoint-Verdrahtungen lösen. | Mittel | 2 Tage |
| `contextra-router` | **Intent-Dispatcher** | Unabhängiges Routing von Anfragen an LLM-Endpunkte. | Gering | 1 Tag |
| `contextra-store` | **Kompilierfehler beheben**| Fehlende `ResourceTracker`-Imports korrigieren; als robuster On-Disk LSM-Tree bereithalten. | Mittel | 1.5 Tage |
| `contextra-core` | **Löschen (Deprecate)** | Bereits aus Workspace-Members entfernt; Ordner rückstandslos entfernen. | Keine | 5 Minuten |
| `contextra-engine` | **Fassade minimieren** | Keine Weiterentwicklung als Monolith-Zentrum. Dient als optionales Integrationsbeispiel. | Gering | Beibehalten als Legacy |

---

## 4. Phase 4: Umsetzungsstrategie & Code-Ebene-Änderungen

### 4.1 Vereinheitlichte Fehlerbehandlung (No Panics, Pure `thiserror`)
Bisher führten Fehler in manchen Hilfsmodulen zu Panics oder unübersichtlichen `ContextraError`-Kaskaden. Zukünftiger Standard für alle Bausteine:

```rust
// Vorbild: Baustein-spezifische Fehler in rust-llm-vector
#[derive(Debug, thiserror::Error)]
pub enum VectorIndexError {
    #[error("Vector dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Vector element contains invalid float value (NaN or Infinity)")]
    InvalidVectorData,

    #[error("Node ID not found in HNSW graph: {0}")]
    NodeNotFound(u64),

    #[error("IO operation failed during index persistence: {0}")]
    Io(#[from] std::io::Error),
}
```

### 4.2 Schlanke Feature-Flags in `Cargo.toml`
Um Abhängigkeiten für Endprojekte minimal zu halten, werden schwere Bibliotheken gekapselt:

```toml
# Beispiel: Cargo.toml für den universellen Inferenz-Baustein
[package]
name = "my-llm-infer"
version = "0.1.0"
edition = "2021"

[features]
default = ["ollama"]
ollama = ["dep:reqwest"]
onnx = ["dep:ort"]
candle = ["dep:candle-core", "dep:candle-transformers"]

[dependencies]
thiserror = "2.0"
tokio = { version = "1.40", features = ["rt", "sync"] }

# Optionale schwere Abhängigkeiten
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"], optional = true }
ort = { version = "2.0", optional = true }
candle-core = { version = "0.8", optional = true }
candle-transformers = { version = "0.8", optional = true }
```

---

## 5. Phase 5: Test- & Benchmark-Isolationskonzept

### 5.1 Das modulare Test-Paradigma
Bisher wurden Integrationstests oft nur im Verbund des gesamten Monolithen ausgeführt. Für die Baustein-Toolbox gilt das Prinzip der **strengen Isolation**:
- **Autarke Unit-Tests (`src/**`):** Verifizieren interne Invarianten und Edge-Cases direkt im Baustein.
- **Isolierte Integrationstests (`tests/**`):** Testen die öffentliche API des Bausteins, ohne dass andere Workspace-Crates kompiliert werden müssen.
- **Property-Based Testing (`proptest`):** Mathematische Invarianten (z. B. Kosinus-Distanz $d(u, v) \in [0, 2]$, Symmetrie $d(u, v) = d(v, u)$) werden durch Zufallsvektoren automatisiert abgesichert.

### 5.2 Benchmark-Harness (`benches/`)
Performancekritische Bausteine erhalten Crate-lokale Criterion-Benchmarks:
1. **`contextra-simd`**:
   - Durchsatzmessung (GFLOPS) für L2-, Dot- und Cosine-Distanz über 128-, 384-, 768- und 1536-dimensionale Vektoren.
2. **`contextra-vector`**:
   - Abfrage-Latenz (p50, p99) für HNSW-$k$-NN-Suche bei 100.000 Vektoren.
   - Konstruktionszeit (Vektoren pro Sekunde) bei Indexaufbau.
3. **`contextra-kvcache`**:
   - Lookup-Latenz im Prefix-Radix-Tree bei 10.000 parallelen Konversations-Sessions.
   - Speicherverbrauch pro gecachtem Token.

---

## 6. Phase 6: Abschlussbericht & Baustein-Steckbriefe

### 6.1 Sofort einsatzfähige Bausteine (Copy-Paste-Ready)

#### 1. `contextra-sys` (Low-Level OS & Memory Safety)
- **Zweck:** Bereitstellung speichersicherer OS-Primitives für POSIX und Windows.
- **Highlights:** Sicheres Memory-Locking (`mlock` gegen Swap-Auslagerung sensibler Schlüssel/Prompts), Fsync für Dateien und Verzeichnisse, Dateizugriffsrechte (ACLs).
- **Abhängigkeiten:** Absolut **keine** internen Abhängigkeiten. Sofort kopierbar.

#### 2. `contextra-sandbox` (WASM Tool-Execution Boundary)
- **Zweck:** Isolierte und ressourcenlimitierte Ausführung unvertrauenswürdiger Skripte oder LLM-generierter Werkzeuge.
- **Highlights:** Wasmtime-basiert, strikte Default-Deny-Richtlinie (kein Netzwerk, kein Dateisystem), Fuel-Metering gegen Endlosschleifen, Stack- und Memory-Limits.
- **Abhängigkeiten:** Keine internen Abhängigkeiten. Sofort kopierbar.

#### 3. `contextra-wire` (Gehärtetes RPC & IPC)
- **Zweck:** Schnelle Binär- und JSON-Serialisierung für Kommunikation zwischen Services oder Prozessen.
- **Highlights:** FlatBuffers Zero-Copy Parser, JSON-RPC 2.0 Schutz gegen Rekursionsbomben (Tiefenlimitierung), Erkennung doppelter Keys.
- **Abhängigkeiten:** Keine internen Abhängigkeiten. Sofort kopierbar.

#### 4. `contextra-adapt` (Regelungstechnik & Adaptive Controller für Robotik)
- **Zweck:** Mathematische Regelkreise für Robotik-Aktoren und dynamisches Batching.
- **Highlights:** PID-Regler mit Anti-Windup, Multi-Armed Bandits (Thompson-Sampling, UCB1), Kalman-Filter-Grundlagen.
- **Abhängigkeiten:** Nach leichtem Lösen von `ports` vollkommen autark.

#### 5. `contextra-privacy` (Prompt-DLP & PII-Maskierung)
- **Zweck:** Schutz vor Datenabfluss in externen LLM-APIs (OpenAI, Anthropic).
- **Highlights:** Regex- und Checksum-basierte Erkennung von IBANs (Mod-97), Kreditkarten (Luhn), deutschen Steuer-IDs, E-Mails, Telefonnummern und Homoglyphen-Angriffen.
- **Abhängigkeiten:** Nahezu autark, hervorragend als Middleware vor LLM-Aufrufen geeignet.

---

### 6.2 Offene Punkte & Empfohlene nächste Schritte

1. **Behebung der Kompilierfehler in `contextra-store`:**
   - In `crates/contextra-store/src/lsm/engine.rs` und `recovery.rs` müssen die fehlenden Importe für `ResourceTracker` und `ResourceBudget` auf `contextra_types::ResourceTracker` angepasst werden, damit der gesamte Workspace wieder fehlerfrei via `cargo check` durchläuft.
2. **Physikalisches Löschen von `crates/contextra-core`:**
   - Bereinigung des Dateisystems durch Entfernen dieses verwaisten Ordners.
3. **Schrittweise Entflechtung von `contextra-simd`:**
   - Entfernen von `contextra-types` aus `crates/contextra-simd/Cargo.toml` und Ersetzen durch lokale Enums für maximale Portabilität.
4. **Erstellung von Standalone-Examples:**
   - Ergänzung einfacher `examples/*_standalone.rs` in `contextra-vector` und `contextra-text`, die zeigen, wie Indexe in unter 20 Zeilen Code direkt instanziiert werden.

---
*Ende des Architektur- und Transformationsdokuments.*
