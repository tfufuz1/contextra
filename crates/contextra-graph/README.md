# contextra-graph

CSR Graph-Engine, Path Graphing und GraphRAG Community Detection.

Normativitätsklasse: **Rand `[R]`** (Ring 0, `experimental`; §6.1, §6.9, §11, `capabilities.toml`).

## Rolle in einem Satz
`contextra-graph` stellt strukturelle Graph-Traversierung, CSR-Repräsentation, PPR und Community Detection für GraphRAG bereit.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Graph-Engine)
- **Klasse:** Rand `[R]`
- **Reifegrad:** `experimental` (abweichend in `capabilities.toml` als `stable` geführt, siehe `docs/status/ABW-L3.md`)
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`)
