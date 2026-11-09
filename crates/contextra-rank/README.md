# contextra-rank

Rangfusion (Reciprocal Rank Fusion, mRRF), Score-Normalisierung, Kalibrierung (Isotonic, Platt, Conformal) und Drift-Erkennung für das Contextra-System.

Normativitätsklasse: **Klasse A** (Ring 0, autarker Rechnerkern; SPEC v4.0 §2, §5).

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

## Minimalbeispiel (RRF-Fusion)

```rust
use contextra_rank::fusion::{rrf_fuse, ScoredItem};
use contextra_rank::calibration::{calibrate_score, Calibrator, PlattScaler};

// 1. Multi-Signal-Fusion via rrf_fuse
let bm25_list = vec![
    ScoredItem::new("doc_1", 12.0),
    ScoredItem::new("doc_2", 8.5),
];
let vector_list = vec![
    ScoredItem::new("doc_2", 0.95),
    ScoredItem::new("doc_1", 0.82),
];

let fused = rrf_fuse(&[bm25_list, vector_list], 60).unwrap();
assert_eq!(fused.first().map(|r| r.id.as_str()), Some("doc_1"));

// 2. Score-Kalibrierung via calibrate_score
let platt = PlattScaler::new(1.5, -0.2);
let calibrator = Calibrator::platt(platt);
let calibrated_prob = calibrate_score(0.85, &calibrator).unwrap();
assert!(calibrated_prob >= 0.0 && calibrated_prob <= 1.0);
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
