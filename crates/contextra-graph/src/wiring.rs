//! Composition root wiring module for `contextra-graph`.
//!
//! Provides [`GraphWiring`] which unifies graph algorithms (Personalized PageRank with APPRH routing,
//! hyperedge co-occurrence suggestion and LLM validation, and relation ingestion)
//! into a cohesive execution pipeline for engine workers and composition roots.

use crate::apprh::{ApprhParams, ApprhSelector};
use crate::csr::CsrGraph;
use crate::entity_extraction::{ingest_extracted_relations, EntityExtractor, ExtractedRelation};
use crate::hyperedge::HyperEdge;
use crate::hyperedge_suggest::{
    compute_co_occurrence_candidates, validate_candidates_with_llm, ValidatedHyperEdgeCandidate,
};
use crate::ppr::{compute_ppr_with_apprh_selector, PprContext};
use contextra_ports::{GraphIndex, LlmTextGenerator};
use contextra_types::{ContextraError, DocId, EntityId, PprConfig, Result};
use std::sync::Arc;

/// Threshold on total graph node count above which APPRH selector routing is activated.
pub const APPRH_LARGE_GRAPH_NODE_THRESHOLD: usize = 100;

/// Dependencies required to construct a [`GraphWiring`] composition root instance.
pub struct GraphWiringDependencies {
    /// Target CSR graph instance.
    pub graph: Arc<CsrGraph>,
    /// Optional LLM text generator port for hyperedge candidate semantic validation.
    pub llm_generator: Option<Arc<dyn LlmTextGenerator>>,
    /// Optional entity extractor port for text-to-relation parsing.
    pub entity_extractor: Option<Arc<dyn EntityExtractor>>,
    /// Optional APPRH selector for hypergraph diffusion routing on large graphs.
    pub apprh_selector: Option<Arc<ApprhSelector>>,
}

/// Unified execution pipeline for graph signal processing and background workers.
///
/// Binds Personalized PageRank (PPR) power iteration with APPRH selector routing for large graphs,
/// document co-occurrence hyperedge candidate extraction with LLM semantic validation,
/// and extracted relation ingestion into hyperedges.
pub struct GraphWiring {
    graph: Arc<CsrGraph>,
    llm_generator: Option<Arc<dyn LlmTextGenerator>>,
    entity_extractor: Option<Arc<dyn EntityExtractor>>,
    apprh_selector: Arc<ApprhSelector>,
}

impl GraphWiring {
    /// Builds a new [`GraphWiring`] pipeline instance from provided dependencies.
    ///
    /// Implements the composition root contract for `contextra/src/builder.rs` (Rule 7).
    /// If `deps.apprh_selector` is not provided, initializes a default [`ApprhSelector`].
    pub fn build(deps: GraphWiringDependencies) -> Result<Self> {
        let apprh_selector = deps
            .apprh_selector
            .unwrap_or_else(|| Arc::new(ApprhSelector::default()));

        Ok(Self {
            graph: deps.graph,
            llm_generator: deps.llm_generator,
            entity_extractor: deps.entity_extractor,
            apprh_selector,
        })
    }

    /// Accessor for the underlying CSR graph.
    pub fn graph(&self) -> &Arc<CsrGraph> {
        &self.graph
    }

    /// Calculates Personalized PageRank (PPR) using APPRH selector routing for large graphs.
    ///
    /// # Parameters
    /// - `seed_nodes`: Anchor entity IDs for the random walk.
    /// - `config`: PPR configuration parameters:
    ///   - `damping_factor`: Teleport probability (default 0.85, restart probability $\alpha = 1 - d$).
    ///   - `convergence_epsilon`: Termination threshold for residual norm (default $10^{-6}$).
    ///   - `max_iterations`: Maximum push steps ceiling (hard cap 1000).
    ///
    /// # Determinism & APPRH Selector Routing
    /// For graphs with $\ge$ [`APPRH_LARGE_GRAPH_NODE_THRESHOLD`] (100) nodes, routes through
    /// [`compute_ppr_with_apprh_selector`]. Otherwise routes through standard [`CsrGraph::personalized_page_rank`].
    /// Yields 100% bit-identical, reproducible scores across repeated executions given identical seeds (AGT-GRAPH-001, ADR-026/027).
    pub async fn compute_ppr_pipeline(
        &self,
        seed_nodes: &[EntityId],
        config: &PprConfig,
    ) -> Result<Vec<(EntityId, f32)>> {
        let node_count = self.graph.entity_count();

        if node_count >= APPRH_LARGE_GRAPH_NODE_THRESHOLD {
            let deleted_view = self.graph.deleted_view().await;
            let _ = self.graph.compact_async().await;
            let inner = self.graph.inner_read();
            let mut ctx = PprContext::new();

            let apprh_params = ApprhParams {
                ppr: crate::path_rag::PprParams {
                    alpha: if config.damping_factor.is_nan()
                        || config.damping_factor <= 0.0
                        || config.damping_factor >= 1.0
                    {
                        0.15
                    } else {
                        1.0 - config.damping_factor
                    },
                    epsilon: if config.convergence_epsilon.is_nan()
                        || config.convergence_epsilon <= 0.0
                    {
                        1e-6
                    } else {
                        config.convergence_epsilon
                    },
                    hyperedge_decay: 0.85,
                },
                hyperedge_decay_factor: 0.85,
                max_iterations: config.max_iterations * 10,
            };

            Ok(compute_ppr_with_apprh_selector(
                &inner,
                seed_nodes,
                config,
                &deleted_view,
                &mut ctx,
                &self.apprh_selector,
                &apprh_params,
            ))
        } else {
            self.graph.personalized_page_rank(seed_nodes, config).await
        }
    }

    /// Computes co-occurrence candidates and validates them using the configured LLM text generator port.
    ///
    /// # Pipeline Steps
    /// 1. Runs [`compute_co_occurrence_candidates`] over document entity lists.
    /// 2. If candidates are found, validates them via [`validate_candidates_with_llm`].
    ///
    /// # Error Handling
    /// If no LLM text generator port is configured (`llm_generator == None`), returns
    /// `Err(ContextraError::InvalidInput)` indicating missing LLM port without inventing mock data.
    pub async fn suggest_and_validate_hyperedges(
        &self,
        entities_per_doc: &[(DocId, Vec<EntityId>)],
        min_co_occurrence: usize,
        max_llm_calls: usize,
        entity_labels: &dyn Fn(EntityId) -> Option<String>,
    ) -> Result<Vec<ValidatedHyperEdgeCandidate>> {
        let candidates = compute_co_occurrence_candidates(entities_per_doc, min_co_occurrence);
        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        let gen_ref = self
            .llm_generator
            .as_deref()
            .map(|g| g as &dyn LlmTextGenerator);
        validate_candidates_with_llm(&candidates, entity_labels, gen_ref, max_llm_calls)
            .await
            .map_err(|e| ContextraError::InvalidInput(e.to_string()))
    }

    /// Ingests pre-extracted relations into the underlying CSR graph as hyperedges.
    ///
    /// Filters by `min_confidence_threshold` and calls [`ingest_extracted_relations`].
    pub fn ingest_relations(
        &self,
        relations: &[ExtractedRelation],
        min_confidence_threshold: f32,
        doc_id: Option<DocId>,
    ) -> Result<Vec<HyperEdge>> {
        ingest_extracted_relations(&self.graph, relations, min_confidence_threshold, doc_id)
    }

    /// Extracts relations from raw text using the configured [`EntityExtractor`] and ingests them into the CSR graph.
    ///
    /// # Fallback Behavior
    /// If no custom entity extractor port is provided in dependencies, uses a default [`crate::entity_extraction::RuleBasedEntityExtractor`].
    pub fn extract_and_ingest_relations(
        &self,
        text: &str,
        min_confidence_threshold: f32,
        doc_id: Option<DocId>,
    ) -> Result<Vec<HyperEdge>> {
        let default_extractor = crate::entity_extraction::RuleBasedEntityExtractor::default();
        let extractor: &dyn EntityExtractor = match self.entity_extractor.as_ref() {
            Some(e) => e.as_ref(),
            None => &default_extractor,
        };

        let relations = extractor.extract(text);
        self.ingest_relations(&relations, min_confidence_threshold, doc_id)
    }
}
