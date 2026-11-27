//! ReadGate-Implementierung (`StoreGate`, `GateSnapshot`, `KeyIndex`, `DocIndex`) (§6).
//!
//! Synchron, I/O-frei und allokationsfrei auf dem `Open`-Fast-Path.

use ahash::{AHashMap, RandomState};
use contextra_ports::erasure::{DocScope, KeyPattern, KeyScope, Verdict};
use contextra_ports::gate::{DiscardRule, GateDecision, RangeGate, ReadGate};
use contextra_types::{DocId, ErasureId, Seq, TenantId};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

const GATE_HASH_SEED_0: u64 = 0x517c_c1b7_2722_0a95;
const GATE_HASH_SEED_1: u64 = 0x9e37_79b9_7f4a_7c15;
const GATE_HASH_SEED_2: u64 = 0x243f_6a88_85a3_08d3;
const GATE_HASH_SEED_3: u64 = 0x1319_8a2e_0370_7344;

const MAX_PREFIX_LENGTHS: usize = 64;

#[inline]
fn fixed_hasher() -> RandomState {
    RandomState::with_seeds(
        GATE_HASH_SEED_0,
        GATE_HASH_SEED_1,
        GATE_HASH_SEED_2,
        GATE_HASH_SEED_3,
    )
}

/// Phase eines aktiven Gate-Eintrags (§6.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum GatePhase {
    Restricted,
    Erased,
}

/// Unveränderlicher Gate-Zustand je aktivem Löschauftrag.
#[derive(Clone, Debug)]
pub(crate) struct JobGate {
    pub id: ErasureId,
    pub scope: Arc<KeyScope>,
    pub docs: Vec<DocScope>,
    pub up_to: Seq,
    pub phase: GatePhase,
    pub reinsert_after_sealed: bool,
    pub sealed: bool,
}

/// Kompakte Menge von Job-Indizes innerhalb eines `GateSnapshot`.
#[derive(Clone, Default, Debug)]
struct SmallJobSet {
    indices: Vec<usize>,
}

impl SmallJobSet {
    fn insert(&mut self, idx: usize) {
        if !self.indices.contains(&idx) {
            self.indices.push(idx);
        }
    }
}

/// O(Präfixlängen)-Schlüsselindex für `KeyScope` (§6.1.1).
#[derive(Clone, Debug)]
pub(crate) struct KeyIndex {
    exact: AHashMap<Box<[u8]>, SmallJobSet, RandomState>,
    by_len: BTreeMap<usize, AHashMap<Box<[u8]>, SmallJobSet, RandomState>>,
    fallback_linear: bool,
}

impl Default for KeyIndex {
    fn default() -> Self {
        Self {
            exact: AHashMap::with_hasher(fixed_hasher()),
            by_len: BTreeMap::new(),
            fallback_linear: false,
        }
    }
}

impl KeyIndex {
    fn build(jobs: &[Arc<JobGate>]) -> Self {
        let mut idx = Self::default();
        for (job_idx, job) in jobs.iter().enumerate() {
            for (pat, verdict) in job.scope.rules() {
                if *verdict != Verdict::In {
                    continue;
                }
                match pat {
                    KeyPattern::Exact(b) => {
                        idx.exact
                            .entry(b.clone().into_boxed_slice())
                            .or_default()
                            .insert(job_idx);
                    }
                    KeyPattern::Prefix(p) => {
                        let map = idx
                            .by_len
                            .entry(p.len())
                            .or_insert_with(|| AHashMap::with_hasher(fixed_hasher()));
                        map.entry(p.clone().into_boxed_slice())
                            .or_default()
                            .insert(job_idx);
                    }
                }
            }
        }
        if idx.by_len.len() > MAX_PREFIX_LENGTHS {
            idx.fallback_linear = true;
        }
        idx
    }

    fn matching_jobs<'a>(&self, jobs: &'a [Arc<JobGate>], raw_key: &[u8]) -> Vec<&'a Arc<JobGate>> {
        if jobs.is_empty() {
            return Vec::new();
        }
        if self.fallback_linear {
            return jobs.iter().filter(|j| j.scope.contains(raw_key)).collect();
        }

        let mut candidate_indices: Vec<usize> = Vec::new();
        if let Some(set) = self.exact.get(raw_key) {
            for &i in &set.indices {
                if !candidate_indices.contains(&i) {
                    candidate_indices.push(i);
                }
            }
        }
        for (&len, map) in &self.by_len {
            if len > raw_key.len() {
                break;
            }
            if let Some(prefix_slice) = raw_key.get(..len) {
                if let Some(set) = map.get(prefix_slice) {
                    for &i in &set.indices {
                        if !candidate_indices.contains(&i) {
                            candidate_indices.push(i);
                        }
                    }
                }
            }
        }

        let mut matched = Vec::with_capacity(candidate_indices.len());
        for i in candidate_indices {
            if let Some(job) = jobs.get(i) {
                if job.scope.contains(raw_key) {
                    matched.push(job);
                }
            }
        }
        matched
    }
}

#[derive(Clone, Debug)]
enum DocEntry {
    All(GatePhase),
    Set(BTreeMap<DocId, GatePhase>),
}

/// Dokument-Index für sekundäre Indizes (§6.1.2).
#[derive(Clone, Default, Debug)]
pub(crate) struct DocIndex {
    tenants: BTreeMap<TenantId, GatePhase>,
    collections: BTreeMap<(TenantId, Box<str>), DocEntry>,
}

impl DocIndex {
    fn merge_phase(existing: GatePhase, new: GatePhase) -> GatePhase {
        match (existing, new) {
            (GatePhase::Erased, _) | (_, GatePhase::Erased) => GatePhase::Erased,
            _ => GatePhase::Restricted,
        }
    }

    fn build(jobs: &[Arc<JobGate>]) -> Self {
        let mut idx = Self::default();
        for job in jobs {
            if job.sealed && job.reinsert_after_sealed {
                continue;
            }
            for ds in &job.docs {
                match ds {
                    DocScope::Tenant { tenant } => {
                        idx.tenants
                            .entry(*tenant)
                            .and_modify(|p| *p = Self::merge_phase(*p, job.phase))
                            .or_insert(job.phase);
                    }
                    DocScope::Collection { tenant, collection } => {
                        let key = (*tenant, collection.as_str().into());
                        idx.collections
                            .entry(key)
                            .and_modify(|e| {
                                let merged = match e {
                                    DocEntry::All(p) => Self::merge_phase(*p, job.phase),
                                    DocEntry::Set(_) => job.phase,
                                };
                                *e = DocEntry::All(merged);
                            })
                            .or_insert(DocEntry::All(job.phase));
                    }
                    DocScope::Set {
                        tenant,
                        collection,
                        docs,
                    } => {
                        let key = (*tenant, collection.as_str().into());
                        let entry = idx
                            .collections
                            .entry(key)
                            .or_insert_with(|| DocEntry::Set(BTreeMap::new()));
                        match entry {
                            DocEntry::All(p) => {
                                *p = Self::merge_phase(*p, job.phase);
                            }
                            DocEntry::Set(map) => {
                                for doc_id in docs {
                                    map.entry(*doc_id)
                                        .and_modify(|p| *p = Self::merge_phase(*p, job.phase))
                                        .or_insert(job.phase);
                                }
                            }
                        }
                    }
                }
            }
        }
        idx
    }

    fn check_doc(&self, tenant: TenantId, collection: &str, doc: DocId) -> GateDecision {
        if let Some(p) = self.tenants.get(&tenant) {
            return match p {
                GatePhase::Restricted => GateDecision::Restricted,
                GatePhase::Erased => GateDecision::Erased,
            };
        }
        // BTreeMap lookup über temporären Allokations-Key vermeiden oder Box<str> bilden.
        let key: (TenantId, Box<str>) = (tenant, collection.into());
        match self.collections.get(&key) {
            Some(DocEntry::All(p)) => match p {
                GatePhase::Restricted => GateDecision::Restricted,
                GatePhase::Erased => GateDecision::Erased,
            },
            Some(DocEntry::Set(set)) => match set.get(&doc) {
                Some(GatePhase::Restricted) => GateDecision::Restricted,
                Some(GatePhase::Erased) => GateDecision::Erased,
                None => GateDecision::Open,
            },
            None => GateDecision::Open,
        }
    }
}

/// Unveränderlicher Copy-on-Write-Snapshot des Gates (§6.1).
#[derive(Clone, Default, Debug)]
pub(crate) struct GateSnapshot {
    pub epoch: u64,
    pub jobs: Vec<Arc<JobGate>>,
    pub key_index: KeyIndex,
    pub docs: DocIndex,
}

impl GateSnapshot {
    pub fn from_jobs(epoch: u64, mut jobs: Vec<Arc<JobGate>>) -> Self {
        jobs.sort_by(|a, b| a.id.0.cmp(&b.id.0));
        let key_index = KeyIndex::build(&jobs);
        let docs = DocIndex::build(&jobs);
        Self {
            epoch,
            jobs,
            key_index,
            docs,
        }
    }

    pub fn with_upserted_job(&self, new_epoch: u64, job: JobGate) -> Self {
        let mut jobs: Vec<Arc<JobGate>> = self
            .jobs
            .iter()
            .filter(|j| j.id != job.id)
            .cloned()
            .collect();
        jobs.push(Arc::new(job));
        Self::from_jobs(new_epoch, jobs)
    }

    pub fn without_job(&self, new_epoch: u64, id: ErasureId) -> Self {
        let jobs: Vec<Arc<JobGate>> = self.jobs.iter().filter(|j| j.id != id).cloned().collect();
        Self::from_jobs(new_epoch, jobs)
    }

    pub fn check_version(&self, raw_key: &[u8], version_seq: Seq) -> GateDecision {
        let matches = self.key_index.matching_jobs(&self.jobs, raw_key);
        let mut decision = GateDecision::Open;
        for job in matches {
            let job_dec = match job.phase {
                GatePhase::Restricted => GateDecision::Restricted,
                GatePhase::Erased => {
                    if version_seq <= job.up_to {
                        GateDecision::Erased
                    } else if job.sealed && job.reinsert_after_sealed {
                        GateDecision::Open
                    } else {
                        GateDecision::Erased
                    }
                }
            };
            match (decision, job_dec) {
                (_, GateDecision::Erased) => decision = GateDecision::Erased,
                (GateDecision::Open, GateDecision::Restricted) => {
                    decision = GateDecision::Restricted;
                }
                _ => {}
            }
        }
        decision
    }

    pub fn discard_rule(&self, raw_key: &[u8], version_seq: Seq) -> DiscardRule {
        let matches = self.key_index.matching_jobs(&self.jobs, raw_key);
        if matches.is_empty() {
            return DiscardRule::Keep;
        }
        let mut any_may_drop = false;
        for job in matches {
            match job.phase {
                GatePhase::Restricted => {
                    // Solange mindestens ein Auftrag noch nicht im Purge ist, nicht verwerfen.
                    return DiscardRule::Keep;
                }
                GatePhase::Erased => {
                    if version_seq <= job.up_to {
                        any_may_drop = true;
                    }
                }
            }
        }
        if any_may_drop {
            DiscardRule::MayDrop
        } else {
            DiscardRule::Keep
        }
    }
}

/// Zentrales ReadGate des Stores (§6).
pub(crate) struct StoreGate {
    active: AtomicBool,
    epoch: AtomicU64,
    snap: RwLock<Arc<GateSnapshot>>,
    fail_closed: AtomicBool,
}

impl StoreGate {
    pub fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            epoch: AtomicU64::new(0),
            snap: RwLock::new(Arc::new(GateSnapshot::default())),
            fail_closed: AtomicBool::new(false),
        }
    }

    /// Aktiviert den Fail-Closed-Modus bei korruptem Journal (I-7, §5.4).
    pub fn set_fail_closed(&self) {
        self.fail_closed.store(true, Ordering::Release);
        self.epoch.fetch_add(1, Ordering::Release);
        self.active.store(true, Ordering::Release);
    }

    pub fn is_fail_closed(&self) -> bool {
        self.fail_closed.load(Ordering::Acquire)
    }

    pub fn snapshot(&self) -> Arc<GateSnapshot> {
        Arc::clone(&self.snap.read())
    }

    /// Atomarer Snapshot-Tausch unter Rang 960. Liefert die neue Epoche zurück.
    pub fn install_snapshot(
        &self,
        mut build: impl FnMut(&GateSnapshot, u64) -> GateSnapshot,
    ) -> u64 {
        let mut guard = self.snap.write();
        let next_epoch = guard.epoch.saturating_add(1);
        let new_snap = Arc::new(build(&guard, next_epoch));
        let has_jobs = !new_snap.jobs.is_empty() || self.fail_closed.load(Ordering::Acquire);
        *guard = new_snap;
        self.epoch.store(next_epoch, Ordering::Release);
        self.active.store(has_jobs, Ordering::Release);
        next_epoch
    }
}

impl ReadGate for StoreGate {
    #[inline]
    fn is_active(&self) -> bool {
        // Acquire verhindert veraltete Sicht nach Rückkehr von publish_gate (I-5, K-28).
        self.active.load(Ordering::Acquire)
    }

    #[inline]
    fn epoch(&self) -> u64 {
        self.epoch.load(Ordering::Acquire)
    }

    fn check(&self, raw_key: &[u8]) -> GateDecision {
        if self.fail_closed.load(Ordering::Acquire) {
            return GateDecision::Restricted;
        }
        if !self.is_active() {
            return GateDecision::Open;
        }
        let snap = self.snapshot();
        let matches = snap.key_index.matching_jobs(&snap.jobs, raw_key);
        let mut decision = GateDecision::Open;
        for job in matches {
            if job.sealed && job.reinsert_after_sealed {
                continue;
            }
            match job.phase {
                GatePhase::Erased => return GateDecision::Erased,
                GatePhase::Restricted => decision = GateDecision::Restricted,
            }
        }
        decision
    }

    fn check_version(&self, raw_key: &[u8], version_seq: Seq) -> GateDecision {
        if self.fail_closed.load(Ordering::Acquire) {
            return GateDecision::Restricted;
        }
        if !self.is_active() {
            return GateDecision::Open;
        }
        let snap = self.snapshot();
        snap.check_version(raw_key, version_seq)
    }

    fn check_prefix(&self, prefix: &[u8]) -> RangeGate {
        if self.fail_closed.load(Ordering::Acquire) {
            return RangeGate::Partial;
        }
        if !self.is_active() {
            return RangeGate::EntirelyOpen;
        }
        let snap = self.snapshot();
        let mut any_overlap = false;

        for job in &snap.jobs {
            if job.sealed && job.reinsert_after_sealed {
                // Für bereits versiegelte Aufträge mit Reinsert können neuere Versionen existieren -> Partial prüfen.
                any_overlap = true;
                continue;
            }
            let mut job_entirely_blocks = false;
            let mut job_has_out_under_prefix = false;

            for (pat, verdict) in job.scope.rules() {
                let pb = pat.bytes();
                let overlaps = pb.starts_with(prefix) || prefix.starts_with(pb);
                if overlaps {
                    any_overlap = true;
                }
                match verdict {
                    Verdict::In => {
                        if let KeyPattern::Prefix(p) = pat {
                            if prefix.starts_with(p.as_slice()) {
                                job_entirely_blocks = true;
                            }
                        }
                    }
                    Verdict::Out => {
                        if overlaps {
                            job_has_out_under_prefix = true;
                        }
                    }
                }
            }

            if job_entirely_blocks && !job_has_out_under_prefix {
                return RangeGate::EntirelyBlocked;
            }
        }

        if any_overlap {
            RangeGate::Partial
        } else {
            RangeGate::EntirelyOpen
        }
    }

    fn discard_rule(&self, raw_key: &[u8], version_seq: Seq) -> DiscardRule {
        if self.fail_closed.load(Ordering::Acquire) {
            return DiscardRule::Keep;
        }
        if !self.is_active() {
            return DiscardRule::Keep;
        }
        let snap = self.snapshot();
        snap.discard_rule(raw_key, version_seq)
    }

    fn check_doc(&self, tenant: TenantId, collection: &str, doc: DocId) -> GateDecision {
        if self.fail_closed.load(Ordering::Acquire) {
            return GateDecision::Restricted;
        }
        if !self.is_active() {
            return GateDecision::Open;
        }
        let snap = self.snapshot();
        snap.docs.check_doc(tenant, collection, doc)
    }

    fn gated_seq(&self, raw_key: &[u8]) -> Option<Seq> {
        if self.fail_closed.load(Ordering::Acquire) || !self.is_active() {
            return None;
        }
        let snap = self.snapshot();
        let matches = snap.key_index.matching_jobs(&snap.jobs, raw_key);
        matches
            .into_iter()
            .filter(|j| !(j.sealed && j.reinsert_after_sealed))
            .map(|j| j.up_to)
            .min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant_codec::TenantKeyCodec;
    use proptest::prelude::*;

    fn make_job(
        id_byte: u8,
        rules: Vec<(KeyPattern, Verdict)>,
        up_to: u64,
        phase: GatePhase,
    ) -> JobGate {
        JobGate {
            id: ErasureId([id_byte; 16]),
            scope: Arc::new(KeyScope::new(rules).unwrap()),
            docs: Vec::new(),
            up_to: Seq(up_to),
            phase,
            reinsert_after_sealed: false,
            sealed: false,
        }
    }

    /// TST-STORE-GATE-5: is_active() ohne Gates -> Open, Epoche streng monoton.
    #[test]
    fn test_gate_5_empty_and_monotonic_epoch() {
        let gate = StoreGate::new();
        assert!(!gate.is_active());
        assert_eq!(gate.epoch(), 0);
        assert_eq!(gate.check(b"t:1:doc"), GateDecision::Open);
        assert_eq!(gate.check_prefix(b"t:1:"), RangeGate::EntirelyOpen);

        let e1 = gate.install_snapshot(|s, ep| {
            s.with_upserted_job(
                ep,
                make_job(
                    1,
                    vec![(KeyPattern::Prefix(b"t:1:".to_vec()), Verdict::In)],
                    10,
                    GatePhase::Restricted,
                ),
            )
        });
        assert_eq!(e1, 1);
        assert!(gate.is_active());
        assert_eq!(gate.epoch(), 1);
        assert_eq!(gate.check(b"t:1:doc"), GateDecision::Restricted);
        assert_eq!(gate.discard_rule(b"t:1:doc", Seq(5)), DiscardRule::Keep);

        let e2 = gate.install_snapshot(|s, ep| s.without_job(ep, ErasureId([1; 16])));
        assert_eq!(e2, 2);
        assert!(!gate.is_active());
        assert_eq!(gate.check(b"t:1:doc"), GateDecision::Open);
    }

    /// TST-STORE-GATE-2: up_to-Semantik nach Sealed + AllowAfterSealed.
    #[test]
    fn test_gate_2_reinsert_after_sealed_up_to() {
        let gate = StoreGate::new();
        gate.install_snapshot(|s, ep| {
            s.with_upserted_job(
                ep,
                JobGate {
                    id: ErasureId([2; 16]),
                    scope: Arc::new(
                        KeyScope::new(vec![(KeyPattern::Prefix(b"t:2:".to_vec()), Verdict::In)])
                            .unwrap(),
                    ),
                    docs: Vec::new(),
                    up_to: Seq(100),
                    phase: GatePhase::Erased,
                    reinsert_after_sealed: true,
                    sealed: true,
                },
            )
        });

        assert_eq!(gate.check(b"t:2:a"), GateDecision::Open);
        assert_eq!(gate.check_version(b"t:2:a", Seq(100)), GateDecision::Erased);
        assert_eq!(gate.check_version(b"t:2:a", Seq(101)), GateDecision::Open);
        assert_eq!(gate.discard_rule(b"t:2:a", Seq(100)), DiscardRule::MayDrop);
        assert_eq!(gate.discard_rule(b"t:2:a", Seq(101)), DiscardRule::Keep);
    }

    /// TST-STORE-GATE-3: Fail-Closed-Modus.
    #[test]
    fn test_gate_3_fail_closed() {
        let gate = StoreGate::new();
        gate.set_fail_closed();
        assert!(gate.is_active());
        assert_eq!(gate.check(b"anything"), GateDecision::Restricted);
        assert_eq!(
            gate.check_version(b"anything", Seq(1)),
            GateDecision::Restricted
        );
        assert_eq!(gate.discard_rule(b"anything", Seq(1)), DiscardRule::Keep);
        assert_eq!(
            gate.check_doc(TenantId::try_new(1).unwrap(), "default", DocId(1)),
            GateDecision::Restricted
        );
    }

    /// TST-STORE-GATE-4: > 64 Präfixlängen -> linearer Rückfall liefert identische Ergebnisse.
    #[test]
    fn test_gate_4_over_64_prefix_lengths_fallback() {
        let gate = StoreGate::new();
        let mut rules = Vec::new();
        for len in 1..=70 {
            let p = vec![b'a'; len];
            rules.push((KeyPattern::Prefix(p), Verdict::In));
        }
        gate.install_snapshot(|s, ep| {
            s.with_upserted_job(ep, make_job(3, rules.clone(), 50, GatePhase::Erased))
        });
        assert!(gate.snapshot().key_index.fallback_linear);
        assert_eq!(gate.check(b"aaa"), GateDecision::Erased);
        assert_eq!(gate.check(b"bbb"), GateDecision::Open);
    }

    /// TST-STORE-ISO-1: Mandantenpräfix t:7: berührt t:70: nicht.
    #[test]
    fn test_iso_1_tenant_prefix_trap() {
        let gate = StoreGate::new();
        let t7 = TenantKeyCodec::new(TenantId::try_new(7).unwrap());
        let t70 = TenantKeyCodec::new(TenantId::try_new(70).unwrap());
        assert_eq!(t7.scan_prefix(), b"t:7:");
        assert_eq!(t70.scan_prefix(), b"t:70:");

        gate.install_snapshot(|s, ep| {
            s.with_upserted_job(
                ep,
                make_job(
                    7,
                    vec![(KeyPattern::Prefix(t7.scan_prefix().to_vec()), Verdict::In)],
                    99,
                    GatePhase::Erased,
                ),
            )
        });
        assert_eq!(gate.check(b"t:7:k1"), GateDecision::Erased);
        assert_eq!(gate.check(b"t:70:k1"), GateDecision::Open);
    }

    proptest! {
        /// TST-STORE-GATE-1: Vergleich gegen Brute-Force-Modell.
        #[test]
        fn prop_gate_matches_bruteforce(
            rules in prop::collection::vec(
                (prop::collection::vec(any::<u8>(), 1..12), any::<bool>(), any::<bool>()),
                1..8
            ),
            query in prop::collection::vec(any::<u8>(), 0..16),
            seq in 0u64..200,
        ) {
            let mut parsed_rules = Vec::new();
            for (bytes, is_exact, is_in) in rules {
                let pat = if is_exact { KeyPattern::Exact(bytes) } else { KeyPattern::Prefix(bytes) };
                let v = if is_in { Verdict::In } else { Verdict::Out };
                parsed_rules.push((pat, v));
            }
            if let Ok(scope) = KeyScope::new(parsed_rules) {
                let gate = StoreGate::new();
                let scope_arc = Arc::new(scope.clone());
                gate.install_snapshot(|s, ep| {
                    s.with_upserted_job(ep, JobGate {
                        id: ErasureId([9; 16]),
                        scope: Arc::clone(&scope_arc),
                        docs: Vec::new(),
                        up_to: Seq(100),
                        phase: GatePhase::Erased,
                        reinsert_after_sealed: false,
                        sealed: false,
                    })
                });
                let expected_in = scope.contains(&query);
                let expected_dec = if expected_in { GateDecision::Erased } else { GateDecision::Open };
                prop_assert_eq!(gate.check(&query), expected_dec);
                prop_assert_eq!(gate.check_version(&query, Seq(seq)), expected_dec);
                let expected_discard = if expected_in && seq <= 100 {
                    DiscardRule::MayDrop
                } else {
                    DiscardRule::Keep
                };
                prop_assert_eq!(gate.discard_rule(&query, Seq(seq)), expected_discard);
            }
        }
    }
}
