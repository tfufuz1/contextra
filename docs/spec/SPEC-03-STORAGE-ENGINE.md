# SPEC-03 — Storage Engine: Physisches Layout und Formate

**Version 2.0 · November 2026 · Status: Final**
**Querverweise:** [Index](SPEC-00-INDEX.md) · [Datenmodell](SPEC-02-DATA-MODEL.md) · [Versionierung](SPEC-04-VERSIONING.md) · [Durability](SPEC-07-DURABILITY.md)

---

## 1. Objektmodell

### 1.1 Chunk

Unveränderliches, content-addressed Objekt. `ChunkId = BLAKE3(kanonische Bytes)`.

Zwei Arten:
- **Index-Knoten:** Prolly-Knoten, Vektorsegmentblöcke, FTS-Segmentblöcke, Graph-CSR-Blöcke
- **Lane-Blöcke:** Daten einer Cell-Gruppe (meta, raw, emb.q, etc.)

```rust
/// Chunk-Envelope (nicht physisch gespeichert — Extents speichern direkt die Payload)
pub struct Chunk {
    pub id: ChunkId,          // BLAKE3(payload)
    pub kind: ChunkKind,      // LeafNode | InternalNode | LaneBlock | Segment | Manifest
    pub payload: Bytes,       // rkyv-serialisierte Daten
}

#[repr(u8)]
pub enum ChunkKind {
    ProllyLeaf     = 0,
    ProllyInternal = 1,
    LaneMeta       = 2,
    LaneRaw        = 3,
    LaneEmb        = 4,
    LaneTokCount   = 5,
    LaneZoom       = 6,
    LaneTokIds     = 7,
    VecSegment     = 8,
    FtsSegment     = 9,
    GraphSegment   = 10,
    CommitObj      = 11,
    ManifestEntry  = 12,
}
```

### 1.2 Extent

Physische I/O-Einheit. Enthält mehrere Chunks derselben logischen Gruppe.

| Parameter | Wert | Begründung |
|---|---|---|
| Minimalgröße | 256 KiB | Mindestens ein sinnvoller Hot Head |
| Zielgröße | 512 KiB | Optimal für NVMe-I/O-Muster |
| Maximalgröße | 1 MiB | Begrenzung der Leseverstärkung |
| Alignment | 4 KiB | Page-Alignment für O_DIRECT |

### 1.3 Pack

Datei aus Extents. Append-only während des Schreibens, danach immutable.

| Parameter | Wert |
|---|---|
| Minimalgröße | 64 MiB |
| Zielgröße | 256 MiB |
| Maximalgröße | 1 GiB |
| Dateiendung | `.cxp` (Contextra Pack) |

---

## 2. Pack-Dateiformat

```
Pack-Datei (.cxp)
┌─────────────────────────────────────────────────────────────┐
│ Pack Header (64 Bytes, fester Offset 0)                     │
├─────────────────────────────────────────────────────────────┤
│ Extent 0                                                    │
├─────────────────────────────────────────────────────────────┤
│ Extent 1                                                    │
├─────────────────────────────────────────────────────────────┤
│ ...                                                         │
├─────────────────────────────────────────────────────────────┤
│ Extent N                                                    │
├─────────────────────────────────────────────────────────────┤
│ Pack Footer (variabel, enthält Extent-Directory)            │
└─────────────────────────────────────────────────────────────┘
```

### 2.1 Pack Header (64 Bytes)

```
Offset  Länge  Feld              Beschreibung
  0       4    magic             b"CXP\x01" (Contextra Pack, Format-Version 1)
  4       2    format_version    u16 LE = 1
  6       2    feature_flags     u16 LE (Bit 0: encrypted, Bit 1: compressed, Bits 2–15: reserviert)
  8       8    pack_id           u64 LE (monoton steigender Zähler pro Repository)
 16      32    repo_seed         [u8; 32] (Repository-Seed für Prolly-Boundary-Funktion)
 48       8    extent_count      u64 LE
 56       4    footer_offset     u32 LE (Offset des Footers ab Dateiende, in 4-KiB-Blöcken)
 60       4    header_crc        CRC32c über Bytes 0–59
```

### 2.2 Extent-Layout: Hot-Head / Cold-Tail

```
Extent (z.B. 512 KiB, 4-KiB-aligned)
┌────────────────────────────────────────────────────────────────────────────┐
│ Extent Header (128 Bytes)                                                  │
│   magic[4] | version[2] | flags[2] | extent_len[4] | cell_count[4]       │
│   hot_end[4] | cold_start[4] | lane_dir_offset[4] | lane_count[2]       │
│   compression[1] | encryption[1] | padding[96]                            │
├────────────────────────────────────────────────────────────────────────────┤
│ Lane Directory (lane_count × 16 Bytes)                                     │
│   lane_id[2] | offset[4] | length[4] | codec[1] | flags[1] | crc[4]      │
├────────────────────────────────────────────────────────────────────────────┤
│ HOT HEAD (Offset 128 + lane_dir_size ... hot_end)                          │
│   meta-Lanes aller Cells (spaltenorientiert)                               │
│   tokcount-Lanes                                                           │
│   zoom.L2-Lanes                                                            │
│   zoom.L1-Lanes                                                            │
│   emb.q-Lanes (quantisierte Embeddings)                                    │
│   Kanten-Kurzliste                                                         │
│   Trust-Labels                                                             │
│   Attribut-Bloom-Filter                                                    │
├────────────────────────────────────────────────────────────────────────────┤
│ COLD TAIL (cold_start ... extent_len - 4)                                  │
│   raw-Lanes (komprimiert, dict-basiert)                                    │
│   tok.ids-Lanes (bitgepackt)                                               │
│   emb.full-Lanes (f16/f32)                                                 │
├────────────────────────────────────────────────────────────────────────────┤
│ Extent Footer CRC (4 Bytes)                                                │
│   CRC32c über gesamten Extent-Inhalt                                       │
└────────────────────────────────────────────────────────────────────────────┘
```

**I/O-Muster:**
1. **Ranking-Read:** Erste 32–64 KiB (Hot Head) — enthält alles für Score-Berechnung.
2. **Winner-Read:** Gezielter Zugriff auf Cold Tail nur für die ≤ 12 Gewinner-Cells.
3. **Typischer Recall (200 Kandidaten, 12 Gewinner):** ~200 Head-Reads (Page Cache), ≤ 12 Tail-Reads.

### 2.3 Lane-Codierung innerhalb des Extents

Innerhalb des Extents sind Lanes **spaltenorientiert** (PAX-artig):

```
meta-Lane für N Cells:
┌──────────────────────────────────────────────────┐
│ CellId[0] | CellId[1] | ... | CellId[N-1]       │  ← Array of CellIds
│ Kind[0]   | Kind[1]   | ... | Kind[N-1]          │  ← Array of u8
│ Trust[0]  | Trust[1]  | ... | Trust[N-1]          │  ← Array of u8
│ ValidFrom[0] | ... | ValidFrom[N-1]               │  ← Array of i64
│ AttrBloom[0] | ... | AttrBloom[N-1]               │  ← Array of [u8; 8]
│ EdgeCount[0] | ... | EdgeCount[N-1]               │  ← Array of u16
│ Edges (variable, referenziert über EdgeCount)      │
└──────────────────────────────────────────────────┘
```

**Vorteile:** SIMD-freundliche Distanzberechnung über `emb.q`-Arrays, Prädikatsauswertung über `meta`-Arrays ohne Deserialisierung der gesamten Cell.

### 2.4 Pack Footer

```
Pack Footer:
┌──────────────────────────────────────────────────┐
│ Extent Directory (extent_count × 32 Bytes)       │
│   extent_offset[8] | extent_len[4] | cell_count[4] │
│   first_cell_id[16]                                │
├──────────────────────────────────────────────────┤
│ Chunk Index (sorted array of ChunkId → Location)  │
│   chunk_id[32] | extent_no[4] | offset[4] | len[4] │
├──────────────────────────────────────────────────┤
│ Bloom Filter (Ribbon-Filter über alle ChunkIds)   │
│   ribbon_filter_bytes[variabel]                    │
├──────────────────────────────────────────────────┤
│ Footer Header (32 Bytes)                          │
│   magic[4] = b"CXPF"                              │
│   extent_dir_offset[8]                             │
│   chunk_index_offset[8]                            │
│   bloom_offset[8]                                  │
│   footer_crc[4]                                    │
└──────────────────────────────────────────────────┘
```

---

## 3. Locator-Index

Globale Abbildung `ChunkId → (PackId, ExtentNo, Offset, Len)`.

### 3.1 Struktur

- **Sorted Runs:** Jede Compaction/jeder Pack-Schreibvorgang erzeugt einen sortierten Run von `(ChunkId, Location)`-Paaren.
- **Tiered Merge:** Runs werden im Hintergrund zusammengeführt (size-tiered, wie LSM L0→L1).
- **Ribbon-Filter pro Run:** False-Positive-Rate 0.1 %. Erspart das Lesen von Runs, die einen Chunk nicht enthalten.
- **In-Memory Hot-Set:** Die obersten 2 Runs (neueste) werden im RAM gehalten.

### 3.2 Parent-Hints

Prolly-Tree-Knoten speichern **physische Hints** für ihre Kinder:

```rust
pub struct ChildRef {
    pub chunk_id: ChunkId,
    pub hint: Option<LocationHint>,  // (PackId, ExtentNo) — kann veraltet sein
}
```

Wenn der Hint stimmt (90 %+ der Fälle), wird der Locator-Index übersprungen. Bei Fehlschlag: Locator-Lookup + Hint-Update im nächsten Fold.

### 3.3 Locator-Recovery

Der Locator-Index ist **rekonstruierbar** aus den Pack-Footern. Recovery: alle `.cxp`-Dateien scannen, Footer lesen, Locator-Runs aufbauen. Dauer: O(Pack-Anzahl), typisch < 1 s.

---

## 4. Gelerntes Layout (Affinity Re-Layout)

### 4.1 Motivation

Wer *zusammen gelesen* wird, soll *zusammen liegen*. Das lässt sich nicht aus dem Schlüssel ableiten, wohl aber aus Zugriffsspuren.

### 4.2 Datenquellen (Access-Trace)

| Quelle | Gewicht | Beschreibung |
|---|---|---|
| Co-Retrieval | 1.0 | Cells im selben Recall-Ergebnis |
| Struktur | 0.8 | Gleiche Episode/Dokument/Thread (starkes Prior) |
| Zeit | 0.3 | Zeitliche Nähe des Entstehens (schwaches Prior) |
| Semantik | 0.5 | Gleiche Vektorzelle (IVF-Zentroid) als Fallback bei Kaltstart |

Access-Trace wird **gesampelt** (10 % der Recalls) und **asynchron** verarbeitet.

### 4.3 Verfahren (Hintergrund-Optimizer)

1. **Co-Access-Graph aufbauen:** Gewichtete Kanten zwischen Cells, die gemeinsam gelesen werden.
2. **Partitionierung:** Extent-große Gruppen bilden (Ziel: minimiere erwartete Extent-Zugriffe pro Recall).
   - **Algorithmus:** Inkrementelles Label-Propagation-Clustering.
   - **Fallback:** METIS-artige Multilevel-Partitionierung für Kaltstart mit < 1000 Datenpunkten.
3. **Re-Layout:** Neue Extents schreiben, **nur Locator-Index umbiegen**. ChunkIds, Hashes, Commits bleiben unverändert.
4. **Exploration:** 5 % der Recalls werden mit Zufalls-/Diversitätsbeimischung bedient (verhindert Rich-get-richer).

**Invariante:** Re-Layout ist semantisch unsichtbar und jederzeit abbrechbar (neue Extents werden erst nach vollständigem Schreiben aktiviert).

### 4.4 Re-Layout-Trigger

| Trigger | Schwelle |
|---|---|
| Extent-Zugriffe pro Recall (Mittel) | > 8 (Ziel: ≤ 4) |
| Neuer Access-Trace-Datenpunkte | > 10 000 seit letztem Re-Layout |
| Zeitbasiert | Alle 24 h (falls andere Trigger nicht greifen) |
| Manuell | `ctx relayout` CLI-Befehl |

---

## 5. Kompression

### 5.1 Kompressionsverfahren pro Lane

| Lane | Verfahren | Level | Details |
|---|---|---|---|
| Text (`raw`, Zoom) | zstd mit trainiertem Wörterbuch | 3 (online), 19 (Compaction) | Wörterbuch pro Namespace, max 112 KiB |
| Token-IDs | Bit-Packing + Delta + zstd | 3 | Bit-Breite = ceil(log2(vocab_size)) |
| Embeddings (Head, int8) | keine (bereits kompakt) | — | 768 Bytes für 768-dim int8 |
| Embeddings (Head, binär) | keine | — | 96 Bytes für 768-dim binär |
| Embeddings (Tail, f16) | keine | — | 1536 Bytes für 768-dim f16 |
| Embeddings (Tail, f32) | zstd | 1 | Kompressionsgewinn ~20 % |
| Attribute | Dictionary + RLE + Frame-of-Reference | — | Spaltenweise innerhalb Extent |
| Kanten | Delta-codierte Adjazenzlisten | — | Sortiert nach (src, edge_type, dst) |
| Bloom-Filter | keine | — | Bereits kompakt |

### 5.2 Wörterbuch-Training

- **Trigger:** Alle 10 000 neue Cells pro Namespace oder manuell.
- **Trainingsmaterial:** Zufallsstichprobe von 1 000 raw-Lanes aus dem Namespace.
- **Wörterbuchgröße:** max 112 KiB (zstd-Limit für beste Effizienz).
- **Versionierung:** Wörterbuch-ChunkId im Extent-Header → alte Extents bleiben lesbar.

---

## 6. Embedding-Quantisierung

### 6.1 Formate

| Format | Precision | Größe (768-dim) | Verwendung |
|---|---|---|---|
| `binary` | 1 bit/dim | 96 Bytes | Vorfilter (Hamming-Distanz) |
| `int8` | 8 bit/dim, scalar quantization | 768 Bytes | Hot-Head, Kandidatensuche |
| `f16` | 16 bit/dim | 1 536 Bytes | Cold-Tail, Re-Ranking |
| `f32` | 32 bit/dim | 3 072 Bytes | Nur bei expliziter Anforderung |
| `pq` | Product Quantization | M×nbits Bytes | Große Segmente (> 100k Vektoren) |

### 6.2 Scalar Quantization (int8)

```rust
/// Quantisierung pro Dimension: affine Abbildung [min, max] → [0, 255]
pub struct ScalarQuantizer {
    pub min: Vec<f32>,     // pro Dimension
    pub max: Vec<f32>,     // pro Dimension
}

impl ScalarQuantizer {
    pub fn quantize(&self, v: &[f32]) -> Vec<u8> {
        v.iter().enumerate().map(|(i, &x)| {
            let range = self.max[i] - self.min[i];
            ((x - self.min[i]) / range * 255.0).round().clamp(0.0, 255.0) as u8
        }).collect()
    }
}
```

Quantisierungsparameter werden **pro Segment** trainiert und im Segment-Header gespeichert.

### 6.3 Matryoshka-Präfix

Wenn der Embedder Matryoshka-Dimensionen unterstützt (z. B. 64, 128, 256, 768):
- **Hot Head:** Erste 128 Dimensionen als int8 (128 Bytes) für Vorfilter.
- **Cold Tail:** Volle 768 Dimensionen als f16 für Re-Ranking.
- Konfigurierbar über Namespace-Policy: `matryoshka_prefix_dims = 128`.

### 6.4 Product Quantization (PQ)

Für Segmente mit > 100k Vektoren:
- **M = 32** Sub-Quantizer (768 / 32 = 24 Dimensionen pro Sub-Raum)
- **nbits = 8** (256 Centroids pro Sub-Raum)
- **Codebook:** Trainiert pro Segment, gespeichert im Segment-Header
- **Kompaktheit:** 32 Bytes pro Vektor (statt 768 Bytes int8)

---

## 7. Verzeichnisstruktur auf Disk

```
<repo-root>/
├── contextra.toml              # Repository-Konfiguration
├── manifest/
│   ├── MANIFEST-000001         # Manifest-Log (append-only)
│   ├── MANIFEST-000002         # Nächstes Manifest nach Rotation
│   └── CURRENT                 # Zeiger auf aktives Manifest
├── wal/
│   ├── WAL-000001.cxw          # WAL-Segmente
│   ├── WAL-000002.cxw
│   └── ...
├── packs/
│   ├── PACK-000001.cxp         # Pack-Dateien (immutable)
│   ├── PACK-000002.cxp
│   └── ...
├── locator/
│   ├── LOC-000001.cxl          # Locator-Index-Runs
│   └── ...
├── sidecar/
│   └── access_stats.cxs        # Zugriffsstatistiken
├── dictionaries/
│   ├── ns-0001-zstd.dict       # zstd-Wörterbücher pro Namespace
│   └── ...
├── tmp/                        # Temporäre Dateien (Compaction, Re-Layout)
└── lock                        # flock() für Single-Process-Garantie
```

### 7.1 Lock-Datei

`flock(LOCK_EX | LOCK_NB)` auf `lock`. Bei Fehlschlag: Fehler „Repository already open by another process". Im Server-Modus hält der Server den Lock.

### 7.2 Temporäre Dateien

Alle Schreibvorgänge (Pack, Locator, Manifest) schreiben zuerst in `tmp/` und benennen atomar um (`rename(2)`). Bei Crash werden Dateien in `tmp/` beim nächsten Öffnen gelöscht.

---

## 8. Verschlüsselung (At-Rest)

### 8.1 Schlüsselhierarchie

```
Master Key (extern, nie auf Disk)
    └── Repo Key = HKDF-SHA256(master_key, repo_seed, "contextra-repo-key")
            └── Pack Key = HKDF-SHA256(repo_key, pack_id, "contextra-pack-key")
                    └── Extent wird verschlüsselt mit Pack Key + Extent-Nonce
```

### 8.2 Verschlüsselungsverfahren

| Algorithmus | Nonce | Tag | Verwendung |
|---|---|---|---|
| AES-256-GCM | 12 Bytes (Extent-Counter) | 16 Bytes | Standard, Hardware-beschleunigt |
| XChaCha20-Poly1305 | 24 Bytes (zufällig) | 16 Bytes | Fallback ohne AES-NI |

### 8.3 Keyed ObjectIds

Bei verschlüsselten Repos: `ChunkId = BLAKE3(key=repo_key, encrypted_payload)`. Damit ist der Hash über den verschlüsselten Inhalt gebildet → kein Existenz-Orakel (gleicher Klartext in verschiedenen Repos hat verschiedene ChunkIds).

### 8.4 Key-Management

- **Master Key** wird beim Öffnen als Parameter übergeben (`Config::master_key(key)`)
- **Kein Key-Speicher in der Engine.** Key-Management ist Aufgabe der Anwendung.
- **Key-Rotation:** Neuer Master Key → Hintergrund-Job re-verschlüsselt alle Packs. Alte Packs werden gelöscht nach Re-Verschlüsselung + Verifikation.
- **Optionale KMS-Integration:** Über `CipherProvider`-Trait (externe Implementierung).

---

*Ende SPEC-03. Weiter: [SPEC-04 Versionierung](SPEC-04-VERSIONING.md)*
