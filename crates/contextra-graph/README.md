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

## Kernfunktionen
- **CSR-Speicher:** Hochkompakte, cache-effiziente Adjazenzspeicherung für Millionen von Kanten.
- **Graph-Traversierung & PPR:** Schnelle Pfadsuche und Random-Walk-Bewertungen.
- **Community Detection:** Hierarchische Cluster- und Community-Erkennung zur Aggregation in GraphRAG-Pipelines.
- **Entkopplung:** Keine direkten Abhängigkeiten zu Vektor- oder Volltext-Engines.
