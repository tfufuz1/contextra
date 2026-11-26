# Contextra — Spezifikation des normativen Soll-Zustands

**Version 1.0 · November 2026 · Status: Finale Spezifikation**
**Rolle:** Principal Rust Engineer (Systemprogrammierung, Durability, Concurrency, RAG-Speicherkerne)

> Alle Zahlen zu Performance und Platzbedarf in diesem Dokument sind **Designziele bzw. Hypothesen**, keine Messwerte. Sie müssen durch Prototypen validiert werden (siehe Abschnitt 16).

---

## Inhaltsverzeichnis

0. [Kurzfassung und Designprinzipien](#0-kurzfassung-und-designprinzipien)
1. [Produktvision und Scope](#1-produktvision-und-scope)
2. [Datenmodell: Cells, Facets, Zoom](#2-datenmodell-cells-facets-zoom)
3. [Physisches Layout](#3-physisches-layout)
4. [Versionierung und Transaktionen](#4-versionierung-und-transaktionen)
5. [Indizes](#5-indizes)
6. [Context Engine](#6-context-engine)
7. [Abgeleitete Daten und Lineage](#7-abgeleitete-daten-und-lineage)
8. [Sicherheit und Vertrauen](#8-sicherheit-und-vertrauen)
9. [API-Oberflächen](#9-api-oberflächen)
10. [Crate-Architektur](#10-crate-architektur)
11. [Nebenläufigkeitsmodell](#11-nebenläufigkeitsmodell)
12. [Durability, Recovery, Garbage Collection](#12-durability-recovery-garbage-collection)
13. [Ingest-Pipeline](#13-ingest-pipeline)
14. [Adaptive Ranking und Fusion](#14-adaptive-ranking-und-fusion)
15. [KV-Cache-Store (optional)](#15-kv-cache-store-optional)
16. [Performance-Ziele und KPIs](#16-performance-ziele-und-kpis)
17. [Teststrategie](#17-teststrategie)
18. [Roadmap](#18-roadmap)
19. [Risiken und offene Fragen](#19-risiken-und-offene-fragen)
20. [Glossar](#20-glossar)

---

## 0. Kurzfassung und Designprinzipien

### 0.1 Was ist Contextra?

Contextra ist ein **LLM-natives, versioniertes Storage-System** in Rust. Es macht die Besonderheiten von LLM- und Agenten-Workloads — Token-Budgets, unscharfe Suche, spekulative Verzweigungen, Mehrfachauflösungen, unzuverlässige Clients — zu **Primitiven der Speicherschicht**, statt sie in jeder Anwendung neu zu bauen.

Unterhalb der Versionierungsschicht ist Contextra ein **Git-artiger Objektspeicher mit gelerntem physischen Layout**. Oberhalb ist es eine **kontextbewusste Wissensdatenbank** mit hybrider Retrieval-Engine. Der Übergangspunkt ist der **Commit** — ein Hash über das gesamte Wurzelobjekt inklusive aller Indizes.

### 0.2 Die zehn Kernideen

| # | Idee | Kurzbeschreibung |
|---|------|------------------|
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

### 0.3 Designprinzipien

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

## 1. Produktvision und Scope

### 1.1 Was Contextra ist

Contextra ist eine **eingebettete Datenbank und RAG-Engine** für LLM-Agenten mit folgenden Eigenschaften:

- **Versioniert:** Git-artige Commits, Branches, Fork, Merge, Diff, Zeitreise — atomar über Primärdaten und Indizes.
- **Token-bewusst:** Budget als First-Class-Parameter, Zoom-Stufen, Token-Summen-Bäume.
- **Hybrid-Retrieval:** Vektor + BM25 + Graph + Temporal + Attribute, fusioniert in einer Pipeline.
- **Agent-sicher:** Idempotenz, spekulative Branches, Trust-Labels, LLM-lesbare Fehler.
- **Eingebettet + Server:** Primär als Rust-Library, mit optionalem MCP-/gRPC-Server für Mehrprozess-Zugriff.

### 1.2 Deployment-Modell

- **Primär:** Eingebettete Library (`libcontextra`), vergleichbar mit SQLite/RocksDB.
- **Server-Modus:** Dünner MCP-/gRPC-Server für Multi-Prozess- und Netzwerkzugriff.
- **Ein Repository = eine Instanz**, Single-Process. Mehrprozess läuft über den Server.
- **Tiering:** RAM ↔ NVMe ↔ Objektspeicher (S3-kompatibel).

### 1.3 Nicht-Ziele

| Nicht-Ziel | Begründung |
|---|---|
| Verteilter Konsens in v1 | Einzelknoten; Replikation später über Objekt-Austausch |
| Eigene SQL-Engine | SQLite-vtab als Frontend (der Dialekt, den LLMs kennen) |
| Git-Wire-Protokoll-Kompatibilität | Export/Import-Pfad möglich, kein Ziel |
| Einzeldatei-Format | Verzeichnis wegen Crash-Sicherheit |
| Trigger/Stored Procedures in v1 | Spätere Erweiterung |
| DSGVO-Löschkette als Architektur-Treiber | Nicht im Scope; bei Bedarf über Standard-Löschmechanismen |

### 1.4 Typische Zugriffsmuster

1. **Recall unter Budget:** „Gib mir die besten ≤ 6 000 Token Kontext zu dieser Anfrage." (häufigste Leseoperation)
2. **Fenster-Lesen:** „Gib mir die Turns 40–60 dieser Episode, höchstens 3 000 Token."
3. **Expand:** Der Agent sieht ein Handle `c:k7qd` in kompakter Form und fordert dessen Volltext an.
4. **Append-lastiges Schreiben:** Episoden, Tool-Ergebnisse, Beobachtungen — kleine Cells, hohe Rate, fast nie Updates in place.
5. **Fork → Arbeiten → Merge/Verwerfen:** pro Plan-Schritt oder Experiment.
6. **Hintergrundarbeit:** Konsolidierung, Re-Embedding bei Modellwechsel, Vergessen/Abwerten.

---

## 2. Datenmodell: Cells, Facets, Zoom

### 2.1 Namespace

Ein **Namespace** ist eine Sammlung mit gemeinsamer Policy (vergleichbar mit Collection/Table). Policies sind Teil des versionierten Zustands.

```toml
[namespace."agent/episodic"]
kind              = "episodic"          # episodic | semantic | procedural | working | artifact
chunker           = "markdown-v2"       # strukturbewusst (siehe 2.5)
tokenizers        = ["tok-A", "tok-B"]  # Token-Zählungen für diese Familien persistiert
embedders         = ["emb-768-v3"]
zoom_ladder       = ["L0:raw", "L1:summary<=200tok", "L2:title<=24tok"]
default_trust     = "tool-unverified"
retention         = { hot_days = 14, warm_days = 180, zoom_down_after = "180d" }
merge             = { text = "diff3", tags = "union", counters = "sum", default = "manual" }
durability        = "durable"           # volatile | buffered | durable | replicated
```

### 2.2 Cell — die zentrale Primitive

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
    pub time: BiTemporal,              // Transaktionszeit + Gültigkeitszeit (siehe 4.6)
    pub trust: TrustLabel,             // siehe 8.1
    pub prov: Provenance,              // Autor/Agent/Run/Step, Quelle, ggf. Modell+Prompt-Hash
    pub derived: DerivedRefs,          // Verweise auf abgeleitete Facetten (nicht Teil des Inhalts-Hash)
}
```

**Wichtig:** Veränderliche Zähler (Zugriffe, Salience-Updates) sind **nicht** Teil der Cell, sondern liegen in einer separaten **Sidecar-Tabelle** (`AccessStats`). Sonst würde jeder Lesezugriff eine neue Version erzeugen.

### 2.3 Provenance — Herkunft als First-Class-Citizen

```rust
pub struct Provenance {
    pub author: PrincipalId,       // Mensch, Agent, System
    pub source: SourceRef,         // URI, Tool-Call-ID, Dokument-Referenz
    pub model: Option<ModelRef>,   // bei LLM-generierten Inhalten: Modell + Prompt-Hash
    pub confidence: Option<f32>,   // 0.0–1.0, optional
    pub run_id: Option<RunId>,
    pub step_id: Option<StepId>,
}
```

Provenance ist **unveränderlich** und Teil des Content-Hash. Sie ermöglicht:
- Nachvollziehbarkeit jedes Datenpunkts bis zur Quelle
- Confidence-basiertes Ranking (höheres Vertrauen bei verifizierten Quellen)
- Modellwechsel-Tracking (welches Modell hat welche Ableitung erzeugt)

### 2.4 Facetten (Lanes)

| Lane | Inhalt | Eigenschaft | Hot/Cold |
|---|---|---|---|
| `meta` | Attribute, Typ, Zeitstempel, Trust, Kanten-Kurzliste | klein, selektiv | **hot** |
| `tokcount` | Token-Anzahl pro registriertem Tokenizer (u32) | Budget-Planung ohne Inhaltszugriff | **hot** |
| `zoom.L2` / `zoom.L1` | Titel / Zusammenfassung | die meisten Recalls nutzen nur diese | **hot** |
| `emb.q` | quantisierter Embedding-Code (z. B. int8/binär, Matryoshka-Präfix) | Kandidatensuche | **hot** |
| `raw` (= `zoom.L0`) | voller Inhalt | groß | cold |
| `tok.ids` | Token-IDs (bitgepackt, optional) | spart Re-Tokenisierung | cold |
| `emb.full` | Embedding in f16/f32 | Re-Ranking | cold |

### 2.5 Zoom-Leiter

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

Stufen können **extraktiv** (Titel, erste Sätze, Schlüsselfelder — Engine-intern, deterministisch) oder **abstraktiv** (LLM-Summary — extern erzeugt, über Work Queue) sein. Fehlt eine Stufe, nimmt der Packer die nächstgröbere/feinere.

### 2.6 Strukturbewusstes Chunking

Pluggable `Chunker`-Trait. Standard-Chunker:

- **Markdown/Prosa:** Überschriften-, Absatz- und Satzgrenzen, Zielgröße in *Token*, nicht in Bytes.
- **Code:** Syntaxbaum-basiert (tree-sitter): Funktion/Klasse/Modul als Einheit, mit Signatur als Zoom-Stufe.
- **JSON/Tabellen:** Pfad-/Zeilenbasiert, Schema als Zoom-Stufe.
- **Binär/groß:** Content-Defined Chunking (FastCDC) für Deduplikation.

Das verwendete Chunking-Schema ist Teil der Namespace-Policy und wird pro Cell als Version vermerkt.

### 2.7 Token-sparsame IDs („Handles")

LLMs sehen IDs im Kontext und müssen sie wiedergeben. UUIDs verschwenden Token und werden falsch abgeschrieben.

- **Handle:** kurzer Alias (4–6 Zeichen, Crockford-Base32), **pro Branch** eindeutig und stabil, abgebildet auf `CellId`.
- Validierung bei jedem Zugriff; bei unbekanntem Handle liefert die Engine **„Meintest du `c:k7qd`, `c:k7qb`?"** (Edit-Distanz-Index über Handles).
- Kollisionsfreie Vergabe über eine Alias-Tabelle (Teil des Branch-Zustands, im `RootSet`).

### 2.8 Bitemporalität

Jede Cell-Version trägt:

- **Transaktionszeit** (`ts_tx`): wann hat Contextra es erfahren (Commit-HLC).
- **Gültigkeitszeit** (`valid_from`, `valid_to`): wann galt es in der Welt.

```rust
pub struct BiTemporal {
    pub ts_tx: Hlc,                      // Hybrid Logical Clock
    pub valid_from: Option<Timestamp>,   // None = seit Erstellung
    pub valid_to: Option<Timestamp>,     // None = aktuell gültig
}
```

Kante `supersedes` verknüpft ersetzte Fakten. Defaults: `recall(valid_at = now)` blendet überholte Fakten aus, `include_history = true` zeigt sie. Zeitreise: `as_of = commit | timestamp`.

---

## 3. Physisches Layout

### 3.1 Objektmodell

- **Chunk:** unveränderliches, content-addressed Objekt (`ChunkId = BLAKE3(kanonische Bytes)`). Zwei Arten: *Index-Knoten* (Prolly-Knoten, Segmentblöcke) und *Lane-Blöcke* (Daten einer Cell-Gruppe).
- **Extent:** physische I/O-Einheit, **256 KiB–1 MiB**, 4 KiB-ausgerichtet, enthält mehrere Chunks.
- **Pack:** Datei aus Extents (256 MiB–1 GiB), append-only während des Schreibens, danach immutable.
- **Locator-Index:** `ChunkId → (PackId, ExtentNo, Offset, Len)`. Als sortierte Runs mit Ribbon-/Bloom-Filter organisiert; Eltern-Knoten tragen physische Hints, sodass der Locator meist übersprungen werden kann.

### 3.2 Extent-Layout: Hot-Head / Cold-Tail

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

**Ergebnis:** Ein typischer Recall mit 200 Kandidaten und 12 Gewinnern verursacht ca. 200 Head-Reads (vom Page Cache meist abgedeckt) und ≤ 12 Tail-Reads.

Innerhalb des Extents sind Lanes **spaltenorientiert** (PAX-artig): SIMD-freundliche Distanzberechnung über `emb.q`, Prädikatsauswertung über `meta`.

### 3.3 Gelerntes Layout (Affinity Re-Layout)

**Beobachtung:** Wer *zusammen gelesen* wird, soll *zusammen liegen*. Das lässt sich nicht aus dem Schlüssel ableiten, wohl aber aus Zugriffsspuren.

**Datenquellen (Access-Trace, asynchron, gesampelt):**
1. *Co-Retrieval:* Cells, die im selben Recall-Ergebnis landeten.
2. *Struktur:* gleiche Episode/Dokument/Thread (starkes Prior).
3. *Zeit:* zeitliche Nähe des Entstehens (schwaches Prior).
4. *Semantik:* gleiche Vektorzelle (IVF-Zentroid) als Fallback bei Kaltstart.

**Verfahren (Hintergrund-Optimizer):**
- Aufbau eines gewichteten Co-Access-Graphen.
- Partitionierung in extent-große Gruppen (Ziel: minimiere erwartete Extent-Zugriffe pro Recall), z. B. mit Multilevel-Partitionierung (METIS-artig) oder inkrementellem Label-Propagation-Clustering.
- Neue Extents werden geschrieben, **nur der Locator-Index wird umgebogen**. ChunkIds, Hashes, Commits bleiben unverändert → Re-Layout ist semantisch unsichtbar und jederzeit abbrechbar.
- **Exploration gegen Rich-get-richer:** ein kleiner Anteil (5 %) der Recalls wird mit Zufalls-/Diversitätsbeimischung bedient.

### 3.4 Kompression

| Lane | Verfahren |
|---|---|
| Text (`raw`, Zoom) | zstd mit **trainiertem Wörterbuch pro Namespace** |
| Token-IDs | Bit-Packing nach Vokabulargröße + Delta/zstd |
| Embeddings | Matryoshka-Präfix (int8/binär) als Head-Code; volle Präzision im Tail; PQ/RaBitQ für große Segmente |
| Attribute | Dictionary-/RLE-/Frame-of-Reference-Codierung (spaltenweise) |
| Kanten | Delta-codierte Adjazenzlisten |

### 3.5 Token-IDs speichern — mit Vorbehalt

Persistierte Token-IDs sparen Re-Tokenisierung, **aber** Token-Grenzen sind kontextabhängig. Daher:

- Cells tragen ein Flag `boundary_safe` (Chunker schneidet an stabilen Stellen).
- Nur bei `boundary_safe` darf die Context Engine Token-IDs direkt konkatenieren; sonst Text konkatenieren und neu tokenisieren.
- Token-Zählungen werden mit kleiner Toleranz (Sicherheitsmarge pro Cell-Grenze) kalkuliert.

---

## 4. Versionierung und Transaktionen

### 4.1 Zustandsmodell: Prolly-Tree + Delta-Log

**Prolly-Tree** (probabilistischer B-Baum): Knotengrenzen werden aus dem Hash der Schlüssel bestimmt. Eigenschaften:

- **History-unabhängig:** gleicher Inhalt ⇒ gleiche Struktur ⇒ gleicher Wurzel-Hash.
- **Strukturelles Sharing:** Forks und Commits teilen alle unveränderten Teilbäume.
- **O(Δ·log n) Diff und Merge** (Hash-Vergleich von oben nach unten).
- **Augmentierbar:** Innere Knoten speichern Aggregate (z. B. **Token-Summen**, Zähler, Min/Max-Zeit) → Order-Statistic-Abfragen ohne Blattzugriff.

**Problem:** Prolly-Trees haben bei Einzelwrites erhöhte Schreibamplifikation. **Lösung:** Schreibpfad mit **Delta-Log** (LSM-artig):

```
Write ──► WAL (Group Commit) ──► Branch-MemDelta (sortiert, im RAM)
                                       │
                           Hintergrund-"Fold" (batched)
                                       ▼
                    neue Prolly-Knoten + Extents ──► Manifest-Eintrag
```

Der Fold bündelt viele Commits zu einem Baum-Update; die Amplifikation wird über Batches amortisiert. Leser sehen immer `Basis-Baum ⊕ MemDelta` (Merge-Iterator).

**Adversarial-Schutz:** Die Boundary-Funktion nutzt einen **Repo-Seed** (keyed). Maximalgröße begrenzt degenerierte Chunks.

**Anfangsparameter (Hypothese):** Ziel-Knotengröße 4 KiB, min 1 KiB, max 16 KiB.

### 4.2 Commit-Objekt

```rust
pub struct Commit {
    pub parents: SmallVec<[CommitId; 2]>,
    pub changes: Digest,          // Hash des Changesets (sofort bekannt)
    pub root: Option<RootSet>,    // Zustandshash nach Fold
    pub author: AgentIdentity,    // Prinzipal + Agent + Run
    pub intent: Intent,           // run_id, step_id, idempotency_key, Freitext-Begründung
    pub ts_tx: Hlc,               // Hybrid Logical Clock
    pub sig: Option<Signature>,   // ed25519, optional
}

/// Wurzel des gesamten versionierten Zustands – inklusive aller Indizes.
pub struct RootSet {
    pub primary: ChunkId,         // Prolly-Wurzel der Cells
    pub sequences: ChunkId,       // geordnete Folgen mit Token-Summen
    pub graph: ChunkId,
    pub fts_segments: Vec<SegmentRef>,
    pub vec_segments: Vec<SegmentRef>,
    pub deletion_vectors: ChunkId,
    pub aliases: ChunkId,         // Handle-Tabelle
    pub policies: ChunkId,        // Namespace-Policies
}
```

**Zweistufige Identität:** `CommitId = H(parents, changes, intent)` ist **sofort** nach dem Commit bekannt. `RootSet`-Hash wird beim Fold berechnet und dient der **Zustandsgleichheit**.

### 4.3 Commit-Protokoll (atomar, crash-sicher)

```
1. begin        → Snapshot (Branch-Head H, Epoche e); Read-Set-Tracking
2. buffer       → Writes im Transaktions-ChangeSet (Cells, Facetten, Kanten)
3. validate     → Branch-Head bewegt? → Rebase-Versuch, wenn Read-/Write-Sets disjunkt
4. WAL append   → Frame {CRC, BLAKE3-Header, Intent, Idempotenz-Key, ChangeSet};
                  Group Commit, Durability nach Transaktionsoption
5. apply        → in Branch-MemDelta + SYNCHRONE Index-Deltas
                  (FTS-Postings, Graph-Kanten, Temporal, Token-Summen, Vektor-Delta-Index)
6. publish      → atomarer Pointer-Swap (ArcSwap) auf neuen BranchState ← Sichtbarkeitspunkt
7. ack          → Client erhält CommitId
8. fold (async) → MemDelta → Prolly-Knoten + Extents → fsync Packs → Manifest-Eintrag
```

**Recovery:** Letzten gültigen Manifest-Eintrag laden → WAL-Frames mit `seq > manifest.wal_seq` wiedergeben → Hashes/CRCs prüfen → torn tail verwerfen.

**Atomarität:** Sichtbarkeit nur über Schritt 6 — Leser sehen ein Commit vollständig (inkl. aller Indizes) oder gar nicht.

### 4.4 Branches, Forks, Tags

- **Fork = O(1):** neuer Ref auf bestehenden Commit + leeres MemDelta. Ziel: < 50 µs.
- **Namenskonvention:** `main`, `agent/<id>/run-<n>/step-<k>`, `spec/<uuid>` (spekulativ).
- **Speculative Branches** haben eine **TTL** und werden automatisch garbage-collected.
- **Checkpoints:** Tag auf Commit pro Plan-Schritt → „Rollback auf Schritt 12" ist ein Ref-Update.
- **Scratch-Branches** (`durability = volatile`) leben nur im RAM, ohne WAL-fsync — für Arbeitsgedächtnis pro Schritt; Promotion per `squash_commit` in durablen Branch.

### 4.5 Isolation und Konflikte

- **Standard: Snapshot-Isolation pro Branch.** Da jeder Agent auf eigenem Branch arbeitet, treten Konflikte erst beim **Merge** auf.
- **Optional serializable** pro Transaktion über Read-Set-Validierung (Bereiche + Bloom).
- **Single Sequencer pro Branch** (Actor-Modell), Branches parallel.

### 4.6 Merge: typisiert und mit „Konflikt als Daten"

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
    pub suggestion: Option<MergeSuggestion>,
}
```

Konflikte sind **abfragbare, versionierte Daten**: ein Agent löst sie per `resolve(conflict_id, choice | merged_value)`. Semantische Konflikte lassen sich so von einem Modell auflösen, ohne dass die Engine selbst ein Modell aufrufen muss.

### 4.7 Durability-Stufen (pro Transaktion)

| Stufe | Bedeutung | Einsatz |
|---|---|---|
| `volatile` | nur RAM, kein WAL | Scratchpad pro Schritt |
| `buffered` | WAL geschrieben, fsync gebündelt (≤ 10 ms) | hochfrequente Beobachtungen |
| `durable` | fsync vor Ack (Group Commit) | Standard |
| `replicated` | Quorum-Ack (ab Phase 4) | kritische Fakten |

### 4.8 Idempotenz (Pflicht für Agenten)

Jede schreibende Operation trägt `(run_id, step_id, idempotency_key)`. Das **Idempotenz-Journal** (kurzlebige Map, versioniert) liefert bei Wiederholung dasselbe Ergebnis und führt nichts doppelt aus. Tool-Retries, Timeouts und doppelte Agentenaufrufe sind damit gefahrlos.

### 4.9 Zeitreise

Granularitäten:

| Stufe | Auflösung | Mechanismus |
|---|---|---|
| G1 | Kernel-Seq | `get_at_seq` + Pins (Kurzzeit) |
| G2 | **Commit** | `ObjectId → RootMap` |
| G3 | **Transaktion innerhalb Commit** | `StateAt(commit, i)` via Journal |
| G4 | **Zelle** | Zeilen-/Spaltendiff, `blame` |

Adressierung: `main`, `main~3`, `<commit>`, `main@{2026-11-21T10:00Z}`, `<commit>:tx[7]`.

**Verzweigen aus der Vergangenheit:** `branch_at(<commit>:tx[i])` erzeugt einen synthetischen Commit → „Was-wäre-wenn"-Simulationen.

---

## 5. Indizes

Alle Indizes sind **segmentbasiert, immutable, content-addressed und im `RootSet` verankert**. Dadurch:
- Forks teilen Segmente ohne Kopie.
- Atomare Sichtbarkeit mit den Primärdaten.
- Zeitreise liefert konsistente Indexstände.

### 5.1 Vektorindex

- **Segmente:** je Segment ein Graphindex (Vamana/DiskANN-artig, Disk-optimiert) mit **Quantisierungscodes im Hot Head** und Re-Ranking aus dem Cold Tail.
- **Delta-Index:** neue Vektoren eines Branches landen in einem kleinen **Flat-/Brute-Force-Index** (SIMD, bis ca. 50k Vektoren), der bei jeder Suche mit durchsucht wird → **kein Index-Lag**.
- **Merge:** Hintergrund-Compaction (size-tiered) erzeugt neue Segmente; alte werden über GC entfernt.
- **Löschungen:** versionierte **Deletion Vectors** (Roaring-Bitmaps, content-addressed) je Segment.
- **Gefilterte Suche:** Prädikate (Namespace, Typ, Zeit, Trust) werden zu Bitmaps aus Attributindizes kompiliert; bei hoher Selektivität Wechsel auf Brute-Force über die Treffermenge (ACORN-inspiriert).
- **Mehrere Embedder/Versionen** pro Namespace; Modellwechsel = neue Lane + Re-Embedding über Work Queue, alte Lane bleibt bis zur Migration lesbar.
- **Co-Location:** Vamana-Graphknoten werden physisch nah an ihren Embedding-Daten platziert, damit Graph-Traversal und Distanzberechnung in wenigen I/Os möglich sind.

### 5.2 Volltext (FTS)

Segmentbasierter invertierter Index mit BM25, optionalen Positionen (Phrasen), Feld-Gewichten und Block-Max-WAND für effizientes Top-k-Pruning. Eigene Tokenisierung für Suche, **unabhängig** von LLM-Tokenizern.

- **FST (Finite State Transducer)** für Term-Dictionary mit Fuzzy-Matching und Präfix-Suche.
- **Feldbezogene Gewichtung:** Titel-Treffer > Body-Treffer > Attribut-Treffer.

### 5.3 Graph

Adjazenz als Prolly-Tree-Schlüssel `(src, edge_type, dst)` plus Rückrichtung `(dst, edge_type, src)`. Operationen:

- Begrenzte k-Hop-Expansion mit **Fan-out-Budget** (konfigurierbar).
- **Personalized-PageRank-Approximation** für assoziatives Abrufen (Entity → verknüpfte Episoden).
- **CSR-Segmente** (Compressed Sparse Row) als abgeleitete Indexstruktur für schnelle Traversal auf großen Graphen.

### 5.4 Temporal- und Attributindizes

- Zeitbereichs-Index (`valid_from/valid_to`, `ts_tx`) für Recency-Scoring und temporale Filter.
- Invertierte Attributindizes mit Roaring-Bitmaps; Basis für Vorfilter.
- **Enum-/Tag-Indizes** für häufige Filterdimensionen (CellKind, Trust, Namespace).

### 5.5 Token-Summen (augmentierter Baum)

Für **Sequenzen** (Dokument → geordnete Kind-Cells, Episode → Turns) speichert jeder innere Knoten die **Summe der Token** seines Teilbaums.

```rust
/// Liefert eine zusammenhängende Menge von Cells um `anchor` mit Summe ≤ budget,
/// ohne Inhalte zu lesen (nur Baum-Aggregate).
fn window(seq: SeqId, anchor: CellId, budget: Tokens, tk: TokenizerId) -> Vec<CellId>;
```

Damit sind „Kontextfenster um einen Treffer" und „Turns 40–60 unter 3 000 Token" **O(log n)** in Knoten-Reads.

---

## 6. Context Engine

### 6.1 Primitive Operationen

| Operation | Zweck |
|---|---|
| `recall(q, budget, policy)` | beste Kontextmenge unter Token-Budget |
| `window(seq, anchor, budget)` | zusammenhängendes Fenster |
| `expand(handle, level)` | feinere/gröbere Stufe einer Cell nachladen |
| `explain(recall_id)` | warum wurde was gewählt (Scores, Quellen, Verworfenes) |
| `neighbors(handle, edge_types, fanout)` | Graph-Nachbarschaft |
| `diff(a, b)` | menschen- und LLM-lesbare Änderungszusammenfassung |

### 6.2 Recall-Anfrage (Beispiel)

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
            .render(Render::Compact)
            .order(Order::CacheFriendly),
    )
    .await?;
```

Antwort (Compact-Rendering):

```
[c:k7qd] (L1, 142 tok, trust=verified, 2026-10-14)
Deployment schlug fehl wegen abgelaufenem Registry-Token; Lösung: …
[c:m2xa] (L0, 480 tok, trust=user, 2026-10-14)
<voller Text der Cell>
[c:p9hv] (L2, 18 tok) Rollback-Runbook v3 – Titel
… 11 weitere Handles verfügbar via expand(); Budget genutzt: 5 874 / 6 000
```

### 6.3 Recall-Pipeline

```
Query ──► Kandidaten ──► Score-Fusion ──► Zoom-Knapsack ──► Ordnung ──► Rendering
          (Vektor, BM25,  (RRF/gewichtet,  (Level je Treffer   (Cache-    (Compact/
           Graph, Recency)  + Trust/Recency) unter Budget)      freundlich) XML/JSON)
```

1. **Kandidatengenerierung:** parallel Vektor (Head-Codes), BM25, Graph-Expansion, zeitlicher Prior. Ranking ausschließlich auf **Hot-Head-Lanes**.
2. **Score-Fusion:** Reciprocal Rank Fusion oder gelernte Gewichte; Trust-/Zeit-Faktoren multiplikativ. Kalibration über isotonische Regression (siehe Abschnitt 14).
3. **Zoom-Knapsack:** siehe 6.4.
4. **Ordnung:** siehe 6.5.
5. **Rendering:** Formatierer minimieren Tokenoverhead (Compact ≪ JSON); Provenienz und Handles bleiben erhalten.

### 6.4 Zoom-Knapsack

**Problem:** Gegeben Kandidaten *i* mit Stufen *ℓ* (Kosten `c[i][ℓ]` in Token, Nutzen `u[i][ℓ] = rel_i · gain(ℓ) · trust_i · fresh_i`), wähle genau eine Stufe pro Kandidat (oder „weglassen") so, dass die Summe der Kosten ≤ Budget und der Gesamtnutzen — unter Redundanzabzug (MMR) — maximal ist. Das ist ein **Multiple-Choice-Knapsack**.

**Praxisverfahren:**
1. Pro Kandidat die obere konvexe Hülle der (Kosten, Nutzen)-Punkte bilden.
2. Start: alle Kandidaten auf gröbster Stufe (Handle/Titel).
3. Greedy über Upgrade-Schritte mit größtem **Δ-Nutzen / Δ-Kosten** (Max-Heap), bis das Budget erschöpft ist. Auf der konvexen Hülle ist das die optimale LP-Relaxation.
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

**Vorteil gegenüber Top-k:** Viele *grobe* Treffer + wenige *feine* statt wenige vollständige — der Agent bekommt **Überblick und Tiefe** im selben Budget und kann per `expand` nachsteuern.

### 6.5 Cache-freundliche Ordnung

Inferenz-Server cachen KV-Zustände für gemeinsame Präfixe. Contextra ordnet die Ausgabe, wenn gewünscht:

1. **Stabile Cells zuerst** (System-/Policy-/Skill-Cells mit unveränderlichen Hashes), in kanonischer Reihenfolge (nach ChunkId) → identische Präfixe über Aufrufe hinweg.
2. Volatiles (frische Episoden, Query-spezifische Treffer) danach.
3. Rückgabe enthält einen **Prefix-Digest** (Hash der ersten N Token-Blöcke), den der Aufrufer für Cache-Keys nutzen kann.

### 6.6 Progressive Disclosure

Das Standardmuster für Agenten: `recall` liefert **kompakt mit Handles** → Agent entscheidet → `expand(handle, L0)` nur für wirklich relevante Cells. Die Engine zählt Expand-Aufrufe als **Relevanzsignal** (Access-Trace), das das Ranking und das Layout verbessert.

### 6.7 Explainability

`explain(recall_id)` liefert pro Kandidat: Signalbeiträge (vec/bm25/graph/recency), gewählte Stufe, Gründe für Verwerfen (Budget, Redundanz, Trust, Filter). Wichtig für Debugging und Evaluation.

### 6.8 Evidence Sufficiency

Die Recall-Antwort enthält optional ein **Sufficiency-Signal** (`evidence_quality: f32`), das die Engine aus Score-Verteilung, Treffer-Diversität und Budget-Auslastung schätzt. Ein Agent kann damit entscheiden, ob die Evidenz für eine Antwort ausreicht oder ob er nachfragen/nachladen muss.

---

## 7. Abgeleitete Daten und Lineage

### 7.1 Derivation Cache

Abgeleitete Facetten werden unter dem Schlüssel `(source_chunk_hash, producer_id)` **memoisiert**:

- **Embedding-Cache:** identischer Text → identischer Hash → Embedding wird nie zweimal berechnet; Forks und Branches teilen den Cache.
- **Token-Zählungen** und **extraktive Zoom-Stufen** analog.
- **Tenant-Grenzen:** Cache ist **pro Tenant/Repository** (kein Cross-Tenant-Dedup), um Seitenkanal-Leaks auszuschließen.

### 7.2 Staleness und Lineage

Jede abgeleitete Facette kennt `source` (Hash). Ändert sich der Quellinhalt, ändert sich der Hash ⇒ die Facette ist **per Konstruktion erkennbar veraltet** (keine Invalidierungsnachrichten nötig). Die Engine:

- markiert veraltete Facetten, nimmt sie je nach Policy aus Recall heraus oder liefert sie mit Warnung,
- stellt Neuberechnung in die **Work Queue**.

Ein Dependency-DAG (`derived_from`-Kanten) erlaubt **inkrementelle Neuberechnung** in Ketten (L0 → L1-Summary → Entity-Profil → Themen-Summary).

### 7.3 Work Queue und externe Worker

Die Engine ruft **kein** Modell auf. Stattdessen:

```rust
pub enum Task {
    Embed       { cells: Vec<CellId>, producer: ProducerId },
    Summarize   { cluster: Vec<CellId>, target_level: u8, max_tokens: u32 },
    Consolidate { episode: SeqId, into: NsId },        // Episoden → semantische Fakten
    Dedupe      { candidates: Vec<(CellId, CellId)> },  // Duplikate/Widersprüche prüfen
    Reconcile   { conflict: ConflictId },
}

let tasks = db.work().lease(worker_id, kinds, limit, lease_ttl).await?;
// … externer Worker (LLM-Agent) bearbeitet …
db.work().complete(task_id, result_changeset).await?;
```

Ergebnisse sind normale Commits mit `derived_by = (modell, prompt_hash)` — vollständig nachvollziehbar.

**Leasing-Modell:** At-least-once-Delivery mit TTL-basiertem Lease. Idempotenz-Keys stellen sicher, dass doppelte Completions harmlos sind.

### 7.4 Vergessen als „Zoom-Down"

Retention-Policies sind **gestuft statt binär**:

1. *Hot → Warm:* Cold-Tail-Lanes auf billigeres Tier verschieben.
2. *Zoom-Down:* L0 (Volltext) entfernen, L1/L2 behalten (Original in Archiv-Tier).
3. *Tombstone:* nur Handle + Provenienz + `supersedes`-Spur.

**Salience-Score** = f(Recency, Zugriffshäufigkeit, expliziter Wichtigkeit, Graph-Zentralität, Expand-Rate). Verworfen wird, was niedrig bleibt — mit Schutz für explizit „pinned" Cells.

### 7.5 Konsolidierung

Hintergrundprozess, der episodische Daten zu semantischen Fakten verdichtet:

1. **Episode → Fakten:** Extrahiert aus Konversationsturns persistente Fakten/Erkenntnisse.
2. **Duplikaterkennung:** Identifiziert semantisch äquivalente Cells über Embedding-Ähnlichkeit und erzeugt `Dedupe`-Tasks.
3. **Widerspruchserkennung:** Findet widersprüchliche Fakten über Embedding-Distanz + Attribut-Überlappung und erzeugt `Reconcile`-Tasks.
4. **Profil-Aggregation:** Verdichtet Einzelfakten zu Entity-Profilen (z. B. alle bekannten Attribute einer Person).

Alle Schritte werden über die Work Queue orchestriert und sind damit extern, nachvollziehbar und modellneutral.

---

## 8. Sicherheit und Vertrauen

### 8.1 Trust-/Taint-Labels

```
System > Principal(User) > Tool-Verified > Tool-Unverified > External-Untrusted
```

- Jede Cell trägt ein Label; **abgeleitete Daten erhalten das Minimum** ihrer Quellen.
- `recall` kann `min_trust` filtern oder untrusted Inhalte **markiert und eingerahmt** ausgeben (Delimiter + Quellenvermerk).
- **Ehrliche Einordnung:** Der Storage kann Prompt Injection nicht *verhindern*, aber **verlässliche Herkunftsmetadaten** liefern, auf denen Gegenmaßnahmen aufsetzen. Trust-Labels sind nur durch privilegierte Prinzipale **hochstufbar** (mit Audit-Eintrag).

### 8.2 Zugriffskontrolle: Capabilities

Macaroon-artige, **attenuierbare Tokens** (Caveats: Namespaces, Branches, Operationen, Trust-Obergrenze, TTL, Token-Budget pro Zeitraum). Ein Hauptagent kann Subagenten Tokens mit **eingeschränkten Rechten** ausstellen, ohne den Server zu fragen.

```rust
pub struct Capability {
    pub id: CapabilityId,
    pub caveats: Vec<Caveat>,
    pub hmac: [u8; 32],
}

pub enum Caveat {
    Namespaces(Vec<NsId>),
    Branches(Vec<BranchPattern>),
    Operations(OpSet),           // read | write | fork | merge | admin
    MaxTrust(TrustLabel),
    Ttl(Duration),
    TokenBudget { per_period: Tokens, period: Duration },
    ReadOnly,
}
```

### 8.3 Verschlüsselung (At-Rest)

- **Envelope-Encryption:** Daten-Extents mit AES-256-GCM oder XChaCha20-Poly1305.
- **Schlüsselhierarchie:** Master-Key → Repo-Key → Pack-Keys.
- **Optional pro Namespace:** `encryption = "aes256gcm"` in der Namespace-Policy.
- **Keyed ObjectIds:** für verschlüsselte Repos wird der Content-Hash über den verschlüsselten Inhalt gebildet → kein Existenz-Orakel.

### 8.4 Audit und Integrität

- Die Commit-Kette ist ein **Merkle-DAG** → manipulationsevident; optionale ed25519-Signaturen.
- Lesezugriffe werden (gesampelt oder vollständig, konfigurierbar) im Access-Trace protokolliert, getrennt vom Inhalts-Zustand.
- Reflog ist verkettet (`prev_hash`), Manipulationserkennung möglich.

---

## 9. API-Oberflächen

### 9.1 Rust-API (eingebettet, Referenz)

```rust
let db = Contextra::open(Config::at("/var/lib/contextra"))?;

let main    = db.branch("main")?;
let attempt = main.fork("agent/a1/run-42/step-7")?;           // O(1)

let mut tx = attempt.begin(
    Tx::new()
        .intent("Fehlerursache aus Logs als Fakt ablegen")
        .idempotency_key("run-42/step-7/put-fact-1")
        .durability(Durability::Durable),
);
tx.put(Cell::fact("Registry-Token war abgelaufen")
        .trust(Trust::ToolVerified)
        .edge(Edge::derived_from(log_cell)))?;
let commit = tx.commit().await?;

let ctx   = attempt.recall(Recall::new("Registry Token").budget(Tokens(3_000))).await?;
let merge = main.merge(&attempt, MergePolicy::from_namespace_defaults()).await?;
match merge {
    Merged(c)       => { /* fertig */ }
    Conflicts(list) => { /* ConflictCells abfragen und auflösen */ }
}
```

### 9.2 MCP-Server (Model Context Protocol)

Wenige, mächtige Tools (spart Tool-Beschreibungs-Token):

| Tool | Zweck |
|---|---|
| `mem_recall` | Recall unter Budget, Compact-Rendering, Handles |
| `mem_expand` | Handle → feinere/gröbere Stufe |
| `mem_write` | Cells ablegen (idempotent), optional auf Branch |
| `mem_fork` / `mem_merge` / `mem_discard` | Versuch-Workflow |
| `mem_history` / `mem_diff` | Zeitreise und Änderungsübersicht |
| `mem_describe` | Schema/Namespaces/Policies — **budgetiert** |

Tool-Schemas werden **aus den Rust-Typen generiert** (`schemars`/JSON-Schema).

### 9.3 LLM-lesbare Fehler

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

### 9.4 Selbstbeschreibung (`mem_describe`)

Liefert Namespaces, Cell-Typen, Attribute, Merge-Regeln und Beispiele **in einem Token-Budget** (z. B. 800 Token) im Compact-Format. So lernt ein Agent die Datenbank kennen, ohne Dokumentation vorauszusetzen.

### 9.5 SQL-Zugang (SQLite Virtual Tables)

SQLite als Frontend (der Dialekt, den LLMs am zuverlässigsten generieren):

- **SQL `BEGIN…COMMIT`** = eine Transaktion im Journal. Versions-Commit ist ein eigener Schritt: `SELECT vcs_commit('Nachricht', json_metadata)`.
- **Hidden Columns** für Zeitreise: `SELECT * FROM notes WHERE _asof = 'main~3'`.
- **Systemtabellen:** `vcs_log`, `vcs_branches`, `vcs_tags`, `vcs_diff(a, b, table)`, `vcs_blame(table, pk)`, `vcs_conflicts`, `vcs_merge(a, b)`.
- **Constraints erzwingt `contextra-table`**, nicht SQLite.

### 9.6 Weitere Frontends

- **C-ABI/Python/Node** (`contextra-ffi`, `contextra-py`).
- **CLI `ctx`:** Inspektion, Diff, Dump, Verify, Bench.
- **gRPC/HTTP** für Service-Integration.

---

## 10. Crate-Architektur

### 10.1 Architekturüberblick

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

### 10.2 Crate-Layout (Cargo Workspace)

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

### 10.3 Abhängigkeitsregeln

1. **Abhängigkeiten zeigen nur nach unten.** Kein Crate importiert ein höheres.
2. **`#![deny(unsafe_code)]`** workspace-weit in `Cargo.toml`. `unsafe` erlaubt nur in: `contextra-simd`, `contextra-sys`, `contextra-infer-onnx`, `contextra-infer-candle`, `contextra-py`.
3. **Jede Schicht hängt an Traits** der darunterliegenden, nicht an deren konkreten Strukturen → Fakes (In-Memory-Kernel, `MemVfs`, `ManualClock`) überall möglich.
4. **Rust Edition 2021, stable Toolchain** (≥ 1.89).
5. **Serialisierung:** `rkyv`/`bytemuck` für Extent-Verzeichnisse und Lanes (Zero-Copy). Alle On-Disk-Formate besitzen Magic, Versionsnummer, Feature-Flags.
6. **Bibliotheken-Policy:** Eigenbau dort, wo atomare Integration mit dem Commit-Modell nötig ist. Fremdcrates für Krypto, Kompression, Parser.

### 10.4 Seam-Traits

```rust
// Kernel-Fassade
pub trait KvStore: Send + Sync {
    type Snap: KvSnapshot;
    fn snapshot(&self) -> Self::Snap;
    fn apply(&self, batch: WriteBatch) -> Result<CommitInfo, ApplyError>;
    fn pin(&self, seq: Seq, why: PinReason) -> PinGuard;
    fn subscribe(&self, l: Arc<dyn CommitListener>);
}

// Objektspeicher
pub trait ObjectStore: Send + Sync {
    fn put(&self, kind: ObjKind, bytes: &[u8]) -> Result<ObjectId>;  // idempotent
    fn get(&self, id: &ObjectId) -> Result<Bytes>;
    fn has(&self, id: &ObjectId) -> Result<bool>;
    fn flush_durable(&self) -> Result<DurableMark>;
}

// Baum-Engine
pub trait TreeEngine {
    fn get(&self, root: &ObjectId, k: &[u8]) -> Result<Option<Bytes>>;
    fn range(&self, root: &ObjectId, r: KeyRange, dir: Dir) -> Box<dyn Iterator<Item=Result<(Bytes,Bytes)>>>;
    fn apply(&self, root: &ObjectId, muts: &mut dyn Iterator<Item=Mutation>) -> Result<ObjectId>;
    fn diff(&self, a: &ObjectId, b: &ObjectId, r: KeyRange) -> Box<dyn Iterator<Item=Result<DiffEntry>>>;
    fn merge3(&self, base: &ObjectId, ours: &ObjectId, theirs: &ObjectId, res: &dyn MergeResolver) -> Result<MergeOutcome>;
}

// Erweiterungspunkte
pub trait MergeResolver { fn resolve(&self, c: &Conflict) -> Resolution; }
pub trait CommitHook { fn pre_commit(&self, c: &CommitDraft) -> Result<(), HookRejection>; }
pub trait AccessPolicy { fn check(&self, p: &Principal, a: &Action, t: &Target) -> Result<(), Denied>; }
pub trait DerivedIndex { fn id(&self) -> &str; fn update(&self, prev: Option<&ObjectId>, next: &ObjectId, diff: &mut dyn DiffStream) -> Result<()>; }
```

---

## 11. Nebenläufigkeitsmodell

### 11.1 Lese-Pfad

**Wartefrei/lock-arm:** Immutable Strukturen + `ArcSwap` für `BranchState`, epochenbasierte Rückgewinnung (`crossbeam-epoch`) für Caches und Locator-Runs. Leser blockieren nie Schreiber und umgekehrt.

### 11.2 Schreib-Pfad

**Ein Sequencer-Actor pro Branch.** Group Commit bündelt WAL-Writes über Branches. Der Sequencer serialisiert nur die Commit-Validierung und den Pointer-Swap; der eigentliche Schreibpfad (Objekt-Serialisierung, WAL-Append) ist parallel.

### 11.3 I/O-Pfad

- **Linux:** `io_uring` (Thread-per-Core-Executor, z. B. `monoio`/`glommio`/`io-uring`).
- **Fallback:** `pread/pwrite`-Threadpool (macOS/Windows).
- **API-Schicht:** `tokio`, verbunden über Kanäle.
- **O_DIRECT** für Large-Sequential-Reads (Cold-Tail-Zugriff, Pack-Compaction).

### 11.4 Hintergrundjobs

Fold, Compaction, Re-Layout, GC, Work-Queue-Dispatch laufen mit **Prioritäts- und Bandbreitenbudgets** (Token-Bucket), damit sie Recall-Latenzen nicht verdrängen.

### 11.5 JobRunner und ResourceGovernor

- **`JobRunner`-Port:** Committer als Single-Flight-Job statt eigener Thread je Instanz.
- **`ResourceGovernor`:** Cache-, Memtable-, I/O-Budget über viele Repository-Instanzen teilbar.
- Vermeidet Thread-Explosion bei vielen gleichzeitigen Repositories.

---

## 12. Durability, Recovery, Garbage Collection

### 12.1 WAL und Group Commit

- **WAL-Frame:** `{CRC32c, BLAKE3-Header, Seq, Intent, Idempotenz-Key, ChangeSet}`.
- **Group Commit:** Bündelt WAL-Writes über Branches; ein fsync für viele Commits.
- **Manifest-Log:** Append-only, CRC-geschützt, zeigt auf gültigen Zustand (Pack-Liste, WAL-Seq).

### 12.2 Crash-Recovery

```
1. Letzten gültigen Manifest-Eintrag laden
2. WAL-Frames mit seq > manifest.wal_seq wiedergeben
3. Hashes/CRCs prüfen, torn tail verwerfen
4. Branch-MemDeltas rekonstruieren
5. Abgeleitete Indizes über diff nachholen
```

**Ziel:** Recovery < 5 s bei 1 GiB WAL.

**Invarianten:**
- **INV-1:** Ein Ref/Index-Eintrag wird nie geschrieben, bevor das Pack-Objekt durabel ist.
- **INV-2:** Pack-Inhalt ist Quelle der Wahrheit für Objektexistenz; Index ist rekonstruierbar.
- **INV-3:** Fehler bei Pack-`fsync` ⇒ Pack und Instanz vergiftet (Fail-Stop), kein Retry.
- **INV-4:** Commits sind unveränderlich (neue History = neue Commits).

### 12.3 Garbage Collection

**Generationelles Mark-and-Sweep** mit SATB+Remark-Protokoll für gleichzeitige Schreiber:

1. **Wurzeln:** alle Refs, Reflog-Fenster, Sitzungs-Basen, Pins, In-Flight-Objekte.
2. **Mark** ab Kernel-Snapshot `S0`.
3. **Candidates:** unmarkierte Objekte aus Packs älter als `S0`.
4. **Remark:** alle Ref-/Sitzungsänderungen seit `S0` nachmarkieren.
5. **Dedup-Falle:** Schreiber, der Objekt per Dedup wiederverwendet, das `sweep_candidate` ist, schreibt es erneut oder löscht Markierung.
6. **Physisches Löschen:** nur für Packs mit Live-Anteil < Schwelle (Copy-Forward), nach Grace-Periode.

**Extent-Compaction** bei Live-Anteil < 50 %, kombiniert mit Re-Layout — ein Durchlauf, zwei Zwecke.

### 12.4 Tiering

| Tier | Inhalt | Medium |
|---|---|---|
| T0 | MemDelta, Locator-Hot-Set, Head-Page-Cache, Alias-Tabellen | RAM |
| T1 | Packs mit Hot Heads + aktuelle Cold Tails | NVMe |
| T2 | Kalte Tails, Archiv-Segmente, alte Versionen | Objektspeicher (S3-kompatibel) |

Entscheidung über **Kosten-Nutzen-Score** (Zugriffsfrequenz × Latenzvorteil ÷ Platzkosten).

### 12.5 Retention

`RetentionPolicy` je Ref: `KeepAll | KeepLast(N) | KeepDays(D)` plus `undo_window`. Shallow-Kappung ersetzt alte Commits durch Snapshot-Commits.

### 12.6 Replikation und Sync

Da alles content-addressed ist, entspricht Sync einem **Git-artigen Fetch/Push**: Austausch von Commit-Frontiers, Aushandeln fehlender Chunks, Übertragen als Packs.

- **Multi-Device-Memory** (Laptop ↔ Cloud) ohne Sonderlogik.
- **Federated Memory** (Teams teilen Branches/Namespaces).
- **Read-Replicas** (Pull-basiert) und später **Quorum-Replikation**.

---

## 13. Ingest-Pipeline

### 13.1 Überblick

Contextra enthält ein umfassendes Ingest-System (`contextra-ingest`), das Dokumente aus diversen Formaten in Cells überführt:

```
Quelldokument ──► Format-Erkennung ──► Parser ──► Chunker ──► Cell-Erzeugung ──► Commit
                                                      │
                                              Work Queue (Embedding, Summary)
```

### 13.2 Unterstützte Formate

| Kategorie | Formate |
|---|---|
| Text | Markdown, Plain Text, RST, AsciiDoc |
| Office | PDF, DOCX, XLSX, PPTX, ODT |
| Code | Rust, Python, TypeScript, Go, C/C++, Java, etc. (tree-sitter) |
| Daten | JSON, JSONL, CSV, TSV, YAML, TOML, XML |
| Web | HTML, EPUB |
| Binär | FastCDC für generische Binärdaten |

### 13.3 Chunking-Strategien

Zielgröße in **Token** (nicht Bytes), respektiert strukturelle Grenzen:

- **Strukturbewusst:** Überschriften, Absätze, Sätze, Code-Funktionen als Grenzen.
- **Overlap-Optionen:** konfigurierbarer Überlappungsbereich für Kontext-Kontinuität.
- **Hierarchisches Chunking:** erzeugt automatisch Parent-Child-Beziehungen (Dokument → Abschnitt → Absatz) als Kanten.

### 13.4 Automatische Facetten-Erzeugung

Bei Ingest werden automatisch erzeugt:
- **Extraktive Zoom-Stufen** (Titel, erste Sätze) — deterministisch, in der Engine.
- **Token-Zählungen** für alle registrierten Tokenizer.
- **Embedding-Tasks** und **Summary-Tasks** in der Work Queue.
- **Kantenerzeugung:** `follows`-Kanten für sequentielle Chunks, `part_of` für Hierarchie.

---

## 14. Adaptive Ranking und Fusion

### 14.1 Score-Fusion

Hybrides Retrieval kombiniert Signale aus vier Retrieval-Pfaden. Die Fusion nutzt **Reciprocal Rank Fusion (RRF)** als robusten Baseline, mit optionaler gelernter Gewichtung:

```rust
pub struct FusionConfig {
    pub method: FusionMethod,       // RRF | Weighted | Learned
    pub weights: SignalWeights,     // vec, bm25, graph, recency, trust
    pub rrf_k: f32,                 // RRF-Parameter (default: 60.0)
}

pub struct SignalWeights {
    pub vector: f32,
    pub bm25: f32,
    pub graph: f32,
    pub recency: f32,
    pub trust: f32,
}
```

### 14.2 Kalibration

**Isotonische Regression** für Score-Kalibration: Scores aus verschiedenen Retrieval-Pfaden werden auf eine gemeinsame Skala kalibriert, damit Gewichtung sinnvoll ist.

### 14.3 Recency-Scoring

Exponentieller Zerfall mit konfigurierbarer Halbwertszeit pro Namespace:
```
recency_score = exp(-λ · age_days)
λ = ln(2) / half_life_days
```

### 14.4 Drift-Detection (`contextra-adapt`)

- **Embedding-Drift:** erkennt, wenn die Verteilung der Anfrage-Embeddings sich verschiebt.
- **Relevanz-Drift:** erkennt, wenn Expand-Raten sinken (Signal für nachlassende Retrieval-Qualität).
- **Automatische Anpassung:** passt Fusion-Gewichte und Recency-Parameter adaptiv an, basierend auf Access-Trace-Feedback.

---

## 15. KV-Cache-Store (optional)

*(Bewusst vom Kern getrennt, da modellabhängig und hardwarenah.)*

### 15.1 Motivation

Kontexte werden aus wiederkehrenden Bausteinen zusammengesetzt. Das Berechnen der KV-Zustände ist teuer und wiederholt sich.

### 15.2 Konzept

- **Schlüssel:** `(model_id, tokenizer_id, hash-Kette über Token-Blöcke)` — Merkle-artig, Präfixe teilen gemeinsame Blöcke (Radix-Struktur).
- **Blockgröße:** an Inferenz-Server angepasst (16–256 Token pro Block).
- **Speicherung:** eigene Pack-Art auf NVMe, Tiering RAM ↔ NVMe ↔ Objektspeicher.
- **Eviction:** **kostenbasiert** (`Wiederberechnungskosten = Token × Modellkosten × Nutzungsfrequenz`), nicht reines LRU.
- **Anbindung:** Konnektoren zu Inferenz-Stacks; Contextra liefert Prefix-Digest (6.5) und verwaltet Blöcke.
- **Invalidierung:** automatisch über Hash-Kette.

---

## 16. Performance-Ziele und KPIs

### 16.1 Design-Ziele (Hypothesen, zu validieren)

| Metrik | Ziel |
|---|---|
| Fork | < 50 µs, Speicher O(1) |
| Commit (1–100 Cells, NVMe, Group Commit, `durable`) | p99 < 2 ms |
| Recall (1 M Cells, warm, 8k-Budget, hybrid, 3 Namespaces) | p50 < 5 ms, p99 < 25 ms |
| Extent-Zugriffe pro Recall nach Re-Layout | ≤ 4 im Mittel |
| `window()` über Sequenz mit 10⁶ Turns | < 1 ms ohne Inhaltszugriff |
| Schreibamplifikation (Fold) | < 5× bei Batchgröße ≥ 1 000 Cells |
| Recovery-Zeit nach Crash | < 5 s bei 1 GiB WAL |
| Dedup-/Sharing-Gewinn auf realen Traces | deutlich messbar (Baseline erheben) |

### 16.2 Qualitätsmetriken (entscheidend)

Reine Latenz reicht nicht. **Utility@Budget** ist die zentrale Größe:

1. **Trace-Replay:** Aufzeichnung realer Agentenläufe. Wiedergabe gegen Contextra vs. Baselines (Top-k fester Chunk-Größe; RocksDB+externe Vektor-DB).
2. **Metriken:**
   - **Recall@Budget:** Anteil relevanter Cells im Ergebnis bei gegebenem Token-Budget.
   - **Nutzen-pro-Token:** Anteil der geladenen Token, die der Agent tatsächlich verwendet/zitiert.
   - **Task-Erfolgsrate** bei gleichem Budget.
   - **Expand-Rate:** wie oft der Agent nachlädt (niedriger = bessere Erstauswahl).
   - **Kosten pro gelöster Aufgabe.**
3. **A/B-Betrieb:** Zoom-Knapsack vs. Top-k, Cache-freundliche vs. relevanzbasierte Ordnung.
4. **Layout-Evaluation:** Extent-Zugriffe pro Recall vor/nach Re-Layout; Tail-Latenz.

### 16.3 Benchmarks

- **Mikro-Benchmarks** (`criterion`) pro Crate.
- **Systembenchmarks** mit synthetischen und aufgezeichneten Workloads.
- **Regression-Gates** in CI (Latenz **und** Qualität).
- **Vergleichs-Benchmarks** gegen SQLite, Dolt, LanceDB für vergleichbare Operationen.

---

## 17. Teststrategie

### 17.1 Deterministische Simulation

**FoundationDB/TigerBeetle-Stil:** Gesamte Engine gegen simulierte Zeit, simuliertes Dateisystem (torn writes, verlorene fsyncs, Bit-Flips, volle Platte) und simuliertes Netz; Seeds reproduzierbar.

### 17.2 Modellbasierte Property-Tests

`proptest` gegen ein triviales In-Memory-Referenzmodell (Branches, Merge, Time-Travel, bitemporale Abfragen).

**Gesetze:**
- Historienunabhängigkeit: `gleicher Inhalt ⇒ gleicher Hash`
- `apply(diff(a, b), a) = b`
- `diff(a, a) = ∅`
- `merge3(b, a, a) = a`
- `merge3(b, x, b) = x`
- Rang/`COUNT` stimmt mit Iteration überein

### 17.3 Crash-Matrix

`FaultVfs` bricht an **jedem** I/O-Schritt ab; Erwartung: Zustand = Präfix der bestätigten Commits, `verify` grün.

### 17.4 GC-Simulation

Deterministische Interleavings (Schreiber, Dedup, GC, Abstürze). Property: „kein erreichbares Objekt geht verloren" (≥ 10⁶ Schritte).

### 17.5 Weitere Tests

| Netz | Inhalt |
|---|---|
| **Invarianten-Checker (`ctx verify`)** | Hashbaum, Locator-Konsistenz, Index↔Primärdaten, Lineage, GC-Erreichbarkeit |
| **`loom`** | Lock-freie Strukturen |
| **Miri** | `unsafe`-Crates |
| **Fuzzing** | Alle Decoder (Objekt, Knoten, Commit, Journal, Lane-Codecs) |
| **Differential** | Gleiche SQL gegen SQLite-Nativtabellen und `contextra-sqlite` |
| **Format-Goldens** | Referenzdateien je Schicht; ID-Stabilität über Versionen |
| **Soak/Chaos** | Langläufer mit Agenten-Workload, Absturz-Injektion, Speicherdruck |
| **Agenten-Tests** | Golden-Transkripte typischer Werkzeugaufrufe (Konflikt, Retry, Zeitreise) |
| **Retrieval-Qualität** | CI-Gate: Utility@Budget darf nicht regressieren |
| **Supply Chain** | `cargo deny` (Lizenzen, Advisories) |

---

## 18. Roadmap

| Phase | Inhalt | Exit-Kriterium |
|---|---|---|
| **0 — Spike (6–8 Wochen)** | Format-Spezifikation Entwurf; Prototyp: CAS + WAL + Prolly-Tree + Fork/Commit; Simulationstest-Gerüst; Trace-Recorder | Fork/Commit/Recovery korrekt unter Simulation; Baselines erhoben |
| **1 — MVP Kern (3–4 Monate)** | Cells/Lanes, Hot-Head/Cold-Tail-Extents, Branches/Merge (typisiert), Idempotenz, FTS, Flat-Vektor-Delta, Token-Summen, Rust-API + CLI + `verify` | Alle Invarianten grün; Replay-Evaluation vs. Baseline |
| **2 — Context Engine + MCP (2–3 Monate)** | Segment-Vektorindex (Disk), Hybrid-Fusion, Zoom-Knapsack, Handles/Expand/Explain, MCP-Server, LLM-Fehler | Utility@Budget signifikant über Top-k-Baseline |
| **3 — Lernen & Lebenszyklus (3 Monate)** | Access-Trace, Affinity Re-Layout, Derivation Cache, Work Queue, Retention/Zoom-Down, Trust-Labels, Capabilities | Extent-Zugriffe/Recall messbar reduziert |
| **4 — Verteilung (3–4 Monate)** | Fetch/Push-Sync, Read-Replicas, Quorum-Durability, Tiering zu Objektspeicher | Multi-Device-Szenario; Fault-Tests in Simulation |
| **5 — Prefix/KV-Store (optional)** | Prefix-Digest, KV-Block-Store, Konnektoren, kostenbasierte Eviction | Gemessene Prefix-Cache-Trefferquote und TTFT-Verbesserung |

**Team-Skizze:** 3–4 Storage-Ingenieure (Rust), 1 Retrieval/ML-Ingenieur, 1 Test-/Simulations-Ingenieur; frühe Nutzung durch internes Agenten-Team als Design-Partner.

---

## 19. Risiken und offene Fragen

### 19.1 Risiken

| # | Risiko | Einschätzung | Mitigation |
|---|---|---|---|
| 1 | **Schreibamplifikation Prolly-Tree** bei kleinen, häufigen Commits | mittel | Delta-Log + Fold-Batching; Knotengrößen kalibrieren; Messung in Phase 0 |
| 2 | **ANN mit Versionierung** (Segmente, Deletion Vectors, Filter) ist komplex | hoch | Segmentierter Ansatz mit Delta-Flat-Index; Phase 1 nur Flat/IVF, DiskANN in Phase 2 |
| 3 | **Learned-Layout-Feedbackschleife** (Rich-get-richer, Drift) | mittel | Exploration (5 %), Gewichtszerfall, Rollback über Locator-Versionen |
| 4 | **Tokenizer-Kopplung** (Modellwechsel macht Zählungen ungültig) | mittel | Zählungen pro Familie, lazy Neuberechnung, `boundary_safe` |
| 5 | **Semantische Merge-Konflikte** nicht automatisch lösbar | mittel | `ConflictCell` + externe Auflösung; konservative Defaults |
| 6 | **GC bei vielen Branches** | mittel | TTL für Speculative Branches, generationelles GC, Reflog-Limits |
| 7 | **Format-Lock-in** (On-Disk-Formate schwer änderbar) | hoch | Spezifikation vor Phase 1 einfrieren; Feature-Flags; Migrationswerkzeug |
| 8 | **Scope-Creep** | hoch | Phasen strikt; Kern (Phase 0–2) liefert bereits den Hauptnutzen |
| 9 | **Kein Nutzensnachweis ohne Daten** | hoch | Trace-Recorder in Phase 0; Go/No-Go nach Phase 2 |
| 10 | **Embedding-Lanes bei Modellwechsel** (Dual-Betrieb, Speicher) | mittel | Zeitlich begrenzter Dual-Betrieb; Work Queue priorisiert Re-Embedding |

### 19.2 Offene Fragen

1. **Konsistenzmodell der Zugriffsstatistik:** Reicht eventual consistency für Sidecar-Zähler, oder braucht das Ranking exakte Werte?
2. **Handle-Stabilität über Branches:** Pro-Branch-Aliase vs. global eindeutige Kurz-IDs?
3. **Merge-Funktionen in WASM:** Sandbox-Overhead und Determinismus (Fuel-Limits, keine Zeit/Zufall)?
4. **Eigener FTS vs. Tantivy-Integration** (Segmentformat-Kompatibilität mit `RootSet`)?
5. **Deployment-Modell Priorität:** Embedded vs. Server first? (Vorschlag: Embedded-Kern + dünner Wrapper, beides ab Phase 2)

---

## 20. Glossar

| Begriff | Bedeutung |
|---|---|
| **Cell** | Wissenseinheit mit Facetten (Inhalt, Tokens, Embeddings, Zoom, Kanten, Zeit, Trust) |
| **Facet/Lane** | Physisch getrennt gespeicherter Teil einer Cell (z. B. `meta`, `emb.q`, `raw`) |
| **Zoom-Leiter** | Auflösungsstufen einer Cell (L0 voll … Ln Titel) mit bekannten Token-Kosten |
| **Chunk** | Unveränderliches, content-addressed Objekt (BLAKE3) |
| **Extent** | Physische I/O-Einheit (256 KiB–1 MiB) mit Hot Head und Cold Tail |
| **Pack** | Datei aus Extents |
| **Locator-Index** | Abbildung `ChunkId → physischer Ort` |
| **Prolly-Tree** | Probabilistischer, history-unabhängiger, hash-adressierter B-Baum |
| **RootSet** | Wurzelobjekt des gesamten versionierten Zustands inkl. Indizes |
| **Handle** | Kurzer, token-sparsamer Alias einer Cell (4–6 Zeichen) |
| **Derivation Cache** | Memoisierung abgeleiteter Facetten pro (Quellhash, Producer) |
| **Zoom-Knapsack** | Auswahl der Auflösungsstufe je Treffer unter Token-Budget (Multiple-Choice-Knapsack) |
| **HLC** | Hybrid Logical Clock |
| **Delta-Log** | LSM-artiger Schreibpuffer vor dem Prolly-Tree-Fold |
| **MemDelta** | Sortiertes In-Memory-Delta eines Branches |
| **RRF** | Reciprocal Rank Fusion — robuste Score-Kombination über Retrieval-Pfade |
| **MMR** | Maximal Marginal Relevance — Diversitätsoptimierung |
| **SATB** | Snapshot-at-the-Beginning — GC-Protokoll für gleichzeitige Schreiber |
| **CSR** | Compressed Sparse Row — kompakte Graphadjazenz-Darstellung |
| **FST** | Finite State Transducer — platzspartes Term-Dictionary |
| **Vamana/DiskANN** | Graph-basierter ANN-Index, optimiert für Disk-basierte Suche |
| **Access-Trace** | Gesampelter Strom von Lesezugriffen für Layout-Optimierung und Ranking-Feedback |
| **Work Queue** | At-least-once-Aufgabenwarteschlange für externe Worker (Embedding, Summary, etc.) |
| **Salience** | Berechnete Wichtigkeit einer Cell (Recency × Zugriff × Zentralität × Expand-Rate) |
| **Progressive Disclosure** | Muster: kompakte Handles → selektives Expand nur bei Bedarf |
| **Prefix-Digest** | Hash der stabilen Präfix-Token für KV-Cache-Kompatibilität |

---

## Anhang A: Vergleich mit Alternativen

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

## Anhang B: Entscheidungsregister

| ID | Entscheidung |
|---|---|
| C-01 | Persistente inhaltsadressierte Prolly-Bäume als Versionierungsgrundlage. |
| C-02 | Schichten: Kernel → CAS → VCS → Table → Query/Index → Frontends; Abhängigkeiten nur nach unten. |
| C-03 | Objekte in Packs (Quelle der Wahrheit), Index rekonstruierbar. |
| C-04 | Zwei Ebenen: Transaktion und Commit; Journal für G3-Zeitreise. |
| C-05 | Verschlüsselte Repos nutzen keyed ObjectIds. |
| C-06 | Sitzung = Branch-Arbeitskopie mit Overlay, Snapshot- oder Serializable-Isolation. |
| C-07 | Idempotenzschlüssel für alle Commits; LLM-lesbarer Fehlerkatalog. |
| C-08 | SQL über SQLite-Virtual-Tables; keine eigene SQL-Engine. |
| C-09 | Abgeleitete Indizes außerhalb des Baums, mit Konsistenzvertrag `(index_id, root_id)`. |
| C-10 | Cell als Primitive statt Key-Value; Hot-Head/Cold-Tail-Extent-Layout. |
| C-11 | Zoom-Knapsack (Multiple-Choice-Knapsack) statt Top-k für Context Packing. |
| C-12 | Work Queue statt Engine-internes LLM-Calling für abgeleitete Daten. |
| C-13 | GC mit SATB+Remark; GC ist Phase-Gate mit deterministischer Simulation. |
| C-14 | Ein Repository = eine Kernel-Instanz, Single-Process; Mehrprozess über Server. |
| C-15 | Trust-Labels propagieren über Ableitungsketten; Minimum-Vererbung. |
| C-16 | Gelerntes physisches Layout über Access-Trace mit Exploration-Anteil. |

---

*Ende der Spezifikation.*
