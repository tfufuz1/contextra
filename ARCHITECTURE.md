# Contextra Memory Engine — Systemarchitektur & Ring-Modell

Dieses Dokument ist die maßgebliche technische Architekturbeschreibung der Contextra Memory Engine. Es übersetzt die normative Gesamtspezifikation (`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`) in eine vertiefte Systembeschreibung, dokumentiert das Ring-Modell (§6.0), die Single-Node Ein-Prozess-Garantie (`cargo add contextra`, REQ-ARCH-001), die Layering-Invarianten und das Inferenz-Modell (§14.0, Candle als Standard).

---

## Inhaltsverzeichnis
1. [Ring-Modell (Ring 0–4) & Layering-Invarianten (§6.0, §6.10)](#1-ring-modell)
2. [Crate-Inventar & Normativitätsklassen](#2-crate-inventar)
3. [Unsafe-Inseln, Invarianten & Locking-Disziplin (§5.0, §27.6)](#3-unsafe-inseln-invarianten--locking-disziplin)
4. [Referenzen & Weiterführende Dokumente](#4-referenzen)

---

<a id="1-ring-modell"></a>
## 1. Ring-Modell (Ring 0–4) & Layering-Invarianten (§6.0, §6.10)

Contextra gliedert seine Funktionalität in ein Fünf-Ring-Schichtenmodell (Ring 0 bis Ring 4; §6.0). Abhängigkeiten dürfen ausschließlich **von höheren Ringen auf tiefere Ringe** verlaufen (Kantenregel §6.10). Aufwärtskanten (z. B. ein Ring-0-Crate, das von einem Ring-3-Crate abhängt) sind streng verboten.

### 1.1 Definition der Ringe (§6.0)

* **Ring 0 — Foundation & Pure Numerics / Types**:
  Enthält kerneigenen Datenstrukturen, Typdefinitionen, Traits, Kryptographie, Mathematik, Vektor-, Volltext-, Graph- und Ranking-Primitive. Ring 0 enthält **kein `tokio`** (Sync-Kern, async-Schale).
* **Ring 1 — Persistence & Cache**:
  Enthält Persistenz-Engines (LSM-Tree, WAL, Block-Cache, KV-Cache-Segmentdateien) und Checkpoint-Engines. Async-I/O ist an den Grenzen erlaubt.
* **Ring 2 — External Adapters & Sandboxing**:
  Isoliert flüchtige oder externe Schwergewicht-Abhängigkeiten (`candle`, `ollama`, `ort`/ONNX, `wasmtime`). Ring 2 Crates sind Blätter im Graphen und hängen nie von Ring 1 oder 3 ab.
* **Ring 3 — Orchestration & Cognition**:
  Enthält die Geschäfts- und Orchestrierungslogik (Transaktions-Management, Collection-Engine, Gedächtniskonsolidierung, Egress-Vault/Privacy Gateway, Profil-Routing, Agent-Workflow).
* **Ring 4 — Boundary & Composition Roots**:
  Öffentliche Fassade (`contextra`), MCP-Server (`contextra-mcp`), Language-Bindings (`contextra-py`) und Lizenzschicht (`contextra-license`). Bildet die Composition Root.

### 1.2 DAG-Integrität & Entkopplung über Ports (§6.10, REQ-ARCH-003)

Das System erzwingt strikte Directed Acyclic Graph (DAG) Modularität.
Engine und Speicher-Engines erhalten Inferenz- und Host-Dienste ausschließlich über Trait-Objekte aus `contextra-ports` (Ring 0). Konkrete Verdrahtung erfolgt erst in Ring 4 (`contextra` Fassade).

---

<a id="2-crate-inventar"></a>
## 2. Crate-Inventar & Normativitätsklassen (§6.0, `capabilities.toml`)

`capabilities.toml` ist die **einzige** Quelle für Ring, Klasse (`class`: `kern` `[K]`, `rand` `[R]`, `experimentell` `[E]`) und `maturity` je Crate (INV-DOC-SINGLE-SOURCE-1, REQ-GOV-004):

| Crate-Name | Ring | Klasse | Maturity | Beschreibung (§6.0, `capabilities.toml`) |
|---|---|---|---|---|
| `contextra` | Ring 4 | Kern `[K]` | `stable` | Haupt-Library Facade (§22.1, REQ-FAC-001) |
| `contextra-types` | Ring 0 | Kern `[K]` | `stable` | Typen, IDs, Budgets, Fehlerklassen (§6.0, REQ-TYPES-001) |
| `contextra-ports` | Ring 0 | Kern `[K]` | `stable` | Trait-Schnittstellen für Port-Entkopplung (§6.0, REQ-ARCH-003) |
| `contextra-simd` | Ring 0 | Kern `[K]` | `stable` | SIMD Distanzberechnungen (Unsafe Island; §6.0, REQ-ARCH-015) |
| `contextra-sys` | Ring 0 | Kern `[K]` | `stable` | Systemnahe Low-Level I/O Bindings (Unsafe Island; §6.0) |
| `contextra-wire` | Ring 0 | Kern `[K]` | `stable` | FlatBuffers Wire-Protokolle (Unsafe Island; §12.0) |
| `contextra-mvcc` | Ring 0 | Kern `[K]` | `stable` | MVCC, SSI Tracking & Read-Sets (§8.1, REQ-MVCC-001) |
| `contextra-core` | Ring 0 | Kern `[K]` | `stable` | Kern-Buffer & Strangler-Reexporte (§6.0) |
| `contextra-crypto` | Ring 0 | Kern `[K]` | `stable` | AES-256-GCM-SIV, HMAC, DeletionProof v4 (§20.1, REQ-KEY-001) |
| `contextra-vector` | Ring 0 | Kern `[K]` | `stable` | Vektorindex HNSW, DiskANN, SQ8 (§9.0, REQ-IDX-001) |
| `contextra-text` | Ring 0 | Kern `[K]` | `stable` | Volltextsuche BM25/BM25F (§10.1, REQ-IDX-005) |
| `contextra-graph` | Ring 0 | Kern `[K]` | `stable` | CSR Wissensgraph & PPR (§11.1, REQ-IDX-008) |
| `contextra-rank` | Ring 0 | Kern `[K]` | `stable` | RRF Signal-Fusion & Kalibrierung (§12.1, REQ-RET-001) |
| `contextra-store` | Ring 1 | Kern `[K]` | `stable` | LSM Storage Engine & WAL v3/v4 (§7.1, REQ-DUR-001) |
| `contextra-kvcache` | Ring 1 | Ring 1 / Kern `[K]` | `stable` | Tiered KV-Cache & Segment Storage (§13.1, REQ-MEM-001) |
| `contextra-checkpoint` | Ring 1 | Kern `[K]` | `stable` | Checkpoint Engine & Manifests (§16.1, REQ-DUR-010) |
| `contextra-infer-candle` | Ring 2 | Rand `[R]` | `stable` | Pure Rust GGUF/Candle Inferenz (§14.1, REQ-INF-001) |
| `contextra-infer-ollama` | Ring 2 | Rand `[R]` | `stable` | Ollama External Provider Client (§14.2, REQ-INF-002) |
| `contextra-infer-onnx` | Ring 2 | Rand `[R]` | `experimental` | ONNX Embedding & Reranker (§14.3, REQ-INF-003) |
| `contextra-sandbox` | Ring 2 | Rand `[R]` | `stable` | Wasmtime Isolation Sandbox (§15.1, REQ-SBX-001) |
| `contextra-engine` | Ring 3 | Kern `[K]` | `stable` | Compute Pool & Storage Execution (§7.2, REQ-DUR-011) |
| `contextra-cognition` | Ring 3 | Kern `[K]` | `stable` | Konsolidierung & Maintenance (§17.1, REQ-COG-001) |
| `contextra-privacy` | Ring 3 | Rand `[R]` | `stable` | Egress-Vault & PII-Filter (§19.1, REQ-PRV-001) |
| `contextra-db` | Ring 3 | Kern `[K]` | `stable` | Main DB Abstraction & Collections (§22.1) |
| `contextra-agent` | Ring 3 | Kern `[K]` | `stable` | Persistent Agent Workflows & Audit Loop (§16.2, REQ-AUD-001) |
| `contextra-mcp` | Ring 4 | Kern `[K]` | `stable` | MCP Server über stdio (§21.1, REQ-MCP-001) |
| `contextra-py` | Ring 4 | Kern `[K]` | `stable` | PyO3 Python Bindings (§22.2) |
| `contextra-adapt` | Ring 0 | Experimentell `[E]` | `experimental` | Adaptive Regler (PID / LinUCB) (§18.1, REQ-EXP-001) |
| `contextra-router` | Ring 3 | Experimentell `[E]` | `stable` | Drift & Cascade Router (§18.2, REQ-EXP-002) |
| `contextra-audit-export` | Ring 0 | Experimentell `[E]` | `experimental` | GDPR Art. 30 Audit-Export (§20.10, REQ-EXP-004) |
| `contextra-avv-generator` | Ring 0 | Experimentell `[E]` | `experimental` | Art. 28 AVV-Template-Generator (§20.11, REQ-EXP-005) |
| `contextra-license` | Ring 4 | Experimentell `[E]` | `experimental` | Lizenzschicht & Ring-Gating (§23.5, REQ-EXP-003) |
| `contextra-testkit` | Tooling | Tooling | `stable` | Test Fixtures & FaultVfs (§27.3) |

---

<a id="3-unsafe-inseln-invarianten--locking-disziplin"></a>
## 3. Unsafe-Inseln, Invarianten & Locking-Disziplin (§5.0, §27.6)

### 3.1 Zero-Panic-Doktrin & Unsafe-Inseln (§5.0)

Standardmäßig gilt in allen Nicht-Insel-Crates `#![forbid(unsafe_code)]`. Es gibt genau **drei zulässige Unsafe-Inseln** im Produktionscode, die per `#![allow(unsafe_code)]` ausgenommen sind (INV-RING0-DETERMINISM-1, REQ-ARCH-015):

1. **`contextra-sys`**: Abstraktionen für Low-Level-OS-Operationen (`mmap`, `mlock`, Win32-ACLs).
2. **`contextra-simd`**: SIMD-Vektordistanzberechnungskerne (AVX2, AVX-512, NEON) mit Laufzeit-Dispatch.
3. **`contextra-wire`**: FlatBuffers-generierter Code für Zero-Copy-Transfers.

Jeder `unsafe`-Block in diesen Inseln muss zwingend mit einem `// SAFETY:`-Kommentar begründet werden.

### 3.2 Sperrenhierarchie & Lock-Ordnung (INV-LOCK-ORDER-1, REQ-MVCC-004)

Zur Vermeidung von Deadlocks gilt im gesamten System eine strikte Sperrenhierarchie:

```
storage.wal (RwLock Read-Guard) → pending_commit_queue (Mutex) → index_finalizer (Lock)
```

Sperren müssen ausnahmslos in dieser Reihenfolge erworben werden. Kein `.await`-Aufruf darf gehalten werden, während ein synchroner Mutex/RwLock-Guard existiert.

---

<a id="4-referenzen"></a>
## 4. Referenzen & Weiterführende Dokumente

* **Normative Spezifikation**: [`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`](docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md)
* **Agenten-Betriebsanleitung**: `AGENTS.md`
* **Sicherheitsrichtlinie**: `SECURITY.md`
