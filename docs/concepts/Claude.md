# EngramDB – Spezifikation

*Hybride Vektor-, Graph- und Volltext-Engine mit Git-Semantik als Gedächtnis-Rückgrat für KI-Agenten und RAG-Systeme · Implementierung in Rust · Version 0.9 (Entwurf) · 26. November 2026*

## 0. Zusammenfassung und Leitprinzipien

EngramDB ist eine eingebettete bzw. als Server betreibbare Datenbank-Engine, die drei Zugriffsmodelle – dichte Vektoren, Property-Graph und BM25-Volltext – auf **einem** gemeinsamen, versionierten Datenbestand bereitstellt. Statt drei Systeme zu synchronisieren, gibt es einen kanonischen **Truth Layer** (content-addressed, Merkle-verkettet, deterministisch) und einen daraus ableitbaren **Index Layer** (ANN-Graphen, Postings, Adjazenz-CSR, Bitmaps). Jede Abfrage – auch jede hybride – läuft gegen genau einen Snapshot (Commit-Hash plus Memtable-Sequenz) und ist damit ohne verteilte Transaktion konsistent.

Die zentrale Idee für Agenten: **Gedächtnis ist ein Git-Repository mit Suchindizes.** Agenten arbeiten in kurzlebigen Branches, jede Aussage trägt kryptografisch gesicherte Provenienz, und ein token-bewusster Planner liefert nicht „Top-k-Zeilen“, sondern ein budgetiertes, zitierfähiges Kontextpaket samt Warnungen (Widerspruch, Veraltung, fehlende Evidenz).

### Leitprinzipien

| # | Prinzip | Konsequenz im Design |
| --- | --- | --- |
| P1 | **Truth ≠ Index** | Der Commit-Hash deckt nur kanonische Daten ab. Indizes sind rebuild-bar; nicht-deterministische Index-Builds gefährden keine Hashes und keinen Merge. |
| P2 | **Immutability by default** | Chunks werden nie überschrieben. Veränderlich sind nur Memtables, Refs und der WAL-Tail. Der Buffer Pool cached ausschließlich unveränderliche Daten (keine Dirty Pages). |
| P3 | **Jede Aussage hat Herkunft** | Provenienz ist Teil des Truth Layers und damit im Merkle-Root enthalten (manipulationssicher). |
| P4 | **Agenten schreiben nie direkt nach main** | Schreiben nur in Session-Branches; Promotion nur über Merge-Gates. |
| P5 | **Optimiere Tokens, nicht Zeilen** | Der Planner löst ein Auswahlproblem unter Token-Budget, nicht ein Top-k-Problem. |
| P6 | **Zeit ist eine Dimension** | Bitemporal: Transaktionszeit (Commit) und Gültigkeitszeit (Realwelt). Widersprüche und Veraltung sind Datenmodell-Konzepte, keine Anwendungslogik. |
| P7 | **Ehrliche Garantien** | Deterministisch und verifizierbar ist der Truth Layer; Recall und Latenz der Indizes sind statistische Eigenschaften. |
| P8 | **Safe Rust im Kern** | `unsafe` nur in klar gekapselten Modulen (SIMD, io\_uring, Mmap-Lesepfad), getestet mit Miri, Loom und Fuzzing. |

### Designziele (Zielwerte, noch nicht gemessen)

- **Größe:** 10^5 bis 10^8 Chunks pro Knoten (384–3072 Dimensionen), bis 10^9 Kanten.
- **Lesen:** p99 einer hybriden Query (k = 20) unter 30 ms bei 10 Mio. Chunks (warm), unter 80 ms bei kaltem NVMe.
- **Schreiben:** mindestens 50 000 Entitäten/s Batch-Ingest je Branch (ohne Embedding-Berechnung); Commit-Latenz p99 unter 5 ms (Group Commit, NVMe).
- **Versionierung:** Branch-Fork O(1) (unter 100 µs); Diff und Merge O(Δ · log N).
- **Durability:** Ack erst nach `fdatasync` (Standard).

**Nicht-Ziele (v1):** OLAP-Scans über Milliarden Zeilen, automatisches Sharding, LLM-Inferenz im Kern (nur Hooks: `Embedder`, `ContradictionJudge`, `Summarizer`).

---

## 1. High-Level-Architektur und Schichtenmodell

### 1.1 Schichten

```
+----------------------------------------------------------------------+
| L7 Zugriff        Rust-API | gRPC/Arrow Flight | MCP-Server | CLI     |
+----------------------------------------------------------------------+
| L6 Agent-Services Sessions | Merge-Gates | Provenance | Konflikte      |
|                   Token-aware Context Planner | Citation-Verifier   |
+----------------------------------------------------------------------+
| L5 Query-Engine   Parser/Binder | Kostenbasierter Optimizer          |
|                   Operatoren: VectorScan, Bm25Scan, GraphExpand, RRF |
+----------------------------------------------------------------------+
| L4 Txn & Version  Snapshot-Manager | OCC | Commit-DAG | Refs | Merge  |
+----------------------------------------------------------------------+
| L3 Index Layer    ANN-Segmente | Postings | CSR | Roaring-Bitmaps     |
|    (abgeleitet)   (rebuild-bar, nicht Teil des Commit-Hashs)         |
+----------------------------------------------------------------------+
| L2 Truth Layer    Prolly-Trees: ent | edge | emb | prov | conf | cat  |
+----------------------------------------------------------------------+
| L1 Chunk Store    CAS (BLAKE3) | Packs | Buffer Pool | Blob Store     |
+----------------------------------------------------------------------+
| L0 I/O & Durab.   WAL | Reflog | io_uring/O_DIRECT | Prüfsummen      |
+----------------------------------------------------------------------+
```

### 1.2 Komponenten

| Komponente | Verantwortung |
| --- | --- |
| `engram-store` | Content-Addressed Chunk Store, Packs, Buffer Pool, Blob Store, GC |
| `engram-wal` | Write-Ahead-Log, Reflog, Recovery, Group Commit |
| `engram-prolly` | Prolly-Trees (history-independent), Diff, 3-Wege-Merge, Proofs |
| `engram-vec` | Vektorblöcke, Quantisierung, Flat-/Vamana-Segmente, Kompaktierung |
| `engram-graph` | Kantenmodell, CSR-Segmente, Traversal, PPR, Community-Jobs |
| `engram-fts` | Analyzer, FST-Wörterbuch, Postings, Block-Max-WAND, BM25 |
| `engram-txn` | Snapshots, OCC, Savepoints, Idempotenz-Tabelle, Commit-Pipeline |
| `engram-vcs` | Commit-DAG, Refs, Branch, Merge-Gates, Rebase, Cherry-pick |
| `engram-query` | Parser, Planner, Operatoren (Anbindung an DataFusion/Arrow) |
| `engram-agent` | Sessions, Provenienz, Konfliktmanager, Token-Planner, Citation-Verifier |
| `engram-server` | gRPC/Flight, MCP-Server, Authentifizierung, Quoten, Telemetrie |

### 1.3 Datenmodell

Alles ist eine **Entität** (Dokument, Chunk, Fakt, Konzept, Zusammenfassung, Agent). Beziehungen sind **Kanten erster Klasse** mit eigenen Eigenschaften, Gültigkeit, Vertrauensstufe und Provenienz. Vektoren und Text werden genau einmal gespeichert.

```rust
struct EntityRecord {
    id: EntityId,                 // 128 Bit, zeitlich sortierbar (UUIDv7-artig)
    kind: KindId,                 // Doc | Chunk | Fact | Concept | Summary | ...
    labels: SmallBitSet,
    props: ArchivedProps,         // rkyv, typisiert laut Schema
    content: Option<ContentRef>,  // inline unter 2 KiB, sonst Blob-Hash
    content_hash: Hash,           // BLAKE3 über kanonischen Inhalt
    valid: Interval<Hlc>,         // Gültigkeitszeit (Realwelt)
    tier: TrustTier,              // Verified | Curated | AgentInferred | Hypothesis
    belief: Beta,                 // (alpha, beta) Evidenzzähler -> Konfidenz
    prov: ProvId,                 // Verweis auf Provenienz-Record
    level: u8,                    // 0 Rohchunk, 1 Abschnitt, 2 Dokument, 3 Thema
    flags: Flags,                 // STALE | TOMBSTONE | DERIVED | ...
}
struct EdgeRecord { id: EdgeId, src: EntityId, dst: EntityId, etype: TypeId,
                    props: ArchivedProps, valid: Interval<Hlc>,
                    tier: TrustTier, belief: Beta, prov: ProvId }
```

Weitere Objekte: `ProvRecord` (Abschnitt 4.1), `Conflict` (4.6), `Session` (4.3), `EmbeddingSpace` (Modell-ID, Dimension, Metrik, Quantisierung, Tokenizer) und `DecayProfile` im Katalog.

### 1.4 Write-Path

1. **Begin:** Die Transaktion pinnt einen Snapshot `(manifest_hash(B), seq)` auf Branch `B`.
2. **Stage:** Schreiboperationen landen in einem Schreibpuffer (Arrow-RecordBatch-Builder). Schema-Validierung: Typen, Kardinalitäten, **Pflichtfeld Provenienz** (Quelle oder Ableitungsnachweis).
3. **Commit-Validierung:** Optimistic Concurrency Control. Write-Write-Konflikte auf Schlüsselebene gegen zwischenzeitliche Commits auf `B`; optional SSI mit Read-Set-Prüfung.
4. **Group Commit:** Der Branch-Committer bündelt wartende Transaktionen, schreibt sie als WAL-Records (LSN, CRC32C) und führt **einen** `fdatasync` pro Gruppe aus. Erst danach Ack an den Client.
5. **Apply:** Anwendung auf die Branch-Memtable (Skip-List für Records und Kanten, flacher Delta-Vektorpuffer, Mini-Invertindex). Sofort sichtbar für neue Snapshots.
6. **Seal (Hintergrund oder explizit):** Memtable → sortierter Batch → Prolly-Tree-Updates mit Path-Copying → neue Chunks (BLAKE3) in ein Pack → neues Manifest → **Commit-Objekt** → Ref-Update per Compare-and-Swap, gesichert über das Reflog. Danach Checkpoint-LSN vorrücken.
7. **Derived Jobs (async, dauerhaft in einer Jobtabelle):** Index-Segmente bauen, Embeddings berechnen, Zusammenfassungen und Konfliktprüfung.

**Wichtig:** Eine *Transaktion* ist nicht automatisch ein *Git-Commit*. Viele Transaktionen werden beim Seal zu einem Commit verschmolzen; `COMMIT 'Nachricht'` erzwingt einen synchronen Seal. Time-Travel adressiert Commits (versiegelte Zustände).

### 1.5 Read-Path

1. **Pin:** `Arc<Manifest>` plus Memtable-Sequenz (Epochen-Pin via `crossbeam-epoch`/`arc-swap`); Lesen blockiert nie Schreiben.
2. **Bind und Optimize:** Logischer Plan, Kostenmodell auf Basis von Manifest-Statistiken (Kardinalitäten, Gradhistogramme, Dokumentfrequenzen, Roaring-Kardinalitäten).
3. **Physischer Plan:** Operatoren laufen segmentweise (Fan-out) mit gemeinsam genutzten Filter-Bitmaps und gemeinsamer Score-Schwelle.
4. **Fusion und Fetch:** RRF der Kandidatenlisten, danach Punktlesezugriffe in den Truth Layer (Buffer Pool).
5. **Nachbearbeitung:** Vertrauens- und Zeitgewichtung, Konflikt-Annotation, **Token-aware Planner**, Citation-Token.
6. **Ergebnis:** Arrow-RecordBatch-Stream oder `EvidencePack`.

### 1.6 Abfrage-Oberfläche

Eine GQL/Cypher-nahe Sprache mit Erweiterungen (alternativ als typisierter Rust-Builder oder JSON-Plan über MCP):

```
MATCH (c:Chunk)-[:MENTIONS]->(e:Concept {name: 'Kubernetes'})
WHERE c.tier >= CURATED AND c.valid CONTAINS NOW
SEARCH c USING HYBRID(
    vector(embed('Wie skaliert man Pods?'), space = 'e5-large', k = 100),
    bm25('pods skalieren hpa', k = 100),
    fuse = RRF(k = 60, weights = [1.0, 0.7]))
DECAY CLASS volatility
EXPAND 1..2 HOPS VIA [:RELATED_TO, :PART_OF] SCORE = PPR
PACK CONTEXT budget = 3500 TOKENS tokenizer = 'cl100k'
     placement = EDGES_FIRST zoom = ADAPTIVE
ON BRANCH 'agent/planner/s-42' AS OF COMMIT 'a1b2c3'
RETURN EVIDENCE;
```

---

## 2. Physisches Storage- und Index-Design

### 2.1 Truth Layer und Index Layer

|  | **Truth Layer** | **Index Layer** |
| --- | --- | --- |
| Inhalt | Entitäten, Kanten, Text, Embeddings (Rohvektoren), Provenienz, Konflikte, Katalog | ANN-Graphen und Quantisierungscodes, Postings, CSR-Adjazenz, Label-Bitmaps |
| Struktur | Prolly-Trees, content-addressed | Immutable Segmente (LSM-artig), content-addressed per Quell-Row-Set und Parameter |
| Determinismus | Ja (history-independent): gleiche Menge, gleicher Hash | Nein (parallele Builds, Seeds); optional deterministisch im Single-Thread-Modus |
| Im Commit-Hash | Ja | Nein, nur referenziert (`IndexManifest` mit `source_root_hash`) |
| Verlust | Nicht tolerierbar | Rebuild aus dem Truth Layer |

Damit gilt: **Kein Datum wird dupliziert.** Der Volltext steht nur im Record-Store, die Rohvektoren nur im Vektorblock, Kanteneigenschaften nur im Kantenbaum. Indizes enthalten ausschließlich Zeiger, Postings, Graph-Links und quantisierte Codes (ca. 3–12 % der Vektorgröße). Letztere sind echte Beschleunigerstrukturen, keine redundante Datenhaltung.

### 2.2 Chunk Store und Verzeichnislayout

```
db/
  wal/000001.wal ...           # segmentiert (64 MiB), vorallokiert
  refs/reflog.seg              # Branch-Heads, Tags, Reflog (append-only, CRC)
  chunks/pack-<id>.pack        # immutable Pack: [Header | Chunk* | Footer]
  chunks/pack-<id>.idx         # Fanout + sortierte Hashes -> (Offset, Länge)
  segments/<hash>/             # Index-Segmente: vec.graph, vec.pq, fts.*, csr.*
  blobs/<hash>                 # Quelldokumente (große Binärdaten, CAS)
  CURRENT                      # doppelt gepufferter Superblock (Verweis auf Reflog)
```

- **Adressierung:** BLAKE3-256 über den Chunk-Inhalt (mit Typ-Präfix). Hash-Kosten sind bei mehreren GB/s (SIMD) vernachlässigbar.
- **Chunk-Header:** `magic | type | codec | len | crc32c`. Typen: `ProllyLeaf`, `ProllyInternal`, `Commit`, `Manifest`, `VectorBlock`, `Blob`, `ProvBatch`.
- **Kompression:** zstd (Level 3) mit pro Entitätsart trainierten Wörterbüchern; Vektoren optional f16 (Standard) oder f32, Quantisierung nur im Index.
- **Pack-Index:** Fanout-Tabelle (256 Einträge) plus sortierte Hashliste; für sehr große Stores zusätzlich minimale perfekte Hashfunktion (`boomphf`). Frische Packs werden in der Hintergrundkompaktierung zusammengelegt.
- **Buffer Pool:** Da alles unveränderlich ist, ist der Pool ein reiner Lese-Cache mit Pin-Zählern. Rahmen sind seitenausgerichtet (4 KiB bzw. 16 KiB), Eviction per W-TinyLFU/CLOCK-Pro, getrennte Budgets pro Klasse (Prolly-Innenknoten, Postings, Neighbor-Sektoren, Vektorblöcke), damit ein großer Scan nicht die heißen Strukturen verdrängt.
- **Warum kein reines Mmap:** unkontrollierte Eviction, TLB-Shootdowns, SIGBUS bei I/O-Fehlern, keine sauberen Prefetch-Hints. Mmap bleibt als optionaler Lesepfad für Entwicklung und kleine Datenbanken.

### 2.3 Prolly-Trees und Schlüsselraum

Pro logischer Relation existiert ein **Prolly-Tree** (probabilistischer B-Baum): Knotengrenzen werden durch einen Rolling Hash (Gear/Buzhash) über die Schlüssel bestimmt, Zielgröße ca. 8 KiB (Min 2 KiB, Max 32 KiB), Fan-out 100–300, Höhe 3–4 bei 10^8 Schlüsseln. Eigenschaften: **history-independent** (gleiche Schlüsselmenge ergibt gleichen Baum und gleichen Hash), **strukturelles Sharing** zwischen Branches, **Diff in O(d · log N)** durch Überspringen identischer Teilbäume und effiziente 3-Wege-Merges.

| Baum | Schlüssel | Wert |
| --- | --- | --- |
| `ent` | `EntityId` | `EntityRecord` (rkyv) plus Content-Referenz |
| `edge` | `(src, etype, dst, edge_id)` | `EdgeRecord` |
| `emb` | `(content_hash, space_id)` | `VectorRef(block_hash, row)` |
| `prov` | `ProvId` | `ProvRecord` |
| `conf` | `ConflictId` | `Conflict` |
| `cat` | Name | Schema, Embedding-Spaces, Decay-Profile, Policies |
| `idem` | Idempotency-Key | Ergebnis-Hash der Transaktion (Retry-Dedup) |
| `alog` | `(session, seq)` | Agent-Action-Log (Tool-Call, Argument-Hash, Ergebnis-Hash) |

Werte über 2 KiB liegen out-of-line als Blob (Hash im Blatt). Die Embedding-Tabelle ist nach `content_hash` statt nach Entität geschlüsselt: **identischer Inhalt wird nur einmal eingebettet** (Cache über Branches und Entitäten hinweg), und ein Merge erzeugt keine Vektor-Konflikte, solange der Inhalt gleich bleibt.

### 2.4 Vektor-Subsystem

- **Vektorblock (Truth):** spaltenorientiert, 4-KiB-ausgerichtet, `rows × dim` row-major (f16/f32) plus `entity_id`-Spalte; 16–64 k Zeilen je Block, content-addressed.
- **ANN-Segment (Index):** Vamana/DiskANN-Graph (R = 64, L\_build = 100, α = 1,2). Neighbor-Listen sind in 4-KiB-Sektoren gepackt (ca. 16 Listen je Sektor). Im RAM liegen **PQ- bzw. RaBitQ-Codes** (z. B. 1024 d → 64–128 B), Einstiegspunkte und die obersten Graphebenen. Suche: Beam Search mit Distanzen auf den Codes, ein Sektor-Read je Hop (ca. 20–40 Reads), danach **Rerank** der besten ca. 2k Kandidaten mit Vollpräzision aus dem Vektorblock.
- **Kleine Segmente (unter ca. 50 k Vektoren):** Flat-Scan mit SIMD (AVX2/AVX-512/NEON); schneller und exakt, kein Graph.
- **Frische Daten:** flacher Delta-Puffer im Memtable (bis ca. 20 k Vektoren), danach Mini-HNSW im RAM, im Hintergrund versiegelt zu einem Vamana-Segment.
- **Segment-Hierarchie:** größenabgestufte Kompaktierung (Faktor 4–8). Merge per Neuaufbau oder per Streaming-Merge (überlebende Punkte einfügen, Nachbarn gelöschter Knoten reparieren, nach FreshDiskANN).
- **Liveness:** Eine Zeile gilt als sichtbar, wenn `truth[entity_id].content_hash == row.content_hash`. Das Ergebnis wird je Branch als **Roaring-Bitmap** gecacht (Tombstones). Gelöschte Knoten bleiben bis zur Kompaktierung als Routing-Knoten im Graph.
- **Mehrere Spaces je Entität** (z. B. Chunk-Embedding und Summary-Embedding), Metrik Cosine/Dot/L2, Normalisierung beim Ingest.

### 2.5 Graph-Subsystem

- **Truth:** sortierter Kantenbaum `(src, etype, dst, edge_id)`. Adjazenz ist ein Range-Scan: O(log N + Grad).
- **Index:** **CSR-Segmente** je Richtung (vorwärts und rückwärts): Offsets Elias-Fano, Ziele delta-codiert und bitgepackt, Kantenzeiger für Eigenschaften; Hub-Knoten (Grad über 10^4) erhalten blockweise Adjazenz mit Skip-Index. Frische Kanten liegen in einem sortierten Delta-Overlay im Memtable.
- **Operatoren:** Frontier als Roaring-Bitmap; richtungsoptimierte BFS (Push/Pull); k-Hop mit Budget-Abbruch; Kantentyp-Masken; Temporal-Pushdown (`valid ∩ as_of`) über Spalten neben dem CSR; **approximatives Personalized PageRank** (Forward Push, ε ≈ 1e-4) für relevanzgeordnete Expansion.
- **Label-/Property-Bitmaps:** Roaring je Label und je niedrig-kardinaler Eigenschaft für schnelle Filter; hoch-kardinale Eigenschaften über abgeleitete Sekundärbäume.
- **Community-Detection (Leiden)** als Batch-Job; Ergebnis speist die Zusammenfassungs-Hierarchie (Abschnitt 4.2).

### 2.6 Volltext-Subsystem

- **Segment:** FST-Termwörterbuch (`fst`), Postings in Blöcken zu 128 Dokumenten (Delta plus SIMD-Bitpacking), Term-Frequenzen, Skip-Daten mit **Block-Max-Scores** für Block-Max-WAND/MaxScore, optionale Positionsströme für Phrasenabfragen, 1-Byte-Längennormen.
- **Analyzer-Pipeline (hash-adressiert):** Unicode-Segmentierung, Lowercasing, Sprachdetektion, Stemming, **German Decompounding** (Komposita), N-Gramme für Code und Bezeichner. Analyzer-ID und Parameter fließen in den Segmentschlüssel ein.
- **BM25 mit globalen Statistiken:** `N`, `df`, `avgdl` werden aus den Segment-Statistiken im Manifest aggregiert; Scores sind damit über Segmente und Branches vergleichbar (k1 = 1,2; b = 0,75, je Feld konfigurierbar).
- **Filter-Integration:** Ein Filter-Bitmap überspringt komplette Postings-Blöcke; BM25 läuft damit auch „innerhalb einer Graph-Nachbarschaft“.

### 2.7 Gemeinsamer Docid-Raum und Filter-Pushdown

Jedes Segment vergibt dichte lokale `u32`-Docids. Spalten des Segments: `entity_id`, `content_hash`, Zeiger auf Vektorblock-Zeilen, ANN-Graph und Postings teilen **dieselbe** Docid. Ein Graph-Operator liefert eine `Roaring64`-Menge von `EntityId`s; sie wird pro Segment per sortiertem Merge in eine `Roaring32`-Docid-Menge übersetzt. Da `EntityId`s zeitlich sortiert sind und Segmente zeitlich zusammenhängende Bereiche abdecken, ist die Übersetzung meist Range-Slicing und wird pro Query gecacht. Dieselbe Bitmap filtert danach ANN-Suche **und** BM25 – das ist der Kern des „ohne Duplikation hybrid“-Versprechens. Die Übersetzungskosten sind der Hauptpreis dieser Architektur (siehe Abschnitt 5.3).

### 2.8 RAM-/Disk-Tiering (Beispiel: 10 Mio. Chunks, 1024 d)

| Tier | Inhalt | Größenordnung |
| --- | --- | --- |
| RAM fix | Manifeste, FST-Wörterbücher, Skip-Daten, PQ-Codes (ca. 128 B/Vektor), CSR-Offsets, Roaring-Bitmaps, Prolly-Innenknoten | 3–6 GB |
| RAM Buffer Pool | Prolly-Blätter, Record-Blöcke, Postings-Blöcke, Neighbor-Sektoren | konfigurierbar, 8–32 GiB |
| NVMe | Vektorblöcke f16 (ca. 20 GB), Vamana-Links (ca. 3 GB), Postings (5–10 GB), komprimierter Text (ca. 10 GB), Prolly-Blätter | 40–60 GB |
| Objektspeicher (optional) | Kalte Packs, Backups, Replikate | beliebig |

### 2.9 Hybride Ausführung

**Graph-gefilterte Vektorsuche** – der Optimizer wählt anhand der Selektivität σ = |S| / N:

- **(A) Exakter Pre-Filter:** falls |S| ≤ τ (ca. 20 k): SIMD-Distanzen direkt über die Vektorblock-Zeilen von S, nach Docid sortiert gelesen.
- **(B) Gefilterte Graph-Traversierung:** Bitmap als Prädikat im Beam Search; bei σ unter ca. 10 % Zwei-Hop-Expansion der Nachbarn (ACORN-Prinzip), damit der Graph nicht zerfällt.
- **(C) Post-Filter** mit adaptivem Over-Fetch bei σ über ca. 50 %.
- **Laufzeit-Fallback:** Liefert (B) nach `ef` Pops weniger als k gültige Treffer, wechselt der Operator auf (A).

**Fusion:** Reciprocal Rank Fusion `s(d) = Σ_i w_i / (k + rank_i(d))`, k = 60 als Standard. Alternativ `NORM`-Fusion mit robust geclippter Min-Max-Normalisierung je Retriever. Optional kalibriert der Planner Scores mit isotonischer Regression anhand von Feedback, damit „Relevanz“ als Wahrscheinlichkeit nutzbar ist (Abschnitt 4.2).

**Early Termination:** Alle Segmente teilen eine atomare k-te-Score-Schwelle (WAND-artig). Alle Beine einer hybriden Query nutzen **denselben Snapshot**; es gibt keine 2PC- oder Eventual-Consistency-Lücke zwischen Vektor-, Graph- und Textindex.

---

## 3. Versionierungs- und Concurrency-Engine

### 3.1 Objektmodell (Merkle-DAG)

```
Ref (heads/main, sessions/..., tags/...)  -->  Commit  -->  Manifest
                                               |             |-- ent    (Prolly-Root)
                                               +-- parents   |-- edge   (Prolly-Root)
                                                             |-- emb | prov | conf | cat | idem | alog
```

```rust
struct Commit {
    parents: SmallVec<[Hash; 2]>,   // 1 = normal, 2 = Merge
    manifest: Hash,                 // Merkle-Root über alle Truth-Bäume
    author: AgentId,
    session: Option<SessionId>,
    hlc: Hlc,                       // Hybrid Logical Clock = Transaktionszeit
    message: String,
    stats: DeltaStats,              // +/- Entitäten, Kanten, Bytes
    sig: Ed25519Signature,          // über den Commit-Hash
}
```

Der Commit-Hash ist `BLAKE3(kanonische Kodierung)`. Er bindet Eltern, Daten **und Provenienz** kryptografisch; jede nachträgliche Veränderung eines Fakts oder seiner Herkunft ändert den Root-Hash. Der `IndexManifest` (Liste der zugehörigen Index-Segmente plus Tombstone-Bitmaps) hängt als **Seitenobjekt** am Commit und gehört nicht zum Hash.

**Refs** (`heads/main`, `sessions/<agent>/<id>`, `tags/<name>`, `merge-requests/<n>`) sind benannte Zeiger. Ein Ref-Update ist ein Compare-and-Swap (erwarteter alter Hash) im Reflog. Das Reflog bewahrt auch verworfene Session-Branches für Forensik und Post-Mortem-Replay auf.

### 3.2 Zwei MVCC-Ebenen

1. **Innerhalb eines Branches: Snapshot Isolation.** Das Manifest ist unveränderlich, die Memtable trägt Sequenznummern; ein Reader sieht alles mit `seq ≤ snapshot.seq`. Leser blockieren nie Schreiber. Pro Branch gibt es genau **einen** serialisierenden Committer (kein Schreib-Lock im Hot Path), über Branches hinweg wird parallel geschrieben. SSI ist optional zuschaltbar.
2. **Zwischen Branches: Git-Semantik.** Branches sind vollständig isolierte Welten; es gibt keine Locks zwischen ihnen. Synchronisation geschieht ausschließlich explizit über Merge, Rebase oder Cherry-pick.

Zeitstempel basieren auf einer **Hybrid Logical Clock (HLC)**, damit Commit-Ordnung und Wanduhrzeit konsistent bleiben.

### 3.3 Branching und Session-Branches

- **Fork:** neuer Ref auf einen Commit, leere Memtable. O(1), keine Daten werden kopiert. Segmente, Prolly-Knoten und Vektorblöcke werden über Hashes geteilt.
- **Branch-lokaler Overlay:** Ein Branch sieht `geerbte Segmente + eigene Segmente + Memtable`, jeweils maskiert durch seine Tombstone-Bitmaps. Eine Query fragt alle Segmente ab und mergt die Top-k-Listen.
- **Segment-Explosion vermeiden:** Session-Branches halten Schreibzugriffe zunächst in der Memtable (flache Suche), versiegeln sie erst bei Schwellwert zu **einem** kleinen Segment und kompaktieren beim Merge nach `main`.
- **Durability-Klassen:**

| Klasse | Verhalten | Einsatz |
| --- | --- | --- |
| `Strict` | `fdatasync` pro Commit | Finanz-/Compliance-Daten |
| `Group` (Standard) | Group Commit, ein Sync pro Gruppe | `main`, kuratierte Branches |
| `Relaxed` | Ack nach `write()`, Sync alle 100 ms | Session-Branches |
| `Volatile` | nur RAM, kein WAL | Wegwerf-Experimente, Tests |

### 3.4 Diff

Der Diff zweier Commits läuft **pro Prolly-Tree** durch paralleles Absteigen und Überspringen von Teilbäumen mit gleichem Hash: O(d · log N) statt O(N). Ausgabe als Strom von `(key, old, new)` mit den Klassen `Added`, `Removed`, `Modified{felder}`. Eine **semantische Diff-Schicht** gruppiert nach Entität und Konzept, hängt Provenienz an jede Änderung und erzeugt eine kompakte Zusammenfassung für menschliche oder automatische Reviewer („+12 Fakten zu *Kubernetes*, 3 Fakten ersetzt, 1 Konflikt“).

### 3.5 Drei-Wege-Merge

1. **Merge-Base:** LCA im Commit-DAG (Generationsnummern); bei mehreren Basen (Criss-Cross) eine rekursiv gemergte virtuelle Basis.
2. **Doppel-Diff:** `base → ours` und `base → theirs` je Baum (parallel).
3. **Merge-Walk** über die schlüsselsortierte Vereinigung der Diffs:

| Situation | Regel |
| --- | --- |
| nur eine Seite geändert | übernehmen |
| beide Seiten identisch geändert (gleicher Hash) | übernehmen |
| `ent`: disjunkte Felder | automatisch feldweise zusammenführen |
| `ent`: gleiches Feld verschieden, oder `content_hash` beidseitig geändert | **Konflikt** (Embeddings folgen dem Inhalt, werden nie semantisch gemergt) |
| `ent.belief` (Beta-Zähler) | additiv: `base + Δours + Δtheirs` (Evidenz summiert sich, Counter-CRDT-artig) |
| `ent.tier` | strengere (niedrigere) Stufe; Hochstufung nur durch ein Gate |
| `edge` | Mengensemantik; Add/Add identisch ergibt eine Kante; Delete vs. Modify nach Policy je Kantentyp: `AddWins`, `RemoveWins` oder `Manual` |
| `emb`, `prov` | nie Konflikt (content-addressed bzw. append-only, Vereinigung) |
| `conf` | Vereinigung, Deduplizierung über das Paar `(a, b)` |
| `cat` (Schema) | Kompatibilitätsprüfung, sonst manuell |
| abgeleitete Entitäten (Zusammenfassungen, `level > 0`) | **nicht mergen**, sondern `STALE` markieren und neu erzeugen |

4. **Semantische Validierung** des Ergebnis-Manifests: Kardinalitätsregeln (z. B. zwei konkurrierende `CEO_OF`-Kanten ohne Textkonflikt), referentielle Integrität (Kante auf gelöschte Entität), Widerspruchsscan der berührten Subjekte, Provenienz-Vollständigkeit. Verstöße landen als Violations im Merge-Request-Ref.
5. **Abschluss:** Merge-Commit mit zwei Eltern auf einem temporären Ref, danach ein CAS auf das Ziel. Hat sich das Ziel bewegt, wird mit neuer Basis wiederholt (begrenzte Schleife) oder abgelehnt. **Konflikte leben im Merge-Request-Ref**, nie in `main` – `main` ist zu jedem Zeitpunkt konsistent.

**Strategien:** `fast-forward`, `merge`, `squash`, `rebase` (Commits einzeln wiederholen), `cherry-pick` einzelner Fakten (ausgewählte Entitäten plus ihr Provenienz-Abschluss und zugehörige Kanten). Komplexität: O(Δ · log N).

### 3.6 Time-Travel (bitemporal)

- `AS OF COMMIT <hash>`: direkter Zugriff auf das Manifest.
- `AS OF TIMESTAMP t`: Binärsuche im per-Branch-Commit-Index (B-Baum über HLC, plus Skip-Zeiger in der Commit-Kette).
- `VALID AT t`: Filter auf die Gültigkeitszeit der Fakten; kombinierbar zu `AS OF TX t1 VALID AT t2` („Was glaubten wir am 1. März über den Zustand am 1. Januar?“).
- **Reproduzierbarkeit:** `(Commit-Hash, Query-Hash, Parameter)` reproduziert jede Agentenantwort exakt – Grundlage für Audits und Regressionstests.
- **Historie einer Entität:** jeder Commit trägt eine kleine „Touched-Keys“-Zusammenfassung (Bloom-Filter plus sortierte Liste bei kleinen Änderungen), damit `HISTORY(entity)` nicht alle Bäume absteigen muss.

### 3.7 Rollback und Savepoints

- **Savepoint** = (Commit-Hash, Memtable-Sequenz). `ROLLBACK TO sp` ist ein Ref-CAS auf den älteren Commit (O(1)); noch nicht versiegelte Schreibvorgänge werden über eine kleine Menge „aborted seq ranges“ für Reader unsichtbar und beim Seal ignoriert.
- **Session-Abort** = Ref löschen (Reflog behält ihn).
- **Atomarität:** Der einzige atomare Punkt ist das Ref-Update. Entweder ist der alte oder der neue Zustand vollständig sichtbar – auch bei Merges.

### 3.8 Concurrency-Modell in Rust

**Runtimes und Threads:**

- **`tokio` (Multi-Thread):** Netzwerk, Scheduling, asynchrone Jobs, Timer.
- **I/O-Reaktor (Thread-per-Core, Linux `io_uring`):** `O_DIRECT`, registrierte Puffer, optional SQPOLL. Auf anderen Plattformen Fallback auf `pread` im Blocking-Pool.
- **`rayon`-Pool:** CPU-lastige Arbeit (Distanzen, BM25, Index-Builds, Hashing). Strikte Trennung, damit CPU-Last nie den Async-Executor blockiert; Backpressure über begrenzte Kanäle und Semaphoren (I/O-Slots, Speicherbudget).

**Lock-freie Bausteine:**

- **Manifest-Publikation:** `ArcSwap<Manifest>` je Branch → wait-free Reads.
- **Memtable:** `crossbeam-skiplist` plus append-only Arena für Vektoren (Länge per Release/Acquire veröffentlicht); Speicherfreigabe über `crossbeam-epoch`.
- **Buffer Pool:** geshardete Hashtabelle, pro Rahmen atomarer Pin-Zähler und Zustand; kein globaler Lock.
- **Commit-Pipeline:** MPSC-Kanal → Single-Writer je Branch → WAL-Writer mit Ringpuffer; Refs werden von einem einzelnen Reflog-Thread per CAS aktualisiert.

**Typsystem als Sicherheitsnetz:**

- Newtypes und Phantom-Typen (`Hash<T>`, `EntityId`, `SegmentId`, `Lsn`, `Seq`) verhindern Verwechslungen.
- **Typestate:** `Txn<ReadOnly>` vs. `Txn<ReadWrite>`; `Snapshot<'db>` ist lifetime-gebunden, sodass Zero-Copy-Referenzen (`&'snap ArchivedEntity`) den Pin nicht überleben können.
- `#![forbid(unsafe_code)]` in `engram-query`, `-agent`, `-vcs`, `-txn`; `unsafe` nur in `-store` (io\_uring, Mmap) und `-vec` (SIMD), jeweils mit dokumentierten Safety-Invarianten.
- **Tests:** `loom` (Interleavings), Miri, `proptest`, `cargo-fuzz` und eine **deterministische Simulation** mit Crash- und Fault-Injection (Failpoints in WAL und Pack-Schreibpfad).

**Zero-Copy:** Records als `rkyv`-Archive direkt auf Buffer-Pool-Rahmen (`bytecheck`-Validierung einmal beim Laden, danach Trust-Flag). Vektoren und Ergebnisse als Apache-Arrow-Arrays (`FixedSizeList<f16|f32>`); Arrow Flight für Streaming. Rohvektoren gehen ohne Kopie vom O\_DIRECT-Puffer in die SIMD-Kernel.

### 3.9 Durability: WAL, Checkpoint, Recovery

**WAL-Record:** `[len u32 | crc32c u32 | lsn u64 | type u8 | txn_id | payload]`. Segmente zu 64 MiB, per `fallocate` vorallokiert. Torn Writes werden durch CRC, Länge und monotone LSN erkannt; die Recovery stoppt am ersten ungültigen Record (Prefix-Konsistenz). Payload ist logisch und idempotent (Replay über `applied_lsn` im Manifest).

**Group Commit:** Leader/Follower, Wartezeit maximal 200 µs oder 1 MiB je Gruppe; ein `fdatasync` pro Gruppe.

**Seal-/Checkpoint-Protokoll (Reihenfolge ist entscheidend):**

1. Neue Chunks in ein Pack schreiben.
2. Pack-Footer und Index `fsync`en.
3. Index-Segmente und ihre Manifeste `fsync`en.
4. **Reflog-Record** (Ref-CAS plus `checkpoint_lsn`) anhängen und `fdatasync` – **das ist der Commit-Punkt**.
5. Doppelt gepufferten Superblock `CURRENT` aktualisieren.
6. WAL-Segmente unterhalb von `checkpoint_lsn` recyceln.

**Recovery:**

1. Superblock → Reflog-Tail scannen (CRC) → Refs rekonstruieren.
2. Verwaiste Packs (nach Crash vor Schritt 4) werden ignoriert und von der GC entsorgt; Footer werden geprüft.
3. WAL ab `checkpoint_lsn` je Branch in Memtables wiederholen.
4. Fehlende abgeleitete Segmente über die Jobtabelle neu bauen (Engine ist bereits lesebereit; Queries fallen bis dahin auf Flat-Scans zurück).
5. Ziel: Recovery-Zeit unter 10 s bei einigen hundert MB WAL.

**Integrität:** CRC bei jedem Read; vollständige BLAKE3-Verifikation im Hintergrund-`scrub` (Merkle-DAG-Traversal erkennt Bit-Rot). **Verschlüsselung:** AEAD (XChaCha20-Poly1305 oder AES-GCM) auf Datei-/Pack-Ebene mit Mandantenschlüsseln; Hashes werden über den Klartext gebildet, Schlüssel fließen nie in Hashes ein.

**Backup und Replikation:** Ein Snapshot ist eine Menge von Refs plus erreichbaren Chunks. Inkrementelles Backup funktioniert wie `git fetch` (have/want-Verhandlung über den Commit-DAG), Point-in-Time-Recovery ist jeder Commit. Optional kontinuierliches WAL-Shipping. In v2: Raft (`openraft`) über Reflog und WAL; Follower holen Chunks lazy nach und bedienen Time-Travel-Reads.

### 3.10 Garbage Collection

- **Roots:** alle Refs, Reflog-Einträge innerhalb der Retention (Standard 30 Tage), Tags, gepinnte Commits, aktive Snapshots (Epochen-Register).
- **Mark:** Traversal Commit → Manifest → Bäume → Chunks in eine Reachability-Bitmap; inkrementell über Generationsnummern und Per-Commit-Deltas.
- **Sweep:** Packs mit weniger als 50 % Live-Bytes werden umgeschrieben, vollständig tote Packs gelöscht. Abgeleitete Segmente und Blobs werden freigegeben, sobald kein Manifest sie referenziert.
- **Nebenläufigkeit:** Die GC arbeitet auf einem Root-Snapshot; während des Marks neu entstandene Chunks gelten als „jung“ und sind ausgenommen (Write-Barrier).

---

## 4. LLM- und Agenten-Spezialfunktionen

Die folgenden Mechanismen sind keine Anwendungsbibliothek über der Datenbank, sondern Teil der Engine: Sie nutzen Versionierung, Hashing und die gemeinsame Index-Basis direkt.

| Schwäche | Mechanismus | Verankerung in der Engine |
| --- | --- | --- |
| Halluzination, fehlende Quellen | Kryptografische Provenienz, Konfidenz-Modell, session-gebundene Citation-Token, Grounding-Verifier | `prov`-Baum im Merkle-Root, `engram-agent` |
| Context-Window-Limit, Lost-in-the-Middle | Token-aware Planner mit Renditionen, Dedup, Abstraktionshierarchie und U-förmiger Platzierung | `PACK CONTEXT` im Query-Plan |
| Nicht-Determinismus, Tool-Fehler | Session-Branches, Savepoints je Tool-Call, Idempotenz, Merge-Gates, Quarantäne | `engram-vcs`, `idem`- und `alog`-Baum |
| Veraltetes Wissen | Bitemporalität, volatilitätsabhängiger Time-Decay, Supersession, Segment-Pruning | Katalog `DecayProfile`, Segment-Statistiken |
| Widersprüche | Mehrstufige Konflikterkennung beim Schreiben, Konflikt-Objekte, Konflikt-sichtbare Reads | `conf`-Baum, Gate G4 |
| Prompt-Injection, Memory Poisoning | Taint-Tracking, Content-Klassen, Tier-Cap | `ProvRecord.taint`, Gate G6 |

### 4.1 Provenienz, Konfidenz und Zitierpflicht

**ProvRecord** (angelehnt an W3C PROV: `wasGeneratedBy`, `wasDerivedFrom`, `wasAttributedTo`). `ProvId = (subject_id, ev_seq)`; weitere Evidenz zum selben Subjekt (Re-Beobachtung) wird als `ev_seq + 1` angehängt, `EntityRecord.prov` zeigt auf `ev_seq = 0`.

```rust
struct ProvRecord {
    subject: SubjectRef,              // EntityId oder EdgeId plus content_hash
    activity: Activity,               // Ingest | Extract | Infer | Summarize | HumanEdit | Merge
    tool: Option<ToolRef>,            // Modell/Tool, Version, Parameter-Hash, Prompt-Hash
    sources: Vec<SourceRef>,          // { blob_hash, byte_span, page, quote_hash }
    derived_from: Vec<SubjectRef>,    // bei Inferenz: genutzte Fakten
    agent: AgentId, session: SessionId, tx_time: Hlc,
    extractor_conf: f16,              // Rohkonfidenz des Extraktors
    source_root: Hash,                // Wurzel der Ableitungskette (Unabhängigkeitsprüfung)
    taint: Taint,                     // Trusted | Untrusted | Quarantined
    sig: Ed25519Signature,            // Schlüssel des schreibenden Agenten
}
```

Quelldokumente liegen unveränderlich im Blob-Store (`blob_hash`). Dadurch ist jeder Chunk rückführbar bis auf Byte-Spannen im Original.

**Grounding-Stufen** (im Read-Pfad als Filter nutzbar):

| Stufe | Bedeutung | Standard-Sichtbarkeit |
| --- | --- | --- |
| G0 | Quelle vorhanden und Spanne verifiziert (Zitat steht im Blob) | immer |
| G1 | Quelle vorhanden, Spanne nicht verifiziert | immer, markiert |
| G2 | Agenteninferenz mit Eltern-Fakten | nur mit Konfidenzgrenze |
| G3 | ohne Quelle (Hypothese) | nur bei `include_hypotheses` |

**Konfidenzmodell.** Jede Entität und jede Kante trägt eine Beta-Verteilung `(α, β)`. Priors hängen von der Quellenklasse ab (kuratiert `Beta(2,1)`, unbekannt `Beta(1,1)`, Agenteninferenz `Beta(1,2)`).

- **Unterstützung:** `α += w · c` (w = Quellenzuverlässigkeit, c = kalibrierte Extraktorkonfidenz). **Widerspruch:** `β += w`.
- **Unabhängigkeit:** Bekräftigung zählt nur von **verschiedenen** `source_root`s. Ein Agent, der seine eigene frühere Ausgabe zitiert, verstärkt nichts (Schutz vor Echokammern).
- **Ableitung:** Inferierte Fakten erhalten `conf = Π conf(Eltern) · c_regel` (noisy-AND, konfigurierbar), nie mehr als das Minimum der Eltern.
- **Nutzung:** Der Planner rechnet mit der **unteren Konfidenzgrenze** (z. B. 10-%-Quantil der Beta-Verteilung), nicht dem Mittelwert; wenig Evidenz wird dadurch automatisch vorsichtiger bewertet.

**Verifizierbarer Speicher.** `PROVE(entity, commit)` liefert Commit-Signatur plus Merkle-Pfad (Manifest → Baumknoten → Blatt, typisch 3–4 Chunks, einige zehn KB). Ein externer Prüfer kann damit belegen, dass ein Fakt mit dieser Herkunft in diesem Commit stand. Gedacht für Audits, nicht für jede Query.

**Session-gebundene Citation-Token.** Jeder Chunk im Kontextpaket trägt ein kurzes Token, z. B. `[[c:7QK4-3F]]`. Es besteht aus einem Handle-Index und einem MAC-Teil `trunc_40(BLAKE3-keyed(session_key, chunk_hash ‖ commit_hash ‖ span))`. Die Abbildung Token → `(chunk_hash, span, commit)` liegt serverseitig je Session. Ein vom Modell erfundenes Token ist mit Wahrscheinlichkeit 1 − 2^-40 nicht auflösbar.

**Grounding-Verifier** (`VERIFY(claims[])`, rein deterministisch, optional mit `GroundingJudge`-Hook):

| Verdikt | Bedingung |
| --- | --- |
| `Supported` | Handle gültig; alle Zitate (normalisiert, Edit-Distanz ≤ ε) sowie alle Zahlen, Daten und Eigennamen des Claims stehen in der zitierten Spanne |
| `UnanchoredParaphrase` | Handle gültig, aber keine lexikalischen Anker; Entscheidung via `GroundingJudge` (NLI) oder Hinweis |
| `InvalidHandle` | Token unbekannt oder aus anderer Session (erfunden) |
| `Stale` | zitierter Chunk wurde im aktuellen HEAD ersetzt (Supersession) |
| `Contradicted` | zu den Quellen existiert ein offener Konflikt |

**Abstention-Signal.** Jedes Ergebnis trägt `evidence_sufficiency ∈ {Sufficient, Thin, Conflicting, None}`, berechnet aus Anzahl unabhängiger `source_root`s, kalibrierter Top-Relevanz und Konfliktstatus. Bei `None` rendert der Planner einen expliziten Block `NO_EVIDENCE_FOUND` samt Suchbereich, damit das Modell abstinent antwortet, statt zu raten.

### 4.2 Token-aware Query Planner

Der Planner löst **Auswahl unter Budget**, nicht Top-k. Er ist ein Operator (`PACK CONTEXT`) hinter der Hybrid-Suche.

**Anfrage und Antwort:**

```rust
struct ContextRequest {
    budget_tokens: u32,           // harte Obergrenze (inkl. Rendering-Overhead)
    tokenizer: TokenizerId,       // z. B. cl100k, Modell-spezifisch
    placement: Placement,         // EdgesFirst | Chronological | ByDocument | Sandwich
    zoom: Zoom,                   // Raw | Summary | Adaptive
    min_tier: TrustTier, min_grounding: Grounding,
    dedup: DedupPolicy, include_conflicts: bool, explain: bool,
}
struct EvidencePack {
    blocks: Vec<ContextBlock>,    // Text, Handle, Rendition, Level, Konfidenz, Staleness
    omitted: Vec<OmittedGroup>,   // was fehlt, inkl. Expand-Handles
    sufficiency: Sufficiency,
    conflicts: Vec<ConflictView>,
    tokens_used: u32, trace: Option<PlanTrace>,
}
```

**Pipeline:**

1. **Kandidaten:** 200–500 Treffer aus Hybrid-Suche (RRF) plus Graph-Expansion (PPR).
2. **Renditionen:** Jeder Kandidat besitzt mehrere Darstellungen: `raw`, `trimmed` (satzweise gekürzt), `summary` (Eltern-Zusammenfassung, nächsthöhere Ebene), `triples` (kompakte Fakten-Notation). Kosten `c(r)` sind exakte Token-Zählungen für den angeforderten Tokenizer; sie werden je Segment im **Index Layer** gecacht (`tok_count[tokenizer_id]`), damit der Truth Layer deterministisch bleibt.
3. **Nutzen:** `u(d) = rel(d)^a · trust_lcb(d)^b · fresh(d)^c · quality(d)`. `rel` ist kalibriert (isotonische Regression auf Feedback, sonst normalisierter RRF-Rang), `fresh` kommt aus Abschnitt 4.5. `quality` ist eine beim Ingest berechnete **Informationsdichte**: Summe der IDF-Gewichte distinkter Terme und Entitäten pro Token, abzüglich Boilerplate-Anteil (wiederkehrende Shingles über Dokumente hinweg), zstd-Kompressionsrate als billiger Proxy.
4. **Selektion:** Maximiere eine **submodulare Abdeckungsfunktion** `F(S) = Σ_j u(j) · max_{d∈S} sim(j, d)` (Facility Location) unter `Σ c(d, r_d) ≤ B`. Verfahren: **lazy kosten-nutzen-Greedy** (`ΔF / c`), verglichen mit dem besten Einzelelement; Worst-Case-Garantie (1/2)(1 − 1/e) ≈ 0,32, in der Praxis nahe am Optimum. Mehrere Renditionen je Dokument bilden eine Partitions-Nebenbedingung (genau eine pro Dokument), die Garantie schwächt sich dadurch weiter ab. **Redundanz** wird doppelt bekämpft: harte Near-Duplicate-Erkennung beim Ingest (SimHash/MinHash, Schwelle ≥ 0,9 ergibt kanonischen Chunk plus Alias-Liste) und weiche Diversität über den Abdeckungsterm (MMR-ähnlich).
5. **Hierarchischer Zoom:** Zusammenfassungen sind abgeleitete Entitäten (`level` 1 Abschnitt, 2 Dokument, 3 Thema/Community) mit `SUMMARIZES`-Kanten auf ihre Kinder und `derivation_hash = H(Kind-Hashes, Summarizer-ID, Prompt-Hash)`. Ändert sich ein Kind, stimmt der Hash nicht mehr: Die Zusammenfassung wird `STALE` markiert (O(Kinder), dank Merkle-Hashes) und zur Neuerzeugung eingereiht; bis dahin nutzt der Planner sie mit Warnung oder fällt auf die Kinder zurück. Im Modus `Adaptive` ersetzt der Planner mehrere ausgewählte Kinder durch die Eltern-Zusammenfassung, wenn deren Nutzen pro Token den der Kinder übertrifft. Randthemen erscheinen als Zusammenfassung, Kernbelege als Rohtext; zu jedem verdichteten Block gibt es einen **Expand-Handle** für progressives Nachladen durch den Agenten.
6. **Platzierung gegen Lost-in-the-Middle:** `EdgesFirst` ordnet nach Nutzen in der Reihenfolge Rang 1, n, 2, n−1, … (wichtigste Belege am Anfang und am Ende, Randbelege in der Mitte). Alternativen: `Chronological` und `ByDocument` für erzählerische Kohärenz (gruppenweise Platzierung), `Sandwich` (Top-Beleg am Anfang und als Kurzfassung am Ende).
7. **Rendering:** stabile Block-IDs und Citation-Token; ein kurzes **Inhaltsverzeichnis** vorweg (`enthalten: 9 Blöcke, ausgelassen: 14 zu Thema X, Expand via h-12`); Konfliktblöcke; Staleness-Hinweise; **Budgetgarantie** durch exakte Zählung mit 2–3 % Sicherheitsmarge, im Überlauf wird der Block mit dem geringsten Grenznutzen entfernt, nie mitten im Satz gekürzt.

`explain = true` liefert den `PlanTrace` (warum gewählt, warum verworfen, Grenznutzen je Block) – wichtig für Debugging und Retrieval-Evaluierung.

### 4.3 Agenten-Sessions und Fehlertoleranz

**Session-Lebenszyklus:** `Active → Validating → Merged | Aborted | Expired`.

```rust
struct Session {
    id: SessionId, agent: AgentId,
    base: Hash,                   // Commit, von dem geforkt wurde
    branch: RefName,              // sessions/<agent>/<id>
    ttl: Duration, durability: Durability,
    quota: Quota,                 // max Bytes, Entitäten, Schreibrate, Löschquote
    write_tier: TrustTier,        // Tier, den Schreibzugriffe dieser Session erhalten
    gate_policy: GatePolicyId,
}
```

**Mechanismen:**

- **Isolation:** Der Agent liest und schreibt ausschließlich auf seinem Session-Branch; `main` ist **geschützt** (kein direkter Schreibzugriff, nur Merge-Requests). Halluzinierte Fakten oder fehlerhafte Tool-Calls können die globale Wissensbasis nie unkontrolliert verändern.
- **Savepoint je Tool-Call:** Jeder Tool-Aufruf wird als benannter Savepoint verbucht (`tool-call #17`). Bei Fehler oder verworfenem Plan: Rollback in O(1).
- **Idempotenz:** `idem_key = H(session, step, tool, args)` im `idem`-Baum. Retries mit identischem Key sind No-Ops und liefern das gespeicherte Ergebnis; liefert dasselbe `step` bei Wiederholung andere Argumente (Nicht-Determinismus), meldet die Engine `IdemMismatch` statt stillschweigend doppelt zu schreiben.
- **Action-Log:** Der `alog`-Baum protokolliert Tool, Argument-Hash, Ergebnis-Hash und Prompt-Hash je Schritt. Damit ist eine Session auditierbar und für Debugging **wiedergebbar** (Replay gegen einen Fork derselben Basis).
- **Quarantäne:** Agentenschreibzugriffe ohne Quelle erhalten Tier `Hypothesis` und sind im Standard-Read ausgeschlossen (`min_tier = Curated`). Hochstufung nur durch Korroboration (untere Konfidenzgrenze ≥ 0,7 bei mindestens zwei unabhängigen `source_root`s) oder menschliche Freigabe.
- **Speculative Branching:** N Agenten-Strategien forken denselben Commit, laufen parallel, eine Bewertungsfunktion wählt den Gewinner, nur dieser wird gemergt, der Rest verworfen. Kosten: nahezu null Kopieraufwand.
- **Forensik:** Verworfene Sessions bleiben über das Reflog erhalten; `DIFF(base, session_head)` zeigt exakt, was der Agent verändert hätte.

### 4.4 Merge-Gates (Validierungspipeline)

Ein Merge-Request läuft vor dem CAS auf `main` durch Gates. Sie arbeiten **diff-inkrementell** (nur berührte Schlüssel) auf dem virtuellen Ergebnis-Manifest. Implementierung als `trait MergeGate` oder als sandboxed WASM-Plugin (`wasmtime`) für eigene Regeln.

| Gate | Prüfung | Standard |
| --- | --- | --- |
| G1 Schema | Typen, Kardinalitäten, referentielle Integrität | blockierend |
| G2 Provenienz | jeder neue Fakt hat Quelle oder Ableitung; Span-Verifikation (Zitat steht im Blob); Signatur gültig | blockierend |
| G3 Blast Radius | Anteil gelöschter/geänderter `main`-Entitäten (z. B. höchstens 1 %), absolute Obergrenzen | blockierend |
| G4 Widerspruch | Konfliktscan gegen `main` (Abschnitt 4.6) | blockierend bei `Direct` gegen `Verified`, sonst Warnung |
| G5 Retrieval-Regression | Golden-Queries gegen `main` vs. Merge-Ergebnis (Top-k-Überlappung, NDCG-Delta) | warnend oder blockierend per Schwelle |
| G6 Taint/Injection | Inhalte aus untrusted Quellen: Scan, Tier-Cap | blockierend |
| G7 Freigabe | menschliche Bestätigung für Namespaces oder Tier `Verified` | konfigurierbar |

Besteht ein Gate nicht, bleibt der Merge-Request offen; der Agent erhält eine strukturierte Fehlerbeschreibung (welche Fakten, welche Regel) und kann auf seinem Branch korrigieren.

### 4.5 Temporales Indexing und Time-Decay

**Bitemporales Modell:** `tx_time` (Commit-HLC, unveränderlich) und `valid = [valid_from, valid_to)` (Realwelt). Fakten werden nie überschrieben, sondern **superseded**: Ein neuer Fakt zu `(Subjekt, Prädikat)` mit späterem `valid_from` schließt das Intervall des alten und legt eine `SUPERSEDES`-Kante an. Standardabfragen laufen mit `VALID AT NOW`; die Historie bleibt vollständig.

**DecayProfile (im Katalog, je Prädikat oder Entitätsart):**

```rust
struct DecayProfile {
    class: VolatilityClass,       // news | policy | api_docs | constant | custom
    kind: DecayKind,              // Exponential{half_life} | Weibull{k, lambda} | Step{valid_until} | None
    floor: f32,                   // Mindestgewicht, damit alte Belege nie ganz verschwinden
    refresh_on_reaffirm: bool,    // Re-Beobachtung setzt last_confirmed neu
}
// fresh(d) = max(floor, 2^(-dt / half_life)),  dt = as_of - max(valid_from, last_confirmed)
```

Beispiele: `news` 14 Tage, `policy` 365 Tage, `api_docs` 180 Tage plus versionsbasierte Supersession, `constant` ohne Decay.

- **Reaffirmation:** Wird derselbe Fakt (gleicher `content_hash` und Subjektschlüssel) erneut aus **unabhängiger** Quelle beobachtet, entsteht kein Duplikat: `last_confirmed` und `α` steigen, ein weiterer `ProvRecord` (`ev_seq + 1`) wird angehängt.
- **Adaptive Volatilitätsschätzung:** Aus der eigenen Historie schätzt die Engine je Prädikat die Änderungsrate (`λ̂ = Supersessions / Beobachtungszeit`) und schlägt Halbwertszeiten vor, statt sie von Hand zu raten.
- **Segment-Pruning:** Jedes Segment speichert Min/Max von `valid_from` und `last_confirmed`. Recency-Filter (`valid_from > t`) überspringen ganze Segmente; zeitlich ausgerichtete Kompaktierung verstärkt das.
- **Anwendung im Read-Pfad:** Decay wirkt **nach** der Kandidatengenerierung (Re-Scoring vor RRF-Gewichtung und im Planner-Nutzen), mit adaptivem Over-Fetch, damit frische, leicht weniger relevante Treffer nicht vorab abgeschnitten werden. `IGNORE DECAY` und `AS OF` sind jederzeit möglich.

### 4.6 Konflikterkennung beim Schreiben

Beim Einfügen eines Fakts (oder einer extrahierten Aussage) läuft eine gestufte Prüfung. Eine Aussage wird bei Bedarf durch den `Extractor`-Hook als Claim `(Subjekt, Prädikat, Objekt, valid, Polarität)` normalisiert.

| Stufe | Prüfung | Ausführung |
| --- | --- | --- |
| 1 Deterministisch | Schema-Prädikat mit Kardinalität 1 (`functional`), überlappende Gültigkeit, abweichendes Objekt ergibt `Direct`; numerische Abweichung über Toleranz ergibt `Numeric`; neuere Gültigkeit plus Policy `SupersedeOnNewer` ergibt automatische Supersession statt Konflikt | synchron im Commit (µs bis ms) |
| 2 Kandidatensuche | ANN-Nachbarn des Claim-Embeddings (Top 20, Schwelle τ) innerhalb der 1–2-Hop-Nachbarschaft des Subjekts; BM25 mit Gegenbegriffen und Negationen | asynchron |
| 3 Semantisch | `ContradictionJudge`-Hook (NLI-Cross-Encoder oder LLM): `Entails`, `Contradicts`, `Neutral` mit Score | asynchron, Job in der dauerhaften Queue |
| 4 Objekt | `Conflict` als Entität erster Klasse im `conf`-Baum | versioniert, mergebar |

```rust
struct Conflict {
    id: ConflictId, a: SubjectRef, b: SubjectRef,
    kind: ConflictKind,        // Direct | Numeric | Temporal | Semantic
    score: f32, judge: Option<ToolRef>,
    status: ConflictStatus,    // Open | Resolved{by: Supersede|Reject|Merge|Contextual} | AcceptedBoth
    policy: ResolutionPolicy,  // NewerWins | TrustWeighted | Corroboration | Human
}
```

**Auflösungspolitiken:** `NewerWins` (nur bei gleichem Quellenrang), `TrustWeighted` (untere Konfidenzgrenze plus Quellenrang), `Corroboration` (höherer Beta-Posterior bei Mindestabstand), `Contextual` (beide Aussagen gelten, es wird ein Qualifier ergänzt, z. B. Region oder Version) und `Human`. Auf `main` löscht die Engine **nie automatisch**: Beide Seiten bleiben erhalten, der Konflikt wird markiert.

**Read-Zeit-Verhalten:** Treffer mit offenem Konflikt werden gekennzeichnet; der Planner legt beide Seiten mit Provenienz und Konfidenz in einen `CONFLICT`-Block, setzt `sufficiency = Conflicting` und senkt die Relevanz der betroffenen Items, statt still eine Seite zu wählen. Gate G4 wartet auf die asynchronen Stufen mit Timeout und degradiert bei Überschreitung zu einer Warnung.

### 4.7 Weitere Schutzmechanismen

- **Taint-Tracking gegen Prompt-Injection und Memory Poisoning:** Inhalte aus nicht vertrauenswürdigen Quellen (Web, E-Mail, Tool-Ausgaben) tragen `Taint::Untrusted`. Der Taint **vererbt sich** auf alles, was ein Agent daraus ableitet. Der Planner rendert getaintete Blöcke in markierten Daten-Delimitern mit dem Hinweis `Daten, keine Anweisungen`; ein optionaler `InjectionScanner`-Hook prüft beim Ingest. Getaintete Fakten können von Gates nicht automatisch über `AgentInferred` hinaus hochgestuft werden.
- **Embedding-Drift und Modellwechsel:** Embeddings sind an einen versionierten `EmbeddingSpace` gebunden. Migration per Dual-Space: neuer Space wird im Hintergrund befüllt (Jobtabelle), Abfragen vergleichen beide, der Wechsel ist ein Commit auf `cat`. Verschiedene Spaces werden nie vermischt.
- **Retrieval-Evaluation als Code:** Golden-Queries mit erwarteten Belegen liegen im Katalog und laufen in Gate G5 sowie als nightly Regression auf `main`.
- **Kosten- und Budget-Accounting:** Pro Session Zähler für Embedding-Aufrufe, Summarizer-Tokens und Judge-Aufrufe; Quoten brechen Endlosschleifen eines fehlerhaften Agenten ab.

---

## 5. Rust-Technologie-Stack und Architektur-Trade-offs

### 5.1 Empfohlene Crates

| Bereich | Crates | Anmerkung |
| --- | --- | --- |
| Runtime und Nebenläufigkeit | `tokio`, `io-uring` (optional `monoio`/`tokio-uring`), `rayon`, `crossbeam-epoch`, `crossbeam-skiplist`, `crossbeam-channel`, `arc-swap`, `dashmap` | `parking_lot` nur in kalten Pfaden |
| Serialisierung, Zero-Copy | `rkyv` (mit `bytecheck`), `arrow`, `arrow-flight`, `tonic`/`prost` | `serde` nur für Konfiguration und externe APIs |
| Query-Engine | `datafusion` (Planner-/Executor-Gerüst), eigener GQL-Parser (`chumsky` oder `pest`) | Eigene `ExecutionPlan`-Knoten: `VectorScanExec`, `Bm25Exec`, `GraphExpandExec`, `RrfExec`, `PackContextExec` |
| Hash, Krypto, Prüfsummen | `blake3`, `ed25519-dalek`, `chacha20poly1305`/`aes-gcm`, `crc32c` | Hashes über Klartext, Verschlüsselung nur auf Dateiebene |
| Kompression | `zstd` (Dictionaries), `lz4_flex` | zstd für Text/Records, LZ4 für heiße Postings optional |
| Vektoren | `simsimd` oder eigene SIMD-Kernel, `half`, `faer` (PQ-Training), `hnsw_rs` o. ä. als Delta-Index | Vamana/RaBitQ voraussichtlich Eigenbau |
| Volltext | `fst`, `roaring`, `bitpacking`, `unicode-segmentation`, `rust-stemmers`, `whatlang` | Tantivy als Referenz für Postings-Layout und BM25; eigenes content-addressed Segmentformat nötig |
| Graph | eigener CSR-Code, `roaring`, Leiden-Implementierung (Eigenbau oder Crate) | `petgraph` nur in Tests/Prototypen |
| Tokenisierung | `tokenizers`, `tiktoken-rs` | Token-Zählung je Tokenizer-ID |
| Caching | eigener Buffer Pool (W-TinyLFU), `moka` für Metadaten |  |
| Perfect Hashing | `boomphf` | Pack-Index und Segment-ID-Maps |
| Plugins, Hooks | `wasmtime` (Gates), `ort` oder `candle` (lokale Embedder/Judges, optional) | Hooks sind Traits; Modelle laufen außerhalb des Kerns |
| HA (v2) | `openraft` | Replikation von Reflog und WAL |
| Agenten-Schnittstelle | `rmcp` (MCP-Server) | Tools siehe Anhang B |
| Observability | `tracing`, `tracing-opentelemetry`, `metrics`, `criterion`/`divan` | `EXPLAIN ANALYZE` mit Operator-Statistiken |
| Qualitätssicherung | `loom`, Miri, `proptest`, `cargo-fuzz`, `insta`, eigene Crash-Simulation | Deterministische Simulation ist Pflicht für WAL und Seal |

**Build-vs-Reuse:** Prolly-Trees, das Pack-/Reflog-Format, Segment-Kompaktierung und der Planner sind Eigenbau (Kernkompetenz). Wiederverwendet werden Primitive (SIMD, FST, Roaring, Arrow, DataFusion, BLAKE3), nicht fertige Index-Bibliotheken, weil deren Dateiformate nicht content-addressed und nicht branch-fähig sind.

### 5.2 Wichtige Entwurfsentscheidungen im Überblick

| Entscheidung | Gewählt | Alternative | Grund |
| --- | --- | --- | --- |
| Basisstruktur | Prolly-Tree (Truth) plus LSM-artige Segmente (Index) | reiner LSM-Tree | Diff, Merge und Struktur-Sharing sind beim LSM nicht billig; Prolly liefert sie nativ |
| Hash-Umfang | nur Truth Layer | alles inklusive Indizes | nicht-deterministische Index-Builds würden Hashes und Merges zerstören |
| ANN-Algorithmus | Vamana/DiskANN-artig plus Flat für kleine Segmente | HNSW durchgehend | HNSW ist RAM-lastig und änderbar; Segmente sollen immutable sein |
| Schreibmodell | Single-Writer je Branch, OCC | Multi-Writer mit Locks | einfacher, korrekt, parallel über Branches |
| I/O | eigener Buffer Pool mit `O_DIRECT` | Mmap | Eviction-Kontrolle, keine SIGBUS-Risiken |
| Vektoren im Truth Layer | Rohvektoren (f16/f32), content-keyed | Vektoren nur im Index | Embeddings sind teuer und modellabhängig; Merge ohne Re-Embedding |

### 5.3 Ehrliche Abwägung der Zielkonflikte

| Zielkonflikt | Auswirkung | Gegenmaßnahmen |
| --- | --- | --- |
| **Write-Amplification** | Path-Copying im Prolly-Tree schreibt pro geändertem Schlüssel im schlimmsten Fall `H × Chunkgröße` (bei H = 4 und 8 KiB grob 30 KB). ANN-Segmente werden bei Kompaktierung neu gebaut (bei Faktor 4–8 pro Vektor ca. 3–6 Mal über den Lebenslauf bei 10^8 Vektoren). Größenordnungen, noch zu messen. | Memtable-Batching vor dem Seal (sortierte Batches amortisieren Pfadkosten); Bulk-Load-Pfad (Bottom-up-Aufbau ohne Path-Copying); große L0-Segmente; Streaming-Merge statt Neuaufbau; Flat-Scan für kleine Segmente |
| **RAM vs. Latenz** | PQ-Codes im RAM sind Voraussetzung für wenige SSD-Reads; weniger RAM bedeutet mehr Reads (NVMe ca. 80–100 µs je Random Read; 30 abhängige Reads ergeben mehrere ms) und sinkenden Recall bei starker Quantisierung | Beam-Width 4–8 für parallele Reads; größerer Rerank-Pool; Tier-Budgets; Prefetch über `io_uring` |
| **Segment-Fan-out durch Branches** | Jede Query fragt alle Segmente im Branch-View ab; Latenz wächst mit der Segmentzahl | Obergrenze (z. B. 16–32 Segmente je View); Kompaktierung beim Merge; geteilte Score-Schwelle; Memtable-Flat-Suche bis Schwellwert |
| **Docid-Übersetzung** | `EntityId → Docid` je Segment kostet CPU; die Zeit-Lokalitätsannahme bricht bei stark überschriebenen alten Entitäten | Range-Slicing, Query-Cache, MPHF; zeitlich ausgerichtete Kompaktierung |
| **Filter-Selektivität und Recall** | Restriktive Filter lassen Graphsuche zerfallen; Tombstone-Knoten verschlechtern die Navigation; der mittlere Selektivitätsbereich ist am schwierigsten | Exakter Pfad (A) bei kleinen Mengen, ACORN-artig dichtere Nachbarschaft (mehr RAM), Laufzeit-Fallback, regelmäßige Kompaktierung |
| **Merge-Konflikte bei Vektoren** | Embeddings folgen dem Inhalt; zwei Branches mit geändertem Inhalt sind ein Entitätskonflikt, kein Vektorkonflikt. Embeddings verschiedener Modelle sind nicht mergebar | Content-keyed `emb`-Baum, Spaces, Dual-Space-Migration |
| **Merge-Konflikte bei Graphen** | Textlich konfliktfreie Merges können **semantisch** verletzen (zwei `CEO_OF`-Kanten); additive Beta-Zähler können Evidenz doppelt zählen, wenn zwei Branches unabhängig dieselbe Quelle ingestieren | Kardinalitätsgates; Belief nach dem Merge **aus der deduplizierten Evidenzmenge** (`source_root`) neu berechnen, nicht aus Zählern; Edge-Policies |
| **Abgeleitete Daten sind nicht verifizierbar** | LLM-Zusammenfassungen und Embeddings sind nicht deterministisch reproduzierbar; der Hash belegt ihre Unveränderlichkeit, nicht ihre Korrektheit | Speicherung als Daten mit `tool`/`prompt`-Hash; `STALE`-Erkennung über `derivation_hash`; Grounding-Verifier |
| **Unveränderlichkeit vs. Löschpflicht (DSGVO)** | Historie ist per Design dauerhaft; Löschen personenbezogener Daten widerspricht dem | **Crypto-Shredding** (Blobs und Content mit mandanten-/entitätsbezogenen Schlüsseln), Redaction-Commits als dokumentierte Ausnahme mit History-Rewrite und neuen Hashes, Retention-Policies |
| **Time-Decay** | Falsch konfigurierte Halbwertszeiten verstecken gültiges altes Wissen | `floor`, `IGNORE DECAY`, adaptive Volatilitätsschätzung, Erklärung im `PlanTrace` |
| **Single-Writer je Branch** | Begrenzt den Durchsatz eines einzelnen Branches auf die Commit-Pipeline | Parallele Ingest-Branches plus Merge; Batch-API; Bulk-Load |
| **Punktzugriff vs. LSM** | Prolly-Lookup kostet `H` Chunk-Zugriffe statt Bloom plus wenige Level | Pinned Innenknoten im RAM, Buffer Pool, Pack-Index; Lookups sind meist gecacht |
| **Betriebs- und Implementierungskomplexität** | Zwei Welten (Truth/Index), GC, Retention, Segmentverwaltung | Strikte Schichtung, Simulationstests, `scrub`, klare Rebuild-Garantie für Indizes |
| **Einzelknoten in v1** | Kein Sharding, Ausfallsicherheit über Backups | Raft-Replikation in v2; Read-Replicas für Time-Travel |

### 5.4 Teststrategie und Qualitätsziele

- **Korrektheit:** deterministische Simulation mit Crash-Injection an jedem Schritt des Seal-Protokolls; Invariante: Nach Recovery entspricht jeder Ref entweder dem alten oder dem neuen Zustand. Property-Tests für Prolly-Trees (History-Independence, Diff/Merge-Rundlauf), `loom` für Memtable und Buffer Pool.
- **Such-Qualität:** Recall@10 ≥ 0,95 bei definierter Latenz (ANN-Benchmarks-Stil), BM25- und Hybrid-Qualität auf BEIR-artigen Korpora, Multi-Hop-Retrieval auf eigenen Agent-Workloads; Graph-Operatoren gegen LDBC-SNB-artige Workloads.
- **Agenten-Szenarien:** Fehlerinjektion (doppelte Tool-Calls, halluzinierte Handles, widersprüchliche Fakten, Injection-Inhalte) mit erwarteten Gate-Ergebnissen.
- **Performance-Gates:** `criterion`-Benchmarks in CI; Regressionsgrenzen auf Commit-Latenz, Seal-Durchsatz, p99-Hybridlatenz.

### 5.5 Meilensteine

| Meilenstein | Inhalt |
| --- | --- |
| M0 Kern | Chunk Store, WAL, Reflog, Prolly-Trees, Commit-DAG, Branch/Diff, Crash-Simulation |
| M1 Suche | FTS-Segmente (BM25), Flat- und Vamana-Segmente, CSR-Graph, RRF, einfache Hybrid-Queries |
| M2 Versionierung komplett | 3-Wege-Merge, Rebase, Cherry-pick, Time-Travel, GC, Backup/Fetch |
| M3 Agenten-Kern | Sessions, Savepoints, Idempotenz, Provenienz, Citation-Token und Verifier, Merge-Gates G1–G3 |
| M4 Planner und Zeit | Token-aware Planner, Abstraktionshierarchie, Time-Decay, Konflikterkennung, G4–G6, MCP-Server |
| M5 Härtung | Verschlüsselung, Crypto-Shredding, Raft-Replikation, Performance-Tuning, Beta-Release |

---

## Anhang A: Beispielablauf einer Agenten-Session

1. Der Agent öffnet eine Session: `FORK main` ergibt `sessions/planner/s-42` (O(1)), Quote und Tier `AgentInferred`.
2. `memory.search` läuft als Hybrid-Query mit `PACK CONTEXT budget = 3500`; der Agent erhält Blöcke mit Citation-Token, einen `CONFLICT`-Block und `sufficiency = Thin`.
3. Der Agent recherchiert extern, schreibt drei neue Fakten mit Quellenspannen; jeder Tool-Call ist ein Savepoint mit `idem_key`.
4. Ein Tool-Call schlägt fehl, der Agent führt `ROLLBACK TO tool-call #17` aus (Ref-CAS) und wiederholt ihn ohne Duplikate.
5. Beim Schreiben prüft die Engine synchron Kardinalitäten und startet asynchron Kandidatensuche und `ContradictionJudge`: ein Fakt kollidiert mit einem `Verified`-Fakt auf `main` (`Direct`).
6. Der Agent stellt einen Merge-Request. Gates G1, G2 und G3 bestehen; G4 blockiert wegen des Konflikts und liefert Fakt-IDs.
7. Der Agent korrigiert (der strittige Fakt wird als `Hypothesis` mit Qualifier umgeschrieben), G4 wird zur Warnung, G5 (Golden-Queries) besteht, ein Mensch gibt frei (G7).
8. Merge-Commit auf `main` per CAS; Zusammenfassungen betroffener Themen werden `STALE` und regeneriert; die Session wechselt auf `Merged`, der Branch wird nach TTL entfernt, der Reflog bleibt.

## Anhang B: MCP-/API-Werkzeuge (Auszug)

| Werkzeug | Zweck |
| --- | --- |
| `memory.search` | Hybride Suche mit `PACK CONTEXT`, liefert `EvidencePack` |
| `memory.expand` | Expand-Handle auflösen (Zusammenfassung zu Rohdaten, Nachbarschaft) |
| `memory.write` | Fakten, Chunks, Kanten mit Pflicht-Provenienz in den Session-Branch schreiben |
| `memory.savepoint` / `memory.rollback` | Savepoint setzen und zurückkehren |
| `memory.fork` / `memory.diff` | Branch erstellen, Diff zwischen Commits oder Branches |
| `memory.merge_request` | Merge-Request mit Gate-Lauf erzeugen, Status und Verstöße abfragen |
| `memory.verify` | Claims gegen Citation-Token prüfen (Grounding-Verifier) |
| `memory.history` | Verlauf einer Entität, bitemporal (`AS OF TX`, `VALID AT`) |
| `memory.conflicts` | Offene Konflikte listen und auflösen |
