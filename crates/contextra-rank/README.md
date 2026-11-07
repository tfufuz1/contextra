# contextra-rank

Ranking-, Fusion- und Score-Kalibrierungs-Algorithmen.

Normativitätsklasse: **Rand `[R]`** (Ring 0, `experimental`; §6.1, §6.9, §12, `capabilities.toml`).

## Rolle in einem Satz
`contextra-rank` bietet Score-Kalibrierung (Platt, Isotonisch, Conformal) und Ergebnis-Anreicherungen zur Verfeinerung von Retrieval-Rankings.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Ranking & Kalibrierung)
- **Klasse:** Rand `[R]`
- **Reifegrad:** `experimental` (abweichend in `capabilities.toml` als `stable` geführt, siehe `docs/status/ABW-L3.md`)
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`)

## Features & Garantien
- **Zero-Panic:** Strikte Absicherung (`#![forbid(unsafe_code)]`, keine Panics/Unwraps im Produktivcode).
- **Deterministische Fusion:** Deterministische Tie-Breaker für RRF und mRRF (Score absteigend, DocId aufsteigend).
- **Reine Funktionen & Autarkie:** `fusion` und `metrics` arbeiten rein funktional ohne I/O, Netzwerk, Systemzeit oder externe Laufzeit.
- **Isotonische & Platt-Kalibrierung:** PAVA-Algorithmus zur Score-Kalibrierung sowie Expected Calibration Error (ECE).
- **Retrieval-Metriken:** Reciprocal Rank, MRR, Precision@k, Recall@k und nDCG@k.

## Einbettungsbeispiel

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
