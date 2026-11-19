> **Löschpfad v3.0:** Die löschrelevanten Abschnitte dieser Spezifikation sind mit Stand 19.11.2026 durch `docs/SPEC.md` v3.0 ersetzt bzw. präzisiert (§5, §6, §7, §9, §12, §13, §14, §16 der SPEC). Widerspricht ein Abschnitt unten der SPEC, gilt die SPEC. Abschnitte, die so markiert sind, sind überholt. Weitere Verweise: §23 SPEC (Ersetzungstabelle).

# contextra-vector — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E1 (Domänenbaustein / Ring 1)  
**Kern-Closure:** `ids`, `env`, `simd`, `durable-fs` (E0-Fundamente, max. 5)  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-vector`  
**Referenzdokumente:** `docs/SPEC.md` §2, §3, §5.3 (Schritt 4e), §6, §7; `ANALYSIS-E0-CONTRADICTIONS.md`  

---

## 1. Zweck & Architekturgarantien

`contextra-vector` implementiert die hochskalierbare Vektorindexierung und semantische Approximate Nearest Neighbor (ANN) Suche für Contextra:
- **Hierarchical Navigable Small World (HNSW):** Mehrschichtiger Graph für sub-lineare Suchzeiten mit hoher Recall-Garantie.
- **Hardwarebeschleunigung:** Nutzung der SIMD-Kerne aus `contextra-simd` (AVX2, AVX-512, NEON) mit garantiert bitidentischem skalarem IEEE-754 Fallback (INV-5).
- **Vektor-Quantisierung:** Skalare Quantisierung (SQ8, SQ4) zur Reduktion des RAM-Bedarfs bei minimalem Recall-Verlust.
- **Graph-Reparatur & Slot-Nullung (DSGVO Phase 4e):** Heuristische Neuvernetzung isolierter Nachbarknoten zur Erhaltung der Small-World-Konnektivität und physisches Nullen von Vektor-Slots im Speicher und auf Disk.

### Beseitigung von Ebenenverstößen (R1)
Die optionalen Abhängigkeiten zu `contextra-types` und `contextra-ports` sowie die Features `workspace` und `ports-compat` werden vollständig entfernt. `contextra-vector` operiert als reiner E1-Baustein ausschließlich auf `contextra-ids`, `contextra-env`, `contextra-simd` und `contextra-durable-fs`.

### Architekturgarantien (R1–R6, INV-3, INV-4, INV-5, INV-11)
1. **`#![forbid(unsafe_code)]`:** Vollständig speichersicher.
2. **Zero-Panic-Policy:** Alle Operationen liefern `VectorError`.
3. **Leere Default-Features (R3):** `default = []`.
4. **Konnektivitäts-Invariante (INV-VEC-CONNECTIVITY-1):** Das Entfernen gelöschter Vektoren darf den HNSW-Graphen nicht fragmentieren oder den Recall unbeteiligter Dokumente degradieren.
5. **Slot-Zeroize (INV-VEC-ZEROIZE-1):** Gelöschte Vektor-Slots werden vor ihrer Reallokation physisch mit Nullen überschrieben.

---

## 2. Architektur & Komponenten

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
│   │   │ SimdDistanceEngine │    │ VectorPurger      │   │   │
│   │   │ (contextra-simd)   │    │ (DSGVO Phase 4e)  │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. DSGVO-Löschkette: HNSW-Reparatur (Phase 4e)

Unter exklusivem Halt der `PurgeBarrier`:

```rust
use contextra_ids::DocId;
use std::path::Path;

pub struct VectorPurgeOutcome {
    pub vectors_purged: u64,
    pub edges_repaired: u64,
    pub bytes_reclaimed: u64,
}

pub struct VectorPurger;

impl VectorPurger {
    /// Führt die physische Reparatur des HNSW-Graphen und Vektorspeichers aus (Phase 4e).
    ///
    /// 1. Trennt alle Kanten des gelöschten Knotens auf allen HNSW-Schichten.
    /// 2. Heuristische Wiedervernetzung: Führt k-NN Suchen für Nachbarn des gelöschten Knotens
    ///    durch, um Konnektivitätsverluste auszugleichen (INV-VEC-CONNECTIVITY-1).
    /// 3. Überschreibt Vektordaten im Speicher und in der .vec-Datei mit Nullen (INV-VEC-ZEROIZE-1).
    /// 4. Synchronisiert die Datei via contextra-durable-fs.
    pub fn purge_vectors(
        index_dir: &Path,
        doc_ids: &[DocId],
    ) -> Result<VectorPurgeOutcome, VectorError> {
        // HNSW Heuristic Re-Linking und Slot-Zeroize
        Ok(VectorPurgeOutcome {
            vectors_purged: doc_ids.len() as u64,
            edges_repaired: 0,
            bytes_reclaimed: 0,
        })
    }
}
```

---

## 4. Ähnlichkeitssuche & Ausschlussfilter

```rust
use contextra_ids::DocId;
use roaring::RoaringBitmap;

pub struct VectorSearchHit {
    pub doc_id: DocId,
    pub distance: f32,
}

pub trait VectorIndex: Send + Sync {
    /// Führt k-NN Suche aus. Ausgeschlossene Vektoren (ReadGate / ExclusionFilter)
    /// werden mit O(1) übersprungen.
    fn search_knn(
        &self,
        query: &[f32],
        k: usize,
        exclusion_filter: Option<&RoaringBitmap>,
    ) -> Result<Vec<VectorSearchHit>, VectorError>;
}
```

---

## 5. Fehlerbehandlung (`VectorError`)

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum VectorError {
    #[error("Vector dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Vector contains invalid floating point values (NaN/Inf)")]
    InvalidFloat,

    #[error("Vector storage corrupted: {0}")]
    CorruptedStorage(String),

    #[error("Graph index disconnected: connectivity repair failed")]
    ConnectivityLoss,

    #[error("I/O error during index persistence: {0}")]
    Io(#[from] std::io::Error),
}
```

---

## 6. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-vector"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "HNSW vector index, SIMD distance acceleration, and quantized search for Contextra"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-simd = { workspace = true }
contextra-durable-fs = { workspace = true }
contextra-mvcc = { workspace = true }
ahash = { workspace = true }
parking_lot = { workspace = true }
roaring = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true, optional = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"
tokio = { workspace = true, features = ["full", "test-util"] }
criterion = { workspace = true }

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
tracing = ["dep:tracing"]
quantize = []
diskann = []
```

---

## 7. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml` Bereinigung:**
   - Entfernen von `contextra-types` und `contextra-ports`.
   - Features `workspace` und `ports-compat` entfernen.
   - `default = []` setzen (R3).
2. **Konnektivitäts-Heilung absichern:**
   - Heuristische Neuverlinkung von Nachbarknoten bei Vektor-Löschung verifizieren (`INV-VEC-CONNECTIVITY-1`).
3. **Slot-Zeroize:**
   - Sicherstellen, dass Vektordaten vor Slot-Freigabe physisch genullt werden (`INV-VEC-ZEROIZE-1`).
4. **Verifikation:**
   - `cargo test -p contextra-vector`
