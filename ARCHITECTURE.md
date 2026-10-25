# Contextra Memory Engine — Systemarchitektur & Ring-Modell

Dieses Dokument ist die maßgebliche technische Architekturbeschreibung der Contextra Memory Engine. Es übersetzt die Normative Gesamtspezifikation (Fassung 1.2; §0–§34) in eine vertiefte Systembeschreibung, dokumentiert das Ring-Modell (§6.0), die Single-Node Ein-Prozess-Garantie (`cargo add contextra`), die Kantenregeln (§6.10), Rand-Brücken (§6.11), Unsafe-Inseln (§6.7) sowie die Kern-Geschlossenheit (§6.8).

---

## Inhaltsverzeichnis
1. [Ring-Modell (Ring 0–4) & Layering-Invarianten (§6.0, §6.10)](#1-ring-modell)
2. [Crate-Inventar & Normativitätsklassen (§6.0, `capabilities.toml`)](#2-crate-inventar)
3. [Rand-Brücken & Entkopplung (§6.11)](#3-rand-bruecken)
4. [Unsafe-Inseln, Invarianten & Locking-Disziplin (§5, §6.7, §27.6)](#4-unsafe-inseln)
5. [Referenzen & Weiterführende Dokumente](#5-referenzen)

---

<a id="1-ring-modell"></a>
## 1. Ring-Modell (Ring 0–4) & Layering-Invarianten (§6.0, §6.10)

Contextra gliedert seine Funktionalität in ein Fünf-Ring-Schichtenmodell (Ring 0 bis Ring 4; §6.0). Die Kantenregeln (§6.10) bestimmen verbindlich den erlaubten Abhängigkeitsfluss.

### 1.1 Definition der Ringe (§6.0)

* **Ring 0 — Foundation & Pure Numerics / Types**:
  Enthält kerneigenen Datenstrukturen, Typdefinitionen, Traits, Kryptographie, Mathematik, Vektor-, Volltext-, Graph- und Ranking-Primitive.
* **Ring 1 — Persistence & Cache**:
  Enthält Persistenz-Engines (LSM-Tree, WAL, Block-Cache, KV-Cache-Segmentdateien) und Checkpoint-Engines.
* **Ring 2 — External Adapters & Sandboxing**:
  Isoliert flüchtige oder externe Schwergewicht-Abhängigkeiten (`candle`, `ollama`, `ort`/ONNX, `wasmtime`). Ring 2 Crates sind Blätter im Graphen und hängen nie von Ring 1 oder 3 ab.
* **Ring 3 — Orchestration & Cognition**:
  Enthält die Geschäfts- und Orchestrierungslogik (Transaktions-Management, Collection-Engine, Gedächtniskonsolidierung, Egress-Vault/Privacy Gateway, Profil-Routing, Agent-Workflow).
* **Ring 4 — Boundary & Composition Roots**:
  Öffentliche Fassade (`contextra`), MCP-Server (`contextra-mcp`), Language-Bindings (`contextra-py`) und Lizenzschicht (`contextra-license`). Bildet die Composition Root.

### 1.2 DAG-Integrität & Kantenregeln (§6.10)

Das System erzwingt strikte Directed Acyclic Graph (DAG) Modularität.
1. **Fundament:** `types`, `ports` sowie die Inseln `wire`, `sys`, `simd` bilden das Fundament.
2. **Kerne untereinander:** Ring-0-Kerne (`vector`, `text`, `graph`, `rank`, `adapt`, `mvcc`, `crypto`) hängen nur dann voneinander ab, wenn die Kante in `capabilities.toml` deklariert ist.
3. **Ring 2:** Adapter (`infer-*`, `sandbox`) nutzen nur das Fundament.
4. **Ring 4:** `mcp` und `py` erreichen Ring 2 und 3 ausschließlich über die Fassade `contextra`; kein direkter Zugriff auf `crypto`, `store` oder Inferenz-Backends.
5. **Port-Besitz:** Jeder Port aus Anhang A ist in `ports` definiert; ein Trait mit Port-Rolle außerhalb von `ports` ist unzulässig.
6. **Geschlossenheit des Kerns (§6.8):** `[R]`- und `[E]`-Code dürfen die Beweislast, CI-Zeit oder Angriffsfläche des Kerns nicht vergrößern.

---

<a id="2-kern-geschlossenheit--rand-bruecken"></a>
## 2. Kern-Geschlossenheit (§6.8) & Rand-Brücken (§6.11)

`capabilities.toml` ist die **einzige** Quelle für Ring, Klasse (`class`: `kern` `[K]`, `rand` `[R]`, `experimentell` `[E]`) und `maturity` je Crate (INV-DOC-SINGLE-SOURCE-1):

| Crate-Name | Ring | Klasse (§6.0) | Status / Maturity (`capabilities.toml`) | Beschreibung (`capabilities.toml`) |
|---|---|---|---|---|
| `contextra` | Ring 4 | Kern `[K]` | `stable` | Haupt-Library Facade für Endanwender |
| `contextra-store` | Ring 1 | Kern `[K]` | `stable` | LSM-Tree Storage Engine, WAL Durability und Compaction Engine |
| `contextra-vector` | Ring 0 | Kern `[K]` | `stable` | Vektorsuche und Indexierung (HNSW, DiskANN, Quantisierung) |
| `contextra-text` | Ring 0 | Kern `[K]` | `stable` | Volltextsuche und Inverted-Index (BM25 Engine) |
| `contextra-types` | Ring 0 | Kern `[K]` | `stable` | Grundlegende Typdefinitionen (DocId, TxId, TenantId, Error-Typen) |
| `contextra-ports` | Ring 0 | Kern `[K]` | `stable` | Abstrakte Port-Schnittstellen und Trait-Definitionen für Entkopplung |
| `contextra-simd` | Ring 0 | Kern `[K]` | `stable` | SIMD-beschleunigte Distanzberechnungen und Vektor-Operationen |
| `contextra-sys` | Ring 0 | Kern `[K]` | `stable` | Systemnahe Bindings, Low-Level I/O und Memory-Mapping |
| `contextra-wire` | Ring 0 | Kern `[K]` | `stable` | FlatBuffers Wire-Protokolle und Serialisierungs-Formate |
| `contextra-mvcc` | Ring 0 | Kern `[K]` | `stable` | Multi-Version Concurrency Control und Isolation-Mechanismen |
| `contextra-core` | Ring 0 | Kern `[K]` | `stable` | Kern-Datenstrukturen, Tombstones, Invarianten und Memory-Buffer |
| `contextra-crypto` | Ring 0 | Kern `[K]` | `stable` | Kryptographische Vaults, Zeroize, Egress-Verschlüsselung und HMAC |
| `contextra-checkpoint` | Ring 1 | Kern `[K]` | `stable` | Persistent Checkpoint Engine und Blake3 Manifest-Verifikation |
| `contextra-engine` | Ring 3 | Kern `[K]` | `stable` | Compute Pool und Asynchrone Storage-Execution Layer |
| `contextra-db` | Ring 3 | Kern `[K]` | `stable` | Contextra Main Database Abstraction, Hybrid Search & Collections |
| `contextra-mcp` | Ring 4 | Kern `[K]` | `stable` | Model Context Protocol Server & Cloud Egress Gateway |
| `contextra-graph` | Ring 0 | Rand `[R]` | `stable` | CSR Graph-Engine, Path Graphing und GraphRAG Community Detection |
| `contextra-rank` | Ring 0 | Rand `[R]` | `stable` | Ranking-, Fusion- und Score-Kalibrierungs-Algorithmen |
| `contextra-adapt` | Ring 0 | Rand `[R]` | `experimental` | Anpassungs- und Transformations-Layer (PID Latency Controller) |
| `contextra-kvcache` | Ring 1 | Rand `[R]` | `stable` | Stufenmodell KV-Cache Storage & Offloading |
| `contextra-infer-candle` | Ring 2 | Rand `[R]` | `stable` | Candle LLM Inference Client und KV-Bridge Adapter |
| `contextra-infer-ollama` | Ring 2 | Rand `[R]` | `stable` | Ollama External Provider Client und Embeddings Integration |
| `contextra-sandbox` | Ring 2 | Rand `[R]` | `stable` | WASM Execution Boundary und Isolations-Sandbox |
| `contextra-cognition` | Ring 3 | Rand `[R]` | `stable` | Konsolidierungs- und Background Sleep Passes |
| `contextra-privacy` | Ring 3 | Rand `[R]` | `stable` | Egress Filtering, Anonymisierung und Privacy Compliance Rules |
| `contextra-py` | Ring 4 | Rand `[R]` | `stable` | PyO3 Python Bindings |
| `contextra-router` | Ring 3 | Experimentell `[E]` | `stable` | Lyapunov Drift Control und Multi-Candidate Search Routing Engine |
| `contextra-agent` | Ring 3 | Experimentell `[E]` | `stable` | Audit Engine, InMemoryStorageEngine und Agent Execution Pipeline |
| `contextra-infer-onnx` | Ring 2 | Experimentell `[E]` | `experimental` | ONNX Embeddings und Cross-Encoder Reranking Execution Provider |
| `contextra-audit-export` | Ring 0 | Experimentell `[E]` | `experimental` | Audit Log Export und DSGVO Art. 30 Verarbeitungsverzeichnis-Export |
| `contextra-avv-generator` | Ring 0 | Experimentell `[E]` | `experimental` | AVV Template Generator gemäß Art. 28 DSGVO |
| `contextra-license` | Ring 4 | Experimentell `[E]` | `experimental` | Lizenz- und Aktivierungsprüfung für Contextra Feature-Ringe |
| `contextra-testkit` | Tooling | Tooling | `stable` | Test Fixtures, Mock Engines und Chaos Matrix Harness |

---

<a id="3-rand-bruecken"></a>
## 3. Rand-Brücken & Entkopplung (§6.11)

Kern-Crates (`[K]`) erreichen Rand-Fähigkeiten (`[R]`) ausschließlich über Ports, die die Composition Root (`contextra`) bei aktiviertem Feature injiziert (P30). Fehlt eine Rand-Fähigkeit, verhält sich der Kern gewohnt stabil ohne Funktionalitätsausfall.

---

<a id="4-unsafe-inseln"></a>
## 4. Unsafe-Inseln, Invarianten & Locking-Disziplin (§5, §6.7, §27.6)

### 4.1 Zero-Panic-Doktrin & Unsafe-Inseln (§6.7)

Standardmäßig gilt in allen Nicht-Insel-Crates `#![forbid(unsafe_code)]`. Es gibt genau **drei zulässige Unsafe-Inseln** im Produktionscode (§6.7):

1. **`contextra-sys`**: Low-Level OS-Operationen (`mmap`, `mlock`, Win32-ACLs).
2. **`contextra-simd`**: SIMD-Vektordistanzberechnungskerne (AVX2, AVX-512, NEON) mit Laufzeit-Dispatch.
3. **`contextra-wire`**: FlatBuffers-generierter Code für Zero-Copy-Transfers.

Jeder `unsafe`-Block in diesen Inseln muss zwingend mit einem `// SAFETY:`-Kommentar begründet werden (§6.7).

### 4.2 Sperrenhierarchie & Lock-Ordnung (INV-LOCK-ORDER-1)

Zur Vermeidung von Deadlocks gilt im gesamten System eine strikte Sperrenhierarchie:

```
storage.wal (RwLock Read-Guard) → pending_commit_queue (Mutex) → index_finalizer (Lock)
```

---

<a id="5-referenzen"></a>
## 5. Referenzen & Weiterführende Dokumente

* **Normative Spezifikation Fassung 1.2**: [`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`](docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md)
* **Agenten-Betriebsanleitung**: `AGENTS.md`
* **Sicherheitsrichtlinie**: `SECURITY.md`
