# SPEC-05 — Indizes: Gemeinsamer Adressraum, Segmente und Watermarks

**Version 2.1 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Versionierung](SPEC-04-VERSIONING.md) · [Context Engine](SPEC-06-CONTEXT-ENGINE.md)

> Alle abgeleiteten Indizes sind **segmentbasiert, unveränderlich, content-addressed** und über das separate **`IndexManifest`** (mit `source_truth_hash` + `watermark_seq`) an den Commit gebunden — sie sind **nicht** Teil des deterministischen `RootSet`-Hashes (löst Reibungspunkt 1). Index-Verlust führt niemals zu Datenverlust; fehlende Segmente lösen sofortigen exakten **Watermark-Fallback** aus.

---

## 1. Gemeinsamer Adressraum (`Gid ↔ DocId`) & Watermark-Vertrag

### 1.1 Einheitlicher Filter-Pushdown (INV-5: ACL vor Ranking)

Alle Retrieval-Modalitäten teilen denselben globalen Adressraum **`Gid(u64)`** und pro Segment einen dichten, lokalen **`DocId(u32)`** (`0..segment_len - 1`):

1. **Vorab-Filterung im `Gid`-Raum (`Roaring64`):**
   - `ACL-Bitmap` (Vereinigung der berechtigten Gruppen-Bitmaps des Aufrufers)
   - `∩ Namespace ∩ TrustTier ∩ ¬TaintBlock ∩ CellKind ∩ ValidAt ∩ Attrs`
   - `∩ ¬dead_at(snapshot) ∩ ¬Shadow(Branch)` (Liveness & Branch-Overlay)
   - `∩ GraphFrontier` (falls Graph-Anker-Constraint aktiv)
2. **Segment-Übersetzung (`Roaring64(Gid) → Roaring32(DocId)`):**
   Jedes Index-Segment (Vektor, FTS, CSR) und jedes Truth-Extent speichert sein sortiertes `gids: [Gid; N]`-Array. Die Übersetzung erfolgt durch sortiertes Range-Slicing / Galloping-Merge in **O(min(|S|, log N))** und wird pro Query und Segment gecacht.
3. **Dieselbe `Roaring32(DocId)`-Bitmap** schränkt **Vektor-Traversal, BM25F-Postings (Block-Skipping) und Flat-Scan direkt ein** — niemals nur nachträgliche Post-Filterung.

### 1.2 Watermarks & Exakter Fallback

```rust
pub struct IndexManifest {
    pub manifest_id: ChunkId,
    pub source_truth_hash: Digest,       // Hash des RootSet, das alle Segmente abdecken
    pub watermark_seq: u64,              // Höchste indizierte globale Seq
    pub vec_segments: Vec<SegmentRef>,
    pub fts_segments: Vec<SegmentRef>,
    pub fts_corpus_stats: Vec<FtsCorpusStats>, // Globale (N, avgdl, df) je (NsId, Analyzer)
    pub csr_segments: Vec<SegmentRef>,
    pub bitmap_segments: Vec<SegmentRef>,
    pub deletion_vectors: Vec<DeletionVectorRef>,
}
```

- Eine Abfrage zum Snapshot `S` durchsucht:
  1. alle Segmente in `IndexManifest` (maskiert über `DeletionVector` + `Liveness`), **plus**
  2. alle In-Memory-Deltas (`vec_delta`, `fts_delta`, `graph_delta`) und alle noch nicht indizierten Truth-Extents mit `seq > watermark_seq` per exaktem Scan.
- **Ergebnis-Transparenz:** `RecallResult` meldet `lane_coverage ∈ [0.0, 1.0]` je Modalität und eine `PartialIndex`-Warnung, falls auf Flat-Fallback ausgewichen wurde.

---

## 2. Vektorindex (`contextra-vector`)

### 2.1 Gestufte Segment-Architektur

```
Neue Vektoren (Commits in EmbeddingSpace X)
        │
        ▼
  Branch Delta-Arena (RAM: Flat SIMD ≤ 20 000 Vektoren; bei Backlog > 20k: Mini-HNSW RAM)
        │  Asynchroner Index-Job (Work Queue / Hintergrund-Worker)
        ▼
  Unveränderliche Segmente (je EmbeddingSpace getrennt):
    ├── Flat-Segment (< 50 000 Zeilen)        → exakter SIMD-Scan über body.vec
    ├── HNSW-Segment (Profil lite/standard)    → SQ8 + Graph im RAM (~≤ 8M/Segment)
    └── Vamana/DiskANN (Profil standard/scale) → PQ/RaBitQ-Codes im RAM, 4-KiB-Sektoren auf NVMe
        │  Size-Tiered Compaction (Faktor 4–8, max. 8M Vektoren/Segment)
        ▼
  Neues Segment + Update von IndexManifestRef (ohne Änderung am RootSet!)
```

### 2.2 Parameter & Quantisierung

| Parameter | HNSW (RAM) | Vamana / DiskANN (SSD) |
|---|---|---|
| **Graph-Grad** | `M = 16`, `ef_construction = 200` | `R = 64`, `L_build = 125`, `α = 1.2` |
| **Suche** | `ef_search` adaptiv 16..256 | Beam Search (Beamweite 8, `L_search = 100`), 1 Sektor-Read je Hop |
| **Quantisierung im Graph** | `SQ8` (0.5%/99.5% Perzentile) | `SQ8` (`standard`) oder `PQ(M=32, 8-bit)` / `RaBitQ` (`scale`) |
| **Matryoshka-Präfix** | Optional (z. B. 128–256 Dim) im Hot-Head | Vorfilter über Präfix, Re-Rank auf vollem `f16` |
| **Re-Ranking** | Top-2 000 Kandidaten gegen `f16` aus `emb.full` (Cold Tail) | wie links |

- **Co-Location im Vamana-Segment:** 4-KiB-Sektor enthält pro `DocId` den quantisierten Code + adjazente `R` Nachbar-`DocId`s (~1 KiB/Vektor → 4 Knoten pro 4-KiB-Sektor).
- **Quantisierung ist Segment-identisch:** Parameter werden im Segment-Header gespeichert (`(EmbeddingSpaceId, QuantParams, Seed)`). Kein Live-Rekalibrieren; Drift wird beim Neuaufbau behoben.
- **Mehrere Spaces:** Vektoren verschiedener `EmbeddingSpace`s werden **niemals** gemischt oder direkt verglichen. Modellwechsel nutzt Dual-Space-Betrieb + Re-Embedding-Jobs.

### 2.3 Gefilterte Suche (Strategie A/B/C mit Laufzeit-Fallback)

Für gegebene Segment-Kandidatenmenge `S` (`Roaring32` aus §1.1) mit Selektivität `σ = |S| / N`:

| Strategie | Bedingung | Verfahren |
|---|---|---|
| **A: Exakter Pre-Filter** | `|S| ≤ τ_A` (Default 20 000) oder `σ < 0.01` | Direkte SIMD-Distanzberechnung (`contextra-simd`) über `S` in `emb.full` / `emb.q` |
| **B: Gefilterter Graph-Beam (ACORN)** | `0.01 ≤ σ < 0.5` | `S` wird als hartes Prädikat während Graph-Expansion geprüft; bei `σ < 0.1` mit 2-Hop-Nachbar-Brücke |
| **C: Post-Filter (Over-Fetch)** | `σ ≥ 0.5` | Standard-Beam mit Over-Fetch Faktor `ceil(1/σ) · k` |
| **Pflicht-Fallback** | Strategie B liefert nach `ef` Pops `< k` Treffer | Sofortiger Umschalt auf Strategie A |

*Geteilte Top-k-Schwelle:* Alle parallelen Segment-Scans teilen eine atomare k-te Score-Untergrenze (`AtomicU32`), um frühe Zweige abzubrechen.

### 2.4 Deletion Vectors & Compaction

Jedes Vektorsegment hat einen `DeletionVector` (`Roaring32` der ersetzten/retired `DocId`s) im `IndexManifest`. Gelöschte Knoten bleiben bis zur Compaction als **Routing-Knoten** im Vamana-/HNSW-Graph (keine Online-Graphreparatur nötig!).
- **Compaction-Trigger:** `> 8` Segmente pro Tier/Space ODER `> 30 %` gelöschte Vektoren ODER Segmentgröße `> 8 Mio.` (Split).
- Compaction schreibt neue Segmente + aktualisiert **ausschließlich `IndexManifestRef`** im Manifest-Log.

---

## 3. Volltextindex (`contextra-text`)

### 3.1 Segmentaufbau & Globale BM25F-Statistiken

Ein FTS-Segment umfasst:
1. **Term-Dictionary (FST):** Kompakter Finite State Transducer (`1–3 B/Term`) für exakte Suche, Präfixsuche und Fuzzy-Matching via Levenshtein-Automat (Edit-Distanz ≤ 2).
2. **Posting Lists (128er-Blöcke):** Nach `DocId` delta-kodiert und SIMD-bitgepackt; pro Block ein **Block-Max-Upper-Bound-Score** für **Block-Max-WAND / MaxScore** (skippt ganze Blöcke ohne Dekompression). Optionale Positionsströme für Phrasen-/Nähesuche.
3. **Field Norms:** 1-Byte-komprimierte Dokumentlängen je Feld.

**Globale BM25F-Statistiken (korrigiert pro-Segment-Verzerrung):**
Segment-Footer speichern `(doc_count, sum_field_len, per_term_df)`. Das **`IndexManifest`** aggregiert daraus **globale `(N, avgdl_field, df_term)`** über alle aktiven Segmente (+ In-Memory-`fts_delta`), sodass BM25F-Scores über Segmente und Branches hinweg **konsistent und vergleichbar** bleiben!

### 3.2 BM25F-Parameter & Analyzer-Pipeline

- **Parameter:** `k1 = 1.2`, `b = 0.75` (konfigurierbar).
- **Feldgewichte:** `title` / `zoom.L2` (`3.0`), `tags` (`2.0`), `heading_path` (`1.5`), `body` (`1.0`), `edge_labels` (`0.5`).
- **Analyzer-Pipeline** (ID + Parameter sind Teil des Segmentschlüssels; Änderung erzwingt kontrollierten Neuaufbau):
  - Unicode-Segmentierung (UAX #29), Lowercasing, Umlaut-/ß-Normalisierung, Spracherkennung je Chunk, Snowball-Stemming (`de`, `en`, ...), **deutsches Decompounding** (DP-Zerlegung mit Fugenelementen), CamelCase-/Snake-Case-Splitting & N-Gramme für Code und Bezeichner.

---

## 4. Graphindex (`contextra-graph`)

### 4.1 Truth & Abgeleitete CSR-Segmente

- **Truth (`RootSet.graph`):** Prolly-Tree mit Präfixen `G:(src_gid, edge_kind, dst_gid, edge_id) → EdgePayload` und `R:(dst_gid, edge_kind, src_gid, edge_id) → ()`. Knotenidentität ist stets `Gid(u64)`.
- **Referentielle Sichtbarkeit:** Das `retire` einer Cell löscht ihre Kanten **nicht** kaskadierend. Stattdessen prüft jede Graph-Traversal die Sichtbarkeit beider Endpunkte gegen die aktive Snapshot-Liveness-Maske (und Merge-Gate `G1`/`G3` meldet *dangling edges*).
- **Abgeleitete CSR-Segmente (`IndexManifest.csr_segments`):** Gebaut bei `> 10 000` Kanten oder `> 20 %` Änderung; Elias-Fano-codierte Offsets, delta+bitpack-codierte Nachbar-`Gid`s, parallele `EdgeKind`-/Gewichts-/Valid-Spalten und **Skip-Index für Hub-Knoten (`degree > 10 000`)**.

### 4.2 Graph-Operatoren mit Pflichtlimits

Unbegrenzte Traversals sind unzulässig (Schutz vor runaway Agent-Queries):

| Operator | Parameter (Defaults & Harte Limits) | Verfahren |
|---|---|---|
| **k-Hop-Expansion** | `max_hops = 2` (Max `5`), `fan_out_budget = 50` (Max `500`), `max_visited = 100 000`, `max_paths = 10 000`, `timeout = 200 ms` | Richtungsoptimierte BFS über CSR ⊕ `graph_delta`, Frontier als `Roaring64`, Temporal-/EdgeKind-Masken. Bei Limit-Erreichen: Teilergebnis mit `truncated = true`. |
| **Personalized PageRank** | **Einziger Standard:** `Forward-Push PPR` (`α = 0.85`, `ε = 1e-4`, `max_iter = 50`, `top_k = 100`) | Pfad-Konfidenz-Dämpfung `C(P) = (Π conf(e_i)) · γ^(k-1)` (`γ = 0.9`) optional als Kantengewicht; Hub-Dämpfung. |
| **Leiden-Communities** | Deterministischer Batch-Job (mit festem Seed) | Community-IDs fließen als Boost (`× 1.2`) in Retrieval und speisen hierarchische `Summary`-Cells (`level = 3`). |

---

## 5. Temporal-, Attribut- und Token-Summen-Indizes

### 5.1 Temporal- & Attribut-Bitmaps

- **Zeitbereichs-Prolly-Keyspace:** `(valid_from, gid) → valid_to` im Truth Layer für exakte Intervallabfragen.
- **Roaring64-Bitmaps** in `IndexManifest.bitmap_segments` + RAM-Delta für: `NsId`, `CellKind`, `TrustTier`, `TaintFlags`, `ACL-Gruppen`, `Tags`, `Enum`-Attribute.
- **Extent-Hot-Head-Bloom (8 B/Cell):** Schnelles Vorab-Ausschließen beim sequentiellen Extent-Scan.

### 5.2 Augmentierter Token-Summen-Baum

In `RootSet.sequences` speichert jeder innere Prolly-Knoten (Ziel-Fanout ~128) die aggregierten **Token-Summen pro registriertem `TokenizerId`**, Zellzahl und `first_position..last_position`:
- **`window(seq, anchor, budget, dir)`** sammelt zusammenhängende Kontextfenster in **O(log n)** Knoten-Reads, ohne vorab `raw`-Inhalte aus dem Cold Tail zu lesen.

---

## 6. Index-Lifecycle & Konsistenz-Tabelle

| Struktur | Lebensort | Update-Modus | Crash-Schutz |
|---|---|---|---|
| `primary` / `graph` / `sequences` / `aliases` / `policies` / `conflicts` / `provenance` | **`RootSet` (Truth)** | `MemDelta` (sync) → Prolly-Fold (async) | WAL Group Commit |
| `vec_delta` / `fts_delta` / `graph_delta` / `bitmap_delta` | **Branch RAM** | Synchron beim Commit publiziert | Aus WAL/MemDelta rekonstruiert |
| `Vamana` / `HNSW` / `FTS` / `CSR` / `Bitmap` Segmente | **`IndexManifest` (Side-Ref)** | Asynchroner Background-Job | Jederzeit aus Truth rekonstruierbar; Flat-Fallback aktiv |

---

*Ende SPEC-05. Weiter: [SPEC-06 Context Engine](SPEC-06-CONTEXT-ENGINE.md)*
