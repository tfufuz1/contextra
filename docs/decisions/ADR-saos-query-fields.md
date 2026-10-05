# ADR-saos-query-fields: Evidenzbasierte Architekturentscheidung zu den Feldern von HybridQuery / SaosQuery

- **Status**: Vorschlag (proposed)

## Context

In `crates/contextra-types/src/types/saos.rs` wird die zentrale Abfrage-Datenstruktur `HybridQuery` (im Kontext auch als `QueryOptions` bzw. historisch `SaosQuery` bezeichnet) sowie ihr zugehöriger `HybridQueryBuilder` (`QueryOptionsBuilder`) definiert. Es existiert die Vermutung, dass eine Reihe von Feldern im Type-Builder zwar deklariert ist, in der Ausführungsorchestrierung des Retrieval-Cores (`contextra-engine`) jedoch nicht ausgelesen wird. Eine systematische Repository-Analyse ergab, dass `HybridQuery` insgesamt aus **exakt 16 Feldern** besteht. Sämtliche 16 Felder werden von `contextra-engine` in Produktionscode (unter `crates/contextra-engine/src/collection/search/hybrid/query.rs` sowie `crates/contextra-engine/src/collection/query_builder/builder.rs`) direkt ausgelesen und zur Steuerung von BM25-, Vektor-, Graph-, RRF-Fusion-, Post-Filterungs- oder Pre-Reranking-Prozessen verwendet.

## Feldanalyse und Klassifikation

| Field | Typ | Klassifikation | Beleg (file:line) | Empfehlung | Begründung |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `text_query` | `Option<String>` | VERDRAHTET | `crates/contextra-engine/src/collection/search/hybrid/query.rs:60` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld speichert den Volltext-Suchstring für das BM25F-Lexikal-Signal und wird im Engine-Core ausgelesen. |
| `vector_query` | `Option<Vec<f32>>` | VERDRAHTET | `crates/contextra-engine/src/collection/search/hybrid/query.rs:61` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld liefert den Dichten Vektor-Embedding-Query für die HNSW/Vektorsuche im Engine-Core. |
| `graph_start_node` | `Option<String>` | VERDRAHTET | `crates/contextra-engine/src/collection/search/hybrid/query.rs:314` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld dient als Startknoten-ID für die Graph-Traversierung und Personalized PageRank im Engine-Core. |
| `graph_strategy` | `GraphTraversalStrategy` | VERDRAHTET | `crates/contextra-engine/src/collection/search/hybrid/query.rs:346` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld steuert die Graph-Traversierungsstrategie (BFS Hops, PPR oder PathRAG) in der Suchausführung. |
| `fusion_weights` | `FusionWeights` | VERDRAHTET | `crates/contextra-engine/src/collection/search/hybrid/query.rs:419` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld definiert die Signal-Gewichtung zwischen Vektor, Text und Graph bei der RRF-Score-Fusion. |
| `fusion_strategy` | `SignalFusionStrategies` | VERDRAHTET | `crates/contextra-engine/src/collection/search/hybrid/query.rs:449` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld bestimmt die Reciprocal Rank Fusion oder Score-Normalized Fusion Strategie pro Signal. |
| `filter` | `Option<FilterExpr>` | VERDRAHTET | `crates/contextra-engine/src/collection/search/hybrid/query.rs:110` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld steuert den Metadaten-Prädikatfilter vor und nach der RRF-Score-Fusion im Engine-Core. |
| `same_community_as` | `Option<EntityId>` | VERDRAHTET | `crates/contextra-engine/src/collection/search/hybrid/query.rs:422` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld aktiviert den Graph-Community-Kontext-Boost für Dokumente derselben Knoten-Community. |
| `memory_type_filter` | `Option<Vec<MemoryType>>` | VERDRAHTET | `crates/contextra-engine/src/collection/search/hybrid/query.rs:117` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld filtert Suchergebnisse vor RRF basierend auf ihrem kognitiven Gedächtnistyp (z.B. Episodic, Semantic). |
| `include_superseded` | `bool` | VERDRAHTET | `crates/contextra-engine/src/collection/search/hybrid/query.rs:95` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld steuert, ob verdrängte Zettelkasten-Gedächtnis-Chunks in Resultaten zugelassen werden (ADR-038). |
| `include_provenance` | `bool` | VERDRAHTET | `crates/contextra-engine/src/collection/search/hybrid/query.rs:447` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld entscheidet, ob ProvenanceRecord Audit-Daten an die Suchresultate angehängt werden. |
| `rerank_pool_multiplier` | `Option<usize>` | VERDRAHTET | `crates/contextra-engine/src/collection/search/hybrid/query.rs:83` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld bestimmt den Multiplikator zur Pre-Reranking Kandidatenpool-Expansion für Reranker Cross-Encoder. |
| `rerank_pool_max` | `Option<usize>` | VERDRAHTET | `crates/contextra-engine/src/collection/search/hybrid/query.rs:83` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld begrenzt die maximale Obergrenze der Pre-Reranking Kandidatenpool-Erweiterung. |
| `has_reranker` | `bool` | VERDRAHTET | `crates/contextra-engine/src/collection/search/hybrid/query.rs:81` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld zeigt an, ob ein Cross-Encoder Reranker an der Anfrage hängt und steuert die Kandidatenpool-Größe. |
| `on_signal_failure` | `OnSignalFailure` | VERDRAHTET | `crates/contextra-engine/src/collection/query_builder/builder.rs:294` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld definiert die Fehlerbehandlung bei Signalausfall (`Fail` Abbruch vs. `Degrade` Teil-Ergebnis). |
| `k` | `usize` | VERDRAHTET | `crates/contextra-engine/src/collection/search/hybrid/query.rs:55` | VERDRAHTEN (`contextra-engine::collection::search::hybrid`) | Das Feld begrenzt die maximale Anzahl der an den Aufrufer zurückzugebenden Suchergebnisse. |

## Summary

- **Gesamtzahl analysierter Felder**: 16
- **Klassifikations-Zählung**:
  - `VERDRAHTET`: 16
  - `NUR_TEST`: 0
  - `OHNE_AUFRUFER`: 0
- **Empfehlungs-Zählung**:
  - `VERDRAHTEN`: 16 (alle in `contextra-engine::collection::search::hybrid`)
  - `ENTFERNEN`: 0

## Offene Fragen an das Team

Soll `contextra_types::HybridQueryBuilder` in `contextra_types::QueryOptionsBuilder` umbenannt werden (Alias existiert bereits), um Verwechslungen mit dem ausführenden `contextra_engine::HybridQueryBuilder` zu vermeiden?
Soll in `crates/contextra-types/src/types/saos.rs` ein Modulalias `pub use HybridQuery as SaosQuery;` ergänzt werden, um historische Dokumentationsreferenzen direkt im Code abzubilden?
Soll für `contextra_types::HybridQueryBuilder` eine erweiterte Integrationstestsuite in `contextra-types` hinzugefügt werden, um den DTO-Builder isoliert ohne Engine-Abhängigkeit zu testen?
