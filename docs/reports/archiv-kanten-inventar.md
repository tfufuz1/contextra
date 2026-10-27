# Kanten-Inventar der Archiv-Crates (WP 2.1)

**Erstellt:** WP 2.1 (Workspace-Archivierung)
**Normativer Bezug:** docs/SPEC.md §2.3, PLAN.md WP 2.1, AGENTS.md §5.4, INV-CORE-ISOLATION-1

## 1. Übersicht & Status Tag `archiv/v0`

- **Tag `archiv/v0`:** Existiert bereits im Git-Repository auf Commit `05a6a9afecf31889dab6ea8d40b983d4299b5e1b`.
- **Befehl zur Tag-Erstellung (falls Maintainer erneut setzt):**
  ```bash
  git tag -a archiv/v0 -m "archiv/v0 — Stand vor Phase-2-Archivierung (WP 2.1)" 05a6a9afecf31889dab6ea8d40b983d4299b5e1b
  ```

## 2. Involvierte Archiv-Crates & Kanten-Zusammenfassung

| Archiv-Crate | Ring | Manifest-Kanten | Quellcode-Treffer (`use contextra_<name>`) | Haupt-Entkopplungs-WP |
|---|---|---|---|---|
| `contextra-graph` | Archiv | 3 | 112 | WP 2.1b (physische Entfernung), WP 2.2 (engine), WP 2.3 (db/mcp/contextra) |
| `contextra-cognition` | Archiv | 1 | 12 | WP 2.3 (db/mcp/contextra) |
| `contextra-adapt` | Archiv | 3 | 14 | WP 2.2 (engine), WP 2.3 (db/mcp/contextra) |
| `contextra-kvcache` | Archiv | 0 | 0 | WP 2.1b |
| `contextra-router` | Archiv | 2 | 2 | WP 2.1b (physische Entfernung), WP 2.3 (db/mcp/contextra) |
| `contextra-agent` | Archiv | 0 | 0 | WP 2.1b |
| `contextra-infer-onnx` | Archiv | 2 | 4 | WP 2.1b (physische Entfernung), WP 2.2 (engine), WP 2.3 (db/mcp/contextra) |
| `contextra-infer-ollama` | Archiv | 1 | 1 | WP 2.3 (db/mcp/contextra) |

---

## 3. Detaillierte Kanten je Archiv-Crate

### `contextra-graph`

#### Manifest-Abhängigkeiten (`Cargo.toml`)

| Dependent Crate | Ring | Manifest-Datei | Typ | Entkopplungs-WP |
|---|---|---|---|---|
| `contextra-db` | Ring 4 | `crates/contextra-db/Cargo.toml` | normal | WP 2.3 (db/mcp/contextra) |
| `contextra-store` | Ring 1 | `crates/contextra-store/Cargo.toml` | dev-dependency | WP 2.1b (physische Entfernung) |
| `contextra-bench` | Tooling | `benchmarks/contextra-bench/Cargo.toml` | normal | WP 2.1b (physische Entfernung) |

#### Quellcode-Nutzung (`.rs`)

| Dependent Crate | Ring | Datei:Zeile | Quellcode-Nutzung | Entkopplungs-WP |
|---|---|---|---|---|
| `contextra-db` | Ring 4 | `crates/contextra-db/src/multistep.rs:373` | `use contextra_graph::CsrGraph;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:107` | `pub use contextra_graph::percolation::PercolationConfig;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/tests/integration_temporal_filter.rs:2` | `use contextra_graph::CsrGraph;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/tests/hnsw_delete_and_backfill.rs:5` | `use contextra_graph::CsrGraph;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/tests/fault_injection_2pc.rs:6` | `use contextra_graph::CsrGraph;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/tests/pid_pool_production_caller.rs:7` | `use contextra_graph::CsrGraph;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/tests/cross_domain_chaos_matrix_test.rs:7` | `use contextra_graph::csr::CsrGraph;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/tests/graph_entity_delete_test.rs:3` | `use contextra_graph::CsrGraph;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/tests/truncation_filter_recall_test.rs:2` | `use contextra_graph::CsrGraph;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/tests/multistep_query_rewriter_modalities.rs:6` | `use contextra_graph::CsrGraph;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/tests/diskann_collection_fallback.rs:8` | `use contextra_graph::csr::CsrGraph;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/tests/supersedes_cascading_tombstone_test.rs:4` | `use contextra_graph::PathRAGEngine;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/tests/consolidation_double_trigger_test.rs:11` | `use contextra_graph::CsrGraph;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/tests/cascade_invalidation_integration.rs:2` | `use contextra_graph::path_rag::PathRAGEngine;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/tests/cascade_invalidation_test.rs:2` | `use contextra_graph::path_rag::PathRAGEngine;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/tests/search_result_bound.rs:2` | `use contextra_graph::CsrGraph;` | WP 2.3 (db/mcp/contextra) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/maintenance.rs:10` | `use contextra_graph::{detect_communities, CommunityAssignment, CommunityDetectionConfig};` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/maintenance.rs:869` | `config: &contextra_graph::percolation::PercolationConfig,` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/maintenance.rs:872` | `let health = contextra_graph::percolation::compute_percolation_health(` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/maintenance.rs:878` | `contextra_graph::percolation::should_trigger_rebonding(phi, config)` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/maintenance.rs:919` | `let candidate_pairs = contextra_graph::percolation::find_rebonding_candidates(` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/mod.rs:32` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/relate.rs:105` | `) -> Result<contextra_graph::hyperedge::HyperEdgeId> {` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/relate.rs:125` | `let role_id = contextra_graph::hyperedge::RoleId::new(role_doc_id.inner() as u32);` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/relate.rs:141` | `let hyperedge_id = contextra_graph::hyperedge::HyperEdgeId::new(db_tx.tx_id.inner());` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/relate.rs:143` | `let role_bindings: Vec<contextra_graph::hyperedge::RoleBinding> = resolved_participants` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/relate.rs:146` | `contextra_graph::hyperedge::RoleBinding::new(role_id, entity_id)` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/relate.rs:155` | `let hyperedge = contextra_graph::hyperedge::HyperEdge::new(` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/relate.rs:157` | `contextra_graph::csr::EdgeType::Default,` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/relate.rs:169` | `contextra_graph::hyperedge::HYPEREDGE_PREFIX,` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/query_builder/tests.rs:3` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/crud/links.rs:180` | `contextra_graph::cascade_invalidate_edges_for_superseded_doc(` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:5` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:69` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:191` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:235` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:279` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:368` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:485` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:547` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:586` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/link_tests.rs:5` | `use contextra_graph::csr::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/link_tests.rs:56` | `use contextra_graph::csr::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/link_tests.rs:167` | `use contextra_graph::csr::{CsrGraph, CsrGraphConfig};` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/link_tests.rs:307` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/link_tests.rs:404` | `let config = contextra_graph::percolation::PercolationConfig {` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/search_tests.rs:5` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/search_tests.rs:48` | `use contextra_graph::csr::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/search_tests.rs:198` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/search_tests.rs:273` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/search_tests.rs:352` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/search_tests.rs:449` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/read_tests.rs:5` | `use contextra_graph::csr::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/read_tests.rs:148` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/read_tests.rs:282` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/insert_tests.rs:5` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/insert_tests.rs:74` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/insert_tests.rs:125` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/insert_tests.rs:176` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/insert_tests.rs:227` | `use contextra_graph::csr::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/misc_tests.rs:27` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/misc_tests.rs:133` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/misc_tests.rs:345` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/misc_tests.rs:397` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/misc_tests.rs:448` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/misc_tests.rs:490` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/misc_tests.rs:538` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:5` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:51` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:103` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:163` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:288` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:407` | `use contextra_graph::csr::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:463` | `use contextra_graph::csr::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:547` | `use contextra_graph::csr::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:662` | `use contextra_graph::csr::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:746` | `use contextra_graph::csr::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:827` | `use contextra_graph::csr::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:928` | `use contextra_graph::csr::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:995` | `use contextra_graph::csr::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:1101` | `use contextra_graph::csr::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:1196` | `use contextra_graph::csr::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/update_delete_tests.rs:5` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/update_delete_tests.rs:59` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/update_delete_tests.rs:129` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/update_delete_tests.rs:196` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/update_delete_tests.rs:244` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/update_delete_tests.rs:292` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/transaction/db_transaction.rs:23` | `pub(super) staged_hyperedges: Mutex<Vec<contextra_graph::hyperedge::HyperEdge>>,` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/transaction/db_transaction.rs:127` | `pub fn stage_hyperedge(&self, hyperedge: contextra_graph::hyperedge::HyperEdge) {` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/transaction/tests.rs:4` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/contextra_impl/collections.rs:59` | `contextra_graph::CsrGraph::load_from_storage(tenant_storage.as_ref()).await?;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/contextra_impl/collections.rs:169` | `let mut graph = contextra_graph::CsrGraph::load_from_storage(self.storage.as_ref()).await?;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/background_workers/tests.rs:5` | `use contextra_graph::hyperedge::HyperEdgeId;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/background_workers/tests.rs:102` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/background_workers/tests.rs:164` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/background_workers/tests.rs:230` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/background_workers/tests.rs:388` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/background_workers/tests.rs:497` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/background_workers/tests.rs:551` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/background_workers/tests.rs:594` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/background_workers/tests.rs:687` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/background_workers/hyperedge_worker.rs:6` | `use contextra_graph::hyperedge::HyperEdgeId;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/tests/commit_crash_consistency.rs:10` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/tests/hybrid_adaptive_overfetch.rs:3` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/tests/hybrid_signal_failure_semantics.rs:4` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/tests/engine_recovery_crash_windows.rs:9` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/tests/ssi_read_path_coverage.rs:171` | `let graph = Arc::new(contextra_graph::CsrGraph::new());` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/tests/hybrid_pathrag_snapshot.rs:3` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/tests/query_builder_scope_acorn.rs:9` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/tests/engine_commit_crash_points.rs:4` | `use contextra_graph::CsrGraph;` | WP 2.2 (engine) |
| `contextra-types` | Ring 0 | `crates/contextra-types/src/types/domain/misc.rs:344` | `/// Graph algorithms in `contextra-graph` map this configuration to `contextra_graph::TlHfdParams`.` | WP 2.1b (physische Entfernung) |

---

### `contextra-cognition`

#### Manifest-Abhängigkeiten (`Cargo.toml`)

| Dependent Crate | Ring | Manifest-Datei | Typ | Entkopplungs-WP |
|---|---|---|---|---|
| `contextra-db` | Ring 4 | `crates/contextra-db/Cargo.toml` | normal | WP 2.3 (db/mcp/contextra) |

#### Quellcode-Nutzung (`.rs`)

| Dependent Crate | Ring | Datei:Zeile | Quellcode-Nutzung | Entkopplungs-WP |
|---|---|---|---|---|
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:7` | `pub use contextra_cognition as cognition;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:30` | `pub use contextra_cognition::execute_sleep_cycle;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:31` | `pub use contextra_cognition::{` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:43` | `pub use contextra_cognition::context::*;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:46` | `pub use contextra_cognition::context_compaction::*;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:49` | `pub use contextra_cognition::memory_consolidation::*;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:52` | `pub use contextra_cognition::synthesis_phase::*;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:55` | `pub use contextra_cognition::consolidation_executor::*;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:58` | `pub use contextra_cognition::consolidation_locks::*;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:61` | `pub use contextra_cognition::maintenance_scheduler::*;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:64` | `pub use contextra_cognition::maintenance_config::*;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:73` | `pub use contextra_cognition::start_consolidation_reaper;` | WP 2.3 (db/mcp/contextra) |

---

### `contextra-adapt`

#### Manifest-Abhängigkeiten (`Cargo.toml`)

| Dependent Crate | Ring | Manifest-Datei | Typ | Entkopplungs-WP |
|---|---|---|---|---|
| `contextra-db` | Ring 4 | `crates/contextra-db/Cargo.toml` | normal | WP 2.3 (db/mcp/contextra) |
| `contextra-py` | Ring 4 | `crates/contextra-py/Cargo.toml` | normal | WP 2.3 (db/mcp/contextra) |
| `contextra-rank` | Ring 0 | `crates/contextra-rank/Cargo.toml` | dev-dependency | WP 2.3 (db/mcp/contextra) |

#### Quellcode-Nutzung (`.rs`)

| Dependent Crate | Ring | Datei:Zeile | Quellcode-Nutzung | Entkopplungs-WP |
|---|---|---|---|---|
| `contextra-db` | Ring 4 | `crates/contextra-db/src/homeostat.rs:5` | `pub use contextra_adapt::homeostat::RerankDeadline;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/homeostat.rs:7` | `pub use contextra_adapt::homeostat::RerankPidController;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/multistep.rs:8` | `use contextra_adapt::PidController;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:77` | `pub use contextra_adapt::decay_controller::*;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:83` | `pub use contextra_adapt::pid_latency_controller::*;` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/src/lib.rs:358` | `contextra_adapt::PidController::new(150.0, 50, 200, Some(100)),` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `crates/contextra-db/tests/pid_pool_production_caller.rs:4` | `use contextra_adapt::PidController;` | WP 2.3 (db/mcp/contextra) |
| `contextra-rank` | Ring 0 | `crates/contextra-rank/tests/calibration_deep_tests.rs:7` | `use contextra_adapt::PidController;` | WP 2.3 (db/mcp/contextra) |
| `contextra-rank` | Ring 0 | `crates/contextra-rank/tests/calibration_stress_and_fault_tests.rs:7` | `use contextra_adapt::PidController;` | WP 2.3 (db/mcp/contextra) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/query_builder/builder.rs:45` | `pub(super) pid_controller: Option<Arc<parking_lot::Mutex<contextra_adapt::PidController>>>,` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/query_builder/builder.rs:226` | `pid: Arc<parking_lot::Mutex<contextra_adapt::PidController>>,` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/query_builder/tests.rs:448` | `contextra_adapt::PidController::default(),` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/search_tests.rs:255` | `// Regression test: verify that exactly one PID controller type (contextra_adapt::PidController)` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/collection/tests/search_tests.rs:257` | `let mut pid = contextra_adapt::PidController::default();` | WP 2.2 (engine) |

---

### `contextra-kvcache`

#### Manifest-Abhängigkeiten (`Cargo.toml`)

Keine direkten Manifest-Abhängigkeiten in aktiven Crates.

#### Quellcode-Nutzung (`.rs`)

Keine Quellcode-Nutzungen in aktiven Crates gefunden.

---

### `contextra-router`

#### Manifest-Abhängigkeiten (`Cargo.toml`)

| Dependent Crate | Ring | Manifest-Datei | Typ | Entkopplungs-WP |
|---|---|---|---|---|
| `contextra-py` | Ring 4 | `crates/contextra-py/Cargo.toml` | normal | WP 2.3 (db/mcp/contextra) |
| `contextra` | Ring 4 | `crates/contextra/Cargo.toml` | normal | WP 2.3 (db/mcp/contextra) |

#### Quellcode-Nutzung (`.rs`)

| Dependent Crate | Ring | Datei:Zeile | Quellcode-Nutzung | Entkopplungs-WP |
|---|---|---|---|---|
| `contextra` | Ring 4 | `crates/contextra/src/lib.rs:26` | `pub use contextra_router as router;` | WP 2.3 (db/mcp/contextra) |
| `contextra-types` | Ring 0 | `crates/contextra-types/src/retrieval_strategy.rs:2` | `//! ACHTUNG: NICHT zu verwechseln mit `contextra_router::RoutingStrategy`` | WP 2.1b (physische Entfernung) |

---

### `contextra-agent`

#### Manifest-Abhängigkeiten (`Cargo.toml`)

Keine direkten Manifest-Abhängigkeiten in aktiven Crates.

#### Quellcode-Nutzung (`.rs`)

Keine Quellcode-Nutzungen in aktiven Crates gefunden.

---

### `contextra-infer-onnx`

#### Manifest-Abhängigkeiten (`Cargo.toml`)

| Dependent Crate | Ring | Manifest-Datei | Typ | Entkopplungs-WP |
|---|---|---|---|---|
| `contextra` | Ring 4 | `crates/contextra/Cargo.toml` | normal | WP 2.3 (db/mcp/contextra) |
| `contextra-bench` | Tooling | `benchmarks/contextra-bench/Cargo.toml` | normal | WP 2.1b (physische Entfernung) |

#### Quellcode-Nutzung (`.rs`)

| Dependent Crate | Ring | Datei:Zeile | Quellcode-Nutzung | Entkopplungs-WP |
|---|---|---|---|---|
| `contextra` | Ring 4 | `crates/contextra/src/lib.rs:38` | `pub use contextra_infer_onnx as onnx;` | WP 2.3 (db/mcp/contextra) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/contextra_impl/lifecycle.rs:183` | `let model_dir = contextra_infer_onnx::ensure_onnx_model_download(` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `crates/contextra-engine/src/contextra_impl/lifecycle.rs:188` | `let embedder = contextra_infer_onnx::OnnxEmbedder::load(model_dir)?;` | WP 2.2 (engine) |
| `contextra-bench` | Tooling | `benchmarks/contextra-bench/src/main.rs:14` | `use contextra_infer_onnx::{CrossEncoderReranker, RerankConfig};` | WP 2.1b (physische Entfernung) |

---

### `contextra-infer-ollama`

#### Manifest-Abhängigkeiten (`Cargo.toml`)

| Dependent Crate | Ring | Manifest-Datei | Typ | Entkopplungs-WP |
|---|---|---|---|---|
| `contextra` | Ring 4 | `crates/contextra/Cargo.toml` | normal | WP 2.3 (db/mcp/contextra) |

#### Quellcode-Nutzung (`.rs`)

| Dependent Crate | Ring | Datei:Zeile | Quellcode-Nutzung | Entkopplungs-WP |
|---|---|---|---|---|
| `contextra` | Ring 4 | `crates/contextra/src/lib.rs:32` | `pub use contextra_infer_ollama as ollama;` | WP 2.3 (db/mcp/contextra) |

---

## 4. Verstöße gegen INV-CORE-ISOLATION-1 (Kern -> Rand/Archiv)

INV-CORE-ISOLATION-1 verlangt, dass Kern-Crates (`[K]`: Ring 0-3 Core Product) im Release-Default weder normal noch build von Rand- (`[R]`), Experimentell- (`[E]`) oder Archiv-Crates (`[A]`) abhängen.

| Kern-Crate [K] | Ring | Ziel-Crate | Ziel-Klasse | Kanten-Art | Datei / Fundstelle | Entkopplungs-WP |
|---|---|---|---|---|---|---|
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Manifest (normal) | `crates/contextra-db/Cargo.toml` | WP 2.3 (db/mcp/contextra) |
| `contextra-store` | Ring 1 | `contextra-graph` | Archiv [A] | Manifest (dev-dependency) | `crates/contextra-store/Cargo.toml` | WP 2.1b (physische Entfernung) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/multistep.rs:373` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:107` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/tests/integration_temporal_filter.rs:2` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/tests/hnsw_delete_and_backfill.rs:5` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/tests/fault_injection_2pc.rs:6` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/tests/pid_pool_production_caller.rs:7` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/tests/cross_domain_chaos_matrix_test.rs:7` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/tests/graph_entity_delete_test.rs:3` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/tests/truncation_filter_recall_test.rs:2` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/tests/multistep_query_rewriter_modalities.rs:6` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/tests/diskann_collection_fallback.rs:8` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/tests/supersedes_cascading_tombstone_test.rs:4` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/tests/consolidation_double_trigger_test.rs:11` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/tests/cascade_invalidation_integration.rs:2` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/tests/cascade_invalidation_test.rs:2` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-db/tests/search_result_bound.rs:2` | WP 2.3 (db/mcp/contextra) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/maintenance.rs:10` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/maintenance.rs:869` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/maintenance.rs:872` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/maintenance.rs:878` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/maintenance.rs:919` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/mod.rs:32` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/relate.rs:105` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/relate.rs:125` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/relate.rs:141` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/relate.rs:143` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/relate.rs:146` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/relate.rs:155` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/relate.rs:157` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/relate.rs:169` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/query_builder/tests.rs:3` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/crud/links.rs:180` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:5` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:69` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:191` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:235` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:279` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:368` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:485` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:547` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/concurrency_tests.rs:586` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/link_tests.rs:5` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/link_tests.rs:56` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/link_tests.rs:167` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/link_tests.rs:307` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/link_tests.rs:404` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/search_tests.rs:5` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/search_tests.rs:48` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/search_tests.rs:198` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/search_tests.rs:273` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/search_tests.rs:352` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/search_tests.rs:449` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/read_tests.rs:5` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/read_tests.rs:148` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/read_tests.rs:282` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/insert_tests.rs:5` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/insert_tests.rs:74` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/insert_tests.rs:125` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/insert_tests.rs:176` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/insert_tests.rs:227` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/misc_tests.rs:27` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/misc_tests.rs:133` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/misc_tests.rs:345` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/misc_tests.rs:397` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/misc_tests.rs:448` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/misc_tests.rs:490` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/misc_tests.rs:538` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:5` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:51` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:103` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:163` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:288` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:407` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:463` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:547` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:662` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:746` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:827` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:928` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:995` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:1101` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:1196` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/update_delete_tests.rs:5` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/update_delete_tests.rs:59` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/update_delete_tests.rs:129` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/update_delete_tests.rs:196` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/update_delete_tests.rs:244` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/update_delete_tests.rs:292` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/transaction/db_transaction.rs:23` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/transaction/db_transaction.rs:127` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/transaction/tests.rs:4` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/contextra_impl/collections.rs:59` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/contextra_impl/collections.rs:169` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/background_workers/tests.rs:5` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/background_workers/tests.rs:102` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/background_workers/tests.rs:164` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/background_workers/tests.rs:230` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/background_workers/tests.rs:388` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/background_workers/tests.rs:497` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/background_workers/tests.rs:551` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/background_workers/tests.rs:594` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/background_workers/tests.rs:687` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/background_workers/hyperedge_worker.rs:6` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/tests/commit_crash_consistency.rs:10` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/tests/hybrid_adaptive_overfetch.rs:3` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/tests/hybrid_signal_failure_semantics.rs:4` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/tests/engine_recovery_crash_windows.rs:9` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/tests/ssi_read_path_coverage.rs:171` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/tests/hybrid_pathrag_snapshot.rs:3` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/tests/query_builder_scope_acorn.rs:9` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/tests/engine_commit_crash_points.rs:4` | WP 2.2 (engine) |
| `contextra-types` | Ring 0 | `contextra-graph` | Archiv [A] | Quellcode-Import | `crates/contextra-types/src/types/domain/misc.rs:344` | WP 2.1b (physische Entfernung) |
| `contextra-db` | Ring 4 | `contextra-cognition` | Archiv [A] | Manifest (normal) | `crates/contextra-db/Cargo.toml` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-cognition` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:7` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-cognition` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:30` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-cognition` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:31` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-cognition` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:43` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-cognition` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:46` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-cognition` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:49` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-cognition` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:52` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-cognition` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:55` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-cognition` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:58` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-cognition` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:61` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-cognition` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:64` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-cognition` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:73` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-adapt` | Archiv [A] | Manifest (normal) | `crates/contextra-db/Cargo.toml` | WP 2.3 (db/mcp/contextra) |
| `contextra-rank` | Ring 0 | `contextra-adapt` | Archiv [A] | Manifest (dev-dependency) | `crates/contextra-rank/Cargo.toml` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-adapt` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/homeostat.rs:5` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-adapt` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/homeostat.rs:7` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-adapt` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/multistep.rs:8` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-adapt` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:77` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-adapt` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:83` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-adapt` | Archiv [A] | Quellcode-Import | `crates/contextra-db/src/lib.rs:358` | WP 2.3 (db/mcp/contextra) |
| `contextra-db` | Ring 4 | `contextra-adapt` | Archiv [A] | Quellcode-Import | `crates/contextra-db/tests/pid_pool_production_caller.rs:4` | WP 2.3 (db/mcp/contextra) |
| `contextra-rank` | Ring 0 | `contextra-adapt` | Archiv [A] | Quellcode-Import | `crates/contextra-rank/tests/calibration_deep_tests.rs:7` | WP 2.3 (db/mcp/contextra) |
| `contextra-rank` | Ring 0 | `contextra-adapt` | Archiv [A] | Quellcode-Import | `crates/contextra-rank/tests/calibration_stress_and_fault_tests.rs:7` | WP 2.3 (db/mcp/contextra) |
| `contextra-engine` | Ring 3 | `contextra-adapt` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/query_builder/builder.rs:45` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-adapt` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/query_builder/builder.rs:226` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-adapt` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/query_builder/tests.rs:448` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-adapt` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/search_tests.rs:255` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-adapt` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/collection/tests/search_tests.rs:257` | WP 2.2 (engine) |
| `contextra-types` | Ring 0 | `contextra-router` | Archiv [A] | Quellcode-Import | `crates/contextra-types/src/retrieval_strategy.rs:2` | WP 2.1b (physische Entfernung) |
| `contextra-engine` | Ring 3 | `contextra-infer-onnx` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/contextra_impl/lifecycle.rs:183` | WP 2.2 (engine) |
| `contextra-engine` | Ring 3 | `contextra-infer-onnx` | Archiv [A] | Quellcode-Import | `crates/contextra-engine/src/contextra_impl/lifecycle.rs:188` | WP 2.2 (engine) |

*Hinweis: Diese Verstöße sind Ist-Befunde. Sie werden in WP 2.2 (`engine`) und WP 2.3 (`db`/`mcp`/`contextra`) behoben.*
