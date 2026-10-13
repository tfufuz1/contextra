// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Composition Root Wiring für contextra-router (Fassaden-Schnittstelle §4.14).
// INVARIANTEN: Re-exportiert RouterWiring und RouterHandle; implementiert DriftStatusProvider.

//! Composition root wiring module for `contextra-router`.

use crate::arm_registry::ArmRegistry;
use crate::dispatch::route_and_dispatch_to_slm;
use crate::profile::SlmProfile;
use crate::router::RouterEngine;
use crate::transport::Transport;
use contextra_ports::{
    Clock, CommunityResolver, ContextPreparer, DriftStatusProvider, HybridSearchProvider,
};
use contextra_types::{ContextraError, Result};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Dependencies required to assemble a [`RouterWiring`] instance.
pub struct RouterWiringDeps {
    pub search_provider: Arc<dyn HybridSearchProvider>,
    pub community_resolver: Arc<dyn CommunityResolver>,
    pub context_preparer: Arc<dyn ContextPreparer>,
    pub catalog: Vec<SlmProfile>,
    pub clock: Arc<dyn Clock>,
    pub arm_registry: Option<ArmRegistry>,
    pub calibration_store_path: Option<PathBuf>,
    pub transport: Option<Transport>,
    pub route_timeout: Option<Duration>,
}

/// Assembly root for `contextra-router` composing [`RouterEngine`], [`ArmRegistry`],
/// SLM profile catalog, drift monitoring, and transport.
///
/// Implements [`DriftStatusProvider`] so downstream consumers (e.g., `contextra-engine`)
/// can register it via `set_router(Arc::downgrade(&router) as Weak<dyn DriftStatusProvider>)`.
pub struct RouterWiring {
    pub engine: Arc<RouterEngine>,
    pub arm_registry: ArmRegistry,
    pub catalog: Vec<SlmProfile>,
    pub clock: Arc<dyn Clock>,
    pub transport: Transport,
    pub route_timeout: Duration,
}

impl RouterWiring {
    /// Builds a new `RouterWiring` instance from provided dependencies.
    ///
    /// Adheres to the facade entrypoint contract (`pub fn build(deps) -> Result<Self>`).
    pub fn build(deps: RouterWiringDeps) -> Result<Self> {
        let engine = RouterEngine::try_new(
            deps.search_provider,
            deps.community_resolver,
            deps.context_preparer,
            deps.catalog.clone(),
            deps.calibration_store_path,
        )?;

        let arm_registry = deps.arm_registry.unwrap_or_default();
        let transport = deps.transport.unwrap_or_default();
        let route_timeout = deps.route_timeout.unwrap_or(Duration::from_secs(30));

        Ok(Self {
            engine: Arc::new(engine),
            arm_registry,
            catalog: deps.catalog,
            clock: deps.clock,
            transport,
            route_timeout,
        })
    }

    /// Returns a lightweight, cloneable handle for executing query routing and SLM dispatching.
    pub fn handle(self: &Arc<Self>) -> RouterHandle {
        RouterHandle {
            wiring: Arc::clone(self),
        }
    }
}

impl DriftStatusProvider for RouterWiring {
    fn overall_drift_status(&self) -> String {
        self.engine.overall_drift_status()
    }
}

/// Lightweight handle providing a single-entrypoint interface for query routing and dispatching.
#[derive(Clone)]
pub struct RouterHandle {
    wiring: Arc<RouterWiring>,
}

impl RouterHandle {
    /// Creates a new `RouterHandle` from a shared [`RouterWiring`] reference.
    pub fn new(wiring: Arc<RouterWiring>) -> Self {
        Self { wiring }
    }

    /// Routes a query embedding and text to the best matching SLM profile and dispatches
    /// the prepared context over the configured transport channel.
    ///
    /// Execution is bounded by the timeout configured in `RouterWiring` (or derived from `Clock`).
    /// Automatic outcome feedback (`RoutingOutcome::Success` / `RoutingOutcome::Rejected`)
    /// is recorded directly on the underlying `RouterEngine`.
    pub async fn route(&self, query_embedding: &[f32], query_text: &str) -> Result<String> {
        let timeout_duration = self.wiring.route_timeout;

        tokio::time::timeout(
            timeout_duration,
            route_and_dispatch_to_slm(&self.wiring.engine, query_embedding, query_text),
        )
        .await
        .map_err(|_| ContextraError::Timeout {
            operation: "Routing and SLM dispatch".to_string(),
            timeout_ms: timeout_duration.as_millis() as u64,
        })?
    }

    /// Returns a reference to the underlying [`RouterEngine`].
    pub fn engine(&self) -> &Arc<RouterEngine> {
        &self.wiring.engine
    }
}
