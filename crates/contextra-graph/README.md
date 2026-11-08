# contextra-graph

CSR Graph-Engine, Path Graphing, Hyperkanten und GraphRAG Community Detection.

Normativitätsklasse: **Klasse A Baustein** (Ring 0, SPEC v4.0 §2, §5).

## Rolle & Zweck
`contextra-graph` ist der autarke Graph-Baustein von Contextra. Er stellt speichereffiziente Graph-Datenstrukturen im Compressed Sparse Row (CSR) Format, Graph-Traversierung, Personalized PageRank (PPR), Hyperkanten und Community Detection für GraphRAG bereit.

Der Baustein kann völlig isoliert ohne interne Workspace-Abhängigkeiten in beliebigen Rust-Anwendungen für Graph-Algorithmen und semantische Relationen eingesetzt werden.

## Baukasten-Export & Standalone-Nutzung
Der Baustein erfüllt alle Klasse-A-Kriterien (B1–B12) und kann mit dem Standalone-Exporter extrahiert werden:

```bash
python3 tools/export_crate.py contextra-graph --out /tmp/my-graph-crate
```

Im Zielverzeichnis baut das Crate vollkommen autark:
```bash
cargo check --no-default-features
```

## Features
- `types-compat`: Transitionale Kompatibilitätsanbindung an `contextra-types` und `contextra-ports` (Default bis Welle 6).
- `ports-compat`: Alias für `types-compat`.
- `ppr-forward-push`: Schneller Andersen-Chung-Lang Forward-Push PPR.
- `k-path-diffusion`: K-Path Traversal und Pfad-Diffusionssuche.
- `apprh-diffusion`: Adaptiver PageRank mit Hyperkanten-Expansion.
- `graph-connectivity-health`: Perkolations- und Resilienzmetriken für Netze.
- `edge-reinforcement-learning`: Dynamisches Kanten-Reinforcement.

## Autarkes Minimalbeispiel (ohne interne Workspace-Deps)

```rust
use contextra_graph::csr::CsrGraph;
use contextra_graph::types::{Edge, Entity, EntityId, TxId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    let e1 = EntityId::new(100);
    let e2 = EntityId::new(200);

    graph.add_entity(tx, Entity::new(e1, "Concept A", "Type")).await?;
    graph.add_entity(tx, Entity::new(e2, "Concept B", "Type")).await?;
    graph.add_edge(tx, Edge::new(e1, e2, "relates").with_weight(0.9)).await?;
    graph.commit(tx).await?;

    let results = graph.traverse(e1, 2).await?;
    assert!(!results.is_empty());
    Ok(())
}
```
