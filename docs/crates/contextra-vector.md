# contextra-vector — Crate-Spezifikation (HNSW-Vektorindex)

**Ebene:** E1b (Kern-Closure: `ids`, `env`, `simd`, `durable-fs`)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-vector`  
**Referenz:** `docs/SPEC.md` §2, §3, §5, §6, §7

---

## 1. Zweck & Systemrolle

`contextra-vector` implementiert die hochperformante Vektorindexierung und semantische Ähnlichkeitssuche für Contextra:
- **Hierarchical Navigable Small World (HNSW):** Mehrschichtiger Graph für sub-lineare Approximate Nearest Neighbor (ANN) Suche.
- **Hardware-Beschleunigung:** Vektor-Distanzberechnungen (Cosine, L2, Dot-Product) über SIMD-Intrinsics via `contextra-simd` (AVX2, AVX-512, NEON) mit automatischem skalarem Fallback.
- **Quantisierung:** Optionale Skalare Quantisierung (SQ8, SQ4) zur signifikanten Reduzierung des RAM-Bedarfs.
- **Physische Graph-Reparatur:** Beseitigung gelöschter Vektoren unter Erhaltung der Konnektivität des Small-World-Graphen im Rahmen der DSGVO-Löschkette.

---

## 2. Architektur & Datenstrukturen

```
┌─────────────────────────────────────────────────────────────┐
│                       contextra-vector                      │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                      HnswIndex                      │   │
│   │                                                     │   │
│   │   Layer 2:  (•) ───────────────► (•)                │   │
│   │              │                    │                 │   │
│   │   Layer 1:  (•) ───► (•) ───────► (•) ───► (•)      │   │
│   │              │        │           │        │        │   │
│   │   Layer 0:  (•) ─► (•) ─► (•) ─► (•) ─► (•) ─► (•)  │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ VectorStorage      │    │ ExclusionFilter   │   │   │
│   │   │ (Flat / Quantized) │    │ (RoaringBitmap)   │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ SimdDistanceEngine │    │ ErasureLayer Impl │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Schutz personenbezogener Vektoren (Klasse D)

- **Inversions-Risiko:** Vektoren und Embeddings sind Ableitungen (Klasse D). Sie können durch Embedding-Inversion-Angriffe rekonstruierbar sein und gelten daher bei Personenbezug als schutzwürdig.
- **Adressierung & Kapselung:**
  - Vektor-IDs sind numerische Indizes, die intern über eine geschützte Map auf `DocId` oder `PersonalRef` (Klasse P) abgebildet werden.
  - Vektor-Nutzlasten (Metadata / Payload) werden nicht im Vektorindex im Klartext gehalten, sondern verweisen auf den LSM-Store (`contextra-store`).

---

## 4. DSGVO-Löschkette: ErasureLayer-Implementierung

`contextra-vector` implementiert `contextra_ports::erasure::ErasureLayer` (Priorität 4e — Vektor-Reparatur):

```rust
impl ErasureLayer for HnswIndex {
    fn id(&self) -> LayerId {
        LayerId(*b"CTX_VEC_HNSW__01")
    }

    fn key_coverage(&self) -> KeyCoverage {
        KeyCoverage::Full
    }

    fn resolve(&self, target: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>>;
    fn on_gate(&self, scope: &LayerScope) -> BoxFuture<'_, Result<(), String>>;
    fn purge(&self, scope: &LayerScope, cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>>;
    fn witness(&self, scope: &LayerScope, sc: &Scanner<Armed>) -> BoxFuture<'_, Result<LayerWitness, String>>;
}
```

### 4.1 Die 4 Phasen in `contextra-vector`

#### 1. `resolve(target)`
- Mappt die betroffenen `DocId`s oder Schlüsselbereiche auf die internen numerischen Vektor-IDs (`InternalId`).
- Erzeugt einen `LayerScope` mit der Menge der zu entfernenden Knoten.

#### 2. `on_gate(scope)`
- Setzt die ermittelten Vektor-IDs in einer atomaren `RoaringBitmap` (`excluded_vectors`).
- Ab diesem Zeitpunkt überspringt jede Ähnlichkeitssuche (`search_knn`) diese Knoten mit $O(1)$ Overhead.

#### 3. `purge(scope, cx)` — Physische Graph-Reparatur
Wird unter `PurgeBarrier::purge` ausgeführt:
- **Knoten-Isolation:** Für jeden betroffenen Knoten werden alle eingehenden und ausgehenden Kanten auf allen HNSW-Ebenen abgetrennt.
- **Graph-Konnektivitäts-Heilung:**
  - Für alle Nachbarknoten des gelöschten Knotens wird das Heuristik-Linking erneut aufgerufen, um alternative Verbindungen herzustellen.
  - Dadurch bleibt die Navigierbarkeit des Small-World-Netzwerks mathematisch stabil.
- **Vektordaten-Nullung:** Die Speicher-Slots in den Vektordateien (`.vec`) werden mit Nullen überschrieben (`zeroize`) und mittels `fsync` synchronisiert (`contextra-durable-fs`).
- **Index-Kompaktion:** Übersteigt der Anteil gelöschter Knoten 20%, wird im Hintergrund ein sequentielles Re-Packing der Vektordatei durchgeführt.

#### 4. `witness(scope, sc)` — Verifikation & Messung
- Führt eine gezielte Lookup-Abfrage nach den gelöschten Vektor-IDs im Index durch.
- Prüft die binären Segmente auf Abwesenheit der Vektoren.
- Liefert `LayerWitness { residue_matches: 0, ... }`.

---

## 5. Synchroner Lesepfad: ReadGate-Integration

Während der Suche konsultiert der HNSW-Graph-Traversal-Algorithmus den synchronen `ReadGate`:

```rust
pub fn search_knn(&self, query: &[f32], k: usize) -> Result<Vec<VectorHit>, ContextraError> {
    // 1. Initialisierung PriorityQueue
    // 2. Traversal Layer L_max bis Layer 0
    // 3. Auf Layer 0: Für jeden Kandidaten Prüfung gegen In-Memory-Maske und ReadGate
    for candidate in candidates {
        if self.excluded_mask.contains(candidate.id) {
            continue;
        }
        if self.read_gate.check_internal(candidate.id) == GateDecision::Erased {
            continue;
        }
        // Score-Berechnung via SIMD
    }
    // 4. Auffüllen auf k (Z-E8 konform)
}
```

---

## 6. Invarianten & Qualitätsstandards

1. **`#![forbid(unsafe_code)]`:** Alle Low-Level SIMD-Befehle sind strikt in `contextra-simd` gekapselt.
2. **Determinismus (INV-5):** Identische Abfragen auf identischen Indexständen liefern exakt identische Trefferlisten. Bei identischen Distanzen erfolgt der Tie-Break nach `DocId` aufsteigend.
3. **Keine Panics:** Fehlerhafte Vektordimensionen oder NaNs in Eingabevektoren führen zu `Err(ContextraError::InvalidInput)`, niemals zu Panics.
