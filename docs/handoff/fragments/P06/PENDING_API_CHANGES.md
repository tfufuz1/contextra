# Pending API Changes P06 · contextra-graph

Symbol | betroffene Crates | gewünschte Änderung
---|---|---
`contextra_graph::community` | `contextra-engine` | Vollständige Auslagerung hinter `experimental-graph`, sobald Engine das Community-Boosting modularisiert
`contextra_graph::hyperedge` | `contextra-engine`, `contextra-cognition` | Entkopplung auf eigenständige Traits / SPI; verbleibt vorerst im default
`contextra_graph::cascade` | `contextra-engine`, `contextra-db` | Verbleibt im default für `cascade_invalidate_edges_for_superseded_doc`
`contextra_graph::percolation` | `contextra-engine` | Verbleibt im default für `PercolationConfig` und Rebonding
`contextra_graph::session_dag` | interne Tests | Kann perspektivisch vollständig entkoppelt werden
`contextra_graph::apprh` | `contextra-graph` | Gated hinter `apprh-diffusion` / `experimental-graph`
`contextra_graph::tl_hfd` | `contextra-types` (Doku), interne Tests | Experimentelle Diffusions-Pipeline
`contextra_graph::path_rag` | `contextra-db` (Tests) | Pfadsuche / RAG-Traversierung
`contextra_graph::edge_reinforcement` | interne Tests | Gated hinter `edge-reinforcement-learning` / `experimental-graph`
