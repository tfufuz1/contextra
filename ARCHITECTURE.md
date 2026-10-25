# Contextra Memory Engine — Systemarchitektur & Ring-Modell

Dieses Dokument ist die maßgebliche technische Architekturbeschreibung der Contextra Memory Engine. Es übersetzt die normative **Spezifikation 1.2** (`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`, §0–§34) in eine vertiefte Systembeschreibung, dokumentiert das Ring-Modell (§6.0), die Single-Node Ein-Prozess-Garantie (`cargo add contextra`, REQ-ARCH-001), die Layering-Invarianten und das Inferenz-Modell (§14.0).

---

## Inhaltsverzeichnis
1. [Ring-Modell (Ring 0–4) & Layering-Invarianten (§6.0, §6.10)](#1-ring-modell)
2. [Kern-Geschlossenheit (§6.8) & Rand-Brücken (§6.11)](#2-kern-geschlossenheit--rand-bruecken)
3. [Crate-Inventar & Normativitätsklassen (§6.0, `capabilities.toml`)](#3-crate-inventar)
4. [Unsafe-Inseln (§6.7) & Locking-Disziplin (INV-LOCK-ORDER-1, §5.0, §27.6)](#4-unsafe-inseln--locking-disziplin)
5. [Referenzen & Weiterführende Dokumente](#5-referenzen)

---

<a id="1-ring-modell"></a>
## 1. Ring-Modell (Ring 0–4) & Layering-Invarianten (§6.0, §6.10)

Contextra gliedert seine Funktionalität in ein Fünf-Ring-Schichtenmodell (Ring 0 bis Ring 4; §6.0). Die Kantenregeln (§6.10) bestimmen verbindlich den erlaubten Abhängigkeitsfluss.

### 1.1 Definition der Ringe (§6.0)

* **Ring 0 — Foundation & Pure Numerics / Types (§6.0):**
  Enthält kerneigene Datenstrukturen, Typdefinitionen (`contextra-types`), Traits (`contextra-ports`), Kryptographie (`contextra-crypto`), Mathematik, Vektor- (`contextra-vector`), Volltext- (`contextra-text`), Graph- (`contextra-graph`), Rank- (`contextra-rank`), Adaptiv-Regler (`contextra-adapt`), MVCC (`contextra-mvcc`), System-Bindings (`contextra-sys`), SIMD-Kernel (`contextra-simd`), Wire-Protokolle (`contextra-wire`), Core-Reexporte (`contextra-core`) sowie Compliance-Auszüge (`contextra-audit-export`, `contextra-avv-generator`). Ring 0 ist synchron und frei von Async-Runtimes (Kern-Geschlossenheit, §6.8).
* **Ring 1 — Persistence & Cache (§6.0):**
  Enthält Persistenz-Engines (LSM-Tree, WAL, Block-Cache; `contextra-store`), Tiered KV-Cache (`contextra-kvcache`) und Checkpoint-Engines (`contextra-checkpoint`). Async-I/O ist an den Grenzen erlaubt.
* **Ring 2 — External Adapters & Sandboxing (§6.0):**
  Isoliert flüchtige oder externe Schwergewicht-Abhängigkeiten (`contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-sandbox`). Ring 2 Crates sind funktionale Blätter im Graphen und hängen nie von Ring 1 oder 3 ab (§6.10).
* **Ring 3 — Orchestration & Cognition (§6.0):**
  Enthält die Geschäfts- und Orchestrierungslogik (Collection-Engine `contextra-db`, Compute Pool `contextra-engine`, Gedächtniskonsolidierung `contextra-cognition`, Egress-Vault `contextra-privacy`, Drift-Router `contextra-router`, Agent-Workflow `contextra-agent`).
* **Ring 4 — Boundary & Composition Roots (§6.0):**
  Öffentliche Fassade (`contextra`), MCP-Server (`contextra-mcp`), Language-Bindings (`contextra-py`) und Lizenzschicht (`contextra-license`). Bildet die Composition Root.

### 1.2 Kantenregeln & DAG-Integrität (§6.10, REQ-ARCH-003)

1. **Unidirektionalität (§6.10):** Abhängigkeiten dürfen ausschließlich **von höheren Ringen auf tiefere Ringe** verlaufen (Ring 4 → Ring 3 → Ring 1 → Ring 0).
2. **Keine Aufwärtskanten (§6.10):** Aufwärtskanten (z. B. ein Ring-0-Crate, das von Ring 1, 2, 3 oder 4 abhängt) sind als Architekturdefekt streng verboten.
3. **Blatt-Isolierung für Ring 2 (§6.10):** Inferenz- und Sandbox-Adapter in Ring 2 dürfen nicht untereinander oder von Ring 1/3 abhängen (ausgenommen spezifizierte Dev-Allowlists).

---

<a id="2-kern-geschlossenheit--rand-bruecken"></a>
## 2. Kern-Geschlossenheit (§6.8) & Rand-Brücken (§6.11)

### 2.1 Kern-Geschlossenheit (§6.8)

Der Kern (Ring 0) ist architektonisch geschlossen:
- **Keine Async-Runtime in Ring 0 (§6.8, P26):** Ring 0 Crates sind rein synchron konzipiert und enthalten keine direkte Abhängigkeit von `tokio` oder anderen Async-Runtimes.
- **Pure Numerics & Determinismus (§6.8, P28):** Mathematische Kerne und Index-Traversierungen arbeiten deterministisch. Zeit, Zufall und Nebeneffekte werden ausschließlich über injizierte Port-Traits (`Clock`, `Rng`, `IdGenerator` aus `contextra-ports`) bereitgestellt.

### 2.2 Rand-Brücken (§6.11)

Um die Entkopplung nach P5 und REQ-ARCH-003 zu garantieren, kommunizieren Kern und Orchestrierung mit externen Rand-Diensten (Ring 2) ausschließlich über dyn-safe Trait-Objekte (Ports) aus `contextra-ports` (Ring 0):
- **Port-Schnittstellen (§6.11):** `Embedder`, `LlmTextGenerator`, `MergeOperatorHost`, `ToolSandbox` definieren die Schnittstellen in Ring 0.
- **Injektion & Facade (§6.11):** Engine (`contextra-engine`) und DB (`contextra-db`) hängen nur von den Trait-Abstraktionen ab. Konkrete Implementierungen aus Ring 2 (`contextra-infer-candle`, `contextra-sandbox`) werden erst in der Composition Root (`contextra` Facade, Ring 4) verdrahtet.

---

<a id="3-crate-inventar"></a>
## 3. Crate-Inventar & Normativitätsklassen (§6.0, `capabilities.toml`)

`capabilities.toml` ist die **einzige** Quelle für Ring, Klasse (`Kern [K]`, `Rand [R]`, `Experimentell [E]`) und `maturity` (`stable`, `experimental`) je Crate (INV-DOC-SINGLE-SOURCE-1, REQ-GOV-004):

| Crate-Name | Ring | Klasse | Maturity | Beschreibung (§6.0, `capabilities.toml`) |
|---|---|---|---|---|
| **`contextra`** | Ring 4 | Kern `[K]` | `stable` | Haupt-Library Facade (§22.1, REQ-FAC-001) |
| **`contextra-adapt`** | Ring 0 | Experimentell `[E]` | `experimental` | Adaptive Regler (PID / LinUCB) (§18.1, REQ-EXP-001) |
| **`contextra-agent`** | Ring 3 | Kern `[K]` | `stable` | Agent Workflow & Audit Loop (§16.2, REQ-AUD-001) |
| **`contextra-audit-export`** | Ring 0 | Experimentell `[E]` | `experimental` | DSGVO Art. 30 Audit-Export (§20.10, REQ-EXP-004) |
| **`contextra-avv-generator`** | Ring 0 | Experimentell `[E]` | `experimental` | Art. 28 AVV-Template-Generator (§20.11, REQ-EXP-005) |
| **`contextra-checkpoint`** | Ring 1 | Kern `[K]` | `stable` | Persistent Checkpoint Engine (§16.1, REQ-DUR-010) |
| **`contextra-cognition`** | Ring 3 | Kern `[K]` | `stable` | Gedächtniskonsolidierung & Maintenance (§17.1, REQ-COG-001) |
| **`contextra-core`** | Ring 0 | Kern `[K]` | `stable` | Kern-Buffer & Strangler-Reexporte (§6.0) |
| **`contextra-crypto`** | Ring 0 | Kern `[K]` | `stable` | AES-256-GCM-SIV, HMAC, DeletionProof v4 (§20.1, REQ-KEY-001) |
| **`contextra-db`** | Ring 3 | Kern `[K]` | `stable` | Main DB Abstraction & Collections (§22.1) |
| **`contextra-engine`** | Ring 3 | Kern `[K]` | `stable` | Compute Pool & Storage Execution (§7.2, REQ-DUR-011) |
| **`contextra-graph`** | Ring 0 | Kern `[K]` | `stable` | CSR Wissensgraph & PPR (§11.1, REQ-IDX-008) |
| **`contextra-infer-candle`** | Ring 2 | Rand `[R]` | `stable` | Pure Rust GGUF/Candle Inferenz (§14.1, REQ-INF-001) |
| **`contextra-infer-ollama`** | Ring 2 | Rand `[R]` | `stable` | Ollama External Provider Client (§14.2, REQ-INF-002) |
| **`contextra-infer-onnx`** | Ring 2 | Rand `[R]` | `experimental` | ONNX Embedding & Reranker (§14.3, REQ-INF-003) |
| **`contextra-kvcache`** | Ring 1 | Kern `[K]` | `stable` | Tiered KV-Cache & Segment Storage (§13.1, REQ-MEM-001) |
| **`contextra-license`** | Ring 4 | Experimentell `[E]` | `experimental` | Lizenzschicht & Ring-Gating (§23.5, REQ-EXP-003) |
| **`contextra-mcp`** | Ring 4 | Kern `[K]` | `stable` | MCP Server über stdio (§21.1, REQ-MCP-001) |
| **`contextra-mvcc`** | Ring 0 | Kern `[K]` | `stable` | MVCC, SSI Tracking & Read-Sets (§8.1, REQ-MVCC-001) |
| **`contextra-ports`** | Ring 0 | Kern `[K]` | `stable` | Trait-Schnittstellen für Port-Entkopplung (§6.0, REQ-ARCH-003) |
| **`contextra-privacy`** | Ring 3 | Rand `[R]` | `stable` | Egress-Vault & PII-Filter (§19.1, REQ-PRV-001) |
| **`contextra-py`** | Ring 4 | Kern `[K]` | `stable` | PyO3 Python Bindings (§22.2) |
| **`contextra-rank`** | Ring 0 | Kern `[K]` | `stable` | RRF Signal-Fusion & Kalibrierung (§12.1, REQ-RET-001) |
| **`contextra-router`** | Ring 3 | Experimentell `[E]` | `stable` | Cascade & Drift Router Engine (§18.2, REQ-EXP-002) |
| **`contextra-sandbox`** | Ring 2 | Rand `[R]` | `stable` | Wasmtime Isolation Sandbox (§15.1, REQ-SBX-001) |
| **`contextra-simd`** | Ring 0 | Kern `[K]` | `stable` | SIMD Distanzberechnungen (Unsafe Island; §6.0, REQ-ARCH-015) |
| **`contextra-store`** | Ring 1 | Kern `[K]` | `stable` | LSM Storage Engine & WAL v3/v4 (§7.1, REQ-DUR-001) |
| **`contextra-sys`** | Ring 0 | Kern `[K]` | `stable` | Systemnahe Low-Level I/O Bindings (Unsafe Island; §6.0) |
| **`contextra-testkit`** | Tooling | Tooling | `stable` | Test Fixtures & FaultVfs (§27.3) |
| **`contextra-text`** | Ring 0 | Kern `[K]` | `stable` | Volltextsuche BM25/BM25F (§10.1, REQ-IDX-005) |
| **`contextra-types`** | Ring 0 | Kern `[K]` | `stable` | Typen, IDs, Budgets, Fehlerklassen (§6.0, REQ-TYPES-001) |
| **`contextra-vector`** | Ring 0 | Kern `[K]` | `stable` | Vektorindex HNSW, DiskANN, SQ8 (§9.0, REQ-IDX-001) |
| **`contextra-wire`** | Ring 0 | Kern `[K]` | `stable` | FlatBuffers Wire-Protokolle (Unsafe Island; §12.0) |

---

<a id="4-unsafe-inseln--locking-disziplin"></a>
## 4. Unsafe-Inseln (§6.7) & Locking-Disziplin (INV-LOCK-ORDER-1, §5.0, §27.6)

### 4.1 Unsafe-Inseln (§6.7, §0.4, §5.0)

Standardmäßig gilt in allen Nicht-Insel-Crates `#![forbid(unsafe_code)]`. Es gibt genau **drei zulässige Unsafe-Inseln** im Produktionscode, die per `#![allow(unsafe_code)]` ausgenommen sind (§6.7):

1. **`contextra-sys`**: Abstraktionen für Low-Level-OS-Operationen (`mmap`, `mlock`, Win32-ACLs).
2. **`contextra-simd`**: SIMD-Vektordistanzberechnungskerne (AVX2, AVX-512, NEON) mit Laufzeit-Dispatch.
3. **`contextra-wire`**: FlatBuffers-generierter Code für Zero-Copy-Transfers.

Jeder `unsafe`-Block in diesen Inseln muss zwingend mit einem `// SAFETY:`-Kommentar begründet werden (§6.7).

### 4.2 Sperrenhierarchie & Lock-Ordnung (INV-LOCK-ORDER-1, REQ-MVCC-004)

Zur Vermeidung von Deadlocks gilt im gesamten System eine strikte Sperrenhierarchie:

```
storage.wal (RwLock Read-Guard) → pending_commit_queue (Mutex) → index_finalizer (Lock)
```

Sperren müssen ausnahmslos in dieser Reihenfolge erworben werden. Kein `.await`-Aufruf darf gehalten werden, während ein synchroner Mutex/RwLock-Guard existiert.

---

<a id="5-referenzen"></a>
## 5. Referenzen & Weiterführende Dokumente

* **Normative Spezifikation 1.2**: [`docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md`](docs/spec/CONTEXTRA_SPEZIFIKATION_v15.md)
* **Agenten-Betriebsanleitung**: `AGENTS.md`
* **Sicherheitsrichtlinie**: `SECURITY.md`
