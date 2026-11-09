# contextra-engine

Transaktions-, Index- und Ausführungsorchestrator für hybrides Retrieval.

## Zweck
`contextra-engine` orchestriert LSM-Speicher (`contextra-store`), Vektor-Index (`contextra-vector`), BM25-Volltextsuche (`contextra-text`) und Graph-Indizierung (`contextra-graph`) zu einer einheitlichen, transaktional konsistenten Retrieval-Engine mit reziproker Rangfusion (RRF via `contextra-rank`).

## Reifegrad
- **Status:** Beta (Konsolidierung und Entkopplung)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Interne Abhängigkeiten
Normale interne Abhängigkeiten (`cargo tree -p contextra-engine -e normal --depth 1`):
- `contextra-types`
- `contextra-ports`
- `contextra-mvcc`
- `contextra-crypto`
- `contextra-store`
- `contextra-vector`
- `contextra-text`
- `contextra-checkpoint`
- `contextra-rank`
- `contextra-graph`

## Features
- `default = []`: Minimaler Kernbau ohne externe Inferenz oder experimentelle Erweiterungen.
- `onnx`: Anbindung an `contextra-infer-onnx`.
- `sandbox`: Anbindung an WebAssembly-Sandbox (`contextra-sandbox`).
- `background-maintenance`: Erweiterte Entropie- und Decay-Hintergrundbereinigung.
- `experimental-diskann`: DiskANN-Indexunterstützung über `contextra-vector`.

## Minimalbeispiel (Doctest)
```rust
use contextra_engine::ContextraConfig;

let config = ContextraConfig {
    dimension: 128,
    max_elements: 10_000,
    ..Default::default()
};
assert_eq!(config.dimension, 128);
```

## Entnahme-Rezept
Zur isolierten Nutzung von `contextra-engine`:
1. `contextra-engine` als Abhängigkeit einbinden.
2. Standardmäßig ohne Features verwenden (`default-features = false`).
3. Embedder via `contextra-ports::TextEmbeddingEngine` injizieren oder Dokumente direkt mit vorgefertigten Vektoren einfügen.

## Bekannte Grenzen
- **Tokio-Laufzeit:** `contextra-engine` erfordert zur asynchronen Orchestrierung eine aktive `tokio`-Laufzeitumgebung.
- **Hintergrund-Worker:** Expiry-Worker startet bei Collection-Erstellung automatisch; feingranulare Worker-Abschalter befinden sich im Übergang (siehe `PENDING_API_CHANGES.md`).
- **Inferenz-Entkopplung:** Die Engine führt keine interne Inferenz aus; Vektoren müssen vom Aufrufer bereitgestellt oder über injizierte Embedder erzeugt werden.
