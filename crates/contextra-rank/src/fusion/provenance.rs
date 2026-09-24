use super::types::{ProvenanceRecord, SignalContribution};
use ahash::AHashMap;

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
