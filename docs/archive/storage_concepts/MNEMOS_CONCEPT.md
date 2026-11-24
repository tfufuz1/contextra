# Mnemos — ein LLM-natives, versioniertes Storage-System in Rust

**Konzeptdokument v0.1 · 21.11.2026 · Status: Entwurf zur Architekturdiskussion**
**Autor-Rolle:** Principal Storage-Engine-Ingenieur
**Zielsprache der Implementierung:** Rust (stable, Edition 2024)

> Alle Zahlen zu Performance und Platzbedarf in diesem Dokument sind **Designziele bzw. Hypothesen**, keine Messwerte. Sie müssen in Phase 0/1 durch Prototypen validiert werden (siehe Abschnitt 16).

---

## Inhaltsverzeichnis

0. [Kurzfassung und Kernideen](#0-kurzfassung-und-kernideen)
1. [Problemanalyse: Was ist an LLM-Workloads anders?](#1-problemanalyse-was-ist-an-llm-workloads-anders)
2. [Designprinzipien](#2-designprinzipien)
3. [Architekturüberblick](#3-architekturüberblick)
4. [Datenmodell: Cells, Facets, Zoom](#4-datenmodell-cells-facets-zoom)
5. [Physisches Layout](#5-physisches-layout)
6. [Versionierung und Transaktionen](#6-versionierung-und-transaktionen)
7. [Indizes](#7-indizes)
8. [Context Engine: Recall unter Token-Budget](#8-context-engine-recall-unter-token-budget)
9. [Abgeleitete Daten, Lineage, Konsolidierung](#9-abgeleitete-daten-lineage-konsolidierung)
10. [Sicherheit, Vertrauen, Datenschutz](#10-sicherheit-vertrauen-datenschutz)
11. [API-Oberflächen und Agenten-Ergonomie](#11-api-oberflächen-und-agenten-ergonomie)
12. [Optional: Prefix-/KV-Cache-Store](#12-optional-prefix-kv-cache-store)
13. [Tiering, Garbage Collection, Replikation](#13-tiering-garbage-collection-replikation)
14. [Rust-Implementierung](#14-rust-implementierung)
15. [Vergleich mit RocksDB und Alternativen](#15-vergleich-mit-rocksdb-und-alternativen)
16. [Ziele, KPIs und Evaluationsmethodik](#16-ziele-kpis-und-evaluationsmethodik)
17. [Roadmap](#17-roadmap)
18. [Risiken und offene Fragen](#18-risiken-und-offene-fragen)

---

## 0. Kurzfassung und Kernideen

RocksDB, LMDB, SQLite & Co. speichern **Bytes unter Schlüsseln** und optimieren für Programme, die exakte Lookups und Range-Scans ausführen. Ein LLM-Agent ist ein anderer Client:

- Er liest **nicht alles**, sondern genau so viel, wie in sein **Token-Budget** passt.
- Er sucht **unscharf** (semantisch, lexikalisch, über Beziehungen, über Zeit).
- Er arbeitet **spekulativ**: probiert, verwirft, wiederholt, rollt zurück.
- Er **retryt**, **halluziniert IDs** und kann durch gelesene Inhalte **manipuliert** werden.
- Er braucht denselben Inhalt in **verschiedenen Auflösungen** (Titel → Zusammenfassung → Volltext).

**Mnemos** macht diese Eigenschaften zu Primitiven der Storage-Engine statt sie in jeder Anwendung neu zu bauen.

### Die zehn Kernideen

| # | Idee | Kurzbeschreibung |
|---|------|------------------|
| 1 | **Cell statt Key-Value** | Die Primitive ist eine *Wissenseinheit* mit Facetten (Rohinhalt, Tokens, Embeddings, Zusammenfassungen, Kanten, Zeit, Vertrauen), nicht ein opaker Blob. |
| 2 | **Token-native Speicherung** | Token-Zählungen (und optional Token-IDs) sind persistiert und in Indexstrukturen als Summen eingebettet. „Gib mir 3000 Token rund um Cell X" kostet O(log n) ohne Inhalte zu lesen. |
| 3 | **Zoom-Leiter + Zoom-Knapsack** | Jede Cell hat Auflösungsstufen L0…Ln. Die Engine wählt pro Treffer die Stufe so, dass der Nutzen unter dem Token-Budget maximal wird. |
| 4 | **Ortsunabhängige Identität, gelerntes Layout** | Inhalte sind content-addressed (BLAKE3). Wo sie physisch liegen, entscheidet ein Hintergrundprozess anhand **echter Zugriffsspuren** (Co-Retrieval-Clustering). |
| 5 | **Hot-Head/Cold-Tail-Extents** | Ein 256 KiB–1 MiB-Extent enthält vorne alles, was zum *Ranken* nötig ist, hinten die schweren Inhalte → ein I/O für Ranking, ein weiterer nur für die Gewinner, beide am selben Ort. |
| 6 | **Versionierung *ist* der Transaktionsmechanismus** | O(1)-Forks pro Agentenversuch, Merge mit typisierten Strategien, Konflikte als Daten, bitemporale Zeitreise. |
| 7 | **Derivation Cache + Lineage** | Embeddings, Summaries, Token-Zählungen sind *abgeleitet*, per Inhalts-Hash memoisiert, über Branches geteilt und bei Quelländerung automatisch als veraltet markiert. |
| 8 | **Trust-/Taint-Labels im Storage** | Jede Cell trägt eine Vertrauensstufe; abgeleitete Daten erben das Minimum ihrer Quellen. Grundlage für Prompt-Injection-Abwehr auf Anwendungsebene. |
| 9 | **Crypto-Shredding** | Unveränderliche History und DSGVO-Löschung (Art. 17) werden über Schlüsselvernichtung statt Datenumschreiben vereinbart. |
| 10 | **Agent-sichere Semantik** | Idempotenz-Schlüssel, LLM-lesbare Fehler mit Handlungsvorschlägen, kurze token-sparsame IDs mit „Meintest du …?", Progressive Disclosure per Handle/Expand. |

Zusätzlich (optional, Phase 5): ein **Prefix-/KV-Cache-Store**, der Prompt-Cache-Blöcke persistiert und die Context Engine **cache-freundlich** ordnen lässt.

---

## 1. Problemanalyse: Was ist an LLM-Workloads anders?

| Klassische Annahme (RocksDB, SQL-DBs) | LLM-/Agenten-Realität | Konsequenz für Mnemos |
|---|---|---|
| Der Leser ist ein Programm und verarbeitet beliebig viele Bytes | Der Leser ist ein Modell mit **hartem Token-Budget**; jedes Token kostet Latenz und Geld | Budget ist First-Class-Parameter jeder Leseoperation; Tokenkosten sind pro Item bekannt |
| Lesen = exakter Key oder Range | Lesen = **unscharfe Relevanz** (Semantik + Lexik + Graph + Recency + Vertrauen) | Hybride Indizes im Kern, Score-Fusion in der Engine |
| Ein Datensatz hat genau eine Darstellung | Derselbe Inhalt wird in **mehreren Auflösungen** gebraucht | Zoom-Leiter pro Cell |
| Transaktionen sind kurz und klein | Agenten-Läufe sind **lang, verzweigt, spekulativ** | Branch/Fork als Transaktionsprimitive |
| Clients sind deterministisch und vertrauenswürdig | Clients **retryen**, **erfinden IDs**, **lesen Fremdinhalte**, die sie manipulieren | Idempotenz, ID-Validierung, Trust-Labels |
| Gespeichert ist „die Wahrheit" | Vieles ist **abgeleitet** (Embeddings, Summaries) und veraltet | Lineage, Staleness-Tracking, Derivation Cache |
| Layout folgt der Schlüsselordnung | Zusammen gelesen wird, was **thematisch, zeitlich oder kausal** zusammengehört | Zugriffsgelerntes physisches Layout |
| Löschen = Bytes überschreiben | Unveränderliche History kollidiert mit **Recht auf Löschung** | Crypto-Shredding |
| Zugriffsstatistik ist Betriebsdetail | Zugriffsspuren sind die **beste Quelle für Relevanz-Priors und Layout** | Access-Trace als First-Class-Datenstrom |

### Typische Zugriffsmuster (Arbeitsannahmen)

1. **Recall unter Budget:** „Gib mir die besten ≤ 6 000 Token Kontext zu dieser Anfrage." (häufigste Leseoperation)
2. **Fenster-Lesen:** „Gib mir die Turns 40–60 dieser Episode, höchstens 3 000 Token."
3. **Expand:** Der Agent sieht ein Handle `c:k7qd` in kompakter Form und fordert dessen Volltext an.
4. **Append-lastiges Schreiben:** Episoden, Tool-Ergebnisse, Beobachtungen — kleine Cells, hohe Rate, fast nie Updates in place.
5. **Fork → Arbeiten → Merge/Verwerfen:** pro Plan-Schritt oder Experiment.
6. **Hintergrundarbeit:** Konsolidierung, Re-Embedding bei Modellwechsel, Vergessen/Abwerten.

---

## 2. Designprinzipien

1. **Immutable first.** Inhalte und Indexknoten sind unveränderlich und content-addressed. Mutabilität existiert nur als Zeiger (Refs) und als kleine Seitentabellen (z. B. Zugriffsstatistik).
2. **Identität ≠ Ort.** Logische Adresse (Hash) und physischer Ort sind entkoppelt, damit das Layout jederzeit neu optimiert werden kann.
3. **Ein Commit, ein Wahrheitsraum.** Primärdaten *und alle Indizes* sind Teil desselben versionierten Wurzelobjekts. Es gibt keinen „Index hinkt hinterher"-Zustand für Leser.
4. **Budget statt Limit.** Leseoperationen optimieren Nutzen pro Token, nicht „Top-k".
5. **Abgeleitet ist ersetzbar.** Nur Rohdaten sind Wahrheit; alles andere kann neu berechnet werden und trägt seine Herkunft.
6. **Der Client ist unzuverlässig.** Jede Schreiboperation ist idempotent, jede Antwort verständlich für ein Modell.
7. **Die Engine ruft kein LLM.** Mnemos stellt Arbeit (Work Queue) bereit, externe Worker/Agenten erledigen sie. Das hält die Engine deterministisch, testbar und modellneutral.
8. **Korrektheit vor Geschwindigkeit, dann Geschwindigkeit.** Deterministische Simulationstests, Prüfsummen überall, `unsafe` nur in klar begrenzten Crates.

---

## 3. Architekturüberblick

```
┌──────────────────────────────────────────────────────────────────────────┐
│  L6  API-Oberflächen                                                     │
│      Rust-Embedded-Crate · MCP-Server · gRPC/HTTP · Python/TS-Bindings   │
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

**Leitgedanke:** Unterhalb von L3 ist Mnemos ein *Git-artiger Objektspeicher mit gelerntem Layout*. Oberhalb von L3 ist es eine *kontextbewusste Wissensdatenbank*. Der Übergang ist der **Commit** — ein Hash über das gesamte Wurzelobjekt.

---

## 4. Datenmodell: Cells, Facets, Zoom

### 4.1 Namespace

Ein **Namespace** ist eine Sammlung mit gemeinsamer Policy (vergleichbar mit Collection/Table). Policies sind Teil des Zustands und damit versioniert.

```toml
[namespace."agent/episodic"]
kind              = "episodic"          # episodic | semantic | procedural | working | artifact
chunker           = "markdown-v2"       # strukturbewusst (siehe 4.5)
tokenizers        = ["tok-A", "tok-B"]  # Token-Zählungen werden für diese Familien persistiert
embedders         = ["emb-768-v3"]
zoom_ladder       = ["L0:raw", "L1:summary<=200tok", "L2:title<=24tok"]
default_trust     = "tool-unverified"
retention         = { hot_days = 14, warm_days = 180, zoom_down_after = "180d" }
merge             = { text = "diff3", tags = "union", counters = "sum", default = "manual" }
durability        = "durable"           # volatile | buffered | durable | replicated
encryption_scope  = "per-subject"       # none | per-tenant | per-subject (siehe 10.3)
```

### 4.2 Cell — die zentrale Primitive

```rust
/// Stabile, ortsunabhängige Identität eines Inhaltsblocks (BLAKE3-256).
pub struct ChunkId([u8; 32]);

/// Logische Identität einer Cell, stabil über Versionen hinweg.
pub struct CellId {
    pub ns: NsId,
    pub key: CellKey, // UUIDv7/ULID (zeitlich sortierbar) oder anwendungsdefiniert
}

/// Kanonischer, content-addressed Teil einer Cell.
pub struct Cell {
    pub id: CellId,
    pub kind: CellKind,                // Episode, Fact, Doc, ToolResult, Plan, Skill, Summary, …
    pub raw: ChunkRef,                 // kanonischer Inhalt (UTF-8 / JSON / Binary)
    pub attrs: Attrs,                  // typisierte Attribute (Tags, Zahlen, Zeiten, Enums)
    pub edges: SmallVec<[Edge; 4]>,    // typisierte Kanten (cites, follows, supersedes, derived_from…)
    pub time: BiTemporal,              // Transaktionszeit + Gültigkeitszeit (siehe 6.6)
    pub trust: TrustLabel,             // siehe 10.1
    pub prov: Provenance,              // Autor/Agent/Run/Step, Quelle, ggf. Modell+Prompt-Hash
    pub derived: DerivedRefs,          // Verweise auf abgeleitete Facetten (nicht Teil des Inhalts-Hash)
}

/// Abgeleitete Facetten: ersetzbar, memoisiert, mit Lineage (siehe Abschnitt 9).
pub struct Derived<T> {
    pub value: T,
    pub source: ChunkId,       // Hash des Quellinhalts, aus dem abgeleitet wurde
    pub producer: ProducerId,  // z. B. "emb-768-v3", "sum-v2@promptHash"
    pub produced_at: Hlc,
}
```

**Wichtig:** Veränderliche Zähler (Zugriffe, Salience-Updates) sind **nicht** Teil der Cell, sondern liegen in einer separaten **Sidecar-Tabelle** (`AccessStats`). Sonst würde jeder Lesezugriff eine neue Version erzeugen.

### 4.3 Facetten (Lanes)

| Lane | Inhalt | Eigenschaft | Hot/Cold |
|---|---|---|---|
| `meta` | Attribute, Typ, Zeitstempel, Trust, Kanten-Kurzliste | klein, selektiv | **hot** |
| `tokcount` | Token-Anzahl pro registriertem Tokenizer (u32) | Budget-Planung ohne Inhaltszugriff | **hot** |
| `zoom.L2` / `zoom.L1` | Titel / Zusammenfassung | die meisten Recalls nutzen nur diese | **hot** |
| `emb.q` | quantisierter Embedding-Code (z. B. int8/binär, Matryoshka-Präfix) | Kandidatensuche | **hot** |
| `raw` (= `zoom.L0`) | voller Inhalt | groß | cold |
| `tok.ids` | Token-IDs (bitgepackt, optional) | spart Re-Tokenisierung | cold |
| `emb.full` | Embedding in f16/f32 | Re-Ranking | cold |

### 4.4 Zoom-Leiter

Jede Cell hat optional mehrere **Auflösungsstufen**, deren Kosten (`tok_count`) und Provenienz bekannt sind:

```rust
pub struct ZoomLevel {
    pub level: u8,              // 0 = voll, höher = gröber
    pub text: ChunkRef,
    pub tok_count: SmallVec<[(TokenizerId, u32); 2]>,
    pub derived_from: ChunkId,  // Quelle (L0 oder nächstfeinere Stufe)
    pub producer: ProducerId,   // Mensch, Regel (Extractive), oder Modell+Prompt-Hash
}
```

Stufen können **extraktiv** (Titel, erste Sätze, Schlüsselfelder — Engine-intern, deterministisch) oder **abstraktiv** (LLM-Summary — extern erzeugt, über Work Queue) sein. Fehlt eine Stufe, nimmt der Packer die nächstgröbere/feinere (siehe Abschnitt 8).

### 4.5 Strukturbewusstes Chunking

Pluggable `Chunker`-Trait. Standard-Chunker:

- **Markdown/Prosa:** Überschriften-, Absatz- und Satzgrenzen, Zielgröße in *Token*, nicht in Bytes.
- **Code:** Syntaxbaum-basiert (z. B. tree-sitter): Funktion/Klasse/Modul als Einheit, mit Signatur als Zoom-Stufe.
- **JSON/Tabellen:** Pfad-/Zeilenbasiert, Schema als Zoom-Stufe.
- **Binär/groß:** Content-Defined Chunking (FastCDC) für Deduplikation.

Das verwendete Chunking-Schema ist Teil der Namespace-Policy und wird pro Cell als Version vermerkt, damit spätere Re-Chunk-Jobs reproduzierbar sind.

### 4.6 Token-sparsame IDs („Handles")

LLMs sehen IDs im Kontext und müssen sie wiedergeben. UUIDs verschwenden Token und werden falsch abgeschrieben.

- **Handle:** kurzer Alias (z. B. 4–6 Zeichen, Crockford-Base32 ohne verwechselbare Zeichen), **pro Branch** eindeutig und stabil, abgebildet auf `CellId`.
- Validierung bei jedem Zugriff; bei unbekanntem Handle liefert die Engine **„Meintest du `c:k7qd`, `c:k7qb`?"** (Edit-Distanz-Index über Handles).
- Kollisionsfreie Vergabe über eine Alias-Tabelle (Teil des Branch-Zustands).

---

## 5. Physisches Layout

### 5.1 Objektmodell

- **Chunk:** unveränderliches, content-addressed Objekt (`ChunkId = BLAKE3(kanonische Bytes)`). Zwei Arten: *Index-Knoten* (Prolly-Knoten, Segmentblöcke) und *Lane-Blöcke* (Daten einer Cell-Gruppe).
- **Extent:** physische I/O-Einheit, **256 KiB–1 MiB**, 4 KiB-ausgerichtet, enthält mehrere Chunks.
- **Pack:** Datei aus Extents (z. B. 256 MiB–1 GiB), append-only während des Schreibens, danach immutable.
- **Locator-Index:** `ChunkId → (PackId, ExtentNo, Offset, Len)`. Als sortierte Runs mit Ribbon-/Bloom-Filter organisiert; Eltern-Knoten tragen zusätzlich **physische Hints** (nicht Teil des Hashes), sodass der Locator meist übersprungen werden kann. Veraltete Hints sind harmlos (Fallback auf Locator).

### 5.2 Extent-Layout: Hot-Head / Cold-Tail

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

Ergebnis: Ein typischer Recall mit 200 Kandidaten und 12 Gewinnern verursacht ca. 200 Head-Reads (vom Page Cache meist abgedeckt) und ≤ 12 Tail-Reads, und diese liegen dank Layout (5.3) in wenigen Extents.

Innerhalb des Extents sind Lanes **spaltenorientiert** (PAX-artig): SIMD-freundliche Distanzberechnung über `emb.q`, Prädikatsauswertung über `meta`.

### 5.3 Gelerntes Layout (Affinity Re-Layout)

**Beobachtung:** Wer *zusammen gelesen* wird, soll *zusammen liegen*. Das lässt sich nicht aus dem Schlüssel ableiten, wohl aber aus Spuren.

**Datenquellen (Access-Trace, asynchron, gesampelt):**
1. *Co-Retrieval:* Cells, die im selben Recall-Ergebnis (oder derselben Prompt-Assemblierung) landeten.
2. *Struktur:* gleiche Episode/Dokument/Thread (starkes Prior).
3. *Zeit:* zeitliche Nähe des Entstehens (schwaches Prior).
4. *Semantik:* gleiche Vektorzelle (IVF-Zentroid) als Fallback bei Kaltstart.

**Verfahren (Hintergrund-Optimizer):**
- Aufbau eines gewichteten Co-Access-Graphen (bzw. Hypergraphen: ein Recall = eine Hyperkante).
- Partitionierung in extent-große Gruppen (Ziel: **minimiere erwartete Extent-Zugriffe pro Recall**), z. B. mit Multilevel-Partitionierung (METIS-artig) oder inkrementellem Label-Propagation-Clustering.
- Neue Extents werden geschrieben, **nur der Locator-Index wird umgebogen**. ChunkIds, Hashes, Commits bleiben unverändert → Re-Layout ist semantisch unsichtbar und jederzeit abbrechbar.
- **Exploration gegen Rich-get-richer:** ein kleiner Anteil (z. B. 5 %) der Recalls wird mit Zufalls-/Diversitätsbeimischung bedient, damit nicht ständig dasselbe gelernt wird (siehe 18).

### 5.4 Kompression

| Lane | Verfahren |
|---|---|
| Text (`raw`, Zoom) | zstd mit **trainiertem Wörterbuch pro Namespace** (kleine Cells komprimieren ohne Dictionary schlecht); optional Front-Coding für Schlüssel |
| Token-IDs | Bit-Packing nach Vokabulargröße (z. B. 17 Bit bei ~128k Vokabular) + Delta/zstd |
| Embeddings | Matryoshka-Präfix (z. B. 256 Dim.) als Head-Code in int8/binär; volle Präzision im Tail für Re-Ranking; PQ/RaBitQ-artige Codes für große Segmente |
| Attribute | Dictionary-/RLE-/Frame-of-Reference-Codierung (spaltenweise) |
| Kanten | Delta-codierte Adjazenzlisten |

### 5.5 Token-IDs speichern — mit Vorbehalt

Persistierte Token-IDs sparen Re-Tokenisierung beim Prompt-Bau, **aber** Token-Grenzen sind kontextabhängig: `tok(A) ++ tok(B) ≠ tok(A ++ B)` ist möglich. Daher:

- Cells tragen ein Flag **`boundary_safe`** (Chunker schneidet an Stellen, an denen die Konkatenation stabil ist, z. B. nach Zeilenumbruch).
- Nur bei `boundary_safe` darf die Context Engine Token-IDs direkt konkatenieren; sonst Text konkatenieren und neu tokenisieren.
- Die *Token-Zählungen* (für Budgetierung) sind davon unabhängig und werden mit kleiner Toleranz (Sicherheitsmarge pro Cell-Grenze, konfigurierbar) kalkuliert.

---

## 6. Versionierung und Transaktionen

### 6.1 Zustandsmodell: Prolly-Tree + Delta-Log

**Prolly-Tree** (probabilistischer B-Baum): Knotengrenzen werden aus dem Hash der Schlüssel bestimmt. Eigenschaften:

- **History-unabhängig:** gleicher Inhalt ⇒ gleiche Struktur ⇒ gleicher Wurzel-Hash.
- **Strukturelles Sharing:** Forks und Commits teilen alle unveränderten Teilbäume.
- **O(Diff)** für Diff und Merge (Hash-Vergleich von oben nach unten).
- **Augmentierbar:** Innere Knoten speichern Aggregate (z. B. **Token-Summen**, Zähler, Min/Max-Zeit) → Order-Statistic-Abfragen ohne Blattzugriff.

**Problem:** Prolly-Trees haben bei Einzelwrites erhöhte Schreibamplifikation (Pfad zur Wurzel). **Lösung:** Schreibpfad mit **Delta-Log** (LSM-artig):

```
Write ──► WAL (Group Commit) ──► Branch-MemDelta (sortiert, im RAM)
                                       │
                           Hintergrund-"Fold" (batched)
                                       ▼
                    neue Prolly-Knoten + Extents ──► Manifest-Eintrag
```

Der Fold bündelt viele Commits zu einem Baum-Update; die Amplifikation wird über Batches amortisiert. Leser sehen immer `Basis-Baum ⊕ MemDelta` (Merge-Iterator).

### 6.2 Commit-Objekt

```rust
pub struct Commit {
    pub parents: SmallVec<[CommitId; 2]>,
    pub changes: Digest,          // Hash des Changesets (sofort bekannt)
    pub root: Option<RootSet>,    // Zustandshash nach Fold (siehe unten)
    pub author: AgentIdentity,    // Prinzipal + Agent + Run
    pub intent: Intent,           // run_id, step_id, idempotency_key, Freitext-Begründung ("warum")
    pub ts_tx: Hlc,               // Hybrid Logical Clock
    pub sig: Option<Signature>,   // ed25519, optional
}

/// Wurzel des gesamten versionierten Zustands – inklusive aller Indizes.
pub struct RootSet {
    pub primary: ChunkId,         // Prolly-Wurzel der Cells
    pub sequences: ChunkId,       // geordnete Folgen (Episoden/Dokumente) mit Token-Summen
    pub graph: ChunkId,
    pub fts_segments: Vec<SegmentRef>,
    pub vec_segments: Vec<SegmentRef>,
    pub deletion_vectors: ChunkId,
    pub aliases: ChunkId,         // Handle-Tabelle
    pub policies: ChunkId,        // Namespace-Policies
}
```

**Zweistufige Identität:** `CommitId = H(parents, changes, intent)` ist **sofort** nach dem Commit bekannt (kein Warten auf den Fold). `RootSet`-Hash wird beim Fold berechnet und dient der **Zustandsgleichheit** („zwei Branches haben identischen Zustand").

### 6.3 Commit-Protokoll (atomar, crash-sicher)

```
1. begin        → Snapshot (Branch-Head H, Epoche e); Read-Set-Tracking (Bereiche/Prädikate)
2. buffer       → Writes im Transaktions-ChangeSet (Cells, Facetten, Kanten, Löschungen)
3. validate     → Branch-Head bewegt? → Rebase-Versuch, wenn Read-/Write-Sets disjunkt, sonst Konflikt
4. WAL append   → Frame {CRC, BLAKE3-Header, Intent, Idempotenz-Key, ChangeSet}; Group Commit,
                  Durability nach Transaktionsoption (siehe 6.8)
5. apply        → in Branch-MemDelta + SYNCHRONE Index-Deltas
                  (FTS-Postings, Graph-Kanten, Temporal, Token-Summen, Vektor-Delta-Index)
6. publish      → atomarer Pointer-Swap (ArcSwap) auf neuen BranchState  ← Sichtbarkeitspunkt
7. ack          → Client erhält CommitId
8. fold (async) → MemDelta → Prolly-Knoten + Extents → fsync Packs → Manifest-Eintrag
                  (append-only, CRC-geschützt) → WAL-Segment recyceln
```

**Recovery:** Letzten gültigen Manifest-Eintrag laden → WAL-Frames mit `seq > manifest.wal_seq` wiedergeben → Hashes/CRCs prüfen → torn tail verwerfen.
**Atomarität:** Sichtbarkeit nur über Schritt 6 — Leser sehen ein Commit vollständig (inkl. aller Indizes) oder gar nicht.

### 6.4 Branches, Forks, Tags

- **Fork = O(1):** neuer Ref auf bestehenden Commit + leeres MemDelta. Ziel: < 50 µs.
- **Namenskonvention** für Agenten: `main`, `agent/<id>/run-<n>/step-<k>`, `spec/<uuid>` (spekulativ).
- **Speculative Branches** haben eine **TTL** und werden bei Nichtgebrauch automatisch garbage-collected; sie brauchen weder Namen noch Cleanup durch den Agenten.
- **Checkpoints:** Tag auf Commit pro Plan-Schritt → „Rollback auf Schritt 12" ist ein Ref-Update.
- **Scratch-Branches** (`durability = volatile`) leben nur im RAM mit Spill, ohne WAL-fsync — für Arbeitsgedächtnis pro Schritt; Promotion per `squash_commit` in einen durablen Branch.

### 6.5 Isolation und Konflikte

- **Standard: Snapshot-Isolation pro Branch.** Da jeder Agent typischerweise *auf eigenem Branch* arbeitet, treten Konflikte erst beim **Merge** auf, nicht pro Write.
- **Optional serializable** pro Transaktion über Read-Set-Validierung (Bereiche + Bloom für Punktlesezugriffe).
- **Single Sequencer pro Branch** (Actor), Branches parallel.

### 6.6 Merge: typisiert und mit „Konflikt als Daten"

Dreiwege-Merge (Base, Ours, Theirs) **pro Cell und pro Attribut** mit Strategie aus der Namespace-Policy:

| Strategie | Verhalten |
|---|---|
| `lww` | Last-Writer-Wins nach HLC |
| `union` | Mengen-Vereinigung (Tags, Kanten) |
| `sum` | Zähler/CRDT-Counter |
| `max` / `min` | Salience, Confidence |
| `diff3` | Textmerge auf Zeilen-/Satzebene |
| `custom(wasm)` | sandboxed Merge-Funktion |
| `manual` | **erzeugt `ConflictCell`** |

```rust
pub struct ConflictCell {
    pub target: CellId,
    pub base: Option<CellVersion>,
    pub ours: CellVersion,
    pub theirs: CellVersion,
    pub field: FieldPath,
    pub suggestion: Option<MergeSuggestion>, // z. B. aus Extraktiv-Heuristik
}
```

Konflikte sind **abfragbare, versionierte Daten**: ein Agent (oder Mensch) löst sie per `resolve(conflict_id, choice | merged_value)`. Semantische Konflikte („zwei Fakten widersprechen sich") lassen sich so von einem Modell auflösen, ohne dass die Engine selbst ein Modell aufrufen muss.

### 6.7 Bitemporalität

Jede Cell-Version trägt:

- **Transaktionszeit** (`ts_tx`): wann hat Mnemos es erfahren (Commit-HLC).
- **Gültigkeitszeit** (`valid_from`, `valid_to`): wann galt es in der Welt.

Kante `supersedes` verknüpft ersetzte Fakten. Defaults: `recall(valid_at = now)` blendet überholte Fakten aus, `include_history = true` zeigt sie. Zeitreise: `as_of = commit | timestamp`.

Beispiel: „Nutzer wohnt in Mannheim" (gültig bis 2026-03) wird durch „Nutzer wohnt in Berlin" (ab 2026-04) abgelöst, die alte Aussage bleibt prüfbar.

### 6.8 Durability-Stufen (pro Transaktion)

| Stufe | Bedeutung | Einsatz |
|---|---|---|
| `volatile` | nur RAM, kein WAL | Scratchpad pro Schritt |
| `buffered` | WAL geschrieben, fsync gebündelt (≤ 10 ms) | hochfrequente Beobachtungen |
| `durable` | fsync vor Ack (Group Commit) | Standard |
| `replicated` | Quorum-Ack (ab Phase 4) | kritische Fakten |

### 6.9 Idempotenz (Pflicht für Agenten)

Jede schreibende Operation trägt `(run_id, step_id, idempotency_key)`. Das **Idempotenz-Journal** (kurzlebige Map, versioniert) liefert bei Wiederholung dasselbe Ergebnis (gleiche `CommitId`) und führt nichts doppelt aus. Tool-Retries, Timeouts und doppelte Agentenaufrufe sind damit gefahrlos.

---

## 7. Indizes

Alle Indizes sind **segmentbasiert, immutable, content-addressed und im `RootSet` verankert**. Dadurch:
- Forks teilen Segmente ohne Kopie.
- Atomare Sichtbarkeit mit den Primärdaten.
- Zeitreise liefert konsistente Indexstände.

### 7.1 Vektorindex

- **Segmente:** je Segment ein Graphindex (Vamana/DiskANN-artig, Disk-optimiert) mit **Quantisierungscodes im Hot Head** und Re-Ranking aus dem Cold Tail.
- **Delta-Index:** neue Vektoren eines Branches landen in einem kleinen **Flat-/Brute-Force-Index** (SIMD, bis ca. 50k Vektoren), der bei jeder Suche mit durchsucht wird → **kein Index-Lag**.
- **Merge:** Hintergrund-Compaction (size-tiered) erzeugt neue Segmente; alte werden über GC entfernt.
- **Löschungen:** versionierte **Deletion Vectors** (Roaring-Bitmaps, content-addressed) je Segment.
- **Gefilterte Suche:** Prädikate (Namespace, Typ, Zeit, Trust) werden zu Bitmaps aus Attributindizes kompiliert; bei hoher Selektivität Wechsel auf Brute-Force über die Treffermenge.
- **Mehrere Embedder/Versionen** pro Namespace; Modellwechsel = neue Lane + Re-Embedding über Work Queue, alte Lane bleibt bis zur Migration lesbar.

### 7.2 Volltext (FTS)

Segmentbasierter invertierter Index mit BM25, optionalen Positionen (Phrasen) und Feld-Gewichten. Eigene Tokenisierung für Suche, **unabhängig** von LLM-Tokenizern.

### 7.3 Graph

Adjazenz als Prolly-Tree-Schlüssel `(src, edge_type, dst)` plus Rückrichtung. Operationen: begrenzte k-Hop-Expansion mit **Fan-out-Budget**, Personalized-PageRank-Approximation für assoziatives Abrufen (z. B. Entity → verknüpfte Episoden).

### 7.4 Temporal- und Attributindizes

Zeitbereichs-Index (`valid_from/valid_to`, `ts_tx`), invertierte Attributindizes mit Bitmaps; Basis für Vorfilter und Recency-Scoring.

### 7.5 Token-Summen (augmentierter Baum)

Für **Sequenzen** (Dokument → geordnete Kind-Cells, Episode → Turns) speichert jeder innere Knoten die **Summe der Token** seines Teilbaums.

```rust
/// Liefert eine zusammenhängende Menge von Cells um `anchor` mit Summe ≤ budget,
/// ohne Inhalte zu lesen (nur Baum-Aggregate).
fn window(seq: SeqId, anchor: CellId, budget: Tokens, tk: TokenizerId) -> Vec<CellId>;
```

Damit sind „Kontextfenster um einen Treffer" und „Turns 40–60 unter 3 000 Token" **O(log n)** in Knoten-Reads.

---

## 8. Context Engine: Recall unter Token-Budget

### 8.1 Primitive Operationen

| Operation | Zweck |
|---|---|
| `recall(q, budget, policy)` | beste Kontextmenge unter Token-Budget |
| `window(seq, anchor, budget)` | zusammenhängendes Fenster |
| `expand(handle, level)` | feinere/gröbere Stufe einer Cell nachladen |
| `explain(recall_id)` | warum wurde was gewählt (Scores, Quellen, Verworfenes) |
| `neighbors(handle, edge_types, fanout)` | Graph-Nachbarschaft |
| `diff(a, b)` | menschen- und LLM-lesbare Änderungszusammenfassung |

### 8.2 Recall-Anfrage

```rust
let ctx = branch
    .recall(
        Recall::new("Wie haben wir den Deployment-Fehler vom Oktober gelöst?")
            .namespaces(["agent/episodic", "agent/semantic"])
            .budget(Tokens(6_000), TokenizerId::A)
            .signals(Signals::hybrid()          // Vektor + BM25 + Graph + Recency
                .recency_half_life(days(30))
                .min_trust(Trust::ToolVerified))
            .diversity(Mmr { lambda: 0.7 })
            .valid_at(Now)
            .render(Render::Compact)            // spart Token gegenüber JSON
            .order(Order::CacheFriendly),
    )
    .await?;
```

Antwort (schematisch, Compact-Rendering):

```
[c:k7qd] (L1, 142 tok, trust=verified, 2026-10-14)
Deployment schlug fehl wegen abgelaufenem Registry-Token; Lösung: …
[c:m2xa] (L0, 480 tok, trust=user, 2026-10-14)
<voller Text der Cell>
[c:p9hv] (L2, 18 tok) Rollback-Runbook v3 – Titel
… 11 weitere Handles verfügbar via expand(); Budget genutzt: 5 874 / 6 000
```

### 8.3 Pipeline

```
Query ──► Kandidaten ──► Score-Fusion ──► Zoom-Knapsack ──► Ordnung ──► Rendering
          (Vektor, BM25,  (RRF/gewichtet,  (Level je Treffer   (Cache-    (Compact/
           Graph, Recency)  + Trust/Recency) unter Budget)      freundlich) XML/JSON)
```

1. **Kandidatengenerierung:** parallel Vektor (Head-Codes), BM25, Graph-Expansion, zeitlicher Prior. Ranking ausschließlich auf **Hot-Head-Lanes**.
2. **Score-Fusion:** Reciprocal Rank Fusion oder gelernte Gewichte; Trust-/Zeit-Faktoren multiplikativ.
3. **Zoom-Knapsack (Kernidee):** siehe 8.4.
4. **Ordnung:** siehe 8.5.
5. **Rendering:** Formatierer minimieren Tokenoverhead (Compact ≪ JSON); Provenienz und Handles bleiben erhalten.

### 8.4 Zoom-Knapsack

**Problem:** Gegeben Kandidaten *i* mit Stufen *ℓ* (Kosten `c[i][ℓ]` in Token, Nutzen `u[i][ℓ] = rel_i · gain(ℓ) · trust_i · fresh_i`), wähle genau eine Stufe pro Kandidat (oder „weglassen") so, dass die Summe der Kosten ≤ Budget und der Gesamtnutzen — unter Redundanzabzug (MMR) — maximal ist. Das ist ein **Multiple-Choice-Knapsack**.

**Praxisverfahren:**
1. Pro Kandidat die obere konvexe Hülle der (Kosten, Nutzen)-Punkte bilden.
2. Start: alle Kandidaten auf gröbster Stufe (Handle/Titel).
3. Greedy über Upgrade-Schritte mit größtem **Δ-Nutzen / Δ-Kosten** (Max-Heap), bis das Budget erschöpft ist. Auf der konvexen Hülle ist das die optimale LP-Relaxation und praktisch nahezu optimal.
4. Redundanzabzug dynamisch: Nutzen eines Kandidaten sinkt, wenn semantisch ähnliche bereits gewählt sind.

```rust
pub struct Candidate {
    pub cell: CellId,
    pub relevance: f32,
    pub levels: SmallVec<[(Level, Tokens, f32 /*gain*/); 3]>,
    pub trust: Trust,
    pub emb_head: EmbHead,
}

pub fn pack(cands: Vec<Candidate>, budget: Tokens, policy: &PackPolicy) -> Plan;
```

Der entscheidende Vorteil gegenüber „Top-k Chunks fester Größe": viele *grobe* Treffer + wenige *feine* statt wenige vollständige — der Agent bekommt **Überblick und Tiefe** im selben Budget und kann per `expand` nachsteuern.

### 8.5 Cache-freundliche Ordnung

Inferenz-Server cachen KV-Zustände für **gemeinsame Präfixe**. Mnemos ordnet die Ausgabe, wenn gewünscht:

1. **Stabile Cells zuerst** (System-/Policy-/Skill-Cells mit unveränderlichen Hashes), in **kanonischer Reihenfolge** (nach ChunkId/ID) → identische Präfixe über Aufrufe hinweg.
2. Volatiles (frische Episoden, Query-spezifische Treffer) danach.
3. Rückgabe enthält einen **Prefix-Digest** (Hash der ersten N Token-Blöcke), den der Aufrufer für Cache-Keys nutzen kann (Anbindung an Abschnitt 12).

### 8.6 Progressive Disclosure

Das Standardmuster für Agenten: `recall` liefert **kompakt mit Handles** → Agent entscheidet → `expand(handle, L0)` nur für wirklich relevante Cells. Die Engine zählt Expand-Aufrufe als **Relevanzsignal** (Access-Trace), das das Ranking und das Layout verbessert.

### 8.7 Explainability

`explain(recall_id)` liefert pro Kandidat: Signalbeiträge (vec/bm25/graph/recency), gewählte Stufe, Gründe für Verwerfen (Budget, Redundanz, Trust, Filter). Wichtig für Debugging und Evaluation von Agenten.

---

## 9. Abgeleitete Daten, Lineage, Konsolidierung

### 9.1 Derivation Cache

Abgeleitete Facetten werden unter dem Schlüssel `(source_chunk_hash, producer_id)` **memoisiert**:

- **Embedding-Cache:** identischer Text → identischer Hash → Embedding wird nie zweimal berechnet; Forks und Branches teilen den Cache.
- **Token-Zählungen** und **extraktive Zoom-Stufen** analog.
- **Mandantengrenzen:** Cache ist **pro Tenant** (kein Cross-Tenant-Dedup), um Seitenkanal-Informationsleaks („existiert dieser Text bei jemand anderem?") auszuschließen.

### 9.2 Staleness und Lineage

Jede abgeleitete Facette kennt `source` (Hash). Ändert sich der Quellinhalt, ändert sich der Hash ⇒ die Facette ist **per Konstruktion erkennbar veraltet** (keine Invalidierungsnachrichten nötig). Die Engine:

- markiert veraltete Facetten, nimmt sie je nach Policy aus Recall heraus (`allow_stale = false`) oder liefert sie mit Warnung,
- stellt Neuberechnung in die **Work Queue**.

Ein Dependency-DAG (`derived_from`-Kanten) erlaubt **inkrementelle Neuberechnung** in Ketten (L0 → L1-Summary → Entity-Profil → Themen-Summary).

### 9.3 Work Queue und „Sleep Cycles"

Die Engine ruft **kein** Modell auf. Stattdessen:

```rust
pub enum Task {
    Embed       { cells: Vec<CellId>, producer: ProducerId },
    Summarize   { cluster: Vec<CellId>, target_level: u8, max_tokens: u32 },
    Consolidate { episode: SeqId, into: NsId },        // Episoden → semantische Fakten
    Dedupe      { candidates: Vec<(CellId, CellId)> },  // Duplikate/Widersprüche prüfen
    Reconcile   { conflict: ConflictId },
}

let tasks = db.work().lease(worker_id, kinds, limit, lease_ttl).await?; // Leasing, At-least-once
// … externer Worker (LLM-Agent) bearbeitet …
db.work().complete(task_id, result_changeset).await?;                   // landet als normaler Commit
```

Ergebnisse sind normale Commits mit `derived_by = (modell, prompt_hash)` — vollständig nachvollziehbar und per Branch/Review prüfbar.

### 9.4 Vergessen als „Zoom-Down"

Retention-Policies sind **gestuft statt binär**:

1. *Hot → Warm:* Cold-Tail-Lanes auf billigeres Tier verschieben.
2. *Zoom-Down:* L0 (Volltext) entfernen, L1/L2 behalten (Original in Archiv-Tier oder crypto-geshreddert).
3. *Tombstone:* nur Handle + Provenienz + `supersedes`-Spur.

**Salience-Score** = f(Recency, Zugriffshäufigkeit, expliziter Wichtigkeit, Graph-Zentralität, Expand-Rate). Verworfen wird, was niedrig bleibt — mit Schutz für explizit „pinned" Cells.

---

## 10. Sicherheit, Vertrauen, Datenschutz

### 10.1 Trust-/Taint-Labels

```
System > Principal(User) > Tool-Verified > Tool-Unverified > External-Untrusted (Web, Mails, Dokumente Dritter)
```

- Jede Cell trägt ein Label; **abgeleitete Daten erhalten das Minimum** ihrer Quellen (eine Summary einer Webseite bleibt „untrusted").
- `recall` kann `min_trust` filtern oder untrusted Inhalte **markiert und eingerahmt** ausgeben (Delimiter + Quellenvermerk), damit die Agenten-Schicht sie als Daten behandelt und nicht als Instruktionen.
- **Ehrliche Einordnung:** Der Storage kann Prompt Injection nicht *verhindern*, aber **verlässliche Herkunftsmetadaten** liefern, auf denen Gegenmaßnahmen aufsetzen. Trust-Labels sind nur durch privilegierte Prinzipale **hochstufbar** (Auditeintrag).

### 10.2 Zugriffskontrolle: Capabilities

Macaroon-artige, **attenuierbare Tokens** (Caveats: Namespaces, Branches, Operationen, Trust-Obergrenze, TTL, Token-Budget pro Zeitraum). Ein Hauptagent kann Subagenten Tokens mit **eingeschränkten Rechten** ausstellen, ohne den Server zu fragen.

### 10.3 Verschlüsselung und Crypto-Shredding

- **Envelope-Encryption:** Daten-Extents mit AES-256-GCM oder XChaCha20-Poly1305; Schlüssel pro Tenant/Namespace.
- **Per-Subject-Keys (`encryption_scope = "per-subject"`):** Cells, die personenbezogene Daten *einer* betroffenen Person enthalten, werden mit deren Schlüssel verschlüsselt.
- **Löschung nach DSGVO Art. 17** = Schlüssel vernichten (**Crypto-Shredding**). History, Hashes, Merkle-Struktur und Commit-Kette bleiben **intakt und verifizierbar**; die Inhalte sind unwiederbringlich unlesbar. Abgeleitete Facetten (Embeddings, Summaries) derselben Person nutzen denselben Schlüssel bzw. werden per Lineage mit-geshreddert.
- **Wichtig:** Hash-Werte von Klartext können Rückschlüsse erlauben → für Per-Subject-Daten wird der Content-Hash über den **verschlüsselten/gesalzenen** Inhalt gebildet (Dedup wird dort bewusst geopfert).

### 10.4 Audit und Integrität

- Die Commit-Kette ist ein **Merkle-DAG** → manipulationsevident; optionale ed25519-Signaturen.
- Lesezugriffe werden (gesampelt oder vollständig, konfigurierbar) im Access-Trace protokolliert, getrennt vom Inhalts-Zustand.
- PII-Tagging als Attribut; Policies können PII-Cells von Recall oder Export ausschließen.

---

## 11. API-Oberflächen und Agenten-Ergonomie

### 11.1 Rust-API (eingebettet)

```rust
let db = Mnemos::open(Config::at("/var/lib/mnemos"))?;

let main    = db.branch("main")?;
let attempt = main.fork("agent/a1/run-42/step-7")?;           // O(1)

let mut tx = attempt.begin(
    Tx::new()
        .intent("Fehlerursache aus Logs als Fakt ablegen")
        .idempotency_key("run-42/step-7/put-fact-1")
        .durability(Durability::Durable),
);
tx.put(Cell::fact("Registry-Token war abgelaufen").trust(Trust::ToolVerified)
        .edge(Edge::derived_from(log_cell)))?;
let commit = tx.commit().await?;

let ctx   = attempt.recall(Recall::new("Registry Token").budget(Tokens(3_000))).await?;
let merge = main.merge(&attempt, MergePolicy::from_namespace_defaults()).await?;
match merge {
    Merged(c)       => { /* fertig */ }
    Conflicts(list) => { /* ConflictCells abfragen und auflösen */ }
}
```

### 11.2 MCP-Server (Model Context Protocol)

Mnemos liefert einen **MCP-Server** mit wenigen, gut beschriebenen Tools — Ziel: wenige, mächtige Tools statt vieler kleiner (spart Tool-Beschreibungs-Token):

| Tool | Zweck |
|---|---|
| `mem_recall` | Recall unter Budget, Compact-Rendering, Handles |
| `mem_expand` | Handle → feinere/gröbere Stufe |
| `mem_write` | Cells ablegen (idempotent), optional auf Branch |
| `mem_fork` / `mem_merge` / `mem_discard` | Versuch-Workflow |
| `mem_history` / `mem_diff` | Zeitreise und Änderungsübersicht |
| `mem_describe` | Schema/Namespaces/Policies — **budgetiert** |

Tool-Schemas werden **aus den Rust-Typen generiert** (`schemars`/JSON-Schema), damit Beschreibung und Implementierung nicht auseinanderlaufen.

### 11.3 LLM-lesbare Fehler

```json
{
  "error": "unknown_handle",
  "message": "Handle c:k7qx existiert auf Branch agent/a1/run-42 nicht.",
  "retryable": false,
  "did_you_mean": ["c:k7qd", "c:k7qb"],
  "suggested_call": { "tool": "mem_recall", "args": { "q": "…" } }
}
```

Konflikte, Budgetüberschreitungen und Berechtigungsfehler folgen demselben Schema (`error`, `message`, `retryable`, `hint`, `suggested_call`).

### 11.4 Selbstbeschreibung (`mem_describe`)

Liefert Namespaces, Cell-Typen, Attribute, Merge-Regeln und Beispiele **in einem Token-Budget** (z. B. 800 Token) und im Compact-Format. So lernt ein Agent die Datenbank kennen, ohne Dokumentation vorauszusetzen.

---

## 12. Optional: Prefix-/KV-Cache-Store

*(Phase 5 — bewusst vom Kern getrennt, da modellabhängig und hardwarenah.)*

**Motivation:** Kontexte werden aus wiederkehrenden Bausteinen zusammengesetzt (System-Prompt, Skills, Policies, Projekt-Wissen). Das Berechnen der KV-Zustände dafür ist teuer und wiederholt sich.

**Konzept:**

- **Schlüssel:** `(model_id, tokenizer_id, hash-Kette über Token-Blöcke)` — Merkle-artig, sodass Präfixe gemeinsame Blöcke teilen (Radix-Struktur).
- **Blockgröße:** an Inferenz-Server angepasst (z. B. 16–256 Token pro Block).
- **Speicherung:** eigene Pack-Art auf NVMe (große, ausgerichtete Blöcke, ggf. GPUDirect-Storage-kompatibel), Tiering RAM ↔ NVMe ↔ Objektspeicher.
- **Eviction:** **kostenbasiert** (`Wiederberechnungskosten = Token × Modellkosten`, `Nutzungsfrequenz`, `Alter`), nicht reines LRU.
- **Anbindung:** Konnektoren zu gängigen Inferenz-Stacks (z. B. über deren KV-Cache-Offloading-Schnittstellen); Mnemos liefert **Prefix-Digest** (8.5) und verwaltet Blöcke.
- **Invalidierung:** automatisch über Hash-Kette — ändert sich ein stabiler Baustein, ändert sich der Schlüssel, alte Blöcke altern aus.

---

## 13. Tiering, Garbage Collection, Replikation

### 13.1 Tiering

| Tier | Inhalt | Medium |
|---|---|---|
| T0 | MemDelta, Locator-Hot-Set, Head-Page-Cache, Alias-Tabellen | RAM |
| T1 | Packs mit Hot Heads + aktuellen Cold Tails | NVMe |
| T2 | Kalte Tails, Archiv-Segmente, alte Versionen | Objektspeicher (S3-kompatibel), Range-Reads |

Entscheidung über **Kosten-Nutzen-Score** (Zugriffsfrequenz × Latenzvorteil ÷ Platzkosten). Hot Heads bleiben möglichst auf T1, auch wenn Tails wandern.

### 13.2 Garbage Collection

- **Generationelles Mark-and-Sweep** ausgehend von Refs, Reflog-Retention, gepinnten Snapshots und Speculative-TTL.
- **Extent-Compaction** bei Live-Anteil < Schwellwert (z. B. 50 %), kombiniert mit Re-Layout (5.3) — ein Durchlauf, zwei Zwecke.
- Löschungen im Sinne von Crypto-Shredding (10.3) benötigen **keine** Voraussetzung für GC, sparen aber nach Ablauf der Retention auch den Platz.

### 13.3 Replikation und Sync

Da alles content-addressed ist, entspricht Sync einem **Git-artigen Fetch/Push**: Austausch von Commit-Frontiers, Aushandeln fehlender Chunks, Übertragen als Packs. Das ergibt:

- **Multi-Device-Memory** (Laptop-Agent ↔ Cloud-Agent) ohne Sonderlogik,
- **Federated Memory** (Teams teilen ausgewählte Branches/Namespaces),
- **Read-Replicas** (Pull-basiert) und später **Quorum-Replikation** für `Durability::Replicated` (Raft über den Branch-Sequencer).

---

## 14. Rust-Implementierung

### 14.1 Crate-Layout (Cargo Workspace)

```
mnemos/
├─ mnemos-core        Typen, IDs, Fehler, Zeit (HLC), Konfiguration         [safe]
├─ mnemos-codec       Lane-Codecs (zstd+dict, bitpack, FOR, quantisierung)  [safe, SIMD optional]
├─ mnemos-io          io_uring/pread-Backends, O_DIRECT, Pack/Extent-I/O    [unsafe isoliert]
├─ mnemos-wal         WAL, Group Commit, Manifest-Log, Recovery             [safe]
├─ mnemos-cas         Chunk-Store, Locator-Index, Layout-Optimizer, GC      [safe]
├─ mnemos-tree        Prolly-Tree (augmentierbar), Delta-Log, Merge-Iterator[safe]
├─ mnemos-txn         Commit-DAG, Branches, Isolation, Merge, Idempotenz    [safe]
├─ mnemos-idx-vec     Vektorsegmente, Delta-Index, Distanz-Kernels          [unsafe isoliert (SIMD)]
├─ mnemos-idx-fts     Invertierter Index, BM25                              [safe]
├─ mnemos-idx-graph   Adjazenz, Expansion, PPR                              [safe]
├─ mnemos-ctx         Context Engine: Fusion, Zoom-Knapsack, Rendering      [safe]
├─ mnemos-work        Work Queue, Lineage, Retention, Konsolidierung        [safe]
├─ mnemos-sec         Capabilities, Verschlüsselung, Schlüsselverwaltung    [safe, auditierte Krypto-Crates]
├─ mnemos-sim         Deterministische Simulation (Fake-Clock/IO/Netz)      [test]
├─ mnemos-server      gRPC/HTTP + MCP                                       [safe]
├─ mnemos-cli         Inspektion, Diff, Dump, Verify, Bench
└─ mnemos-bindings    Python (PyO3), TypeScript (napi-rs), C-ABI
```

`#![forbid(unsafe_code)]` in allen Crates außer `mnemos-io` und `mnemos-idx-vec`; dort `unsafe` nur hinter kleinen, mit Miri/loom/Fuzzing getesteten Abstraktionen.

### 14.2 Nebenläufigkeit

- **Lese-Pfad:** wartefrei/lock-arm — immutable Strukturen + `ArcSwap` für `BranchState`, epochenbasierte Rückgewinnung (`crossbeam-epoch`) für Caches und Locator-Runs.
- **Schreib-Pfad:** ein Sequencer-Actor pro Branch; Group Commit bündelt WAL-Writes über Branches.
- **I/O-Pfad:** auf Linux `io_uring` (eigener Thread-per-Core-Executor, z. B. auf Basis von `monoio`/`glommio`/`io-uring`), Fallback `pread/pwrite`-Threadpool (macOS/Windows). API-Schicht auf `tokio`, verbunden über Kanäle.
- **Hintergrundjobs** (Fold, Compaction, Re-Layout, GC) laufen mit Prioritäts- und Bandbreitenbudgets (Token-Bucket), damit sie Recall-Latenzen nicht verdrängen.

### 14.3 Serialisierung und Zero-Copy

`rkyv`/`bytemuck` für Extent-Verzeichnisse und Lanes, damit Lesen = Zeiger-Cast statt Deserialisierung. Alle On-Disk-Formate besitzen **Magic, Versionsnummer, Feature-Flags** und sind vorwärtskompatibel dokumentiert (Format-Spezifikation als eigenes Dokument, vor Phase 1 einfrieren).

### 14.4 Bibliotheken (Auswahl, jeweils zu evaluieren)

| Bereich | Kandidaten |
|---|---|
| Hash/Krypto | `blake3`, `aes-gcm`/`chacha20poly1305`, `ed25519-dalek`, `zeroize` |
| Kompression | `zstd` (mit Dictionaries), `lz4_flex` |
| Concurrency | `arc-swap`, `crossbeam`, `parking_lot` |
| I/O | `io-uring`, `monoio`/`glommio`, `memmap2` (nur read-only Pfade) |
| SIMD | `std::simd` (sobald stabil), `pulp`/`wide`, optional `simsimd` |
| Bitmaps | `roaring` |
| Serde | `rkyv`, `serde` (nur Rand), `schemars` (Tool-Schemas) |
| Tests | `proptest`, `loom`, `cargo-fuzz`, `criterion`, `insta` |
| Parser | `tree-sitter` (Code-Chunker) |

Entscheidungsregel: *Eigenbau dort, wo atomare Integration mit dem Commit-Modell nötig ist (Segmente, Index-Roots, Layout), Fremdcrates für Krypto, Kompression, Parser.* Ein Einsatz von Tantivy o. ä. ist nur als Übergangslösung in Phase 1 denkbar, da dessen Segmentmodell nicht Teil unseres `RootSet` ist.

### 14.5 Teststrategie (zentral, nicht nachgelagert)

1. **Deterministische Simulation** (FoundationDB/TigerBeetle-Stil): gesamte Engine gegen simulierte Zeit, simuliertes Dateisystem (torn writes, verlorene fsyncs, Bit-Flips, volle Platte) und simuliertes Netz; Seeds reproduzierbar.
2. **Modellbasierte Property-Tests:** `proptest` gegen ein triviales In-Memory-Referenzmodell (Branches, Merge, Time-Travel, bitemporale Abfragen).
3. **Invarianten-Checker (`mnemos verify`):** Hashbaum, Locator-Konsistenz, Index↔Primärdaten, Lineage, GC-Erreichbarkeit.
4. **`loom`** für lock-freie Strukturen, **Miri** für `unsafe`, **Fuzzing** für alle Decoder.
5. **Crash-Recovery-Schleifen:** kill -9 an zufälligen Punkten, anschließend Verify.
6. **Retrieval-Qualitätstests** (siehe 16.2) als CI-Gate, nicht nur Latenz.

---

## 15. Vergleich mit RocksDB und Alternativen

*(Vereinfachte Einordnung nach allgemeinem Kenntnisstand; konkrete Produktfeatures ändern sich und sind vor Entscheidungen gegenzuprüfen.)*

| Aspekt | RocksDB / LSM-KV | Dolt (Prolly) | Lance/LanceDB | Vektor-DBs (Qdrant u. ä.) | Git | **Mnemos** |
|---|---|---|---|---|---|---|
| Primitive | Key → Bytes | SQL-Tabellen | Spalten-Tabellen + Vektoren | Vektor + Payload | Blobs/Trees | **Cell mit Facetten + Zoom** |
| Versionierung | nur Snapshots | Commits/Branches | Dataset-Versionen | meist keine/limitiert | ja (Dateien) | **Branch/Fork/Merge im Kern, O(1)-Fork** |
| Atomare Änderung inkl. Indizes | Batch (nur KV) | ja (Tabellen) | ja (Manifest) | je nach System | ja (Dateien) | **ja, Indizes im Commit-Wurzelobjekt** |
| Vektor + Volltext + Graph | nein (extern) | begrenzt | Vektor/FTS | Vektor, teils Hybrid | nein | **alle drei, versioniert, hybrid fusioniert** |
| Token-Budget-Awareness | nein | nein | nein | nein | nein | **ja (Token-Summen, Zoom-Knapsack)** |
| Layout-Optimierung | Schlüsselordnung | Schlüsselordnung | Spaltenlayout | Index-Layout | Packfile-Delta | **zugriffsgelerntes Co-Retrieval-Layout** |
| Abgeleitete Daten/Lineage | nein | nein | teils (Spalten) | nein | nein | **Derivation Cache + Staleness** |
| Trust/Taint | nein | nein | nein | nein | nein | **ja, propagierend** |
| DSGVO-Löschung bei History | Compaction | schwierig | schwierig | Delete | schwierig | **Crypto-Shredding** |
| Agent-Semantik (Idempotenz, Handles, LLM-Fehler) | nein | nein | nein | nein | nein | **ja** |

**Warum nicht „RocksDB + Aufsatz"?** LSM-Compaction sortiert nach Schlüssel und schreibt Daten mehrfach um; unsere zentralen Anforderungen (inhaltsadressierte Unveränderlichkeit, strukturelles Sharing, ortsunabhängiges Re-Layout, Index-Wurzeln im Commit) stehen quer dazu. Wo LSM-Ideen passen — der **Delta-Log** für Schreibabsorption — übernehmen wir sie bewusst.

**Abgrenzung zu Memory-Frameworks** (Anwendungsschicht): Sie bauen Retrieval/Konsolidierung *über* generischen Datenbanken. Mnemos verlagert die dafür nötigen Primitive in die Storage-Schicht, sodass sie atomar, versioniert und physisch optimiert sind.

---

## 16. Ziele, KPIs und Evaluationsmethodik

### 16.1 Design-Ziele (zu validieren)

| Metrik | Ziel |
|---|---|
| Fork | < 50 µs, Speicher O(1) |
| Commit (1–100 Cells, NVMe, Group Commit, `durable`) | p99 < 2 ms |
| Recall (1 M Cells, warm, 8k-Budget, hybrid, 3 Namespaces) | p50 < 5 ms, p99 < 25 ms |
| Extent-Zugriffe pro Recall nach Re-Layout | ≤ 4 im Mittel (Hypothese; Baseline: ohne Re-Layout messen) |
| `window()` über Sequenz mit 10⁶ Turns | < 1 ms ohne Inhaltszugriff |
| Dedup-/Sharing-Gewinn auf realen Agenten-Traces | Hypothese: deutlich messbar; **Baseline erheben** |
| Schreibamplifikation (Fold) | < 5× bei Batchgröße ≥ 1 000 Cells (Hypothese) |
| Recovery-Zeit nach Crash | < 5 s bei 1 GiB WAL |

### 16.2 Qualitätsmetriken (entscheidend)

Reine Latenz reicht nicht. **Utility@Budget** ist die zentrale Größe:

1. **Trace-Replay:** Aufzeichnung realer Agentenläufe (Anfragen, tatsächlich verwendete Cells, Ergebnis der Aufgabe). Wiedergabe gegen Mnemos vs. Baselines (Top-k fester Chunk-Größe; RocksDB+externe Vektor-DB).
2. **Metriken:** Recall@Budget, Nutzen-pro-Token (Anteil der geladenen Token, die der Agent tatsächlich verwendet/zitiert), Task-Erfolgsrate bei gleichem Budget, Expand-Rate, Kosten pro gelöster Aufgabe.
3. **A/B-Betrieb:** Zoom-Knapsack vs. Top-k, Cache-freundliche vs. relevanzbasierte Ordnung (inkl. Prefix-Cache-Trefferquote).
4. **Layout-Evaluation:** erwartete Extent-Zugriffe pro Recall vor/nach Re-Layout; Tail-Latenz.

### 16.3 Benchmarks

Mikro-Benchmarks (`criterion`) pro Crate; Systembenchmarks mit synthetischen *und* aufgezeichneten Workloads; Regression-Gates in CI (Latenz **und** Qualität).

---

## 17. Roadmap

| Phase | Inhalt | Exit-Kriterium |
|---|---|---|
| **0 — Spike (6–8 Wochen)** | Format-Spezifikation Entwurf; Prototyp: CAS + WAL + Prolly-Tree + Fork/Commit; Simulationstest-Gerüst; Trace-Recorder für Agenten | Fork/Commit/Recovery korrekt unter Simulation; Mess-Baselines erhoben |
| **1 — MVP Kern (3–4 Monate)** | Cells/Lanes, Hot-Head/Cold-Tail-Extents, Branches/Merge (typisiert), Idempotenz, FTS, Flat-Vektor-Delta, Token-Summen, Rust-API + CLI + `verify` | Alle Invarianten grün; Replay-Evaluation vs. Baseline vorhanden |
| **2 — Context Engine + MCP (2–3 Monate)** | Segment-Vektorindex (Disk), Hybrid-Fusion, Zoom-Knapsack, Handles/Expand/Explain, MCP-Server, LLM-lesbare Fehler | Utility@Budget signifikant über Top-k-Baseline (Replay) |
| **3 — Lernen & Lebenszyklus (3 Monate)** | Access-Trace, Affinity Re-Layout, Derivation Cache, Work Queue, Retention/Zoom-Down, Trust-Labels, Crypto-Shredding, Capabilities | Extent-Zugriffe/Recall messbar reduziert; DSGVO-Löschtest bestanden |
| **4 — Verteilung (3–4 Monate)** | Fetch/Push-Sync, Read-Replicas, Quorum-Durability, Tiering zu Objektspeicher | Multi-Device-Szenario; Partitions-/Fault-Tests in Simulation |
| **5 — Prefix/KV-Store (optional)** | Prefix-Digest, KV-Block-Store, Konnektoren, kostenbasierte Eviction | Gemessene Prefix-Cache-Trefferquote und TTFT-Verbesserung |

**Team-Skizze:** 3–4 Storage-Ingenieure (Rust), 1 Retrieval/ML-Ingenieur (Index, Fusion, Evaluation), 1 Test-/Simulations-Ingenieur; frühe Nutzung durch ein internes Agenten-Team als „Design-Partner".

---

## 18. Risiken und offene Fragen

| # | Risiko / Frage | Einschätzung | Mitigation |
|---|---|---|---|
| 1 | **Schreibamplifikation Prolly-Tree** bei kleinen, häufigen Commits | mittel | Delta-Log + Fold-Batching; Knotengrößen/Fan-out kalibrieren; Messung in Phase 0 |
| 2 | **ANN mit Versionierung** (Segmente, Deletion Vectors, Filter) ist komplex | hoch | segmentierter Ansatz mit Delta-Flat-Index; Phase 1 zunächst nur Flat/IVF, DiskANN-artig in Phase 2 |
| 3 | **Learned-Layout-Feedbackschleife** (Rich-get-richer, Drift) | mittel | Exploration (5 %), Gewichtszerfall, Rollback über Locator-Versionen |
| 4 | **Tokenizer-Kopplung** (Modellwechsel macht Zählungen/IDs ungültig) | mittel | Zählungen pro Tokenizer-Familie, lazy Neuberechnung (Derivation Cache), `boundary_safe`-Flag |
| 5 | **Semantische Merge-Konflikte** sind nicht automatisch lösbar | mittel | `ConflictCell` + externe Auflösung; konservative Defaults (`manual`) |
| 6 | **GC bei vielen Branches** (Erreichbarkeit, Pins) | mittel | TTL für Speculative Branches, generationelles GC, Reflog-Limits |
| 7 | **Dedup vs. Datenschutz** (Seitenkanäle, Hash von Klartext) | hoch | Tenant-lokale Caches, gesalzene Hashes für Per-Subject-Daten |
| 8 | **Scope-Creep** (KV-Store, Verteilung, Merge-WASM) | hoch | Phasen strikt; Kern (Phase 0–2) liefert bereits den Hauptnutzen |
| 9 | **Format-Lock-in** (On-Disk-Formate schwer änderbar) | hoch | Spezifikation vor Phase 1 einfrieren; Feature-Flags; Migrationswerkzeug von Anfang an |
| 10 | **Kein Nachweis des Nutzens ohne Daten** | hoch | Trace-Recorder und Replay-Evaluation in Phase 0; Go/No-Go-Entscheidung nach Phase 2 |

### Offene Fragen für die Architekturrunde

1. **Konsistenzmodell der Zugriffsstatistik:** Reicht eventual consistency für Sidecar-Zähler, oder braucht das Ranking exakte Werte?
2. **Handle-Stabilität über Branches:** Pro-Branch-Aliase vs. global eindeutige Kurz-IDs mit Kollisionsbehandlung beim Merge?
3. **Granularität der Verschlüsselung:** Per-Subject-Keys vs. Extent-Layout — wie verhindern wir, dass Extents Cells vieler Subjekte mischen (Re-Layout muss Schlüsselgrenzen respektieren)?
4. **Embedding-Lanes bei Modellwechsel:** Dual-Betrieb bis zur Migration — wie lange und mit welchem Speicherbudget?
5. **Merge-Funktionen in WASM:** Sandbox-Overhead und Determinismus (Fuel-Limits, keine Zeit/Zufall)?
6. **Eigener FTS vs. Tantivy-Integration** (Segmentformat-Kompatibilität mit `RootSet`)?
7. **Deployment-Modell:** primär *embedded* (wie SQLite/RocksDB) oder *Server* (wie Postgres)? Vorschlag: Embedded-Kern + dünner Server-Wrapper, beides ab Phase 2.

---

## Anhang A: Glossar

| Begriff | Bedeutung |
|---|---|
| **Cell** | Wissenseinheit mit Facetten (Inhalt, Tokens, Embeddings, Zoom, Kanten, Zeit, Trust) |
| **Facet/Lane** | physisch getrennt gespeicherter Teil einer Cell (z. B. `meta`, `emb.q`, `raw`) |
| **Zoom-Leiter** | Auflösungsstufen einer Cell (L0 voll … Ln Titel) mit bekannten Token-Kosten |
| **Chunk** | unveränderliches, content-addressed Objekt (BLAKE3) |
| **Extent** | physische I/O-Einheit (256 KiB–1 MiB) mit Hot Head und Cold Tail |
| **Pack** | Datei aus Extents |
| **Locator-Index** | Abbildung `ChunkId → physischer Ort` |
| **Prolly-Tree** | probabilistischer, history-unabhängiger, hash-adressierter B-Baum |
| **RootSet** | Wurzelobjekt des gesamten versionierten Zustands inkl. Indizes |
| **Handle** | kurzer, token-sparsamer Alias einer Cell |
| **Derivation Cache** | Memoisierung abgeleiteter Facetten pro (Quellhash, Producer) |
| **Zoom-Knapsack** | Auswahl der Auflösungsstufe je Treffer unter Token-Budget |
| **Crypto-Shredding** | Löschung durch Vernichtung des Schlüssels statt Überschreiben der Daten |
| **HLC** | Hybrid Logical Clock |

## Anhang B: Erste konkrete Schritte (nächste 4 Wochen)

1. **Trace-Recorder** für zwei bis drei reale Agenten-Workflows bauen (Anfragen, geladene Inhalte, Zitationen/Nutzung, Task-Ergebnis) — ohne Daten keine Evaluation.
2. **Format-Spezifikation v0** (Extent-Header, Lane-Directory, WAL-Frame, Manifest-Eintrag) schreiben und reviewen.
3. **Spike:** BLAKE3-CAS + WAL + Prolly-Tree + O(1)-Fork in `mnemos-sim` unter fehlerinjizierter Simulation.
4. **Baselines messen:** RocksDB + HNSW-Bibliothek + Top-k-Recall auf den aufgezeichneten Traces (Latenz, Extent-/Page-Zugriffe, Utility@Budget).
5. **Entscheidungsvorlage** (Go/No-Go für Phase 1) auf Basis der Messungen.
