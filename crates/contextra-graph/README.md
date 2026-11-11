# contextra-graph

CSR Graph-Engine, Path Graphing, Hyperkanten und GraphRAG Community Detection.

## Rolle & Zweck (Klasse A Baustein)
`contextra-graph` ist der eigenständige, hochperformante Graph-Baustein von Contextra. Er stellt speichereffiziente Graph-Datenstrukturen im Compressed Sparse Row (CSR) Format, Graph-Traversierung, Personalized PageRank (PPR), Hyperkanten, regelbasierte Entitätsextraktion und Community Detection für GraphRAG bereit.

Entwickelt nach den Grundsätzen modularer Baukastensysteme (SPEC v2 / Klasse A), kann dieser Crate mit minimalem Aufwand in beliebig strukturierte Fremdsysteme und standalone Rust-Anwendungen integriert werden.

## Profil & Architektur-Garantien
- **Datenstrukturen & Typen:** Autonome, neutrale Typen in `src/types.rs` (`EntityId`, `DocId`, `TxId`, `Entity`, `Edge`, `HyperEdgeId`, `RoleBinding`).
- **Fehlerbehandlung:** Kanonische Fehlertaxonomie `GraphError` und `GraphMutationError` (`thiserror`, `#[non_exhaustive]`) ohne unkontrollierte Panics (Zero-Panic-Disziplin).
- **Vertrag & Entkopplung:** Neutraler Baustein-Vertrag in `src/contract.rs` mit abstrakten Traits `Clock`, `Rng`, `NodeId`, `Weight`. Keine harten Workspace-Abhängigkeiten im Kern.
- **Kompatibilitätsbrücke:** Sämtliche Workspace-Adapter (`contextra-types`, `contextra-ports`) liegen strikt gekapselt im Untermodul `src/compat/` hinter optionalen Feature-Flags (`types-compat`, `ports-compat`).
- **Sicherheit & Korrektheit:** `#![forbid(unsafe_code)]` in allen Produktionsmodulen.
- **Determinismus:** Deterministische Datenstrukturen und garantierter Tie-Break bei gleichen Scores nach EntityId aufsteigend.
- **Bitemporale Sichtbarkeit:** Transaktionale Snapshot-Isolation über `born_seq` und `died_seq` zur Vermeidung von Dirty Reads oder Phantoms.
- **Lock-Disziplin:** Typisierte Lock-Akquise (`NodesGuard` in `session_dag.rs`), keine Locks über `.await`-Punkte (`deny(clippy::await_holding_lock)`).

## Features
- `default = ["workspace", "experimental-graph"]`
- `workspace`: Aktiviert die Kompatibilitätsbrücke für den Einsatz im Monorepo (`types-compat`, `ports-compat`).
- `types-compat`: Konvertierung zwischen lokalen Typen und `contextra-types`.
- `ports-compat`: Implementierung von Trait-Schnittstellen aus `contextra-ports` (`GraphIndex`, `Purger`).
- `experimental-graph`: Bündelt experimentelle Erweiterungen.
- `ppr-forward-push`: Schneller Andersen-Chung-Lang Forward-Push PPR.
- `k-path-diffusion`: K-Path Traversal und Pfad-Diffusionssuche für GraphRAG.
- `apprh-diffusion`: Adaptiver PageRank mit Hyperkanten-Expansion.
- `graph-connectivity-health`: Perkolations- und Resilienzmetriken für Netze.
- `edge-reinforcement-learning`: Dynamisches Kanten-Reinforcement.
- `docid-128`: Unterstützung für 128-Bit Dokument-IDs.

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
