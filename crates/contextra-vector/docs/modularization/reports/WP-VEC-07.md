# Modularisierungs-Report WP-VEC-07: DiskANN-Isolation (Vorbereitung der Auslagerung S-80)

- **Crate:** `contextra-vector`
- **Paket:** WP-VEC-07
- **Bearbeiter:** Google Jules
- **Datum:** 2026-11-11
- **Status:** In Bearbeitung

---

## 1. Baseline

### 1.1 Matrix-Ergebnisse (Vorher)
- Modularitäts-Score M: 67 (Ebene E1, Bridge Refs: 14)
- Determinismus-Guard Audit: 6 Fundstellen in `contextra-vector` (in `src/fixed_hasher.rs`)
- Workspace-Status: `crates/contextra-store/Cargo.toml` enthält eine nicht-optionale `dep:contextra-types`-Deklaration im unvollständigen Decoupling-Zustand (`15ef710776`), die workspace-weites `--locked` blockiert. Dies ist extern vorbestehend und unter Übergaben an Z dokumentiert.

### 1.2 Ist-Zustand `src/diskann/` vor Refactoring
- Dateien: 11 Quelltext-Dateien (inkl. `tests.rs`)
- LOC-Übersicht:
  - `tests.rs`: 1087 Zeilen
  - `build.rs`: 819 Zeilen
  - `persistence.rs`: 690 Zeilen
  - `search.rs`: 398 Zeilen
  - `types.rs`: 301 Zeilen
  - `predicate_augmented.rs`: 274 Zeilen
  - `format.rs`: 194 Zeilen
  - `config.rs`: 78 Zeilen
  - `filtered.rs`: 74 Zeilen
  - `integrity.rs`: 45 Zeilen
  - `mod.rs`: 23 Zeilen

---

## 2. Kopplungskarte (Aufgabe 1)

### 2.1 Ausgehende Referenzen von `diskann/**` auf Nicht-`diskann`-Module

| Datei : Zeile | Ziel-Item | Art der Nutzung |
|---|---|---|
| `build.rs:12` | `crate::distance::compute_distance_trusted` | Distanzberechnung beim Vamana-Graphaufbau |
| `build.rs:13` | `crate::error::{Result, VectorError}` | Fehlerbehandlung |
| `build.rs:14` | `crate::types::DocId` | Dokumenten-ID |
| `build.rs:648` | `crate::quantize::ScalarQuantizer::train` | Training des Skalar-Quantisierers |
| `config.rs:6` | `crate::ComputePool` | Thread-Pool-Referenz für Parallelisierung |
| `config.rs:7` | `crate::types::{DistanceMetric, DocId}` | Metrik & Dokumenten-ID |
| `filtered.rs:6` | `crate::error::{Result, VectorError}` | Fehlerbehandlung |
| `filtered.rs:7` | `crate::types::{DocId, ScoredDocument}` | Dokumenten-ID & Suchergebnis |
| `filtered.rs:21` | `crate::error::VectorError::CapabilityUnsupported` | Fehlerdokumentation |
| `filtered.rs:28,31,39,59,64` | `crate::types::MAX_SEARCH_K` | Schranke für Suchtiefe |
| `format.rs:6` | `crate::error::{Result, VectorError}` | Fehlerbehandlung |
| `integrity.rs:3` | `crate::error::{Result, VectorError}` | Fehlerbehandlung |
| `persistence.rs:14` | `crate::error::{Result, VectorError}` | Fehlerbehandlung |
| `persistence.rs:15` | `crate::types::DocId` | Dokumenten-ID |
| `persistence.rs:233` | `crate::hnsw::HnswIndex` | HNSW-Fallback-Typ |
| `persistence.rs:234,238` | `crate::hnsw::HnswConfig` | Konfiguration des HNSW-Fallbacks |
| `persistence.rs:240` | `crate::hnsw::HnswIndex::try_new` | Instanziierung des HNSW-Fallbacks |
| `persistence.rs:341` | `crate::quantize::ScalarQuantizer` | SQ8-Quantisierer-Wiederherstellung |
| `persistence.rs:637` | `crate::types::{DistanceMetric, DocId}` | Metrik & Dokumenten-ID |
| `predicate_augmented.rs:8` | `crate::error::{Result, VectorError}` | Fehlerbehandlung |
| `predicate_augmented.rs:9,194` | `crate::types::{DistanceMetric, DocId, ScoredDocument}` | Suchtypen |
| `predicate_augmented.rs:33,36` | `crate::types::MAX_SEARCH_K` | Schranke für Suchtiefe |
| `search.rs:10` | `crate::distance::compute_distance_trusted` | Distanzberechnung beim Greedy-Routing |
| `search.rs:11` | `crate::error::{Result, VectorError}` | Fehlerbehandlung |
| `search.rs:12` | `crate::types::{DistanceMetric, DocId, ScoredDocument}` | Suchtypen |
| `search.rs:227,231` | `crate::types::MAX_SEARCH_K` | Schranke für Suchtiefe |
| `tests.rs:15` | `crate::error::{Result, VectorError}` | Fehlerbehandlung |
| `tests.rs:16` | `crate::types::{DistanceMetric, DocId, TxId}` | Testtypen |
| `types.rs:9` | `crate::ComputePool` | Thread-Pool-Referenz |
| `types.rs:11` | `crate::error::{Result, VectorError}` | Fehlerbehandlung |
| `types.rs:12` | `crate::types::DocId` | Dokumenten-ID |
| `types.rs:37` | `crate::quantize::ScalarQuantizer` | SQ8-Zustand |
| `types.rs:46` | `crate::hnsw::HnswIndex` | Fallback-Index |

### 2.2 Eingehende Referenzen von außerhalb auf `diskann/**`
- `crates/contextra-vector/src/compat/diskann_vector_index.rs`:
  - `crate::diskann::format::compute_adaptive_flush_threshold`
  - `crate::diskann::types::DiskAnnIndex`
- `crates/contextra-vector/src/lib.rs`:
  - `pub mod diskann` (unter `experimental-diskann`)
  - `pub use diskann::{DiskAnnConfig, DiskAnnFallbackPolicy, DiskAnnIndex}`
- Externe Integrationstests und Crates (`contextra-db`, `contextra-engine`):
  - Nutzen ausschließlich `contextra_vector::diskann::{DiskAnnConfig, DiskAnnFallbackPolicy, DiskAnnIndex}`.

---


## 3. Aufgaben-Status und Änderungen

| Aufgabe | Status | Geänderte Dateien | Prüfbefehl & Ergebnis |
|---|---|---|---|
| 1. Kopplungskarte erstellen | Erledigt | `WP-VEC-07.md` | `python3 tools/crate_coupling_report.py` → grün |
| 2. HNSW-Kopplung bündeln | Erledigt | `hnsw_bridge.rs`, `types.rs`, `search.rs`, `mod.rs` | `grep -rn "crate::hnsw" src/diskann` → grün (nur `hnsw_bridge.rs`) |
| 3. Dateizugriffe bündeln & Durability | Erledigt | `io.rs`, `build.rs`, `persistence.rs`, `types.rs` | `grep -rn "std::fs" src/diskann` → grün (nur `io.rs`) |
| 4. build, persistence, tests aufteilen | Erledigt | `build/*`, `persistence/*`, `tests/*` | `wc -l` → alle Dateien ≤ 400 Zeilen |
| 5. Determinismus & Fehlerhärtung | Erledigt | `types.rs`, `format.rs`, `tests/wp07_diskann_formats.rs` | `cargo test --test wp07_diskann_formats` → grün |
| 6. S-80 Bereitschaft | Erledigt | `WP-VEC-07.md` | Vollständige Analyse dokumentiert |

---

## 4. S-80 Bereitschaft (Vorbereitung der Auslagerung in `contextra-vector-diskann`)

### 4.1 Verbleibende Kopplungen (Nachher)

Alle verbleibenden Referenzen aus `src/diskann` auf externe Module des Crates sind streng auf 5 Kernbereiche begrenzt:

1. **Fehlerbehandlung:** `crate::error::{Result, VectorError}`
   - Wird bei Auslagerung durch ein eigenes `DiskAnnError` oder das neutrale `ContextraError` ersetzt.
2. **Domänentypen:** `crate::types::{DocId, DistanceMetric, ScoredDocument, MAX_SEARCH_K}`
   - Standard-Domänentypen, die bei Auslagerung aus `contextra-types` bezogen werden.
3. **Distanzberechnung:** `crate::distance::compute_distance_trusted`
   - Kann direkt über `contextra-simd` / `DistanceKernel` bezogen werden.
4. **Quantisierung:** `crate::quantize::ScalarQuantizer`
   - SQ8-Quantisierungskomponente für komprimierte DiskANN-Speicherung.
5. **HNSW-Brücke:** `crate::diskann::hnsw_bridge::*`
   - Einziger Kopplungspunkt zu `crate::hnsw::*`. Bei Auslagerung als optionales Feature (`features = ["hnsw-fallback"]`) oder über Dependency Injection abbildbar.

### 4.2 Empfohlene öffentliche Trait-Grenze
Für das ausgelagerte Crate `contextra-vector-diskann` wird folgende Trait-Grenze empfohlen:
```rust
pub trait DiskAnnEngine: Send + Sync {
    async fn insert(&self, id: DocId, embedding: &[f32]) -> Result<()>;
    async fn search(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>>;
    async fn persist_delta(&self) -> Result<()>;
    async fn build(&self, vectors: &[Vec<f32>], ids: &[DocId]) -> Result<()>;
}
```

### 4.3 LOC-Vergleich Vorher / Nachher

| Komponente | Vorher (Zeilen) | Nachher (Zeilen, aufgeteilt) | Status |
|---|---:|---:|---|
| `build.rs` | 819 | `build/mod.rs` (147), `build/pending_wal.rs` (223), `build/tombstone_wal.rs` (173), `build/flush.rs` (223) | Aufgeteilt, alle ≤ 250 LOC |
| `persistence.rs` | 690 | `persistence/mod.rs` (10), `persistence/delta.rs` (72), `persistence/loader.rs` (270), `persistence/writer.rs` (115), `persistence/fallback.rs` (22), `persistence/node.rs` (172) | Aufgeteilt, alle ≤ 270 LOC |
| `tests.rs` | 1087 | `tests/mod.rs` (9), `tests/config_tests.rs` (242), `tests/persistence_tests.rs` (273), `tests/build_search_tests.rs` (327), `tests/recall_tests.rs` (168), `tests/wal_tests.rs` (116) | Aufgeteilt, alle ≤ 330 LOC |
| `hnsw_bridge.rs` | - | 58 | Neu angelegt |
| `io.rs` | - | 133 | Neu angelegt |

---

## 5. Übergaben an Z (Befunde außerhalb des Schreibrechts)

1. **`crates/contextra-store/Cargo.toml:17`:**
   - **Befund:** `feature "workspace" includes "dep:contextra-types", but "contextra-types" is not an optional dependency`.
   - **Zeile:** 17 in `crates/contextra-store/Cargo.toml`.
   - **Ursache:** Merge in Commit `57b0955cb` führte zu unvollständigem Feature-Gating in `contextra-store`.
   - **Lösung:** `contextra-types` und `contextra-ports` unter `[dependencies]` in `crates/contextra-store/Cargo.toml` auf `optional = true` setzen oder `workspace = [...]` bereinigen.
2. **`crates/contextra-vector/src/diskann/vector_index_impl.rs`:**
   - **Befund:** Verwaistes Duplikat, das nicht in `mod.rs` eingebunden ist und veraltete Imports enthält.
   - **Besitz:** WP-VEC-08.

---

