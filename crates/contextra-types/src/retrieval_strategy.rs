//! Semantische Retrieval-Strategie (Spec §9.5). Additiv, kein Breaking Change.
//! ACHTUNG: NICHT zu verwechseln mit `contextra_router::RoutingStrategy`
//! (SLM-Profil-Routing, Cascade/ContextualBandit/FlowCorrectedThompson) — dieser Typ hier
//! beschreibt, WELCHES der vier Fusionssignale (§8) ein Bandit-Arm repräsentiert.

use core::fmt;

/// Semantische Retrieval-Strategie für Signal-Fusion (§8/§9).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RetrievalStrategy {
    /// Vektorbasiertes semantisches Signal.
    Vector,
    /// Textbasiertes sparse/BM25 Signal.
    Text,
    /// Graphbasiertes Beziehungs-Signal.
    Graph,
    /// Hybrides Verbund-Signal.
    Hybrid,
    /// Nutzt LeanRAG-aggregierte Hyperkanten-Cluster statt Rohdokumente —
    /// für Fragen wie "Was sind die Hauptthemen in meiner Wissensbasis?".
    Global {
        /// Obergrenze der einbezogenen Community-Knoten.
        max_community_nodes: Option<usize>,
        /// Filtert Communities unterhalb dieser Mitgliederzahl heraus.
        min_community_size: Option<usize>,
    },
}

impl fmt::Display for RetrievalStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vector => write!(f, "Vector"),
            Self::Text => write!(f, "Text"),
            Self::Graph => write!(f, "Graph"),
            Self::Hybrid => write!(f, "Hybrid"),
            Self::Global { .. } => write!(f, "Global"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieval_strategy_display() {
        assert_eq!(RetrievalStrategy::Vector.to_string(), "Vector");
        assert_eq!(RetrievalStrategy::Text.to_string(), "Text");
        assert_eq!(RetrievalStrategy::Graph.to_string(), "Graph");
        assert_eq!(RetrievalStrategy::Hybrid.to_string(), "Hybrid");
        assert_eq!(
            RetrievalStrategy::Global {
                max_community_nodes: Some(10),
                min_community_size: Some(3),
            }
            .to_string(),
            "Global"
        );
    }

    #[test]
    fn test_retrieval_strategy_serde_roundtrip() {
        let strategies = [
            RetrievalStrategy::Vector,
            RetrievalStrategy::Text,
            RetrievalStrategy::Graph,
            RetrievalStrategy::Hybrid,
            RetrievalStrategy::Global {
                max_community_nodes: Some(20),
                min_community_size: Some(5),
            },
            RetrievalStrategy::Global {
                max_community_nodes: None,
                min_community_size: None,
            },
        ];

        for strategy in strategies {
            let json = serde_json::to_string(&strategy).unwrap_or_default();
            let deserialized: RetrievalStrategy =
                serde_json::from_str(&json).unwrap_or(RetrievalStrategy::Vector);
            assert_eq!(strategy, deserialized);
        }
    }
}
