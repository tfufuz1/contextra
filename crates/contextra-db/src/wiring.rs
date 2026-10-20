//! MultiStep Wiring composition root (Rule 7 compliant).

use crate::multistep::{MultiStepConfig, MultiStepEngine, MultiStepResult, QueryRewriter};
use contextra_engine::collection::Collection;
use contextra_engine::SearchResult;
use contextra_ports::{Clock, StorageEngine, TextEmbeddingEngine};
use contextra_store::LsmStorage;
use contextra_types::Result;
use std::sync::Arc;

/// Fixed sequence of multi-step retrieval pipeline stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PipelineStage {
    Retrieval = 1,
    Rerank = 2,
    Plan = 3,
    EgressFilter = 4,
}

/// The fixed stage cascade: Retrieval -> Rerank -> Plan -> Egress-Filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineStages {
    pub stages: [PipelineStage; 4],
}

impl Default for PipelineStages {
    fn default() -> Self {
        Self {
            stages: [
                PipelineStage::Retrieval,
                PipelineStage::Rerank,
                PipelineStage::Plan,
                PipelineStage::EgressFilter,
            ],
        }
    }
}

impl PipelineStages {
    /// Returns the canonical 4-stage pipeline sequence.
    pub fn canonical() -> Self {
        Self::default()
    }

    /// Verifies that the stage sequence matches the strict order:
    /// Retrieval -> Rerank -> Plan -> EgressFilter.
    pub fn verify_order(&self) -> bool {
        self.stages
            == [
                PipelineStage::Retrieval,
                PipelineStage::Rerank,
                PipelineStage::Plan,
                PipelineStage::EgressFilter,
            ]
    }
}

/// Input dependencies required to construct [`MultiStepWiring`].
pub struct MultiStepWiringDeps<S: StorageEngine = LsmStorage> {
    pub collection: Arc<Collection<S>>,
    pub config: MultiStepConfig,
    pub clock: Option<Arc<dyn Clock>>,
    pub egress_filter: Option<Arc<dyn Fn(&SearchResult) -> bool + Send + Sync>>,
}

/// Composition Root wiring for `contextra-db` multi-step retrieval.
pub struct MultiStepWiring<S: StorageEngine + 'static = LsmStorage> {
    engine: Arc<MultiStepEngine<S>>,
    stages: PipelineStages,
    egress_filter: Option<Arc<dyn Fn(&SearchResult) -> bool + Send + Sync>>,
}

impl<S: StorageEngine + 'static> MultiStepWiring<S> {
    /// Builds `MultiStepWiring` from dependencies conforming to Rule 7.
    pub fn build(deps: MultiStepWiringDeps<S>) -> Result<Self> {
        let mut engine = MultiStepEngine::new(deps.collection, deps.config);
        if let Some(clock) = deps.clock {
            engine = engine.with_clock(clock);
        }

        Ok(Self {
            engine: Arc::new(engine),
            stages: PipelineStages::canonical(),
            egress_filter: deps.egress_filter,
        })
    }

    /// Accessor for the pipeline stages configuration.
    pub fn stages(&self) -> &PipelineStages {
        &self.stages
    }

    /// Accessor for the underlying `MultiStepEngine`.
    pub fn engine(&self) -> &Arc<MultiStepEngine<S>> {
        &self.engine
    }

    /// Executes the full multi-stage retrieval cascade:
    /// Retrieval -> Rerank -> Plan (MultiStep) -> EgressFilter.
    pub async fn execute_cascade(
        &self,
        original_query: &str,
        vector: &[f32],
        k: usize,
        rewriter: Option<&dyn QueryRewriter>,
        embedder: Option<&dyn TextEmbeddingEngine>,
    ) -> Result<MultiStepResult> {
        // Stage 1-3: Retrieval -> Rerank -> Plan
        let mut result = self
            .engine
            .search_with_embedder(original_query, vector, k, rewriter, embedder)
            .await?;

        // Stage 4: EgressFilter
        if let Some(ref filter) = self.egress_filter {
            result.results.retain(|item| filter(item));
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_stages_order_verification() {
        let stages = PipelineStages::canonical();
        assert!(stages.verify_order());
        assert_eq!(stages.stages[0], PipelineStage::Retrieval);
        assert_eq!(stages.stages[1], PipelineStage::Rerank);
        assert_eq!(stages.stages[2], PipelineStage::Plan);
        assert_eq!(stages.stages[3], PipelineStage::EgressFilter);

        let default_stages = PipelineStages::default();
        assert_eq!(default_stages, stages);
        assert!(default_stages.verify_order());

        let non_canonical = PipelineStages {
            stages: [
                PipelineStage::Rerank,
                PipelineStage::Retrieval,
                PipelineStage::Plan,
                PipelineStage::EgressFilter,
            ],
        };
        assert!(!non_canonical.verify_order());
    }
}
