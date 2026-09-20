# MemFuse Cognitive OS — Systemarchitektur & Ring-Modell

Dieses Dokument ist die maßgebliche technische Architekturbeschreibung des MemFuse Cognitive OS. Es übersetzt die normative Gesamtspezifikation (`README.md`) in eine vertiefte Systembeschreibung, dokumentiert den tatsächlichen Crate-Bestand, das Ring-0–4-Modell, die Layering-Invarianten und die bekannten Abweichungen zwischen dem Soll- und Ist-Zustand.

---

## Inhaltsverzeichnis
1. [Ring-Modell (Ring 0–4) & Layering-Invarianten](#1-ring-modell)
2. [Crate-Inventar (Ist-Zustand)](#2-crate-inventar)
3. [Abhängigkeitsdiagramm](#3-abhaengigkeitsdiagramm)
4. [Bekannte Abweichungen IST vs. SOLL](#4-bekannte-abweichungen-ist-vs-soll)
5. [Unsafe-Inseln, Invarianten & Locking-Disziplin](#5-unsafe-inseln-invarianten--locking-disziplin)
6. [Referenzen & Weiterführende Dokumente](#6-referenzen)

---

<a id="1-ring-modell"></a>
## 1. Ring-Modell (Ring 0–4) & Layering-Invarianten

MemFuse gliedert seine Funktionalität in ein Fünf-Ring-Schichtenmodell (Ring 0 bis Ring 4). Abhängigkeiten dürfen ausschließlich **von höheren Ringen auf tiefere Ringe** verlaufen. Aufwärtskanten (z. B. ein Ring-0-Crate, das von einem Ring-3-Crate abhängt) sind streng verboten.

### 1.1 Definition der Ringe

* **Ring 0 — Foundation & Pure Numerics / Types**:
  Enthält kerneigenen Datenstrukturen, Typdefinitionen, Traits, Kryptographie, Mathematik, Vektor- und Graph-Primitive. Ring 0 enthält **kein `tokio`** (Grundsatz *Sync-Kern, async-Schale*, P26).
* **Ring 1 — Persistence & Cache**:
  Enthält Persistenz-Engines (LSM-Tree, WAL, Block-Cache, KV-Cache-Segmentdateien) und Time-Travel-Registries. Async-I/O ist an den Grenzen erlaubt.
* **Ring 2 — External Adapters & Sandboxing (Leaf Engines)**:
  Isoliert flüchtige oder externe Schwergewicht-Abhängigkeiten (`candle`, `ollama`, `ort`/ONNX, `wasmtime`). Ring 2 Crates sind Blätter im Graphen und hängen nie von Ring 1 oder 3 ab.
* **Ring 3 — Orchestration & Cognition**:
  Enthält die Geschäfts- und Orchestrierungslogik (Transaktions-Management, Collection-Engine, Synthese, PII-Vault/Privacy Gateway, Profil-Routing, Agent-Workflow).
* **Ring 4 — Boundary & Composition Roots**:
  Öffentliche Fassade (`memfuse`), MCP-Server (`memfuse-mcp`) und Language-Bindings (`memfuse-py`). Bildet die einzige Composition Root.

### 1.2 Verbot von Aufwärtskanten (DAG-Integrität, P5)

Das System erzwingt strikte Directed Acyclic Graph (DAG) Modularität.

**Konkretes Negativ-Beispiel für eine verbotene Aufwärtskante:**
In einer früheren Version hing das Datenbank-Crate `memfuse-db` (damals Layer 2 / Ring 3) direkt von `memfuse-candle` und `memfuse-ollama` (damals Layer 3 / Ring 2) ab, um Embedding-Backends direkt zu instanziieren. Dies verletzte P5, da eine Kern-Engine von konkreten Inferenz-Adaptern abhing.
Im Zielmodell (`ARCHITECTURE.md` / `README.md` §4.2) erhält die Engine stattdessen Trait-Objekte (`Arc<dyn Embedder>`) aus `memfuse-ports` (Ring 0), und die konkrete Verdrahtung erfolgt ausschließlich in Ring 4 (`memfuse` Fassade).

---

<a id="2-crate-inventar"></a>
## 2. Crate-Inventar (Ist-Zustand)

Die folgende Tabelle führt alle 27 im Repository unter `crates/` vorgefundenen Fach-Crates auf, eingeordnet in das Ring-Modell basierend auf ihrem tatsächlichen Stand:

| Crate-Name | Ring | Verantwortlichkeit (1 Satz) | Status |
|---|---|---|---|
| `memfuse-types` | 0 | Identifikatoren (`DocId`, `TxId`, `TenantId`), Filter-AST und Kernel-Budgets. | Fertig |
| `memfuse-ports` | 0 | `dyn`-kompatible Trait-Definitionen für Storage, Indizes, Embedder und System-Uhren. | Fertig |
| `memfuse-mvcc` | 0 | `SeqLog`, `SnapshotRegistry` und transaktionaler `TxBuffer`. | Fertig |
| `memfuse-wire` | 0 | FlatBuffers-IPC-Generat und Zero-Copy-Adapter (Unsafe-Insel). | Fertig |
| `memfuse-sys` | 0 | System-Abstraktionen für `mmap`, `mlock` und Win32-ACLs (Unsafe-Insel). | Fertig |
| `memfuse-simd` | 0 | AVX2/AVX-512/NEON SIMD-Distanzberechnungskerne mit Laufzeit-Dispatch (Unsafe-Insel). | Fertig |
| `memfuse-crypto` | 0 | AES-256-GCM-SIV, WAL-HMAC-Integritätsketten, Zeroize und Deletion-Proofs. | In Migration |
| `memfuse-vector` | 0 | HNSW- und DiskANN-Vektorindex-Implementierungen (vormals `memfuse-index`). | In Migration |
| `memfuse-text` | 0 | BM25/BM25F-Volltextindexierung und deutsche Kompositazerlegung. | Fertig |
| `memfuse-graph` | 0 | Compressed Sparse Row (CSR) Graph, Forward-Push PPR, Leiden und Hyperkanten. | Fertig |
| `memfuse-rank` | 0 | 4-Signal-Retrieval-Fusion, Score-Kalibrierung und Drift-Messung. | In Migration |
| `memfuse-adapt` | 0 | LinUCB-Bandit-Mathematik, Lyapunov-Drift-Wächter und PID-Regler. | Fertig |
| `memfuse-store` | 1 | LSM-Tree Storage Engine mit WAL-Group-Commit und HMAC-Integritätsprüfung. | Fertig |
| `memfuse-kvcache` | 1 | Verschlüsselter Prefix-Radix-Baum und KV-Cache-Segment-Verwaltung. | Fertig |
| `memfuse-checkpoint` | 1 | Time-Travel-Registry und Checkpoint-Verwaltung ohne globalen Zustand. | Fertig |
| `memfuse-infer-candle` | 2 | Native GGUF-Modell-Inferenz via Candle (vormals `memfuse-candle`). | In Migration |
| `memfuse-infer-ollama` | 2 | HTTP-Inferenz-Client für Ollama mit Contextual-Chunk-Prefixing (vormals `memfuse-ollama`). | In Migration |
| `memfuse-infer-onnx` | 2 | ONNX-Embeddings und Reranking via `ort` (vormals `memfuse-embed`). | In Migration |
| `memfuse-sandbox` | 2 | WASM-Ausführungs-Isolation mit Fuel- und Wall-Clock-Limits via Wasmtime. | Fertig |
| `memfuse-engine` | 3 | Collection-LSM-Anbindung, Multi-Index-Pläne und Schreibtransaktionen (vormals Teil von `memfuse-db`). | In Migration |
| `memfuse-cognition` | 3 | Hintergrund-Kompaktierung, Synthese und Konsolidierungs-Scheduler. | In Migration |
| `memfuse-privacy` | 3 | Cloud-Egress Privacy Gateway, PII-Vault, Surrogat-Tokenisierung und DLP. | In Migration |
| `memfuse-router` | 3 | SLM-Profil-Routing und MCP-Dispatch (Numerik nach `adapt` ausgelagert). | In Migration |
| `memfuse-agent` | 3 | Agenten-Workflow-Engine mit auditierbarer State-Machine und DLQ. | Fertig |
| `memfuse` | 4 | Öffentliche Haupt-Fassade und Composition Root für Rust-Anwendungen. | Fertig |
| `memfuse-mcp` | 4 | Stdio-JSON-RPC MCP-Server-Protokoll-Adapter. | In Migration |
| `memfuse-py` | 4 | PyO3 Python-FFI-Bindings. | In Migration |

---

<a id="3-abhaengigkeitsdiagramm"></a>
## 3. Abhängigkeitsdiagramm

Das folgende Mermaid-Diagramm bildet die tatsächlichen `[dependencies]` zwischen allen Crates unter `crates/` zum Ausführungszeitpunkt ab:

```mermaid
graph TD
    memfuse_adapt[memfuse-adapt]
    memfuse_agent[memfuse-agent] --> memfuse_core[memfuse-core]
    memfuse_agent[memfuse-agent] --> memfuse_db[memfuse-db]
    memfuse_agent[memfuse-agent] --> memfuse_graph[memfuse-graph]
    memfuse_agent[memfuse-agent] --> memfuse_checkpoint[memfuse-checkpoint]
    memfuse_agent[memfuse-agent] --> memfuse_store[memfuse-store]
    memfuse_agent[memfuse-agent] --> memfuse_router[memfuse-router]
    memfuse_calibration[memfuse-calibration] --> memfuse_core[memfuse-core]
    memfuse_calibration[memfuse-calibration] --> memfuse_adapt[memfuse-adapt]
    memfuse_candle[memfuse-candle] --> memfuse_core[memfuse-core]
    memfuse_candle[memfuse-candle] --> memfuse_calibration[memfuse-calibration]
    memfuse_candle[memfuse-candle] --> memfuse_crypto[memfuse-crypto]
    memfuse_candle[memfuse-candle] --> memfuse_store[memfuse-store]
    memfuse_checkpoint[memfuse-checkpoint] --> memfuse_core[memfuse-core]
    memfuse_core[memfuse-core] --> memfuse_types[memfuse-types]
    memfuse_core[memfuse-core] --> memfuse_ports[memfuse-ports]
    memfuse_core[memfuse-core] --> memfuse_mvcc[memfuse-mvcc]
    memfuse_core[memfuse-core] --> memfuse_wire[memfuse-wire]
    memfuse_crypto[memfuse-crypto] --> memfuse_core[memfuse-core]
    memfuse_db[memfuse-db] --> memfuse_sys[memfuse-sys]
    memfuse_db[memfuse-db] --> memfuse_core[memfuse-core]
    memfuse_db[memfuse-db] --> memfuse_crypto[memfuse-crypto]
    memfuse_db[memfuse-db] --> memfuse_store[memfuse-store]
    memfuse_db[memfuse-db] --> memfuse_index[memfuse-index]
    memfuse_db[memfuse-db] --> memfuse_text[memfuse-text]
    memfuse_db[memfuse-db] --> memfuse_checkpoint[memfuse-checkpoint]
    memfuse_db[memfuse-db] --> memfuse_graph[memfuse-graph]
    memfuse_db[memfuse-db] --> memfuse_calibration[memfuse-calibration]
    memfuse_db[memfuse-db] --> memfuse_adapt[memfuse-adapt]
    memfuse_embed[memfuse-embed] --> memfuse_core[memfuse-core]
    memfuse_embed[memfuse-embed] --> memfuse_calibration[memfuse-calibration]
    memfuse_embed[memfuse-embed] --> memfuse_candle[memfuse-candle]
    memfuse_graph[memfuse-graph] --> memfuse_core[memfuse-core]
    memfuse_index[memfuse-index] --> memfuse_core[memfuse-core]
    memfuse_index[memfuse-index] --> memfuse_crypto[memfuse-crypto]
    memfuse_index[memfuse-index] --> memfuse_simd[memfuse-simd]
    memfuse_kvcache[memfuse-kvcache] --> memfuse_core[memfuse-core]
    memfuse_kvcache[memfuse-kvcache] --> memfuse_crypto[memfuse-crypto]
    memfuse_mcp[memfuse-mcp] --> memfuse_db[memfuse-db]
    memfuse_mcp[memfuse-mcp] --> memfuse_core[memfuse-core]
    memfuse_mcp[memfuse-mcp] --> memfuse_crypto[memfuse-crypto]
    memfuse_mcp[memfuse-mcp] --> memfuse_ollama[memfuse-ollama]
    memfuse_mcp[memfuse-mcp] --> memfuse_router[memfuse-router]
    memfuse_mcp[memfuse-mcp] --> memfuse_calibration[memfuse-calibration]
    memfuse_mcp[memfuse-mcp] --> memfuse_embed[memfuse-embed]
    memfuse_mcp[memfuse-mcp] --> memfuse_agent[memfuse-agent]
    memfuse_mcp[memfuse-mcp] --> memfuse_candle[memfuse-candle]
    memfuse_mvcc[memfuse-mvcc] --> memfuse_types[memfuse-types]
    memfuse_ollama[memfuse-ollama] --> memfuse_core[memfuse-core]
    memfuse_ollama[memfuse-ollama] --> memfuse_calibration[memfuse-calibration]
    memfuse_ports[memfuse-ports] --> memfuse_types[memfuse-types]
    memfuse_py[memfuse-py] --> memfuse_router[memfuse-router]
    memfuse_py[memfuse-py] --> memfuse_calibration[memfuse-calibration]
    memfuse_py[memfuse-py] --> memfuse_core[memfuse-core]
    memfuse_py[memfuse-py] --> memfuse_db[memfuse-db]
    memfuse_router[memfuse-router] --> memfuse_core[memfuse-core]
    memfuse_router[memfuse-router] --> memfuse_adapt[memfuse-adapt]
    memfuse_router[memfuse-router] --> memfuse_store[memfuse-store]
    memfuse_router[memfuse-router] --> memfuse_db[memfuse-db]
    memfuse_sandbox[memfuse-sandbox] --> memfuse_core[memfuse-core]
    memfuse_simd[memfuse-simd] --> memfuse_core[memfuse-core]
    memfuse_store[memfuse-store] --> memfuse_sys[memfuse-sys]
    memfuse_store[memfuse-store] --> memfuse_core[memfuse-core]
    memfuse_store[memfuse-store] --> memfuse_crypto[memfuse-crypto]
    memfuse_testkit[memfuse-testkit] --> memfuse_core[memfuse-core]
    memfuse_text[memfuse-text] --> memfuse_core[memfuse-core]
    memfuse[memfuse] --> memfuse_core[memfuse-core]
    memfuse[memfuse] --> memfuse_db[memfuse-db]
    memfuse[memfuse] --> memfuse_candle[memfuse-candle]
    memfuse[memfuse] --> memfuse_ollama[memfuse-ollama]
    memfuse[memfuse] --> memfuse_embed[memfuse-embed]
    memfuse[memfuse] --> memfuse_router[memfuse-router]
```

---

<a id="4-bekannte-abweichungen-ist-vs-soll"></a>
## 4. Bekannte Abweichungen IST vs. SOLL

Während der schrittweisen Strangler-Migration (§20) existieren vorübergehende Diskrepanzen zwischen dem Zielmodell laut `README.md` und dem vorgefundenen Repository-Stand:

1. **`memfuse-core` als Fassaden-Re-Export:**
   * *SOLL:* `memfuse-core` entfällt als monolithisches Crate vollständig.
   * *IST:* `memfuse-core` existiert im Repo als Re-Export-Fassade über `memfuse-types`, `memfuse-ports`, `memfuse-mvcc` und `memfuse-wire`, um Abwärtskompatibilität während der Migration zu sichern.
2. **`memfuse-db` Re-Export & Aufwärtskante in `memfuse-router`:**
   * *SOLL:* `memfuse-db` wird in `engine`/`cognition`/`rank`/`adapt`/`router`/`privacy` zerlegt. `memfuse-router` darf nicht von `memfuse-db` abhängen.
   * *IST:* `memfuse-db` existiert weiterhin als Fassade. `memfuse-router` importiert noch `memfuse-db` (dokumentiertes Audit: `docs/refactor/router-db-edge-audit.md`).
3. **Inferenz-Namenskonvention:**
   * *SOLL:* Umbenennung in `memfuse-infer-candle`, `memfuse-infer-ollama` und `memfuse-infer-onnx`.
   * *IST:* Crates heißen aktuell noch `memfuse-candle`, `memfuse-ollama` und `memfuse-embed`.
4. **Vektorindex-Namenskonvention:**
   * *SOLL:* Umbenennung in `memfuse-vector`.
   * *IST:* Crate heisst aktuell noch `memfuse-index`.
5. **Legacy-Calibration-Crate:**
   * *SOLL:* `memfuse-calibration` geht in `memfuse-adapt` und `memfuse-rank` auf.
   * *IST:* `memfuse-calibration` existiert aktuell noch als eigenständiges Crate.

---

<a id="5-unsafe-inseln-invarianten--locking-disziplin"></a>
## 5. Unsafe-Inseln, Invarianten & Locking-Disziplin

### 5.1 Zero-Panic-Doctrine & Unsafe-Inseln

Standardmäßig gilt in allen Nicht-Insel-Crates `#![forbid(unsafe_code)]`. Es gibt genau **drei zulässige Unsafe-Inseln** im Produktionscode, die per `#![allow(unsafe_code)]` ausgenommen sind:

1. **`memfuse-sys`**: Abstraktionen für Low-Level-OS-Operationen (`mmap`, `mlock`, Win32-ACLs).
2. **`memfuse-simd`**: SIMD-Vektordistanzberechnungskerne (AVX2, AVX-512, NEON) mit Laufzeit-Dispatch.
3. **`memfuse-wire`**: FlatBuffers-generierter Code, der konstruktionsbedingt `unsafe` Blöcke für Zero-Copy-Transfers benötigt.

Jeder `unsafe`-Block in diesen Inseln muss zwingend mit einem `// SAFETY:`-Kommentar begründet werden.

### 5.2 Locking-Disziplin & Sperrenhierarchie

Zur Vermeidung von Deadlocks gilt im gesamten System eine strikte Sperrenhierarchie:

```
collections (RwLock) → kv_locks (schlüssel-granular via KvKeyLocks) → embedder (RwLock)
```

* **Sperren-Reihenfolge**: Wenn mehrere Locks akquiriert werden müssen, geschieht dies ausnahmslos von links nach rechts.
* **Key-Granulares Locking (`KvKeyLocks`)**: Schreibzugriffe auf Schlüsselebene nutzen Sharded Locks (`RwLock<()>`). Bei Mehrschlüssel-Operationen (`acquire_multi_sorted`) werden die Shard-Indizes zwingend **aufsteigend sortiert** akquiriert, um Zyklen im Wait-For-Graph auszuschließen.
* **Kein Async unter Sync-Locks**: Kein `.await`-Aufruf darf gehalten werden, während ein synchroner Mutex/RwLock-Guard existiert.

---

<a id="6-referenzen"></a>
## 6. Referenzen & Weiterführende Dokumente

* **Normative Spezifikation**: `README.md`
* **Entwickler- & Agent-Anweisungen**: `AGENTS.md`
* **Refactoring- & Audit-Protokolle**:
  * `docs/refactor/router-db-edge-audit.md` (Audit der `router` → `db` Abhängigkeit)
  * `docs/refactor/mcp-upward-edges-audit.md` (Audit der MCP-Gateway-Schichten)
