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
use ahash::AHashSet;
use contextra_rank::fusion::{reciprocal_rank_fusion, SearchResult};
use contextra_rank::metrics::reciprocal_rank;

// Suchergebnisse aus zwei Signalquellen definieren
let bm25_set = vec![SearchResult {
    id: "doc_1".to_string(),
    score: 12.0,
    metadata: None,
    matched_signals: vec!["bm25".to_string()],
    provenance: None,
}];
let vector_set = vec![SearchResult {
    id: "doc_1".to_string(),
    score: 0.95,
    metadata: None,
    matched_signals: vec!["vector".to_string()],
    provenance: None,
}];

// RRF-Fusion
let fused = reciprocal_rank_fusion(vec![bm25_set, vector_set], 10);
assert_eq!(fused.first().map(|r| r.id.as_str()), Some("doc_1"));

// Evaluierung
let mut relevant = AHashSet::new();
relevant.insert("doc_1");
let ranked_ids: Vec<&str> = fused.iter().map(|r| r.id.as_str()).collect();
let rr = reciprocal_rank(&ranked_ids, &relevant);
assert_eq!(rr, 1.0);
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
