//! Public erase API (`impl Contextra`, K-8): `erase`, `erase_status`, `erase_dry_run`.

use super::registry::LayerRegistry;
use super::resolver::ScopeResolverImpl;
use super::types::{EraseHandle, ErasePlanSummary};
use super::{EraseOptions, ErasureRuntimeInner};
use crate::Contextra;
use contextra_ports::erasure::{DocScope, EraseRequest, ResolvedScope};
use contextra_types::{
    AwaitLevel, ContextraError, DocId, EraseTarget, ErasureError, ErasureId, ErasurePolicy,
    ErasureState, ErasureStatus, Guarantee,
};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

/// Maps the erasure error space onto the public error type (Spezifikation §12).
pub(crate) fn map_erasure_error(e: ErasureError) -> ContextraError {
    match e {
        ErasureError::InvalidTarget(msg) => ContextraError::invalid_input(msg),
        ErasureError::TenantViolation => ContextraError::invalid_input("erase scope violates tenant isolation"),
        ErasureError::ScopeAmbiguous | ErasureError::IdempotencyConflict => {
            ContextraError::Conflict("erase scope is ambiguous or conflicts with an earlier request".into())
        }
        ErasureError::NotFound => ContextraError::NotFound("erasure job".into()),
        other => ContextraError::Storage(other.to_string()),
    }
}

impl ErasureRuntimeInner {
    /// Waits until job `id` reached G3 (`Sealed`/`Completed`). Fails if the job is quarantined
    /// or the engine shuts down.
    pub(crate) async fn wait_purged(&self, id: ErasureId) -> Result<ErasureStatus, ContextraError> {
        loop {
            let st = self
                .kernel
                .status(id)
                .ok_or_else(|| ContextraError::NotFound("erasure job".into()))?;
            if st.reached.purged || matches!(st.state, ErasureState::Sealed | ErasureState::Completed) {
                return Ok(st);
            }
            if matches!(st.state, ErasureState::Quarantined { .. }) {
                return Err(ContextraError::Storage("erasure job quarantined".into()));
            }
            if self.cancel.is_cancelled() {
                return Err(ContextraError::Storage("engine shutting down".into()));
            }
            let poll = self.cfg.recheck_interval.min(Duration::from_secs(1));
            tokio::select! {
                _ = self.progress.notified() => {}
                _ = tokio::time::sleep(poll) => {}
                _ = self.cancel.cancelled() => {}
            }
        }
    }

    /// Waits until job `id` is `Completed` (or fails like [`Self::wait_purged`]).
    pub(crate) async fn wait_completed(&self, id: ErasureId) -> Result<(), ContextraError> {
        loop {
            let st = self.wait_purged(id).await?;
            if matches!(st.state, ErasureState::Completed) {
                return Ok(());
            }
            let poll = self.cfg.recheck_interval.min(Duration::from_secs(1));
            tokio::select! {
                _ = self.progress.notified() => {}
                _ = tokio::time::sleep(poll) => {}
                _ = self.cancel.cancelled() => {
                    return Err(ContextraError::Storage("engine shutting down".into()));
                }
            }
        }
    }

    fn new_id(&self) -> ErasureId {
        let mut b = [0u8; 16];
        self.rng.fill_bytes(&mut b);
        ErasureId::new(b)
    }
}

impl Contextra {
    /// Resolves `target` into a [`ResolvedScope`]. Read-only and deterministic; shared by
    /// `erase` and `erase_dry_run`.
    pub(crate) async fn resolve_erase_plan(
        &self,
        id: ErasureId,
        target: &EraseTarget,
    ) -> Result<ResolvedScope, ContextraError> {
        target.validate().map_err(map_erasure_error)?;
        match target {
            EraseTarget::Document { tenant, collection, key } => {
                let col = {
                    let map = self.tenant_collections.read().await;
                    map.get(&(*tenant, collection.clone())).cloned()
                }
                .ok_or_else(|| ContextraError::NotFound("collection".into()))?;

                let doc_id = DocId::from_key(key)?;
                let doc_key = col.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
                let user_key = col.namespaced_key(key.as_bytes(), 0);
                let reverse = {
                    use contextra_ports::StorageEngine;
                    col.storage().get(&doc_key).await?
                }
                .ok_or_else(|| ContextraError::invalid_input("document not found"))?;
                let extras = col.erasure_extra_document_keys(key, doc_id).await?;
                ScopeResolverImpl::document_with_extras(
                    id, *tenant, collection, key, user_key, doc_key, &reverse, extras, 0,
                )
                .map_err(map_erasure_error)
            }
            EraseTarget::Collection { tenant, collection } => {
                ScopeResolverImpl::collection(id, *tenant, collection).map_err(map_erasure_error)
            }
            EraseTarget::Tenant { tenant } => {
                ScopeResolverImpl::tenant(id, *tenant).map_err(map_erasure_error)
            }
            _ => Err(ContextraError::invalid_input("unsupported erase target")),
        }
    }

    /// Starts an erase job. Returns once the read gate is durably active (G1);
    /// `AwaitLevel::Purged` additionally waits until the physical purge is verified (G3).
    ///
    /// Each call creates a new job id. G1 runs in the caller's context and is never throttled.
    pub async fn erase(
        &self,
        target: EraseTarget,
        opts: EraseOptions,
    ) -> Result<EraseHandle, ContextraError> {
        let rt = Arc::clone(self.erasure.inner());
        if rt.shutting_down.load(Ordering::Acquire) {
            return Err(ContextraError::Storage("engine shutting down".into()));
        }
        let id = rt.new_id();
        let plan = self.resolve_erase_plan(id, &target).await?;
        let policy = opts.policy.clone().unwrap_or_else(|| ErasurePolicy {
            checkpoint: rt.cfg.checkpoint_policy,
            sla_secs: rt.cfg.sla.as_secs(),
            ..ErasurePolicy::default()
        });
        let req = EraseRequest { id, target, reason: opts.reason, policy };

        if opts.await_level == AwaitLevel::Purged {
            // The caller waits: skip the coalescing window for this job.
            let now = rt.clock.monotonic_nanos();
            let window = rt.cfg.coalesce_window.as_nanos().min(u64::MAX as u128) as u64;
            rt.sched.lock().insert(
                id,
                super::driver::JobSched {
                    first_seen: now.saturating_sub(window),
                    next_attempt: 0,
                    sla_reported: false,
                },
            );
        }

        rt.accept_and_gate(&req, &plan).await.map_err(map_erasure_error)?;
        rt.notify.notify_one();

        let (reached, state) = if opts.await_level == AwaitLevel::Purged {
            let st = rt.wait_purged(id).await?;
            (st.reached, st.state)
        } else {
            let st = rt.kernel.status(id);
            (
                Guarantee { gated: true, ..Guarantee::default() },
                st.map(|s| s.state).unwrap_or(ErasureState::Gated),
            )
        };
        Ok(EraseHandle { id, reached, state })
    }

    /// Current status of a job. Unknown id → `NotFound`.
    pub async fn erase_status(&self, id: ErasureId) -> Result<ErasureStatus, ContextraError> {
        self.erasure
            .inner()
            .kernel
            .status(id)
            .ok_or_else(|| ContextraError::NotFound("erasure job".into()))
    }

    /// Same planner as `erase`, no effect. Returns counts only, never keys.
    pub async fn erase_dry_run(&self, target: EraseTarget) -> Result<ErasePlanSummary, ContextraError> {
        let plan = self
            .resolve_erase_plan(ErasureId::new([0u8; 16]), &target)
            .await?;
        let docs = plan
            .docs
            .iter()
            .map(|d| match d {
                DocScope::Set { docs, .. } => docs.len() as u64,
                _ => 0,
            })
            .sum();
        let layers = self
            .erasure
            .inner()
            .registry
            .read()
            .layers_for(&plan)
            .iter()
            .map(|(_, l)| l.layer().as_str().to_owned())
            .collect();
        Ok(ErasePlanSummary {
            rules: plan.keys.rules().len() as u32,
            docs,
            unprovenanced: plan.unprovenanced,
            layers,
            est_bytes: 0,
            checkpoints: 0,
        })
    }

    /// Registers a secondary erasure layer for a tenant collection (text, graph, vector,
    /// checkpoint). Registrations are deterministic and replace atomically.
    pub fn register_erasure_layer(
        &self,
        order: super::registry::LayerOrder,
        tenant: contextra_types::TenantId,
        collection: &str,
        layer: Arc<dyn contextra_ports::ErasureLayer>,
    ) {
        let key = LayerRegistry::collection_key(tenant, collection);
        self.erasure.inner().registry.write().register(order, key, layer);
    }

    /// Registers an observer that is notified (detached, never blocking) on `Gated`/`Completed`.
    pub fn register_erasure_observer(&self, observer: Arc<dyn contextra_ports::ErasureObserver>) {
        self.erasure.inner().observers.write().push(observer);
    }
}
