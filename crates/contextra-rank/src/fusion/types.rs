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
