# contextra-rank

Rangfusion (Reciprocal Rank Fusion, mRRF), Score-Normalisierung, Kalibrierung (Isotonic, Platt, Conformal) und Drift-Erkennung für das Contextra-System.

## Fusionspfade und Rollen
In `contextra-rank` existieren bewusst getrennte Fusionspfade für unterschiedliche Anforderungen:
- **`weighted_reciprocal_rank_fusion` (Produktivpfad):**
  Vollständige Signal-Rangfusion auf `SearchResult`-Instanzen mit Metadaten-Zusammenführung, Signal-Gewichtung, Resonanz-Bonus und Provenance-Aufzeichnung.
- **`reciprocal_rank_fusion` (Convenience-Wrapper):**
  Dünner Wrapper um `weighted_reciprocal_rank_fusion` mit Einheitsgewichten (`unit weights`) und Standardglättung ($k=60$).
- **`rrf_fuse` (Referenz-Orakel):**
  Kompaktes, rein mathematisches Orakel auf `ScoredItem`-Listen mit konfigurierbarem Glättungsfaktor $k$.

## Feature-Flags und Reduktion (WP-62)
- **Standardumfang (Kern):**
  RRF (`reciprocal_rank_fusion`, `rrf_fuse`), gewichtetes RRF (`weighted_reciprocal_rank_fusion`) und score-normalisierte Fusionskombinationen.
- **`experimental-rank` (nicht evaluiert):**
  Erweiterte Fusions- und Kalibrierungsverfahren, darunter mRRF (`fusion::mrrf`), Resonanz-Bonus (`resonance`), globale Graph-Fusion (`global`), Streaming-Fusion (`dibud`), Kalibrierung (`calibration/` mit Isotonic, Platt, Conformal), Laufzeit-Drift-Erkennung (`drift`) sowie erweiterte Explainability-Formate. Diese Funktionen gelten im modularen Retrieval-Kern als experimentell und nicht abschließend evaluiert.

## Copy-Paste / Standalone-Export

`contextra-rank` ist ein autarker Baustein der Klasse A ohne zwingende interne Workspace-Abhängigkeiten. Zur Extraktion in ein externes Projekt:

```bash
python3 tools/export_crate.py contextra-rank --out /pfad/zu/deinem/projekt
```

## Externe Abhängigkeiten
- `ahash`
- `serde`, `serde_json`
- `thiserror`
- `tracing`

## Minimalbeispiel (RRF-Fusion mit handgerechnetem Ergebnis)

```rust
use contextra_rank::fusion::{rrf_fuse, ScoredItem};

// Liste 1 (Text):   [doc_1 (Rang 1), doc_2 (Rang 2)]
// Liste 2 (Vektor): [doc_2 (Rang 1), doc_1 (Rang 2)]
let text = vec![
    ScoredItem::new("doc_1", 10.0),
    ScoredItem::new("doc_2", 5.0),
];
let vector = vec![
    ScoredItem::new("doc_2", 0.9),
    ScoredItem::new("doc_1", 0.8),
];

// Handrechnung mit k = 60:
// doc_1: 1/(60+1) + 1/(60+2) = 1/61 + 1/62 ≈ 0.032522475
// doc_2: 1/(60+2) + 1/(60+1) = 1/62 + 1/61 ≈ 0.032522475
// Tie-Break: "doc_1" < "doc_2"
let fused = rrf_fuse(&[text, vector], 60).unwrap();
assert_eq!(fused.len(), 2);
assert_eq!(fused[0].id, "doc_1");
assert_eq!(fused[1].id, "doc_2");
let expected = (1.0 / 61.0_f32) + (1.0 / 62.0_f32);
assert!((fused[0].score - expected).abs() < 1e-6);
```

## Garantien & Nicht-Ziele
- **Garantien:**
  - Zero-Panic (`#![forbid(unsafe_code)]`).
  - Deterministische Gleichstandsauflösung.
  - Vollständige Autarkie im Standardmodus (`--no-default-features`).
- **Nicht-Ziele:**
  - Keine eigene Vektorsuche oder Indexhaltung (gehört in `contextra-vector` / `contextra-text`).
  - Keine I/O- oder Persistenzoperationen.
  - Keine Lösch- oder DSGVO-Logik (gehört in `contextra-erasure`).
