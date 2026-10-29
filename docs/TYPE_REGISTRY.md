# Contextra Type Registry (`docs/TYPE_REGISTRY.md`)

<!-- doc-stamp: 2026-10-29 10cc17922 -->

Stand: **29.10.2026** | Normativ: `docs/SPEC.md` §5 (Baukasten-Modell / Single Type Family Principle A2)

Dieses Dokument erfasst alle öffentlichen Typen (Structs, Enums, Traits, Type Aliases), die in der Workspace-Analyse in $\ge 2$ Dateien definiert oder re-deklariert wurden (35+ Typnamen). Es ordnet jeden Typ seiner kanonischen Primärdefinition zu, beschreibt seine Aufgabenstellung und legt den Migrations- bzw. Entkopplungspfad fest, um Re-Deklarations-Drift zu verhindern.

---

## Kanonische Typen-Matrix

| Typname | Kanonischer Ort (Crate / Modul) | Vorkommen in Dateien | Aufgabe & Abgrenzung | Migrations- / Entkopplungspfad |
|---|---|---|---|---|
| `AdaptiveDecayController` | `contextra-adapt::decay_controller` | `contextra-adapt`, `contextra-engine` | PID-Latenzregler für adaptive Ausführungsfristen. | Kanonisch in `contextra-adapt`. Re-Deklaration in `engine` wurde in WP 2.2 decoupled. |
| `BoundedTopK` | `contextra-rank::topk` | `contextra-rank`, `contextra-db` | Bounded Heap für die Top-K Ergebnis-Aggregation. | Kanonisch in `contextra-rank`. In `db` nur als Import/Re-export verwenden. |
| `BoxFuture` | `contextra-ports::future` | `contextra-ports`, `contextra-types` | Type-Alias für gepinnte Heap-Futures `Pin<Box<dyn Future>>`. | Kanonisch in `contextra-ports`. Direct usage in `types` auf `ports` konsolidieren. |
| `CandleModelInner` | `contextra-infer-candle::model` | `contextra-infer-candle`, `contextra-mcp` | Interner Wrapper für Candle LLM/Embedding Inferenz. | Kanonisch in `contextra-infer-candle`. Aus `mcp` entfernen, nur über Trait-Port ansprechen. |
| `CommunityDetectionConfig` | `contextra-graph::community` | `contextra-graph`, `contextra-cognition` | Parameter für GraphRAG Community Detection. | Kanonisch in `contextra-graph`. Cognition bindet `graph` als Baukasten `[M]`. |
| `CompactionStrategy` | `contextra-store::lsm::compaction` | `contextra-store`, `contextra-engine` | LSM-Tree Kompaktierungsstrategie (Leveled / Tiered). | Kanonisch in `contextra-store`. In `engine` nur via `contextra-store` Referenz. |
| `Confidence` | `contextra-types::confidence` | `contextra-types`, `contextra-rank` | Fließkomma-Konfidenzwert mit Validierung. | Kanonisch in `contextra-types`. `rank` verwendet Import aus `types`. |
| `ConformalCalibrator` | `contextra-rank::calibration` | `contextra-rank`, `contextra-infer-candle` | Kalibrierung von Konfidenz-Intervallen. | Kanonisch in `contextra-rank`. |
| `ConsolidationNodesGuard` | `contextra-cognition::consolidation` | `contextra-cognition`, `contextra-db` | RAII Guard für Hintergrund-Konsolidierung. | Kanonisch in `contextra-cognition`. |
| `ContextEditAuditRecord` | `contextra-types::audit` | `contextra-types`, `contextra-agent` | Audit-Eintrag für Kontextänderungen. | Kanonisch in `contextra-types`. |
| `DecayControllerConfig` | `contextra-adapt::decay_controller` | `contextra-adapt`, `contextra-engine` | Konfiguration für PID Latenzregler. | Kanonisch in `contextra-adapt`. |
| `DecaySignalInputs` | `contextra-adapt::decay_controller` | `contextra-adapt`, `contextra-engine` | Eingabesignale für Adaption. | Kanonisch in `contextra-adapt`. |
| `DefaultCandleLlmModel` | `contextra-infer-candle::model` | `contextra-infer-candle`, `contextra-mcp` | Standard-Implementierung des Candle LLM Providers. | Kanonisch in `contextra-infer-candle`. |
| `DeletionProof` | `contextra-crypto::deletion_proof` | `contextra-crypto`, `contextra-engine` | Signierter V5-Löschbericht mit BLAKE3-Witnesses. | **A2 Single Source:** Kanonisch ausschließlich in `contextra-crypto`. `engine` re-exportiert. |
| `DriftDetector` | `contextra-adapt::drift` | `contextra-adapt`, `contextra-rank` | O(1) Online Sequential Drift Signal Trait. | Kanonisch in `contextra-adapt`. `rank::DriftDetector` in Rustdoc differenziert. |
| `EgressGuard` | `contextra-privacy::egress_guard` | `contextra-privacy`, `contextra-mcp` | Sicherheits-Gateway für Egress-Prüfung. | Kanonisch in `contextra-privacy`. `contextra-mcp` re-exportiert reines Port-Schnittstellen-Proxy. |
| `Embedding` | `contextra-types::embedding` | `contextra-types`, `contextra-vector` | DTO/Dense-Vector Repräsentation. | Kanonisch in `contextra-types`. |
| `EncryptedSegmentPayload` | `contextra-crypto::aead` | `contextra-crypto`, `contextra-store` | Verschlüsselter WAL/SSTable Daten-Payload. | Kanonisch in `contextra-crypto`. |
| `EntityExtractionConfig` | `contextra-graph::extraction` | `contextra-graph`, `contextra-engine` | Konfiguration für Entitätsextraktion. | Kanonisch in `contextra-graph`. |
| `EntityExtractor` | `contextra-graph::extraction` | `contextra-graph`, `contextra-engine` | Trait für Entitätsextraktion aus Texten. | Kanonisch in `contextra-graph`. |
| `EvictionWorker` | `contextra-kvcache::eviction` | `contextra-kvcache`, `contextra-engine` | Hintergrund-Worker für KV-Cache Evakuierung. | Kanonisch in `contextra-kvcache`. |
| `HybridQueryBuilder` | `contextra-db::query` | `contextra-db`, `contextra-engine` | Builder für Hybridsuche (Vektor + Text + Rank). | Kanonisch in `contextra-db`. |
| `HyperEdge` | `contextra-graph::hyper` | `contextra-graph`, `contextra-db` | Hypergraph-Kante für n-äre Beziehungen. | Kanonisch in `contextra-graph`. |
| `HyperEdgeId` | `contextra-types::id` | `contextra-types`, `contextra-graph` | Typisierte ID für Hypergraph-Kanten. | Kanonisch in `contextra-types`. |
| `InMemoryStorageEngine` | `contextra-agent::storage` | `contextra-agent`, `contextra-testkit` | In-Memory Speicher-Engine für Tests & Mocks. | Kanonisch in `contextra-agent`. `testkit` nutzt `contextra-agent`. |
| `KvSegment` | `contextra-kvcache::segment` | `contextra-kvcache`, `contextra-store` | Cache-Segment im KV-Stufenmodell. | Kanonisch in `contextra-kvcache`. |
| `LoomMetadata` | `contextra-types::loom` | `contextra-types`, `contextra-store` | Concurrency Test Metadata. | Kanonisch in `contextra-types`. |
| `MockEmbedder` | `contextra-testkit::mocks` | `contextra-testkit`, `contextra-db` | Mock Embedding Provider für Unittests. | Kanonisch in `contextra-testkit`. In Produktionscode verboten. |
| `PercolationConfig` | `contextra-graph::percolation` | `contextra-graph`, `contextra-cognition` | Parameter für Graph-Perkolationsanalyse. | Kanonisch in `contextra-graph`. |
| `QuantizedLlamaModel` | `contextra-infer-candle::quant` | `contextra-infer-candle`, `contextra-mcp` | Quantisiertes GGUF-Modell. | Kanonisch in `contextra-infer-candle`. |
| `ResourceBudget` | `contextra-ports::budget` | `contextra-ports`, `contextra-core` | CPU/Memory Ressourcen-Budgetierungs-Typ. | Kanonisch in `contextra-ports`. Re-export in `core`. |
| `RoleBinding` | `contextra-types::auth` | `contextra-types`, `contextra-privacy` | Tenant Role-Binding Struktur. | Kanonisch in `contextra-types`. |
| `RoleId` | `contextra-types::auth` | `contextra-types`, `contextra-privacy` | Typisierte ID für Rollen. | Kanonisch in `contextra-types`. |
| `ScoredDocument` | `contextra-types::search` | `contextra-types`, `contextra-rank` | Trefferstruktur mit Score und Payload. | Kanonisch in `contextra-types`. |
| `Signal` | `contextra-adapt::signal` | `contextra-adapt`, `contextra-router` | Signal-Typ für Latenz/Drift Regelung. | Kanonisch in `contextra-adapt`. |
| `SpillHandler` | `contextra-kvcache::spill` | `contextra-kvcache`, `contextra-store` | Auslagerungs-Handler bei Cache-Overflow. | Kanonisch in `contextra-kvcache`. |
| `SynthesisPhaseResult` | `contextra-cognition::synthesis` | `contextra-cognition`, `contextra-db` | Ergebnis der Konsolidierungssynthese. | Kanonisch in `contextra-cognition`. |
| `TenantIsolatedKvStore` | `contextra-kvcache::tenant` | `contextra-kvcache`, `contextra-engine` | Mandanten-isolierter KV-Cache Store. | Kanonisch in `contextra-kvcache`. |
| `TlHfdParams` | `contextra-graph::tl_hfd` | `contextra-graph`, `contextra-engine` | Parameter für Topologische HFD-Diffusion. | Kanonisch in `contextra-graph`. |

---

## Invarianten der Type Registry (A2 / Rule 11)

1. **Rule 11 (Eine Typfamilie je Konzept):** Löschtypen existieren nur in `crypto`/`ports`; Ports nur in `ports`; Grundtypen in `types`.
2. **Keine verdoppelten Struct- / Enum-Definitionen:** Wenn ein Typ in mehreren Crates benötigt wird, wird er in `contextra-types` oder `contextra-ports` definiert und von Konsumenten re-exportiert oder importiert.
3. **Fail-closed bei Drift:** Das Gate `symbol-uniqueness` bzw. `check-type-registry` verifiziert die Einhaltung dieser Registratur.
