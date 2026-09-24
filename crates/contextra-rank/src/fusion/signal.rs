use super::types::SignalContribution;
use ahash::AHashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum SignalKey<'a> {
    Known(SignalKind),
    Custom(&'a str),
}

impl<'a> SignalKey<'a> {
    pub(super) fn from_name(name: &'a str) -> Option<Self> {
        if name.is_empty() || name.eq_ignore_ascii_case("unnamed") {
            None
        } else if let Some(kind) = SignalKind::from_name(name) {
            Some(SignalKey::Known(kind))
        } else {
            Some(SignalKey::Custom(name))
        }
    }

    pub(super) fn as_str(&self) -> &'a str {
        match self {
            SignalKey::Known(kind) => kind.as_str(),
            SignalKey::Custom(s) => s,
        }
    }
}

#[derive(Default)]
pub(super) struct FusedEntry<'a> {
    pub(super) matched_signals: Vec<SignalKey<'a>>,
    pub(super) vector_distance: Option<f32>,
    pub(super) bm25_score: Option<f32>,
    pub(super) graph_score: Option<f32>,
    pub(super) rerank_score: Option<f32>,
    pub(super) source_collection: Option<&'a str>,
    pub(super) index_type: Option<&'a str>,
    pub(super) signal_ranks: AHashMap<SignalKey<'a>, u32>,
    pub(super) extra_signal_ranks: Option<AHashMap<String, u32>>,
    pub(super) signal_contributions: AHashMap<SignalKey<'a>, SignalContribution>,
    pub(super) extra_signal_contributions: Option<AHashMap<String, SignalContribution>>,
    pub(super) deferred_metadata: Vec<&'a Option<serde_json::Value>>,
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
