//! Saga driver (Spezifikation §6): resumable, idempotent, no lock held across layer calls.

use super::adapters::{KernelProgress, TokenCancel};
use super::registry::LayerOrder;
use super::ErasureRuntimeInner;
use contextra_ports::erasure::{
    DocScope, EraseRequest, GateReceipt, IoBudget, JobView, PressureHint, PrivilegedRead,
    PurgeCx, ResolvedScope,
};
use contextra_ports::observer::{ErasureEvent, ObservedPhase};
use contextra_ports::scanner::{LayerScanner, LayerWitness, ScanTarget, ScanTechnique, WitnessLayer};
use contextra_ports::ErasureLayer;
use contextra_types::{
    ErasureError, ErasureId, ErasureState, LayerErrorKind, RequestContext, TargetKind, TenantId,
    Timestamp, TxId,
};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

/// Scheduling state of one open job (monotonic nanoseconds of the engine clock).
#[derive(Debug, Clone, Copy)]
pub struct JobSched {
    /// When the worker first saw the job (start of the coalescing window).
    pub first_seen: u64,
    /// Earliest time of the next attempt (backoff), `0` = immediately.
    pub next_attempt: u64,
    /// Whether the SLA breach was already reported.
    pub sla_reported: bool,
}

/// Result of one `drive_job` call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveOutcome {
    /// Job reached `Completed`.
    Completed,
    /// Sealed, waiting for remaining tainted checkpoints / legal hold.
    Waiting,
    /// Retry after the given delay (backoff with jitter).
    Retry(Duration),
    /// Cancelled; resume on the next pass.
    Paused,
    /// Quarantined; needs operator intervention.
    Quarantined,
    /// Journal poisoned; job stays gated, health degraded.
    Poisoned,
}

fn hex(id: &ErasureId) -> String {
    let mut s = String::with_capacity(32);
    for b in id.as_bytes() {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn tenant_of(scope: &ResolvedScope) -> TenantId {
    scope
        .docs
        .iter()
        .map(|d| match d {
            DocScope::Set { tenant, .. }
            | DocScope::Collection { tenant, .. }
            | DocScope::Tenant { tenant } => *tenant,
        })
        .next()
        .unwrap_or(TenantId::SYSTEM)
}

impl ErasureRuntimeInner {
    /// Fires an observer event. Delivery is detached; failures are only counted (02 §4.8).
    pub(crate) fn notify_observers(
        self: &Arc<Self>,
        id: ErasureId,
        kind: TargetKind,
        phase: ObservedPhase,
    ) {
        let observers: Vec<_> = self.observers.read().iter().cloned().collect();
        if observers.is_empty() {
            return;
        }
        let ev = ErasureEvent {
            id,
            target_kind: kind,
            phase,
            at: Timestamp::from_micros(self.clock.now_unix_nanos() / 1_000),
        };
        for obs in observers {
            let fut = obs.on_erased(ev.clone());
            let rt = Arc::clone(self);
            tokio::spawn(async move {
                if let Err(e) = fut.await {
                    rt.observer_failures.fetch_add(1, Ordering::Relaxed);
                    tracing::warn!(error = %e, "erasure observer delivery failed");
                }
            });
        }
    }

    /// RAM masking on all layers participating in `plan` (fast, no file I/O). Failures never
    /// abort: the durable gate in the store already protects reads.
    pub(crate) async fn mask_layers(&self, plan: &ResolvedScope) {
        let layers = { self.registry.read().layers_for(plan) };
        for (_, layer) in layers {
            if let Err(e) = layer.on_gate(plan).await {
                tracing::warn!(layer = %e.layer.as_str(), kind = ?e.kind, "on_gate failed");
            }
        }
    }

    /// G1 path, runs in the caller's context and is never throttled:
    /// `accept` → `publish_gate` → observers → RAM masking.
    pub async fn accept_and_gate(
        self: &Arc<Self>,
        req: &EraseRequest,
        plan: &ResolvedScope,
    ) -> Result<GateReceipt, ErasureError> {
        self.kernel.accept(req, plan).await?;
        let gate = match self.kernel.publish_gate(req.id).await {
            Ok(g) => g,
            Err(e) => {
                // Before `Gated` the job is aborted; the gate was never published.
                let _ = self
                    .kernel
                    .fail(req.id, None, LayerErrorKind::Permanent)
                    .await;
                return Err(e);
            }
        };
        self.notify_observers(req.id, plan.target_kind, ObservedPhase::Gated);
        self.mask_layers(plan).await;
        Ok(gate)
    }

    /// Backoff with jitter: `min · 2^attempt` capped at `max`, plus up to 25 % jitter from the
    /// `Rng` port.
    pub(crate) fn backoff(&self, attempt: u32) -> Duration {
        let (min, max) = self.cfg.retry_backoff;
        let factor = 1u32.checked_shl(attempt.min(20)).unwrap_or(u32::MAX);
        let base = min.saturating_mul(factor).min(max);
        let jitter_ms = self.rng.next_u64() % ((base.as_millis() as u64) / 4 + 1);
        base + Duration::from_millis(jitter_ms)
    }

    async fn run_layer(
        &self,
        id: ErasureId,
        layer: &Arc<dyn ErasureLayer>,
        scope: &ResolvedScope,
        reader: &Arc<dyn PrivilegedRead>,
        cancel: &TokenCancel,
        progress: &KernelProgress,
    ) -> Result<(), ErasureError> {
        let mut cursor = None;
        loop {
            if cancel.0.is_cancelled() {
                return Err(ErasureError::Cancelled);
            }
            let cx = PurgeCx {
                erasure_id: id,
                clock: &*self.clock,
                cancel,
                budget: &*self.budget,
                cursor: cursor.clone(),
                reader: &**reader,
                progress,
                max_units: self.cfg.max_units_per_tick,
            };
            let out = layer.purge(scope, &cx).await.map_err(ErasureError::from)?;
            self.kernel
                .record_layer(id, &layer.layer(), &out.progress)
                .await?;
            match out.progress.cursor {
                None => return Ok(()),
                Some(c) => cursor = Some(c),
            }
        }
    }

    async fn purge_phase(
        &self,
        job: &JobView,
        scope: &ResolvedScope,
    ) -> Result<(), ErasureError> {
        let id = job.id;
        let reader = self.kernel.privileged_reader(id)?;
        let cancel = TokenCancel(self.cancel.clone());
        let progress = KernelProgress {
            kernel: Arc::clone(&self.kernel),
            id,
        };
        let layers = { self.registry.read().layers_for(scope) };

        // (2) Text → Graph → Vector, then (3) store, then (4) checkpoints.
        for (order, layer) in layers.iter().filter(|(o, _)| *o != LayerOrder::Checkpoint) {
            let _ = order;
            if job.layers_done.contains(&layer.layer()) {
                continue;
            }
            self.run_layer(id, layer, scope, &reader, &cancel, &progress)
                .await?;
        }

        self.purge_rounds.fetch_add(1, Ordering::Relaxed);
        let mut cursor = None;
        loop {
            if cancel.0.is_cancelled() {
                return Err(ErasureError::Cancelled);
            }
            let cx = PurgeCx {
                erasure_id: id,
                clock: &*self.clock,
                cancel: &cancel,
                budget: &*self.budget,
                cursor: cursor.clone(),
                reader: &*reader,
                progress: &progress,
                max_units: self.cfg.max_units_per_tick,
            };
            let out = self.kernel.purge_store(id, &cx).await?;
            match out.progress.cursor {
                None => break,
                Some(c) => cursor = Some(c),
            }
        }

        for (_, layer) in layers.iter().filter(|(o, _)| *o == LayerOrder::Checkpoint) {
            if job.layers_done.contains(&layer.layer()) {
                continue;
            }
            self.run_layer(id, layer, scope, &reader, &cancel, &progress)
                .await?;
        }
        Ok(())
    }

    /// Collects one witness per expected layer plus the store scanners (blocking work runs in
    /// `spawn_blocking`).
    #[allow(deprecated)]
    async fn collect_witnesses(
        &self,
        id: ErasureId,
        scope: &ResolvedScope,
    ) -> Result<Vec<LayerWitness>, ErasureError> {
        let layers: Vec<Arc<dyn ErasureLayer>> = {
            self.registry
                .read()
                .layers_for(scope)
                .into_iter()
                .map(|(_, l)| l)
                .collect()
        };
        let store_scanners: Vec<Arc<dyn LayerScanner>> = self.kernel.store_scanners();
        let clock = Arc::clone(&self.clock);
        let tenant = tenant_of(scope);
        let plan_hash = scope.plan_hash;
        let target_id = hex(&id);

        let joined = tokio::task::spawn_blocking(move || {
            let ctx = RequestContext::new(tenant, TxId::new(0));
            let target = || ScanTarget::new(target_id.clone(), plan_hash);
            let mut out = Vec::with_capacity(layers.len() + store_scanners.len());
            for l in &layers {
                out.push(l.scanner().scan(
                    &ctx,
                    target(),
                    ScanTechnique::IndexIntrospection,
                    &*clock,
                )?);
            }
            for s in &store_scanners {
                out.push(s.scan(&ctx, target(), ScanTechnique::Structural, &*clock)?);
            }
            Ok::<_, contextra_types::ContextraError>(out)
        })
        .await;
        match joined {
            Ok(Ok(w)) => Ok(w),
            Ok(Err(e)) => Err(ErasureError::Io(e.to_string())),
            Err(e) => Err(ErasureError::Io(format!("scan task failed: {e}"))),
        }
    }

    async fn verify_and_seal(
        &self,
        id: ErasureId,
        scope: &ResolvedScope,
    ) -> Result<(), ErasureError> {
        let witnesses = self.collect_witnesses(id, scope).await?;
        if let Some(w) = witnesses.iter().find(|w| w.remaining() > 0) {
            // Residue found: back to purge via a retryable failure (attempt++).
            return Err(ErasureError::Layer {
                layer: w.layer().to_owned(),
                kind: LayerErrorKind::Retryable,
            });
        }
        self.kernel.seal(id, &witnesses).await?;
        Ok(())
    }

    fn layer_of(err: &ErasureError) -> Option<WitnessLayer> {
        match err {
            ErasureError::Layer { layer, .. } => Some(WitnessLayer::from(layer.as_str())),
            _ => None,
        }
    }

    async fn on_failure(&self, job: &JobView, err: ErasureError) -> DriveOutcome {
        let id = job.id;
        let layer = Self::layer_of(&err);
        tracing::warn!(attempt = job.attempt, kind = ?err.kind(), "erasure step failed");
        match err {
            ErasureError::Cancelled => return DriveOutcome::Paused,
            ErasureError::JournalPoisoned => return DriveOutcome::Poisoned,
            // Space/busy problems keep the job gated and are retried; no state transition.
            ErasureError::InsufficientSpace { .. } | ErasureError::Busy { .. } => {
                return DriveOutcome::Retry(self.backoff(job.attempt));
            }
            _ => {}
        }
        let mut kind = err.kind();
        if kind == LayerErrorKind::Retryable
            && job.attempt.saturating_add(1) >= self.cfg.quarantine_after_attempts
        {
            kind = LayerErrorKind::Permanent;
        }
        match kind {
            LayerErrorKind::Cancelled => DriveOutcome::Paused,
            LayerErrorKind::Poisoned => DriveOutcome::Poisoned,
            LayerErrorKind::Retryable => {
                let _ = self.kernel.fail(id, layer.as_ref(), LayerErrorKind::Retryable).await;
                DriveOutcome::Retry(self.backoff(job.attempt))
            }
            LayerErrorKind::Permanent | LayerErrorKind::ScopeViolation => {
                let _ = self.kernel.fail(id, layer.as_ref(), kind).await;
                tracing::error!("erasure job quarantined");
                DriveOutcome::Quarantined
            }
        }
    }

    /// Drives one job as far as it can go in this call. Resumable from any state.
    pub async fn drive_job(self: &Arc<Self>, job: &JobView) -> DriveOutcome {
        let id = job.id;
        let scope = Arc::clone(&job.scope);
        let mut state = job.state.clone();

        // Resumption before the gate existed (crash between accept and publish).
        if matches!(
            state,
            ErasureState::Requested | ErasureState::Planned | ErasureState::Gating
        ) {
            match self.kernel.publish_gate(id).await {
                Ok(_) => {
                    self.notify_observers(id, scope.target_kind, ObservedPhase::Gated);
                    self.mask_layers(&scope).await;
                    state = ErasureState::Gated;
                }
                Err(e) => return self.on_failure(job, e).await,
            }
        }

        if matches!(state, ErasureState::Quarantined { .. }) {
            return DriveOutcome::Quarantined;
        }

        if matches!(
            state,
            ErasureState::Gated
                | ErasureState::KeysDestroying
                | ErasureState::Purging
                | ErasureState::Failed { .. }
        ) {
            if let Err(e) = self.purge_phase(job, &scope).await {
                return self.on_failure(job, e).await;
            }
            state = ErasureState::Verifying;
        }

        if matches!(state, ErasureState::Verifying) {
            if let Err(e) = self.verify_and_seal(id, &scope).await {
                return self.on_failure(job, e).await;
            }
        }

        // Sealed (now or from an earlier pass): release the gate lifetime when nothing remains.
        match self.kernel.complete(id).await {
            Ok(()) => {
                self.notify_observers(id, scope.target_kind, ObservedPhase::Completed);
                DriveOutcome::Completed
            }
            Err(ErasureError::InvalidTransition) => DriveOutcome::Waiting,
            Err(e) => self.on_failure(job, e).await,
        }
    }

    /// One worker pass. Returns how long the worker may sleep (`None` = until notified).
    pub async fn run_pass(self: &Arc<Self>) -> Option<Duration> {
        let jobs = self.kernel.open_jobs();
        {
            let mut sched = self.sched.lock();
            let open: Vec<ErasureId> = jobs.iter().map(|j| j.id).collect();
            sched.retain(|id, _| open.contains(id));
        }
        if jobs.is_empty() {
            self.progress.notify_waiters();
            return None;
        }

        let now = self.clock.monotonic_nanos();
        let now_unix = self.clock.now_unix_nanos();
        let sla_ns = self.cfg.sla.as_nanos().min(u64::MAX as u128) as u64;
        let window_ns = self.cfg.coalesce_window.as_nanos().min(u64::MAX as u128) as u64;

        // Due time per job; SLA-endangered jobs skip the coalescing wait.
        let mut due: Vec<(u64, &JobView)> = Vec::with_capacity(jobs.len());
        let mut any_at_risk = false;
        for job in &jobs {
            if matches!(job.state, ErasureState::Quarantined { .. }) {
                continue;
            }
            let (first_seen, next_attempt) = {
                let mut sched = self.sched.lock();
                let e = sched.entry(job.id).or_insert(JobSched {
                    first_seen: now,
                    next_attempt: 0,
                    sla_reported: false,
                });
                (e.first_seen, e.next_attempt)
            };
            let mut at_risk = false;
            if let Some(st) = self.kernel.status(job.id) {
                let elapsed = now_unix.saturating_sub(st.requested_at);
                if sla_ns > 0 && elapsed >= sla_ns / 5 * 4 {
                    at_risk = true;
                }
                if sla_ns > 0 && elapsed >= sla_ns {
                    let mut sched = self.sched.lock();
                    if let Some(e) = sched.get_mut(&job.id) {
                        if !e.sla_reported {
                            e.sla_reported = true;
                            self.sla_breaches.fetch_add(1, Ordering::Relaxed);
                            tracing::error!("erasure SLA exceeded");
                        }
                    }
                }
            }
            any_at_risk |= at_risk;
            let base = if at_risk { 0 } else { first_seen.saturating_add(window_ns) };
            due.push((base.max(next_attempt), job));
        }
        self.budget.set_sla_boost(any_at_risk);

        let Some(leader) = due.iter().map(|(d, _)| *d).min() else {
            return None; // only quarantined jobs: wait for notify
        };
        if now < leader {
            return Some(Duration::from_nanos(leader - now).min(self.cfg.recheck_interval));
        }

        if self.budget.pressure() == PressureHint::Critical {
            // G3 pauses under critical pressure (G1 never goes through here).
            let jitter = self.rng.next_u64() % 250;
            return Some(Duration::from_millis(1_000 + jitter));
        }

        // One round for everything that becomes due inside the coalescing window.
        let batch: Vec<&JobView> = due
            .iter()
            .filter(|(d, _)| *d <= leader.saturating_add(window_ns))
            .map(|(_, j)| *j)
            .collect();

        if let Some(snaps) = &self.snapshots {
            let policy = contextra_mvcc::ReapPolicy {
                max_pin_age: self.cfg.max_pin_age,
                max_guard_age: None,
            };
            let _ = snaps.reap_expired(policy, now_unix);
        }

        for job in batch {
            if self.cancel.is_cancelled() {
                return None;
            }
            let outcome = self.drive_job(job).await;
            let now2 = self.clock.monotonic_nanos();
            let mut sched = self.sched.lock();
            match outcome {
                DriveOutcome::Retry(d) => {
                    if let Some(e) = sched.get_mut(&job.id) {
                        e.next_attempt = now2.saturating_add(d.as_nanos().min(u64::MAX as u128) as u64);
                    }
                }
                DriveOutcome::Completed | DriveOutcome::Quarantined => {
                    sched.remove(&job.id);
                }
                DriveOutcome::Waiting | DriveOutcome::Poisoned => {
                    // Nothing to do until the coarse recheck timer fires (no hot loop).
                    if let Some(e) = sched.get_mut(&job.id) {
                        let r = self.cfg.recheck_interval.as_nanos().min(u64::MAX as u128) as u64;
                        e.next_attempt = now2.saturating_add(r);
                    }
                }
                DriveOutcome::Paused => {}
            }
        }
        self.progress.notify_waiters();

        // Next wake-up: earliest remaining due time, but never later than the recheck interval.
        let now3 = self.clock.monotonic_nanos();
        let next = self
            .sched
            .lock()
            .values()
            .map(|s| s.next_attempt.max(s.first_seen.saturating_add(window_ns)))
            .min();
        let recheck = self.cfg.recheck_interval;
        match next {
            None => None,
            Some(t) => Some(Duration::from_nanos(t.saturating_sub(now3)).min(recheck).max(Duration::from_millis(1))),
        }
    }
}
