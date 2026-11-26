# Contextra — Finale normative Spezifikation des Soll-Zustands

**Version 2.0 · November 2026 · Status: Final und verbindlich**
**Rolle:** Principal Rust Engineer (Systemprogrammierung, Durability, Concurrency, RAG-Speicherkerne)

> **Schlüsselwörter:** MUSS / MUSS NICHT = zwingende Anforderung. SOLL / SOLL NICHT = Regelfall, Abweichung nur per Entscheidungsregister. KANN = optional. Alle Performance-Zahlen sind **Designziele (Hypothesen)**, keine Messwerte; Validierung durch Prototypen auf Referenzhardware.

---

## Inhaltsverzeichnis

0. [Kurzfassung und Leitprinzipien](#0-kurzfassung-und-leitprinzipien)
1. [Produktvision, Scope und Profile](#1-produktvision-scope-und-profile)
2. [Datenmodell: Entkoppelte Cells, Facetten, Zoom](#2-datenmodell-entkoppelte-cells-facetten-zoom)
3. [Physisches Layout: Packs, Extents, Locator](#3-physisches-layout-packs-extents-locator)
4. [Versionierung, Transaktionen und Branches](#4-versionierung-transaktionen-und-branches)
5. [Indizes](#5-indizes)
6. [Context Engine](#6-context-engine)
7. [Abgeleitete Daten, Lineage und Work Queue](#7-abgeleitete-daten-lineage-und-work-queue)
8. [Sicherheit, Trust und Zugriffskontrolle](#8-sicherheit-trust-und-zugriffskontrolle)
9. [API-Oberflächen](#9-api-oberflächen)
10. [Crate-Architektur](#10-crate-architektur)
11. [Nebenläufigkeitsmodell](#11-nebenläufigkeitsmodell)
12. [Durability, Recovery, Garbage Collection](#12-durability-recovery-garbage-collection)
13. [Ingest-Pipeline](#13-ingest-pipeline)
14. [Adaptive Ranking und Fusion](#14-adaptive-ranking-und-fusion)
15. [KV-Cache-Store (optional)](#15-kv-cache-store-optional)
16. [Ressourcenmodell und Speicherbudgets](#16-ressourcenmodell-und-speicherbudgets)
17. [Performance-Ziele und KPIs](#17-performance-ziele-und-kpis)
18. [Teststrategie](#18-teststrategie)
19. [Roadmap](#19-roadmap)
20. [Risiken und Entscheidungsregister](#20-risiken-und-entscheidungsregister)
21. [Glossar](#21-glossar)

---

## 0. Kurzfassung und Leitprinzipien

### 0.1 Was ist Contextra?

Contextra ist ein **LLM-natives, versioniertes Storage-System und hybrider RAG-Speicherkern** in Rust. Es macht die Besonderheiten von LLM- und Agenten-Workloads — Token-Budgets, hybride Suche über denselben Snapshot, spekulative Verzweigungen, Mehrfachauflösungen (Zoom-Leiter), Provenienz, Trust/Taint und unzuverlässige Clients — zu **Primitiven der Speicherschicht**, statt sie als zerbrechliche Anwendungsschicht über generischen Datenbanken neu zu bauen.

Unterhalb der Versionierungsschicht ist Contextra ein **content-addressed Objektspeicher mit spaltenorientierten Hot-Head/Cold-Tail-Extents**. Oberhalb ist es eine **kontextbewusste Wissensdatenbank** mit deterministischem Truth-Layer, reproduzierbaren Index-Segmenten (mit Watermarks) und budgetoptimierender Context Engine. Der Übergangspunkt ist der **Commit** — ein Hash über das deterministische **`RootSet` (Truth Layer)**, begleitet von einem separaten **`IndexManifest`** für abgeleitete Suchstrukturen.

### 0.2 Die zehn Kernideen

| # | Idee | Beschreibung |
|---|------|-------------|
| 1 | **Entkoppelte Cell statt Key-Value** | Jede Wissenseinheit trennt unveränderlichen `CellContent` (`content_hash`, woran alle teuren Derivate hängen) von leichtgewichtigen versionierten `CellMeta` (Attribute, Trust, Taint, Belief, Gültigkeitszeit) und `Edge`-Bäumen. Tagging oder Fakt-Supersession schreibt nie den Rohinhalt neu. |
| 2 | **Truth ≠ Index** | Der Prolly-Tree-Commit-Hash (`RootSet`) deckt ausschließlich den deterministischen, history-unabhängigen Truth-Layer ab. Physische Index-Segmente (Vamana, FTS, CSR) hängen am separaten `IndexManifest` mit Watermarks und exaktem Flat-Fallback. |
| 3 | **Token-native Speicherung & Zoom-Knapsack** | Token-Zählungen pro Tokenizer-Familie und Zoom-Stufen (L0…Ln) sind persistiert und in Sequenzbäumen als Summen eingebettet. Der Context Packer maximiert Nutzen unter hartem Token-Budget mit Submodularitäts-/MMR-Redundanzabzug und Small-to-Big-Expansion. |
| 4 | **Dichter Adressraum (`Gid`) + MAC-geschützte Handles** | Jede Cell erhält eine dichte, monotone `Gid(u64)` für gemeinsame Roaring64-Filter über Vektor, BM25 und Graph. LLM-Handles tragen ein 2-Zeichen-BLAKE3-MAC-Tag (`c:k7qd-3f`), um halluzinierte Zufallstreffer zu 99,9 % auszuschließen. |
| 5 | **Hot-Head / Cold-Tail PAX-Extents** | Ein Extent packt bis zu *N* Cells spaltenorientiert: vorne kompakte Ranking-Lanes (Meta, Token-Counts, Embedding-Codes, Zoom-L2), hinten schwere Rohinhalte. Ein I/O für Ranking, ein zweiter nur für Gewinner. |
| 6 | **Agentensichere Versionierung & Merge-Gates** | Agenten schreiben **nie direkt nach `main`**, sondern auf O(1)-Session-Branches mit Savepoints je Tool-Call. Promotion nach `main` passiert deterministische **Merge-Gates (`G1`–`G7`)**. Unaufgelöste Konflikte leben als `ConflictCell`s im Merge-Request, nie auf `main`. |
| 7 | **Derivation Cache + Lineage-DAG** | Embeddings, Token-Zählungen und Summaries sind per `(content_hash, producer_id)` über Branches hinweg memoisiert und bei Quelländerung per Konstruktion als `stale` erkennbar. |
| 8 | **Provenienz, Konfidenz (BeliefBeta), Trust & Taint** | Jede Aussage trägt Pflichtprovenienz, `source_root`-Unabhängigkeit gegen Echokammern, `TrustTier` (Minimum-Vererbung) sowie orthogonale `TaintFlags` (Union-Vererbung). Nicht vertrauenswürdige Daten werden im Prompt gekapselt. |
| 9 | **Verlässliche Runtime-Grenzen & LLM-Fehler** | Idempotenz-Journal und Work-Queue-Leasing liegen in **nicht-versionierten Kernel-Sidecars** (kein Commit-Spam). Alle Fehler sind strukturiert, bei fehlender Berechtigung neutral und LLM-lesbar mit Remediation. |
| 10 | **Die Engine ruft kein LLM (Deterministische Ports)** | Uhr, Zufall, Dateisystem und Inferenz sind injizierbare Traits. Der Speicherkern ist vollständig deterministisch simulierbar. |

### 0.3 Leitprinzipien

1. **Truth ≠ Index.** Das `RootSet` bildet den alleinigen, deterministischen Wahrheitsraum. Index-Segmente sind abgeleitete, verwerfbare und rekonstruierbare Projektionen im separaten `IndexManifest`.
2. **Immutable first.** Truth-Chunks, Packs, Prolly-Knoten, Index-Segmente und Commits werden nie in-place überschrieben. Mutabilität existiert nur als atomar publizierte Pointer (`ArcSwap`), WAL-Tail, MemDelta/Memtable und nicht-versionierte Kernel-Sidecars.
3. **Ein Commit-Punkt, kein Mehr-Store-2PC.** Ein Commit ist dauerhaft, sobald sein WAL-Frame die geforderte Durability-Stufe erfüllt. Index-Builds laufen entkoppelt mit Watermarks.
4. **Identität ≠ Ort.** Content-Addressing (BLAKE3) bestimmt Identität; physische Platzierung `(PackId, ExtentNo, RowIndex)` liegt im austauschbaren Locator.
5. **Agenten schreiben nie nach `main`.** Schreibpfad ist `Session-Branch → Merge-Gates → Atomarer Merge`. Direktes Schreiben auf geschützte Branches erfordert explizite Admin-Privilege.
6. **Budget & Belegbarkeit statt Top-k.** Leseoperationen optimieren Informationsabdeckung und Nutzen pro Token unter harter Budgetgarantie, liefern `EvidenceSufficiency` (inkl. `None` / `NO_EVIDENCE_FOUND`) und verifizierbare Zitate.
7. **Abgeleitet ist ersetzbar, Herkunft ist Pflicht.** Nur Rohdaten + Provenienz sind Primärwahrheit; Derivate sind memoisiert und regenerierbar.
8. **Fail-closed in Sicherheit, Fail-open in Index-Verfügbarkeit.** Fehlende ACL/Capabilities blenden Treffer aus; fehlende Indizes fallen transparent auf exakten Scan zurück (`lane_coverage`).
9. **Kein stilles Verwerfen.** Weder Decay noch Merge noch Compaction löschen lebendige Daten stillschweigend. Decay steuert nur Ranking; Konflikte erzeugen sichtbare `ConflictCell`s; Löschungen sind explizite Tombstone-Commits (`retire`).
10. **Schichten zeigen nur nach unten.** Kein Crate hängt von einer höheren Schicht oder externen Inferenz-Implementierungen ab; Durchsetzung in CI.

### 0.4 Systeminvarianten (normativ, in Simulation geprüft)

| ID | Invariante |
|---|---|
| **INV-1** | **Deterministischer Truth-Hash:** Gleicher logischer Zustand (`RootSet`) ⇒ identischer `truth_hash`, unabhängig von Commit-Historie, Re-Layout oder Index-Compaction. |
| **INV-2** | **Snapshot-Einheit:** Alle Modalitäten (Vektor, BM25F, Graph, Filter, Fusion, Packer) einer Query lesen exakt einen konsistenten Branch-Snapshot `(head, mem_delta_seq, now)`. |
| **INV-3** | **Prefix-Konsistenz nach Crash:** Nach Recovery entspricht der sichtbare Zustand exakt einem Präfix der gemäß Durability-Klasse bestätigten WAL-Commits. |
| **INV-4** | **Index ⊆ Truth & Watermark-Deckung:** Jeder Index-Treffer wird gegen den Snapshot validiert; Zeilen nach `watermark_seq` werden exakt aus MemDelta/Delta-Scans ergänzt. |
| **INV-5** | **ACL vor Ranking:** Zugriffs-Bitmaps schränken ANN-/BM25-/Graph-Operatoren direkt ein; Scores, Zähler und Fehlermeldungen leaken nie unberechtigte Existenz. |
| **INV-6** | **Idempotenz:** Gleicher `IdempotencyKey` + gleicher Payload-Hash ⇒ identisches Ergebnis ohne zweiten Commit; abweichender Payload ⇒ `idempotency_mismatch`. |
| **INV-7** | **`main`-Integrität:** Geschützte Branches enthalten nie unaufgelöste Merge-Konflikte; Konflikte leben auf `mr/*` oder Feature-Branches. |
| **INV-8** | **Budget-Garantie:** `pack_result.budget_used ≤ request.budget.max_tokens` gilt in 100 % der Fälle (inkl. Rendering-Overhead). |
| **INV-9** | **GC-Sicherheit:** Kein von einem Root erreichbares Objekt wird physisch entfernt. |
| **INV-10** | **Speichergrenzen:** Kein Teilsystem überschreitet sein Speicherbudget; Überschreitung führt zu Backpressure, nie zu OOM. |
| **INV-11** | **Format-Versionierung:** Jede Datei trägt Magic, Formatversion, Feature-Flags; unbekannte inkompatible Flags verhindern das Öffnen. |
| **INV-12** | **Branch-Isolation:** Ein Branch ändert die Sicht eines anderen Branches nie, außer durch Merge/Rebase. |

---

## 1. Produktvision, Scope und Profile

### 1.1 Was Contextra ist

Contextra ist eine **eingebettete Datenbank und RAG-Engine** für LLM-Agenten und Unternehmens-Wissensbestände mit folgenden Eigenschaften:

- **Versioniert:** Git-artige Commits, Branches, Fork, Merge, Diff, Zeitreise — atomar über Primärdaten und Indizes.
- **Token-bewusst:** Budget als First-Class-Parameter, Zoom-Stufen, Token-Summen-Bäume.
- **Hybrid-Retrieval:** Vektor + BM25F + Graph + Temporal + Attribute, fusioniert in einer Pipeline über denselben Snapshot und denselben `Gid`-Adressraum.
- **Agent-sicher:** Idempotenz, spekulative Branches, Merge-Gates, Trust-Labels, LLM-lesbare Fehler.
- **Eingebettet + Server:** Primär als Rust-Library, mit optionalem MCP-/gRPC-/Arrow-Flight-Server.

### 1.2 Deployment-Modell

| Aspekt | Entscheidung |
|---|---|
| **Primärmodus** | Eingebettete Library (`contextra`), vergleichbar mit SQLite/RocksDB |
| **Server-Modus** | Dünner MCP-/gRPC-/Arrow-Flight-/HTTP-Daemon für Multi-Prozess-/Netzwerkzugriff |
| **Instanz-Modell** | Ein Repository/Store = eine Writer-Instanz (`flock`), Single-Process; Mehrprozess über Server |
| **Tiering** | RAM (Buffer Pool + MemDelta) ↔ NVMe (Packs/Segmente) ↔ Objektspeicher (S3-kompatibel, ab Phase 4) |
| **Plattformen** | Linux x86-64/aarch64 (primär; `pread`/`pwrite` + opt. `io_uring`), macOS/Windows (`DurableFs`+`pread`). **Kein `mmap` im Produktionspfad** |

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
| Verteilter Multi-Writer-Konsens in v1 | Single-Node; asynchrone Replikation ab Phase 4 |
| Eigene vollständige SQL-Engine | Rust-API, JSON-Plan, MCP; optional SQLite Virtual Tables |
| Git-Wire-Protokoll-Kompatibilität | Export/Import möglich, kein Protokollziel |
| Einzeldatei-Format | Verzeichnis nötig für WAL, Packs, Manifest, Sidecars |
| DSGVO-Löschkette & Löschnachweise | Vollständig ausgeschlossen. Löschung = `retire` + Retention + `purge` |
| Eigene LLM-Inferenz im Speicherkern | Inferenz über Port-Traits und externe Worker |
| Forschungs-Heuristiken im Kern | Bandit-Routing, 6 PPR-Varianten usw. entfallen zugunsten je eines validierten Standard-Algorithmus |
| Trigger/Stored Procedures in v1 | Über Commit-Subscriptions und Work Queue |

### 1.5 Typische Zugriffsmuster

| # | Muster | Beschreibung | Anteil |
|---|---|---|---|
| 1 | **Recall unter Budget** | „Gib mir die besten ≤ 6 000 Token belegten Kontext mit Abstention-Check." | ~65 % |
| 2 | **Fenster-Lesen (`window`)** | „Gib mir die Turns 40–60 dieser Episode, ≤ 3 000 Token" in O(log n). | ~10 % |
| 3 | **Progressive Disclosure (`expand`)** | Agent prüft `c:k7qd-3f` (L1/L2) und lädt selektiv L0 nach. | ~15 % |
| 4 | **Append- und Revision-Ingest** | Episoden, Tool-Ergebnisse, Dokument-Revisionen (per ContentHash-Diff inkrementell). | Hauptlast Schreiben |
| 5 | **Session-Fork → Gates → Merge** | Agenten arbeiten isoliert auf Branches; Promotion nur nach bestandenen Gates. | ~5 % Transaktionen |
| 6 | **Hintergrundarbeit (Sidecar-Queue)** | Asynchrone Embeddings, Summaries, Compaction, Vamana-Builds, Re-Layout. | kontinuierlich |

---

## 2. Datenmodell: Entkoppelte Cells, Facetten, Zoom

### 2.1 Namespace und Katalog

Ein **Namespace** ist die Einheit für Schema, Richtlinien und Zugriff. Der **Katalog** (Namespace-Policies, Embedding-Spaces, Tokenizer-Registry, Analyzer-Profile, ACL-Gruppen) ist als Prolly-Tree (`policies` im `RootSet`) **versioniert** (Zeitreise gilt auch für Schemata).

**Namespace-Klassen (`MemoryClass`):**

| Class | Typischer Inhalt | Standard-Retention & Decay |
|---|---|---|
| `episodic` | Konversationsturns, Tool-Ergebnisse | `hot_days=14`, `warm_days=180`, `decay=30d` |
| `semantic` | Claims, Entity-Profile, Definitionen | `history_days=365`, `decay=180d` |
| `procedural` | Skills, Runbooks, Workflows | unbegrenzt, `decay=none` |
| `working` | Scratchpad, Hypothesen, Session-Pläne | `ttl=24h` |
| `artifact` | Code, Reports, Dokumente | unbegrenzt, `decay=730d` |
| `reference` | Handbücher, API-Docs, Richtlinien | unbegrenzt, `zoom_down_after=365d` |

**Namespace-Policy (vollständig):**

```toml
[namespace."agent/episodic"]
class                 = "episodic"
chunker               = "markdown-v2"
tokenizers            = ["cl100k", "o200k"]
embedders             = ["emb-768-v3"]
active_embedder       = "emb-768-v3"
zoom_ladder           = ["L0:raw", "L1:summary<=200tok", "L2:title<=24tok"]
default_trust         = "tool-unverified"
min_main_trust        = "tool-verified"
acl_default           = "grp:default"
functional_predicates = ["current_status", "owner", "endpoint_url"]
max_cell_size         = "256KiB"
decay                 = { kind = "exponential", half_life_days = 30, floor = 0.1 }
retention             = { history_days = 180, branches_days = 7, hot_days = 14 }
merge                 = { text = "diff3", attrs = "field", counters = "sum", edges = "add-wins", default = "manual" }
durability            = "durable"
sequence_enabled      = true
```

**Mandantenisolation:** Ein Store (`<repo-root>/`) bildet die **harte** Isolationsgrenze (eigener `repo_seed`, eigene Schlüssel, eigener WAL, eigener Derivation Cache). Innerhalb eines Stores trennen Namespaces + ACL-Bitmaps Rollen und Teams. Kein mandantenübergreifender Derivation-Cache (kein Seitenkanal).

### 2.2 Cell — die entkoppelte Wissensprimitive

#### 2.2.1 Identitäts- und Adressierungsmodell

Jede Cell besitzt vier klar getrennte Identitäten:

```rust
/// 1. Logische, branch- und versionsübergreifend stabile Identität.
pub struct CellId { pub ns: NsId, pub key: CellKey }
pub struct CellKey(pub [u8; 16]); // ULID-kompatibel oder rein zufällig

/// 2. Globaler dichter Index-Adressraum (monoton pro Repo, NIE wiederverwendet).
pub struct Gid(pub u64); // Kompakte Roaring64-Bitmaps für ACL, Filter, Graph-Frontier

/// 3. Content-addressed Hash (BLAKE3-256 über kanonischen Klartext).
pub struct ChunkId(pub [u8; 32]);
pub type ContentHash = ChunkId;

/// 4. Optionaler externer Anwendungsschlüssel (max 512 B, pro Namespace eindeutig).
pub struct ExternalKey(pub CompactString); // z. B. "sharepoint:/docs/arch.md#chunk-04"
```

**Kern-Invarianten:**
- `CellId` und `Gid` bleiben über **alle** Versionen einer Cell stabil.
- `content_hash` ändert sich **ausschließlich**, wenn sich der unveränderliche Inhalt ändert.
- `meta_hash` ändert sich bei Attribut-/Trust-/Gültigkeits-Updates, **ohne** `content_hash` oder davon abhängige Derivate zu invalidieren.

#### 2.2.2 Entkoppelte Cell-Struktur

```rust
/// Eintrag im `primary`-Prolly-Tree (Schlüssel: CellId). ~74 Bytes.
pub struct CellRecord {
    pub gid: Gid,
    pub content_hash: ChunkId,
    pub meta_hash: ChunkId,
    pub flags: CellFlags, // TOMBSTONE | ARCHIVED | PINNED | STALE
}

/// Teil A: Unveränderlicher Inhaltsteil (content_hash = BLAKE3(canonical(CellContent))).
pub struct CellContent {
    pub raw: RawPayload,          // Inline (≤ 2 KiB) oder BlobRef
    pub mime: CompactString,
    pub boundary_safe: bool,
    pub chunker_id: Option<CompactString>,
    pub origin_prov: OriginProvenance,
}

/// Teil B: Versionierter Metadatenteil (meta_hash = BLAKE3(canonical(CellMeta))).
pub struct CellMeta {
    pub id: CellId,
    pub gid: Gid,
    pub kind: CellKind,
    pub parent: Option<Gid>,        // Hierarchie: Document → Section → Chunk
    pub level: u8,                   // 0 = Rohchunk, 1 = Abschnitt, 2 = Dokument, 3 = Thema
    pub attrs: Attrs,                // typisierte Attribute
    pub time: BiTemporal,
    pub trust: TrustTier,
    pub taint: TaintFlags,           // EXTERNAL | INJECTION_SUSPECT | UNVERIFIED_TOOL | PII
    pub belief: BeliefBeta,          // Beta(alpha, beta)
    pub grounding: GroundingLevel,   // G0..G3
    pub acl: AclRef,
    pub prov_id: ProvId,
}
```

**Teil C (Kanten):** Kanten liegen im eigenen **`graph`-Prolly-Tree** (`(src_gid, edge_kind, dst_gid, edge_id) → EdgePayload` plus Reverse). Kanten-Änderungen modifizieren weder `content_hash` noch `meta_hash`.

**Claim-Modell für `CellKind::Fact`:** Strukturierte Claim-Felder in `attrs` (`subject_gid`, `predicate`, `object_ref | literal`, `polarity`). Funktionale Prädikate erlauben pro `(subject, predicate)` und `valid_at` höchstens ein gültiges Objekt.

### 2.3 Provenienz, Grounding und Konfidenzmodell

```rust
/// Append-only Provenance-Eintrag im `provenance`-Prolly-Tree.
pub struct ProvRecord {
    pub id: ProvId,                      // (subject_gid, ev_seq)
    pub activity: ActivityKind,          // Ingest | Extract | Infer | Summarize | HumanEdit | Merge
    pub origin: Origin,
    pub source_uri: Option<CompactString>,
    pub source_hash: Option<ChunkId>,
    pub source_span: Option<(u64, u64)>,
    pub quote_hash: Option<[u8; 16]>,
    pub source_root: [u8; 16],           // Wurzel der Ableitungskette (gegen Echokammern!)
    pub extractor_conf: f32,
    pub observed_at: Hlc,
    pub run_id: Option<RunId>,
}

/// Beta-Verteilung für Unsicherheit und Evidenz-Akkumulation.
pub struct BeliefBeta {
    pub alpha: f32,
    pub beta: f32,
    pub source_roots: SmallVec<[[u8; 16]; 4]>, // deduplizierte unabhängige Wurzeln
}
```

**Normative Konfidenz-Regeln:**
1. **Priors:** `System`/`Verified` = `Beta(4,1)`, `Curated`/`ToolVerified` = `Beta(2,1)`, `Unknown` = `Beta(1,1)`, `AgentInferred`/`Hypothesis` = `Beta(1,2)`.
2. **Unabhängigkeitsgebot (`source_root`):** Eine Bestätigung erhöht `alpha` **nur**, wenn `prov.source_root ∉ belief.source_roots` (verhindert Echokammern).
3. **Ableitung (Noisy-AND):** `lcb(derived) = min(lcb(parent_i)) * extractor_conf`.
4. **Planer-Score:** Der Context Planner rechnet mit der **unteren 10-%-Konfidenzgrenze** der Beta-Verteilung.

**Grounding-Stufen:**
- **`G0`** (`VerifiedSpan`): Quelle vorhanden, Spanne verifiziert.
- **`G1`** (`UnverifiedSource`): Quelle vorhanden, Spanne nicht geprüft.
- **`G2`** (`Inferred`): Agenteninferenz mit Eltern-Gids und Noisy-AND-Konfidenz.
- **`G3`** (`Hypothesis`): Keine Quelle; Quarantäne (im Standard-Recall ausgeschlossen).

### 2.4 Facetten (Lanes)

| Lane | Inhalt | Hot/Cold |
|---|---|---|
| `meta` | gid, CellId, kind, trust, taint, belief, valid_from/to, acl, AttrBloom | **hot** |
| `tokcount` | Exakte Token-Anzahl je Tokenizer (u32) | **hot** |
| `zoom.L2` | Extraktiver Titel / Gist (≤ 24 Tok) | **hot** |
| `zoom.L1` | Zusammenfassung (≤ 200 Tok) | **hot** |
| `emb.q` | Quantisierter Embedding-Kopf (SQ8/binär/Matryoshka-Präfix) | **hot** |
| `raw` (= `L0`) | Kanonischer Rohinhalt | **cold** |
| `tok.ids` | Bitgepackte Token-IDs (optional) | **cold** |
| `emb.full` | Volles Embedding (f16/f32) für Re-Ranking | **cold** |
| `attrs.ext` | Große/seltene Attribute | **cold** |

### 2.5 Zoom-Leiter

| Stufe | Name | Ziel-Token | Erzeugung |
|---|---|---|---|
| **L0** | `raw` | unbegrenzt | Original-Chunk; Eltern fassen Kinder per `Summarizes` zusammen (Small-to-Big) |
| **L1** | `summary` | ≤ 200 | Abstraktiv (Work Queue) oder deterministisch extraktiv (Fallback) |
| **L2** | `title` | ≤ 24 | **Synchron & deterministisch beim Commit** (Überschrift, AST-Signatur, erster Satz) |
| **L3** | `handle` | ~4 | Engine-intern (`c:<gid>-<mac2>`) |

**Lineage-Staleness:** Jede Zoom-Stufe speichert `derived_from = content_hash`. Ändert sich die Quelle, ist das Derivat sofort als `stale` erkennbar.

### 2.6 MAC-geschützte Handles

```rust
/// Textform: "c:<seq_crockford>-<mac2>", z. B. "c:k7qd-3f"
pub struct Handle {
    pub gid: Gid,         // Crockford-Base32, mind. 4 Zeichen
    pub mac: [u8; 2],     // 10 Bit BLAKE3-MAC
}
```

**MAC-Berechnung:** `mac = trunc_10bit(BLAKE3::keyed_hash(hkdf(repo_seed, "contextra-handle-mac-v1"), gid))`. Ein halluziniertes Handle schlägt in **99,90 %** der Fälle bei der Prüfung fehl. Bei unbekanntem Handle liefert die Engine `did_you_mean` mit Damerau-Levenshtein-Kandidaten (max. 3, nur ACL-berechtigte).

### 2.7 Bitemporalität

```rust
pub struct BiTemporal {
    pub ts_tx: Hlc,                    // Transaktionszeit (Commit-HLC, unveränderlich)
    pub valid_from: Option<Timestamp>,
    pub valid_to: Option<Timestamp>,
}
pub struct Hlc(pub u64); // 48b ms + 16b logisch
```

**Funktionale Prädikate & Supersession:** Neuer Fakt für `(subject, functional_predicate)` mit neuerer `valid_from` → Engine schließt atomar `old.meta.time.valid_to` (nur CellMeta, kein Re-Embedding) und legt `Supersedes`-Kante an. Gleichrangige Widersprüche erzeugen `ConflictCell`.

### 2.8 Typisierte Kanten

```rust
pub struct Edge {
    pub id: u64,
    pub kind: EdgeKind,         // PartOf, Follows, Cites, DerivedFrom, Supersedes, Contradicts, Supports, ...
    pub src: Gid, pub dst: Gid,
    pub weight: f32,
    pub valid: Interval<Hlc>,
    pub trust: TrustTier,
    pub belief: BeliefBeta,
    pub prov_id: ProvId,
    pub attrs: SmallAttrs,
}
```

Kanten liegen im `graph`-Prolly-Tree: Schlüssel `G:(src, etype, dst, eid)` plus Reverse `R:(dst, etype, src, eid)`. Merge-Strategie: `add-wins` (Add/Add mit gleicher Kante ergibt eine Kante; Delete gegen Modify per Namespace-Policy).

### 2.9 Statistik-Sidecar

Zugriffsstatistiken sind **NICHT** Teil von `CellContent`, `CellMeta` oder `RootSet`:

```rust
/// sidecar/access_stats.cxs — eventual-consistent, nicht im WAL.
pub struct AccessStats {
    pub gid: Gid,
    pub read_count: u32, pub expand_count: u32,
    pub last_read: Timestamp, pub last_expand: Timestamp,
    pub salience: f32,
}
```

---

## 3. Physisches Layout: Packs, Extents, Locator

### 3.1 Verzeichnislayout

```
store/
  CURRENT                        # doppelt gepufferter Superblock
  LOCK                           # exklusiver Prozess-Lock (flock)
  wal/000042.cxw …               # 64 MiB-WAL-Segmente, vorallokiert
  manifest/000007.mlog …         # append-only Manifest-Log, CRC + Hash-Verkettung
  truth/seg-<id>.cxp             # Truth-Packs (immutable)
  index/<seg-id>/<kind>.cidx     # abgeleitete Index-Segmente (vec/fts/csr)
  blobs/<aa>/<hash>              # große Inhalte, content-addressed
  sidecar/work_queue.cxs         # Work Queue (WAL-gesichert, nicht im RootSet!)
  sidecar/idempotency.cxs        # Idempotenz-Journal (WAL-gesichert, nicht im RootSet!)
  sidecar/access_stats.cxs       # Zugriffsstatistik (eventual-consistent)
  tmp/                           # Schreibziele vor atomarem Rename
```

Alle Dateien sind **write-once** (außer WAL-Tail, Manifest-Tail, Superblock und Sidecars). Ersetzung durch neue Datei + Manifest-Eintrag. Nach dem Schreiben: `fdatasync(datei)`, `rename`, `fdatasync(verzeichnis)`.

### 3.2 Pack-Format (`.cxp`)

Packs enthalten mehrere **Extents** (I/O-Einheiten, 256 KiB–1 MiB, 4 KiB-aligned).

**Extent-Layout: Hot-Head / Cold-Tail (PAX-artig):**

```
Extent (z. B. 512 KiB, 4 KiB-aligned)
┌────────────┬──────────────────────────────────────┬─────────────────────────────┐
│ Header     │ HOT HEAD                             │ COLD TAIL                   │
│ Dir+CRC    │ meta │ tokcount │ zoom L2/L1 │ emb.q │ raw │ tok.ids │ emb.full     │
│ Offsets    │ edges │ trust │ attr-Bloom           │ (komprimiert, dict-basiert) │
└────────────┴──────────────────────────────────────┴─────────────────────────────┘
      │                      │                                   │
      └─ 1× Read (erste      └─ Ranking, Filterung, Packing      └─ nur für Gewinner:
         32–64 KiB)             komplett ohne Cold Tail              2. Read im selben Extent
```

Innerhalb des Extents sind Lanes **spaltenorientiert** (PAX-artig): SIMD-freundliche Distanzberechnung über `emb.q`, Prädikatsauswertung über `meta`.

### 3.3 Zweistufiger Locator

| Locator | Schlüssel | Wert | Zweck |
|---|---|---|---|
| **CellLocator** | `Gid` | `(PackId, ExtentNo, RowIndex: u16)` | Adressiert eine Cell im PAX-Extent; Re-Layout biegt nur diesen um |
| **ObjectLocator** | `ChunkId` | `(PackId, Offset, Len)` | Adressiert Prolly-Knoten, Blobs und Index-Segmentblöcke |

### 3.4 Optionales Affinity Re-Layout

**Prinzip:** Wer zusammen gelesen wird, soll zusammen liegen. Datenquellen: Co-Retrieval (Cells im selben Recall), Struktur (gleiche Episode/Dokument), Zeit (Entstehungsnähe), Semantik (gleiche Vektorzelle als Fallback).

**Verfahren:** Gewichteter Co-Access-Graph → Partitionierung in Extent-Gruppen → neue Extents schreiben, **nur CellLocator umbiegen**. ChunkIds, Commits bleiben unverändert. 5 % Exploration-Anteil gegen Rich-get-richer.

### 3.5 Kompression

| Lane | Verfahren |
|---|---|
| Text (`raw`, Zoom) | zstd Level 3 mit trainiertem Wörterbuch pro Namespace-Klasse |
| Token-IDs | Bitpacking nach Vokabulargröße + Delta/zstd |
| Embeddings | SQ8/PQ/RaBitQ als Head-Code; F16/F32 im Tail; Vektoren unkomprimiert für O_DIRECT |
| Attribute | Dictionary-/RLE-/Frame-of-Reference (spaltenweise) |
| Kanten | Delta-codierte Adjazenzlisten |

### 3.6 Buffer Pool

Reiner Lese-Cache über **unveränderliche** Daten (W-TinyLFU oder CLOCK-Pro), seitenausgerichtete Rahmen (4/16 KiB), Pin-Zähler je Rahmen. **Getrennte Budgets pro Klasse** (Truth-Innenknoten, Blätter, Postings, Graph, Vektorsektoren, Metadaten), damit ein großer Scan heiße Strukturen nicht verdrängt.

---

## 4. Versionierung, Transaktionen und Branches

### 4.1 Zustandsmodell: Prolly-Tree + MemDelta

**Prolly-Tree** (probabilistischer B-Baum): Knotengrenzen aus Hash der Schlüssel. Eigenschaften:
- **History-unabhängig:** gleicher Inhalt ⇒ gleiche Struktur ⇒ gleicher Hash.
- **Strukturelles Sharing:** Forks und Commits teilen unveränderte Teilbäume.
- **O(Δ·log n) Diff und Merge.**
- **Augmentierbar:** Innere Knoten speichern Aggregate (Token-Summen, Zähler, Min/Max-Zeit).

**Schreibpfad mit MemDelta (LSM-artig):**
```
Write ──► WAL (Group Commit) ──► Branch-MemDelta (sortiert, im RAM)
                                       │
                           Hintergrund-"Fold" (batched)
                                       ▼
                    neue Prolly-Knoten + Packs ──► Manifest-Eintrag
```

Leser sehen immer `Basis-Baum ⊕ MemDelta` (Merge-Iterator).

**Adversarial-Schutz:** Boundary-Funktion nutzt `repo_seed` (keyed). Maximalgröße begrenzt degenerierte Chunks. Anfangsparameter: Ziel-Knotengröße 8 KiB, min 2 KiB, max 32 KiB.

### 4.2 `RootSet` und `IndexManifest` (löst Truth/Index-Trennung)

```rust
/// Rein logische, 100 % deterministische Prolly-Wurzeln.
/// Einziger Hashraum des Commits — kein physischer Index-State!
pub struct RootSet {
    pub primary: ChunkId,       // CellRecord-Baum
    pub sequences: ChunkId,     // geordnete Folgen mit Token-Summen
    pub graph: ChunkId,         // Kanten-Baum (fwd + rev)
    pub aliases: ChunkId,       // CellId ↔ Gid
    pub policies: ChunkId,      // Namespace-Katalog
    pub conflicts: ChunkId,     // ConflictCells
    pub provenance: ChunkId,    // Provenienz-Baum
}

/// Separater Seitenzeiger — NICHT im RootSet-Hash!
pub struct IndexManifestRef {
    pub source_truth_hash: ChunkId,  // RootSet-Hash, aus dem gebaut
    pub watermark_seq: Seq,
    pub vec_segments: Vec<SegmentRef>,
    pub fts_segments: Vec<SegmentRef>,
    pub csr_segments: Vec<SegmentRef>,
    pub bitmap_segments: Vec<SegmentRef>,
}
```

### 4.3 Commit-Objekt

```rust
pub struct Commit {
    pub parents: SmallVec<[CommitId; 2]>,
    pub root: RootSet,
    pub index_manifest: IndexManifestRef,
    pub author: AgentIdentity,
    pub intent: Intent,           // run_id, step_id, idempotency_key, Begründung
    pub ts_tx: Hlc,
    pub seq: Seq,                 // globaler monotoner Commit-Zähler
    pub sig: Option<Ed25519Signature>,
}
```

### 4.4 Commit-Protokoll (crash-sicher)

```
1. begin        → Snapshot (Branch-Head, Seq); Read-Set-Tracking
2. buffer       → Writes im Transaktions-ChangeSet
3. validate     → OCC auf Schlüsselebene gegen Branch-Head; opt. SSI
4. WAL append   → Frame {CRC32c, LSN=Seq, Branch, Intent, IdempotencyKey, ChangeSet}
                  Group Commit: Leader/Follower, ≤ 200 µs oder ≥ 1 MiB, ein fdatasync
5. apply        → in Branch-MemDelta + synchrone Mini-Indizes (Flat-Vektoren, Mini-Invertindex)
6. publish      → atomarer ArcSwap auf neuen BranchState ← Sichtbarkeitspunkt
7. ack          → Client erhält CommitId
8. fold (async) → MemDelta → Prolly-Knoten + Packs → Manifest → WAL-Recycling
```

### 4.5 Branches, Sessions und Merge-Gates

- **Fork = O(1):** neuer Ref auf bestehenden Commit + leeres MemDelta. Ziel: < 100 µs.
- **Session-Branches:** `sessions/<agent>/<id>`, TTL, Quoten (Bytes, Entities, Write-Rate, Embedding-Calls, Tokens), `write_tier`, `gate_policy`.
- **Savepoint je Tool-Call:** benannt, O(1)-Rollback.
- **Speculative Branches:** N Strategien forken parallel; Bewertung wählt Gewinner; Rest verworfen.

**Merge-Gates (G1–G7):** Deterministische Prüfungen vor CAS auf `main`, diff-inkrementell auf virtuellem Ergebnis-Manifest:

| Gate | Prüfung | Modus |
|---|---|---|
| G1 Schema | Typen, Kardinalitäten, referentielle Integrität | blockierend |
| G2 Provenienz | Jeder neue Fakt hat Quelle oder Ableitung; Signatur gültig falls Pflicht | blockierend |
| G3 Blast Radius | Anteil geänderter/gelöschter `main`-Entitäten, Obergrenze (Default 1 %) | blockierend |
| G4 Widerspruch | Konfliktscan gegen `main` (synchron deterministisch + async semantisch) | blockierend/warnend |
| G5 Retrieval-Regression | Golden-Queries, Top-k-Überlappung und NDCG-Delta | warnend/blockierend |
| G6 Taint & Injection | Untrusted-Scan, Tier-Cap | blockierend |
| G7 Freigabe | Menschliche Bestätigung für Namespaces oder Tier `Verified` | konfigurierbar |

Nicht bestandenes Gate hält Merge-Request offen; Agent erhält strukturierte Fehlerbeschreibung mit betroffenen Fakten.

### 4.6 3-Wege-Merge (normative Regeltabelle)

| Situation | Regel |
|---|---|
| Nur eine Seite geändert | übernehmen |
| Beide identisch geändert | übernehmen |
| Disjunkte Felder geändert | feldweise zusammenführen |
| Gleiches Feld, verschiedene Werte | **ConflictCell** |
| `belief` | neu berechnen aus dedupliziertem Quell-Set |
| `trust` | niedrigere Stufe; Hochstufung nur durch Gate G7 |
| Kanten | Add-Wins-Mengensemantik (konfigurierbar: AddWins/RemoveWins/Manual) |
| Embeddings, Provenienz | nie Konflikt (content-addressed bzw. append-only, Vereinigung) |
| Abgeleitete Summaries (level > 0) | nicht mergen, `STALE` markieren und neu erzeugen |
| `custom(wasm)` | sandboxed Merge-Funktion mit Fuel-Limits (wasmtime) |

**Vektoren werden nie gemittelt.** Konflikte leben im Merge-Request, nie in `main`.

### 4.7 Durability-Stufen (pro Transaktion/Branch)

| Stufe | Verhalten | Einsatz |
|---|---|---|
| `volatile` | nur RAM, kein WAL | Tests, Scratchpad |
| `relaxed` | WAL geschrieben, fdatasync alle 100 ms | Session-Branches |
| `durable` (Group, Default) | Gruppen-fdatasync, Ack danach | `main`, kuratierte Branches |
| `strict` | fdatasync je Commit (keine Gruppierung) | Compliance-/Auditdaten |

`main` DARF NICHT `relaxed` oder `volatile` sein.

### 4.8 Idempotenz (Kernel-Sidecar)

`idem_key = H(session, step, tool, args)` im WAL-gesicherten Sidecar (`sidecar/idempotency.cxs`). Ein Retry mit identischem Key ist ein No-Op; abweichender Payload → `E_IDEMPOTENT_MISMATCH`.

### 4.9 Zeitreise (bitemporal)

- `AS OF COMMIT <hash>` → direkter Zugriff auf Manifest.
- `AS OF TIMESTAMP t` → binäre Suche im Commit-Index.
- `VALID AT t` → Filter auf Realzeit der Fakten.
- `AS OF TX t1 VALID AT t2` → „Was glaubten wir am Stichtag t1 über t2?"
- Reproduzierbarkeit: `(Commit, Query-Hash, Parameter)` reproduziert Ergebnis exakt (soweit Indizes deterministisch; Abweichung im Explain ausgewiesen).

---

## 5. Indizes

Alle Indizes sind **segmentbasiert, immutable, content-addressed** und im `IndexManifest` (nicht im `RootSet`-Hash!) verankert. **Watermark-Vertrag:** Jedes Segment trägt eine `watermark_seq`; Zeilen nach dieser Marke werden exakt aus MemDelta/Delta-Overlay ergänzt.

### 5.1 Gemeinsamer Adressraum (`Gid` ↔ `DocId`)

Segmente vergeben segmentlokale dichte `DocId(u32)`. Die Übersetzung `Gid → DocId` ist eine sortierte Verschmelzung (Segmente nach `(gid, born_seq)` sortiert). Graph-/ACL-/Filter-Ergebnisse als `Roaring64` über `Gid` werden pro Segment zu `Roaring32` über `DocId` übersetzt und filtern direkt ANN- und BM25-Suche.

### 5.2 Vektorindex

- **Flat:** SIMD-Scan für Segmente < 50 000 Vektoren. Exakt, ohne Graph.
- **HNSW (RAM):** SQ8-Codes, M=16, efC=200. Für ≤ ~8 Mio Vektoren.
- **Vamana/DiskANN (Standard für große Bestände):** R=64, L=100, α=1.2. Kompakte Codes (PQ/RaBitQ) im RAM, Nachbar-Sektoren (4 KiB, Codes + Adjazenz co-lokalisiert) auf NVMe. Beam-Search mit Rerank aus F16-Vollvektoren.
- **Delta-Index:** Frische Vektoren im MemDelta als flacher Puffer (SIMD, bis ~20k), danach Mini-HNSW im RAM → kein Index-Lag.
- **Compaction:** Größenabgestuft mit Faktor 4–8, Streaming-Merge (FreshDiskANN). Gelöschte Knoten als Routing-Knoten bis Compaction.
- **Liveness:** Versionierte Deletion/Tombstone-Bitmaps (Roaring, content-addressed) je Segment.
- **Gefilterte Suche (3 Strategien je nach Selektivität σ = |S|/N):**
  - **A (Pre-Filter, σ klein):** SIMD-Distanzen über Kandidatenmenge, wenn |S| ≤ τ_A (~20 000).
  - **B (In-Filter, mittleres σ):** Filter als Prädikat im Beam; bei σ < 10 % ACORN-Zwei-Hop-Expansion.
  - **C (Post-Filter, σ > 50 %):** Adaptives Over-Fetch.
  - **Laufzeit-Fallback:** B liefert < k gültige Treffer → Wechsel auf A.
- **Mehrere Spaces:** Vektoren verschiedener EmbeddingSpaces werden nie verglichen. Modellwechsel = neuer Space + Doppelbetrieb + `lane_coverage`.

### 5.3 Volltext (FTS/BM25F)

- Segmentbasierter invertierter Index mit **BM25F** (k1=1.2, b=0.75, je Feld konfigurierbar), globale Statistiken (N, df, avgdl) aus Segmentstatistiken im Manifest.
- **FST** (Finite State Transducer) für Term-Dictionary mit Fuzzy-Matching und Präfix-Suche.
- **Block-Max-WAND** für effizientes Top-k-Pruning. Postings in 128er-Blöcken, delta-codiert, SIMD-bitgepackt. Positionen optional (Phrasen).
- **Analyzer-Pipeline:** Unicode-Norm, Lowercasing, Sprachdetektion, Tokenisierung, Stemming (Snowball), Deutsche Komposita-Zerlegung, N-Gramme. Analyzer-ID im Segmentschlüssel; Änderung → kontrollierter Rebuild.
- Feldbezogene Gewichtung (Titel > Body > Attribute). Filter-Bitmaps überspringen ganze Postings-Blöcke.

### 5.4 Graph (CSR + PPR)

- **Truth:** Kanten im `graph`-Prolly-Tree `(src, etype, dst, eid)` plus Reverse.
- **Index:** CSR-Segmente (fwd + rev). Offsets Elias-Fano-codiert, Ziele delta-codiert/bitgepackt. Hub-Knoten (Grad > 10⁴) mit Skip-Index.
- **Operatoren:** BFS, k-Hop mit Budget-Limits, Kantentyp-Masken, Temporal-Pushdown.
- **Forward-Push-PPR** (approximativ, ε ≈ 10⁻⁴) für assoziatives Abrufen.
- **Community-Detection** (Leiden) als Batch-Job; speist Zusammenfassungshierarchie.
- **TraversalLimits sind Pflicht:** Tiefe 3, 100 000 besuchte Knoten, 10 000 Pfade, 200 ms. Bei Überschreitung Teilergebnis mit `truncated = true`.

### 5.5 Filter-Bitmaps und Attribute

- Zeitbereichs-Index (`valid_from/valid_to`, `ts_tx`) für Recency-Scoring und temporale Filter.
- Invertierte Attributindizes mit Roaring-Bitmaps über `Gid`.
- ACL-Gruppen-Bitmaps (vorberechnet, je Gruppe eine Roaring64).
- Filter-Bitmaps (ACL ∩ Zeit ∩ Trust ∩ Attribute ∩ Graph-Menge) werden im `Gid`-Raum gebildet und pro Segment zu `DocId`-Bitmaps übersetzt.

### 5.6 Token-Summen (augmentierter Baum)

Geordnete Folgen (`Doc → Chunks`, `Episode → Turns`) als Prolly-Keyspace `(SeqId, position) → (Gid, CellRecordRef)`. Innere Knoten tragen Token-Summen pro `TokenizerId`:

```rust
/// O(log n) ohne Inhaltszugriff.
fn window(seq: SeqId, anchor: CellId, budget: Tokens, tk: TokenizerId) -> Vec<CellId>;
```

---

## 6. Context Engine

### 6.1 Primitive Operationen

| Operation | Zweck |
|---|---|
| `recall(q, budget, policy)` | beste Kontextmenge unter Token-Budget → `EvidencePack` |
| `window(seq, anchor, budget)` | zusammenhängendes Fenster |
| `expand(handle, level)` | feinere/gröbere Stufe nachladen |
| `explain(recall_id)` | Scores, Quellen, Verworfenes |
| `neighbors(handle, edge_types, fanout)` | Graph-Nachbarschaft |
| `diff(a, b)` | menschen- und LLM-lesbare Änderungszusammenfassung |
| `verify(claims[])` | Citation-Verifier |

### 6.2 Recall-Pipeline

```
Query ──► Snapshot pinnen (Branch, Seq, now)
      ──► Filter-Bitmaps (ACL ∩ Zeit ∩ Trust ∩ Attribute ∩ Graph-Menge) [Gid-Raum]
      ──► Operatoren je Segment parallel: Vektor │ BM25F │ Graph │ Prior   (geteilte Top-k-Schwelle)
      ──► Fusion (RRF/Norm) ──► Validierung gegen Truth (Liveness/Sichtbarkeit)
      ──► optional Rerank (Cross-Encoder, deadline-gesteuert)
      ──► Pack (Zoom-Knapsack, MMR, Small-to-Big, Platzierung, Handles, Warnungen)
      ──► EvidencePack + Explain
```

### 6.3 Zoom-Knapsack

**Problem:** Gegeben Kandidaten *i* mit Stufen *ℓ* (Kosten `c[i][ℓ]` in Token, Nutzen `u[i][ℓ]`), wähle genau eine Stufe pro Kandidat (oder weglassen), sodass Summe der Kosten ≤ Budget und Gesamtnutzen — unter Redundanzabzug — maximal.

**Nutzenfunktion:**
```
u(d) = rel(d)^a · trust_lcb(d)^b · fresh(d)^c · quality(d)
```
- `rel`: kalibriert (isotonische Regression auf Feedback; ohne Feedback: normalisierter RRF-Rang).
- `trust_lcb`: 10-%-LCB der Beta-Verteilung.
- `fresh`: `max(floor, 2^(-Δt / T½))`.
- `quality`: Informationsdichte (IDF-Summe pro Token; Proxy: zstd-Kompressionsrate).

**Verfahren (submodularer Greedy mit Facility-Location-Abdeckung):**
1. Maximiere `F(S) = Σ_j u(j) · max_{d∈S} sim(j, d)` unter `Σ c ≤ B`.
2. Lazy Greedy über `ΔF / c` (Garantie ½(1 − 1/e) ≈ 0.32, praktisch nahe optimal).
3. Pro Dokument genau eine Rendition (Raw/Trimmed/Summary/Handle).
4. Near-Duplicate-Erkennung hart (SimHash/MinHash ≥ 0.9 → kanonischer Chunk + Alias); weiche Diversität über Abdeckungsterm.

**Small-to-Big:** Der Packer expandiert bei Bedarf zum Elternabschnitt, wenn dessen Nutzen pro Token höher ist als die Summe der Kinder. Jeder verdichtete Block erhält einen **Expand-Handle**.

### 6.4 Platzierung

Konfigurierbar: `RankOrder` (Default, gruppiert nach Quelle) | `EdgesFirst` | `Chronological` | `ByDocument` | `Sandwich`. `EdgesFirst` wird Default erst bei belegtem A/B-Gewinn.

**Cache-freundliche Ordnung:** Stabile Cells (System/Policy/Skills mit unveränderlichen Hashes) zuerst in kanonischer Reihenfolge → identische Präfixe über Aufrufe hinweg. Rückgabe enthält **Prefix-Digest** für KV-Cache-Keys.

### 6.5 Evidence Sufficiency und Abstention

Jedes Ergebnis trägt `evidence_sufficiency ∈ {Sufficient, Thin, Conflicting, None}`. Berechnung: Anzahl unabhängiger `source_root`s, kalibrierte Top-Relevanz, Konfliktstatus. Bei `None` rendert der Packer `NO_EVIDENCE_FOUND` mit Angabe des Suchbereichs.

### 6.6 Citation-Verifier (`VERIFY`)

Deterministischer Verifier + optionaler `GroundingJudge` (NLI):

| Verdikt | Bedingung |
|---|---|
| `Supported` | Handle gültig, Zitate/Zahlen/Daten/Eigennamen in zitierter Spanne |
| `UnanchoredParaphrase` | Handle gültig, keine lexikalischen Anker → Judge oder Hinweis |
| `InvalidHandle` | Token unbekannt oder aus anderer Session |
| `Stale` | Zitierter Chunk im aktuellen Head ersetzt |
| `Contradicted` | Zu den Quellen existiert offener Konflikt |

### 6.7 Explainability

`explain(recall_id)` liefert pro Kandidat: Signalbeiträge (vec/bm25/graph/recency), gewählte Stufe, Gründe für Verwerfen (Budget, Redundanz, Trust, Filter), verwendete Watermarks, Strategie je Segment.

---

## 7. Abgeleitete Daten, Lineage und Work Queue

### 7.1 Derivation Cache

Abgeleitete Facetten unter `(content_hash, producer_id)` **memoisiert**: Embeddings, Token-Zählungen, extraktive/abstraktive Zoom-Stufen, Entitätsextraktionen, Sprachdetektion. **Je Store** (kein Cross-Tenant-Dedup). Forks und Branches teilen den Cache.

### 7.2 Staleness

Jede abgeleitete Facette kennt `source = content_hash`. Ändert sich die Quelle → Facette **per Konstruktion** als veraltet erkennbar. Engine markiert `stale`, nimmt je nach Policy aus Recall heraus oder liefert mit Warnung, stellt Neuberechnung in Work Queue.

### 7.3 Work Queue (nicht-versioniertes Kernel-Sidecar)

Die Engine ruft **kein** Modell auf. Work Queue liegt in `sidecar/work_queue.cxs` (WAL-gesichert, **nicht** im `RootSet` → kein Commit-Spam durch Worker-Leases).

```rust
pub enum Task {
    Embed       { cells: Vec<CellId>, producer: ProducerId },
    Summarize   { cluster: Vec<CellId>, target_level: u8, max_tokens: u32 },
    Consolidate { episode: SeqId, into: NsId },
    Dedupe      { candidates: Vec<(CellId, CellId)> },
    Reconcile   { conflict: ConflictId },
    ExtractEntities { cells: Vec<CellId> },
    CheckContradiction { claim_a: CellId, claim_b: CellId },
}
```

**Leasing-Modell:** At-least-once-Delivery mit TTL-basiertem Lease. Worker-Ergebnisse werden als reguläre Commits eingespielt (vollständig nachvollziehbar, `derived_by = (model, prompt_hash)`). Idempotenz-Keys stellen sicher, dass doppelte Completions harmlos sind.

### 7.4 Retention und Vergessen (kein stilles Löschen!)

**Retire:** `retire(gid)` ist ein **Tombstone-Commit** (sofort unsichtbar ab diesem Snapshot; in Zeitreise innerhalb Retention sichtbar).

**Retention:** Je Namespace `history_days` (Default 30), `branches_days` (Default 1). Nach Ablauf entfernt Compaction tote Zeilen physisch. **Kein Nachweis, keine Signatur.**

**Purge (Admin, Best-Effort):** Tombstone + Retention-Override + Compaction-Auftrag. Blockierende Pins/Tags werden gemeldet. Derivate (Embeddings, Summaries) werden per Referenzzählung entfernt, wenn kein lebender Record den Hash referenziert.

**Zoom-Down:** L0 (Volltext) archivieren, L1/L2 behalten. Dreistufig: Hot → Warm → Tombstone.

**Salience-Score:** `f(Recency, Zugriffshäufigkeit, Graph-Zentralität, Expand-Rate, explicit Importance)`. Steuert Ranking-Prior und Archivierungsempfehlungen, **löscht niemals selbst**.

### 7.5 Konsolidierung (via Work Queue)

1. **Episode → Fakten:** Extrahiert aus Konversationsturns persistente Fakten/Erkenntnisse.
2. **Duplikaterkennung:** Embedding-Ähnlichkeit → `Dedupe`-Tasks.
3. **Widerspruchserkennung:** Embedding-Distanz + Attribut-Überlappung → `Reconcile`-Tasks (3-stufig: deterministisch → Kandidaten-ANN → semantisch via Judge).
4. **Profil-Aggregation:** Einzelfakten → Entity-Profile.
5. **Community-Summaries:** Leiden-Clustering → hierarchische Zusammenfassungen.

---

## 8. Sicherheit, Trust und Zugriffskontrolle

### 8.1 Trust-/Taint-Labels

```
TrustTier:  Verified > Curated > ToolVerified > AgentInferred > Untrusted/Hypothesis
TaintFlags: EXTERNAL | INJECTION_SUSPECT | UNVERIFIED_TOOL | PII   (Flags, kombinierbar)
```

- **TrustTier-Vererbung:** Abgeleitete Records erhalten das **Minimum** ihrer Quellen.
- **TaintFlags-Vererbung:** **Vereinigung** (eine Zusammenfassung einer Webseite bleibt `Untrusted`).
- **Hochstufung** nur durch Gate G7 oder privilegierten Prinzipal (Commit-Eintrag).
- **Injection-Scan:** Phrasenfilter (EN/DE), Struktur-Heuristiken, Unicode-Steuerzeichen, Base64 bis Tiefe 2. Policy: `Quarantine | FlagOnly | Escalate`.
- **Datenrahmen:** Getaintete Blöcke im Pack werden mit markierten Datenbegrenzern und Hinweis „Daten, keine Anweisungen" gerendert.

### 8.2 Zugriffskontrolle

- **Prinzipale und Gruppen** im Katalog. Jeder Record trägt `acl: AclRef`.
- **Bitmap-Auswertung:** Je Gruppe eine Roaring64 über `Gid`; Prinzipal-Filter = Vereinigung seiner Gruppen-Bitmaps. ACL-Filter wirkt **vor/im** Ranking (INV-5).
- **Capabilities (Macaroon-artig):** Attenuierbare Tokens mit Caveats (Namespaces, Branches, Operationen, Trust-Obergrenze, TTL, Token-Budget). Fail-closed: fehlend/ungültig → keine Daten.
- **Egress-Guard:** Vor jeder Übermittlung an Cloud-Ziele: Policy-Check (`LocalOnly | CloudAllowed{...}`), Taint/Trust-Regeln, Muster für Geheimnisse, Rate-Limits. Typestate `GuardedPayload<Unsanitized → Sanitized>`.

### 8.3 Verschlüsselung (At-Rest)

- **AEAD:** XChaCha20-Poly1305 oder AES-256-GCM auf Pack-/Segmentebene.
- **KEK/DEK-Hierarchie:** Master-Key → Repo-KEK → Pack-DEK. O(1)-KEK-Rotation via DEK-Rewrap; lazy Pack-Rotation bei Compaction.
- **Keyed Content-Hashing:** `BLAKE3::keyed_hash(repo_hash_key, plaintext)` — nie über Ciphertext! Schützt vor Cross-Repo-Orakeln ohne Deduplizierung zu brechen.
- KMS-Hook für Schlüsselverwaltung.

### 8.4 Audit und Integrität

- **Commit-Kette:** Hash-verketteter DAG → manipulationsevident. Optionale Ed25519-Signaturen (Pflicht bei `Strict`-Durability).
- **Reflog:** Bewahrt verworfene Sessions (Standard 30 Tage). Hash-verkettet, Manipulationserkennung.
- **CRC je Block**, BLAKE3-Verifikation als Hintergrund-`scrub`.
- **Action-Log** (`alog`): Je Schritt Tool, Argument-Hash, Ergebnis-Hash, Prompt-Hash. Sessions sind prüfbar und wiederholbar.

---

## 9. API-Oberflächen

### 9.1 Rust-API (eingebettet, Referenz)

```rust
let db = Contextra::open(Config::at("/var/lib/contextra"))?;
let main = db.branch("main")?;
let session = main.session("agent/a1/s-42", SessionConfig { ttl: hours(2), ..default() })?;

let mut tx = session.begin(
    Tx::new()
        .intent("Fehlerursache als Fakt ablegen")
        .idempotency_key("run-42/step-7/put-fact-1")
        .durability(Durability::Durable),
);
tx.put(Cell::fact("Registry-Token war abgelaufen")
        .trust(Trust::ToolVerified)
        .edge(Edge::derived_from(log_cell)))?;
let commit = tx.commit().await?;

let ctx = session.recall(Recall::new("Registry Token")
    .budget(Tokens(3_000))
    .signals(Signals::hybrid().recency_half_life(days(30)))
    .min_trust(Trust::ToolVerified)
).await?;

let merge = session.propose(MergePolicy::from_namespace_defaults()).await?;
match merge {
    MergeResult::Merged(c) => { /* Gates bestanden, auf main */ }
    MergeResult::GateRejected(violations) => { /* Agent korrigiert */ }
    MergeResult::Conflicts(list) => { /* ConflictCells auflösen */ }
}
```

### 9.2 MCP-Server (Model Context Protocol)

Wenige, mächtige Tools mit deklarativer Klassifikation (Lesen/Schreiben/Steuerung). Schreibwerkzeuge **standardmäßig gesperrt**:

| Werkzeug | Klasse | Zweck |
|---|---|---|
| `ctx_recall` | Lesen | Hybride Suche + Packer → EvidencePack (Handles, Warnungen, Sufficiency) |
| `ctx_expand` | Lesen | Handle → feinere/gröbere Stufe nachladen |
| `ctx_neighbors` | Lesen | Graph-Nachbarschaft eines Handles |
| `ctx_history` / `ctx_diff` | Lesen | Versionsverlauf, Änderungszusammenfassung |
| `ctx_explain` | Lesen | Plan und Scores einer früheren Abfrage |
| `ctx_describe` | Lesen | Selbstbeschreibung: Namespaces, Schemata, Limits (budgetiert) |
| `ctx_verify` | Lesen | Claims gegen Citation-Tokens prüfen |
| `ctx_remember` | Schreiben | Record/Fakt mit Provenienz in Session-Branch (idempotent) |
| `ctx_relate` | Schreiben | Kante/Fakt anlegen |
| `ctx_retire` | Schreiben | Record ausblenden (Tombstone; Begründung Pflicht) |
| `ctx_session` | Steuerung | `begin`, `savepoint`, `rollback`, `propose`, `discard` |

### 9.3 LLM-lesbare Fehler

```json
{
  "error": "unknown_handle",
  "code": "E_HANDLE_UNKNOWN",
  "message": "Handle 'c:k7qe-3f' failed verification on branch 'main'.",
  "retryable": false,
  "did_you_mean": ["c:k7qd-3f", "c:k7qb-8m"],
  "remediation": "Use ctx_recall to find the correct handle.",
  "suggested_call": { "tool": "ctx_recall", "args": { "q": "…" } }
}
```

Fehlertexte enthalten nie Inhalte unberechtigter Records (INV-5). Vollständiger Fehlerkatalog: `E_CONFLICT`, `E_SNAPSHOT_EXPIRED`, `E_BEYOND_RETENTION`, `E_BACKPRESSURE` (mit `retry_after_ms`), `E_BUDGET_TOO_SMALL`, `E_VECTOR_INVALID`, `E_FORBIDDEN_NAMESPACE` (neutral), `E_GATE_REJECTED`, `E_IDEMPOTENT_REPLAY`, `E_STORE_FAILED`, `E_CORRUPT_TRUTH`.

### 9.4 Weitere Schnittstellen

- **gRPC + Arrow Flight:** Batch-Ingest und gestreamte Ergebnisse als RecordBatches.
- **SQLite Virtual Tables** (optional, `contextra-sqlite`): Hidden Columns für Zeitreise, Systemtabellen für Branches/Log/Diff/Blame.
- **CLI `ctx`:** `init`, `ingest`, `query`, `branch`, `merge`, `scrub`, `verify`, `backup`, `stats`, `upgrade`.
- **Python-Bindings** (`contextra-py`, PyO3, außerhalb default-members).

---

## 10. Crate-Architektur

### 10.1 Schichtenmodell

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  L6  API-Oberflächen & Server                                                │
│      contextra (Fassade) · contextra-mcp · contextra-server · contextra-cli  │
│      Optional: contextra-sqlite · contextra-py                               │
├──────────────────────────────────────────────────────────────────────────────┤
│  L5  Context Engine, Ingest & Agent-Dienste                                  │
│      contextra-engine · contextra-rank · contextra-ingest · contextra-agent   │
│      contextra-cognition · contextra-security                                │
├──────────────────────────────────────────────────────────────────────────────┤
│  L4  Abgeleitete Indizes (IndexManifest + Watermarks, runtime-frei)          │
│      contextra-vector · contextra-text · contextra-graph                     │
├──────────────────────────────────────────────────────────────────────────────┤
│  L3  Versionierte Wahrheit & MVCC (RootSet, runtime-frei)                    │
│      contextra-mvcc (Prolly-Trees, RootSet, MemDelta, Fold, Branches,        │
│      OCC/SSI, Merge, GC, Sidecars)                                           │
├──────────────────────────────────────────────────────────────────────────────┤
│  L2  Content-addressed Object Store & PAX-Layout (runtime-frei)              │
│      contextra-store (Packs, Extents, Locators, Buffer Pool, Re-Layout)      │
├──────────────────────────────────────────────────────────────────────────────┤
│  L1  Durability & System-I/O (runtime-frei)                                  │
│      contextra-wal · contextra-durable-fs                                    │
├──────────────────────────────────────────────────────────────────────────────┤
│  L0  Fundament & Querschnitt                                                 │
│      contextra-types · contextra-ports · contextra-wire · contextra-simd      │
│      contextra-sys · contextra-crypto                                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 10.2 Soll-Workspace

| Ring | Crate | Rolle |
|---|---|---|
| L0 | `contextra-types` | Newtypes (CellId, Gid, ChunkId, Handle, Hlc, Seq), TrustTier, Taint, Fehlerkatalog |
| L0 | `contextra-ports` | Port-Traits (Vfs, Clock, Rng, Embedder, Reranker, Llm, Judge, Tokenizer, MetricsSink) |
| L0 | `contextra-wire` | Kanonische rkyv-/bytemuck-Codecs, Format-Header (Magic/Version/Flags) |
| L0 | `contextra-simd` | SIMD-Kernel (AVX2/AVX-512/NEON/Scalar: Distanzen, Bitpacking) [unsafe isoliert] |
| L0 | `contextra-sys` | OS-Zugriff (io_uring opt., pread/pwrite, flock, O_DIRECT) [unsafe isoliert] |
| L0 | `contextra-crypto` | Keyed BLAKE3, HKDF, AEAD (KEK/DEK), Ed25519, MAC-Handles, Capabilities |
| L1 | `contextra-durable-fs` | Atomares Write-Sync-Rename-DirSync, FaultVfs-Referenz |
| L1 | `contextra-wal` | WAL (.cxw), Group Commit, Torn-Write-Erkennung, Recovery-Replay |
| L2 | `contextra-store` | Packs (.cxp), PAX-Extents, Object/Cell-Locator, Buffer Pool, Re-Layout, S3-Tier-Port |
| L3 | `contextra-mvcc` | Prolly-Trees, RootSet, IndexManifestRef, MemDelta, Fold, BranchSequencer, Typestate Txn, OCC/SSI, 3-Wege-Merge, SATB-GC, Sidecars |
| L4 | `contextra-vector` | Flat-SIMD, RAM-HNSW, Vamana-Segmente, SQ8/PQ/RaBitQ, Filter A/B/C |
| L4 | `contextra-text` | Analyzer, FST, Block-Max-WAND, BM25F global stats |
| L4 | `contextra-graph` | CSR-Segmente, Hub-Skip-Index, k-Hop, Forward-Push-PPR, Leiden |
| L5 | `contextra-rank` | RRF, kalibrierte Norm-Fusion, Isotonische Kalibration, MMR |
| L5 | `contextra-engine` | Katalog, Query-Planer/-Executor, Zoom-Knapsack/Packer, Sufficiency, Verifier, Derivation Cache, WorkQueue-Dispatcher, Retention/Purge |
| L5 | `contextra-ingest` | Parser (PDF/Office/HTML/Code/JSON), Chunker, Revisions-Diff, Injection-Scan |
| L5 | `contextra-agent` | Session-Branches, Merge-Gates (G1–G7), Action-Log, Quoten |
| L5 | `contextra-cognition` | Konsolidierung, L1-Summary, Entity-Resolution, Contradiction-Jobs |
| L5 | `contextra-security` | AEAD at Rest, ACL/Capabilities, Trust/Taint, Egress-Guard |
| L6 | `contextra` | Einzige öffentliche Rust-Fassade (Feature-Flags) |
| L6 | `contextra-mcp` | MCP-Server (stdio / streamable-HTTP, deklarative Tool-Klassifikation) |
| L6 | `contextra-server` | gRPC + Arrow Flight + HTTP/JSON-Daemon |
| L6 | `contextra-cli` | Admin-CLI (`ctx`) |
| Opt | `contextra-infer-{onnx,candle,ollama}` | Inferenz-Adapter für Ports |
| Opt | `contextra-sandbox` | WASM-Plugins für Merge-Gates (wasmtime, Fuel-Limits) |
| Opt | `contextra-kvcache` | Optionaler Radix-Prefix-/KV-Cache (Phase 5) |
| Opt | `contextra-sqlite` | SQLite Virtual Tables (Feature `sqlite`) |
| Dev | `contextra-testkit` | Referenzmodell, Property-/Golden-/ACL-Tests |
| Dev | `contextra-sim` | Deterministischer Simulator (FaultVfs + ManualClock + Crash-Matrix) |
| Dev | `contextra-bench` | Mikro- & Makro-Benchmarks |

### 10.3 Abhängigkeitsregeln

1. **Strikte Gerichtetheit:** Nur nach unten; CI prüft Zyklenfreiheit und Schichtgrenzen.
2. **Runtime-Freiheit L0–L4:** `tokio` ist in L0–L4 verboten. Storage-Kernel: synchrone APIs + I/O-/Rayon-Pools.
3. **`#![forbid(unsafe_code)]`** in allen Crates außer `contextra-simd`, `contextra-sys`, `contextra-sqlite`, `contextra-infer-*`, `contextra-py` und dokumentierten O_DIRECT/Archive-Helfern.
4. **Kein `unwrap()`/`expect()`/`panic!` in Produktionspfaden** (Clippy-Deny).
5. **Rust Edition 2021, stable ≥ 1.89**, `--locked`, `cargo-deny` + `cargo-vet` Pflicht.

---

## 11. Nebenläufigkeitsmodell

### 11.1 Lese-Pfad

**Lock-arm:** Immutable Strukturen + `ArcSwap<Manifest>` je Branch (wait-free Head-Lookup), epochenbasierte Rückgewinnung (`crossbeam-epoch`). Leser blockieren nie Schreiber. Keine globale Sperre im Lese-Hotpath.

### 11.2 Schreib-Pfad

**Ein Sequencer-Actor pro Branch.** Group Commit bündelt WAL-Writes über Branches. Der Sequencer serialisiert nur Commit-Validierung und Pointer-Swap; Objekt-Serialisierung und WAL-Append sind parallel. MPSC → Single-Writer je Commit-Gruppe.

### 11.3 I/O-Pfad

| Komponente | Ausführung |
|---|---|
| Netzwerk, Jobs, Timer | `tokio` (ab L5) |
| Storage-Kernel (WAL-Writer, Sealer, Compactor) | eigene Threads, synchrone API, Completion-Handles |
| Datei-I/O | Linux: `pread`/`pwrite` (Standard) + opt. `io_uring` (registrierte Puffer, O_DIRECT); andere: Blocking-Pool |
| CPU-lastig | eigener Compute-Pool (rayon), strikt getrennt vom Async-Executor |

### 11.4 Hintergrundjobs

Fold, Compaction, Re-Layout, GC, Work-Queue-Dispatch, Index-Builds laufen mit **Prioritäts- und Bandbreitenbudgets** (Token-Bucket).

### 11.5 Typsystem als Sicherheitsnetz

- **Typestates:** `Txn<ReadOnly|ReadWrite>`, `Snapshot<'db>`, `GuardedPayload<Unsanitized|Sanitized>`, `Segment<Building|Sealed>`.
- **Newtypes:** `Gid`, `CellId`, `ChunkId`, `Handle`, `Hlc`, `Seq`, `Lsn`, `SegmentId`, `NsId`, `BranchId`, `ContentHash<T>`.
- Lock-Hierarchie dokumentiert; `loom`-Tests für kritische Pfade.

---

## 12. Durability, Recovery, Garbage Collection

### 12.1 WAL und Group Commit

- **Ein gemeinsamer Log** für alle Branches → ein `fdatasync` pro Gruppe.
- **Record:** `[len u32 | crc32c u32 | lsn u64 | type u8 | branch u32 | txn u64 | payload]`. Payload ist logisch und idempotent.
- **Group Commit:** ≤ 200 µs oder ≥ 1 MiB. Ack erst nach fdatasync (Klasse `Group`/`Strict`).
- **Torn Writes:** CRC + Länge + monotone LSN; Recovery stoppt am ersten ungültigen Record.
- **Segment-Recycling:** unterhalb `checkpoint_lsn` aus Manifest.

### 12.2 Manifest-Log

Append-only, CRC-geschützt, Hash-verkettet, zeigt auf gültigen Zustand (Segment-Liste, WAL-Seq, IndexManifest). Doppelt gepufferter Superblock `CURRENT`.

### 12.3 Crash-Recovery

```
1. Superblock laden → Manifest-Tail prüfen (CRC)
2. Refs rekonstruieren → verwaiste Packs ignorieren
3. WAL ab checkpoint_lsn je Branch in Memtables wiederholen
4. Hashes/CRCs prüfen, torn tail verwerfen
5. Fehlende abgeleitete Indizes im Hintergrund neu bauen (Flat-Fallback)
```

**Ziel:** Recovery < 10 s bei einigen hundert MB WAL.

**Durability-Invarianten:**
- **INV-D1:** Ein Ref/Index-Eintrag wird nie geschrieben, bevor das Pack-Objekt durabel ist.
- **INV-D2:** Pack-Inhalt ist Quelle der Wahrheit; Index ist rekonstruierbar.
- **INV-D3:** Fehler bei `fsync` → Store wechselt in `ReadOnly`/`Failed` (Fail-Stop).
- **INV-D4:** Commits sind unveränderlich.

### 12.4 Garbage Collection (SATB + Remark)

1. **Wurzeln:** alle Refs, Reflog-Fenster, Tags, aktive Snapshots (Epochen-Register), aktive Sessions, Backup-Leases.
2. **Mark** ab Kernel-Snapshot, Traversal Commit → Manifest → Bäume → Chunks. Ergebnis als Reachability-Bitmap.
3. **Write-Barrier:** Während Mark entstandene Chunks gelten als jung, sind ausgenommen.
4. **Remark:** Ref-/Sitzungsänderungen seit Mark-Start nachmarkieren.
5. **Sweep:** Packs mit < 50 % lebenden Bytes → Copy-Forward. Vollständig tote Packs → löschen. Grace-Periode.

### 12.5 Tiering

| Tier | Inhalt | Medium |
|---|---|---|
| T0 | MemDelta, Locator-Hot-Set, Head-Page-Cache, Alias-Tabellen | RAM |
| T1 | Packs mit Hot Heads + aktuelle Cold Tails | NVMe |
| T2 | Kalte Tails, Archiv-Segmente, alte Versionen | Objektspeicher (S3) |

Entscheidung über Kosten-Nutzen-Score (Zugriffsfrequenz × Latenzvorteil ÷ Platzkosten).

---

## 13. Ingest-Pipeline

### 13.1 Überblick

```
Quelldokument ──► Format-Erkennung ──► Parser ──► Chunker ──► Hash-Abgleich ──► Cell-Erzeugung ──► Commit
                                                       │                            │
                                               Work Queue (Embedding, Summary)   Derivation Cache (Hit?)
```

### 13.2 Unterstützte Formate

| Kategorie | Formate |
|---|---|
| Text | Markdown, Plain Text, RST, AsciiDoc |
| Office | PDF, DOCX, XLSX, PPTX, ODT |
| Code | Rust, Python, TypeScript, Go, C/C++, Java (tree-sitter) |
| Daten | JSON, JSONL, CSV, TSV, YAML, TOML, XML |
| Web | HTML, EPUB |
| E-Mail | EML (mail-parser) |
| Binär | FastCDC für generische Binärdaten |

### 13.3 Strukturbewusstes Chunking

Zielgröße in **Token** (Default 512, min 50, max 1024), respektiert strukturelle Grenzen:

| Chunker-ID | Ziel | Grenzen | `boundary_safe` |
|---|---|---|---|
| `markdown-v2` | Prosa/Markdown | Überschriften > Absätze > Sätze | ja |
| `code-treesitter` | Quellcode | Funktion/Klasse/Modul (AST), Signatur als L2 | ja |
| `json-path` / `table-row` | JSON/CSV/XLSX | Objekte / Zeilengruppen inkl. Header | ja |
| `html-semantic` / `pdf-layout` | HTML / PDF | Semantische Blöcke, Spaltenbereinigung | ja |
| `binary-fastcdc` | Binärdaten | Content-Defined Chunking (8 KiB Ziel) | nein |
| `passthrough` | Atomare Einheiten | Keine Zerlegung | ja |

### 13.4 Revisions-Diff

Dokument-Revisionen werden auf Chunk-Ebene per ContentHash abgeglichen: Unveränderte Chunks behalten `Gid`, `content_hash`, Embeddings und Zusammenfassungen. Nur geänderte/neue Chunks durchlaufen die Pipeline. `Supersedes`-Kanten verbinden alte und neue Versionen.

### 13.5 Automatische Facetten-Erzeugung bei Ingest

- **Extraktive Zoom-L2** (Titel, Überschrift, erster Satz) — synchron, deterministisch.
- **Token-Zählungen** für alle registrierten Tokenizer.
- **Embedding-/Summary-Tasks** in Work Queue (oder Derivation-Cache-Hit).
- **Kantenerzeugung:** `Follows`, `PartOf`, `Contains` automatisch.
- **Injection-Scan** über Egress-Guard Hooks.

---

## 14. Adaptive Ranking und Fusion

### 14.1 Score-Fusion

Standard: **Reciprocal Rank Fusion (RRF)**, `score(d) = Σ_i w_i / (k + rank_i(d))`, k = 60.

Optional: **NORM-Fusion** mit geklemmter Min-Max-Normalisierung, kalibriert per isotonischer Regression.

**Nachfusions-Faktoren (multiplikativ, konfigurierbar):**
- **Freshness:** `max(floor, 2^(-Δt / T½))`.
- **Trust:** Faktor aus unterer Konfidenzgrenze (10-%-LCB der Beta-Verteilung).
- **Gültigkeit:** Filter `valid_at`.

### 14.2 Recency-Scoring

```
recency_score = max(floor, exp(-λ · age_days))
λ = ln(2) / half_life_days
```

Decay wirkt **nur auf Ranking**, löscht nie. `IGNORE DECAY` und `AS OF` jederzeit möglich.

### 14.3 Re-Ranking

Optionaler Cross-Encoder über `Reranker`-Port, **deadline-gesteuert**. Überschreitet er die Frist → Ergebnis ohne Rerank, mit Flag.

### 14.4 Drift-Detection (optional, ab Phase 3)

- **Embedding-Drift:** Verschiebung der Query-Embedding-Verteilung.
- **Relevanz-Drift:** sinkende Expand-Raten.
- **Automatische Anpassung:** Fusion-Gewichte und Recency-Parameter adaptiv, basierend auf Access-Trace.

---

## 15. KV-Cache-Store (optional)

*(Bewusst vom Kern getrennt, Phase 5, da modellabhängig und hardwarenah.)*

- **Schlüssel:** `(model_id, tokenizer_id, hash-Kette über Token-Blöcke)` — Merkle-artig, Radix-Struktur.
- **Blockgröße:** 16–256 Token, an Inferenz-Server angepasst.
- **Speicherung:** eigene Pack-Art auf NVMe; Tiering RAM ↔ NVMe ↔ Objektspeicher.
- **Eviction:** kostenbasiert (`Wiederberechnungskosten × Nutzungsfrequenz`), nicht reines LRU.
- **Anbindung:** Contextra liefert Prefix-Digest (Cache-freundliche Ordnung) und verwaltet Blöcke.
- **Invalidierung:** automatisch über Hash-Kette.

---

## 16. Ressourcenmodell und Speicherbudgets

### 16.1 Globales `MemoryBudget` (INV-10)

| Klasse | Inhalt | Default-Anteil |
|---|---|---|
| `fixed` | Manifeste, FST, Skip-Daten, ANN-Codes, CSR-Offsets, Bitmaps, Katalog | 25 % |
| `memtable` | Memtables aller Branches (inkl. Mini-HNSW) | 15 % |
| `block_cache` | Truth-/Index-Blöcke, getrennte Teilbudgets je Klasse | 40 % |
| `query` | Arbeitsspeicher aktiver Abfragen, Pack, Rerank | 10 % |
| `jobs` | Compaction, Index-Builds, Ingest-Puffer | 10 % |

### 16.2 Backpressure

`PressureLevel ∈ {Normal, Elevated, Critical}`, berechnet aus Memtable-Füllstand, WAL-Warteschlange, Compaction-Rückstand, Embedding-Queue.

| Stufe | Wirkung |
|---|---|
| `Elevated` | Ingest-Drosselung, früherer Seal, niedrigprioritäre Jobs pausieren |
| `Critical` | Schreibtransaktionen warten oder `E_BACKPRESSURE`; nur Flush/Compaction/Lesen |

### 16.3 Limits (konfigurierbar)

| Limit | Default |
|---|---|
| Chunk-Text | ≤ 256 KiB |
| Props je Record | ≤ 128 Felder, ≤ 32 KiB |
| Ops je Transaktion | ≤ 100 000; ≤ 64 MiB |
| Kanten je Knoten (aktiv) | Warnung ab 10⁴, hartes Limit 10⁶ |
| Aktive Snapshots | ≤ 4 096; Lease 300 s |
| Branches je Store | ≤ 100 000 |
| Query-k | ≤ 1 000 |
| Namespaces | ≤ 4 096 |

---

## 17. Performance-Ziele und KPIs

### 17.1 Design-Ziele (Hypothesen, zu validieren)

| Metrik | Ziel | Messbedingung |
|---|---|---|
| Fork | < 100 µs, O(1) | beliebige Größe |
| Commit p99 | < 5 ms | NVMe, Group Commit, `durable`, ≤ 100 Cells |
| Hybrid Recall p99 | < 30 ms warm, < 80 ms kalt | 10 Mio. Cells, 1024d, Profil `standard` |
| Batch-Ingest | ≥ 30 000 Cells/s je Branch | ohne Embedding-Berechnung |
| `window()` | < 1 ms | 10⁶ Turns, ohne Inhaltszugriff |
| Schreibamplifikation (Fold) | < 5× | Batchgröße ≥ 1 000 |
| Recovery | < 10 s | einige hundert MB WAL |
| Recall@10 Vektor-Leg | ≥ 0,95 | gegen exakte Suche |
| Speicherbedarf/Chunk (Index) | ≤ 12 % der Rohvektorgröße | PQ/SQ8/RaBitQ + Graph |

Öffentliche Zahlen MÜSSEN Messbedingungen nennen und ausweisen, ob Embedding-Inferenz enthalten ist.

### 17.2 Qualitätsmetriken (entscheidend)

- **Utility@Budget:** Anteil relevanter Cells bei gegebenem Token-Budget (> Top-k-Baseline).
- **Nutzen-pro-Token:** Anteil geladener Token, die der Agent tatsächlich verwendet/zitiert.
- **Task-Erfolgsrate** bei gleichem Budget.
- **Expand-Rate:** wie oft der Agent nachlädt (niedriger = bessere Erstauswahl).
- **Abstention-Rate:** Anteil korrekt nicht beantworteter Fragen bei `None`.
- **Grounding-Rate:** Anteil belegter Antworten.
- **Konfliktsichtbarkeit:** Rate korrekt aufgezeigter Widersprüche.

---

## 18. Teststrategie

### 18.1 Deterministische Simulation

**FoundationDB/TigerBeetle-Stil:** `contextra-sim` mit `FaultVfs` (torn writes, verlorene fsyncs, Bit-Flips, volle Platte, ENOSPC), `ManualClock`, `SeededRng`. Seeds reproduzierbar. Referenzmodell (einfacher In-Memory-Store) als Orakel. INV-1…INV-12 als Prüfprädikate. ≥ 10⁷ Simulationsläufe in CI-Nightly.

### 18.2 Testebenen

| Ebene | Werkzeug | Gegenstand |
|---|---|---|
| Property-Tests | `proptest` | Merge-Tabelle, Knapsack-Optimalität, RRF, Bitmap-Übersetzung, Tokenzählung, History-Independence |
| Nebenläufigkeit | `loom` | Manifest-Publikation, Arena, Cache, Group Commit, Leases |
| Speichersicherheit | Miri + `cargo-fuzz` | `unsafe`-Inseln, alle Decoder/Parser |
| Crash-Matrix | `FaultVfs` | I/O-Schritt-für-Schritt-Abbruch → Zustand = Präfix bestätigter Commits |
| GC-Simulation | deterministisch | Schreiber + Dedup + GC + Abstürze → kein erreichbares Objekt verloren |
| Differenziell | Referenz | ANN vs. exakt, BM25 vs. Tantivy-Scores, PPR vs. Dense-Power-Iteration |
| End-to-End | Benchmark-Harness | nDCG@10, Recall@k, MRR; Kontext-Qualität via LLM |
| Sicherheit | ACL-Leckage-Suite | Scores/Zähler/Fehlertexte leaken nie unberechtigte Existenz |
| Agenten | Golden-Transkripte | Konflikt, Retry, Zeitreise, halluzinierte Handles |
| Last/Soak/Chaos | Langläufer | 72 h ohne Speicherwachstum jenseits Budget |
| Supply Chain | `cargo deny` + `cargo vet` | Lizenzen, Advisories |

### 18.3 Abnahmekriterien für 1.0

1. Alle INV-1…INV-12 durch Simulation abgedeckt; ≥ 10⁷ Läufe ohne Verletzung.
2. Zielwerte aus 17.1 erreicht oder mit begründetem Registereintrag.
3. Alle Dateiformate haben Fuzz-Targets mit ≥ 24 h Laufzeit ohne Befund.
4. Index-Verlust-Test: Löschen aller `index/` → vollständige Wiederherstellung via Flat-Fallback + Rebuild.
5. Restore aus Backup auf neuer Maschine.
6. ACL-Leckagetest-Suite grün.
7. MCP-Konformität: jedes Werkzeug klassifiziert, Schreibwerkzeuge gesperrt, Fehler LLM-lesbar.
8. Dokumentation: Betriebshandbuch, Formatspezifikation, API-Referenz.

---

## 19. Roadmap

| Phase | Inhalt | Austrittskriterium |
|---|---|---|
| **P0 — Fundament & Messung** | Repo-Aufräumen, Benchmark-Harness, deterministischer Simulator, Datensätze, Baselines | Simulator führt WAL-Crash-Zyklen aus; Baselines vorhanden |
| **P1 — Storage-Kernel** | `wal`, `store`, `mvcc`: WAL + Group Commit, Memtable, Truth-Segmente, Manifest, Liveness, Compaction, Recovery, Block-Cache, opt. AEAD | Crash-Sim ≥ 10⁶ Punkte ohne Prefix-Verletzung; Commit-p99 < 5 ms |
| **P2 — Index-Segmente** | Flat/HNSW/Vamana, FTS-Segmente, CSR, Watermarks, Bitmaps, Gid-Übersetzung | Recall@10 ≥ 0,95; Index-Verlust-Test grün |
| **P3 — Engine & Query** | Katalog, Namespaces, Ingest, Planner/Executor, Fusion, Packer (Knapsack/MMR), Explain, Derivation Cache | Utility@Budget > Top-k-Baseline; Budget-Garantie 100 % |
| **P4 — Branches & Agenten** | Session-Branches, Merge/Diff, Gates G1–G7, Idempotenz, Sessions, MCP, Citation-Verifier | Property: Merge kommutiert; Branch-Isolation; MCP grün |
| **P5 — Betrieb & Optionals** | Backup/Restore, `scrub`/`verify`, Metriken, Server, Inferenz-Adapter, Egress-Guard, opt. KV-Cache, Affinity Re-Layout | 72-h-Soak; Egress-Guard Fail-closed |
| **P6 — Härtung** | Fuzzing, Miri/Loom, Last-/Chaos-Tests, Dokumentation, Formatfreeze 1.0 | Alle Abnahmekriterien (18.3) erfüllt |
| **v2 (nicht v1)** | Raft-Replikation, Objekt-Storage-Tier für kalte Packs, Read-Replicas | — |

**Team-Skizze:** 3–4 Storage-Ingenieure (Rust), 1 Retrieval/ML-Ingenieur, 1 Test-/Simulations-Ingenieur; frühes Design-Partner-Team.

---

## 20. Risiken und Entscheidungsregister

### 20.1 Risiken

| # | Risiko | Mitigation | Frühindikator |
|---|---|---|---|
| R-01 | **Write-Amplification** (Prolly-Tree, ANN-Compaction) | Batch-Fold, Segment-Streaming-Merge, Compaction-Drosselung | Write-Amp > 15 im Bulk-Test |
| R-02 | **Segment-Fan-out** erhöht Query-Latenz | Fan-out-Budget ≤ 24, Compaction-Priorität, geteilte Top-k-Schwelle | Segmentzahl je Sicht |
| R-03 | **Gid→DocId-Übersetzung** bei großen Filtermengen | Sortierte Spalte + Range-Slicing, Cache je Query/Segment | Übersetzungszeit > 20 % der Query |
| R-04 | **Flat-Fallback** bei großem Index-Rückstand | Watermark-Metrik, Index-Build-Priorität, `lane_coverage`-Warnung | Rückstand > N Segmente |
| R-05 | **Embedding-Modellwechsel** | Space-Konzept, Doppelbetrieb, Priorisierung nach Salience | Migrationsdauer-Schätzung |
| R-06 | **Tokenizer-Kopplung** | Zählungen pro Familie, lazy Neuberechnung, `boundary_safe` | — |
| R-07 | **Semantische Merge-Konflikte** | ConflictCell + Gates G1/G4 + externe Auflösung | Konfliktrate |
| R-08 | **GC bei vielen Branches** | TTL, generationelles GC, Branch-Quoten | Speicher durch Pins |
| R-09 | **Format-Lock-in** | Formatfreeze in P6; Feature-Flags; Migrationswerkzeug | — |
| R-10 | **Prompt-Injection** | Trust/Taint, Datenrahmen, Egress-Guard, Quarantäne | Injection-Korpus-Rate |
| R-11 | **Hub-Knoten im Graph** | Degree-Cap, Hub-Skip-Index, Hub-Dämpfung in PPR | Gradhistogramm |
| R-12 | **Windows-Persistenz** | `DurableFs`-Testsuite pro Plattform | Plattformtest rot |
| R-13 | **Scope-Creep** | Phasen strikt; neue Features nur mit Benchmark | — |

### 20.2 Entscheidungsregister (konsolidiert)

| ID | Entscheidung | Begründung |
|---|---|---|
| C-01 | **Prolly-Trees + MemDelta** für Truth-Layer | History-Unabhängigkeit, O(Δ·log n) Diff/Merge, strukturelles Sharing |
| C-02 | **RootSet (Truth) vs. IndexManifest (Indizes)** | Determinismus des Commits; Indizes mit Watermarks entkoppelt |
| C-03 | **Entkoppelte Cell (CellContent vs. CellMeta) & CellLocator** | Tags/valid_to ändern nicht content_hash; kein Re-Embedding |
| C-04 | **Ephemerer Zustand in Kernel-Sidecars** | Work Queue und Idempotenz außerhalb RootSet; kein Commit-Spam |
| C-05 | **MAC-geschützte Handles `c:<base32>-<mac2>`** | 10 Bit MAC schließt Halluzinationen zu 99,9 % aus |
| C-06 | **Dichtes Gid(u64) + DocId(u32)** | Gemeinsamer Bitmap-Raum für alle Retrieval-Modalitäten |
| C-07 | **Keyed CAS-Hash über Klartext** | Kein Cross-Repo-Orakel; Deduplizierung bleibt funktional |
| C-08 | **AEAD mit KEK/DEK-Hierarchie** | O(1)-KEK-Rotation; lazy Pack-Rotation bei Compaction |
| C-09 | **Merge-Gates G1–G7** | `main` bleibt frei von ungelösten Konflikten |
| C-10 | **BeliefBeta + source_root-Unabhängigkeit** | Verhindert Echokammern; 10-%-LCB; noisy-AND |
| C-11 | **TrustTier (min) + TaintFlags (union) + Datenrahmen** | Wirksame Injection-Abwehr |
| C-12 | **Ein Commit-Punkt (WAL Group Commit)** | Beseitigt Split-Brain; kein 4-Store-2PC |
| C-13 | **Zoom-Knapsack + Submodularer Greedy + Small-to-Big + Abstention** | Überblick & Tiefe im Budget; NO_EVIDENCE_FOUND |
| C-14 | **Ein Standardalgorithmus je Modalität** | HNSW/Vamana, BM25F+BMW, CSR+Forward-Push-PPR+Leiden, RRF(k=60) |
| C-15 | **Keine Löschkette; retire + Retention + purge** | Decay beeinflusst nur Ranking, löscht nie still |
| C-16 | **Portables I/O + opt. io_uring; kein mmap** | Vermeidet unkontrollierte Eviction, SIGBUS |
| C-17 | **Eigener FTS statt Tantivy** | Block-Max-WAND auf segmentlokalen DocIds + gemeinsame Bitmaps |
| C-18 | **WASM-Merge/Gates mit Fuel-Limits** | Deterministische Sandbox; Fallback auf ConflictCell |
| C-19 | **Eventual-consistent AccessStats; Re-Layout opt-in** | Hot-Head/Cold-Tail als Basis; Affinity optimiert Cold-Tail-I/O |
| C-20 | **Typestate Txn<RO|RW> & Snapshot<'db>** | Schließt unregistrierte SSI-Reads konstruktiv aus |
| C-21 | **Embedded-first, Single-Process, Server als Hülle** | Minimale Latenz; Daemon für Multi-Client |

### 20.3 Verworfene Konzepte

| ID | Konzept | Grund |
|---|---|---|
| R-01 | Automatische Cosine-Schwelle für Merge-Konflikte | Stille Wahrheitsentscheidung; Konflikte müssen sichtbar bleiben |
| R-02 | mmap im Produktpfad | Unkontrollierte Eviction, SIGBUS, keine Prefetch-Kontrolle |
| R-03 | Multiplikative Pfadkonfidenz `Π c · γ^(k-1)` | Unterschätzt Ketten; ignoriert Korroboration. Ersetzt durch Min-Bound |
| R-04 | Unbegrenzte Branches ohne Gate-Pflicht | Verletzt Prinzip 5 |
| R-05 | Vektoren als CRDT-Menge | Embeddings folgen dem Inhalt |
| R-06 | Feste U-Form-Platzierung als Default | Wirkung nicht gesichert; nur als evaluierte Option |
| R-07 | Klartext-Hashes über Mandanten | Information Leakage |
| R-08 | Globale Lock-free-Versprechen | Nicht belegbar; präzise Garantien stattdessen |
| R-09 | Globale Deduplizierung über Mandanten | Datenschutzrisiko |
| R-10 | Exakter 0/1-Knapsack mit DP | Zu teuer bei 500 Kandidaten; Greedy mit Garantie |
| R-11 | Arrow als primäres Persistenzformat | Key-Value braucht optimierte Layouts; Arrow nur für Batches/Export |

---

## 21. Glossar

| Begriff | Bedeutung |
|---|---|
| **Cell / CellContent / CellMeta** | Entkoppelte Wissenseinheit: unveränderlicher Inhalt, versionierte Metadaten, Graph-Kanten getrennt |
| **CellId / Gid / DocId** | Logische ID, global dichter u64-Index, segmentlokale u32-Position |
| **CellLocator / ObjectLocator** | `(PackId, ExtentNo, RowIndex)` pro Cell vs. `(PackId, Offset, Len)` für Prolly/Blobs |
| **ChunkId (ContentHash)** | Keyed/plain BLAKE3-256 über kanonische Klartext-Bytes |
| **Handle (MAC)** | Token-sparsamer, MAC-geschützter Alias `c:<base32>-<mac2>` |
| **RootSet** | Deterministische Prolly-Wurzeln (primary, sequences, graph, aliases, policies, conflicts, provenance) |
| **IndexManifest** | Separater Zeiger auf physische Index-Segmente mit Watermarks (nicht im RootSet-Hash!) |
| **Prolly-Tree** | History-unabhängiger, inhaltsadressierter B-Baum |
| **MemDelta** | Branch-lokale In-Memory-Skip-List vor asynchronem Prolly-Fold |
| **Extent (PAX)** | 256 KiB–1 MiB I/O-Einheit mit spaltenorientiertem Hot-Head und Cold-Tail |
| **Derivation Cache** | Pro `(content_hash, producer_id)` memoisierte Embeddings, Summaries, Token-Counts |
| **Zoom-Knapsack** | Multiple-Choice-Knapsack + Submodularitäts-Abzug + Small-to-Big unter Token-Budget |
| **BeliefBeta** | Evidenzbasierte Konfidenz `Beta(α, β)` mit `source_root`-Deduplizierung und 10-%-LCB |
| **EvidenceSufficiency** | `Sufficient | Thin | Conflicting | None` mit Abstention-Rendering |
| **TrustTier & TaintFlags** | Vertrauensstufen (min-vererbt) + orthogonale Kontaminations-Flags (union-vererbt) |
| **Merge-Gates (G1–G7)** | Deterministische Qualitäts-/Sicherheitsprüfungen vor Promotion nach `main` |
| **ConflictCell** | Versionierte Darstellung eines Konflikts (niemals ungelöst auf `main`) |
| **GroundingLevel** | Belegstufe: G0 (verifizierte Spanne) bis G3 (Hypothesis/Quarantäne) |
| **Work Queue (Sidecar)** | WAL-gesicherte, nicht-versionierte Job-Tabelle für externe Worker |
| **HLC** | Hybrid Logical Clock (48b ms + 16b logisch) |
| **Seq** | Globaler monotoner Commit-Zähler (Transaktionszeit-Achse) |
| **Watermark-Fallback** | Exakter Scan über Zeilen > watermark_seq bei noch nicht gebauten Indizes |
| **RRF** | Reciprocal Rank Fusion — robuste, skalenfreie Score-Kombination |
| **MMR** | Maximal Marginal Relevance — Diversitätsoptimierung |
| **SATB** | Snapshot-at-the-Beginning — GC-Protokoll mit Write-Barrier |
| **CSR** | Compressed Sparse Row — kompakte Graphadjazenz |
| **FST** | Finite State Transducer — platzspartes Term-Dictionary |
| **Vamana/DiskANN** | Graph-basierter ANN-Index, optimiert für SSD |
| **Access-Trace** | Gesampelter Lesezugriffs-Strom für Layout-Optimierung und Ranking-Feedback |
| **Salience** | Berechnete Wichtigkeit (Recency × Zugriff × Zentralität × Expand-Rate × Importance) |
| **Progressive Disclosure** | Muster: kompakte Handles → selektives Expand nur bei Bedarf |
| **Prefix-Digest** | Hash der stabilen Präfix-Token für KV-Cache-Kompatibilität |
| **Facility Location** | Submodulare Abdeckungsfunktion für informationsoptimale Auswahl unter Budget |

---

## Anhang A: Vergleich mit Alternativen

| Aspekt | RocksDB/LSM | Dolt (Prolly) | LanceDB/Vektor-DB | **Contextra** |
|---|---|---|---|---|
| **Primitive** | Key → Bytes | SQL-Zeilen | Vektor + Payload | **Entkoppelte Cell (Content + Meta) + Zoom** |
| **Versionierung** | Snapshots | Branches/Merge | Dataset-Versionen | **O(1)-Session-Branches, Gates G1–G7, Zeitreise** |
| **Indizes & Konsistenz** | Keine Vektoren/Graph | B-Tree | Vektor/FTS ohne Graph | **RootSet (Truth) + IndexManifest (Watermark + Fallback)** |
| **Filter & Hybrid** | Scan | Join | Post-/Pre-Filter | **Gemeinsames Gid/DocId-Bitmap für Vektor + BM25F + Graph** |
| **LLM-Integration** | Keine | Keine | Top-k Chunks | **Token-Summen, Zoom-Knapsack, MAC-Handles, Abstention, Trust** |
| **Layout** | Key-Order | Key-Order | Spalten | **Hot-Head/Cold-Tail PAX-Extents + Locator Re-Layout** |

---

## Anhang B: Dateiformate (Skizzen)

Allgemeiner Dateikopf: `magic[4] | format_version u16 | compat_flags u16 | incompat_flags u32 | file_kind u8 | file_id u128 | created_hlc u64 | header_crc32c u32`.

Magic je Art: `CWAL` (WAL), `CMLG` (Manifest-Log), `CSEG`/`.cxp` (Truth-Pack), `CIDX` (Index), `CBLB` (Blob), `CSID` (Sidecar).

Blöcke: `len u32 | codec u8 | crc32c u32 | payload`. Footer: Abschnittsverzeichnis, Statistiken, BLAKE3 über Datei. Little-Endian; alle Offsets 8-Byte-aligned; Vektor-/Sektorbereiche 4-KiB-aligned.

WAL-Payload-Typen: `Begin`, `PutContent`, `PutMeta`, `PutEdge`, `Retire`, `PutDerivation`, `CatalogOp`, `BranchOp`, `Commit{idem, hlc, branch, durability}`. Replay idempotent über `applied_lsn`.

---

## Anhang C: Fehlercodes (vollständig)

| Code | Retry | Bedeutung / Remediation |
|---|---|---|
| `E_CONFLICT` | ja | Write-Konflikt; neu begin |
| `E_SNAPSHOT_EXPIRED` | ja | Lease abgelaufen |
| `E_BEYOND_RETENTION` | nein | Zeitreise außerhalb Retention; früheste Seq |
| `E_BACKPRESSURE` | ja | `retry_after_ms` |
| `E_HANDLE_UNKNOWN` | nein | mit `did_you_mean` |
| `E_HANDLE_MAC_INVALID` | nein | MAC-Prüfung fehlgeschlagen; mit `did_you_mean` |
| `E_FORBIDDEN_NAMESPACE` | nein | neutral (keine Existenzauskunft) |
| `E_VECTOR_INVALID` | nein | NaN/∞/Dimension |
| `E_BUDGET_TOO_SMALL` | nein | Mindestbudget angegeben |
| `E_CORRUPT_TRUTH` | nein | Prüfsummenfehler; Restore |
| `E_GATE_REJECTED` | nein | Gate und Grund angegeben |
| `E_IDEMPOTENT_REPLAY` | — | (kein Fehler) gespeichertes Ergebnis |
| `E_IDEMPOTENT_MISMATCH` | nein | gleicher Key, anderer Payload |
| `E_STORE_FAILED` | nein | fatale I/O; Neustart/Recovery |
| `E_QUOTA_EXCEEDED` | nein | Session-Quota erschöpft |
| `E_PROV_MISSING` | nein | Provenienz fehlt (Pflichtfeld) |
| `E_SCHEMA_VIOLATION` | nein | Namespace-Schema verletzt |
| `E_TRUST_INSUFFICIENT` | nein | Trust-Stufe unter min_main_trust |

---

*Ende der Spezifikation. Diese Datei ist der normative Soll-Zustand für Contextra. Änderungen erfolgen ausschließlich über Einträge im Entscheidungsregister (Abschnitt 20.2).*
