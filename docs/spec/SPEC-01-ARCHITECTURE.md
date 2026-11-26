# SPEC-01 — Architektur und Designprinzipien

**Version 2.1 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Storage](SPEC-03-STORAGE-ENGINE.md) · [Versionierung](SPEC-04-VERSIONING.md)

> Alle Performance-Zahlen sind **Designziele (Hypothesen)**, keine Messwerte. Validierung durch Prototypen und Benchmarks auf Referenzhardware (siehe [SPEC-09](SPEC-09-OPERATIONS.md), Abschnitt 3). Schlüsselwörter **MUSS**, **SOLL**, **KANN** folgen RFC 2119.

---

## 1. Produktvision

Contextra ist ein **LLM-natives, versioniertes Storage-System und hybrider RAG-Speicherkern** in Rust. Es macht die Besonderheiten von LLM- und Agenten-Workloads — Token-Budgets, hybride Suche über denselben Snapshot, spekulative Verzweigungen, Mehrfachauflösungen (Zoom-Leiter), Provenienz, Trust/Taint und unzuverlässige Clients — zu **Primitiven der Speicherschicht**, statt sie als zerbrechliche Anwendungsschicht über generischen Datenbanken neu zu bauen.

Unterhalb der Versionierungsschicht ist Contextra ein **Git-artiger content-addressed Objektspeicher mit spaltenorientierten Hot-Head/Cold-Tail-Extents und gelerntem physischen Layout**. Oberhalb ist es eine **kontextbewusste Wissensdatenbank** mit deterministischem Truth-Layer, reproduzierbaren Index-Segmenten (mit Watermarks) und budgetoptimierender Context Engine. Der Übergangspunkt ist der **Commit** — ein Hash über das deterministische **`RootSet` (Truth Layer)**, begleitet von einem separaten **`IndexManifest`** für abgeleitete Suchstrukturen.

### 1.1 Kernaussage in einem Satz

> Contextra ist eine eingebettete, versionierte Datenbank und hybride Context Engine in Rust, die Token-Budgets, Zoom-Stufen, spekulative Agent-Branches mit Merge-Gates, bitemporale Fakten und Trust-/Provenienz-Propagierung als Speicherprimitiven bereitstellt.

### 1.2 Deployment-Modell

| Aspekt | Entscheidung |
|---|---|
| **Primärmodus** | Eingebettete Library (`contextra`), vergleichbar mit SQLite/RocksDB |
| **Server-Modus** | Dünner MCP-/gRPC-/Arrow-Flight-/HTTP-Daemon (`contextra-server` + `contextra-mcp`) für Multi-Prozess- und Netzwerkzugriff |
| **Instanz-Modell** | Ein Repository/Store = eine Writer-Instanz (`flock`/`LockFileEx`), Single-Process. Mehrprozesszugriff ausschließlich über den Server |
| **Tiering** | RAM (Buffer Pool + MemDelta) ↔ NVMe (Packs/Segmente) ↔ Objektspeicher (S3-kompatibel, ab Phase 4) |
| **Priorität** | Embedded-first. Server ab Phase 2 als dünner Wrapper |
| **Plattformen** | Linux x86-64/aarch64 (primär; `pread`/`pwrite` als portabler Baseline-Standard, `io_uring` + `O_DIRECT` optional hinter `IoBackend`-Trait), macOS/Windows (`DurableFs` + `pread`/`pwrite` Threadpool). **Kein `mmap` im Produktionspfad.** |

### 1.3 Ressourcenprofile

Profile sind **reine Parametersätze** (Speicherbudgets, Segment-Schwellen). Sie schalten keine Kernfunktionen ab.

| Profil | Zielhardware | Zielbestand [Designziel] | Primäre Vektorstrategie |
|---|---|---|---|
| `lite` | Laptop/Edge: 4 Kerne, ≤ 8 GiB RAM | bis 2·10⁶ Cells | Flat + HNSW im RAM (SQ8) |
| `standard` | Server: 16 Kerne, 64 GiB RAM, NVMe | bis 5·10⁷ Cells | HNSW (SQ8) + Vamana-Segmente |
| `scale` | Server: 32+ Kerne, 256 GiB RAM, NVMe-RAID | bis 10⁸ Cells, 10⁹ Kanten | Vamana/DiskANN mit PQ/RaBitQ im RAM, SSD-Graph |

### 1.4 Nicht-Ziele (finale Liste)

| Nicht-Ziel | Begründung |
|---|---|
| **Verteilter Multi-Writer-Konsens in v1** | Single-Node-Kern; asynchrone Objekt-/Log-Replikation ab Phase 4 (Raft frühestens v2) |
| **Eigene vollständige SQL-Engine** | Typisierte Rust-API, JSON-Plan, MCP und optional SQLite Virtual Tables (`contextra-sqlite`) |
| **Git-Wire-Protokoll-Kompatibilität** | Export/Import möglich, kein Protokollziel |
| **Einzeldatei-Format** | Verzeichnislayout nötig für crash-sicheres WAL, immutable Packs, Manifest und Sidecars |
| **Trigger / Stored Procedures in v1** | Ereignisgesteuerte Logik läuft über Commit-Subscriptions und die Work Queue |
| **DSGVO-Löschkette & kryptographische Löschnachweise** | **Vollständig ausgeschlossen:** Kein Crypto-Shredding pro Subjekt, keine Löschbeweise, keine `DeletionProof`-/`AuditChain`-/`AVV`-Generatoren. Löschung erfolgt über versionierte Tombstones (`retire`), konfigurierbare Retention und Best-Effort-`purge` |
| **Eigene LLM-/Embedding-Inferenz im Speicherkern** | Inferenz erfolgt über Port-Traits (`Embedder`, `Reranker`, `Llm`, `Judge`) und externe Worker; Adapter liegen in separaten Satelliten-Crates |
| **Forschungs-Heuristiken im Kern** | Bandit-Routing, Thompson-Sampling, PID-/Lyapunov-Regler, Hyperkanten-Kaskaden, 6 parallele PPR-Varianten entfallen zugunsten je eines validierten Standardalgorithmus |

---

## 2. Die zehn Kernideen

| # | Idee | Beschreibung |
|---|------|-------------|
| 1 | **Cell (entkoppelt in Content + Versioned Meta) statt Key-Value** | Jede Wissenseinheit trennt den unveränderlichen `CellContent` (`content_hash`, woran alle teuren Derivate hängen) von leichtgewichtigen versionierten `CellMeta` (Attribute, Trust, Taint, Belief, Gültigkeitszeit) und `Edge`-Bäumen. Tagging oder Fakt-Supersession schreibt nie den Rohinhalt neu. |
| 2 | **Truth ≠ Index (Deterministischer Zustand + Index-Manifest)** | Der Prolly-Tree-Commit-Hash (`RootSet`) deckt ausschließlich die deterministische, history-unabhängige Wahrheitsschicht ab. Physische Index-Segmente (Vamana, FTS, CSR) hängen an einem separaten `IndexManifest`-Zeiger mit Watermarks und exaktem Flat-Fallback. |
| 3 | **Token-native Speicherung & Zoom-Knapsack** | Token-Zählungen pro Tokenizer-Familie und Zoom-Stufen (L0…Ln) sind persistiert und in Sequenzbäumen als Summen eingebettet. Der Context Packer maximiert Nutzen unter hartem Token-Budget mit Submodularitäts-/MMR-Redundanzabzug und Small-to-Big-Expansion. |
| 4 | **Dichter Adressraum (`Gid`) + MAC-geschützte Handles** | Jede Cell erhält eine dichte, monotone `Gid(u64)` für gemeinsame Roaring64-Filter über Vektor, BM25 und Graph. LLM-Handles tragen ein 2-Zeichen-BLAKE3-MAC-Tag (`c:k7qd-3f`), um halluzinierte Zufallstreffer auszuschließen. |
| 5 | **Hot-Head / Cold-Tail PAX-Extents + Locator** | Ein Extent packt bis zu *N* Cells spaltenorientiert (vorne kompakte Ranking-Lanes, hinten schwere Rohinhalte). Der `CellLocator` adressiert per `(PackId, ExtentNo, RowIndex)` — Re-Layout verschiebt Daten ohne Änderung von `ChunkId` oder `CommitId`. |
| 6 | **Agentensichere Versionierung & Merge-Gates** | Agenten schreiben **nie direkt nach `main`**, sondern auf O(1)-Session-Branches mit Savepoints je Tool-Call. Promotion nach `main` passiert deterministische **Merge-Gates (`G1`–`G7`)**. Unaufgelöste Konflikte leben als `ConflictCell`s im Merge-Request, nie auf `main`. |
| 7 | **Derivation Cache + Lineage-DAG** | Embeddings, Token-Zählungen und Summaries sind per `(content_hash, producer_id)` über Branches hinweg memoisiert und bei Quelländerung per Konstruktion als `stale` erkennbar. |
| 8 | **Provenienz, Konfidenz (Beta), Trust & Taint** | Jede Aussage trägt Pflichtprovenienz, `source_root`-Unabhängigkeit gegen Echokammern, `TrustTier` (Minimum-Vererbung) sowie orthogonale `TaintFlags` (Union-Vererbung). Nicht vertrauenswürdige Daten werden im Prompt gekapselt. |
| 9 | **Verlässliche Runtime-Grenzen & LLM-Fehler** | Idempotenz-Journal und Work-Queue-Leasing liegen in **nicht-versionierten Kernel-Sidecars** (kein Commit-Spam durch Worker-Heartbeats). Alle Fehler sind strukturiert, neutral bei fehlender Berechtigung und LLM-lesbar mit Remediation. |
| 10 | **Die Engine ruft kein LLM (Deterministische Ports)** | Uhr, Zufall, Dateisystem und Inferenz sind injizierbare Traits. Der Speicherkern ist vollständig deterministisch simulierbar. |

---

## 3. Designprinzipien und Systeminvarianten

### 3.1 Zehn Leitprinzipien

1. **Truth ≠ Index.** Das `RootSet` bildet den alleinigen, deterministischen Wahrheitsraum. Index-Segmente sind abgeleitete, verwerfbare und rekonstruierbare Projektionen im separaten `IndexManifest`.
2. **Immutable first.** Truth-Chunks, Packs, Prolly-Knoten, Index-Segmente und Commits werden nie in-place überschrieben. Mutabilität existiert nur als atomar publizierte Pointer (`ArcSwap`), WAL-Tail, MemDelta/Memtable und nicht-versionierte Kernel-Sidecars.
3. **Ein Commit-Punkt, kein Mehr-Store-2PC.** Ein Commit ist dauerhaft, sobald sein WAL-Frame die geforderte Durability-Stufe erfüllt. Index-Builds laufen entkoppelt mit Watermarks — kein kompensierendes 2PC.
4. **Identität ≠ Ort.** Content-Addressing (BLAKE3) bestimmt Identität; physische Platzierung `(PackId, ExtentNo, RowIndex)` liegt im austauschbaren Locator.
5. **Agenten schreiben nie nach `main`.** Schreibpfad ist `Session-Branch → Merge-Gates → Atomarer Merge`. Direktes Schreiben auf geschützte Branches erfordert explizite Admin-Privilege.
6. **Budget & Belegbarkeit statt Top-k.** Leseoperationen optimieren Informationsabdeckung und Nutzen pro Token unter harter Budgetgarantie, liefern `EvidenceSufficiency` (inkl. `None` / `NO_EVIDENCE_FOUND` statt Halluzinationszwang) und verifizierbare Zitate.
7. **Abgeleitet ist ersetzbar, Herkunft ist Pflicht.** Nur Rohdaten + Provenienz sind Primärwahrheit; Derivate sind memoisiert und regenerierbar.
8. **Fail-closed in Sicherheit, Fail-open in Index-Verfügbarkeit.** Fehlende ACL/Capabilities blenden Treffer ohne Fehler-Orakel aus (ACL-Bitmaps wirken *vor/im* Ranking); fehlende Indizes fallen transparent auf exakten Scan zurück (`lane_coverage`).
9. **Kein stilles Verwerfen.** Weder Decay noch Merge noch Compaction löschen lebendige Daten stillschweigend: Decay steuert nur Ranking und `ARCHIVED`-Empfehlungen; Konflikte erzeugen sichtbare `ConflictCell`s; Löschungen sind explizite Tombstone-Commits (`retire`).
10. **Schichten zeigen nur nach unten.** Kein Crate hängt von einer höheren Schicht oder externen Inferenz-Implementierungen ab; Durchsetzung in CI.

### 3.2 Systeminvarianten (normativ, in Simulation geprüft)

| ID | Invariante | Durchsetzung |
|---|---|---|
| **INV-1** | **Deterministischer Truth-Hash:** Gleicher logischer Zustand (`RootSet`) ⇒ identischer `truth_hash`, unabhängig von Commit-Historie, Re-Layout oder Index-Compaction. | Property-Tests, SPEC-04 |
| **INV-2** | **Snapshot-Einheit:** Alle Modalitäten (Vektor, BM25F, Graph, Filter, Fusion, Packer) einer Query lesen exakt einen konsistenten Branch-Snapshot `(head, mem_delta_seq, now)`. | Typestate `Snapshot<'db>`, SPEC-06 |
| **INV-3** | **Prefix-Konsistenz nach Crash:** Nach Recovery entspricht der sichtbare Zustand exakt einem Präfix der gemäß Durability-Klasse bestätigten WAL-Commits. | `FaultVfs` Crash-Matrix, SPEC-07 |
| **INV-4** | **Index ⊆ Truth & Watermark-Deckung:** Jeder Index-Treffer wird gegen den Snapshot (`¬dead ∧ ¬Shadow(B) ∧ ACL ∧ valid_at`) validiert; Zeilen nach `watermark_seq` werden exakt aus MemDelta/Delta-Scans ergänzt. | `ctx verify`, SPEC-05 |
| **INV-5** | **ACL vor Ranking:** Zugriffs-Bitmaps schränken ANN-/BM25-/Graph-Operatoren direkt ein; Scores, Zähler und Fehlermeldungen leaken nie unberechtigte Existenz. | ACL-Leakage-Suite, SPEC-07 |
| **INV-6** | **Idempotenz:** Gleicher `IdempotencyKey` + gleicher Payload-Hash ⇒ identisches Ergebnis ohne zweiten Commit; abweichender Payload ⇒ `idempotency_mismatch`. | Kernel-Idempotency-Table, SPEC-04 |
| **INV-7** | **`main`-Integrität:** Geschützte Branches (`main`) enthalten nie unaufgelöste Merge-Konflikte; Konflikte leben auf `mr/*` oder Feature-Branches. | Merge-Gates `G1`–`G7`, SPEC-04 |
| **INV-8** | **Budget-Garantie:** `pack_result.budget_used ≤ request.budget.max_tokens` gilt in 100 % der Fälle (inkl. Rendering-Overhead). | Property-Tests, SPEC-06 |
| **INV-9** | **GC-Sicherheit:** Kein von einem Root (Branch-Head, Tag, Reflog-Fenster, aktiver Snapshot-Pin, Savepoint) erreichbares Objekt wird physisch entfernt. | SATB+Remark-Simulation, SPEC-07 |

---

## 4. Schichtenarchitektur

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  L6  API-Oberflächen & Server                                                │
│      Rust-Fassade (contextra) · MCP-Server · gRPC/Arrow Flight/HTTP · CLI    │
│      Optional: SQLite Virtual Tables (contextra-sqlite) · Python (PyO3)      │
├──────────────────────────────────────────────────────────────────────────────┤
│  L5  Context Engine, Ingest & Agent-Dienste                                  │
│      Recall-Pipeline · Zoom-Knapsack/MMR · Citation-Verifier · EvidencePack  │
│      Agent-Sessions & Merge-Gates (G1–G7) · Egress-Guard · Ingest-Chunker    │
│      Cognition-Jobs (via Ports) · Retention/Zoom-Down/Purge                  │
├──────────────────────────────────────────────────────────────────────────────┤
│  L4  Abgeleitete Indizes (IndexManifest + Watermarks, runtime-frei)          │
│      Vektor (Flat / RAM-HNSW / SSD-Vamana) · FTS/BM25F (FST + BlockMaxWAND)  │
│      Graph (CSR + Forward-Push-PPR + Leiden) · Roaring-Filter-Bitmaps        │
├──────────────────────────────────────────────────────────────────────────────┤
│  L3  Versionierte Wahrheit & MVCC (RootSet, runtime-frei)                    │
│      Commit-DAG · Prolly-Trees (primary, seq, graph, aliases, policies,      │
│      conflicts, provenance) · MemDelta · OCC/SSI Typestate · 3-Wege-Merge    │
│      SATB+Remark GC · Nicht-versionierte Kernel-Sidecars (Idem, WorkQ, Stats)│
├──────────────────────────────────────────────────────────────────────────────┤
│  L2  Content-addressed Object Store & PAX-Layout (runtime-frei)              │
│      BLAKE3-Chunks · Packs (.cxp) · Hot-Head/Cold-Tail Extents               │
│      ObjectLocator + CellLocator (PackId, ExtentNo, RowIndex) · Re-Layout    │
│      Partitionierter Buffer Pool (W-TinyLFU) · S3-Tier-Port                  │
├──────────────────────────────────────────────────────────────────────────────┤
│  L1  Durability & System-I/O (runtime-frei)                                  │
│      WAL (.cxw, Group Commit) · Manifest-Log (.mlog + CURRENT Superblock)    │
│      Crash-Recovery · DurableFs · io_uring/pread-Backends (kein mmap!)       │
├──────────────────────────────────────────────────────────────────────────────┤
│  L0  Fundament & Querschnitt                                                 │
│      Typen/IDs/HLC · Port-Traits · Wire/rkyv-Codecs · SIMD · Crypto/AEAD/MAC │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 4.1 Schichtverantwortungen & Runtime-Grenze

| Schicht | Verantwortung | Verbot / Grenze |
|---|---|---|
| **L0** | Newtypes, `Gid`/`CellId`/`ChunkId`/`Handle`, Port-Traits, Codecs, SIMD, AEAD, Keyed-BLAKE3, Capabilities | Keine Geschäftslogik, keine I/O-Loops |
| **L1** | WAL, Group Commit, Manifest, Superblock, `DurableFs`, Crash-Recovery, OS-I/O | Kennt kein Datenmodell; **einziger Ort für `fsync`/`fdatasync`** |
| **L2** | Packs, PAX-Extents, Kompression, Locator (`Object` + `Cell`), Buffer Pool, Re-Layout | Kennt keine Commits/Branches/Merge-Semantik |
| **L3** | Prolly-Trees, `RootSet`, `MemDelta`, Fold, Branches, OCC/SSI, Merge, GC, Kernel-Sidecars | Kennt keine ANN-/BM25-Algorithmen; **L0–L4 sind `tokio`-frei** (eigene I/O- + Rayon-Pools) |
| **L4** | Vektor-, FTS-, Graph-CSR-, Bitmap-Segmente, Watermark-Tracking, Delta-Overlays | Kennt keine Score-Fusion oder LLM-Prompts |
| **L5** | Hybrid-Planer, RRF, Packer, Verifier, Ingestion, Session-Lifecycle, Gates, Egress-Guard | Keine direkten OS-Dateizugriffe (`Vfs`-Port); `tokio` ab L5 |
| **L6** | Protokolle (MCP, gRPC/Flight, HTTP, SQLite-vtab, CLI), FFI | Keine Storage-/Index-Interna |

### 4.2 Port-Schnittstellen (`contextra-ports` & Kern-Traits)

```rust
// ── L0: Injizierbare Umgebungs-Ports (für deterministische Simulation) ──
pub trait Clock: Send + Sync { fn now_hlc(&self) -> Hlc; fn now_ms(&self) -> i64; }
pub trait RngSource: Send + Sync { fn fill_bytes(&self, dest: &mut [u8]); }
pub trait Vfs: Send + Sync { /* open, rename, remove, list_dir, sync_dir */ }

// ── L1/L2: Objektspeicher & Layout-Lokator ──
pub trait ObjectStore: Send + Sync {
    fn put_chunk(&self, kind: ChunkKind, bytes: &[u8]) -> Result<ChunkId>;
    fn get_chunk(&self, id: &ChunkId) -> Result<Bytes>;
    fn get_extent_row(&self, addr: ExtentRowAddr, lanes: LaneMask) -> Result<ExtentRowSlice>;
    fn flush_durable(&self) -> Result<DurableMark>;
}

// ── L3: Prolly-Tree-Engine (deterministischer Truth-Layer) ──
pub trait TreeEngine {
    fn get(&self, root: &ChunkId, k: &[u8]) -> Result<Option<Bytes>>;
    fn range(&self, root: &ChunkId, r: KeyRange, dir: Dir)
        -> Box<dyn Iterator<Item = Result<(Bytes, Bytes)>>>;
    fn apply(&self, root: &ChunkId, muts: &mut dyn Iterator<Item = Mutation>)
        -> Result<ChunkId>;
    fn diff(&self, a: &ChunkId, b: &ChunkId, r: KeyRange)
        -> Box<dyn Iterator<Item = Result<DiffEntry>>>;
}

// ── L4: Abgeleitete Indizes mit Watermark-Vertrag ──
pub trait DerivedIndex: Send + Sync {
    fn kind(&self) -> SegmentKind;
    fn watermark(&self) -> WatermarkSeq;
    fn build_from_truth(&self, snap: &TruthSnapshot, delta: &DiffStream) -> Result<IndexManifestDelta>;
}

// ── L5: Erweiterungs-Ports (ohne Kern-Abhängigkeit) ──
pub trait MergeGate: Send + Sync { fn check(&self, ctx: &MergeGateContext) -> GateOutcome; }
pub trait Embedder: Send + Sync { fn embed(&self, space: &SpaceId, inputs: &[&str]) -> Result<Vec<Vec<f32>>>; }
pub trait Tokenizer: Send + Sync { fn id(&self) -> TokenizerId; fn count(&self, text: &str) -> u32; }
```

---

## 5. Crate-Layout (Soll-Workspace)

Bereinigtes Target-Layout ohne zirkuläre oder historische Altlasten (`contextra-core`, `contextra-db`, `contextra-checkpoint`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-license`, `contextra-adapt`, `contextra-router` werden entfernt bzw. aus dem Kern ausgelagert).

```
contextra/
├─ L0: Fundament & Querschnitt
│  ├─ contextra-types        Newtypes (CellId, Gid, ChunkId, Handle, Hlc), TrustTier, Taint, Fehlerkatalog [safe]
│  ├─ contextra-ports        Port-Traits (Vfs, Clock, Rng, Embedder, Reranker, Llm, Judge, Tokenizer)      [safe]
│  ├─ contextra-wire         Kanonische rkyv-/bytemuck-Codecs, Format-Header (Magic/Version/Flags)        [safe + checked]
│  ├─ contextra-simd         SIMD-Kernel (AVX2/AVX-512/NEON/Scalar: Distanzen, Bitpacking)                [unsafe isoliert]
│  ├─ contextra-sys          OS-Zugriff (io_uring optional, pread/pwrite, flock, O_DIRECT-Buffer)         [unsafe isoliert]
│  └─ contextra-crypto       Keyed BLAKE3, HKDF, AEAD (DEK/KEK), Ed25519, MAC-Handles, Capabilities, Egress [safe]
├─ L1: Durability
│  ├─ contextra-durable-fs   Atomares Write-Sync-Rename-DirSync, FaultVfs-Referenz                       [safe]
│  └─ contextra-wal          WAL (.cxw), Group Commit, Torn-Write-Erkennung, Recovery-Replay             [safe]
├─ L2: Object Store
│  └─ contextra-store        Packs (.cxp), PAX-Extents (Hot-Head/Cold-Tail), Object/Cell-Locator,        [safe + O_DIRECT]
│                            Partitionierter Buffer Pool (W-TinyLFU), Re-Layout, S3-Tier-Port
├─ L3: Versionierung & MVCC
│  └─ contextra-mvcc         Prolly-Trees, RootSet, IndexManifestRef, MemDelta, Fold, BranchSequencer,   [safe]
│                            Typestate Txn<ReadOnly|ReadWrite>, OCC/SSI, 3-Wege-Merge, SATB-GC, Sidecars
├─ L4: Indizes (runtime-frei)
│  ├─ contextra-vector       Flat-SIMD, RAM-HNSW-Delta, Vamana-Segmente, SQ8/PQ/RaBitQ, Filter A/B/C    [safe via simd]
│  ├─ contextra-text         Unicode/Snowball/DE-Decompounding, FST, Block-Max-WAND, BM25F global stats [safe]
│  └─ contextra-graph        CSR-Segmente (fwd/rev), Hub-Skip-Index, k-Hop (mit Limits), Forward-Push-PPR[safe]
├─ L5: Context Engine, Ingest & Agenten
│  ├─ contextra-rank         RRF, kalibrierte Norm-Fusion, Isotonische Kalibration, MMR                  [safe]
│  ├─ contextra-engine       Katalog, Query-Planer/-Executor, Zoom-Knapsack/Packer, Sufficiency,         [safe]
│  │                         Citation-Verifier, Derivation Cache, WorkQueue-Dispatcher, Retention/Purge
│  ├─ contextra-ingest       Parser (PDF/Office/HTML/Code/JSON), Chunker, Revisions-Diff, Injection-Scan [safe]
│  ├─ contextra-agent        Session-Branches, Merge-Gates (G1–G7), Action-Log, Quoten, Conflict-Resolve  [safe]
│  └─ contextra-cognition    Konsolidierung, L1-Summary, Entity-Resolution, Contradiction-Jobs (Ports)   [safe]
├─ L6: APIs & Integration
│  ├─ contextra              Einzige öffentliche Rust-Fassade (Feature-Flags für optionale Crates)       [safe]
│  ├─ contextra-mcp          MCP-Server (stdio / streamable-HTTP, deklarative Tool-Klassifikation)       [safe]
│  ├─ contextra-server       gRPC + Arrow Flight + HTTP/JSON-Daemon                                      [safe]
│  ├─ contextra-cli          Admin-CLI (`ctx`)                                                           [safe]
│  └─ contextra-sqlite       Optionales SQLite-Virtual-Table-Frontend (Feature `sqlite`)                 [unsafe isoliert]
├─ Optional / Satelliten (Feature-gated oder eigenes Packaging)
│  ├─ contextra-infer-onnx / -candle / -ollama   Inferenz-Adapter für Ports                              [isolierte FFI/safe]
│  ├─ contextra-sandbox      WASM-Plugins für benutzerdefinierte MergeGates/Resolver (wasmtime)          [safe]
│  ├─ contextra-kvcache      Optionaler Radix-Prefix-/KV-Cache (Phase 5, ohne Shredding)                 [safe]
│  └─ contextra-py           Python-Bindings (PyO3, außerhalb default-members)                           [unsafe isoliert]
└─ Dev / Test
   ├─ contextra-testkit      Referenzmodell, Property-/Golden-/ACL-Tests                                 [test]
   ├─ contextra-sim          Deterministischer Simulator (`FaultVfs` + `ManualClock` + Crash-Matrix)     [test]
   └─ contextra-bench        Mikro- & Makro-Benchmarks (10⁶–10⁸ Skalierung)                              [bench]
```

### 5.1 Abhängigkeits- und Sicherheitsregeln

1. **Strikte Gerichtetheit:** Nur nach unten; CI prüft per `cargo metadata` auf Zyklenfreiheit und Schichtgrenzen.
2. **Runtime-Freiheit L0–L4:** `tokio` ist in L0–L4 verboten. Der Storage-Kernel arbeitet mit synchronen APIs + dedizierten I/O- und Compute-(`rayon`)-Pools.
3. **`#![forbid(unsafe_code)]`:** Workspace-Standard (`#![deny(unsafe_code)]` in `Cargo.toml`, `#![forbid(unsafe_code)]` in allen Kern-Crates außer `contextra-simd`, `contextra-sys`, `contextra-sqlite`, `contextra-infer-*`, `contextra-py` sowie `O_DIRECT`/Archive-Helfern mit dokumentierten `// SAFETY:`-Invarianten).
4. **Kein `unwrap()`/`expect()`/`panic!` in Produktionspfaden** (Workspace-Clippy-Lint `deny`).
5. **Rust Edition 2021, stable Toolchain (≥ 1.89)**, `--locked` Builds, `cargo-deny` + `cargo-vet` Pflicht in CI.

---

## 6. Typische Zugriffsmuster

| # | Muster | Beschreibung | Anteil |
|---|---|---|---|
| 1 | **Recall unter Budget** | „Gib mir die besten ≤ 6 000 Token belegten Kontext mit Abstention-Check." | ~65 % |
| 2 | **Fenster-Lesen (`window`)** | „Gib mir die Turns 40–60 dieser Episode, ≤ 3 000 Token" in O(log n) ohne Volltext-Scan. | ~10 % |
| 3 | **Progressive Disclosure (`expand`)** | Agent prüft `c:k7qd-3f` (L1/L2) und lädt selektiv L0 nach. | ~15 % |
| 4 | **Append- und Revision-Ingest** | Episoden, Tool-Ergebnisse, Dokument-Revisionen (per ContentHash-Diff inkrementell). | Hauptlast Schreiben |
| 5 | **Session-Fork → Gates → Merge** | Agenten arbeiten isoliert auf Branches; Promotion nur nach bestandenen Gates `G1`–`G7`. | ~5 % Transaktionen |
| 6 | **Hintergrundarbeit (Sidecar-Queue)** | Asynchrone Embeddings, L1-Summaries, Compaction, Vamana-Builds, Affinity Re-Layout. | kontinuierlich |

---

## 7. Entscheidungsregister (konsolidiert)

| ID | Entscheidung | Begründung / Aufgelöster Widerspruch |
|---|---|---|
| **C-01** | **Prolly-Trees + MemDelta** für deterministischen Truth-Layer | History-Unabhängigkeit, O(Δ·log n) Diff/Merge, strukturelles Sharing; MemDelta + Batch-Fold begrenzt Schreibamplifikation |
| **C-02** | **Trennung `RootSet` vs. `IndexManifest`** *(löst Reibungspunkt 1)* | `RootSet` hasht nur logische Truth-Bäume; Vamana-/FTS-/CSR-Segmente hängen am separaten `IndexManifest` mit Watermarks & Flat-Fallback |
| **C-03** | **Entkoppelte Cell (`CellContent` vs. `CellMeta`) & PAX-`CellLocator`** *(löst Reibungspunkt 3)* | Tags/`valid_to`/Trust ändern nicht `content_hash` (kein Re-Embedding); `CellLocator` nutzt `(PackId, ExtentNo, RowIndex)` statt Byte-Offsets je Lane |
| **C-04** | **Ephemerer Laufzeitzustand in Kernel-Sidecars** *(löst Reibungspunkt 2)* | Work-Queue-Leases und Idempotenz-Journal außerhalb `RootSet`; kein Commit-Spam und kein Merge-Leakage operativer Leases |
| **C-05** | **MAC-geschützte Handles `c:<base32>-<mac2>`** *(löst Reibungspunkt 4)* | 10-Bit BLAKE3-Tag schließt zufällige Treffer bei LLM-Halluzinationen aus und triggert zuverlässig Fuzzy-Suggestions |
| **C-06** | **Dichtes `Gid(u64)` + segmentlokales `DocId(u32)`** | Gemeinsamer Roaring64/Roaring32-Filterraum für ACL, Zeit, Trust, Graph, Vektor und BM25F |
| **C-07** | **Keyed CAS-Hashing über Klartext (`BLAKE3(repo_hash_key, plaintext)`)** | Verhindert Cross-Repo Existenz-Orakel **ohne** Content-Addressing und Deduplizierung durch Nonce-Ciphertext zu brechen |
| **C-08** | **AEAD at Rest mit KEK/DEK-Hierarchie** | O(1) Master-Key-Rotation via DEK-Rewrap; lazy Pack-Rotation bei Compaction |
| **C-09** | **Agenten schreiben nie nach `main`; Merge-Gates `G1`–`G7`** | `main` bleibt frei von unaufgelösten Konflikten und ungeprüften Schreibzugriffen |
| **C-10** | **Obligatorische Provenienz, `BeliefBeta(α,β)` & `source_root`-Unabhängigkeit** | Verhindert LLM-Echokammern; Konfidenz nutzt 10-%-LCB und noisy-AND bei Ableitungen |
| **C-11** | **TrustTier (min) + orthogonale TaintFlags (union) + Datenrahmen** | Wirksame Abwehr gegen Prompt-Injection-Privilegieneskalation im Retrieval |
| **C-12** | **Ein Commit-Punkt (WAL Group Commit); kein 4-Store-2PC** | Beseitigt Split-Brain-Risiken; Packs sind Wahrheitsquelle, Indizes rekonstruierbar |
| **C-13** | **Zoom-Knapsack + Submodularitäts-Abzug + Small-to-Big + Abstention** | Liefert Überblick & Tiefe im Budget sowie explizites `NO_EVIDENCE_FOUND` bei Evidenzmangel |
| **C-14** | **Ein validierter Standardalgorithmus je Modalität** | HNSW/Vamana, BM25F+BMW (globale Stats), CSR+Forward-Push-PPR+Leiden, RRF(k=60) |
| **C-15** | **Keine Löschkette, keine Löschnachweise; `retire` + Retention + Best-Effort-`purge`** | Explizite Produktentscheidung; Decay beeinflusst nur Ranking und Archivierung, löscht nie still |
| **C-16** | **Portables I/O (`pread`/`pwrite`) + opt. `io_uring`; kein `mmap` im Produktpfad** | Vermeidet unkontrollierte Eviction, TLB-Shootdowns und SIGBUS |
| **C-17** | **Eigener FTS statt Tantivy** | Block-Max-WAND auf segmentlokalen `DocId`s + Content-Addressing + gemeinsame ACL-/Graph-Bitmaps |
| **C-18** | **WASM-Merge/Gates mit Fuel-Limits (`wasmtime`)** | Deterministische Sandbox ohne WASI/Float; Fallback auf `ConflictCell` |
| **C-19** | **Eventual Consistency für `AccessStats`; Re-Layout opt-in nach Benchmark** | Hot-Head/Cold-Tail-Extents sind Basis; Affinity Re-Layout optimiert Cold-Tail-I/O ohne Semantikänderung |
| **C-20** | **Typestate-Transaktionen (`Txn<ReadOnly|ReadWrite>`) & `Snapshot<'db>`** | Schließt unregistrierte SSI-Reads konstruktiv aus |
| **C-21** | **Embedded-first, Single-Process (`flock`), Server als dünne Hülle** | Minimale Latenz im Standardfall; Daemon für Multi-Client-Betrieb |

---

## 8. Abgrenzung zu Alternativen

| Aspekt | RocksDB / LSM | Dolt (Prolly) | LanceDB / Vektor-DB | **Contextra** |
|---|---|---|---|---|
| **Primitive** | Key → Bytes | SQL-Zeilen | Vektor + Payload | **Entkoppelte Cell (`Content` + `Meta`) + Zoom** |
| **Versionierung** | Snapshots | Branches/Merge | Dataset-Versionen | **O(1)-Session-Branches, Merge-Gates `G1`–`G7`, Zeitreise** |
| **Indizes & Konsistenz** | Keine Vektoren/Graph | B-Tree | Vektor/FTS ohne Graph | **`RootSet` (Truth) + `IndexManifest` (Watermark + Flat-Fallback)** |
| **Filter & Hybrid** | Scan | Join | Post-/Pre-Filter | **Gemeinsames `Gid`/`DocId` Bitmap für Vektor + BM25F + Graph** |
| **LLM-Integration** | Keine | Keine | Top-k Chunks | **Token-Summen, Zoom-Knapsack, MAC-Handles, Abstention, Taint** |
| **Layout** | Key-Order | Key-Order | Spalten | **Hot-Head/Cold-Tail PAX-Extents + Locator Re-Layout** |

---

*Ende SPEC-01. Weiter: [SPEC-02 Datenmodell](SPEC-02-DATA-MODEL.md)*
