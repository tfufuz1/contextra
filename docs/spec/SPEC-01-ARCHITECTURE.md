# SPEC-01 — Architektur und Designprinzipien

**Version 2.0 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Storage](SPEC-03-STORAGE-ENGINE.md)

> Alle Performance-Zahlen sind **Designziele**, keine Messwerte. Validierung durch Prototypen (siehe [SPEC-09](SPEC-09-OPERATIONS.md), Abschnitt 3).

---

## 1. Produktvision

Contextra ist ein **LLM-natives, versioniertes Storage-System** in Rust. Es macht die Besonderheiten von LLM- und Agenten-Workloads — Token-Budgets, unscharfe Suche, spekulative Verzweigungen, Mehrfachauflösungen, unzuverlässige Clients — zu **Primitiven der Speicherschicht**, statt sie in jeder Anwendung neu zu bauen.

Unterhalb der Versionierungsschicht ist Contextra ein **Git-artiger Objektspeicher mit gelerntem physischen Layout**. Oberhalb ist es eine **kontextbewusste Wissensdatenbank** mit hybrider Retrieval-Engine. Der Übergangspunkt ist der **Commit** — ein Hash über das gesamte Wurzelobjekt inklusive aller Indizes.

### 1.1 Kernaussage in einem Satz

> Contextra ist eine eingebettete, versionierte Datenbank und RAG-Engine, die Token-Budget, Zoom-Stufen, spekulative Branches und Trust-Propagierung als Speicherprimitiven bereitstellt.

### 1.2 Deployment-Modell

| Aspekt | Entscheidung |
|---|---|
| **Primärmodus** | Eingebettete Library (`libcontextra`), vergleichbar mit SQLite/RocksDB |
| **Server-Modus** | Dünner MCP-/gRPC-Server (< 2 000 LOC) für Multi-Prozess- und Netzwerkzugriff |
| **Instanz-Modell** | Ein Repository = eine Instanz, Single-Process. Mehrprozess ausschließlich über den Server |
| **Tiering** | RAM ↔ NVMe ↔ Objektspeicher (S3-kompatibel) |
| **Priorität** | Embedded-first. Server ab Phase 2 als Wrapper |
| **Plattformen** | Linux (primär, io_uring), macOS/Windows (Fallback pread/pwrite) |

### 1.3 Nicht-Ziele (finale Liste)

| Nicht-Ziel | Begründung |
|---|---|
| Verteilter Konsens in v1 | Einzelknoten; Replikation ab Phase 4 über Objekt-Austausch |
| Eigene SQL-Engine | SQLite-vtab als Frontend (der Dialekt, den LLMs kennen) |
| Git-Wire-Protokoll-Kompatibilität | Export/Import-Pfad möglich, kein Ziel |
| Einzeldatei-Format | Verzeichnis wegen Crash-Sicherheit (WAL + Packs + Manifest) |
| Trigger/Stored Procedures in v1 | Spätere Erweiterung über Work Queue |
| DSGVO-Löschkette als Architektur-Treiber | Standard-Löschmechanismen ausreichend |
| Eigene Embedding-/LLM-Inferenz im Kern | Work Queue + externe Worker; optionale ONNX/Candle-Backends als separate Crates |
| Streaming-/CDC-Protokoll in v1 | Commit-Subscription reicht; Kafka-artige Garantien nicht im Scope |

---

## 2. Die zehn Kernideen

| # | Idee | Beschreibung |
|---|------|-------------|
| 1 | **Cell statt Key-Value** | Die Primitive ist eine *Wissenseinheit* mit Facetten (Rohinhalt, Tokens, Embeddings, Zusammenfassungen, Kanten, Zeit, Vertrauen), nicht ein opaker Blob. |
| 2 | **Token-native Speicherung** | Token-Zählungen sind persistiert und in Indexstrukturen als Summen eingebettet. „Gib mir 3 000 Token rund um Cell X" kostet O(log n) ohne Inhalte zu lesen. |
| 3 | **Zoom-Leiter + Zoom-Knapsack** | Jede Cell hat Auflösungsstufen L0…Ln. Die Engine wählt pro Treffer die Stufe so, dass der Nutzen unter dem Token-Budget maximal wird (Multiple-Choice-Knapsack). |
| 4 | **Ortsunabhängige Identität, gelerntes Layout** | Inhalte sind content-addressed (BLAKE3). Wo sie physisch liegen, entscheidet ein Hintergrundprozess anhand echter Zugriffsspuren (Co-Retrieval-Clustering). |
| 5 | **Hot-Head/Cold-Tail-Extents** | Ein Extent enthält vorne alles zum Ranken, hinten die schweren Inhalte → ein I/O für Ranking, ein weiterer nur für Gewinner, beide am selben Ort. |
| 6 | **Versionierung *ist* der Transaktionsmechanismus** | O(1)-Forks pro Agentenversuch, Merge mit typisierten Strategien, Konflikte als Daten, bitemporale Zeitreise. |
| 7 | **Derivation Cache + Lineage** | Embeddings, Summaries, Token-Zählungen sind *abgeleitet*, per Inhalts-Hash memoisiert, über Branches geteilt und bei Quelländerung automatisch als veraltet markiert. |
| 8 | **Trust-/Taint-Labels im Storage** | Jede Cell trägt eine Vertrauensstufe; abgeleitete Daten erben das Minimum ihrer Quellen. Grundlage für Prompt-Injection-Abwehr auf Anwendungsebene. |
| 9 | **Agent-sichere Semantik** | Idempotenz-Schlüssel, LLM-lesbare Fehler mit Handlungsvorschlägen, kurze token-sparsame IDs mit „Meintest du …?", Progressive Disclosure per Handle/Expand. |
| 10 | **Die Engine ruft kein LLM** | Contextra stellt Arbeit (Work Queue) bereit, externe Worker erledigen sie. Die Engine bleibt deterministisch, testbar und modellneutral. |

---

## 3. Designprinzipien

1. **Immutable first.** Inhalte und Indexknoten sind unveränderlich und content-addressed. Mutabilität existiert nur als Zeiger (Refs) und als kleine Seitentabellen (z. B. Zugriffsstatistik).
2. **Identität ≠ Ort.** Logische Adresse (Hash) und physischer Ort sind entkoppelt, damit das Layout jederzeit neu optimiert werden kann.
3. **Ein Commit, ein Wahrheitsraum.** Primärdaten *und alle Indizes* sind Teil desselben versionierten Wurzelobjekts. Es gibt keinen „Index hinkt hinterher"-Zustand für Leser.
4. **Budget statt Limit.** Leseoperationen optimieren Nutzen pro Token, nicht „Top-k".
5. **Abgeleitet ist ersetzbar.** Nur Rohdaten sind Wahrheit; alles andere kann neu berechnet werden und trägt seine Herkunft.
6. **Der Client ist unzuverlässig.** Jede Schreiboperation ist idempotent, jede Antwort verständlich für ein Modell.
7. **Die Engine ruft kein LLM.** Contextra stellt Arbeit (Work Queue) bereit, externe Worker/Agenten erledigen sie. Das hält die Engine deterministisch, testbar und modellneutral.
8. **Korrektheit vor Geschwindigkeit, dann Geschwindigkeit.** Deterministische Simulationstests, Prüfsummen überall, `unsafe` nur in klar begrenzten Crates.
9. **Truth Layer / Index Layer.** Primärdaten (Cells im Prolly-Tree) bilden die Wahrheitsschicht. Indizes (Vektor, FTS, Graph) sind abgeleitete, reproduzierbare Projektionen.
10. **Schichten zeigen nur nach unten.** Kein Crate kennt ein höheres. Jede Schicht hängt an Traits der darunterliegenden.

---

## 4. Schichtenarchitektur

```
┌──────────────────────────────────────────────────────────────────────────┐
│  L6  API-Oberflächen                                                     │
│      Rust-Embedded · MCP-Server · gRPC/HTTP · Python/TS-Bindings · CLI   │
├──────────────────────────────────────────────────────────────────────────┤
│  L5  Context Engine                                                      │
│      recall · window · expand · explain · pack (Zoom-Knapsack)           │
│      consolidation work queue · retention/forgetting policies            │
├──────────────────────────────────────────────────────────────────────────┤
│  L4  Indizes (alle Teil des Commit-Wurzelobjekts)                        │
│      Vektor (segmentiert) · FTS/BM25 · Graph · Temporal · Attribute      │
│      Token-Summen (augmentierter Baum)                                   │
├──────────────────────────────────────────────────────────────────────────┤
│  L3  Versionierter Zustand                                               │
│      Commit-DAG · Refs (Branches/Tags) · Prolly-Tree · Delta-Log         │
│      Transaktionen · Merge · Bitemporalität · Idempotenz-Journal         │
├──────────────────────────────────────────────────────────────────────────┤
│  L2  Content-addressed Object Store                                      │
│      Chunks (BLAKE3) · Packs · Extents · Locator-Index · Layout-Optimizer│
├──────────────────────────────────────────────────────────────────────────┤
│  L1  I/O & Durability                                                    │
│      WAL (Group Commit) · Manifest-Log · io_uring/pread-Backends         │
│      Tiering: RAM ↔ NVMe ↔ Object Store (S3-kompatibel)                  │
├──────────────────────────────────────────────────────────────────────────┤
│  L0  Querschnitt                                                         │
│      Verschlüsselung (Envelope) · Capabilities · Metriken · Access-Trace │
└──────────────────────────────────────────────────────────────────────────┘
```

### 4.1 Schichtverantwortungen

| Schicht | Verantwortung | Darf nicht |
|---|---|---|
| L0 | Kryptographie, Autorisierung, Telemetrie | Geschäftslogik kennen |
| L1 | Physische I/O, Durability-Garantien, Tiering-Entscheidungen | Datenmodell kennen |
| L2 | Content-Addressing, Packing, Locator, physisches Layout | Versionierungs-Semantik kennen |
| L3 | Commits, Branches, Merge, Transaktionen, bitemporale Zeitreise | Index-Algorithmen kennen |
| L4 | Vektor-, FTS-, Graph-, Temporal-Indizes; Index-Lifecycle | Retrieval-Fusion kennen |
| L5 | Recall-Pipeline, Score-Fusion, Zoom-Knapsack, Rendering | Physisches Layout kennen |
| L6 | Protokoll-Adaptation (MCP, gRPC, SQL), Serialisierung | Storage-Interna kennen |

### 4.2 Kommunikation zwischen Schichten

Jede Schicht exponiert **Port-Traits** (`contextra-ports`). Die darüberliegende Schicht programmiert gegen diese Traits. Konkrete Implementierungen werden beim Start per Builder injiziert.

**Ausnahme:** L0 (Querschnitt) wird von allen Schichten verwendet, aber nie von einer höheren injiziert.

---

## 5. Crate-Layout (Cargo Workspace)

```
contextra/
├─ contextra-core        Typen, IDs, Fehler, Zeit (HLC), Konfiguration         [safe]
├─ contextra-types       Shared Domain-Typen (Cell, Edge, Trust, Provenance)    [safe]
├─ contextra-ports       Port-Traits (Vfs, Clock, Entropy, Cipher)             [safe]
├─ contextra-wire        Kanonische Serialisierung, Lane-Codecs                [safe]
├─ contextra-simd        SIMD-Kernels (Distanz, Quantisierung)                 [unsafe isoliert]
├─ contextra-sys         System-Abstraktionen, io_uring/pread                  [unsafe isoliert]
├─ contextra-durable-fs  Durables Dateisystem (fsync, O_DIRECT, Manifest)      [safe]
├─ contextra-crypto      BLAKE3, AEAD, Schlüsselhierarchie, Signaturen         [safe, auditiert]
├─ contextra-store       Kernel-KV-Store (Batches, Snapshots, Keyspaces)       [safe]
├─ contextra-mvcc        MVCC/SSI-Isolation über dem Kernel                     [safe]
├─ contextra-vector      Vektorsegmente, Vamana/DiskANN, Delta-Index           [safe + SIMD]
├─ contextra-text        Invertierter Index, BM25, FST                         [safe]
├─ contextra-graph       Adjazenz, Expansion, PPR, CSR-Segmente                [safe]
├─ contextra-db          Tabellenschicht, Schema, Sekundärindizes               [safe]
├─ contextra-rank        Score-Fusion, Kalibration, RRF                        [safe]
├─ contextra-adapt       Adaptive Parameter, Drift-Detection                    [safe]
├─ contextra-engine      Context Engine (Recall, Pack, Render, Work Queue)     [safe]
├─ contextra-cognition   Höhere Abstraktion (Konsolidierung, Profiling)         [safe]
├─ contextra-agent       Sitzungs-API, Tool-Fassade, Fehlerkatalog, Quoten     [safe]
├─ contextra-checkpoint  Checkpoint/Snapshot-Management                         [safe]
├─ contextra-kvcache     Prefix-/KV-Cache-Store (optional)                     [safe]
├─ contextra-ingest      Dokumentparsing, Chunking, Format-Erkennung           [safe]
├─ contextra-mcp         MCP-Server                                             [safe]
├─ contextra-router      Request-Routing, Load-Balancing                        [safe]
├─ contextra-license     Lizenzmanagement                                       [safe]
├─ contextra-infer-onnx  ONNX-Inference-Backend                                 [unsafe isoliert]
├─ contextra-infer-candle Candle-Inference-Backend                              [unsafe isoliert]
├─ contextra-infer-ollama Ollama-Client                                         [safe]
├─ contextra-testkit     Test-Utilities, Fakes, Simulation                      [test]
├─ contextra-sandbox     Sandboxed Execution (WASM Merge-Funktionen)            [safe]
├─ contextra-py          Python-Bindings (PyO3)                                 [unsafe isoliert]
├─ contextra             Top-Level-Crate (Fassade)                              [safe]
├─ contextra-bench       Benchmarks                                              [bench]
└─ xtask / xtask-heavy   Build-/Code-Generierungs-Tools
```

### 5.1 Abhängigkeitsregeln

1. **Abhängigkeiten zeigen nur nach unten.** Kein Crate importiert ein höheres.
2. **`#![deny(unsafe_code)]`** workspace-weit in `Cargo.toml`. `unsafe` erlaubt nur in: `contextra-simd`, `contextra-sys`, `contextra-infer-onnx`, `contextra-infer-candle`, `contextra-py`.
3. **Jede Schicht hängt an Traits** der darunterliegenden, nicht an deren konkreten Strukturen → Fakes (In-Memory-Kernel, `MemVfs`, `ManualClock`) überall möglich.
4. **Rust Edition 2021, stable Toolchain** (≥ 1.89).
5. **Serialisierung:** `rkyv`/`bytemuck` für Extent-Verzeichnisse und Lanes (Zero-Copy). Alle On-Disk-Formate besitzen Magic, Versionsnummer, Feature-Flags.
6. **Bibliotheken-Policy:** Eigenbau dort, wo atomare Integration mit dem Commit-Modell nötig ist. Fremdcrates für Krypto, Kompression, Parser.

### 5.2 Seam-Traits (Port-Schnittstellen)

```rust
// ── L1: Kernel-Fassade ──
pub trait KvStore: Send + Sync {
    type Snap: KvSnapshot;
    fn snapshot(&self) -> Self::Snap;
    fn apply(&self, batch: WriteBatch) -> Result<CommitInfo, ApplyError>;
    fn pin(&self, seq: Seq, why: PinReason) -> PinGuard;
    fn subscribe(&self, l: Arc<dyn CommitListener>);
}

// ── L2: Objektspeicher ──
pub trait ObjectStore: Send + Sync {
    fn put(&self, kind: ObjKind, bytes: &[u8]) -> Result<ObjectId>;  // idempotent
    fn get(&self, id: &ObjectId) -> Result<Bytes>;
    fn has(&self, id: &ObjectId) -> Result<bool>;
    fn flush_durable(&self) -> Result<DurableMark>;
}

// ── L3: Baum-Engine ──
pub trait TreeEngine {
    fn get(&self, root: &ObjectId, k: &[u8]) -> Result<Option<Bytes>>;
    fn range(&self, root: &ObjectId, r: KeyRange, dir: Dir)
        -> Box<dyn Iterator<Item=Result<(Bytes,Bytes)>>>;
    fn apply(&self, root: &ObjectId, muts: &mut dyn Iterator<Item=Mutation>)
        -> Result<ObjectId>;
    fn diff(&self, a: &ObjectId, b: &ObjectId, r: KeyRange)
        -> Box<dyn Iterator<Item=Result<DiffEntry>>>;
    fn merge3(&self, base: &ObjectId, ours: &ObjectId, theirs: &ObjectId,
              res: &dyn MergeResolver) -> Result<MergeOutcome>;
}

// ── L0: Querschnitt (Erweiterungspunkte) ──
pub trait MergeResolver  { fn resolve(&self, c: &Conflict) -> Resolution; }
pub trait CommitHook     { fn pre_commit(&self, c: &CommitDraft) -> Result<(), HookRejection>; }
pub trait AccessPolicy   { fn check(&self, p: &Principal, a: &Action, t: &Target) -> Result<(), Denied>; }
pub trait DerivedIndex   {
    fn id(&self) -> &str;
    fn update(&self, prev: Option<&ObjectId>, next: &ObjectId,
              diff: &mut dyn DiffStream) -> Result<()>;
}
```

---

## 6. Typische Zugriffsmuster

| # | Muster | Beschreibung | Häufigkeit |
|---|--------|-------------|------------|
| 1 | **Recall unter Budget** | „Gib mir die besten ≤ 6 000 Token Kontext zu dieser Anfrage." | häufigste Leseoperation (~70 %) |
| 2 | **Fenster-Lesen** | „Gib mir die Turns 40–60 dieser Episode, höchstens 3 000 Token." | ~10 % |
| 3 | **Expand** | Agent sieht Handle `c:k7qd` in kompakter Form und fordert Volltext an. | ~10 % |
| 4 | **Append-lastiges Schreiben** | Episoden, Tool-Ergebnisse, Beobachtungen — kleine Cells, hohe Rate, fast nie Updates in place. | häufigste Schreiboperation |
| 5 | **Fork → Arbeiten → Merge/Verwerfen** | Pro Plan-Schritt oder Experiment. | ~5 % der Transaktionen |
| 6 | **Hintergrundarbeit** | Konsolidierung, Re-Embedding, Vergessen/Abwerten. | kontinuierlich, niedrige Priorität |

---

## 7. Entscheidungsregister

| ID | Entscheidung | Begründung |
|---|---|---|
| C-01 | Persistente inhaltsadressierte Prolly-Bäume als Versionierungsgrundlage | History-unabhängig, strukturelles Sharing, O(Δ·log n) Diff/Merge |
| C-02 | Schichten: L0–L6; Abhängigkeiten nur nach unten | Testbarkeit, Austauschbarkeit, klare Verantwortung |
| C-03 | Objekte in Packs (Quelle der Wahrheit), Index rekonstruierbar | Crash-Sicherheit ohne Doppelbuchführung |
| C-04 | Zwei Ebenen: Transaktion und Commit; Journal für G3-Zeitreise | Feingranulare Historie bei effizienter Speicherung |
| C-05 | Verschlüsselte Repos nutzen keyed ObjectIds | Kein Existenz-Orakel |
| C-06 | Sitzung = Branch-Arbeitskopie mit Overlay, Snapshot- oder Serializable-Isolation | Agent-natürliches Arbeitsmodell |
| C-07 | Idempotenzschlüssel für alle Commits; LLM-lesbarer Fehlerkatalog | Unzuverlässige Clients |
| C-08 | SQL über SQLite-Virtual-Tables; keine eigene SQL-Engine | LLMs generieren SQL zuverlässig |
| C-09 | Abgeleitete Indizes im RootSet, mit Konsistenzvertrag `(index_id, root_id)` | Atomare Sichtbarkeit |
| C-10 | Cell als Primitive statt Key-Value; Hot-Head/Cold-Tail-Extent-Layout | Retrieval-optimiertes I/O |
| C-11 | Zoom-Knapsack (Multiple-Choice-Knapsack) statt Top-k | Überblick + Tiefe im selben Budget |
| C-12 | Work Queue statt Engine-internes LLM-Calling | Determinismus, Testbarkeit, Modellneutralität |
| C-13 | GC mit SATB+Remark; deterministisch simulierbar | Korrektheit bei concurrent Writes |
| C-14 | Ein Repository = eine Kernel-Instanz, Single-Process | Einfachheit; Mehrprozess über Server |
| C-15 | Trust-Labels propagieren über Ableitungsketten; Minimum-Vererbung | Konservativ-sicheres Default |
| C-16 | Gelerntes physisches Layout über Access-Trace mit 5% Exploration | Verhindert Rich-get-richer |
| C-17 | **Eigener FTS statt Tantivy** | Tantivy-Segmentformat ist nicht content-addressed, kann nicht im RootSet verankert werden, kein strukturelles Sharing |
| C-18 | **Handles global pro Repository**, nicht pro Branch | Stabilität über Branch-Grenzen; Union-Merge der Alias-Tabelle |
| C-19 | **Eventual Consistency für Zugriffsstatistik** | Soft-Signal für Ranking/Layout; exakte Werte nicht nötig |
| C-20 | **WASM-Merge-Funktionen mit Fuel-Limits** | Determinismus und Sicherheit durch Sandbox; Fallback auf ConflictCell |
| C-21 | **Embedded-first, Server ab Phase 2** | Library ist das Produkt; Server ist dünner Wrapper |

---

## 8. Vergleich mit Alternativen

| Aspekt | RocksDB/LSM | Dolt (Prolly) | LanceDB | Vektor-DBs | Git | **Contextra** |
|---|---|---|---|---|---|---|
| Primitive | Key → Bytes | SQL-Tabellen | Spalten + Vektoren | Vektor + Payload | Blobs/Trees | **Cell mit Facetten + Zoom** |
| Versionierung | nur Snapshots | Commits/Branches | Dataset-Versionen | limitiert | ja (Dateien) | **Branch/Fork/Merge, O(1)-Fork** |
| Atomar inkl. Indizes | Batch (KV) | ja (Tabellen) | ja (Manifest) | je nach System | ja (Dateien) | **ja, Indizes im Commit-Wurzelobjekt** |
| Vektor + FTS + Graph | nein (extern) | begrenzt | Vektor/FTS | Vektor, teils Hybrid | nein | **alle drei, versioniert, hybrid fusioniert** |
| Token-Budget-Awareness | nein | nein | nein | nein | nein | **ja (Token-Summen, Zoom-Knapsack)** |
| Layout-Optimierung | Schlüsselordnung | Schlüsselordnung | Spaltenlayout | Index-Layout | Packfile-Delta | **zugriffsgelerntes Co-Retrieval-Layout** |
| Abgeleitete Daten/Lineage | nein | nein | teils | nein | nein | **Derivation Cache + Staleness** |
| Trust/Taint | nein | nein | nein | nein | nein | **ja, propagierend** |
| Agent-Semantik | nein | nein | nein | nein | nein | **Idempotenz, Handles, LLM-Fehler** |

**Warum nicht „RocksDB + Aufsatz"?** LSM-Compaction sortiert nach Schlüssel und schreibt Daten mehrfach um; die zentralen Anforderungen (inhaltsadressierte Unveränderlichkeit, strukturelles Sharing, ortsunabhängiges Re-Layout, Index-Wurzeln im Commit) stehen quer dazu.

**Abgrenzung zu Memory-Frameworks:** Sie bauen Retrieval/Konsolidierung *über* generischen Datenbanken. Contextra verlagert die dafür nötigen Primitive in die Storage-Schicht, sodass sie atomar, versioniert und physisch optimiert sind.

---

*Ende SPEC-01. Weiter: [SPEC-02 Datenmodell](SPEC-02-DATA-MODEL.md)*
