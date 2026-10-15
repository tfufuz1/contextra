//! Cognition Wiring composition root (Rule 7 compliant).

use crate::aggregation_phase::{AggregationConfig, ConsolidationPipelineResult};
use crate::consolidation_executor::ConsolidationEngine;
use crate::maintenance_config::MaintenanceConfig;
use crate::maintenance_scheduler::MaintenanceScheduler;
use crate::memory_consolidation::{ConsolidationConfig, SynthesisConfig};
use contextra_engine::collection::Collection;
use contextra_ports::{
    Clock, GroundingValidator, LlmTextGenerator, Rng, StorageEngine, SystemClock, SystemRng,
    VectorIndex,
};
use contextra_store::LsmStorage;
use contextra_types::Result;
use contextra_vector::HnswIndex;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

/// Input dependencies required to construct [`CognitionWiring`].
pub struct CognitionWiringDeps<S: StorageEngine = LsmStorage, V: VectorIndex = HnswIndex> {
    pub collection: Arc<Collection<S, V>>,
    pub consolidation_config: ConsolidationConfig,
    pub synthesis_config: SynthesisConfig,
    pub maintenance_config: Option<MaintenanceConfig>,
    pub llm: Option<Arc<dyn LlmTextGenerator>>,
    pub rich_validator: Option<Arc<dyn GroundingValidator>>,
    pub leanrag_config: Option<AggregationConfig>,
    pub clock: Option<Arc<dyn Clock>>,
    pub rng: Option<Arc<dyn Rng>>,
}

/// Composition Root wiring for `contextra-cognition`.
pub struct CognitionWiring<
    S: StorageEngine + 'static = LsmStorage,
    V: VectorIndex + 'static = HnswIndex,
> {
    engine: Arc<ConsolidationEngine<S, V>>,
    scheduler: Arc<MaintenanceScheduler<S, V>>,
    clock: Arc<dyn Clock>,
    rng: Arc<dyn Rng>,
}

impl<S: StorageEngine + 'static, V: VectorIndex + 'static> CognitionWiring<S, V> {
    /// Builds `CognitionWiring` from dependencies conforming to Rule 7.
    pub fn build(deps: CognitionWiringDeps<S, V>) -> Result<Self> {
        let clock = deps.clock.unwrap_or_else(|| Arc::new(SystemClock::new()));
        let rng = deps.rng.unwrap_or_else(|| Arc::new(SystemRng::new()));

        let interval = Duration::from_secs(300);
        let cancel_token = CancellationToken::new();

        let mut engine = ConsolidationEngine::new(
            deps.collection.clone(),
            deps.consolidation_config.clone(),
            deps.synthesis_config,
            interval,
            cancel_token,
        );

        if let Some(llm) = deps.llm {
            engine = engine.with_llm(llm);
        }
        if let Some(rv) = deps.rich_validator {
            engine = engine.with_rich_validator(rv);
        }
        if let Some(l_cfg) = deps.leanrag_config {
            engine = engine.with_leanrag(l_cfg);
        }

        let maint_cfg = deps.maintenance_config.unwrap_or_default();
        let scheduler = Arc::new(MaintenanceScheduler::new(
            maint_cfg,
            deps.collection,
            deps.consolidation_config,
        ));

        Ok(Self {
            engine: Arc::new(engine),
            scheduler,
            clock,
            rng,
        })
    }

    /// Accessor for the underlying `ConsolidationEngine`.
    pub fn engine(&self) -> &Arc<ConsolidationEngine<S, V>> {
        &self.engine
    }

    /// Accessor for the underlying `MaintenanceScheduler`.
    pub fn scheduler(&self) -> &Arc<MaintenanceScheduler<S, V>> {
        &self.scheduler
    }

    /// Accessor for the `Clock` port.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// Accessor for the `Rng` port.
    pub fn rng(&self) -> &Arc<dyn Rng> {
        &self.rng
    }

    /// Executes the consolidation pipeline and returns `ConsolidationPipelineResult`.
    pub async fn run_pipeline(&self) -> Result<ConsolidationPipelineResult> {
        let (structural, synth_opt) = self.engine.run_cycle().await?;
        let synthesis = synth_opt.unwrap_or_default();

        Ok(ConsolidationPipelineResult {
            structural,
            synthesis,
            aggregation: None,
        })
    }
}
