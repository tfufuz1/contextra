# API-Dokumentation `contextra-graph`

## 1. Übersicht
`contextra-graph` stellt die Graph- und Relationensignale im Contextra-Retrieval-Kern bereit.

## 2. Kern-Module & Typen

### 2.1 Graph-Index & CSR (`csr`)
- `CsrGraph`: Hauptstruktur für speichereffiziente Graph-Operationen (Compressed Sparse Row).
  - Implementiert `contextra_ports::GraphIndex` und `contextra_ports::GraphCollectionMutation`.
  - Methoden: `add_entity`, `remove_entity`, `add_edge`, `remove_edge`, `traverse`, `personalized_page_rank`, `commit`, `rollback_to_tx`.

### 2.2 Personalized PageRank (`ppr`)
- `PprConfig`: Konfiguration für Dämpfung, Iterationen, Epsilon und Algorithmuswahl.
- `PprAlgorithm`:
  - `DensePowerIteration`: Klassische dichte Matrix-Vektor-Iteration.
  - `ForwardPush`: Lokale Approximation nach Andersen-Chung-Lang.
  - `Auto`: Kostenmodell-gesteuerte dynamische Selektion basierend auf Graphdichte und Seed-Menge.
  - `ShadowMode`: Schattenvergleich von Approximationsalgorithmen gegen exakte Baselines.

### 2.3 Hyperkanten (`hyperedge`)
- `HyperEdge`: N-äre Relationen mit geteilten Teilnehmern und Rollenbindungen.
- `HyperEdgeId`, `RoleId`, `RoleBinding`: Kanonische Identifikatoren.

### 2.4 Entitätsextraktion (`entity_extraction`)
- `RuleBasedEntityExtractor`: Deterministischer, regelbasierter Extraktor für Beziehungen (`WORKS_AT`, `LIVES_IN`, `PARTNERED_WITH`, `FOUNDED`, `CONNECTED_TO`, `LEADS`).
- `ExtractedRelation`: Extrahiertes Subjekt-Objekt-Prädikat-Tripel mit Konfidenzwert.

### 2.5 Bereinigung & Purger (`purger`)
- `GraphPurger` / `graph_purger`: Bereinigung von Dokumentkanten über `contextra_ports::scanner::Purger`.

### 2.6 Experimentelle Module (`session_dag` etc.)
- `SessionBranchTree`, `AgentStateNode`: Session-Branching und Konversationsbaum (hinter Feature `experimental-graph`).

### 2.7 Fehlerbehandlung (`error`)
- `GraphError`: Primärer Fehlertyp (`Error`).
- `GraphMutationError`: Detaillierte Fehler bei Graph-Mutationen.
- `Result<T>`: Crate-weites Resultat.
