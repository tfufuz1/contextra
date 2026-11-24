# contextra-graph

CSR Graph-Engine, Path Graphing, Hyperkanten und GraphRAG Community Detection.

## Rolle & Zweck
`contextra-graph` ist der Graph-Baustein von Contextra. Er stellt speichereffiziente Graph-Datenstrukturen im Compressed Sparse Row (CSR) Format, Graph-Traversierung, Personalized PageRank (PPR), Hyperkanten, regelbasierte Entitätsextraktion und Community Detection für GraphRAG bereit.

Der Baustein kann für Graph-Algorithmen und semantische Relationen eingesetzt werden.

## Profil
- **Datenstrukturen & Typen:** Der Kern operiert auf Standardtypen sowie Typen wie `DocId`, `EntityId`, `TxId`, `Entity`, `Edge`, `HyperEdgeId`, `RoleBinding`.
- **Fehlerbehandlung:** Kanonischer Fehlertyp `GraphError` und Subtyp `GraphMutationError`.
- **Vertrag:** Crate-Vertrag in `src/contract.rs` mit neutralen Traits `Clock`, `Rng`, `NodeId`, `Weight`.
- **Sicherheit & Unsafe-Freiheit:** `#![forbid(unsafe_code)]` in allen Produktionsmodulen, keine unkontrollierten Panics.
- **Determinismus:** Deterministische Datenstrukturen (`BTreeMap`, `BTreeSet`), injizierbare Zeit- und Zufalls-Traits (`Clock`, `Rng`).
- **Autarkie:** Eigene `LICENSE-MIT` und `LICENSE-APACHE`.

## Features
- `default = ["types-compat", "ports-compat", "experimental-graph"]`
- `experimental-graph`: Bündelt experimentelle Erweiterungen.
- `types-compat`: Typ-Kompatibilität zu Workspace-Typen.
- `ports-compat`: Port-Kompatibilität zu Workspace-Traits.
- `ppr-forward-push`: Schneller Andersen-Chung-Lang Forward-Push PPR.
- `k-path-diffusion`: K-Path Traversal und Pfad-Diffusionssuche.
- `apprh-diffusion`: Adaptiver PageRank mit Hyperkanten-Expansion.
- `graph-connectivity-health`: Perkolations- und Resilienzmetriken für Netze.
- `edge-reinforcement-learning`: Dynamisches Kanten-Reinforcement.

## Minimalbeispiel (Doctest)

```rust
use contextra_graph::csr::CsrGraph;
use contextra_types::{Edge, Entity, EntityId, TxId};
use contextra_ports::GraphIndex;

# tokio_test::block_on(async {
// 1. Graph instanziieren
let graph = CsrGraph::new();
let tx = TxId::new(1);
let e1 = EntityId::new(100);
let e2 = EntityId::new(200);

// 2. Entitäten und Kanten anlegen
graph.add_entity(tx, Entity::new(e1, "Concept A", "Concept")).await.unwrap();
graph.add_entity(tx, Entity::new(e2, "Concept B", "Concept")).await.unwrap();
let mut edge = Edge::new(e1, e2, "relates");
edge.weight = 0.9;
graph.add_edge(tx, edge).await.unwrap();
graph.commit(tx).await.unwrap();

// 3. Traversierung & BFS
let results = graph.traverse(e1, 2).await.unwrap();
assert!(!results.is_empty());
# });
```
