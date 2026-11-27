//! Integration and property tests for `contextra-ports` erasure contracts (TST-PORTS-03..09).

#![allow(deprecated, clippy::unwrap_used, clippy::expect_used)]

use contextra_ports::clock::SystemClock;
use contextra_ports::erasure::{
    compute_plan_hash, AcceptReceipt, Cancel, DocScope, EraseRequest, ErasureKernel, GateReceipt,
    IoBudget, IoClass, JobView, KeyPattern, KeyScope, PressureHint, PrivilegedRead, ProgressSink,
    PurgeCx, PurgeOutcome, ResolvedScope, SealReceipt, Verdict,
};
use contextra_ports::gate::{DiscardRule, GateDecision, RangeGate, ReadGate};
use contextra_ports::scanner::{
    LayerScanner, LayerWitness, ScanSession, ScanTarget, ScanTechnique, WitnessLayer,
};
use contextra_ports::{BoxFuture, RequestContext, TxId};
use bytes::Bytes;
use contextra_types::{
    DocId, EraseTarget, ErasureError, ErasureId, ErasurePolicy, ErasureReason, ErasureState,
    ErasureStatus, Guarantee, LayerErrorKind, LayerProgress, Seq, TargetKind, TenantId,
};
use futures_util::FutureExt;
use proptest::prelude::*;
use std::any::TypeId;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

/// TST-PORTS-03: KeyScope longest-prefix-wins, Exact-over-Prefix, and complement case.
#[test]
fn tst_ports_03_key_scope_longest_match_and_complement() {
    let scope = KeyScope::new(vec![
        (KeyPattern::Prefix(b"t:7:".to_vec()), Verdict::In),
        (KeyPattern::Prefix(b"t:7:__col:".to_vec()), Verdict::Out),
        (
            KeyPattern::Prefix(b"t:7:__txt:default:".to_vec()),
            Verdict::In,
        ),
        (KeyPattern::Exact(b"t:7:pinned".to_vec()), Verdict::Out),
        (KeyPattern::Prefix(b"t:7:pinned".to_vec()), Verdict::In),
    ])
    .expect("valid key scope");

    // Default tenant key -> In
    assert!(scope.contains(b"t:7:doc1"));
    // Named collection excluded -> Out
    assert!(!scope.contains(b"t:7:__col:other:doc2"));
    // Text index for default collection re-included -> In (longest match)
    assert!(scope.contains(b"t:7:__txt:default:term"));
    // Exact beats Prefix of equal length
    assert!(!scope.contains(b"t:7:pinned"));
    // Longer key with same prefix matches Prefix(In)
    assert!(scope.contains(b"t:7:pinned_extra"));
    // Unmatched tenant -> false
    assert!(!scope.contains(b"t:8:doc1"));
}

// TST-PORTS-04: `KeyScope::could_overlap_range` must have zero false negatives.
proptest! {
    #[test]
    fn tst_ports_04_could_overlap_range_no_false_negatives(
        rules_raw in prop::collection::vec(
            (prop::collection::vec(any::<u8>(), 0..8), any::<bool>(), any::<bool>()),
            0..12
        ),
        bounds in (prop::collection::vec(any::<u8>(), 0..8), prop::collection::vec(any::<u8>(), 0..8)),
        probe_keys in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..10), 1..32),
    ) {
        let (min, max) = if bounds.0 <= bounds.1 {
            (bounds.0, bounds.1)
        } else {
            (bounds.1, bounds.0)
        };

        // Build unambiguous ruleset by deduplicating patterns
        let mut seen = BTreeSet::new();
        let mut rules = Vec::new();
        for (bytes, is_exact, is_in) in rules_raw {
            let pat = if is_exact {
                KeyPattern::Exact(bytes)
            } else {
                KeyPattern::Prefix(bytes)
            };
            if seen.insert((matches!(pat, KeyPattern::Exact(_)), pat.bytes().to_vec())) {
                rules.push((pat, if is_in { Verdict::In } else { Verdict::Out }));
            }
        }

        let scope = KeyScope::new(rules).expect("valid proptest scope");
        let overlaps = scope.could_overlap_range(&min, &max);

        for key in probe_keys {
            let in_range = key.as_slice() >= min.as_slice() && key.as_slice() <= max.as_slice();
            if in_range && scope.contains(&key) {
                prop_assert!(
                    overlaps,
                    "false negative: key {:?} in [{:?}, {:?}] matched scope but could_overlap_range returned false",
                    key,
                    min,
                    max
                );
            }
        }
    }
}

/// TST-PORTS-05: `KeyScope::new` rejects > MAX_RULES (4096) and > MAX_BYTES (1 MiB).
#[test]
fn tst_ports_05_key_scope_limits() {
    let too_many_rules: Vec<(KeyPattern, Verdict)> = (0u32..=4096)
        .map(|i| (KeyPattern::Exact(i.to_le_bytes().to_vec()), Verdict::In))
        .collect();
    assert!(matches!(
        KeyScope::new(too_many_rules),
        Err(ErasureError::InvalidTarget(_))
    ));

    let too_many_bytes = vec![
        (KeyPattern::Prefix(vec![b'a'; KeyScope::MAX_BYTES]), Verdict::In),
        (KeyPattern::Exact(vec![b'b']), Verdict::In),
    ];
    assert!(matches!(
        KeyScope::new(too_many_bytes),
        Err(ErasureError::InvalidTarget(_))
    ));
}

/// TST-PORTS-06: `plan_hash` golden value and input-order independence.
#[test]
fn tst_ports_06_plan_hash_golden_and_order_independent() {
    let tenant = TenantId::try_new(7).expect("valid tenant");
    let target = EraseTarget::Document {
        tenant,
        collection: "docs".to_string(),
        key: "k1".to_string(),
    };

    let r1 = (KeyPattern::Exact(b"t:7:k1".to_vec()), Verdict::In);
    let r2 = (KeyPattern::Prefix(b"t:7:__txt:".to_vec()), Verdict::Out);

    let scope_a = KeyScope::new(vec![r1.clone(), r2.clone()]).expect("scope a");
    let scope_b = KeyScope::new(vec![r2, r1]).expect("scope b");

    let mut set_a = BTreeSet::new();
    set_a.insert(DocId::new(10));
    set_a.insert(DocId::new(20));
    let docs_a = vec![
        DocScope::Set {
            tenant,
            collection: "docs".to_string(),
            docs: set_a.clone(),
        },
        DocScope::Tenant { tenant },
    ];
    let docs_b = vec![
        DocScope::Tenant { tenant },
        DocScope::Set {
            tenant,
            collection: "docs".to_string(),
            docs: set_a,
        },
    ];

    let h1 = compute_plan_hash(&target, &scope_a, &docs_a, 3);
    let h2 = compute_plan_hash(&target, &scope_b, &docs_b, 3);
    assert_eq!(h1, h2, "plan_hash must be independent of rule/doc order");

    // Golden check: changing any field changes plan_hash deterministically.
    let h_diff = compute_plan_hash(&target, &scope_a, &docs_a, 4);
    assert_ne!(h1, h_diff);

    // Fix golden hash for canonical serialization format v1.
    let expected_golden: [u8; 32] = [
        175, 125, 74, 153, 108, 66, 190, 211, 197, 111, 7, 209, 76, 118, 218, 213, 96, 141, 100,
        91, 79, 112, 33, 252, 19, 231, 63, 84, 190, 144, 112, 127,
    ];
    assert_eq!(h1, expected_golden);
}

/// TST-PORTS-07: `Debug` of `ResolvedScope`, `KeyScope`, `KeyPattern`, `DocScope` never leaks key bytes.
#[test]
fn tst_ports_07_debug_redacts_key_bytes() {
    let secret_key = b"SUPER_SECRET_KEY_BYTES_12345";
    let secret_collection = "SECRET_COLLECTION_NAME";
    let tenant = TenantId::try_new(42).expect("valid tenant");

    let keys = KeyScope::new(vec![
        (KeyPattern::Exact(secret_key.to_vec()), Verdict::In),
        (KeyPattern::Prefix(secret_key.to_vec()), Verdict::Out),
    ])
    .expect("valid scope");

    let mut docs_set = BTreeSet::new();
    docs_set.insert(DocId::new(1));
    let docs = vec![DocScope::Set {
        tenant,
        collection: secret_collection.to_string(),
        docs: docs_set,
    }];

    let target = EraseTarget::Document {
        tenant,
        collection: secret_collection.to_string(),
        key: "SECRET_DOC_KEY".to_string(),
    };
    let resolved = ResolvedScope::new(
        ErasureId::new([1u8; 16]),
        TargetKind::Document,
        &target,
        keys.clone(),
        docs.clone(),
        0,
    );

    let dbg_keys = format!("{keys:?}");
    let dbg_resolved = format!("{resolved:?}");
    let dbg_docs = format!("{docs:?}");

    assert!(!dbg_keys.contains("SUPER_SECRET"));
    assert!(!dbg_resolved.contains("SUPER_SECRET"));
    assert!(!dbg_resolved.contains("SECRET_COLLECTION"));
    assert!(!dbg_docs.contains("SECRET_COLLECTION"));
}

/// TST-PORTS-08: Re-exported types match `contextra_types` `TypeId`.
#[test]
fn tst_ports_08_reexports_match_contextra_types() {
    assert_eq!(
        TypeId::of::<contextra_ports::Seq>(),
        TypeId::of::<contextra_types::Seq>()
    );
    assert_eq!(
        TypeId::of::<contextra_ports::gate::Seq>(),
        TypeId::of::<contextra_types::Seq>()
    );
    assert_eq!(
        TypeId::of::<contextra_ports::Timestamp>(),
        TypeId::of::<contextra_types::Timestamp>()
    );
    assert_eq!(
        TypeId::of::<contextra_ports::ledger::Timestamp>(),
        TypeId::of::<contextra_types::Timestamp>()
    );
    assert_eq!(
        TypeId::of::<contextra_ports::TargetKind>(),
        TypeId::of::<contextra_types::TargetKind>()
    );
    assert_eq!(
        TypeId::of::<contextra_ports::observer::TargetKind>(),
        TypeId::of::<contextra_types::TargetKind>()
    );
    assert_eq!(
        TypeId::of::<contextra_ports::erasure::EraseTarget>(),
        TypeId::of::<contextra_types::EraseTarget>()
    );
    assert_eq!(
        TypeId::of::<contextra_ports::erasure::ErasureError>(),
        TypeId::of::<contextra_types::ErasureError>()
    );
}

struct MockGate;
impl ReadGate for MockGate {
    fn is_active(&self) -> bool {
        false
    }
    fn epoch(&self) -> u64 {
        1
    }
    fn check(&self, _raw_key: &[u8]) -> GateDecision {
        GateDecision::Open
    }
    fn check_version(&self, _raw_key: &[u8], _version_seq: Seq) -> GateDecision {
        GateDecision::Open
    }
    fn check_prefix(&self, _prefix: &[u8]) -> RangeGate {
        RangeGate::EntirelyOpen
    }
    fn discard_rule(&self, _raw_key: &[u8], _version_seq: Seq) -> DiscardRule {
        DiscardRule::Keep
    }
    fn check_doc(&self, _tenant: TenantId, _collection: &str, _doc: DocId) -> GateDecision {
        GateDecision::Open
    }
    fn gated_seq(&self, _raw_key: &[u8]) -> Option<Seq> {
        None
    }
}

struct MockKernel {
    jobs: Mutex<BTreeMap<ErasureId, (ErasureState, Arc<ResolvedScope>)>>,
}

impl MockKernel {
    fn new() -> Self {
        Self {
            jobs: Mutex::new(BTreeMap::new()),
        }
    }
}

impl ErasureKernel for MockKernel {
    fn accept<'a>(
        &'a self,
        req: &'a EraseRequest,
        plan: &'a ResolvedScope,
    ) -> BoxFuture<'a, Result<AcceptReceipt, ErasureError>> {
        Box::pin(async move {
            let mut jobs = self.jobs.lock().unwrap();
            if let Some((_, existing_plan)) = jobs.get(&req.id) {
                if existing_plan.plan_hash != plan.plan_hash {
                    return Err(ErasureError::IdempotencyConflict);
                }
                return Ok(AcceptReceipt {
                    id: req.id,
                    existing: true,
                });
            }
            jobs.insert(req.id, (ErasureState::Planned, Arc::new(plan.clone())));
            Ok(AcceptReceipt {
                id: req.id,
                existing: false,
            })
        })
    }

    fn publish_gate<'a>(&'a self, id: ErasureId) -> BoxFuture<'a, Result<GateReceipt, ErasureError>> {
        Box::pin(async move {
            let mut jobs = self.jobs.lock().unwrap();
            let (state, _) = jobs.get_mut(&id).ok_or(ErasureError::NotFound)?;
            if !state.can_transition_to(&ErasureState::Gating) {
                return Err(ErasureError::InvalidTransition);
            }
            *state = ErasureState::Gated;
            Ok(GateReceipt {
                id,
                gated_seq: Seq(10),
                epoch: 1,
            })
        })
    }

    fn purge_store<'a>(
        &'a self,
        id: ErasureId,
        _cx: &'a PurgeCx<'a>,
    ) -> BoxFuture<'a, Result<PurgeOutcome, ErasureError>> {
        Box::pin(async move {
            let mut jobs = self.jobs.lock().unwrap();
            let (state, _) = jobs.get_mut(&id).ok_or(ErasureError::NotFound)?;
            if !matches!(state, ErasureState::Gated | ErasureState::Purging) {
                return Err(ErasureError::InvalidTransition);
            }
            *state = ErasureState::Purging;
            Ok(PurgeOutcome {
                progress: LayerProgress {
                    cursor: None,
                    units_done: 1,
                    bytes_freed: 64,
                },
                units_committed: 1,
                bytes_freed: 64,
            })
        })
    }

    fn record_layer<'a>(
        &'a self,
        id: ErasureId,
        _layer: &'a WitnessLayer,
        _p: &'a LayerProgress,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move {
            let jobs = self.jobs.lock().unwrap();
            if !jobs.contains_key(&id) {
                return Err(ErasureError::NotFound);
            }
            Ok(())
        })
    }

    fn seal<'a>(
        &'a self,
        id: ErasureId,
        w: &'a [LayerWitness],
    ) -> BoxFuture<'a, Result<SealReceipt, ErasureError>> {
        Box::pin(async move {
            let mut jobs = self.jobs.lock().unwrap();
            let (state, _) = jobs.get_mut(&id).ok_or(ErasureError::NotFound)?;
            if !matches!(state, ErasureState::Purging | ErasureState::Verifying) {
                return Err(ErasureError::InvalidTransition);
            }
            let remaining_total: u64 = w.iter().map(|lw| lw.remaining()).sum();
            if remaining_total > 0 {
                return Err(ErasureError::InvalidTransition);
            }
            *state = ErasureState::Sealed;
            Ok(SealReceipt {
                id,
                guarantee: Guarantee::new(true, true, false)?,
                exceptions: vec![],
                ledger_index: Some(0),
            })
        })
    }

    fn complete<'a>(&'a self, id: ErasureId) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move {
            let mut jobs = self.jobs.lock().unwrap();
            let (state, _) = jobs.get_mut(&id).ok_or(ErasureError::NotFound)?;
            if !state.can_transition_to(&ErasureState::Completed) {
                return Err(ErasureError::InvalidTransition);
            }
            *state = ErasureState::Completed;
            Ok(())
        })
    }

    fn fail<'a>(
        &'a self,
        id: ErasureId,
        layer: Option<&'a WitnessLayer>,
        _kind: LayerErrorKind,
    ) -> BoxFuture<'a, Result<ErasureState, ErasureError>> {
        Box::pin(async move {
            let mut jobs = self.jobs.lock().unwrap();
            let (state, _) = jobs.get_mut(&id).ok_or(ErasureError::NotFound)?;
            let next = ErasureState::Failed {
                layer: layer.map(|l| l.as_str().to_string()),
                attempt: 1,
            };
            *state = next.clone();
            Ok(next)
        })
    }

    fn open_jobs(&self) -> Vec<JobView> {
        self.jobs
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, (st, _))| !st.is_terminal())
            .map(|(id, (st, scope))| JobView {
                id: *id,
                state: st.clone(),
                scope: Arc::clone(scope),
                layers_done: vec![],
                attempt: 0,
            })
            .collect()
    }

    fn status(&self, id: ErasureId) -> Option<ErasureStatus> {
        self.jobs.lock().unwrap().get(&id).map(|(st, _)| ErasureStatus {
            id,
            state: st.clone(),
            reached: st.reached(),
            layers: vec![],
            exceptions: vec![],
            requested_at: 0,
            gated_at: None,
            sealed_at: None,
            ledger_index: None,
        })
    }

    fn privileged_reader(&self, _id: ErasureId) -> Result<Arc<dyn PrivilegedRead>, ErasureError> {
        Err(ErasureError::NotFound)
    }

    fn gate(&self) -> Arc<dyn ReadGate> {
        Arc::new(MockGate)
    }

    fn store_scanners(&self) -> Vec<Arc<dyn LayerScanner>> {
        vec![]
    }
}

struct DummyCancel;
impl Cancel for DummyCancel {
    fn is_cancelled(&self) -> bool {
        false
    }
    fn cancelled(&self) -> BoxFuture<'_, ()> {
        Box::pin(futures_util::future::pending())
    }
}

struct DummyBudget;
impl IoBudget for DummyBudget {
    fn acquire<'a>(&'a self, _class: IoClass, _bytes: u64) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async { Ok(()) })
    }
    fn pressure(&self) -> PressureHint {
        PressureHint::Normal
    }
}

struct DummyReader;
impl PrivilegedRead for DummyReader {
    fn get<'a>(&'a self, _raw_key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>, ErasureError>> {
        Box::pin(async { Ok(None) })
    }
    fn scan_prefix<'a>(
        &'a self,
        _prefix: &'a [u8],
        _after: Option<&'a [u8]>,
        _limit: usize,
    ) -> BoxFuture<'a, Result<Vec<(Bytes, Bytes)>, ErasureError>> {
        Box::pin(async { Ok(vec![]) })
    }
}

struct DummyProgress;
impl ProgressSink for DummyProgress {
    fn record<'a>(
        &'a self,
        _layer: &'a WitnessLayer,
        _p: &'a LayerProgress,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async { Ok(()) })
    }
}

struct DummyScanner;
impl LayerScanner for DummyScanner {
    fn layer(&self) -> WitnessLayer {
        WitnessLayer::LsmMemtable
    }
    fn scan_units(&self, session: &mut ScanSession, _ctx: &RequestContext) -> contextra_ports::Result<()> {
        session.record_control_match();
        session.record_unit([1u8; 32], false);
        Ok(())
    }
}

/// TST-PORTS-09: Mock `ErasureKernel` state transition contract.
#[test]
fn tst_ports_09_mock_erasure_kernel_transitions() {
    let kernel = MockKernel::new();
    let tenant = TenantId::try_new(1).expect("valid tenant");
    let target = EraseTarget::Tenant { tenant };
    let id = ErasureId::new([9u8; 16]);
    let scope = ResolvedScope::new(
        id,
        TargetKind::Tenant,
        &target,
        KeyScope::new(vec![(KeyPattern::Prefix(b"t:1:".to_vec()), Verdict::In)]).unwrap(),
        vec![DocScope::Tenant { tenant }],
        0,
    );
    let req = EraseRequest {
        id,
        target: target.clone(),
        reason: ErasureReason::UserRequest,
        policy: ErasurePolicy::default(),
    };

    // Accept is idempotent for identical plan_hash.
    let r1 = kernel.accept(&req, &scope).now_or_never().unwrap().unwrap();
    assert!(!r1.existing);
    let r2 = kernel.accept(&req, &scope).now_or_never().unwrap().unwrap();
    assert!(r2.existing);

    // Diverging plan_hash returns IdempotencyConflict.
    let scope_diff = ResolvedScope::new(
        id,
        TargetKind::Tenant,
        &target,
        KeyScope::default(),
        vec![],
        1,
    );
    assert!(matches!(
        kernel.accept(&req, &scope_diff).now_or_never().unwrap(),
        Err(ErasureError::IdempotencyConflict)
    ));

    // Cannot complete before Sealed.
    assert!(matches!(
        kernel.complete(id).now_or_never().unwrap(),
        Err(ErasureError::InvalidTransition)
    ));

    // Gate -> Purge -> Seal -> Complete.
    let gate_rcpt = kernel.publish_gate(id).now_or_never().unwrap().unwrap();
    assert_eq!(gate_rcpt.epoch, 1);
    assert_eq!(kernel.open_jobs().len(), 1);
    assert_eq!(kernel.status(id).unwrap().state, ErasureState::Gated);

    // Seal before Purging fails.
    assert!(matches!(
        kernel.seal(id, &[]).now_or_never().unwrap(),
        Err(ErasureError::InvalidTransition)
    ));

    let clock = SystemClock::new();
    let cancel = DummyCancel;
    let budget = DummyBudget;
    let reader = DummyReader;
    let progress = DummyProgress;
    let cx = PurgeCx {
        erasure_id: id,
        clock: &clock,
        cancel: &cancel,
        budget: &budget,
        cursor: None,
        reader: &reader,
        progress: &progress,
        max_units: 100,
    };
    let outcome = kernel.purge_store(id, &cx).now_or_never().unwrap().unwrap();
    assert!(outcome.progress.cursor.is_none());

    let scanner = DummyScanner;
    let ctx = RequestContext::new(tenant, TxId(1));
    let witness = scanner
        .scan(&ctx, ScanTarget::new("t1", [0u8; 32]), ScanTechnique::Structural, &clock)
        .unwrap();
    let seal_rcpt = kernel.seal(id, &[witness]).now_or_never().unwrap().unwrap();
    assert!(seal_rcpt.guarantee.gated && seal_rcpt.guarantee.purged);

    kernel.complete(id).now_or_never().unwrap().unwrap();
    assert!(kernel.open_jobs().is_empty());
    assert_eq!(kernel.status(id).unwrap().state, ErasureState::Completed);
}
