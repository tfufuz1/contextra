# API: contextra-rank

> Erzeugt aus dem Quellcode am 2026-11-25, Repository HEAD ce4579a56. Eine Zeile je Item: Datei:Zeile und die **erste Zeile** der Signatur. Mehrzeilige Signaturen sind nur angeschnitten. Nicht kompiliert, daher ohne Typprüfung. Testcode (`tests`, `testing`, `test_doubles`) ist ausgeschlossen.

Umfang: 195 öffentliche Items, 25 crate-interne Sichtbarkeitsgrenzen.

## Innen: Abhängigkeiten zu anderen Workspace-Crates

- keine

## Außen: öffentliche Schnittstelle (`pub`)

```rust
src/calibration/conformal.rs:14  pub enum ConformalError
src/calibration/conformal.rs:38  pub trait ConformalCalibrator
src/calibration/conformal.rs:74  pub struct AdaptiveConformalCalibrator
src/calibration/conformal.rs:86  pub const DEFAULT_LEARNING_RATE: f32 = 0.02
src/calibration/conformal.rs:89  pub const DEFAULT_INITIAL_THRESHOLD: f32 = 0.5
src/calibration/conformal.rs:92  pub const DEFAULT_MIN_THRESHOLD: f32 = 0.0
src/calibration/conformal.rs:95  pub const DEFAULT_MAX_THRESHOLD: f32 = 1.0
src/calibration/conformal.rs:100  pub fn new(alpha: f32) -> Result<Self, ConformalError>
src/calibration/conformal.rs:109  pub fn with_params(
src/calibration/conformal.rs:136  pub fn with_bounds(
src/calibration/conformal.rs:157  pub fn alpha(&self) -> f32
src/calibration/conformal.rs:162  pub fn learning_rate(&self) -> f32
src/calibration/conformal.rs:167  pub fn observation_count(&self) -> u64
src/calibration/conformal.rs:172  pub fn invalidate_on_config_change(&mut self, new_fingerprint: impl Into<ConfigFingerprint>)
src/calibration/isotonic.rs:14  pub trait Clock: Send + Sync
src/calibration/isotonic.rs:26  pub struct SystemClock
src/calibration/isotonic.rs:30  pub fn new() -> Self
src/calibration/isotonic.rs:73  pub struct IsotonicCalibrator
src/calibration/isotonic.rs:127  pub fn new(warmup_required: u32, max_observations: usize) -> Self
src/calibration/isotonic.rs:143  pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self
src/calibration/isotonic.rs:149  pub fn with_defaults() -> Self
src/calibration/isotonic.rs:155  pub fn record_outcome(&mut self, raw_score: f32, outcome: bool)
src/calibration/isotonic.rs:172  pub fn observation_count(&self) -> usize
src/calibration/isotonic.rs:177  pub fn is_calibrated(&self) -> bool
src/calibration/isotonic.rs:182  pub fn last_calibration_at(&self) -> Option<u64>
src/calibration/isotonic.rs:187  pub fn cached_ece(&self) -> Option<f32>
src/calibration/isotonic.rs:193  pub fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32>
src/calibration/isotonic.rs:207  pub fn force_rebuild(&mut self)
src/calibration/isotonic.rs:214  pub fn invalidate_on_config_change(&mut self, new_fingerprint: impl Into<ConfigFingerprint>)
src/calibration/isotonic.rs:330  pub fn expected_calibration_error(&mut self) -> Option<f32>
src/calibration/mod.rs:7  pub mod conformal
src/calibration/mod.rs:8  pub mod isotonic
src/calibration/mod.rs:9  pub mod platt
src/calibration/mod.rs:11  pub use conformal::{AdaptiveConformalCalibrator, ConformalCalibrator, ConformalError}
src/calibration/mod.rs:12  pub use isotonic::{Clock, IsotonicCalibrator, SystemClock}
src/calibration/mod.rs:13  pub use platt::PlattScaler
src/calibration/mod.rs:20  pub enum Calibrator
src/calibration/mod.rs:49  pub fn platt(scaler: PlattScaler) -> Self
src/calibration/mod.rs:57  pub fn isotonic(calibrator: IsotonicCalibrator) -> Self
src/calibration/mod.rs:65  pub fn conformal(calibrator: AdaptiveConformalCalibrator) -> Self
src/calibration/mod.rs:78  pub fn calibrate_score(raw: f32, calibrator: &Calibrator) -> Result<f32>
src/calibration/platt.rs:13  pub struct PlattScaler
src/calibration/platt.rs:27  pub fn new(a: f32, b: f32) -> Self
src/calibration/platt.rs:39  pub fn identity() -> Self
src/calibration/platt.rs:48  pub fn is_identity(&self) -> bool
src/calibration/platt.rs:53  pub fn is_fitted(&self) -> bool
src/calibration/platt.rs:58  pub fn params(&self) -> (f32, f32)
src/calibration/platt.rs:63  pub fn predict(&self, raw_score: f32) -> f32
src/calibration/platt.rs:68  pub fn apply(&self, logit: f32) -> f32
src/calibration/platt.rs:74  pub fn transform(&self, logit: f32) -> f32
src/calibration/platt.rs:86  pub fn invalidate_on_config_change(&mut self, new_fingerprint: impl Into<ConfigFingerprint>)
src/calibration/platt.rs:103  pub fn fit(observations: &[(f32, bool)]) -> Self
src/dibud/driver.rs:17  pub fn fuse_exact_prefix<I1, I2, I3>(
src/dibud/mod.rs:8  pub mod driver
src/dibud/mod.rs:9  pub mod state
src/dibud/mod.rs:10  pub mod types
src/dibud/mod.rs:12  pub use driver::{fuse_exact_prefix, fuse_exact_prefix_async}
src/dibud/mod.rs:13  pub use state::{DiBudFusionState, DiBudStep}
src/dibud/mod.rs:14  pub use types::{BudgetedChannel, DiBudOutcome, FusionBudget}
src/dibud/state.rs:21  pub enum DiBudStep
src/dibud/state.rs:38  pub struct DiBudFusionState
src/dibud/state.rs:55  pub fn new(capacity: usize) -> Self
src/dibud/state.rs:67  pub fn channel_depths(&self) -> [usize; 3]
src/dibud/state.rs:72  pub fn exhausted(&self) -> [bool; 3]
src/dibud/state.rs:77  pub fn accesses(&self) -> usize
src/dibud/state.rs:82  pub fn provenance_of(&self, doc_id: &DocId) -> Option<&ProvenanceRecord>
src/dibud/state.rs:89  pub fn feed(
src/dibud/state.rs:127  pub fn next_request(&self, budget: &FusionBudget) -> DiBudStep
src/dibud/types.rs:18  pub enum BudgetedChannel
src/dibud/types.rs:30  pub const fn index(self) -> usize
src/dibud/types.rs:36  pub const fn from_index(index: usize) -> Option<Self>
src/dibud/types.rs:49  pub struct FusionBudget
src/dibud/types.rs:79  pub fn validate(&self) -> Result<(), RankError>
src/dibud/types.rs:118  pub struct DiBudOutcome
src/drift.rs:16  pub enum DriftStatus
src/drift.rs:35  pub fn as_str(&self) -> &'static str
src/drift.rs:59  pub struct DriftDetector
src/drift.rs:68  pub fn new(window_size: usize, threshold: f32) -> Self
src/drift.rs:83  pub fn with_defaults() -> Self
src/drift.rs:88  pub fn set_baseline(&mut self, baseline: f32)
src/drift.rs:95  pub fn observe(&mut self, score: f32) -> DriftStatus
src/drift.rs:109  pub fn analyze(&self) -> DriftStatus
src/drift.rs:134  pub fn overall_drift_status(&self) -> &'static str
src/error.rs:8  pub enum RankError
src/error.rs:23  pub type Result<T> = core::result::Result<T, RankError>
src/explain.rs:15  pub use crate::fusion::SignalKind
src/explain.rs:20  pub struct ExplanationEntry
src/explain.rs:36  pub struct RetrievalExplanation
src/explain.rs:66  pub fn to_human_readable_de(&self) -> String
src/explain.rs:207  pub fn explain(record: &ProvenanceRecord) -> RetrievalExplanation
src/fusion/global.rs:19  pub struct GlobalFusionConfig
src/fusion/global.rs:47  pub struct GlobalFusionStrategy
src/fusion/global.rs:53  pub fn new(config: GlobalFusionConfig) -> Self
src/fusion/global.rs:58  pub fn config(&self) -> &GlobalFusionConfig
src/fusion/global.rs:144  pub fn fuse(
src/fusion/mod.rs:17  pub use crate::local_types::{FusionStrategy, SignalFusionStrategies}
src/fusion/mod.rs:18  pub use global::{GlobalFusionConfig, GlobalFusionStrategy}
src/fusion/mod.rs:19  pub use normalized::{score_normalized_fusion_with_options, weights_to_signal_factors}
src/fusion/mod.rs:20  pub use provenance::ProvenanceBuilder
src/fusion/mod.rs:21  pub use resonance::{apply_resonance_bonus, fuse_signals, ResonanceConfig}
src/fusion/mod.rs:22  pub use rrf::
src/fusion/mod.rs:28  pub use signal::{MetadataMergePriority, SignalCalibrationContext, SignalKind}
src/fusion/mod.rs:29  pub use topk::BoundedTopK
src/fusion/mod.rs:30  pub use types::
src/fusion/normalized.rs:126  pub fn score_normalized_fusion_with_options(
src/fusion/normalized.rs:188  pub fn weights_to_signal_factors(
src/fusion/provenance.rs:13  pub struct ProvenanceBuilder
src/fusion/provenance.rs:32  pub fn new(rrf_k: f32) -> Self
src/fusion/provenance.rs:57  pub fn vector(mut self, dist: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self
src/fusion/provenance.rs:67  pub fn bm25(mut self, score: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self
src/fusion/provenance.rs:77  pub fn graph(mut self, score: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self
src/fusion/provenance.rs:87  pub fn rerank_score(mut self, score: f32) -> Self
src/fusion/provenance.rs:95  pub fn source_collection(mut self, collection: impl Into<String>) -> Self
src/fusion/provenance.rs:101  pub fn index_type(mut self, index_type: impl Into<String>) -> Self
src/fusion/provenance.rs:107  pub fn expected_total(mut self, total: f32) -> Self
src/fusion/provenance.rs:114  pub fn build(self) -> ProvenanceRecord
src/fusion/provenance.rs:119  pub fn explain_human_readable(&self) -> String
src/fusion/provenance.rs:126  pub fn explain_human_readable(&self) -> String
src/fusion/provenance.rs:131  pub fn explain_human_readable_with_params(
src/fusion/resonance.rs:15  pub struct ResonanceConfig
src/fusion/resonance.rs:33  pub fn fuse_signals(
src/fusion/resonance.rs:42  pub fn apply_resonance_bonus(
src/fusion/rrf.rs:17  pub use crate::local_types::FusionStrategy
src/fusion/rrf.rs:25  pub fn g(x: f32) -> f32
src/fusion/rrf.rs:41  pub fn rrf_fuse(ranked_lists: &[Vec<ScoredItem>], k: usize) -> Result<Vec<ScoredItem>>
src/fusion/rrf.rs:109  pub fn reciprocal_rank_fusion(
src/fusion/rrf.rs:158  pub fn weighted_reciprocal_rank_fusion(
src/fusion/rrf.rs:173  pub fn weighted_reciprocal_rank_fusion_with_priority(
src/fusion/rrf.rs:186  pub fn modulate_and_renormalize_weights(
src/fusion/rrf.rs:264  pub fn weighted_reciprocal_rank_fusion_with_options(
src/fusion/rrf.rs:625  pub fn weighted_reciprocal_rank_fusion_mrrf(
src/fusion/rrf.rs:845  pub fn fuse_search_results_with_signal_strategies(
src/fusion/rrf.rs:926  pub fn fuse_search_results_with_strategy(
src/fusion/signal.rs:18  pub struct SignalCalibrationContext
src/fusion/signal.rs:29  pub fn new(
src/fusion/signal.rs:44  pub fn should_use_calibrated_scores(&self) -> bool
src/fusion/signal.rs:94  pub enum SignalKind
src/fusion/signal.rs:107  pub fn from_name(name: &str) -> Option<Self>
src/fusion/signal.rs:135  pub fn as_str(&self) -> &'static str
src/fusion/signal.rs:145  pub fn name_de(&self) -> &'static str
src/fusion/signal.rs:164  pub enum MetadataMergePriority
src/fusion/signal.rs:178  pub fn signal_rank(&self, signal_name: &str) -> usize
src/fusion/topk.rs:24  pub struct BoundedTopK<T>
src/fusion/topk.rs:31  pub fn new(capacity: usize) -> Self
src/fusion/topk.rs:40  pub fn capacity(&self) -> usize
src/fusion/topk.rs:45  pub fn len(&self) -> usize
src/fusion/topk.rs:50  pub fn is_empty(&self) -> bool
src/fusion/topk.rs:55  pub fn push(&mut self, item: T)
src/fusion/topk.rs:70  pub fn into_sorted_vec(self) -> Vec<T>
src/fusion/types.rs:9  pub use crate::local_types::SignalFailurePolicy
src/fusion/types.rs:13  pub fn map_is_empty<K, V, S>(m: &ahash::AHashMap<K, V, S>) -> bool
src/fusion/types.rs:20  pub struct ProvenanceRecord
src/fusion/types.rs:60  pub fn synthesized_from(source_doc_ids: &[DocId]) -> Self
src/fusion/types.rs:77  pub struct SignalContribution
src/fusion/types.rs:88  pub struct ScoredItem
src/fusion/types.rs:100  pub fn new(id: impl Into<String>, score: f32) -> Self
src/fusion/types.rs:109  pub fn with_provenance(mut self, provenance: ProvenanceRecord) -> Self
src/fusion/types.rs:140  pub struct SearchResult
src/fusion/types.rs:157  pub type FusedScore = SearchResult
src/fusion/types.rs:166  pub enum Signal
src/fusion/types.rs:188  pub struct SearchReport
src/lib.rs:32  pub mod calibration
src/lib.rs:33  pub mod drift
src/lib.rs:34  pub mod error
src/lib.rs:35  pub mod explain
src/lib.rs:36  pub mod fusion
src/lib.rs:37  pub mod local_types
src/lib.rs:38  pub mod metrics
src/lib.rs:40  pub use error::{RankError, Result}
src/lib.rs:41  pub use local_types::
src/lib.rs:47  pub mod dibud
src/lib.rs:49  pub use calibration::
src/lib.rs:54  pub use dibud::
src/lib.rs:58  pub use drift::{DriftDetector, DriftStatus}
src/lib.rs:59  pub use explain::{explain, ExplanationEntry, RetrievalExplanation}
src/lib.rs:60  pub use fusion::
src/lib.rs:69  pub use metrics::{mean_reciprocal_rank, ndcg_at_k, precision_at_k, recall_at_k, reciprocal_rank}
src/local_types.rs:6  pub const MAX_SEARCH_K: usize = 10_000
src/local_types.rs:10  pub struct DocId(pub u64)
src/local_types.rs:14  pub fn from_key(key: &str) -> Result<Self, String>
src/local_types.rs:54  pub struct ConfigFingerprint
src/local_types.rs:61  pub const ZERO: Self = ConfigFingerprint { hash: [0u8; 32] }
src/local_types.rs:64  pub fn new(
src/local_types.rs:86  pub const fn from_bytes(bytes: [u8; 32]) -> Self
src/local_types.rs:91  pub fn as_bytes(&self) -> &[u8; 32]
src/local_types.rs:99  pub enum FusionStrategy
src/local_types.rs:110  pub enum SignalFailurePolicy
src/local_types.rs:120  pub struct SignalFusionStrategies
src/local_types.rs:131  pub fn strategy_for_signal(&self, signal_name: &str) -> FusionStrategy
src/local_types.rs:153  pub struct FusionWeights
src/metrics.rs:22  pub fn reciprocal_rank<'a, I>(ranked_ids: &[I], relevant_ids: &AHashSet<&str>) -> f32
src/metrics.rs:46  pub fn mean_reciprocal_rank<'a, I>(queries: &[(&[I], &AHashSet<&str>)]) -> f32
src/metrics.rs:71  pub fn precision_at_k<'a, I>(ranked_ids: &[I], relevant_ids: &AHashSet<&str>, k: usize) -> f32
src/metrics.rs:105  pub fn recall_at_k<'a, I>(ranked_ids: &[I], relevant_ids: &AHashSet<&str>, k: usize) -> f32
src/metrics.rs:180  pub fn ndcg_at_k<'a, I>(ranked_ids: &[I], relevance_scores: &AHashMap<&str, f32>, k: usize) -> f32
```

## Innen: crate-interne Grenzen (`pub(crate)`, `pub(super)`, `pub(in …)`)

```rust
src/fusion/provenance.rs:244  pub(super) fn build_provenance(builder: &ProvenanceBuilder) -> ProvenanceRecord
src/fusion/rrf.rs:120  pub(super) fn merge_metadata_ref(
src/fusion/rrf.rs:370  pub(super) fn update_fused_entry_from_doc<'a>(
src/fusion/rrf.rs:494  pub(super) fn format_fused_results_from_top_k(
src/fusion/signal.rs:50  pub(super) enum SignalKey<'a>
src/fusion/signal.rs:56  pub(super) fn from_name(name: &'a str) -> Option<Self>
src/fusion/signal.rs:66  pub(super) fn as_str(&self) -> &'a str
src/fusion/signal.rs:75  pub(super) struct FusedEntry<'a>
src/fusion/signal.rs:76  pub(super) matched_signals: Vec<SignalKey<'a>>,
src/fusion/signal.rs:77  pub(super) vector_distance: Option<f32>,
src/fusion/signal.rs:78  pub(super) bm25_score: Option<f32>,
src/fusion/signal.rs:79  pub(super) graph_score: Option<f32>,
src/fusion/signal.rs:80  pub(super) rerank_score: Option<f32>,
src/fusion/signal.rs:81  pub(super) source_collection: Option<&'a str>,
src/fusion/signal.rs:82  pub(super) index_type: Option<&'a str>,
src/fusion/signal.rs:83  pub(super) signal_ranks: AHashMap<SignalKey<'a>, u32>,
src/fusion/signal.rs:84  pub(super) extra_signal_ranks: Option<AHashMap<String, u32>>,
src/fusion/signal.rs:85  pub(super) signal_contributions: AHashMap<SignalKey<'a>, SignalContribution>,
src/fusion/signal.rs:86  pub(super) extra_signal_contributions: Option<AHashMap<String, SignalContribution>>,
src/fusion/signal.rs:87  pub(super) deferred_metadata: Vec<&'a Option<serde_json::Value>>,
src/fusion/topk.rs:8  pub(super) fn cmp_scores(a: f32, b: f32) -> std::cmp::Ordering
src/fusion/topk.rs:75  pub(super) struct TopKCandidate<'a>
src/fusion/topk.rs:76  pub(super) idx: u32,
src/fusion/topk.rs:77  pub(super) score: f32,
src/fusion/topk.rs:78  pub(super) id: &'a str,
```
