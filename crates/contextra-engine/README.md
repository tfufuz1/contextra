# contextra-engine

Transaktions-, Index- und Ausführungsorchestrator (Ebene E2 Integration, Klasse C, Ring 3).

## Rolle & Zweck
`contextra-engine` verbindet unabhängige Speicher- und Index-Bausteine (LSM-Store, HNSW-Vektorindex, BM25-Volltext, CSR-Graph) zu einer konsistenten Transaktions- und RAG-Ausführungsumgebung.

```
                           +------------------------+
                           |    contextra-engine    |
                           +-----------+------------+
                                       |
                   +-------------------+-------------------+
                   |                   |                   |
                   v                   v                   v
           +---------------+   +---------------+   +---------------+
           | StorageEngine |   |  VectorIndex  |   |   TextIndex   |
           | (contextra-   |   | (contextra-   |   | (contextra-   |
           |    store)     |   |    vector)    |   |    text)      |
           +---------------+   +---------------+   +---------------+
                                       |
                                       v
                               +---------------+
                               |  GraphIndex   |
                               | (contextra-   |
                               |    graph)     |
                               +---------------+
```

## Baukasten-Einordnung & Status
- **Ebene:** Ebene E2 (Integration)
- **Klasse:** Klasse C (Integrations- und Kompositionsschicht)
- **Ring:** Ring 3 (Ausführungsorchestrator)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Zero-Panic-Doktrin:** Vollständig konform im Produktivcode

## Tatsächliche Abhängigkeiten (laut `cargo tree`)
- **Kern-Closure (`--no-default-features`):**
  - `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-checkpoint`
  - `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-rank`
  - Transitive Mikrokerne: `contextra-ids`, `contextra-env`, `contextra-sys`, `contextra-crypto`
- **Optionale Feature-Abhängigkeiten:**
  - `contextra-adapt` (über `adaptive-candidate-pool-sizing`)
  - `contextra-infer-onnx` (über `onnx`)
  - `contextra-sandbox` (über `sandbox`)
- **Entwicklungs-Abhängigkeiten:**
  - `contextra-ingest`, `contextra-testkit`, `tokio`, `tempfile`, `proptest`, `loom`, `wat`, `sha2`

## Feature-Tabelle

| Feature | Zweck | Abhängigkeiten / Weiterleitungen |
|---|---|---|
| `onnx` | In-Process ONNX Text-Einbettung | `contextra-infer-onnx/onnx` |
| `reranking` | Aktiviert Re-Ranking Support | — |
| `sandbox` | WASM Merge Operator Isolierung | `contextra-sandbox` |
| `experimental-diskann` | DiskANN Vektor-Index Modus | `contextra-vector/experimental-diskann` |
| `background-maintenance` | Automatische Hintergrundbereinigung | — |
| `graph-connectivity-health` | Graph-Perkolations- und Zustandsanalyse | `contextra-graph/graph-connectivity-health` |
| `edge-reinforcement-learning` | Weiterleitung für Graph RL | Weiterleitung von `cognition` |
| `adaptive-candidate-pool-sizing` | Dynamische Suchpoolanpassung | `contextra-adapt` |
| `exp-extraction` | Experimentelle Wissensextraktion | — |
| `entity-extraction` | Regelbasierte & OpenIE Extraktion | `exp-extraction` |
| `auto-extraction-opt-out` | Deaktiviert automatische Extraktion | — |

## Architektur-Eigenschaften
- **Domänenreinheit (M7):** Keine Saga-, Witness- oder DeletionProof-Logik im Kern. Die physische Bereinigung delegiert rein an neutrale `Purger`- und `LayerScanner`-Schnittstellen.
- **Lizenz-Naht:** Sämtliche Lizenz-Gate Zugriffe sind im internen Modul `src/license_seam.rs` gekapselt für nahtlose spätere Umstellung auf `contextra-license`.
- **Transaktions-Isolation:** 4-Index 2-Phase-Commit mit Intent-Logging und idempotentem `CommitLedger`-Rollback.
- **Konsumenten-Schutz:** Re-Exporte und Konsumentenpfade für `contextra-db` und `contextra-cognition` sind über den Compile-Time Test `tests/s64_public_surface.rs` abgesichert.

## Minimalbeispiel
```rust,no_run
use contextra_engine::{Contextra, ContextraConfig};
use tempfile::tempdir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = ContextraConfig::default();
    let engine = Contextra::open_with_config(dir.path(), config).await?;
    let col = engine.collection("knowledge").await?;
    col.insert("doc1", &[0.1; 768], None).await?;
    Ok(())
}
```
