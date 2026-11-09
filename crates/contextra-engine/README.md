# contextra-engine

Transaktions-, Index- und Ausführungsorchestrator (Klasse C, Ring 3).

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
- **Klasse:** Klasse C (Integrations- und Kompositionsschicht)
- **Ring:** Ring 3 (Ausführungsorchestrator)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Zero-Panic-Doktrin:** Vollständig konform im Produktivcode (`tools/zero_panic_analyzer.py`)

## Architektur-Eigenschaften
- **Domänenreinheit (M7):** Keine Saga-, Witness- oder DeletionProof-Logik im Kern. Die physische Bereinigung delegiert rein an neutrale `Purger`- und `LayerScanner`-Schnittstellen.
- **Transaktions-Isolation:** 4-Index 2-Phase-Commit mit Intent-Logging und idempotentem `CommitLedger`-Rollback.
- **Deterministische Worker:** Hintergrund-Worker stellen synchrone `step()`-Primitiven bereit und nutzen injizierte Clocks.
- **Adapter-Brücken:** Unter `src/adapters/` liegen Brücken zu Port-Traits (`Bm25Scorer`, `OllamaBridge`, `privacy`).

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
