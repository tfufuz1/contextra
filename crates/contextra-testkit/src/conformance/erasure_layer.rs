//! # Konformanzsuite C1–C7 für [`ErasureLayer`] (`conformance/erasure_layer`)
//!
//! Jeder Fall ist eine `async fn`, die eine frische Fixture über [`CrashVfs`] aufbaut.
//! [`crate::assert_layer_conformance!`] erzeugt daraus einzelne `#[tokio::test]`s.
//!
//! `K1` begrenzt die Arbeit von `on_gate` über die Zahl der Polls: der Port reicht
//! `on_gate` keine Uhr, und Wall-Clock-Messungen sind hier unzulässig.

use crate::conformance::{ConformanceFailure, ConformanceResult};
use crate::crash_vfs::{CrashSeed, CrashVfs};
use crate::erasure_mocks::{ManualIoBudget, MockProgressSink};
use crate::manual_clock::ManualClock;
use contextra_ports::erasure::{
    Cancel, ErasureLayer, LayerError, PrivilegedRead, PurgeCx, PurgeOutcome, ResolvedScope,
};
use contextra_ports::gate::ReadGate;
use contextra_ports::scanner::{LayerWitness, ScanTechnique};
use contextra_ports::{BoxFuture, DocScope};
use contextra_types::{Cursor, DocId, ErasureId, LayerErrorKind, RequestContext, TenantId, TxId};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

/// Zu injizierender Fehler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultSpec {
    /// Transienter I/O-Fehler.
    Io,
    /// Korrupte Daten.
    Corrupt,
    /// Zeitüberschreitung (z. B. Lock).
    Timeout,
    /// Der Scanner findet seine Positivkontrolle nicht.
    BrokenScanner,
}

impl FaultSpec {
    /// Erwartete Klassifikation (Spezifikation 06 §5.5) mit Detailtext.
    pub fn expected_error(self) -> (LayerErrorKind, &'static str) {
        match self {
            Self::Io => (LayerErrorKind::Retryable, "injected_io"),
            Self::Timeout => (LayerErrorKind::Retryable, "injected_timeout"),
            Self::Corrupt => (LayerErrorKind::Permanent, "injected_corrupt"),
            Self::BrokenScanner => (LayerErrorKind::Permanent, "injected_scanner"),
        }
    }
}

/// Fixture-Vertrag einer zu prüfenden Schicht.
///
/// Gegenüber 13 §5.1 sind `neighbor_docs`, `state_digest`, `foreign_key` und
/// `direct_blocked_reads` ergänzt: ohne sie sind C1/C2, C3 und C7 nicht prüfbar.
pub trait LayerFixture: Send + Sync {
    /// Geprüfte Schicht.
    type Layer: ErasureLayer;
    /// Neue Fixture über `vfs`.
    fn new(vfs: CrashVfs) -> Self;
    /// Die Schicht.
    fn layer(&self) -> &Self::Layer;
    /// Füllt Scope-Dokumente und Nachbarn deterministisch.
    fn seed(&self, n_scope: usize, n_neighbor: usize);
    /// Aufgelöster Scope. Muss die Scope-Dokumente als [`DocScope::Set`] führen.
    fn scope(&self) -> ResolvedScope;
    /// Nachbardokumente.
    fn neighbor_docs(&self) -> Vec<DocId>;
    /// Kanonischer Digest aller Zustände außerhalb des Scopes.
    fn neighbor_digest(&self) -> [u8; 32];
    /// Kanonischer Digest des gesamten durablen Schichtzustands.
    fn state_digest(&self) -> [u8; 32];
    /// Setzt ein Restobjekt des Scopes absichtlich zurück.
    fn plant_residue(&self);
    /// Fehlerinjektion.
    fn inject_fault(&self, f: FaultSpec);
    /// Scope-prüfender Reader.
    fn reader(&self) -> Arc<dyn PrivilegedRead>;
    /// Existierender Schlüssel außerhalb des Scopes.
    fn foreign_key(&self) -> Vec<u8>;
    /// Gate der Fixture.
    fn gate(&self) -> Arc<dyn ReadGate>;
    /// Suchpfad der Schicht.
    fn is_visible(&self, doc: DocId) -> bool;
    /// Zugriffe der Schicht auf gesperrte Schlüssel am Reader vorbei.
    fn direct_blocked_reads(&self) -> u64;
}

const N_SCOPE: usize = 7;
const N_NEIGHBOR: usize = 5;
const MAX_UNITS: usize = 2;
const MAX_ROUNDS: usize = 1_000;
const MAX_CANCEL_POINTS: usize = 10_000;
const MAX_GATE_POLLS: usize = 16;

/// Token, das beim `k+1`-ten `is_cancelled`-Aufruf feuert (`None` = nie).
struct CountdownCancel {
    after: Option<usize>,
    polls: AtomicUsize,
}

impl CountdownCancel {
    fn new(after: Option<usize>) -> Self {
        Self {
            after,
            polls: AtomicUsize::new(0),
        }
    }

    fn fired(&self) -> bool {
        self.after
            .is_some_and(|k| self.polls.load(Ordering::SeqCst) > k)
    }
}

impl Cancel for CountdownCancel {
    fn is_cancelled(&self) -> bool {
        self.polls.fetch_add(1, Ordering::SeqCst);
        self.fired()
    }

    fn cancelled(&self) -> BoxFuture<'_, ()> {
        Box::pin(std::future::poll_fn(move |cx| {
            if self.is_cancelled() {
                Poll::Ready(())
            } else {
                if self.after.is_some() {
                    cx.waker().wake_by_ref();
                }
                Poll::Pending
            }
        }))
    }
}

/// Pollt höchstens `left` Mal; danach `None` (das Future wird beim Drop verworfen).
struct PollLimit<F> {
    inner: Pin<Box<F>>,
    left: usize,
}

impl<F: Future> Future for PollLimit<F> {
    type Output = Option<F::Output>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.left == 0 {
            return Poll::Ready(None);
        }
        self.left -= 1;
        match self.inner.as_mut().poll(cx) {
            Poll::Ready(v) => Poll::Ready(Some(v)),
            Poll::Pending => {
                // Nach dem letzten erlaubten Poll muss der Abbruch selbst geweckt werden.
                if self.left == 0 {
                    cx.waker().wake_by_ref();
                }
                Poll::Pending
            }
        }
    }
}

fn limit<F: Future>(inner: F, left: usize) -> PollLimit<F> {
    PollLimit {
        inner: Box::pin(inner),
        left,
    }
}

fn fail(case: &'static str, message: impl Into<String>) -> ConformanceFailure {
    ConformanceFailure::new("ErasureLayer", case, message)
}

fn ensure(case: &'static str, ok: bool, message: &str) -> ConformanceResult {
    if ok {
        Ok(())
    } else {
        Err(fail(case, message))
    }
}

fn scope_docs(s: &ResolvedScope) -> Vec<DocId> {
    s.docs
        .iter()
        .filter_map(|d| match d {
            DocScope::Set { docs, .. } => Some(docs.iter().copied()),
            _ => None,
        })
        .flatten()
        .collect()
}

struct Harness<F: LayerFixture> {
    fix: F,
    scope: ResolvedScope,
}

impl<F: LayerFixture> Harness<F> {
    fn new(make: &impl Fn(CrashVfs) -> F, case: &'static str) -> Result<Self, ConformanceFailure> {
        let fix = make(CrashVfs::new(CrashSeed(0x5EED)));
        fix.seed(N_SCOPE, N_NEIGHBOR);
        let scope = fix.scope();
        let docs = scope_docs(&scope);
        ensure(
            case,
            docs.len() == N_SCOPE && fix.neighbor_docs().len() == N_NEIGHBOR,
            "fixture precondition: seed() did not create the requested documents",
        )?;
        ensure(
            case,
            docs.iter()
                .chain(&fix.neighbor_docs())
                .all(|d| fix.is_visible(*d)),
            "fixture precondition: seeded documents must be visible",
        )?;
        Ok(Self { fix, scope })
    }

    async fn gate(&self, case: &'static str) -> ConformanceResult {
        self.fix
            .layer()
            .on_gate(&self.scope)
            .await
            .map_err(|e| fail(case, format!("on_gate failed: {e}")))
    }

    /// Ein einzelner `purge`-Aufruf.
    async fn step(
        &self,
        cursor: Option<Cursor>,
        cancel: &dyn Cancel,
    ) -> Result<PurgeOutcome, LayerError> {
        let clock = ManualClock::new(1);
        let budget = ManualIoBudget::unlimited();
        let progress = MockProgressSink::new();
        let reader = self.fix.reader();
        let cx = PurgeCx {
            erasure_id: ErasureId::from_u128(1),
            clock: &clock,
            cancel,
            budget: &budget,
            cursor,
            reader: reader.as_ref(),
            progress: &progress,
            max_units: MAX_UNITS,
        };
        self.fix.layer().purge(&self.scope, &cx).await
    }

    /// Treibt `purge` ab `cursor` bis `cursor == None`; liefert alle Zwischencursor und
    /// die Summe der committeten Einheiten.
    async fn drive(
        &self,
        case: &'static str,
        mut cursor: Option<Cursor>,
    ) -> Result<(Vec<Cursor>, u64), ConformanceFailure> {
        let mut cursors = Vec::new();
        let mut units = 0u64;
        for _ in 0..MAX_ROUNDS {
            let out = self
                .step(cursor.take(), &CountdownCancel::new(None))
                .await
                .map_err(|e| fail(case, format!("purge failed: {e}")))?;
            units = units.saturating_add(out.units_committed);
            match out.progress.cursor {
                Some(c) => {
                    cursors.push(c.clone());
                    cursor = Some(c);
                }
                None => return Ok((cursors, units)),
            }
        }
        Err(fail(case, "purge does not terminate"))
    }

    fn witness(&self) -> contextra_types::Result<LayerWitness> {
        let clock = ManualClock::new(1);
        let ctx = RequestContext::new(TenantId::try_new(1)?, TxId::new(1));
        #[allow(deprecated)]
        let target = contextra_ports::scanner::ScanTarget::new("conformance", [7; 32]);
        self.fix
            .layer()
            .scanner()
            .scan(&ctx, target, ScanTechnique::Structural, &clock)
    }

    /// Endzustand: nichts aus dem Scope sichtbar, Witness ohne Rest.
    fn ensure_purged(&self, case: &'static str) -> ConformanceResult {
        ensure(
            case,
            scope_docs(&self.scope)
                .iter()
                .all(|d| !self.fix.is_visible(*d)),
            "scope document visible after purge",
        )?;
        let w = self
            .witness()
            .map_err(|e| fail(case, format!("scan failed: {e}")))?;
        ensure(
            case,
            w.remaining() == 0,
            "witness reports residue after purge",
        )
    }
}

/// Endzustands-Digest eines ununterbrochenen Referenzlaufs.
async fn reference<F: LayerFixture>(
    make: &impl Fn(CrashVfs) -> F,
    case: &'static str,
) -> Result<(Vec<Cursor>, [u8; 32]), ConformanceFailure> {
    let h = Harness::new(make, case)?;
    h.gate(case).await?;
    let before = h.fix.state_digest();
    let (cursors, units) = h.drive(case, None).await?;
    h.ensure_purged(case)?;
    ensure(
        case,
        units > 0 && h.fix.state_digest() != before,
        "purge had no observable effect",
    )?;
    Ok((cursors, h.fix.state_digest()))
}

/// C1: Idempotenz.
pub async fn c1_idempotence<F: LayerFixture>(make: impl Fn(CrashVfs) -> F) -> ConformanceResult {
    const C: &str = "C1";
    let h = Harness::new(&make, C)?;
    h.gate(C).await?;
    h.drive(C, None).await?;
    let after_first = h.fix.state_digest();
    let (_, units) = h.drive(C, None).await?;
    ensure(C, units == 0, "second run committed units")?;
    ensure(
        C,
        h.fix.state_digest() == after_first,
        "second run changed the state",
    )?;
    h.ensure_purged(C)
}

/// C2: Wiederaufnahme von jedem Cursor, auch in einer Fixture ohne Vorgeschichte im RAM.
pub async fn c2_resumption<F: LayerFixture>(make: impl Fn(CrashVfs) -> F) -> ConformanceResult {
    const C: &str = "C2";
    let (cursors, expected) = reference(&make, C).await?;
    ensure(
        C,
        !cursors.is_empty(),
        "purge ignored max_units: no intermediate cursor",
    )?;
    for (i, cursor) in cursors.iter().enumerate() {
        let h = Harness::new(&make, C)?;
        h.gate(C).await?;
        let mut current = None;
        for _ in 0..=i {
            current = h
                .step(current.take(), &CountdownCancel::new(None))
                .await
                .map_err(|e| fail(C, format!("purge failed: {e}")))?
                .progress
                .cursor;
        }
        ensure(
            C,
            current.as_ref() == Some(cursor),
            "cursor sequence differs",
        )?;
        h.drive(C, Some(cursor.clone())).await?;
        h.ensure_purged(C)?;
        ensure(
            C,
            h.fix.state_digest() == expected,
            "resumed run differs from uninterrupted run",
        )?;
    }
    Ok(())
}

/// C3: Scope-Isolation.
pub async fn c3_scope_isolation<F: LayerFixture>(
    make: impl Fn(CrashVfs) -> F,
) -> ConformanceResult {
    const C: &str = "C3";
    let h = Harness::new(&make, C)?;
    let before = h.fix.neighbor_digest();
    h.gate(C).await?;
    h.drive(C, None).await?;
    ensure(
        C,
        h.fix.neighbor_digest() == before,
        "neighbor state changed",
    )?;
    ensure(
        C,
        h.fix.neighbor_docs().iter().all(|d| h.fix.is_visible(*d)),
        "neighbor no longer visible",
    )?;
    let denied = h.fix.reader().get(&h.fix.foreign_key()).await;
    ensure(
        C,
        matches!(&denied, Err(e) if e.kind() == LayerErrorKind::ScopeViolation
            || matches!(e, contextra_types::ErasureError::Layer { kind: LayerErrorKind::ScopeViolation, .. })),
        "reader allowed access outside the scope",
    )
}

/// C4: Witness-Ehrlichkeit.
pub async fn c4_witness_honesty<F: LayerFixture>(
    make: impl Fn(CrashVfs) -> F,
) -> ConformanceResult {
    const C: &str = "C4";
    let h = Harness::new(&make, C)?;
    h.gate(C).await?;
    let dirty = h
        .witness()
        .map_err(|e| fail(C, format!("scan before purge failed: {e}")))?;
    ensure(C, dirty.remaining() > 0, "scanner blind before purge")?;
    h.drive(C, None).await?;
    h.ensure_purged(C)?;
    h.fix.plant_residue();
    let planted = h
        .witness()
        .map_err(|e| fail(C, format!("scan with residue failed: {e}")))?;
    ensure(C, planted.remaining() > 0, "planted residue not reported")?;
    h.fix.inject_fault(FaultSpec::BrokenScanner);
    match h.witness() {
        Err(_) => Ok(()),
        Ok(w) => ensure(
            C,
            matches!(w.technique(), ScanTechnique::NotMeasured(_)),
            "broken scanner still minted a measured witness",
        ),
    }
}

/// C5: Fehlerinjektion.
pub async fn c5_fault_injection<F: LayerFixture>(
    make: impl Fn(CrashVfs) -> F,
) -> ConformanceResult {
    const C: &str = "C5";
    for fault in [FaultSpec::Io, FaultSpec::Corrupt, FaultSpec::Timeout] {
        let h = Harness::new(&make, C)?;
        h.gate(C).await?;
        h.fix.inject_fault(fault);
        let (kind, _) = fault.expected_error();
        match h.step(None, &CountdownCancel::new(None)).await {
            Ok(_) => return Err(fail(C, format!("{fault:?}: returned Ok"))),
            Err(e) => ensure(
                C,
                e.kind == kind,
                &format!("{fault:?}: expected {kind:?}, got {:?}", e.kind),
            )?,
        }
    }
    Ok(())
}

/// C6: Abbruchsicherheit — kooperativ (Token) und durch Verwerfen des Futures.
pub async fn c6_cancel_safety<F: LayerFixture>(make: impl Fn(CrashVfs) -> F) -> ConformanceResult {
    const C: &str = "C6";
    let (_, expected) = reference(&make, C).await?;
    for drop_future in [false, true] {
        let mut completed = false;
        for k in 0..MAX_CANCEL_POINTS {
            let h = Harness::new(&make, C)?;
            h.gate(C).await?;
            // Unterbrochener Lauf; `cursor` ist der letzte bestätigte Cursor.
            let token = CountdownCancel::new((!drop_future).then_some(k));
            let mut cursor = None;
            let run = async {
                loop {
                    match h.step(cursor.clone(), &token).await {
                        Ok(out) => match out.progress.cursor {
                            Some(c) => cursor = Some(c),
                            None => return Ok(()),
                        },
                        Err(e) => return Err(e),
                    }
                }
            };
            let interrupted = if drop_future {
                match limit(run, k).await {
                    None => true,
                    Some(Ok(())) => false,
                    Some(Err(e)) => return Err(fail(C, format!("purge failed: {e}"))),
                }
            } else {
                match run.await {
                    Ok(()) => token.fired(),
                    Err(e) if e.kind == LayerErrorKind::Cancelled && token.fired() => true,
                    Err(e) => return Err(fail(C, format!("unexpected error on cancel: {e}"))),
                }
            };
            h.drive(C, cursor).await?;
            h.ensure_purged(C)?;
            ensure(
                C,
                h.fix.state_digest() == expected,
                &format!("k={k} drop={drop_future}: final state differs after cancel"),
            )?;
            if !interrupted {
                completed = true;
                break;
            }
        }
        ensure(C, completed, "no cancel-free run within the poll bound")?;
    }
    Ok(())
}

/// C7: Gate-Respekt.
pub async fn c7_gate_respect<F: LayerFixture>(make: impl Fn(CrashVfs) -> F) -> ConformanceResult {
    const C: &str = "C7";
    let h = Harness::new(&make, C)?;
    h.gate(C).await?;
    ensure(
        C,
        scope_docs(&h.scope).iter().all(|d| !h.fix.is_visible(*d)),
        "scope document visible after on_gate",
    )?;
    ensure(
        C,
        h.fix.neighbor_docs().iter().all(|d| h.fix.is_visible(*d)),
        "neighbor hidden by on_gate",
    )?;
    h.drive(C, None).await?;
    ensure(
        C,
        h.fix.direct_blocked_reads() == 0,
        "layer read gated keys around cx.reader",
    )
}

/// K1: `on_gate` ist idempotent und kurz.
pub async fn k1_gate_idempotent<F: LayerFixture>(
    make: impl Fn(CrashVfs) -> F,
) -> ConformanceResult {
    const C: &str = "K1";
    let h = Harness::new(&make, C)?;
    h.gate(C).await?;
    let once = h.fix.state_digest();
    for _ in 0..2 {
        match limit(h.fix.layer().on_gate(&h.scope), MAX_GATE_POLLS).await {
            Some(Ok(())) => {}
            Some(Err(e)) => return Err(fail(C, format!("repeated on_gate failed: {e}"))),
            None => return Err(fail(C, "on_gate exceeded its poll budget")),
        }
    }
    ensure(
        C,
        h.fix.state_digest() == once,
        "repeated on_gate changed the state",
    )?;
    ensure(
        C,
        scope_docs(&h.scope).iter().all(|d| !h.fix.is_visible(*d)),
        "repeated on_gate re-exposed documents",
    )
}

/// K2: Zwei Läufe liefern identische Cursorfolgen und Endzustände.
pub async fn k2_determinism<F: LayerFixture>(make: impl Fn(CrashVfs) -> F) -> ConformanceResult {
    const C: &str = "K2";
    let first = reference(&make, C).await?;
    let second = reference(&make, C).await?;
    ensure(C, first == second, "two identical runs differ")
}
