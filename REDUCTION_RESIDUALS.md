# Residuen nach Reduktion

Treffer = Referenzen auf entfernte Crates, Module, Features in verbleibendem Code.
Eingabe für die Prompts P1–P6 (`REDUCTION_PROMPTS.md`).

## Zusammenfassung

| Crate | Treffer | Dateien |
|---|---:|---:|
| `contextra-engine` | 193 | 40 |
| `contextra-crypto` | 53 | 18 |
| `contextra-db` | 46 | 14 |
| `contextra-bench` | 18 | 4 |
| `contextra` | 17 | 7 |
| `contextra-vector` | 12 | 5 |
| `contextra-store` | 8 | 5 |
| `contextra-types` | 7 | 3 |
| `contextra-testkit` | 4 | 1 |
| `contextra-ports` | 3 | 1 |
| `contextra-text` | 3 | 3 |
| `contextra-rank` | 2 | 2 |

## Gelöschte Test-/Example-Dateien (restorebar via `git checkout pre-reduction-1.0 -- <Pfad>`)

- keine

## Dokumente mit Verweisen auf entfernte Crates

| Datei | Verweise |
|---|---:|
| `README.md` | 18 |
| `CLAUDE.md` | 12 |
| `AGENTS.md` | 20 |
| `capabilities.toml` | 98 |
| `justfile` | 4 |
| `docs/SPEC.md` | 52 |

## Treffer je Crate

### `contextra-engine` (193)

```
crates/contextra-engine/src/background_workers/expiry_workers.rs:7: use crate::decay_controller::{AdaptiveDecayController, DecayControllerConfig};
crates/contextra-engine/src/background_workers/expiry_workers.rs:85: let decay_controller = AdaptiveDecayController::new(decay_config);
crates/contextra-engine/src/background_workers/expiry_workers.rs:92: match collection.evict_decayed_chunks(&decay_controller, 100).await {
crates/contextra-engine/src/background_workers/hyperedge_worker.rs:6: use contextra_graph::hyperedge::HyperEdgeId;
crates/contextra-engine/src/background_workers/tests.rs:5: use contextra_graph::hyperedge::HyperEdgeId;
crates/contextra-engine/src/background_workers/tests.rs:102: use contextra_graph::CsrGraph;
crates/contextra-engine/src/background_workers/tests.rs:164: use contextra_graph::CsrGraph;
crates/contextra-engine/src/background_workers/tests.rs:230: use contextra_graph::CsrGraph;
crates/contextra-engine/src/background_workers/tests.rs:388: use contextra_graph::CsrGraph;
crates/contextra-engine/src/background_workers/tests.rs:497: use contextra_graph::CsrGraph;
crates/contextra-engine/src/background_workers/tests.rs:551: use contextra_graph::CsrGraph;
crates/contextra-engine/src/background_workers/tests.rs:593: use crate::decay_controller::{AdaptiveDecayController, DecayControllerConfig};
crates/contextra-engine/src/background_workers/tests.rs:594: use contextra_graph::CsrGraph;
crates/contextra-engine/src/background_workers/tests.rs:660: let decay_controller = AdaptiveDecayController::new(DecayControllerConfig {
crates/contextra-engine/src/background_workers/tests.rs:667: .evict_decayed_chunks(&decay_controller, 100)
crates/contextra-engine/src/background_workers/tests.rs:686: use crate::decay_controller::DecayControllerConfig;
crates/contextra-engine/src/background_workers/tests.rs:687: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/crud/auto_extraction.rs:8: #[cfg(feature = "entity-extraction")]
crates/contextra-engine/src/collection/crud/auto_extraction.rs:9: pub use crate::extraction::{EntityExtractionConfig, ExtractorMode};
crates/contextra-engine/src/collection/crud/auto_extraction.rs:16: #[cfg(not(feature = "entity-extraction"))]
crates/contextra-engine/src/collection/crud/auto_extraction.rs:23: #[cfg(not(feature = "entity-extraction"))]
crates/contextra-engine/src/collection/crud/auto_extraction.rs:30: #[cfg(not(feature = "entity-extraction"))]
crates/contextra-engine/src/collection/crud/auto_extraction.rs:38: #[cfg(not(feature = "entity-extraction"))]
crates/contextra-engine/src/collection/crud/auto_extraction.rs:120: if cfg!(feature = "auto-extraction-opt-out") {
crates/contextra-engine/src/collection/crud/auto_extraction.rs:129: let mode = if cfg!(feature = "auto-extraction-opt-out") {
crates/contextra-engine/src/collection/crud/auto_extraction.rs:134: let enabled = mode.is_enabled() && !cfg!(feature = "auto-extraction-opt-out");
crates/contextra-engine/src/collection/crud/auto_extraction.rs:245: #[cfg(feature = "entity-extraction")]
crates/contextra-engine/src/collection/crud/auto_extraction.rs:247: let triples = match crate::extraction::extract_triples_for_mode(
crates/contextra-engine/src/collection/crud/auto_extraction.rs:299: #[cfg(not(feature = "entity-extraction"))]
crates/contextra-engine/src/collection/crud/links.rs:82: contextra_graph::cascade_invalidate_edges_for_superseded_doc(
crates/contextra-engine/src/collection/maintenance.rs:8: use crate::decay_controller::{AdaptiveDecayController, DecaySignalInputs};
crates/contextra-engine/src/collection/maintenance.rs:12: use contextra_graph::{detect_communities, CommunityAssignment, CommunityDetectionConfig};
crates/contextra-engine/src/collection/maintenance.rs:22: #[tracing::instrument(level = "trace", skip(self, decay_controller))]
crates/contextra-engine/src/collection/maintenance.rs:25: decay_controller: &AdaptiveDecayController,
crates/contextra-engine/src/collection/maintenance.rs:101: if decay_controller.should_evict(base_score, elapsed_tx, &inputs) {
crates/contextra-engine/src/collection/maintenance.rs:861: #[cfg(feature = "graph-connectivity-health")]
crates/contextra-engine/src/collection/maintenance.rs:869: #[cfg(feature = "graph-connectivity-health")]
crates/contextra-engine/src/collection/maintenance.rs:876: config: &contextra_graph::percolation::PercolationConfig,
crates/contextra-engine/src/collection/maintenance.rs:879: let health = contextra_graph::percolation::compute_percolation_health(
crates/contextra-engine/src/collection/maintenance.rs:885: contextra_graph::percolation::should_trigger_rebonding(phi, config)
crates/contextra-engine/src/collection/maintenance.rs:926: let candidate_pairs = contextra_graph::percolation::find_rebonding_candidates(
crates/contextra-engine/src/collection/mod.rs:32: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/query_builder/builder.rs:44: #[cfg(feature = "adaptive-candidate-pool-sizing")]
crates/contextra-engine/src/collection/query_builder/builder.rs:45: pub(super) pid_controller: Option<Arc<parking_lot::Mutex<contextra_adapt::PidController>>>,
crates/contextra-engine/src/collection/query_builder/builder.rs:76: #[cfg(feature = "adaptive-candidate-pool-sizing")]
crates/contextra-engine/src/collection/query_builder/builder.rs:223: #[cfg(feature = "adaptive-candidate-pool-sizing")]
crates/contextra-engine/src/collection/query_builder/builder.rs:226: pid: Arc<parking_lot::Mutex<contextra_adapt::PidController>>,
crates/contextra-engine/src/collection/query_builder/builder_exec.rs:95: #[cfg(feature = "adaptive-candidate-pool-sizing")]
crates/contextra-engine/src/collection/query_builder/builder_exec.rs:104: #[cfg(not(feature = "adaptive-candidate-pool-sizing"))]
crates/contextra-engine/src/collection/query_builder/builder_exec.rs:264: #[cfg(feature = "adaptive-candidate-pool-sizing")]
crates/contextra-engine/src/collection/query_builder/tests.rs:3: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/query_builder/tests.rs:428: #[cfg(feature = "adaptive-candidate-pool-sizing")]
crates/contextra-engine/src/collection/query_builder/tests.rs:448: contextra_adapt::PidController::default(),
crates/contextra-engine/src/collection/relate.rs:105: ) -> Result<contextra_graph::hyperedge::HyperEdgeId> {
crates/contextra-engine/src/collection/relate.rs:125: let role_id = contextra_graph::hyperedge::RoleId::new(role_doc_id.inner() as u32);
crates/contextra-engine/src/collection/relate.rs:141: let hyperedge_id = contextra_graph::hyperedge::HyperEdgeId::new(db_tx.tx_id.inner());
crates/contextra-engine/src/collection/relate.rs:143: let role_bindings: Vec<contextra_graph::hyperedge::RoleBinding> = resolved_participants
crates/contextra-engine/src/collection/relate.rs:146: contextra_graph::hyperedge::RoleBinding::new(role_id, entity_id)
crates/contextra-engine/src/collection/relate.rs:155: let hyperedge = contextra_graph::hyperedge::HyperEdge::new(
crates/contextra-engine/src/collection/relate.rs:157: contextra_graph::csr::EdgeType::Default,
crates/contextra-engine/src/collection/relate.rs:169: contextra_graph::hyperedge::HYPEREDGE_PREFIX,
crates/contextra-engine/src/collection/tests/concurrency_tests.rs:5: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/concurrency_tests.rs:69: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/concurrency_tests.rs:191: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/concurrency_tests.rs:235: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/concurrency_tests.rs:279: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/concurrency_tests.rs:368: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/concurrency_tests.rs:485: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/concurrency_tests.rs:547: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/concurrency_tests.rs:586: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:5: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:51: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:101: #[cfg(feature = "reranking")]
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:103: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:161: #[cfg(feature = "experimental-diskann")]
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:163: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:168: use contextra_vector::{DiskAnnConfig, DiskAnnIndex};
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:175: let diskann_path = dir.path().join("diskann.idx");
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:186: let diskann_config = DiskAnnConfig {
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:192: ..DiskAnnConfig::default()
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:195: let diskann = Arc::new(DiskAnnIndex::try_new(diskann_config).unwrap()); // unwrap
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:203: diskann.build(&vectors, &ids).await.unwrap(); // unwrap
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:208: let col = Collection::<LsmStorage, DiskAnnIndex>::new(
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:211: diskann,
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:288: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:407: use contextra_graph::csr::CsrGraph;
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:463: use contextra_graph::csr::CsrGraph;
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:547: use contextra_graph::csr::CsrGraph;
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:662: use contextra_graph::csr::CsrGraph;
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:746: use contextra_graph::csr::CsrGraph;
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:827: use contextra_graph::csr::CsrGraph;
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:928: use contextra_graph::csr::CsrGraph;
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:995: use contextra_graph::csr::CsrGraph;
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:1101: use contextra_graph::csr::CsrGraph;
crates/contextra-engine/src/collection/tests/hybrid_search_tests.rs:1196: use contextra_graph::csr::CsrGraph;
crates/contextra-engine/src/collection/tests/insert_tests.rs:5: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/insert_tests.rs:74: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/insert_tests.rs:125: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/insert_tests.rs:176: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/insert_tests.rs:227: use contextra_graph::csr::CsrGraph;
crates/contextra-engine/src/collection/tests/link_tests.rs:5: use contextra_graph::csr::CsrGraph;
crates/contextra-engine/src/collection/tests/link_tests.rs:56: use contextra_graph::csr::CsrGraph;
crates/contextra-engine/src/collection/tests/link_tests.rs:167: use contextra_graph::csr::{CsrGraph, CsrGraphConfig};
crates/contextra-engine/src/collection/tests/link_tests.rs:307: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/link_tests.rs:365: #[cfg(feature = "graph-connectivity-health")]
crates/contextra-engine/src/collection/tests/link_tests.rs:404: let config = contextra_graph::percolation::PercolationConfig {
crates/contextra-engine/src/collection/tests/misc_tests.rs:27: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/misc_tests.rs:133: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/misc_tests.rs:345: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/misc_tests.rs:397: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/misc_tests.rs:448: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/misc_tests.rs:490: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/misc_tests.rs:538: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/read_tests.rs:5: use contextra_graph::csr::CsrGraph;
crates/contextra-engine/src/collection/tests/read_tests.rs:148: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/read_tests.rs:244: let decay_controller = crate::decay_controller::AdaptiveDecayController::with_defaults();
crates/contextra-engine/src/collection/tests/read_tests.rs:267: .evict_decayed_chunks(&decay_controller, 100)
crates/contextra-engine/src/collection/tests/read_tests.rs:282: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/search_tests.rs:5: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/search_tests.rs:48: use contextra_graph::csr::CsrGraph;
crates/contextra-engine/src/collection/tests/search_tests.rs:198: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/search_tests.rs:256: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/search_tests.rs:335: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/search_tests.rs:432: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/update_delete_tests.rs:5: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/update_delete_tests.rs:59: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/update_delete_tests.rs:129: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/update_delete_tests.rs:196: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/update_delete_tests.rs:244: use contextra_graph::CsrGraph;
crates/contextra-engine/src/collection/tests/update_delete_tests.rs:292: use contextra_graph::CsrGraph;
crates/contextra-engine/src/contextra_impl/collections.rs:59: contextra_graph::CsrGraph::load_from_storage(tenant_storage.as_ref()).await?;
crates/contextra-engine/src/contextra_impl/collections.rs:169: let mut graph = contextra_graph::CsrGraph::load_from_storage(self.storage.as_ref()).await?;
crates/contextra-engine/src/contextra_impl/collections.rs:282: ) -> Result<DeletionProof> {
crates/contextra-engine/src/contextra_impl/collections.rs:284: impl contextra_impl::erasure::ProofKeyProvider for InlineKeyProvider {
crates/contextra-engine/src/contextra_impl/collections.rs:306: "ErasureReport was verified but contained no DeletionProof".to_string(),
crates/contextra-engine/src/contextra_impl/diagnostics.rs:51: pub fn orphan_registry(&self) -> &Arc<contextra_checkpoint::InstanceOrphanRegistry> {
crates/contextra-engine/src/contextra_impl/lifecycle.rs:112: let orphan_registry = Arc::new(contextra_checkpoint::InstanceOrphanRegistry::new(
crates/contextra-engine/src/contextra_impl/lifecycle.rs:186: #[cfg(feature = "onnx")]
crates/contextra-engine/src/contextra_impl/lifecycle.rs:188: let model_dir = contextra_infer_onnx::ensure_onnx_model_download(
crates/contextra-engine/src/contextra_impl/lifecycle.rs:193: let embedder = contextra_infer_onnx::OnnxEmbedder::load(model_dir)?;
crates/contextra-engine/src/contextra_impl/lifecycle.rs:197: #[cfg(not(feature = "onnx"))]
crates/contextra-engine/src/lib.rs:8: pub use contextra_crypto::deletion_proof::{
crates/contextra-engine/src/lib.rs:9: DeletionLayer, DeletionProof, DeletionScope, ExcludedScope,
crates/contextra-engine/src/lib.rs:12: #[cfg(feature = "sandbox")]
crates/contextra-engine/src/lib.rs:48: #[cfg(feature = "sandbox")]
crates/contextra-engine/src/lib.rs:50: #[cfg(feature = "sandbox")]
crates/contextra-engine/src/lib.rs:69: #[cfg(feature = "graph-connectivity-health")]
crates/contextra-engine/src/lib.rs:72: #[cfg(feature = "graph-connectivity-health")]
crates/contextra-engine/src/lib.rs:78: pub use contextra_checkpoint;
crates/contextra-engine/src/lib.rs:338: orphan_registry: Arc<contextra_checkpoint::InstanceOrphanRegistry>,
crates/contextra-engine/src/lib.rs:345: parking_lot::RwLock<Option<Arc<dyn contextra_impl::erasure::ProofKeyProvider>>>,
crates/contextra-engine/src/lib.rs:349: pub use contextra_impl::erasure::{
crates/contextra-engine/src/lib.rs:378: #[cfg(feature = "sandbox")]
crates/contextra-engine/src/lib.rs:396: #[cfg(feature = "sandbox")]
crates/contextra-engine/src/merge_operator_adapter.rs:13: //! [`contextra_sandbox::WasmMergeFunction`] via a dedicated worker thread hosting an isolated
crates/contextra-engine/src/merge_operator_adapter.rs:23: use contextra_sandbox::{MergeOperatorCapabilities, SandboxError, WasmMergeFunction};
crates/contextra-engine/src/transaction/db_transaction.rs:23: pub(super) staged_hyperedges: Mutex<Vec<contextra_graph::hyperedge::HyperEdge>>,
crates/contextra-engine/src/transaction/db_transaction.rs:127: pub fn stage_hyperedge(&self, hyperedge: contextra_graph::hyperedge::HyperEdge) {
crates/contextra-engine/src/transaction/tests.rs:4: use contextra_graph::CsrGraph;
crates/contextra-engine/tests/auto_extraction_default_enabled.rs:11: #[cfg(not(feature = "auto-extraction-opt-out"))]
crates/contextra-engine/tests/auto_extraction_default_enabled.rs:19: #[cfg(feature = "auto-extraction-opt-out")]
crates/contextra-engine/tests/auto_extraction_runtime_mode.rs:11: #[cfg(feature = "entity-extraction")]
crates/contextra-engine/tests/auto_extraction_runtime_mode.rs:92: #[cfg(feature = "entity-extraction")]
crates/contextra-engine/tests/auto_extraction_runtime_mode.rs:151: #[cfg(feature = "entity-extraction")]
crates/contextra-engine/tests/auto_extraction_runtime_mode.rs:153: if cfg!(not(feature = "auto-extraction-opt-out")) {
crates/contextra-engine/tests/auto_extraction_runtime_mode.rs:236: #[cfg(feature = "entity-extraction")]
crates/contextra-engine/tests/auto_extraction_runtime_mode.rs:238: if cfg!(not(feature = "auto-extraction-opt-out")) {
crates/contextra-engine/tests/auto_extraction_wiring.rs:138: #[cfg(feature = "entity-extraction")]
crates/contextra-engine/tests/auto_extraction_wiring.rs:197: #[cfg(feature = "entity-extraction")]
crates/contextra-engine/tests/auto_extraction_wiring.rs:286: #[cfg(feature = "entity-extraction")]
crates/contextra-engine/tests/auto_extraction_wiring.rs:288: if cfg!(not(feature = "auto-extraction-opt-out")) {
crates/contextra-engine/tests/auto_triple_extraction.rs:87: #[cfg(feature = "entity-extraction")]
crates/contextra-engine/tests/auto_triple_extraction.rs:89: if cfg!(not(feature = "auto-extraction-opt-out")) {
crates/contextra-engine/tests/commit_crash_consistency.rs:10: use contextra_graph::CsrGraph;
crates/contextra-engine/tests/engine_commit_crash_points.rs:4: use contextra_graph::CsrGraph;
crates/contextra-engine/tests/engine_recovery_crash_windows.rs:9: use contextra_graph::CsrGraph;
crates/contextra-engine/tests/hybrid_signal_failure_semantics.rs:4: use contextra_graph::CsrGraph;
crates/contextra-engine/tests/invariant_tenant_isolation.rs:51: // 4. Drop collection for Tenant X and verify DeletionProof
crates/contextra-engine/tests/invariant_tenant_isolation.rs:63: "DeletionProof tenant_id must match requested tenant"
crates/contextra-engine/tests/openie_determinism_and_provenance.rs:10: #[cfg(feature = "entity-extraction")]
crates/contextra-engine/tests/openie_determinism_and_provenance.rs:11: use contextra_engine::extraction::extract_triples;
crates/contextra-engine/tests/openie_determinism_and_provenance.rs:14: #[cfg(feature = "entity-extraction")]
crates/contextra-engine/tests/openie_determinism_and_provenance.rs:57: #[cfg(feature = "entity-extraction")]
crates/contextra-engine/tests/openie_determinism_and_provenance.rs:137: #[cfg(feature = "entity-extraction")]
crates/contextra-engine/tests/openie_determinism_and_provenance.rs:194: #[cfg(feature = "entity-extraction")]
crates/contextra-engine/tests/proof_verify_not_constant.rs:1: use contextra_crypto::deletion_proof::{CustodyLevel, DeletionProof};
crates/contextra-engine/tests/proof_verify_not_constant.rs:2: use contextra_ports::scanner::ScanTechnique;
crates/contextra-engine/tests/proof_verify_not_constant.rs:22: let proof = DeletionProof::issue(
crates/contextra-engine/tests/query_builder_scope_acorn.rs:9: use contextra_graph::CsrGraph;
crates/contextra-engine/tests/ssi_read_path_coverage.rs:171: let graph = Arc::new(contextra_graph::CsrGraph::new());
crates/contextra-engine/tests/wasm_merge_operator.rs:1: #![cfg(feature = "sandbox")]
crates/contextra-engine/tests/wasm_merge_operator.rs:228: use contextra_core::{ResourceBudget, ResourceTracker, SnapshotRegistry};
crates/contextra-engine/tests/wasm_merge_operator.rs:299: use contextra_core::{ResourceBudget, ResourceTracker, SnapshotRegistry};
```

### `contextra-crypto` (53)

```
crates/contextra-crypto/benches/crypto_benchmarks.rs:64: use contextra_crypto::kv_segment::{KvSegment, TenantIsolatedKvStore};
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs:6: use contextra_crypto::deletion_proof::{
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs:7: DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope, LayerCleanupProof,
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs:17: pub struct DeletionProofTamperInput {
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs:25: fuzz_target!(|input: DeletionProofTamperInput| {
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs:33: if let Ok(proof) = bincode::deserialize::<DeletionProof>(&input.raw_bytes) {
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs:52: let valid_proof = match DeletionProof::create(
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs:71: match bincode::deserialize::<DeletionProof>(&serialized) {
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs:94: let keypair = DeletionProofKeyPair::generate();
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs:102: if let Ok(mut v3_proof) = DeletionProof::create_v3(
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs:122: use contextra_ports::scanner::ScanTechnique;
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs:135: if let Ok(mut v5_proof) = DeletionProof::issue(
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs:140: contextra_crypto::deletion_proof::CustodyLevel::Local,
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs:6: use contextra_crypto::deletion_proof::{
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs:7: DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope, LayerCleanupProof,
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs:12: /// Arbitrary raw bytes to attempt bincode deserialization into DeletionProof
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs:16: /// Fields to construct synthetic DeletionProof instances
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs:32: if let Ok(deserialized_proof) = bincode::deserialize::<DeletionProof>(&input.raw_proof_bytes) {
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs:36: // Strategy 2: Construct synthetic DeletionProof with arbitrary/corrupt fields
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs:76: let synthetic_proof = DeletionProof {
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs:88: custody: contextra_crypto::deletion_proof::CustodyLevel::Local,
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs:97: let keypair = DeletionProofKeyPair::generate();
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs:103: if let Ok(valid_v3) = DeletionProof::create_v3(
crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs:120: if let Ok(valid_v2) = DeletionProof::create(
crates/contextra-crypto/src/audit_chain.rs:20: use crate::kv_shredding::KeyRegistry;
crates/contextra-crypto/src/kv_cipher.rs:95: use crate::kv_shredding::KeyRegistry;
crates/contextra-crypto/tests/campaign_cov_revocation_log.rs:8: use contextra_crypto::revocation_log::{RevocationLog, RevocationTarget};
crates/contextra-crypto/tests/campaign_j25.rs:12: kv_segment::{segment::KvSegment, store::TenantIsolatedKvStore},
crates/contextra-crypto/tests/campaign_mut_j17.rs:8: use contextra_crypto::kv_shredding::{GroupKek, RecordDek};
crates/contextra-crypto/tests/crypto_crossimpl_python.rs:15: ed25519_proof::{sign_deletion_proof_v3, verify_deletion_proof_v3, DeletionProofKeyPair},
crates/contextra-crypto/tests/crypto_crossimpl_python.rs:151: let keypair = DeletionProofKeyPair {
crates/contextra-crypto/tests/crypto_vectors_extended.rs:14: ed25519_proof::{sign_deletion_proof_v3, verify_deletion_proof_v3, DeletionProofKeyPair},
crates/contextra-crypto/tests/crypto_vectors_extended.rs:37: // Contextra-Anbindung über contextra_crypto::ed25519_proof
crates/contextra-crypto/tests/crypto_vectors_extended.rs:61: let keypair = DeletionProofKeyPair {
crates/contextra-crypto/tests/crypto_vectors_extended.rs:120: let keypair = DeletionProofKeyPair {
crates/contextra-crypto/tests/crypto_vectors_extended.rs:159: let keypair = DeletionProofKeyPair {
crates/contextra-crypto/tests/eviction_worker_tokio_runtime_nonblocking_regression.rs:3: use contextra_crypto::kv_segment::segment::KvSegment;
crates/contextra-crypto/tests/eviction_worker_tokio_runtime_nonblocking_regression.rs:4: use contextra_crypto::kv_segment::store::TenantIsolatedKvStore;
crates/contextra-crypto/tests/eviction_worker_tokio_runtime_nonblocking_regression.rs:5: use contextra_crypto::kv_segment::EvictionWorker;
crates/contextra-crypto/tests/guarantee_audit_chain_tamper.rs:19: compute_record_commitment, AuditChain, AuditChainEntry, DataClass, DeletionProofKeyPair,
crates/contextra-crypto/tests/guarantee_audit_chain_tamper.rs:309: let keypair = DeletionProofKeyPair::generate();
crates/contextra-crypto/tests/guarantee_audit_chain_tamper.rs:377: let keypair = DeletionProofKeyPair::generate();
crates/contextra-crypto/tests/guarantee_audit_chain_tamper.rs:414: let keypair = DeletionProofKeyPair::generate();
crates/contextra-crypto/tests/guarantee_audit_chain_tamper.rs:415: let wrong_keypair = DeletionProofKeyPair::generate();
crates/contextra-crypto/tests/guarantee_audit_chain_tamper.rs:493: let keypair = DeletionProofKeyPair::generate();
crates/contextra-crypto/tests/kv_segment_concurrency.rs:5: use contextra_crypto::kv_segment::{
crates/contextra-crypto/tests/kv_segment_integration.rs:8: use contextra_crypto::kv_segment::{KvSegment, TenantIsolatedKvStore};
crates/contextra-crypto/tests/kv_segment_proptests.rs:7: use contextra_crypto::kv_segment::{KvSegment, TenantIsolatedKvStore};
crates/contextra-crypto/tests/legacy_hmac_collision_property.rs:6: use contextra_crypto::deletion_proof::hash_deleted_keys_length_prefixed;
crates/contextra-crypto/tests/loom_eviction_worker_shutdown_race.rs:54: /// Nachbildung von `EvictionWorker` aus `crates/contextra-crypto/src/kv_segment/eviction_worker.rs`.
crates/contextra-crypto/tests/loom_kv_deferred_zeroize.rs:24: // aus `crates/contextra-crypto/src/kv_segment/store.rs` unter Verwendung von `loom::sync::RwLock`
crates/contextra-crypto/tests/loom_kv_deferred_zeroize.rs:138: use contextra_crypto::kv_segment::segment::KvSegment;
crates/contextra-crypto/tests/loom_kv_deferred_zeroize.rs:139: use contextra_crypto::kv_segment::store::TenantIsolatedKvStore;
```

### `contextra-db` (46)

```
crates/contextra-db/src/lib.rs:41: pub use crate::multistep::{MultiStepConfig, MultiStepEngine, MultiStepResult, QueryRewriter};
crates/contextra-db/src/lib.rs:52: pub use crate::decay_controller::{
crates/contextra-db/src/lib.rs:55: pub use crate::homeostat::RerankDeadline;
crates/contextra-db/src/lib.rs:57: pub use crate::homeostat::RerankPidController;
crates/contextra-db/src/lib.rs:63: pub use contextra_cognition as cognition;
crates/contextra-db/src/lib.rs:64: pub use contextra_cognition::{
crates/contextra-db/src/lib.rs:81: #[cfg(feature = "graph-connectivity-health")]
crates/contextra-db/src/lib.rs:83: #[cfg(feature = "sandbox")]
crates/contextra-db/src/lib.rs:93: pub use contextra_cognition::context::*;
crates/contextra-db/src/lib.rs:97: pub use contextra_cognition::context_compaction::*;
crates/contextra-db/src/lib.rs:101: pub use contextra_cognition::memory_consolidation::*;
crates/contextra-db/src/lib.rs:105: pub use contextra_cognition::synthesis_phase::*;
crates/contextra-db/src/lib.rs:109: pub use contextra_cognition::consolidation_executor::*;
crates/contextra-db/src/lib.rs:113: pub use contextra_cognition::consolidation_locks::*;
crates/contextra-db/src/lib.rs:117: pub use contextra_cognition::maintenance_scheduler::*;
crates/contextra-db/src/lib.rs:121: pub use contextra_cognition::maintenance_config::*;
crates/contextra-db/src/lib.rs:131: pub use contextra_cognition::start_consolidation_reaper;
crates/contextra-db/src/lib.rs:135: pub mod decay_controller {
crates/contextra-db/src/lib.rs:136: pub use contextra_adapt::decay_controller::*;
crates/contextra-db/src/lib.rs:141: pub use contextra_adapt::pid_latency_controller::*;
crates/contextra-db/src/lib.rs:152: pub use decay_controller::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs};
crates/contextra-db/src/lib.rs:159: pub use contextra_checkpoint;
crates/contextra-db/src/lib.rs:160: #[cfg(feature = "graph-connectivity-health")]
crates/contextra-db/src/lib.rs:161: pub use contextra_graph::percolation::PercolationConfig;
crates/contextra-db/src/lib.rs:717: #[cfg(feature = "onnx")]
crates/contextra-db/tests/cascade_invalidation_integration.rs:2: use contextra_graph::path_rag::PathRAGEngine;
crates/contextra-db/tests/cascade_invalidation_test.rs:2: use contextra_graph::path_rag::PathRAGEngine;
crates/contextra-db/tests/cross_domain_chaos_matrix_test.rs:7: use contextra_graph::csr::CsrGraph;
crates/contextra-db/tests/fault_injection_2pc.rs:6: use contextra_graph::CsrGraph;
crates/contextra-db/tests/fusion_edge_cases_test.rs:11: #[cfg(feature = "coherence-bonus-fusion")]
crates/contextra-db/tests/fusion_edge_cases_test.rs:95: #[cfg(feature = "coherence-bonus-fusion")]
crates/contextra-db/tests/fusion_edge_cases_test.rs:189: #[cfg(feature = "coherence-bonus-fusion")]
crates/contextra-db/tests/hnsw_delete_and_backfill.rs:5: use contextra_graph::CsrGraph;
crates/contextra-db/tests/instance_orphan_registry_test.rs:3: use contextra_checkpoint::{global_orphan_registry, PinnedSeqNoOrphan, StateCheckpoint};
crates/contextra-db/tests/instance_orphan_registry_test.rs:80: contextra_checkpoint::clear_all_orphaned_checkpoints();
crates/contextra-db/tests/instance_orphan_registry_test.rs:107: let global_cps = contextra_checkpoint::get_orphaned_checkpoints();
crates/contextra-db/tests/integration_temporal_filter.rs:2: use contextra_graph::CsrGraph;
crates/contextra-db/tests/pid_pool_production_caller.rs:4: use contextra_adapt::PidController;
crates/contextra-db/tests/pid_pool_production_caller.rs:5: use contextra_db::multistep::{MultiStepConfig, MultiStepEngine};
crates/contextra-db/tests/pid_pool_production_caller.rs:7: use contextra_graph::CsrGraph;
crates/contextra-db/tests/pid_pool_production_caller.rs:106: let config = MultiStepConfig {
crates/contextra-db/tests/pid_pool_production_caller.rs:115: let engine = MultiStepEngine::new(col, config).with_clock(clock);
crates/contextra-db/tests/search_result_bound.rs:2: use contextra_graph::CsrGraph;
crates/contextra-db/tests/semantic_recall.rs:176: "diskann",
crates/contextra-db/tests/supersedes_cascading_tombstone_test.rs:4: use contextra_graph::PathRAGEngine;
crates/contextra-db/tests/truncation_filter_recall_test.rs:2: use contextra_graph::CsrGraph;
```

### `contextra-bench` (18)

```
benchmarks/contextra-bench/benches/competitive_kv.rs:5: use contextra_core::{StorageEngine, TxId};
benchmarks/contextra-bench/src/ann_benchmarks.rs:10: use contextra_core::traits::VectorIndex;
benchmarks/contextra-bench/src/ann_benchmarks.rs:11: use contextra_core::types::{DistanceMetric, DocId, TxId};
benchmarks/contextra-bench/src/main.rs:11: use contextra_core::{EmbeddingProvider, Result};
benchmarks/contextra-bench/src/main.rs:13: #[cfg(feature = "onnx-bench")]
benchmarks/contextra-bench/src/main.rs:14: use contextra_infer_onnx::{CrossEncoderReranker, RerankConfig};
benchmarks/contextra-bench/src/main.rs:694: #[cfg(feature = "onnx-bench")]
benchmarks/contextra-bench/src/main.rs:789: #[cfg(not(feature = "onnx-bench"))]
benchmarks/contextra-bench/src/main.rs:795: Err(contextra_core::ContextraError::CapabilityUnsupported {
benchmarks/contextra-bench/src/main.rs:1028: let embedder = std::sync::Arc::new(contextra_infer_candle::CandleEmbedClient::from_dir(
benchmarks/contextra-bench/src/main.rs:1030: contextra_infer_candle::model_registry::CandleQuantization::Q4KM,
benchmarks/contextra-bench/src/main.rs:1057: .map_err(|e| contextra_core::ContextraError::Internal(e.to_string()))?;
benchmarks/contextra-bench/src/main.rs:1076: .map_err(|e| contextra_core::ContextraError::Internal(e.to_string()))?;
benchmarks/contextra-bench/src/main.rs:1105: .map_err(|e| contextra_core::ContextraError::Internal(e.to_string()))?;
benchmarks/contextra-bench/src/main.rs:1498: #[cfg(feature = "onnx-bench")]
benchmarks/contextra-bench/src/path_rag_sweep.rs:8: use contextra_core::Result;
benchmarks/contextra-bench/src/path_rag_sweep.rs:104: Err(contextra_core::ContextraError::SnapshotUnsupportedForSignal(_)) => Vec::new(),
benchmarks/contextra-bench/src/path_rag_sweep.rs:258: Err(contextra_core::ContextraError::SnapshotUnsupportedForSignal(_)) => Vec::new(),
```

### `contextra` (17)

```
crates/contextra/src/agent_memory.rs:5: use contextra_core::error::ContextraError;
crates/contextra/src/builder.rs:9: use contextra_core::error::ContextraError;
crates/contextra/src/builder.rs:10: use contextra_core::DistanceMetric;
crates/contextra/src/builder.rs:12: use contextra_license::SignedLicenseGate;
crates/contextra/src/builder.rs:134: match contextra_license::signed_gate::VerifyingKey::from_bytes(verifying_key_bytes) {
crates/contextra/src/collection_profile.rs:6: pub use contextra_core::types::domain::AutoExtractionMode;
crates/contextra/src/collection_profile.rs:39: #[error("deletion-proof active but KvDeleteMode::TombstoneOnly cannot satisfy a KV-side DeletionProof")]
crates/contextra/src/lib.rs:15: pub use contextra_core::error::ContextraError;
crates/contextra/src/lib.rs:16: pub use contextra_core::types::domain::{DocId, ScoredDocument};
crates/contextra/src/lib.rs:17: pub use contextra_core::DistanceMetric;
crates/contextra/src/lib.rs:27: #[cfg(feature = "candle")]
crates/contextra/src/lib.rs:28: pub use contextra_infer_candle as candle;
crates/contextra/src/query_rewriter.rs:2: // ZWECK: LLM-gestützter QueryRewriter für MultiStep-Retrieval in der Facade (Ring 4).
crates/contextra/src/query_rewriter.rs:5: use contextra_core::error::Result;
crates/contextra/tests/llm_query_rewriter.rs:6: use contextra_core::error::ContextraError;
crates/contextra/tests/llm_query_rewriter.rs:43: ) -> BoxFuture<'a, contextra_core::error::Result<String>> {
crates/contextra/tests/tenant_isolation_facade.rs:2: use contextra_core::{ContextraError, TenantId};
```

### `contextra-vector` (12)

```
crates/contextra-vector/benches/filtered_search_selectivity_bench.rs:24: use contextra_vector::diskann::{DiskAnnConfig, DiskAnnIndex};
crates/contextra-vector/benches/filtered_search_selectivity_bench.rs:75: let index_path = temp_dir.path().join("selectivity_bench.diskann");
crates/contextra-vector/benches/filtered_search_selectivity_bench.rs:93: let config = DiskAnnConfig {
crates/contextra-vector/benches/filtered_search_selectivity_bench.rs:99: ..DiskAnnConfig::default()
crates/contextra-vector/benches/filtered_search_selectivity_bench.rs:102: let index = DiskAnnIndex::try_new(config).expect("DiskAnnIndex creation failed");
crates/contextra-vector/src/hnsw/purge.rs:9: use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};
crates/contextra-vector/src/hnsw/scanner.rs:8: use contextra_ports::scanner::{LayerScanner, ScanSession, WitnessLayer};
crates/contextra-vector/tests/hnsw_scanner_purge_test.rs:10: use contextra_ports::scanner::{ScanTarget, ScanTechnique};
crates/contextra-vector/tests/recall_audit.rs:5: #![cfg(feature = "experimental-diskann")]
crates/contextra-vector/tests/recall_audit.rs:9: use contextra_vector::diskann::{DiskAnnConfig, DiskAnnIndex};
crates/contextra-vector/tests/recall_audit.rs:181: let config = DiskAnnConfig {
crates/contextra-vector/tests/recall_audit.rs:190: let index = DiskAnnIndex::try_new(config).unwrap();
```

### `contextra-store` (8)

```
crates/contextra-store/src/kv/segment.rs:2: // ZWECK: Verdrahtung von KvDeleteMode und contextra_crypto::kv_shredding für KV-Segment-Verschlüsselung und Crypto-Shredding (AP-P0-06 / Sp
crates/contextra-store/src/kv/segment.rs:10: use contextra_crypto::{crypto::KeyManager, kv_shredding::KeyRegistry};
crates/contextra-store/tests/campaign_j21_hypotheses.rs:6: use contextra_crypto::{crypto::KeyManager, kv_shredding::KeyRegistry};
crates/contextra-store/tests/campaign_j42_audit.rs:5: use contextra_crypto::{AuditChain, DataClass, DeletionProofKeyPair};
crates/contextra-store/tests/campaign_j42_audit.rs:12: let keypair = DeletionProofKeyPair::generate();
crates/contextra-store/tests/campaign_j42_audit.rs:70: let _keypair = DeletionProofKeyPair::generate();
crates/contextra-store/tests/compaction_merge_operator.rs:550: use contextra_sandbox::MergeOperatorCapabilities;
crates/contextra-store/tests/kv_delete_mode_wiring_test.rs:6: use contextra_crypto::{crypto::KeyManager, kv_shredding::KeyRegistry};
```

### `contextra-types` (7)

```
crates/contextra-types/src/retrieval_strategy.rs:2: //! ACHTUNG: NICHT zu verwechseln mit `contextra_router::RoutingStrategy`
crates/contextra-types/src/types/domain/ids.rs:14: /// Im Gegensatz dazu ist `contextra_privacy::processing_registry::TenantId` ein lokaler `String`-Alias
crates/contextra-types/src/types/domain/misc.rs:162: /// Im Gegensatz dazu ist `contextra_graph::csr::types::Edge` (bzw. `CsrEdge`) ein speicherkompakter
crates/contextra-types/src/types/domain/misc.rs:355: /// Graph algorithms in `contextra-graph` map this configuration to `contextra_graph::TlHfdParams`.
crates/contextra-types/src/types/domain/misc.rs:361: /// Im Gegensatz dazu ist `contextra_graph::tl_hfd::params::TlHfdParams` die Graph-Engine-Spezifische
crates/contextra-types/src/types/domain/misc.rs:568: #[cfg(not(feature = "auto-extraction-opt-out"))]
crates/contextra-types/src/types/domain/misc.rs:572: #[cfg(feature = "auto-extraction-opt-out")]
```

### `contextra-testkit` (4)

```
crates/contextra-testkit/src/lib.rs:30: layer: impl Into<contextra_ports::scanner::WitnessLayer>,
crates/contextra-testkit/src/lib.rs:33: technique: contextra_ports::scanner::ScanTechnique,
crates/contextra-testkit/src/lib.rs:34: ) -> contextra_ports::scanner::LayerWitness {
crates/contextra-testkit/src/lib.rs:35: contextra_ports::scanner::mint_witness_for_testkit(
```

### `contextra-ports` (3)

```
crates/contextra-ports/src/scanner/mod.rs:28: /// use contextra_ports::scanner::{LayerWitness, WitnessLayer, ScanTechnique};
crates/contextra-ports/src/scanner/mod.rs:41: /// use contextra_ports::scanner::ScanSession;
crates/contextra-ports/src/scanner/mod.rs:49: /// use contextra_ports::scanner::LayerWitness;
```

### `contextra-text` (3)

```
crates/contextra-text/src/inverted/purge.rs:11: use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};
crates/contextra-text/src/inverted/scanner.rs:9: use contextra_ports::scanner::{LayerScanner, ScanSession, ScanTarget, WitnessLayer};
crates/contextra-text/tests/text_scanner_purger_tests.rs:4: use contextra_ports::scanner::{LayerScanner, ScanTarget, ScanTechnique, WitnessLayer};
```

### `contextra-rank` (2)

```
crates/contextra-rank/tests/calibration_deep_tests.rs:7: use contextra_adapt::PidController;
crates/contextra-rank/tests/calibration_stress_and_fault_tests.rs:7: use contextra_adapt::PidController;
```

