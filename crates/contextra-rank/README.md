# contextra-rank

Autarker Baustein für Multi-Signal-Rangfusion (Reciprocal Rank Fusion, mRRF), Score-Normalisierung, Kalibrierung (Isotonic/PAVA, Platt, Conformal) und statistische Drift-Erkennung für das Contextra Cognitive OS.

## 1. Zweck und Rolle
`contextra-rank` ist ein isolierter, zustandsloser Baustein (Ring 0 / Ebene 1 / Tier stable) zur Aggregation und Qualitätsbewertung mehrdimensionaler Suchergebnisse (Dense Vector, Sparse Text / BM25, Graph und Reranking).

**Kernversprechen (Determinismus):**
- Gleiche Eingabe ⇒ bitidentische Ergebnisrangfolge.
- Bei identischen Scores: strikter, stabiler Tie-Break nach `id` lexikografisch aufsteigend.

## 2. Fusionspfade
- **`weighted_reciprocal_rank_fusion` (Produktivpfad):**
  Vollständige Signal-Rangfusion auf `SearchResult`-Instanzen mit Signal-Gewichtung, Metadaten-Zusammenführung, Resonanz-Bonus und Provenance-Aufzeichnung.
- **`reciprocal_rank_fusion` (Convenience-Wrapper):**
  Wrapper um `weighted_reciprocal_rank_fusion` mit Einheitsgewichten (`1.0`) und Standardglättung ($k=60$).
- **`rrf_fuse` (Referenz-Orakel):**
  Kompaktes, rein mathematisches Orakel auf `ScoredItem`-Listen mit konfigurierbarem Glättungsfaktor $k$.
- **`score_normalized_fusion_with_options`:**
  Score-basierte MinMax-Normalisierung und lineare Gewichtskombination mit Fallback-Prüfung.

## 3. Quickstart (Minimalbeispiel)

```rust
use contextra_rank::fusion::{rrf_fuse, ScoredItem};

let text = vec![ScoredItem::new("doc_1", 10.0), ScoredItem::new("doc_2", 5.0)];
let vector = vec![ScoredItem::new("doc_2", 0.9), ScoredItem::new("doc_1", 0.8)];

// Mit k = 60:
// doc_1: 1/(60+1) + 1/(60+2) = 1/61 + 1/62 ≈ 0.032522475
// doc_2: 1/(60+2) + 1/(60+1) = 1/62 + 1/61 ≈ 0.032522475
// Tie-Break: "doc_1" < "doc_2" (lexikografisch aufsteigend)
let fused = rrf_fuse(&[text, vector], 60).unwrap_or_default();
assert_eq!(fused.len(), 2);
assert_eq!(fused[0].id, "doc_1");
assert_eq!(fused[1].id, "doc_2");
```

Beispiel ausführen:
```bash
cargo run -p contextra-rank --example standalone --no-default-features
```

## 4. Tatsächliche interne Abhängigkeiten
Aus `cargo tree -p contextra-rank -e normal --no-default-features`:
- **Keine** internen Workspace-Abhängigkeiten im Kern (`--no-default-features`). Der Kern ist vollständig autark (Ring 0, E0/E1).
- Optionale Workspace-Brücke (`src/compat/`) hinter dem Feature `workspace`: `contextra-types`, `contextra-ports`.

## 5. Externe Abhängigkeiten
- `ahash`
- `serde`, `serde_json`
- `thiserror`
- `tracing`

## 6. Feature-Tabelle

| Feature | Standard | Beschreibung |
|---|---|---|
| `std` | Ja | Rust-Standardbibliotheksanbindung |
| `workspace` | Ja (Kompatibilität) | Brücke zu `contextra_types` und `contextra_ports` in `src/compat/` |
| `types-compat` | Nein (Alias) | Alias für `workspace` |
| `ports-compat` | Nein (Alias) | Alias für `workspace` |
| `dibud` | Ja | Discrete Budgeted Anytime Streaming Fusion (`dibud/`) |
| `exp-resonance` | Ja | Multi-Signal-Resonanz-Bonus |
| `experimental-rank`| Ja | Bündelfeature für experimentelle Verfahren |

## 7. Garantien & Nicht-Ziele
- **Garantien:**
  - Zero-Panic (`#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`).
  - Bitgenauer Determinismus bei identischer Eingabe.
  - Vollständige Standalone-Lauffähigkeit ohne Workspace.
- **Nicht-Ziele:**
  - Keine Speicherung oder Indizierung (gehört in `contextra-store` / `contextra-vector` / `contextra-text`).
  - Keine I/O- oder Persistenzoperationen.
  - Keine Löschketten-, DSGVO- oder Compliance-Logik (gehört in `contextra-erasure`).
