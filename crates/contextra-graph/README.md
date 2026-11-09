# contextra-graph

CSR Graph-Engine, Path Graphing, Hyperkanten und GraphRAG Community Detection.

Normativitätsklasse: **Klasse A Baustein** (Ring 0, SPEC v4.0 §2, §5).

## Rolle & Zweck
`contextra-graph` ist der autarke Graph-Baustein von Contextra. Er stellt speichereffiziente Graph-Datenstrukturen im Compressed Sparse Row (CSR) Format, Graph-Traversierung, Personalized PageRank (PPR), Hyperkanten, regelbasierte Entitätsextraktion und Community Detection für GraphRAG bereit.

Der Baustein kann völlig isoliert ohne interne Workspace-Abhängigkeiten in beliebigen Rust-Anwendungen für Graph-Algorithmen und semantische Relationen eingesetzt werden.

## Entkopplungsprofil (Klasse A / B1–B12)
- **Keine internen Abhängigkeiten (B1–B3):** Der Kern operiert ausschließlich auf Standardtypen und crate-lokalen Typen (`DocId`, `EntityId`, `TxId`, `Entity`, `Edge`, `HyperEdgeId`, `RoleBinding`).
- **Fehlerbehandlung (B4):** Genau ein primärer Fehlertyp `Error` (`GraphError`, `thiserror`, `#[non_exhaustive]`) plus `Result<T>`.
- **Vertrag (SPEC §4.3):** Crate-Vertrag in `src/contract.rs` mit neutralen Traits `Index` und `Maintain`.
- **Zero-Panic & Unsafe-Freiheit (B9, B10):** `#![forbid(unsafe_code)]` in allen Produktionsmodulen, keine unkontrollierten Panics.
- **Determinismus (B11):** Deterministische Datenstrukturen (`BTreeMap`, `BTreeSet`), injizierte Zeit- und Zufalls-Traits (`Clock`, `Rng`).
- **Autarkie (B12):** Eigene `LICENSE-MIT` und `LICENSE-APACHE`, `rust-version = "1.89"`.

## Baukasten-Export & Standalone-Nutzung
Der Baustein erfüllt alle Klasse-A-Kriterien (B1–B12) und kann mit dem Standalone-Exporter extrahiert werden:

```bash
python3 tools/export_crate.py contextra-graph --out /tmp/my-graph-crate
```

Im Zielverzeichnis baut das Crate vollkommen autark:
```bash
cargo check --no-default-features
cargo test
```

## Features
- `default = ["std"]`
- `std`: Standardbibliotheksunterstützung.
- `serde`: Serialisierung und Deserialisierung für Graph-Strukturen und Konfigurationen.
- `tracing`: Diagnose-Logging (ohne Feature entfallen Makros spurlos).
- `inspect`: Lesende Inspektions- und Diagnosemethoden (`GraphMaintainer::inspect_stats`).
- `experimental`: Optionale und experimentelle Algorithmenanbindungen.
- `types-compat`: Transitionale Kompatibilitätsanbindung an `contextra-types` und `contextra-ports`.
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
use contextra_graph::community::{detect_communities, CommunityDetectionConfig};
use contextra_graph::provider::{InMemoryGraphProvider, compute_pagerank};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Graph instanziieren
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    let e1 = EntityId::new(100);
    let e2 = EntityId::new(200);

    // 2. Entitäten und Kanten anlegen
    graph.add_entity(tx, Entity::new(e1, "Concept A", "Concept")).await?;
    graph.add_entity(tx, Entity::new(e2, "Concept B", "Concept")).await?;
    graph.add_edge(tx, Edge::new(e1, e2, "relates").with_weight(0.9)).await?;
    graph.commit(tx).await?;

    // 3. Traversierung & BFS
    let results = graph.traverse(e1, 2).await?;
    assert!(!results.is_empty());

    // 4. Community Detection
    let config = CommunityDetectionConfig::default();
    let communities = detect_communities(&graph, &config).await?;
    println!("Detektierte Communities: {}", communities.len());

    // 5. Standalone Algorithmen-Provider (PPR / PageRank)
    let mut provider = InMemoryGraphProvider::new();
    provider.add_edge(e1, e2, 1.0);
    let ranks = compute_pagerank(&provider, 0.85, 50, 1e-5);
    println!("PageRank für e1: {:?}", ranks.get(&e1));

    Ok(())
}
```
