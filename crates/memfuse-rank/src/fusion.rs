//! Hybrid Search Signal Fusion implementations (Reciprocal Rank Fusion & Score Normalization).

use ahash::AHashMap;
pub use memfuse_types::FusionStrategy;
use memfuse_types::{ContextChunk, DocId, MemFuseError, MemoryLink};
use serde::{Deserialize, Serialize};

fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let words = text.split_whitespace().count();
    let chars = text.chars().count();
    ((words as f64 * 1.3).max(chars as f64 / 4.0)).ceil() as usize
}

impl TryFrom<SearchResult> for ContextChunk {
    type Error = MemFuseError;

    fn try_from(r: SearchResult) -> std::result::Result<Self, Self::Error> {
        let doc_id = DocId::from_key(&r.id).map_err(|e| {
            MemFuseError::InvalidInput(format!("SearchResult-ID '{}' ungültig: {e}", r.id))
        })?;
        let content = r
            .metadata
            .as_ref()
            .and_then(|m| m.get("text").or_else(|| m.get("content")))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        let token_count = estimate_tokens(&content);
        let links: Vec<MemoryLink> = r
            .metadata
            .as_ref()
            .and_then(|m| m.get("links"))
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        Ok(ContextChunk {
            doc_id,
            content,
            relevance: r.score,
            token_count,
            metadata: r.metadata,
            contextual_prefix: None,
            links,
        })
    }
}

/// Provenance record for fused search result.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ProvenanceRecord {
    /// Distance score from vector search signal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector_distance: Option<f32>,

    /// BM25 score from text search signal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bm25_score: Option<f32>,

    /// Score from graph search signal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_score: Option<f32>,

    /// Rerank score.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rerank_score: Option<f32>,

    /// Per-signal rank map.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub signal_ranks: AHashMap<String, u32>,

    /// Source collection name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_collection: Option<String>,

    /// Underlying index type name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_type: Option<String>,

    /// Per-signal contribution details.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub signal_contributions: AHashMap<String, SignalContribution>,

    /// Resonance coherence bonus value.
    #[serde(default)]
    pub coherence_bonus: f32,
}

impl ProvenanceRecord {
    /// Constructs a `ProvenanceRecord` synthesized from source document IDs.
    pub fn synthesized_from(source_doc_ids: &[memfuse_types::DocId]) -> Self {
        let mut signal_ranks = AHashMap::new();
        for (idx, id) in source_doc_ids.iter().enumerate() {
            signal_ranks.insert(id.0.to_string(), (idx + 1) as u32);
        }
        ProvenanceRecord {
            index_type: Some("consolidated".to_string()),
            signal_ranks,
            ..Default::default()
        }
    }
}

/// Signal contribution details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalContribution {
    /// Raw score before normalization/fusion.
    pub raw_score: f32,
    /// 1-based rank in the signal's result list.
    pub rank: u32,
    /// Calculated RRF or normalized contribution.
    pub rrf_contribution: f32,
}

/// Individual search result candidate for fusion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    /// Document or memory entry identifier.
    pub id: String,
    /// Fused score value.
    pub score: f32,
    /// Document metadata payload.
    pub metadata: Option<serde_json::Value>,
    /// List of matched signal names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matched_signals: Vec<String>,
    /// Optional provenance record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<ProvenanceRecord>,
}

/// Type alias for fused score output item.
pub type FusedScore = SearchResult;

/// Configuration for Resonance Coherence Bonus.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResonanceConfig {
    /// Exponent β for coherence bonus. Default: 0.5.
    pub beta: f32,
    /// Boost strength γ. Default: 0.3.
    pub gamma: f32,
}

impl Default for ResonanceConfig {
    fn default() -> Self {
        Self {
            beta: 0.5,
            gamma: 0.3,
        }
    }
}

/// Convenience API to fuse multi-signal search results.
pub fn fuse_signals(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
) -> Vec<FusedScore> {
    weighted_reciprocal_rank_fusion(result_sets, max_results)
}

/// Applies Resonance Coherence Bonus to fused search results.
pub fn apply_resonance_bonus(
    results: Vec<SearchResult>,
    valid_signal_count: usize,
    config: &ResonanceConfig,
) -> Vec<SearchResult> {
    if !config.beta.is_finite() || !config.gamma.is_finite() {
        tracing::warn!(
            beta = config.beta,
            gamma = config.gamma,
            "ResonanceConfig contains non-finite value: beta={}, gamma={}; skipping resonance bonus",
            config.beta,
            config.gamma
        );
        return results;
    }
    if valid_signal_count == 0 {
        return results;
    }
    let beta = config.beta.clamp(0.1, 2.0);
    let gamma = config.gamma.clamp(0.0, 1.0);
    let mut results: Vec<_> = results
        .into_iter()
        .map(|mut r| {
            if !r.score.is_finite() {
                tracing::error!(
                    doc_id = %r.id,
                    raw_score = r.score,
                    "apply_resonance_bonus: non-finite score entering resonance bonus stage"
                );
            }
            let signal_count = r.matched_signals.len();
            let coherence = (signal_count as f32 / valid_signal_count as f32).powf(beta);
            let bonus = gamma * coherence;
            r.score *= 1.0 + bonus;
            if let Some(ref mut prov) = r.provenance {
                prov.coherence_bonus = bonus;
            }
            r
        })
        .collect();

    results.sort_by(|a, b| match (a.score.is_finite(), b.score.is_finite()) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => b.score.total_cmp(&a.score).then_with(|| a.id.cmp(&b.id)),
    });

    results
}

fn cmp_scores(a: f32, b: f32) -> std::cmp::Ordering {
    match (a.is_nan(), b.is_nan()) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.total_cmp(&b),
    }
}

/// Size-bounded min-heap for Top-K candidate selection during Reciprocal Rank Fusion.
pub struct BoundedTopK<T> {
    heap: std::collections::BinaryHeap<T>,
    capacity: usize,
}

impl<T: Ord> BoundedTopK<T> {
    /// Creates a new `BoundedTopK` container with maximum capacity `k`.
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.min(memfuse_types::MAX_SEARCH_K);
        Self {
            heap: std::collections::BinaryHeap::with_capacity(capacity.saturating_add(1)),
            capacity,
        }
    }

    /// Returns maximum allowed capacity `k`.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns current number of items held in the container.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Returns `true` if the container holds zero items.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Pushes an item into top-K container, evicting the worst candidate if size exceeds capacity `k`.
    pub fn push(&mut self, item: T) {
        if self.capacity == 0 {
            return;
        }
        if self.heap.len() < self.capacity {
            self.heap.push(item);
        } else if let Some(worst) = self.heap.peek() {
            if item < *worst {
                self.heap.pop();
                self.heap.push(item);
            }
        }
    }

    /// Consumes the container and returns items sorted from best to worst.
    pub fn into_sorted_vec(self) -> Vec<T> {
        self.heap.into_sorted_vec()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SignalKey<'a> {
    Known(SignalKind),
    Custom(&'a str),
}

impl<'a> SignalKey<'a> {
    fn from_name(name: &'a str) -> Option<Self> {
        if name.is_empty() || name.eq_ignore_ascii_case("unnamed") {
            None
        } else if let Some(kind) = SignalKind::from_name(name) {
            Some(SignalKey::Known(kind))
        } else {
            Some(SignalKey::Custom(name))
        }
    }

    fn as_str(&self) -> &'a str {
        match self {
            SignalKey::Known(kind) => kind.as_str(),
            SignalKey::Custom(s) => s,
        }
    }
}

#[derive(Default)]
struct FusedEntry<'a> {
    matched_signals: Vec<SignalKey<'a>>,
    vector_distance: Option<f32>,
    bm25_score: Option<f32>,
    graph_score: Option<f32>,
    rerank_score: Option<f32>,
    source_collection: Option<&'a str>,
    index_type: Option<&'a str>,
    signal_ranks: AHashMap<SignalKey<'a>, u32>,
    extra_signal_ranks: Option<AHashMap<String, u32>>,
    signal_contributions: AHashMap<SignalKey<'a>, SignalContribution>,
    extra_signal_contributions: Option<AHashMap<String, SignalContribution>>,
    deferred_metadata: Vec<&'a Option<serde_json::Value>>,
}

/// Identifies the kind of search signal used during fusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalKind {
    /// Vector (semantic k-NN) search signal.
    Vector,
    /// Text (BM25 keyword) search signal.
    Text,
    /// Graph (traversal / PageRank) search signal.
    Graph,
    /// Edge-reinforcement weight signal / Bandit recency signal.
    EdgeReinforcement,
}

impl SignalKind {
    /// Identifies `SignalKind` from a signal name string.
    pub fn from_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("vector") || name.eq_ignore_ascii_case("vec") {
            return Some(SignalKind::Vector);
        }
        if name.eq_ignore_ascii_case("text")
            || name.eq_ignore_ascii_case("bm25")
            || name.eq_ignore_ascii_case("keyword")
        {
            return Some(SignalKind::Text);
        }
        if name.eq_ignore_ascii_case("graph") {
            return Some(SignalKind::Graph);
        }
        if name.eq_ignore_ascii_case("edge-reinforcement")
            || name.eq_ignore_ascii_case("cooccurrence")
            || name.eq_ignore_ascii_case("traversal-reinforcement")
            || name.eq_ignore_ascii_case("synaptic")
            || name.eq_ignore_ascii_case("hebbian")
            || name.eq_ignore_ascii_case("recency")
            || name.eq_ignore_ascii_case("bandit")
        {
            return Some(SignalKind::EdgeReinforcement);
        }
        None
    }

    /// Converts `SignalKind` to its canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalKind::Vector => "vector",
            SignalKind::Text => "text",
            SignalKind::Graph => "graph",
            SignalKind::EdgeReinforcement => "edge-reinforcement",
        }
    }
}

/// Configures signal priority order for metadata merging during Reciprocal Rank Fusion.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MetadataMergePriority {
    /// Vector metadata is processed first (default behavior). Order: Vector, Text, Graph.
    #[default]
    VectorFirst,
    /// Text metadata is processed first. Order: Text, Vector, Graph.
    TextFirst,
    /// Graph metadata is processed first. Order: Graph, Vector, Text.
    GraphFirst,
    /// Custom signal priority order.
    Custom(Vec<SignalKind>),
}

impl MetadataMergePriority {
    /// Returns the precedence rank for a given signal name.
    pub fn signal_rank(&self, signal_name: &str) -> usize {
        let kind = SignalKind::from_name(signal_name);
        let order: &[SignalKind] = match self {
            MetadataMergePriority::VectorFirst => {
                &[SignalKind::Vector, SignalKind::Text, SignalKind::Graph]
            }
            MetadataMergePriority::TextFirst => {
                &[SignalKind::Text, SignalKind::Vector, SignalKind::Graph]
            }
            MetadataMergePriority::GraphFirst => {
                &[SignalKind::Graph, SignalKind::Vector, SignalKind::Text]
            }
            MetadataMergePriority::Custom(custom_order) => custom_order.as_slice(),
        };

        if let Some(k) = kind {
            if let Some(pos) = order.iter().position(|&x| x == k) {
                return pos;
            }
        }
        usize::MAX
    }
}

/// Fluent builder for constructing `ProvenanceRecord` instances.
#[derive(Debug, Clone, PartialEq)]
pub struct ProvenanceBuilder {
    vector_distance: Option<f32>,
    vector_rank: Option<u32>,
    vector_weight: Option<f32>,
    bm25_score: Option<f32>,
    bm25_rank: Option<u32>,
    text_weight: Option<f32>,
    graph_score: Option<f32>,
    graph_rank: Option<u32>,
    graph_weight: Option<f32>,
    rerank_score: Option<f32>,
    rrf_k: f32,
    source_collection: Option<String>,
    index_type: Option<String>,
    expected_total: Option<f32>,
}

impl ProvenanceBuilder {
    /// Creates a new `ProvenanceBuilder` with the specified RRF parameter `k`.
    pub fn new(rrf_k: f32) -> Self {
        Self {
            vector_distance: None,
            vector_rank: None,
            vector_weight: None,
            bm25_score: None,
            bm25_rank: None,
            text_weight: None,
            graph_score: None,
            graph_rank: None,
            graph_weight: None,
            rerank_score: None,
            rrf_k,
            source_collection: None,
            index_type: None,
            expected_total: None,
        }
    }

    /// Attaches vector signal distance, rank, and optional weight.
    pub fn vector(mut self, dist: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self {
        self.vector_distance = Some(dist);
        self.vector_rank = Some(rank);
        self.vector_weight = weight.into();
        self
    }

    /// Attaches BM25 text signal score, rank, and optional weight.
    pub fn bm25(mut self, score: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self {
        self.bm25_score = Some(score);
        self.bm25_rank = Some(rank);
        self.text_weight = weight.into();
        self
    }

    /// Attaches graph signal score, rank, and optional weight.
    pub fn graph(mut self, score: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self {
        self.graph_score = Some(score);
        self.graph_rank = Some(rank);
        self.graph_weight = weight.into();
        self
    }

    /// Attaches optional cross-encoder rerank score.
    pub fn rerank_score(mut self, score: f32) -> Self {
        self.rerank_score = Some(score);
        self
    }

    /// Attaches source collection name.
    pub fn source_collection(mut self, collection: impl Into<String>) -> Self {
        self.source_collection = Some(collection.into());
        self
    }

    /// Attaches index type string.
    pub fn index_type(mut self, index_type: impl Into<String>) -> Self {
        self.index_type = Some(index_type.into());
        self
    }

    /// Sets expected ground truth RRF score for debug invariant checking.
    pub fn expected_total(mut self, total: f32) -> Self {
        self.expected_total = Some(total);
        self
    }

    /// Builds the resulting `ProvenanceRecord`.
    pub fn build(self) -> ProvenanceRecord {
        build_provenance(
            self.vector_distance,
            self.vector_rank,
            self.vector_weight,
            self.bm25_score,
            self.bm25_rank,
            self.text_weight,
            self.graph_score,
            self.graph_rank,
            self.graph_weight,
            self.rerank_score,
            self.rrf_k,
            self.source_collection,
            self.index_type,
            self.expected_total,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_provenance(
    vector_distance: Option<f32>,
    vector_rank: Option<u32>,
    vector_weight: Option<f32>,
    bm25_score: Option<f32>,
    bm25_rank: Option<u32>,
    text_weight: Option<f32>,
    graph_score: Option<f32>,
    graph_rank: Option<u32>,
    graph_weight: Option<f32>,
    rerank_score: Option<f32>,
    rrf_k: f32,
    source_collection: Option<String>,
    index_type: Option<String>,
    expected_total: Option<f32>,
) -> ProvenanceRecord {
    debug_assert!(
        rrf_k >= 0.0,
        "rrf_k must be non-negative; division by zero risk"
    );

    let mut signal_ranks = AHashMap::new();
    let mut signal_contributions = AHashMap::new();

    let v_w = vector_weight.unwrap_or(1.0);
    let t_w = text_weight.unwrap_or(1.0);
    let g_w = graph_weight.unwrap_or(1.0);

    let calc_contrib = |w: f32, rank: u32| -> (u32, f32) {
        let rank = if rank == 0 {
            tracing::warn!("build_provenance: rank=0 is invalid input; defaulting to rank 1");
            1
        } else {
            rank
        };
        let rrf_contrib = if rrf_k + rank as f32 == 0.0 {
            0.0
        } else {
            w / (rrf_k + rank as f32)
        };
        let rrf_contrib = if rrf_contrib.is_finite() {
            rrf_contrib
        } else {
            0.0
        };
        (rank, rrf_contrib)
    };

    if let (Some(score), Some(rank)) = (vector_distance, vector_rank) {
        let (rank_1based, rrf_contrib) = calc_contrib(v_w, rank);
        signal_ranks.insert("vector".to_string(), rank_1based);
        signal_contributions.insert(
            "vector".to_string(),
            SignalContribution {
                raw_score: score,
                rank: rank_1based,
                rrf_contribution: rrf_contrib,
            },
        );
    }

    if let (Some(score), Some(rank)) = (bm25_score, bm25_rank) {
        let (rank_1based, rrf_contrib) = calc_contrib(t_w, rank);
        signal_ranks.insert("text".to_string(), rank_1based);
        signal_contributions.insert(
            "text".to_string(),
            SignalContribution {
                raw_score: score,
                rank: rank_1based,
                rrf_contribution: rrf_contrib,
            },
        );
    }

    if let (Some(score), Some(rank)) = (graph_score, graph_rank) {
        let (rank_1based, rrf_contrib) = calc_contrib(g_w, rank);
        signal_ranks.insert("graph".to_string(), rank_1based);
        signal_contributions.insert(
            "graph".to_string(),
            SignalContribution {
                raw_score: score,
                rank: rank_1based,
                rrf_contribution: rrf_contrib,
            },
        );
    }

    let record = ProvenanceRecord {
        vector_distance,
        bm25_score,
        graph_score,
        rerank_score,
        signal_ranks,
        source_collection,
        index_type,
        signal_contributions,
        coherence_bonus: 0.0,
    };

    if let Some(expected) = expected_total {
        let expected_rrf: f32 = record
            .signal_contributions
            .values()
            .map(|c| c.rrf_contribution)
            .sum();
        debug_assert!(
            (expected_rrf - expected).abs() < 1e-6,
            "INV-PROV-1 violation in build_provenance: expected_rrf={expected_rrf}, expected={expected}"
        );
    }

    record
}

/// Fuses multiple sets of ranked search results into a single ranked list using Reciprocal Rank Fusion (RRF).
pub fn reciprocal_rank_fusion(
    result_sets: Vec<Vec<SearchResult>>,
    max_results: usize,
) -> Vec<SearchResult> {
    let weighted_sets = result_sets
        .into_iter()
        .map(|set| ("unnamed".to_string(), set, 1.0))
        .collect();
    weighted_reciprocal_rank_fusion(weighted_sets, max_results)
}

fn merge_metadata_ref(target: &mut Option<serde_json::Value>, source: &Option<serde_json::Value>) {
    if let Some(s_val) = source {
        match target {
            Some(t_val) => {
                if let (Some(t_obj), Some(s_obj)) = (t_val.as_object_mut(), s_val.as_object()) {
                    for (k, v) in s_obj {
                        if !t_obj.contains_key(k) {
                            t_obj.insert(k.clone(), v.clone());
                        }
                    }
                } else if t_val != s_val {
                    let taken_t_val = std::mem::take(t_val);
                    let arr = if let Some(s_arr) = s_val.as_array() {
                        let mut a = vec![taken_t_val];
                        for item in s_arr {
                            if !a.contains(item) {
                                a.push(item.clone());
                            }
                        }
                        a
                    } else {
                        vec![taken_t_val, s_val.clone()]
                    };
                    *t_val = serde_json::Value::Array(arr);
                }
            }
            None => {
                *target = Some(s_val.clone());
            }
        }
    }
}

/// Weighted Reciprocal Rank Fusion with default signal metadata priority (`VectorFirst`).
pub fn weighted_reciprocal_rank_fusion(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
) -> Vec<SearchResult> {
    weighted_reciprocal_rank_fusion_with_options(
        result_sets,
        max_results,
        MetadataMergePriority::default(),
        true,
        None,
    )
}

/// Weighted Reciprocal Rank Fusion with explicit metadata merge priority.
pub fn weighted_reciprocal_rank_fusion_with_priority(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
) -> Vec<SearchResult> {
    weighted_reciprocal_rank_fusion_with_options(result_sets, max_results, priority, true, None)
}

struct TopKCandidate<'a> {
    idx: u32,
    score: f32,
    id: &'a str,
}

impl<'a> PartialEq for TopKCandidate<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl<'a> Eq for TopKCandidate<'a> {}

impl<'a> Ord for TopKCandidate<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        cmp_scores(other.score, self.score).then_with(|| self.id.cmp(other.id))
    }
}

impl<'a> PartialOrd for TopKCandidate<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Weighted Reciprocal Rank Fusion with options.
pub fn weighted_reciprocal_rank_fusion_with_options(
    mut result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
) -> Vec<SearchResult> {
    if max_results == 0 {
        return Vec::new();
    }

    result_sets.sort_by_key(|(signal_name, _, _)| priority.signal_rank(signal_name));

    let k = 60;
    let mut id_to_idx: AHashMap<&str, u32> = AHashMap::new();
    let mut id_table: Vec<&str> = Vec::new();
    let mut scores: Vec<f32> = Vec::new();
    let mut entries: Vec<FusedEntry<'_>> = Vec::new();
    let mut valid_signal_count = 0usize;

    for (signal_name, result_set, weight) in &result_sets {
        let weight = *weight;
        if !weight.is_finite() || weight <= 0.0 {
            tracing::warn!(
                signal = %signal_name,
                weight,
                "RRF fusion: non-finite or non-positive weight skipped"
            );
            continue;
        }
        valid_signal_count += 1;
        let sig_key = SignalKey::from_name(signal_name);
        let signal_kind = sig_key.and_then(|k| match k {
            SignalKey::Known(kind) => Some(kind),
            _ => None,
        });

        let rrf_k = k as f32;
        debug_assert!(
            rrf_k >= 0.0,
            "rrf_k must be non-negative; division by zero risk"
        );

        for (rank_idx, doc) in result_set.iter().enumerate() {
            if !doc.score.is_finite() {
                tracing::error!(
                    signal = %signal_name,
                    doc_id = %doc.id,
                    raw_score = doc.score,
                    "RRF fusion: non-finite raw score from upstream signal detected"
                );
            }
            let rrf_rank = (rank_idx + 1) as u32;
            let denom = rrf_k + rrf_rank as f32;
            debug_assert!(denom > 0.0, "RRF denominator must be positive");
            let score = weight / denom;
            let score = if score.is_finite() { score } else { 0.0 };

            let doc_id_str: &str = doc.id.as_str();
            let idx = match id_to_idx.get(doc_id_str) {
                Some(&i) => i as usize,
                None => {
                    let new_idx = id_table.len();
                    id_to_idx.insert(doc_id_str, new_idx as u32);
                    id_table.push(doc_id_str);
                    scores.push(0.0_f32);
                    entries.push(FusedEntry::default());
                    new_idx
                }
            };
            scores[idx] += score;
            let entry = &mut entries[idx];

            if doc.metadata.is_some() {
                entry.deferred_metadata.push(&doc.metadata);
            }

            if let Some(key) = sig_key {
                if !entry.matched_signals.contains(&key) {
                    entry.matched_signals.push(key);
                }
                entry.signal_ranks.insert(key, rrf_rank);
                entry.signal_contributions.insert(
                    key,
                    SignalContribution {
                        raw_score: doc.score,
                        rank: rrf_rank,
                        rrf_contribution: score,
                    },
                );
            }

            match signal_kind {
                Some(SignalKind::Vector) => {
                    if entry.vector_distance.is_none() {
                        entry.vector_distance = Some(doc.score);
                    }
                    if entry.index_type.is_none() {
                        entry.index_type = Some("hnsw");
                    }
                }
                Some(SignalKind::Text) => {
                    if entry.bm25_score.is_none() {
                        entry.bm25_score = Some(doc.score);
                    }
                    if entry.index_type.is_none() {
                        entry.index_type = Some("bm25");
                    }
                }
                Some(SignalKind::Graph) => {
                    if entry.graph_score.is_none() {
                        entry.graph_score = Some(doc.score);
                    }
                    if entry.index_type.is_none() {
                        entry.index_type = Some("graph");
                    }
                }
                Some(SignalKind::EdgeReinforcement) => {
                    if entry.graph_score.is_none() {
                        entry.graph_score = Some(doc.score);
                    }
                    if entry.index_type.is_none() {
                        entry.index_type = Some("edge-reinforcement");
                    }
                }
                None => {}
            }

            if let Some(ref doc_prov) = doc.provenance {
                if entry.vector_distance.is_none() {
                    entry.vector_distance = doc_prov.vector_distance;
                }
                if entry.bm25_score.is_none() {
                    entry.bm25_score = doc_prov.bm25_score;
                }
                if entry.graph_score.is_none() {
                    entry.graph_score = doc_prov.graph_score;
                }
                if entry.rerank_score.is_none() {
                    entry.rerank_score = doc_prov.rerank_score;
                }
                if entry.source_collection.is_none() {
                    entry.source_collection = doc_prov.source_collection.as_deref();
                }
                if entry.index_type.is_none() {
                    entry.index_type = doc_prov.index_type.as_deref();
                }
                for (sig, r) in &doc_prov.signal_ranks {
                    if let Some(k) = SignalKey::from_name(sig.as_str()) {
                        entry.signal_ranks.entry(k).or_insert(*r);
                    } else {
                        entry
                            .extra_signal_ranks
                            .get_or_insert_with(AHashMap::new)
                            .entry(sig.clone())
                            .or_insert(*r);
                    }
                }
                for (sig, contrib) in &doc_prov.signal_contributions {
                    if let Some(k) = SignalKey::from_name(sig.as_str()) {
                        entry
                            .signal_contributions
                            .entry(k)
                            .or_insert_with(|| contrib.clone());
                    } else {
                        entry
                            .extra_signal_contributions
                            .get_or_insert_with(AHashMap::new)
                            .entry(sig.clone())
                            .or_insert_with(|| contrib.clone());
                    }
                }
            }
        }
    }

    let mut top_k = BoundedTopK::new(max_results);
    for idx in 0..scores.len() as u32 {
        let i = idx as usize;
        top_k.push(TopKCandidate {
            idx,
            score: scores[i],
            id: id_table[i],
        });
    }

    let ranked_candidates = top_k.into_sorted_vec();
    let mut results: Vec<SearchResult> = Vec::with_capacity(ranked_candidates.len());
    for cand in ranked_candidates {
        let i = cand.idx as usize;
        let id = id_table[i].to_string();
        let score = scores[i];
        let entry = &mut entries[i];

        let mut merged_meta: Option<serde_json::Value> = None;
        for meta in entry.deferred_metadata.drain(..) {
            merge_metadata_ref(&mut merged_meta, meta);
        }

        let matched_signals = entry
            .matched_signals
            .iter()
            .map(|k| k.as_str().to_string())
            .collect();

        let mut signal_ranks: AHashMap<String, u32> = entry
            .signal_ranks
            .drain()
            .map(|(k, v)| (k.as_str().to_string(), v))
            .collect();
        if let Some(extras) = entry.extra_signal_ranks.take() {
            for (k, v) in extras {
                signal_ranks.entry(k).or_insert(v);
            }
        }

        let mut signal_contributions: AHashMap<String, SignalContribution> = entry
            .signal_contributions
            .drain()
            .map(|(k, v)| (k.as_str().to_string(), v))
            .collect();
        if let Some(extras) = entry.extra_signal_contributions.take() {
            for (k, v) in extras {
                signal_contributions.entry(k).or_insert(v);
            }
        }

        let prov = ProvenanceRecord {
            vector_distance: entry.vector_distance,
            bm25_score: entry.bm25_score,
            graph_score: entry.graph_score,
            rerank_score: entry.rerank_score,
            signal_ranks,
            source_collection: entry.source_collection.map(|s| s.to_string()),
            index_type: entry.index_type.map(|s| s.to_string()),
            signal_contributions,
            coherence_bonus: 0.0,
        };

        let provenance = if include_provenance
            && (prov.vector_distance.is_some()
                || prov.bm25_score.is_some()
                || prov.graph_score.is_some()
                || prov.rerank_score.is_some()
                || !prov.signal_ranks.is_empty()
                || prov.source_collection.is_some()
                || prov.index_type.is_some())
        {
            let final_prov = if prov.signal_contributions.is_empty() {
                let mut builder = ProvenanceBuilder::new(k as f32);
                if let (Some(dist), Some(rank)) = (
                    prov.vector_distance,
                    prov.signal_ranks.get("vector").copied(),
                ) {
                    builder = builder.vector(dist, rank, None);
                }
                if let (Some(score), Some(rank)) = (
                    prov.bm25_score,
                    prov.signal_ranks
                        .get("text")
                        .copied()
                        .or_else(|| prov.signal_ranks.get("bm25").copied()),
                ) {
                    builder = builder.bm25(score, rank, None);
                }
                if let (Some(score), Some(rank)) =
                    (prov.graph_score, prov.signal_ranks.get("graph").copied())
                {
                    builder = builder.graph(score, rank, None);
                }
                if let Some(score) = prov.rerank_score {
                    builder = builder.rerank_score(score);
                }
                if let Some(col) = prov.source_collection {
                    builder = builder.source_collection(col);
                }
                if let Some(idx) = prov.index_type {
                    builder = builder.index_type(idx);
                }
                builder.build()
            } else {
                prov
            };

            Some(final_prov)
        } else {
            None
        };

        results.push(SearchResult {
            id,
            score,
            metadata: merged_meta,
            matched_signals,
            provenance,
        });
    }

    let results = if let Some(cfg) = resonance_config {
        apply_resonance_bonus(results, valid_signal_count, cfg)
    } else {
        results
    };

    results
}

/// Fuses search result sets using the specified `FusionStrategy`.
pub fn fuse_search_results_with_strategy(
    result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
    strategy: FusionStrategy,
) -> Vec<SearchResult> {
    match strategy {
        FusionStrategy::Rrf => weighted_reciprocal_rank_fusion_with_options(
            result_sets,
            max_results,
            priority,
            include_provenance,
            resonance_config,
        ),
        FusionStrategy::ScoreNormalized => score_normalized_fusion_with_options(
            result_sets,
            max_results,
            priority,
            include_provenance,
            resonance_config,
        ),
    }
}

/// Score-normalized fusion (CombSUM with MinMax normalization).
pub fn score_normalized_fusion_with_options(
    mut result_sets: Vec<(String, Vec<SearchResult>, f32)>,
    max_results: usize,
    priority: MetadataMergePriority,
    include_provenance: bool,
    resonance_config: Option<&ResonanceConfig>,
) -> Vec<SearchResult> {
    if max_results == 0 {
        return Vec::new();
    }

    let active_sets: Vec<&(String, Vec<SearchResult>, f32)> = result_sets
        .iter()
        .filter(|(_, _, w)| w.is_finite() && *w > 0.0)
        .collect();

    if active_sets.is_empty() {
        return Vec::new();
    }

    let mut degraded = false;
    let total_active_signals = active_sets.len();

    for (sig_name, set, _) in &active_sets {
        if total_active_signals > 1 && set.is_empty() {
            tracing::warn!(
                signal = %sig_name,
                strategy = "ScoreNormalized",
                fallback = "RRF",
                "Signal degradation detected: active signal returned 0 candidates; falling back to RRF"
            );
            degraded = true;
            break;
        }

        if !set.is_empty() {
            let mut min_s = f32::INFINITY;
            let mut max_s = f32::NEG_INFINITY;
            let mut non_finite = false;

            for doc in set.iter() {
                if !doc.score.is_finite() {
                    non_finite = true;
                    break;
                }
                min_s = min_s.min(doc.score);
                max_s = max_s.max(doc.score);
            }

            if non_finite || (set.len() > 1 && max_s <= min_s) {
                tracing::warn!(
                    signal = %sig_name,
                    min_s,
                    max_s,
                    non_finite,
                    strategy = "ScoreNormalized",
                    fallback = "RRF",
                    "Signal degradation detected: degenerate score distribution; falling back to RRF"
                );
                degraded = true;
                break;
            }
        }
    }

    if degraded {
        return weighted_reciprocal_rank_fusion_with_options(
            result_sets,
            max_results,
            priority,
            include_provenance,
            resonance_config,
        );
    }

    result_sets.sort_by_key(|(signal_name, _, _)| priority.signal_rank(signal_name));

    let mut id_to_idx: AHashMap<&str, u32> = AHashMap::new();
    let mut id_table: Vec<&str> = Vec::new();
    let mut scores: Vec<f32> = Vec::new();
    let mut entries: Vec<FusedEntry<'_>> = Vec::new();
    let mut valid_signal_count = 0usize;

    for (signal_name, result_set, weight) in &result_sets {
        let weight = *weight;
        if !weight.is_finite() || weight <= 0.0 || result_set.is_empty() {
            continue;
        }
        valid_signal_count += 1;
        let sig_key = SignalKey::from_name(signal_name);
        let signal_kind = sig_key.and_then(|k| match k {
            SignalKey::Known(kind) => Some(kind),
            _ => None,
        });

        let mut min_s = f32::INFINITY;
        let mut max_s = f32::NEG_INFINITY;
        for doc in result_set {
            min_s = min_s.min(doc.score);
            max_s = max_s.max(doc.score);
        }
        let range = max_s - min_s;

        for (rank_idx, doc) in result_set.iter().enumerate() {
            let rank = (rank_idx + 1) as u32;
            let norm_score = if range > 0.0 && doc.score.is_finite() {
                ((doc.score - min_s) / range).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let weighted_score = weight * norm_score;

            let doc_id_str: &str = doc.id.as_str();
            let idx = match id_to_idx.get(doc_id_str) {
                Some(&i) => i as usize,
                None => {
                    let new_idx = id_table.len();
                    id_to_idx.insert(doc_id_str, new_idx as u32);
                    id_table.push(doc_id_str);
                    scores.push(0.0_f32);
                    entries.push(FusedEntry::default());
                    new_idx
                }
            };
            scores[idx] += weighted_score;
            let entry = &mut entries[idx];

            if doc.metadata.is_some() {
                entry.deferred_metadata.push(&doc.metadata);
            }

            if let Some(key) = sig_key {
                if !entry.matched_signals.contains(&key) {
                    entry.matched_signals.push(key);
                }
                entry.signal_ranks.insert(key, rank);
                entry.signal_contributions.insert(
                    key,
                    SignalContribution {
                        raw_score: doc.score,
                        rank,
                        rrf_contribution: weighted_score,
                    },
                );
            }

            match signal_kind {
                Some(SignalKind::Vector) => {
                    if entry.vector_distance.is_none() {
                        entry.vector_distance = Some(doc.score);
                    }
                    if entry.index_type.is_none() {
                        entry.index_type = Some("hnsw");
                    }
                }
                Some(SignalKind::Text) => {
                    if entry.bm25_score.is_none() {
                        entry.bm25_score = Some(doc.score);
                    }
                    if entry.index_type.is_none() {
                        entry.index_type = Some("bm25");
                    }
                }
                Some(SignalKind::Graph) => {
                    if entry.graph_score.is_none() {
                        entry.graph_score = Some(doc.score);
                    }
                    if entry.index_type.is_none() {
                        entry.index_type = Some("graph");
                    }
                }
                Some(SignalKind::EdgeReinforcement) => {
                    if entry.graph_score.is_none() {
                        entry.graph_score = Some(doc.score);
                    }
                    if entry.index_type.is_none() {
                        entry.index_type = Some("edge-reinforcement");
                    }
                }
                None => {}
            }
        }
    }

    let mut top_k = BoundedTopK::new(max_results);
    for idx in 0..scores.len() as u32 {
        let i = idx as usize;
        top_k.push(TopKCandidate {
            idx,
            score: scores[i],
            id: id_table[i],
        });
    }

    let ranked_candidates = top_k.into_sorted_vec();
    let mut results: Vec<SearchResult> = Vec::with_capacity(ranked_candidates.len());

    for cand in ranked_candidates {
        let i = cand.idx as usize;
        let id = id_table[i].to_string();
        let score = scores[i];
        let entry = &mut entries[i];

        let mut merged_meta: Option<serde_json::Value> = None;
        for meta in entry.deferred_metadata.drain(..) {
            merge_metadata_ref(&mut merged_meta, meta);
        }

        let matched_signals = entry
            .matched_signals
            .iter()
            .map(|k| k.as_str().to_string())
            .collect();

        let mut signal_ranks: AHashMap<String, u32> = entry
            .signal_ranks
            .drain()
            .map(|(k, v)| (k.as_str().to_string(), v))
            .collect();
        if let Some(extras) = entry.extra_signal_ranks.take() {
            for (k, v) in extras {
                signal_ranks.entry(k).or_insert(v);
            }
        }

        let mut signal_contributions: AHashMap<String, SignalContribution> = entry
            .signal_contributions
            .drain()
            .map(|(k, v)| (k.as_str().to_string(), v))
            .collect();
        if let Some(extras) = entry.extra_signal_contributions.take() {
            for (k, v) in extras {
                signal_contributions.entry(k).or_insert(v);
            }
        }

        let prov = ProvenanceRecord {
            vector_distance: entry.vector_distance,
            bm25_score: entry.bm25_score,
            graph_score: entry.graph_score,
            rerank_score: entry.rerank_score,
            signal_ranks,
            source_collection: entry.source_collection.map(|s| s.to_string()),
            index_type: entry.index_type.map(|s| s.to_string()),
            signal_contributions,
            coherence_bonus: 0.0,
        };

        let provenance = if include_provenance { Some(prov) } else { None };

        results.push(SearchResult {
            id,
            score,
            metadata: merged_meta,
            matched_signals,
            provenance,
        });
    }

    let results = if let Some(cfg) = resonance_config {
        apply_resonance_bonus(results, valid_signal_count, cfg)
    } else {
        results
    };

    results
}

/// Converts optional `FusionWeights` into (vector, text, graph) weight tuple.
pub fn weights_to_signal_factors(
    weights: Option<&memfuse_types::FusionWeights>,
) -> (f32, f32, f32) {
    match weights {
        Some(w) => (w.vector(), w.text(), w.graph()),
        None => (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0),
    }
}
