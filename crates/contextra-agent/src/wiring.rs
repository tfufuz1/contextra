// FILE-CONTEXT Header (Format v3)
// ZWECK: Central composition root for contextra-agent subsystems.
// INVARIANTEN: Entrypoint contract `AgentWiring::build` for facade binding.
// STAND: TS:2026-10-13T00:00:00Z

//! Wiring entrypoint for contextra-agent subsystems.

use crate::audit::AuditLog;
use crate::dlq::DeadLetterQueue;
use crate::engine::OrchestratorEngine;
use crate::event_source::{EventSource, VecEventSource};
use contextra_db::{Collection, Contextra};
use contextra_ports::{Clock, IdGenerator};
use contextra_store::LsmStorage;
use contextra_types::{Result, TokenBudget};
use std::sync::Arc;

/// Composition root structure assembling agent orchestration runtime dependencies.
pub struct AgentWiring {
    pub orchestrator: OrchestratorEngine,
    pub dlq: Arc<DeadLetterQueue>,
    pub audit_log: AuditLog,
    pub budget: TokenBudget,
    pub event_source: Box<dyn EventSource>,
    pub clock: Arc<dyn Clock>,
    pub id_gen: Arc<dyn IdGenerator>,
}

impl AgentWiring {
    /// Builds a fully configured [`AgentWiring`] composition root.
    pub fn build(
        db: Arc<Contextra>,
        state_collection: Arc<Collection<LsmStorage>>,
        budget: TokenBudget,
        clock: Arc<dyn Clock>,
        id_gen: Arc<dyn IdGenerator>,
    ) -> Result<Self> {
        let event_source = Box::new(VecEventSource::try_new(vec![])?);
        Self::build_with_event_source(db, state_collection, budget, clock, id_gen, event_source)
    }

    /// Builds a fully configured [`AgentWiring`] composition root with a custom [`EventSource`].
    pub fn build_with_event_source(
        db: Arc<Contextra>,
        state_collection: Arc<Collection<LsmStorage>>,
        budget: TokenBudget,
        clock: Arc<dyn Clock>,
        id_gen: Arc<dyn IdGenerator>,
        event_source: Box<dyn EventSource>,
    ) -> Result<Self> {
        let storage = db.inner_storage();
        let orchestrator = OrchestratorEngine::try_new(storage.clone())?.with_clock(clock.clone());
        let dlq = Arc::new(DeadLetterQueue::new(storage));
        let audit_log = AuditLog::new(state_collection);

        Ok(Self {
            orchestrator,
            dlq,
            audit_log,
            budget,
            event_source,
            clock,
            id_gen,
        })
    }
}
