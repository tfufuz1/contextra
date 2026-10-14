// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Text, Graph, Router, and Adapt Subsystem Wiring for contextra facade (Ring 4).
// INVARIANTEN: #![forbid(unsafe_code)]; Connects retrieval engines and holds references to keep owners alive.

#![forbid(unsafe_code)]

#[cfg(feature = "router")]
use std::sync::Arc;

#[cfg(feature = "router")]
use contextra_adapt::wiring::{AdaptWiring, AdaptWiringDeps};
#[cfg(feature = "router")]
use contextra_core::error::ContextraError;
#[cfg(feature = "router")]
use contextra_graph::csr::CsrGraph;
#[cfg(feature = "router")]
use contextra_graph::wiring::{GraphWiring, GraphWiringDependencies};
#[cfg(feature = "router")]
use contextra_ports::clock::Clock;
#[cfg(feature = "router")]
use contextra_ports::types::{ContextChunk, ContextWindow, EntityId, TokenBudget};
#[cfg(feature = "router")]
use contextra_router::wiring::{RouterWiring, RouterWiringDeps};

/// Bundles Text, Graph, Router, and Adapt retrieval subsystem instances.
#[cfg(feature = "router")]
pub struct RetrievalWiringBundle {
    pub graph_wiring: Option<GraphWiring>,
    pub router_wiring: Option<RouterWiring>,
    pub adapt_wiring: AdaptWiring,
}

#[cfg(feature = "router")]
impl RetrievalWiringBundle {
    /// Builds `RetrievalWiringBundle` using injected clock port and settings.
    pub fn build(
        router_enabled: bool,
        adapt_enabled: bool,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, ContextraError> {
        let adapt_wiring = AdaptWiring::build(AdaptWiringDeps {
            clock: clock.clone(),
        })
        .map_err(|e| ContextraError::PolicyViolation(e.to_string()))?;

        let graph = Arc::new(CsrGraph::new());
        let graph_wiring = GraphWiring::build(GraphWiringDependencies {
            graph,
            llm_generator: None,
            entity_extractor: None,
            apprh_selector: None,
        })
        .ok();

        let router_wiring = if router_enabled {
            struct DummyHybridSearch;
            impl contextra_ports::HybridSearchProvider for DummyHybridSearch {
                fn search_hybrid<'a>(
                    &'a self,
                    _query_text: &'a str,
                    _query_embedding: &'a [f32],
                    _top_k: usize,
                ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Vec<ContextChunk>>> {
                    Box::pin(async move { Ok(Vec::new()) })
                }
            }

            struct DummyCommunityResolver;
            impl contextra_ports::CommunityResolver for DummyCommunityResolver {
                fn get_community<'a>(
                    &'a self,
                    _entity_id: EntityId,
                ) -> contextra_ports::BoxFuture<'a, contextra_types::Result<Option<u64>>> {
                    Box::pin(async move { Ok(None) })
                }
            }

            struct DummyContextPreparer;
            impl contextra_ports::ContextPreparer for DummyContextPreparer {
                fn prepare_context(
                    &self,
                    _chunks: Vec<ContextChunk>,
                    _budget: &TokenBudget,
                    _relevance_threshold: f32,
                ) -> contextra_types::Result<ContextWindow> {
                    Ok(ContextWindow {
                        chunks: Vec::new(),
                        total_tokens: 0,
                        truncated: false,
                    })
                }
            }

            RouterWiring::build(RouterWiringDeps {
                search_provider: Arc::new(DummyHybridSearch),
                community_resolver: Arc::new(DummyCommunityResolver),
                context_preparer: Arc::new(DummyContextPreparer),
                catalog: Vec::new(),
                clock: clock.clone(),
                arm_registry: None,
                calibration_store_path: None,
                transport: None,
                route_timeout: None,
            })
            .ok()
        } else {
            None
        };

        let _ = adapt_enabled;

        Ok(Self {
            graph_wiring,
            router_wiring,
            adapt_wiring,
        })
    }
}
