---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "07"
---
## 7. Retrieval-Pipeline: 4-Signal-Fusion und ihre Algorithmen

### 7.1 4-Signal-Fusion

Jede Hybridsuche kombiniert bis zu vier unabhängige Signale — Vektor (HNSW-k-NN), Text (BM25/BM25F), Graph
(PPR-Traversierung inkl. Hyperkanten-Erweiterung), optional Kanten-Reinforcement (feature-gated) — über das
geschlossene `SignalKind`-Enum:

```rust
/// BEWUSST NICHT `#[non_exhaustive]` — Erweiterung erfolgt NIEMALS durch neue Varianten (H3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalKind {
    Vector,
    Text,
    Graph,
    EdgeReinforcement, // feature-gated
}

impl SignalKind {
    /// Allokationsfrei, `eq_ignore_ascii_case` statt `to_lowercase()`-Allokation.
    pub fn from_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("vector") { Some(Self::Vector) }
        else if name.eq_ignore_ascii_case("text") { Some(Self::Text) }
        else if name.eq_ignore_ascii_case("graph") { Some(Self::Graph) }
        else if name.eq_ignore_ascii_case("edgereinforcement") { Some(Self::EdgeReinforcement) }
        else { None }
    }
}
```

Die Fusion erfolgt standardmäßig über **Reciprocal Rank Fusion (RRF)**, score-blind und robust bei
Signalausfall. Score-normalisierte Fusion als Opt-in mit hartem RRF-Fallback.

**⚠️ Opus-Optimierung 1.10 — Top-k-Selektion (Stufe 1, gering):**
Volle Sortierung durch begrenzte Selektion in linearer Zeit ersetzen.

### 7.2 Graph-Signal: Forward-Push-PPR (🟢)

**Andersen-Chung-Lang Forward-Push-Algorithmus.** Exploriert nur Knoten, die signifikant zur PageRank-Masse beitragen.

Initialisierung für Seed-Knoten $s$: $r(s) = 1$, $p(s) = 0$. Für jeden Knoten $u$ mit
$\frac{r(u)}{d(u)} > \epsilon$:

1. $p(u) \leftarrow p(u) + \alpha \cdot r(u)$
2. $r(u) \leftarrow (1 - \alpha) \frac{r(u)}{2}$
3. $r(v) \leftarrow r(v) + (1 - \alpha) \frac{r(u)}{2\, d(u)}$ für alle Nachbarn $v$

Laufzeit: $O\!\left(\frac{1}{\alpha \epsilon}\right)$ — **unabhängig von der Gesamtgröße des Graphen** (P24).

```rust
pub struct PprParams {
    pub alpha: f32,           // Teleport-Wahrscheinlichkeit
    pub epsilon: f32,         // Fehlertoleranz-Schwellenwert
    pub hyperedge_decay: f32, // Default 0.85
}

pub fn forward_push_ppr(
    graph: &CsrGraph,
    seeds: &[EntityId],
    params: &PprParams,
) -> AHashMap<EntityId, f32> {
    let mut p: AHashMap<EntityId, f32> = AHashMap::new();
    let mut r: AHashMap<EntityId, f32> = seeds.iter()
        .map(|&s| (s, 1.0 / seeds.len() as f32)).collect();
    let mut queue: std::collections::VecDeque<EntityId> = seeds.iter().copied().collect();

    while let Some(u) = queue.pop_front() {
        let degree = graph.degree(u).max(1) as f32;
        let r_u = *r.get(&u).unwrap_or(&0.0);
        if r_u / degree <= params.epsilon { continue; }

        *p.entry(u).or_insert(0.0) += params.alpha * r_u;
        let residual_kept = (1.0 - params.alpha) * r_u / 2.0;
        r.insert(u, residual_kept);

        let push_share = (1.0 - params.alpha) * r_u / (2.0 * degree);

        // Binäre Nachbarn (unveränderter Hotpath).
        for (v, _w) in graph.neighbors_with_weights(u) {
            *r.entry(v).or_insert(0.0) += push_share;
            queue.push_back(v);
        }

        // 🔴 NEU (H3): Hyperkanten-Partner als virtuelle Nachbarn, Gewichtsabschlag.
        for hedge_id in graph.hyperedges_for_entity(u) {
            for role_binding in graph.hyperedge_participants(hedge_id) {
                if role_binding.entity == u { continue; }
                *r.entry(role_binding.entity).or_insert(0.0) += push_share * params.hyperedge_decay;
                queue.push_back(role_binding.entity);
            }
        }
    }
    p
}
```

### 7.3 Volltextsuche: BM25 mit Block-Max WAND und BM25F (🟢)

```rust
pub struct ResidentPostingIndex {
    postings: AHashMap<TermId, PostingList>,
    doc_lengths: Vec<u32>,
    field_lengths: AHashMap<(DocId, FieldId), u32>, // BM25F-Voraussetzung
}

pub struct Bm25fParams {
    pub k1: f32,
    pub b: f32,
    pub field_weights: AHashMap<FieldId, f32>,
}

impl ResidentPostingIndex {
    /// Block-Max WAND: Top-k ohne vollständige Postinglisten-Traversierung.
    pub fn search_topk(&self, query_terms: &[TermId], k: usize, params: &Bm25fParams) -> Vec<(DocId, f32)>;

    /// BM25F-Score für ein einzelnes Dokument, feldgewichtet.
    fn bm25f_score(&self, doc: DocId, terms: &[TermId], params: &Bm25fParams) -> f32;
}

/// Deutsche Kompositazerlegung.
pub fn decompose_german_compound(word: &str, dictionary: &CompoundDictionary) -> Vec<String>;
```

Persistenz: Residenter Index wird beim Start aus LSM-Präfix `__text:posting:` materialisiert.

**⚠️ Opus-Optimierung 1.9 — Text-Posting-Format (Stufe 1, hoch):**
Umstellung von Einzelschlüssel- auf Listenspeicherung. Delta-kodierte Dokument-IDs. Ein Lesezugriff
pro Suchbegriff statt unbegrenztem Präfix-Scan.

### 7.4 Vektorindex: HNSW + DiskANN (🟢)

```rust
pub struct HnswIndex<const D: usize> {
    layers: Vec<HnswLayer<D>>,
    entry_point: AtomicUsize,
    sq8_codebook: Sq8Codebook,
}

pub struct Sq8Codebook {
    pub min: [f32; D_MAX],
    pub max: [f32; D_MAX],
    pub clip_percentile: f32, // Default 0.999
}

impl<const D: usize> HnswIndex<D> {
    pub fn search_knn(&self, query: &[f32; D], k: usize, ef_search: usize) -> Vec<(DocId, f32)>;
    pub fn insert(&mut self, id: DocId, vector: [f32; D]) -> Result<(), IndexError>;
    pub fn delete(&mut self, id: DocId) -> Result<(), IndexError>; // native Tombstone
}
```

**NaN-sichere Distanz-Pipeline:**
```rust
/// Bitweise SIMD-Maskierung statt Branch: NaN → f32::INFINITY.
#[inline]
fn masked_l2_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    // SAFETY: `a`/`b` sind 64-Byte-aligned und exakt D Elemente lang.
    unsafe { unimplemented!() }
}
```

**DiskANN (🟢 offizieller Tier):**
```rust
pub struct DiskAnnIndex<const D: usize> {
    mmap: memmap2::Mmap,
    tombstones: scc::HashSet<DocId>, // native, kein HNSW-Fallback nötig
    tombstone_wal: TombstoneWal,
}

pub enum VectorIndexTier {
    Hnsw,
    #[cfg(feature = "experimental-diskann")]
    DiskAnn,
}
```

**Zielarchitektur „HNSW v2" (🔴 Arena-Allocator):**
```rust
pub struct HnswArena<const D: usize> {
    storage: std::sync::Arc<MmapArena>,
    head: crossbeam_epoch::Atomic<NodeRecord<D>>,
    capacity: usize,
}

pub struct NodeRecord<const D: usize> {
    pub vector: [f32; D],
    pub neighbor_offsets: [u32; MAX_M],   // Offsets statt Pointer
    pub neighbor_count: u16,
}

impl<const D: usize> HnswArena<D> {
    /// Relinking über CAS statt Mutex.
    pub fn relink(&self, node_offset: u32, new_neighbors: &[u32]) -> Result<(), IndexError>;
}
```

**SQ8-Bias-Kalibrierung (ab Fassung 2.1).** SQ8 quantisiert je Dimension mit Schrittweite
$\Delta_d = (\max_d - \min_d)/255$ und rundet auf die nächste Stufe; Werte außerhalb des Perzentil-Bereichs werden
geclippt. Das verschiebt die Distanzen **systematisch**. Es betrifft nur **absolute Schwellen** (kalibrierte
Score-Schwellen in `contextra-rank`), nicht das Ranking, weil der Offset im Mittel für alle Kandidaten gleich ist.

- Der Rundungsanteil der Fehlerenergie ist $c_q = \sum_d \Delta_d^2/12$ (`rounding_energy`, Diagnosewert;
  gemessen 4,88e-5 gegen berechnet 4,89e-5 auf synthetischen Daten mit $D=384$).
- Der **Netto-Offset** auf Normen und Distanzen ist **nicht** einfach $+c_q$. Randeffekte an den Bereichsgrenzen
  und vor allem das Perzentil-Clipping können ihn dominieren und umkehren (synthetisch, Gauß, 99,9-%-Clipping:
  etwa $-75\,c_q$, negatives Vorzeichen). Deshalb wird er **gemessen**, nicht analytisch abgeleitet.
- Kalibrierung beim Codebook-Training und bei jedem Rebuild: zwei disjunkte Stichproben (Vektoren und Queries,
  je ≥ 2 000, Seed über den `Rng`-Port, P28); Mittelwert und Standardabweichung von
  $d^2_{\text{quant}} - d^2_{\text{exakt}}$ für den asymmetrischen Pfad (Query exakt) und den symmetrischen Pfad.
- Ergebnis im Index-Header, gebunden an die Codebook-Version. Ein neues Codebook macht Schwellen der alten Version
  ungültig (Header-Prüfung).
- Anwendung: `contextra-rank` subtrahiert `l2sq_asym_mean` von quantisierten Distanzen, bevor sie mit absoluten
  Schwellen verglichen werden. Der Offset ist nur im Mittel konstant; die Streuung `l2sq_asym_std` (synthetisch
  0,37 % der Distanzskala) bleibt als Rauschterm. Kandidaten innerhalb von $\pm 2\,\sigma$ der Schwelle werden, wenn
  der Original-Float-Vektor verfügbar ist, damit neu bewertet.

```rust
pub struct Sq8Bias {
    pub codebook_version: u32,
    pub sample_pairs: u32,
    pub l2sq_asym_mean: f32, // E[d²_quant(q exakt, x quantisiert) − d²_exakt]
    pub l2sq_asym_std: f32,
    pub l2sq_sym_mean: f32,  // beide quantisiert
    pub rounding_energy: f32, // Σ_d Δ_d²/12 — nur der Rundungsanteil
}

impl Sq8Codebook {
    pub fn rounding_energy(&self, dim: usize) -> f32 {
        self.min.iter().zip(self.max.iter()).take(dim)
            .map(|(lo, hi)| { let d = (hi - lo) / 255.0; d * d / 12.0 }).sum()
    }
}
```

Die Zahlen stammen aus synthetischen Daten ($n = 30\,000$, $D = 384$), nicht aus echten Embeddings; die
Nachmessung auf Produktivdaten ist Teil von AK-12 (`sq8_bias_calibration.rs`: Bias auf Holdout-Paaren innerhalb
10 % des kalibrierten Werts, Rundungsenergie innerhalb 2 % von $\sum \Delta^2/12$, Spearman-Korrelation der
Rangfolge ≥ 0,99).

**⚠️ Opus-Optimierungen für den HNSW-Hot-Path:**

| ID | Maßnahme | Aufwand |
|---|---|---|
| 1.1 | Nachbarlisten-Auflösung ohne Allokation — Referenz statt Kopie; mittelfristig fester Stride | Gering → Hoch |
| 1.2 | Backlink-Auflösung von O(P×B) auf O(1) — HashMap pro Suche | Gering |
| 1.3 | Lock auf Quantisierer einmalig pro Suchaufruf, Distanz direkt auf Mmap-Slice | Mittel |

### 7.5 Community-Detection: Leiden (🟢 binärer Pfad)

Stern-Expansion für Hyperkanten-Projektion (🔴), Gewichte nach Konvention K (§6.6 H6). Der
`StarExpansionIterator` hält einen **Snapshot** (`Arc<GraphInner>`), keine Kopie der Hyperkanten. Ist-Zustand im
Repo: `StarExpansionIterator::new` klont alle aktiven Hyperkanten (`.cloned().collect()`).

```rust
pub struct StarEdge {
    pub participant: EntityId,
    pub virtual_node: HyperEdgeId,
    pub weight: f32, // star_weight(w(e), |e|)
}

pub struct StarExpansionIterator {
    snapshot: Arc<GraphInner>, // gehaltener Snapshot, keine Kopie
    edge_pos: usize,           // Index in snapshot.hyperedge_order (aufsteigende HyperEdgeId)
    participant_pos: usize,
}

impl Iterator for StarExpansionIterator {
    type Item = StarEdge;
    fn next(&mut self) -> Option<StarEdge> { unimplemented!() }
}
```

**Vertrag:** nur aktive (nicht tombstonierte) Hyperkanten; Hyperkanten mit $|e| < 2$ werden übersprungen
(`star_weight` liefert `None`); Reihenfolge deterministisch (aufsteigende `HyperEdgeId`, innerhalb einer
Hyperkante in Teilnehmer-Reihenfolge; §4(3)); keine Allokation pro `next()`.

### 7.6 Provenance-Tracking

**⚠️ Opus-Optimierung 1.8 — `build_provenance` Struct (Stufe 1, gering-mittel):**
Von 14 positionellen Parametern auf benannte Struct:

```rust
#[derive(Default)]
pub struct ProvenanceBuilder {
    source_doc_id: Option<DocId>,
    signal_contributions: Vec<(SignalKind, f32)>,
    fusion_mode: Option<FusionMode>,
    calibrated_threshold: Option<f32>,
}

impl ProvenanceBuilder {
    pub fn source_doc_id(mut self, id: DocId) -> Self { self.source_doc_id = Some(id); self }
    pub fn add_signal(mut self, kind: SignalKind, score: f32) -> Self {
        self.signal_contributions.push((kind, score)); self
    }
    pub fn build(self) -> Result<ProvenanceRecord, DbError> { unimplemented!() }
}
```

---

<a id="8-bandit"></a>
