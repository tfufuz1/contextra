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
        let k = if rrf_k.is_finite() && rrf_k >= 0.0 {
            rrf_k
        } else {
            60.0
        };
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
            rrf_k: k,
            source_collection: None,
            index_type: None,
            expected_total: None,
        }
    }

    /// Attaches vector signal distance, rank, and optional weight.
    pub fn vector(mut self, dist: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self {
        if dist.is_finite() {
            self.vector_distance = Some(dist);
            self.vector_rank = Some(rank);
            self.vector_weight = weight.into().filter(|w| w.is_finite());
        }
        self
    }

    /// Attaches BM25 text signal score, rank, and optional weight.
    pub fn bm25(mut self, score: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self {
        if score.is_finite() {
            self.bm25_score = Some(score);
            self.bm25_rank = Some(rank);
            self.text_weight = weight.into().filter(|w| w.is_finite());
        }
        self
    }

    /// Attaches graph signal score, rank, and optional weight.
    pub fn graph(mut self, score: f32, rank: u32, weight: impl Into<Option<f32>>) -> Self {
        if score.is_finite() {
            self.graph_score = Some(score);
            self.graph_rank = Some(rank);
            self.graph_weight = weight.into().filter(|w| w.is_finite());
        }
        self
    }

    /// Attaches optional cross-encoder rerank score.
    pub fn rerank_score(mut self, score: f32) -> Self {
        if score.is_finite() {
            self.rerank_score = Some(score);
        }
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
        build_provenance(&self)
    }

    /// Generates a deterministic human-readable explanation string from the built provenance record.
    pub fn explain_human_readable(&self) -> String {
        self.clone().build().explain_human_readable()
    }
}

impl ProvenanceRecord {
    /// Generates a concise, deterministic human-readable explanation from retrieval signal contributions.
    pub fn explain_human_readable(&self) -> String {
        self.explain_human_readable_with_params(None, None)
    }

    /// Generates a concise, deterministic human-readable explanation with optional fusion mode and calibrated threshold.
    pub fn explain_human_readable_with_params(
        &self,
        fusion_mode: Option<&str>,
        calibrated_threshold: Option<f32>,
    ) -> String {
        let explanation = crate::explain::explain(self);
        let base_text = explanation.to_human_readable_de();

        let mut parts = Vec::new();

        if let Some(mode) = fusion_mode {
            parts.push(format!("Fusion via {mode}"));
        }

        if let Some(thresh) = calibrated_threshold {
            parts.push(format!("Schwellenwert {:.2}", thresh));
        }

        if parts.is_empty() {
            base_text
        } else {
            let extra = parts.join(", ");
            if base_text.ends_with('.') {
                format!("{base_text} {extra}.")
            } else {
                format!("{base_text}, {extra}.")
            }
        }
    }
}

fn compute_rrf_contribution(weight: f32, rank: u32, rrf_k: f32) -> (u32, f32) {
    let rank = if rank == 0 {
        tracing::warn!("build_provenance: rank=0 is invalid input; defaulting to rank 1");
        1
    } else {
        rank
    };
    let denom = rrf_k + rank as f32;
    let rrf_contrib = if denom == 0.0 { 0.0 } else { weight / denom };
    let rrf_contrib = if rrf_contrib.is_finite() {
        rrf_contrib
    } else {
        0.0
    };
    (rank, rrf_contrib)
}

fn add_signal_entry(
    name: &str,
    raw_score: f32,
    rank: u32,
    weight: f32,
    rrf_k: f32,
    signal_ranks: &mut AHashMap<String, u32>,
    signal_contributions: &mut AHashMap<String, SignalContribution>,
) {
    let (rank_1based, rrf_contrib) = compute_rrf_contribution(weight, rank, rrf_k);
    signal_ranks.insert(name.to_string(), rank_1based);
    signal_contributions.insert(
        name.to_string(),
        SignalContribution {
            raw_score,
            rank: rank_1based,
            rrf_contribution: rrf_contrib,
        },
    );
}

fn populate_signal_provenance(
    builder: &ProvenanceBuilder,
    signal_ranks: &mut AHashMap<String, u32>,
    signal_contributions: &mut AHashMap<String, SignalContribution>,
) {
    let v_w = builder.vector_weight.unwrap_or(1.0);
    let t_w = builder.text_weight.unwrap_or(1.0);
    let g_w = builder.graph_weight.unwrap_or(1.0);

    if let (Some(score), Some(rank)) = (builder.vector_distance, builder.vector_rank) {
        add_signal_entry(
            "vector",
            score,
            rank,
            v_w,
            builder.rrf_k,
            signal_ranks,
            signal_contributions,
        );
    }
    if let (Some(score), Some(rank)) = (builder.bm25_score, builder.bm25_rank) {
        add_signal_entry(
            "text",
            score,
            rank,
            t_w,
            builder.rrf_k,
            signal_ranks,
            signal_contributions,
        );
    }
    if let (Some(score), Some(rank)) = (builder.graph_score, builder.graph_rank) {
        add_signal_entry(
            "graph",
            score,
            rank,
            g_w,
            builder.rrf_k,
            signal_ranks,
            signal_contributions,
        );
    }
}

pub(super) fn build_provenance(builder: &ProvenanceBuilder) -> ProvenanceRecord {
    debug_assert!(
        builder.rrf_k >= 0.0,
        "rrf_k must be non-negative; division by zero risk"
    );

    let mut signal_ranks = AHashMap::new();
    let mut signal_contributions = AHashMap::new();

    populate_signal_provenance(builder, &mut signal_ranks, &mut signal_contributions);

    let record = ProvenanceRecord {
        vector_distance: builder.vector_distance,
        bm25_score: builder.bm25_score,
        graph_score: builder.graph_score,
        rerank_score: builder.rerank_score,
        signal_ranks,
        source_collection: builder.source_collection.clone(),
        index_type: builder.index_type.clone(),
        signal_contributions,
        coherence_bonus: 0.0,
    };

    if let Some(expected) = builder.expected_total {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provenance_builder_explain_human_readable_deterministic() {
        let builder = ProvenanceBuilder::new(60.0)
            .vector(0.15, 1, 1.0)
            .bm25(8.5, 3, 1.0)
            .source_collection("docs");

        let text1 = builder.explain_human_readable();
        let text2 = builder.explain_human_readable();

        assert_eq!(text1, text2, "Explanation must be strictly deterministic");
        assert!(text1.contains("Vektor-Ähnlichkeit"));
        assert!(text1.contains("BM25-Textsuche"));
        assert!(text1.contains("[Kollektion: docs]"));

        let record = builder.build();
        let text_params = record.explain_human_readable_with_params(Some("rrf"), Some(0.85));
        assert!(text_params.contains("Fusion via rrf"));
        assert!(text_params.contains("Schwellenwert 0.85"));
    }
}
