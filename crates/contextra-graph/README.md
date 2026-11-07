# contextra-graph

CSR Graph-Engine, Path Graphing und GraphRAG Community Detection.

Normativitätsklasse: **Rand `[R]`** (Ring 0, `experimental`; §6.1, §6.9, §11, `capabilities.toml`).

## Rolle & Zweck
`contextra-graph` ist der autarke Graph-Baustein von Contextra (Ring 0). Er stellt performante Graph-Datenstrukturen im Compressed Sparse Row (CSR) Format, Graph-Traversierung, Personalized PageRank (PPR) und Community Detection für GraphRAG bereit.

Der Baustein kann völlig isoliert in beliebigen Rust-Anwendungen für Graph-Algorithmen und semantische Relationen eingesetzt werden.

## Baukasten-Einordnung & Status
- **Ring:** Ring 0 (Graph-Fundament)
- **Klasse:** Rand `[R]`
- **Status:** 🟢 Autarker Baustein
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Erlaubte Abhängigkeiten:** Ausschließlich `contextra-types` und `contextra-ports`.
- **Dev-Abhängigkeiten:** Ausschließlich `contextra-testkit` und Hilfsbibliotheken (keine Abhängigkeit zu `contextra-store`).

## Kernfunktionen & Garantien
- **CSR-Speicher:** Hochkompakte, cache-effiziente Adjazenzspeicherung für Millionen von Kanten.
- **Graph-Traversierung & PPR:** Schnelle Pfadsuche und Random-Walk-Bewertungen.
- **Community Detection:** Hierarchische Cluster- und Community-Erkennung zur Aggregation in GraphRAG-Pipelines.
- **Entkopplung:** Keine direkten Abhängigkeiten zu Vektor- oder Volltext-Engines.
- **Neutrale Wartung:** Bereinigungs- und Wartungsoperationen erfolgen über neutrale Supersession- und Pruning-Mechanismen (D-E1).

## Einbettungsbeispiel
Ein eigenständiges, isoliert lauffähiges Beispiel findet sich unter `examples/isolated_graph_traversal.rs`.

```rust
use contextra_graph::CsrGraph;
use contextra_ports::GraphIndex;
use contextra_types::{Edge, Entity, EntityId, TxId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    let e1 = EntityId::from("concept_a");
    let e2 = EntityId::from("concept_b");

    graph.add_entity(tx, Entity::new(e1, "Concept A", "Type")).await?;
    graph.add_entity(tx, Entity::new(e2, "Concept B", "Type")).await?;
    graph.add_edge(tx, Edge::new(e1, e2, "relates").with_weight(0.9)).await?;
    graph.commit(tx).await?;

    let neighbors = graph.neighbors(e1).await?;
    assert!(neighbors.contains(&e2));
    Ok(())
}
```
