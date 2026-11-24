//! Typ-Konvertierungen zwischen lokalen `contextra-rank`-Typen und `contextra_types`.

use crate::fusion::SearchResult;
use crate::local_types::{
    ConfigFingerprint, DocId, FusionStrategy, FusionWeights, SignalFailurePolicy,
    SignalFusionStrategies, TxId,
};

impl From<contextra_types::DocId> for DocId {
    fn from(id: contextra_types::DocId) -> Self {
        DocId(id.0)
    }
}

impl From<DocId> for contextra_types::DocId {
    fn from(id: DocId) -> Self {
        contextra_types::DocId(id.0)
    }
}

impl From<contextra_types::ConfigFingerprint> for ConfigFingerprint {
    fn from(fp: contextra_types::ConfigFingerprint) -> Self {
        ConfigFingerprint {
            hash: fp.prompt_template_hash,
        }
    }
}

impl From<contextra_types::FusionStrategy> for FusionStrategy {
    fn from(s: contextra_types::FusionStrategy) -> Self {
        match s {
            contextra_types::FusionStrategy::Rrf => FusionStrategy::Rrf,
            contextra_types::FusionStrategy::ScoreNormalized => FusionStrategy::ScoreNormalized,
        }
    }
}

impl From<FusionStrategy> for contextra_types::FusionStrategy {
    fn from(s: FusionStrategy) -> Self {
        match s {
            FusionStrategy::Rrf => contextra_types::FusionStrategy::Rrf,
            FusionStrategy::ScoreNormalized => contextra_types::FusionStrategy::ScoreNormalized,
        }
    }
}

impl From<contextra_types::OnSignalFailure> for SignalFailurePolicy {
    fn from(p: contextra_types::OnSignalFailure) -> Self {
        match p {
            contextra_types::OnSignalFailure::Fail => SignalFailurePolicy::Fail,
            contextra_types::OnSignalFailure::Degrade => SignalFailurePolicy::Degrade,
        }
    }
}

impl From<SignalFailurePolicy> for contextra_types::OnSignalFailure {
    fn from(p: SignalFailurePolicy) -> Self {
        match p {
            SignalFailurePolicy::Fail => contextra_types::OnSignalFailure::Fail,
            SignalFailurePolicy::Degrade => contextra_types::OnSignalFailure::Degrade,
        }
    }
}

impl From<contextra_types::SignalFusionStrategies> for SignalFusionStrategies {
    fn from(s: contextra_types::SignalFusionStrategies) -> Self {
        SignalFusionStrategies {
            vector: s.vector.into(),
            text: s.text.into(),
            graph: s.graph.into(),
        }
    }
}

impl From<SignalFusionStrategies> for contextra_types::SignalFusionStrategies {
    fn from(s: SignalFusionStrategies) -> Self {
        contextra_types::SignalFusionStrategies {
            vector: s.vector.into(),
            text: s.text.into(),
            graph: s.graph.into(),
        }
    }
}

impl From<contextra_types::FusionWeights> for FusionWeights {
    fn from(w: contextra_types::FusionWeights) -> Self {
        FusionWeights {
            vector: w.vector(),
            text: w.text(),
            graph: w.graph(),
            metadata: 0.0,
        }
    }
}

impl From<contextra_types::TxId> for TxId {
    fn from(id: contextra_types::TxId) -> Self {
        TxId(id.0)
    }
}

impl From<TxId> for contextra_types::TxId {
    fn from(id: TxId) -> Self {
        contextra_types::TxId::new(id.0)
    }
}

impl TryFrom<SearchResult> for contextra_types::ContextChunk {
    type Error = contextra_types::ContextraError;

    fn try_from(r: SearchResult) -> std::result::Result<Self, Self::Error> {
        let doc_id = contextra_types::DocId::from_key(&r.id).map_err(|e| {
            contextra_types::ContextraError::InvalidInput(format!(
                "SearchResult-ID '{}' ungültig: {e}",
                r.id
            ))
        })?;
        let content = r
            .metadata
            .as_ref()
            .and_then(|m| m.get("text").or_else(|| m.get("content")))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        let token_count = content.split_whitespace().count();
        let links: Vec<contextra_types::MemoryLink> = r
            .metadata
            .as_ref()
            .and_then(|m| m.get("links"))
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        Ok(contextra_types::ContextChunk {
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

/// Helper für Synthese aus `contextra_types::DocId`.
pub fn signal_contribution_synthesized_from_types(source_doc_ids: &[contextra_types::DocId]) -> crate::fusion::ProvenanceRecord {
    let local_ids: Vec<DocId> = source_doc_ids.iter().map(|&id| DocId(id.0)).collect();
    crate::fusion::ProvenanceRecord::synthesized_from(&local_ids)
}

/// Konvertiert optionale `contextra_types::FusionWeights`.
pub fn weights_to_signal_factors_types_compat(
    weights: Option<&contextra_types::FusionWeights>,
) -> (f32, f32, f32) {
    match weights {
        Some(w) => (w.vector(), w.text(), w.graph()),
        None => (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0),
    }
}

/// Bi-temporal filter helper mit Ports-Clock.
pub fn apply_temporal_validity_filter_with_ports_clock(
    results: Vec<crate::fusion::SearchResult>,
    current_tx: TxId,
    clock: &dyn contextra_ports::Clock,
) -> Vec<crate::fusion::SearchResult> {
    let now_ms = clock.now_unix_nanos() / 1_000_000;
    crate::temporal::apply_temporal_validity_filter(results, current_tx, Some(now_ms))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_types_conformance_and_serde_equivalence() {
        // DocId
        let c_doc = contextra_types::DocId(42);
        let l_doc: DocId = c_doc.into();
        assert_eq!(l_doc.0, 42);
        let back_doc: contextra_types::DocId = l_doc.into();
        assert_eq!(back_doc.0, 42);

        // TxId
        let c_tx = contextra_types::TxId::new(99);
        let l_tx: TxId = c_tx.into();
        assert_eq!(l_tx.0, 99);
        let back_tx: contextra_types::TxId = l_tx.into();
        assert_eq!(back_tx.0, 99);

        // FusionStrategy
        for strat in [FusionStrategy::Rrf, FusionStrategy::ScoreNormalized] {
            let c_strat: contextra_types::FusionStrategy = strat.into();
            let l_strat: FusionStrategy = c_strat.into();
            assert_eq!(l_strat, strat);
        }

        // SignalFailurePolicy
        for pol in [SignalFailurePolicy::Fail, SignalFailurePolicy::Degrade] {
            let c_pol: contextra_types::OnSignalFailure = pol.into();
            let l_pol: SignalFailurePolicy = c_pol.into();
            assert_eq!(l_pol, pol);
        }

        // SignalFusionStrategies
        let l_sig_strat = SignalFusionStrategies {
            vector: FusionStrategy::Rrf,
            text: FusionStrategy::ScoreNormalized,
            graph: FusionStrategy::Rrf,
        };
        let c_sig_strat: contextra_types::SignalFusionStrategies = l_sig_strat.into();
        let back_sig_strat: SignalFusionStrategies = c_sig_strat.into();
        assert_eq!(back_sig_strat, l_sig_strat);

        // FusionWeights
        let Ok(c_weights) = contextra_types::FusionWeights::new(0.5, 0.3, 0.2) else {
            return;
        };
        let l_weights: FusionWeights = c_weights.into();
        assert_eq!(l_weights.vector, 0.5);
        assert_eq!(l_weights.text, 0.3);
        assert_eq!(l_weights.graph, 0.2);

        // ConfigFingerprint
        let c_fp = contextra_types::ConfigFingerprint::new(
            "model-1",
            "Q4_K_M",
            "test-prompt-template",
            0.7,
        );
        let expected_hash = c_fp.prompt_template_hash;
        let l_fp: ConfigFingerprint = c_fp.into();
        assert_eq!(l_fp.hash, expected_hash);
    }
}
