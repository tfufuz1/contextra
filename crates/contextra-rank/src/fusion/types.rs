use ahash::AHashMap;
use contextra_types::{ContextChunk, ContextraError, DocId, MemoryLink};
use serde::{Deserialize, Serialize};

pub(super) fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let words = text.split_whitespace().count();
    let chars = text.chars().count();
    ((words as f64 * 1.3).max(chars as f64 / 4.0)).ceil() as usize
}

impl TryFrom<SearchResult> for ContextChunk {
    type Error = ContextraError;

    fn try_from(r: SearchResult) -> std::result::Result<Self, Self::Error> {
        let doc_id = DocId::from_key(&r.id).map_err(|e| {
            ContextraError::InvalidInput(format!("SearchResult-ID '{}' ungültig: {e}", r.id))
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
    pub fn synthesized_from(source_doc_ids: &[contextra_types::DocId]) -> Self {
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

/// Fixed-point rank score representation (`FixedRankScore(pub u128)`) in Q64.64 format for bit-identical ISA-independent RRF scoring (§9.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct FixedRankScore(pub u128);

impl FixedRankScore {
    /// Fixed-point shift exponent (Q64.64: 64 fractional bits).
    pub const Q64_ONE: u128 = 1u128 << 64;

    /// Creates a new `FixedRankScore` wrapping a raw Q64.64 `u128` integer.
    pub const fn new(raw: u128) -> Self {
        Self(raw)
    }

    /// Calculates RRF fixed-point contribution for rank $r$ (1-based) with constant $k = 60$:
    /// $\text{score} = \frac{1 \ll 64}{k + r}$.
    pub fn rrf_contrib(k: u32, rank_1based: u32) -> Self {
        let denom = (k as u128).saturating_add(rank_1based as u128);
        if denom == 0 {
            Self(0)
        } else {
            Self(Self::Q64_ONE / denom)
        }
    }

    /// Calculates weighted RRF fixed-point contribution for weight $w \in [0, 1]$ and rank $r$ (1-based):
    /// $\text{score} = \frac{w \times (1 \ll 64)}{k + r}$.
    pub fn weighted_rrf_contrib(weight: f32, k: u32, rank_1based: u32) -> Self {
        if !weight.is_finite() || weight <= 0.0 {
            return Self(0);
        }
        let contrib = Self::rrf_contrib(k, rank_1based);
        let scaled = (contrib.0 as f64) * (weight as f64);
        Self(scaled as u128)
    }

    /// Converts a floating-point score in $[0, 1]$ to Q64.64 fixed point.
    pub fn from_f32(val: f32) -> Self {
        if !val.is_finite() || val <= 0.0 {
            Self(0)
        } else {
            let scaled = (val as f64) * (Self::Q64_ONE as f64);
            Self(scaled.min(u128::MAX as f64) as u128)
        }
    }

    /// Converts Q64.64 fixed point to `f32`.
    pub fn to_f32(self) -> f32 {
        ((self.0 as f64) / (Self::Q64_ONE as f64)) as f32
    }

    /// Adds another `FixedRankScore` with saturating addition.
    pub fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
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

/// Query retrieval signal identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Signal {
    /// Dense vector similarity signal.
    Vector,
    /// Full-text BM25 keyword signal.
    Text,
    /// Graph entity relationship signal.
    Graph,
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Signal::Vector => write!(f, "vector"),
            Signal::Text => write!(f, "text"),
            Signal::Graph => write!(f, "graph"),
        }
    }
}

/// Policy governing multi-signal retrieval error handling semantics.
pub use contextra_types::OnSignalFailure as SignalFailurePolicy;

/// Execution report for search query execution.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SearchReport {
    /// List of signals that failed during query execution when using Degrade policy.
    pub degraded_signals: Vec<Signal>,
    /// Warnings captured during query processing (e.g. anchor entity mismatch).
    pub warnings: Vec<String>,
    /// Number of adaptive candidate overfetch stages executed (1, 2, or 3).
    pub overfetch_stages: u8,
}
