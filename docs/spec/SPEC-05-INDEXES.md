# SPEC-05 — Indizes

**Version 2.0 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Storage](SPEC-03-STORAGE-ENGINE.md) · [Context Engine](SPEC-06-CONTEXT-ENGINE.md)

> Alle Indizes sind **segmentbasiert, immutable, content-addressed und im `RootSet` verankert.** Forks teilen Segmente ohne Kopie. Atomare Sichtbarkeit mit Primärdaten. Zeitreise liefert konsistente Indexstände.

---

## 1. Vektorindex

### 1.1 Architektur: Segmente + Delta

```
Neue Vektoren (Commits)
        │
        ▼
  Delta-Index (Flat, Brute-Force, SIMD)     ← im RAM, pro Branch
        │  Schwelle: 50 000 Vektoren
        ▼
  Segment-Build (Hintergrund)
        │
        ▼
  Vamana-Graph-Segment (Disk-optimiert)     ← immutable, content-addressed
        │  Size-Tiered Compaction
        ▼
  Größeres Segment (Merge)
```

### 1.2 Delta-Index (In-Memory)

| Parameter | Wert | Begründung |
|---|---|---|
| Typ | Flat / Brute-Force | Exakte Suche für kleine Mengen |
| Max. Vektoren | 50 000 | Darüber: Hintergrund-Build eines neuen Segments |
| Distanzberechnung | SIMD (AVX2/NEON), int8-quantisiert | Throughput ~100M Distanzen/s |
| Bei jeder Suche | Mit-durchsucht + Ergebnisse mit Segmenten fusioniert | **Kein Index-Lag** |

### 1.3 Vamana/DiskANN-Segmente

| Parameter | Wert | Begründung |
|---|---|---|
| Graph-Grad R | 64 | Balance: Recall vs. Speicher/Build-Zeit |
| L_build | 125 | Suchlistenlänge beim Graphaufbau |
| L_search | 100 (konfigurierbar) | Suchlistenlänge zur Laufzeit |
| α (Pruning-Parameter) | 1.2 | Standard-DiskANN-Wert |
| Distanzmetrik | Cosine Similarity (normalisiert → Inner Product) | Standard für Embedding-Modelle |
| Quantisierung im Graphknoten | int8 (Scalar Quantization) | Für Distanzberechnung während Traversal |
| Re-Ranking | f16 aus Cold Tail | Für finale Top-k nach Traversal |

### 1.4 Co-Location

Vamana-Graphknoten werden physisch nah an ihren Embedding-Daten platziert:

```
Vamana-Segment auf Disk:
┌──────────────────────────────────────────────────┐
│ Segment Header                                    │
│   segment_id, vector_count, dimensions, R, etc.   │
├──────────────────────────────────────────────────┤
│ Navigating Node Block (Hot, in Page Cache)        │
│   Für jeden Vektor:                               │
│     quantisierter Code (int8, 768 B)              │
│     Nachbar-IDs (R × 4 B = 256 B)                │
│     → ~1 KiB pro Vektor                          │
├──────────────────────────────────────────────────┤
│ Full Precision Block (Cold)                       │
│   Für jeden Vektor:                               │
│     f16-Embedding (768 × 2 B = 1536 B)           │
└──────────────────────────────────────────────────┘
```

### 1.5 Löschungen: Deletion Vectors

Versionierte **Roaring-Bitmaps** (content-addressed) je Segment:

```rust
pub struct DeletionVector {
    pub segment: SegmentRef,
    pub deleted: RoaringBitmap,     // Bit-Positionen der gelöschten Vektoren
}
```

- Bei Fork: DeletionVector wird strukturell geteilt.
- Bei Merge: Union der DeletionVectors.
- Segment-Compaction eliminiert gelöschte Vektoren und erzeugt neues Segment ohne DeletionVector.

### 1.6 Gefilterte Suche

Prädikate (Namespace, Typ, Zeit, Trust) werden zu Bitmaps kompiliert:

```rust
pub fn filtered_search(
    query: &[f32],
    filter: &FilterBitmap,
    segments: &[Segment],
    k: usize,
    l_search: usize,
) -> Vec<(CellId, f32)> {
    let selectivity = filter.count_ones() as f64 / filter.len() as f64;

    if selectivity < 0.01 {
        // Hochselektiv: Brute-Force über Treffermenge
        brute_force_filtered(query, filter, segments, k)
    } else if selectivity < 0.2 {
        // Mittel: ACORN-inspiriert (Filter in Graph-Traversal eingebettet)
        acorn_search(query, filter, segments, k, l_search)
    } else {
        // Niedrig-selektiv: Standard-Suche + Post-Filter
        search_then_filter(query, filter, segments, k * 3, l_search)
    }
}
```

### 1.7 Mehrere Embedder

Pro Namespace können mehrere Embedder registriert sein. Jeder Embedder erzeugt eigene Lanes und Segmente:

```
vec_segments: [
    SegmentRef { embedder: "emb-768-v3", id: ..., version: 2 },
    SegmentRef { embedder: "emb-768-v3", id: ..., version: 2 },
    SegmentRef { embedder: "emb-1024-v4", id: ..., version: 1 },  // Migration läuft
]
```

**Modellwechsel:** Neuer Embedder → neue Lane + Re-Embedding über Work Queue. Alte Lane bleibt bis Migration abgeschlossen lesbar. Suche über beide Embedder mit Ergebnis-Union (nicht Intersection).

### 1.8 Segment-Compaction

| Trigger | Schwelle |
|---|---|
| Segment-Anzahl pro Embedder | > 8 → Merge der kleinsten zwei |
| DeletionVector-Dichte | > 30 % gelöschte Vektoren → Rebuild |
| Segment-Alter + Größe | Size-Tiered: T1 (< 10k), T2 (< 100k), T3 (< 1M), T4 (unbegrenzt) |

---

## 2. Volltext (FTS)

### 2.1 Segmentbasierter invertierter Index

```
FTS-Segment:
┌──────────────────────────────────────────────────┐
│ Segment Header                                    │
│   segment_id, doc_count, term_count, etc.         │
├──────────────────────────────────────────────────┤
│ Term Dictionary (FST)                             │
│   Finite State Transducer: Term → TermInfo        │
│   Unterstützt: exakte Suche, Präfix, Fuzzy (≤2)  │
├──────────────────────────────────────────────────┤
│ Posting Lists (Block-Max-codiert)                 │
│   Pro Term: sortierte DocId-Liste + TF + Positionen │
│   Block-Max-WAND-Metadaten pro 128-Doc-Block      │
├──────────────────────────────────────────────────┤
│ Field Norms                                       │
│   Dokumentlängen pro Feld (für BM25-Normalisierung) │
├──────────────────────────────────────────────────┤
│ Skip Lists / Fast-Forward-Strukturen              │
└──────────────────────────────────────────────────┘
```

### 2.2 BM25-Parameter

| Parameter | Wert | Beschreibung |
|---|---|---|
| k1 | 1.2 | Term-Frequency-Sättigung |
| b | 0.75 | Dokumentlängen-Normalisierung |
| avgdl | pro Segment berechnet | Durchschnittliche Dokumentlänge in Token |

### 2.3 Tokenisierung (Suche, unabhängig von LLM-Tokenizern)

| Sprache | Tokenizer |
|---|---|
| Deutsch | Unicode-Segmentierung + Decompounding + Stemming (Snowball) |
| Englisch | Unicode-Segmentierung + Stemming (Snowball) |
| Fallback | Unicode-Segmentierung + Lowercasing |
| Code | CamelCase-Split + Snake-Case-Split + Lowercasing |

### 2.4 FST (Finite State Transducer)

- Gebaut pro Segment, content-addressed → über Forks geteilt.
- **Fuzzy-Matching:** Levenshtein-Automat über FST, max Edit-Distanz 2.
- **Präfix-Suche:** Native FST-Traversierung.
- **Größe:** ~1–3 Bytes pro Term (kompakter als HashMap).

### 2.5 Block-Max WAND

Effizientes Top-k-Pruning:
- Posting Lists in Blöcke à 128 Dokumente aufgeteilt.
- Pro Block: maximaler Partial Score gespeichert.
- Bei Multi-Term-Queries: Blöcke mit max_score < aktuellem k-ten Score werden übersprungen.
- **Effekt:** 2–10× weniger Posting-List-Einträge gelesen als naives DAAT.

### 2.6 Feldbezogene Gewichtung

| Feld | Gewicht | Mapping |
|---|---|---|
| title (attr oder zoom.L2) | 3.0 | Indexiert aus `attrs.title` oder `zoom.L2` |
| body (raw) | 1.0 | Indexiert aus `raw`-Lane |
| tags (attrs.tags) | 2.0 | Indexiert aus Tag-Attributen |
| edge_labels | 0.5 | Indexiert aus Kanten-Beschreibungen |

---

## 3. Graph

### 3.1 Adjazenz-Speicherung

**Primär (im Prolly-Tree):**

Schlüssel: `(src: CellId, edge_type: EdgeKind, dst: CellId)` → `EdgeAttrs`
Rückrichtung: `(dst: CellId, edge_type: EdgeKind, src: CellId)` → `()`

Beide Richtungen im selben Prolly-Tree-Keyspace (Präfix `G:` und `R:` für Graph/Reverse).

### 3.2 CSR-Segmente (abgeleitete Indexstruktur)

Für schnelle Traversal auf großen Graphen werden Compressed-Sparse-Row-Segmente als abgeleitete Indexstruktur erzeugt:

```rust
pub struct CsrSegment {
    pub segment_id: ChunkId,
    pub node_count: u32,
    pub edge_count: u32,
    /// Offset-Array: offsets[i] = Index des ersten Nachbarn von Knoten i
    pub offsets: Vec<u32>,          // Länge: node_count + 1
    /// Nachbar-Array (sortiert pro Knoten)
    pub neighbors: Vec<CellId>,    // Länge: edge_count
    /// Kantentypen (parallel zu neighbors)
    pub edge_types: Vec<EdgeKind>,
    /// Kantengewichte (parallel zu neighbors)
    pub weights: Vec<f32>,
}
```

| Parameter | Wert |
|---|---|
| CSR-Build-Trigger | > 10 000 Kanten oder > 1 000 Knoten |
| CSR-Rebuild-Trigger | > 20 % der Kanten geändert seit letztem Build |

### 3.3 Graph-Operationen

**Begrenzte k-Hop-Expansion:**

```rust
pub fn expand(
    start: CellId,
    edge_types: &[EdgeKind],
    max_hops: u8,           // max 5
    fan_out_budget: u32,    // max. besuchte Knoten pro Hop
) -> Vec<(CellId, u8 /*hops*/, f32 /*weight*/)>;
```

| Parameter | Default | Max |
|---|---|---|
| max_hops | 2 | 5 |
| fan_out_budget | 50 | 500 |

**Personalized PageRank (PPR):**

```rust
pub fn ppr(
    seeds: &[(CellId, f32)],     // Startknoten mit Gewichten
    alpha: f32,                   // Teleportationswahrscheinlichkeit
    epsilon: f32,                 // Konvergenz-Schwelle
    max_iter: u32,                // Maximale Iterationen
    top_k: usize,
) -> Vec<(CellId, f32 /*score*/)>;
```

| Parameter | Default |
|---|---|
| alpha | 0.85 |
| epsilon | 1e-6 |
| max_iter | 50 |
| top_k | 100 |

---

## 4. Temporal- und Attributindizes

### 4.1 Zeitbereichs-Index

Sortierter Prolly-Tree-Keyspace für zeitliche Abfragen:

**Schlüssel:** `(valid_from: Timestamp, CellId)` → `valid_to: Option<Timestamp>`

Operationen:
- **Intervall-Abfrage:** „Alle Cells gültig zum Zeitpunkt T" → `valid_from ≤ T AND (valid_to > T OR valid_to IS NULL)`
- **Recency-Range:** „Cells der letzten 7 Tage" → Range-Scan auf `valid_from`
- **Ergebnis:** Roaring-Bitmap der Treffer-CellIds (für Verknüpfung mit anderen Indizes)

### 4.2 Attributindizes

Invertierte Indizes mit Roaring-Bitmaps für häufige Filterdimensionen:

| Dimension | Schlüssel | Bitmap-Inhalt |
|---|---|---|
| CellKind | `(CellKind)` | Positionen aller Cells dieses Typs |
| Trust | `(TrustLabel)` | Positionen aller Cells mit diesem Trust |
| Namespace | `(NsId)` | Positionen aller Cells in diesem Namespace |
| Tags | `(Tag: String)` | Positionen aller Cells mit diesem Tag |
| Enum-Attrs | `(AttrKey, AttrValue::Enum)` | Positionen mit diesem Enum-Wert |

**Bitmap-Kompression:** Roaring-Bitmaps (Hybrid aus Sorted Arrays und Bitsets, 16-Bit-Container).

### 4.3 Attribut-Bloom-Filter

Im Extent-Hot-Head: kompakter Bloom-Filter (8 Bytes, 3 Hash-Funktionen) pro Cell über alle Attribut-Keys und Tag-Werte. Ermöglicht schnelles Ausschließen von Cells ohne Attribut-Index-Lookup.

---

## 5. Token-Summen (augmentierter Baum)

### 5.1 Struktur

Für **Sequenzen** (Dokument → geordnete Kind-Cells, Episode → Turns) speichert jeder innere Knoten die **Summe der Token** seines Teilbaums.

```rust
/// Augmentierter B-Baum über Sequenzmitglieder.
/// Jeder innere Knoten speichert Token-Summen des Teilbaums.
pub struct TokenSumNode {
    pub level: u8,
    pub children: Vec<TokenSumChild>,
}

pub struct TokenSumChild {
    pub chunk_id: ChunkId,
    pub count: u32,                                   // Anzahl Cells im Teilbaum
    pub token_sums: SmallVec<[(TokenizerId, u64); 2]>, // Token pro Tokenizer
    pub first_position: u32,                           // Position des ersten Elements
    pub last_position: u32,                            // Position des letzten Elements
}
```

| Parameter | Wert |
|---|---|
| Baum-Ordnung (Fanout) | 128 |
| Implementierung | Separater Prolly-Tree-Keyspace (Schlüssel: `(SeqId, position)`) |
| Augmentierung | Token-Summen in InternalNode.aggregates |

### 5.2 Window-Operation

```rust
/// Liefert eine zusammenhängende Menge von Cells um `anchor` mit Summe ≤ budget,
/// ohne Inhalte zu lesen (nur Baum-Aggregate).
pub fn window(
    seq: SeqId,
    anchor: CellId,
    budget: Tokens,
    tokenizer: TokenizerId,
    direction: WindowDirection,     // Before | After | Centered
) -> Vec<(CellId, u32 /*position*/, u32 /*tok_count*/)>;
```

**Komplexität:** O(log n) Knoten-Reads für ein Fenster beliebiger Größe.

**Algorithmus:**
1. Finde `anchor` im Baum → Position p.
2. Expandiere in `direction` und summiere Token über die Aggregate der inneren Knoten.
3. Stoppe, wenn Budget erreicht. Letzten Knoten ggf. teilweise einbeziehen.

### 5.3 Weitere Aggregate-Abfragen

- **Count:** Anzahl Cells in einem Positionsbereich → O(log n)
- **Budget-Check:** „Passen die Turns 40–60 in 3 000 Token?" → O(log n)
- **Prefix-Sum:** „Wie viele Token sind Turns 1–40?" → O(log n)

---

## 6. Index-Lifecycle

### 6.1 Synchrone vs. asynchrone Index-Updates

| Index | Update-Modus | Begründung |
|---|---|---|
| Prolly-Tree (Primary) | MemDelta (synchron), Fold (asynchron) | Schreibpfad-Latenz |
| Vektor-Delta | Synchron (Flat-Append ins RAM) | Kein Index-Lag für Suche |
| FTS-Postings | Synchron (In-Memory-Delta) | Kein Index-Lag für Suche |
| Graph-Delta | Synchron (In-Memory-Adjazenz) | Kein Index-Lag für Traversal |
| Token-Summen | Synchron (Aggregate-Update) | Budget-Korrektheit |
| Temporal-Index | Synchron (Bitmap-Update im RAM) | Filter-Korrektheit |
| Attribut-Index | Synchron (Bitmap-Update im RAM) | Filter-Korrektheit |
| Vamana-Segmente | Asynchron (Hintergrund-Build) | CPU-intensiv |
| FTS-Segmente | Asynchron (Hintergrund-Merge) | CPU-intensiv |
| CSR-Segmente | Asynchron (Hintergrund-Build) | CPU-intensiv |

### 6.2 Segment-Referenzen im RootSet

Jedes Segment ist über eine `SegmentRef` im RootSet verankert:

```rust
pub struct SegmentRef {
    pub chunk_id: ChunkId,       // Hash des Segments
    pub kind: SegmentKind,       // Vec | Fts | Graph
    pub embedder: Option<CompactString>,  // für Vektor-Segmente
    pub entry_count: u32,
    pub size_bytes: u64,
    pub created_at: Hlc,
}
```

### 6.3 Segment-Sharing über Forks

Bei Fork: alle SegmentRefs werden geteilt (Zero-Copy, da immutable + content-addressed). Neue Commits auf dem Fork erzeugen eigene Segment-Deltas. Segment-Compaction erzeugt neue Segmente, die den Fork-Daten entsprechen — alte Segmente bleiben für andere Branches erreichbar bis GC.

---

*Ende SPEC-05. Weiter: [SPEC-06 Context Engine](SPEC-06-CONTEXT-ENGINE.md)*
