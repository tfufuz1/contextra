# AGENTS.md — `contextra-graph`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-graph` · **Ring:** 1 · **Version:** 0.1.0
- **Beschreibung:** CSR-Graph for entity-relation traversal and graph algorithms
- **src:** 90 Dateien, 17834 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 4):** `contextra-env`, `contextra-ids`, `contextra-ports`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (1):** `contextra-graph`
- **Externe Abhängigkeiten (normal, 11):** `ahash`, `arc-swap`, `bincode`, `blake3`, `bytes`, `parking_lot`, `scc`, `serde`, `serde_json`, `thiserror`, `tracing`
- **Reverse-Abhängigkeiten (2):** `contextra-cognition`, `contextra-db`
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend `contextra-db`
- **Features (16):** `apprh-diffusion`, `default`, `docid-128`, `edge-reinforcement-learning`, `entity-extraction`, `experimental-graph`, `graph-connectivity-health`, `hyperedge-suggest`, `k-path-diffusion`, `physio-percolation`, `ports-compat`, `ppr-forward-push`, `session-dag`, `test-support`, `types-compat`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (86): `AgentStateNode`, `ApprhGateMonitor`, `ApprhGateMonitorConfig`, `ApprhGateMonitorSnapshot`, `ApprhObservationRecord`, `ApprhParams`, `ApprhSelector`, `ApprhSelectorConfig`, `ApprhShadowComparison`, `ArcSlice`, `CascadeDeletionRecord`, `CascadeInvalidationReport`, `CascadeTicket`, `CommunityAssignment`, `CommunityDetectionConfig` … (+71)
- enum (9): `ApprhError`, `ApprhMode`, `CascadeStatus`, `EdgeType`, `GraphError`, `GraphMutationError`, `HyperedgeSuggestError`, `PprAlgorithm`, `TlHfdError`
- trait (13): `ApprhFlipGate`, `ContradictionDetector`, `DefaultFlipGate`, `EntityExtractor`, `GraphGarbageCollection`, `GraphIndexExt`, `GraphProvider`, `GraphStorage`, `GraphView`, `KPathDiffusion`, `PathGraph`, `ShadowSink`, `TextGenerator`
- fn (215): `acquire`, `active_head`, `active_head_write`, `active_patterns`, `add_edge`, `add_hyperedge`, `add_node`, `append_step`, `apply_cooccurrence_reinforcement`, `apply_traversal_reinforcement`, `apply_weight_normalization`, `apprh_diffusion`, `apprh_local`, `apprh_shadow_compare`, `as_arc` … (+200)
- type (5): `CascadeReport`, `EdgeId`, `GraphFuture`, `NodeIdx`, `Result`
- const (18): `CASCADE_QUEUE_PREFIX`, `DEFAULT_HYPEREDGE_CASCADE_FANOUT_LIMIT`, `DEFAULT_PPR_STREAM_BATCH_SIZE`, `DEFAULT_SUFFICIENCY_THRESHOLD`, `DEFAULT_SUPPRESSION_THRESHOLD`, `HYPEREDGE_BY_ENTITY_PREFIX`, `HYPEREDGE_PREFIX`, `MAX_DAG_STRING_BYTES`, `MAX_DAG_TRAVERSAL_DEPTH`, `MAX_HYPEREDGE_CASCADE_FANOUT`, `MAX_RELATE_PARTICIPANTS`, `MAX_SEARCH_K`, `MAX_TRAVERSAL_HOPS`, `MAX_VISITED_NODES`, `MIN_AGREEMENT_THRESHOLD` … (+3)
- mod (59): `algo_error_bridge`, `apprh`, `arc_slice`, `cascade`, `collection_mutation`, `communities`, `community`, `compaction`, `compat`, `conformance`, `consistency_enforcement`, `cost`, `csr`, `dense`, `diffusion` … (+44)
- pub use: `apprh::{ apprh_local, forward_push_apprh, shadow_compare_forward_push_vs_apprh, ApprhError, ApprhFlipGate, ApprhMode, ApprhParams, ApprhSelector, ApprhSelectorConfig, ApprhShadowComparison, DefaultApprhFlipGate, }`, `arc_slice::ArcSlice`, `cascade::{ cascade_invalidate_edges_for_superseded_doc, cascade_invalidate_hyperedges_for_superseded_doc, CascadeDeletionRecord, CascadeInvalidationReport, CascadeReport, CascadeStatus, CascadeTicket, GraphGarbageCollection, HyperedgeCascadeReport, MAX_HYPEREDGE_CASCADE_FANOUT, }`, `community::{ detect_communities, CommunityAssignment, CommunityDetectionConfig, StarExpansionIterator, VirtualHyperedgeNode, }`, `consistency_enforcement::{ ConflictPattern, ConsistencyEnforcer, ContradictionDetector, EdgeAssertion, EdgeId, ExactPredicateConflictDetector, }`, `contextra_ports::GraphIndex`, `contextra_ports::graph::{HyperEdgeId, RoleBinding, RoleId}`, `contextra_types::EntityId`, `contextra_types::{ DocId, Edge, Entity, EntityId, PprAlgorithm, PprConfig, ResourceTracker, TlHfdParams, TxId, MAX_SEARCH_K, }`, `crate::compat::graph_index` … (+47)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->`.

<!-- END GENERATED: crate_inventory.py -->
