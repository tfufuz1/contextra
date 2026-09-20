# memfuse-graph

`memfuse-graph` stellt die CSR-Graphstruktur, Personalized PageRank und n-äre Hyperkanten bereit (Ring 0).

## Zweck

Verwaltet gerichtete Wissensgraphen in CSR-Repräsentation, Forward-Push PPR (Andersen-Chung-Lang), Leiden Community Detection, n-äre Hyperkanten (`relate_n_ary`) und kaskadierende Invalidierung.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 0 (Synchroner Graph-Kern)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **CSR Graph:** `CsrGraph`, `PathGraph`, `PathRAGEngine`
- **Hyperkanten:** `HyperEdge`, `HyperEdgeId`, `RoleBinding`, `RoleId`, `RoleInterner`
- **Cascade Invalidierung:** `cascade_invalidate_edges_for_superseded_doc`, `cascade_invalidate_hyperedges_for_superseded_doc`, `HyperedgeCascadeReport`
- **Community Detection:** `detect_communities`, `CommunityDetectionConfig`, `StarExpansionIterator`

## Architektur & Verweise

Details zum Wissensgraphen, RCU-Snapshots und Hyperkanten finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) sowie `README.md` §6.
