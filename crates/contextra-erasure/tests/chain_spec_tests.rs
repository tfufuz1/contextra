//! Tests der Löschkette nach SPEC §5, §6, §12 (T-DEL-4, T-DEL-12–15) und den Reihenfolge-Invarianten.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use contextra_erasure::chain::adapters::{FileVaultKeys, SagaGateSink, TextKeyLayout};
use contextra_erasure::chain::layer::{
    ErasureLayer, LayerFuture, LayerScope, PurgeCx, PurgeOutcome,
};
use contextra_erasure::chain::ports::{
    DiskReserve, Journal, KeyVault, PositiveControl, ReportSink,
};
use contextra_erasure::chain::{
    Chain, ChainDeps, EraseTarget, FileJournal, JournalRecord, PendingReason, RetentionAttestation,
    RetentionCode, SagaState,
};
use contextra_erasure::error::ErasureError;
use contextra_erasure::gate::saga_gate::{GateDecision, ReadGate, SagaGate};
use contextra_erasure::types_local::{
    Armed, KeyCoverage, LayerId, LayerWitness, OperatorRef, Scanner,
};
use contextra_erasure::vault::FileVault;
use contextra_ids::{CollectionId, DocId, ErasureId, TenantId, Timestamp};
use parking_lot::Mutex;

// ---------- Testdoppel ----------

struct RecLayer {
    id: u8,
    priority: u16,
    log: Arc<Mutex<Vec<u8>>>,
    purges: AtomicU64,
    residue: AtomicU64,
}

impl RecLayer {
    fn new(id: u8, priority: u16, log: Arc<Mutex<Vec<u8>>>) -> Self {
        Self {
            id,
            priority,
            log,
            purges: AtomicU64::new(0),
            residue: AtomicU64::new(0),
        }
    }
}

impl ErasureLayer for RecLayer {
    fn id(&self) -> LayerId {
        LayerId::new([self.id; 16])
    }

    fn key_coverage(&self) -> KeyCoverage {
        KeyCoverage::Full
    }

    fn resolve<'a>(&'a self, _target: &'a EraseTarget) -> LayerFuture<'a, LayerScope> {
        Box::pin(async move {
            Ok(LayerScope {
                scope_id: [self.id; 16],
                affected_units: vec![1],
            })
        })
    }

    fn on_gate<'a>(&'a self, _scope: &'a LayerScope) -> LayerFuture<'a, ()> {
        Box::pin(async move { Ok(()) })
    }

    fn purge<'a>(
        &'a self,
        _scope: &'a LayerScope,
        _cx: &'a PurgeCx,
    ) -> LayerFuture<'a, PurgeOutcome> {
        Box::pin(async move {
            self.purges.fetch_add(1, Ordering::SeqCst);
            self.log.lock().push(self.id);
            Ok(PurgeOutcome { purged_units: 1 })
        })
    }

    fn witness<'a>(
        &'a self,
        _scope: &'a LayerScope,
        scanner: &'a mut Scanner<Armed>,
    ) -> LayerFuture<'a, u64> {
        Box::pin(async move {
            scanner.record(1, 10);
            Ok(self.residue.load(Ordering::SeqCst))
        })
    }
}

struct Disk(AtomicU64);
impl DiskReserve for Disk {
    fn available_bytes(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

struct Canary(AtomicBool);
impl PositiveControl for Canary {
    fn canary_observed(&self) -> Result<bool, ErasureError> {
        Ok(self.0.load(Ordering::SeqCst))
    }
}

#[derive(Default)]
struct Report(Mutex<Vec<ErasureId>>);
impl ReportSink for Report {
    fn publish(
        &self,
        erasure_id: ErasureId,
        _target: &EraseTarget,
        _attestation: &RetentionAttestation,
        _witnesses: &[LayerWitness],
    ) -> Result<(), ErasureError> {
        self.0.lock().push(erasure_id);
        Ok(())
    }
}

/// Journal, das beim Schreiben eines bestimmten Zustands fehlschlägt (Crash-Injektion).
struct FlakyJournal {
    inner: FileJournal,
    fail_on: Mutex<Option<SagaState>>,
}

impl Journal for FlakyJournal {
    fn append(&self, record: &JournalRecord) -> Result<(), ErasureError> {
        if self.fail_on.lock().as_ref() == Some(&record.state) {
            return Err(ErasureError::Io(std::io::Error::other("injected crash")));
        }
        self.inner.append(record)
    }
    fn load(&self) -> Result<Vec<JournalRecord>, ErasureError> {
        self.inner.load()
    }
}

/// Schlüsseltresor, dessen Verifikation erzwungen fehlschlägt.
struct BrokenVerify(Arc<FileVaultKeys>);
impl KeyVault for BrokenVerify {
    fn destroy(&self, target: &EraseTarget) -> Result<(), ErasureError> {
        self.0.destroy(target)
    }
    fn verify_absent(&self, _target: &EraseTarget) -> Result<bool, ErasureError> {
        Ok(false)
    }
}

/// Schlüsseltresor, der beim Vernichten die Gate-Entscheidung protokolliert.
struct ProbeKeys {
    inner: Arc<FileVaultKeys>,
    gate: Arc<SagaGate>,
    seen_at_destroy: Mutex<Option<GateDecision>>,
    key: Vec<u8>,
}
impl KeyVault for ProbeKeys {
    fn destroy(&self, target: &EraseTarget) -> Result<(), ErasureError> {
        *self.seen_at_destroy.lock() = Some(self.gate.check(&self.key));
        self.inner.destroy(target)
    }
    fn verify_absent(&self, target: &EraseTarget) -> Result<bool, ErasureError> {
        self.inner.verify_absent(target)
    }
}

// ---------- Aufbau ----------

fn tmp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ctx-chain-spec-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("tmp dir");
    dir
}

fn doc(n: u64) -> EraseTarget {
    EraseTarget::Document {
        tenant: TenantId(1),
        collection: CollectionId(2),
        doc: DocId(n),
    }
}

fn attest(code: RetentionCode) -> RetentionAttestation {
    RetentionAttestation {
        code,
        attested_by: OperatorRef::try_new(b"operator-1".to_vec()).expect("operator"),
        attested_at: Timestamp(1),
    }
}

fn eid(n: u8) -> ErasureId {
    ErasureId([n; 16])
}

struct Env {
    gate: Arc<SagaGate>,
    keys: Arc<FileVaultKeys>,
    layers: Vec<Arc<RecLayer>>,
    log: Arc<Mutex<Vec<u8>>>,
    disk: Arc<Disk>,
    canary: Arc<Canary>,
    report: Arc<Report>,
}

/// Drei Schichten mit absichtlich nicht sortierter Priorität (Purge-Reihenfolge 1, 2, 3).
fn env() -> Env {
    let log = Arc::new(Mutex::new(Vec::new()));
    Env {
        gate: Arc::new(SagaGate::new()),
        keys: Arc::new(FileVaultKeys::new(FileVault::new_in_memory(1024))),
        layers: vec![
            Arc::new(RecLayer::new(3, 30, Arc::clone(&log))),
            Arc::new(RecLayer::new(1, 10, Arc::clone(&log))),
            Arc::new(RecLayer::new(2, 20, Arc::clone(&log))),
        ],
        log,
        disk: Arc::new(Disk(AtomicU64::new(2 * 1_073_741_824))),
        canary: Arc::new(Canary(AtomicBool::new(true))),
        report: Arc::new(Report::default()),
    }
}

fn chain_with(
    env: &Env,
    journal: Arc<dyn Journal>,
    keys: Arc<dyn KeyVault>,
    gate: Arc<SagaGate>,
) -> Chain {
    Chain::new(ChainDeps {
        journal,
        gate: Arc::new(SagaGateSink::new(gate)),
        keys,
        layers: env
            .layers
            .iter()
            .map(|l| (l.priority, Arc::clone(l) as Arc<dyn ErasureLayer>))
            .collect(),
        disk: Arc::clone(&env.disk) as Arc<dyn DiskReserve>,
        disk_required_bytes: 1_073_741_824,
        canary: Arc::clone(&env.canary) as Arc<dyn PositiveControl>,
        report: Arc::clone(&env.report) as Arc<dyn ReportSink>,
        dedupe_key: [9; 32],
    })
}

fn cx() -> PurgeCx {
    PurgeCx {
        deadline_ms: u64::MAX,
    }
}

fn journal_states(path: &std::path::Path) -> Vec<SagaState> {
    FileJournal::open(path)
        .expect("open")
        .load()
        .expect("load")
        .into_iter()
        .map(|r| r.state)
        .collect()
}

// ---------- Tests ----------

#[tokio::test]
async fn vollkette_erreicht_completed_mit_verifizierter_messung() {
    let dir = tmp_dir("full");
    let path = dir.join("chain.journal");
    let env = env();
    env.keys.provision(&doc(1), [5; 32]).expect("provision");
    let chain = chain_with(
        &env,
        Arc::new(FileJournal::open(&path).expect("open")),
        Arc::clone(&env.keys) as Arc<dyn KeyVault>,
        Arc::clone(&env.gate),
    );

    let r = chain
        .erase(eid(1), doc(1), attest(RetentionCode::NoObligation))
        .await
        .expect("erase");
    assert_eq!(r.state, SagaState::KeyRevoked);
    chain.purge(eid(1), cx()).await.expect("purge");
    let done = chain.verify(eid(1)).await.expect("verify");

    assert_eq!(done.state, SagaState::Completed);
    assert_eq!(
        *env.log.lock(),
        vec![1, 2, 3],
        "Purge in Prioritätsreihenfolge"
    );
    assert_eq!(env.report.0.lock().as_slice(), &[eid(1)]);
    assert_eq!(
        journal_states(&path),
        vec![
            SagaState::Requested,
            SagaState::Gated,
            SagaState::KeyRevoked,
            SagaState::Purging,
            SagaState::Verifying,
            SagaState::Completed,
        ]
    );
}

#[tokio::test]
async fn gate_ist_aktiv_bevor_der_schluessel_vernichtet_wird() {
    let dir = tmp_dir("order");
    let env = env();
    env.keys.provision(&doc(2), [5; 32]).expect("provision");
    let key = TextKeyLayout::document_key(TenantId(1), CollectionId(2), DocId(2)).expect("key");
    let probe = Arc::new(ProbeKeys {
        inner: Arc::clone(&env.keys),
        gate: Arc::clone(&env.gate),
        seen_at_destroy: Mutex::new(None),
        key,
    });
    let chain = chain_with(
        &env,
        Arc::new(FileJournal::open(&dir.join("j")).expect("open")),
        Arc::clone(&probe) as Arc<dyn KeyVault>,
        Arc::clone(&env.gate),
    );
    chain
        .erase(eid(2), doc(2), attest(RetentionCode::NoObligation))
        .await
        .expect("erase");
    assert_eq!(
        *probe.seen_at_destroy.lock(),
        Some(GateDecision::Erased),
        "Gate VOR Schlüsselvernichtung"
    );
}

#[tokio::test]
async fn aufbewahrungspflicht_verweigert_ohne_gate_und_ohne_vernichtung() {
    let dir = tmp_dir("refused");
    let path = dir.join("j");
    let env = env();
    env.keys.provision(&doc(3), [5; 32]).expect("provision");
    let chain = chain_with(
        &env,
        Arc::new(FileJournal::open(&path).expect("open")),
        Arc::clone(&env.keys) as Arc<dyn KeyVault>,
        Arc::clone(&env.gate),
    );
    let r = chain
        .erase(eid(3), doc(3), attest(RetentionCode::ObligationActive))
        .await
        .expect("erase");
    assert_eq!(r.state, SagaState::Refused);
    assert_eq!(
        env.gate.check(
            &TextKeyLayout::document_key(TenantId(1), CollectionId(2), DocId(3)).expect("k")
        ),
        GateDecision::Open
    );
    assert!(
        !env.keys.verify_absent(&doc(3)).expect("verify"),
        "Schlüssel bleibt unangetastet"
    );
    assert_eq!(
        journal_states(&path),
        vec![SagaState::Requested, SagaState::Refused]
    );
}

#[tokio::test]
async fn fehlgeschlagene_schluesselverifikation_laesst_zustand_gated() {
    let dir = tmp_dir("shred-fail");
    let env = env();
    env.keys.provision(&doc(4), [5; 32]).expect("provision");
    let chain = chain_with(
        &env,
        Arc::new(FileJournal::open(&dir.join("j")).expect("open")),
        Arc::new(BrokenVerify(Arc::clone(&env.keys))),
        Arc::clone(&env.gate),
    );
    let err = chain
        .erase(eid(4), doc(4), attest(RetentionCode::NoObligation))
        .await
        .expect_err("muss scheitern");
    assert!(matches!(err, ErasureError::ShredVerificationFailed));
    assert_eq!(chain.status(eid(4)).expect("status"), SagaState::Gated);
}

#[tokio::test]
async fn erneuter_antrag_fuer_dasselbe_ziel_ist_idempotent() {
    let dir = tmp_dir("idem");
    let env = env();
    env.keys.provision(&doc(5), [5; 32]).expect("provision");
    let chain = chain_with(
        &env,
        Arc::new(FileJournal::open(&dir.join("j")).expect("open")),
        Arc::clone(&env.keys) as Arc<dyn KeyVault>,
        Arc::clone(&env.gate),
    );
    let first = chain
        .erase(eid(5), doc(5), attest(RetentionCode::NoObligation))
        .await
        .expect("first");
    let second = chain
        .erase(eid(6), doc(5), attest(RetentionCode::NoObligation))
        .await
        .expect("second");
    assert_eq!(second.id, first.id, "kein zweiter Lauf");
    assert_eq!(second.state, SagaState::KeyRevoked);
}

#[tokio::test]
async fn restrict_dann_lift_ist_terminal_und_gibt_frei() {
    let dir = tmp_dir("restrict");
    let env = env();
    let chain = chain_with(
        &env,
        Arc::new(FileJournal::open(&dir.join("j")).expect("open")),
        Arc::clone(&env.keys) as Arc<dyn KeyVault>,
        Arc::clone(&env.gate),
    );
    let key = TextKeyLayout::document_key(TenantId(1), CollectionId(2), DocId(7)).expect("k");
    let r = chain
        .restrict(
            eid(7),
            doc(7),
            attest(RetentionCode::DataSubjectRequestedRestriction),
        )
        .expect("restrict");
    assert_eq!(r.state, SagaState::Restricted);
    assert_eq!(env.gate.check(&key), GateDecision::Restricted);

    let lifted = chain.lift(eid(7)).expect("lift");
    assert_eq!(lifted.state, SagaState::Lifted);
    assert_eq!(env.gate.check(&key), GateDecision::Open);
    assert!(chain.lift(eid(7)).is_err(), "Lifted ist terminal (INV-9)");
}

#[tokio::test]
async fn restrict_dann_erase_eskaliert_zur_loeschung() {
    let dir = tmp_dir("escalate");
    let env = env();
    env.keys.provision(&doc(8), [5; 32]).expect("provision");
    let chain = chain_with(
        &env,
        Arc::new(FileJournal::open(&dir.join("j")).expect("open")),
        Arc::clone(&env.keys) as Arc<dyn KeyVault>,
        Arc::clone(&env.gate),
    );
    chain
        .restrict(
            eid(8),
            doc(8),
            attest(RetentionCode::DataSubjectRequestedRestriction),
        )
        .expect("restrict");
    let r = chain
        .erase(eid(9), doc(8), attest(RetentionCode::NoObligation))
        .await
        .expect("erase");
    assert_eq!(r.id, eid(8));
    assert_eq!(r.state, SagaState::KeyRevoked);
    let key = TextKeyLayout::document_key(TenantId(1), CollectionId(2), DocId(8)).expect("k");
    assert_eq!(
        env.gate.check(&key),
        GateDecision::Erased,
        "Erased überschreibt Restricted"
    );
}

#[tokio::test]
async fn t_del_14_purge_ist_idempotent() {
    let dir = tmp_dir("purge-idem");
    let env = env();
    env.keys.provision(&doc(10), [5; 32]).expect("provision");
    let chain = chain_with(
        &env,
        Arc::new(FileJournal::open(&dir.join("j")).expect("open")),
        Arc::clone(&env.keys) as Arc<dyn KeyVault>,
        Arc::clone(&env.gate),
    );
    chain
        .erase(eid(10), doc(10), attest(RetentionCode::NoObligation))
        .await
        .expect("erase");
    chain.purge(eid(10), cx()).await.expect("purge 1");
    chain.purge(eid(10), cx()).await.expect("purge 2 (no-op)");
    for layer in &env.layers {
        assert_eq!(
            layer.purges.load(Ordering::SeqCst),
            1,
            "keine zweite physische Bereinigung"
        );
    }
}

#[tokio::test]
async fn t_del_15_verify_key_absent_nur_nach_vernichtung() {
    let env = env();
    env.keys.provision(&doc(11), [5; 32]).expect("provision");
    assert!(
        !env.keys.verify_absent(&doc(11)).expect("verify"),
        "lebender Schlüssel ist nicht absent"
    );
    env.keys.destroy(&doc(11)).expect("destroy");
    assert!(
        env.keys.verify_absent(&doc(11)).expect("verify"),
        "vernichteter Schlüssel ist absent"
    );
    env.keys.destroy(&doc(11)).expect("destroy idempotent");
}

#[tokio::test]
async fn t_del_12_crash_in_phase_3_neustart_setzt_fort_mit_aktivem_gate() {
    let dir = tmp_dir("crash");
    let path = dir.join("chain.journal");
    let env = env();
    env.keys.provision(&doc(12), [5; 32]).expect("provision");

    // Lauf 1: Absturz beim Schreiben von KeyRevoked (nach der Vernichtung, vor dem Journal-Eintrag).
    let flaky = Arc::new(FlakyJournal {
        inner: FileJournal::open(&path).expect("open"),
        fail_on: Mutex::new(Some(SagaState::KeyRevoked)),
    });
    let chain1 = chain_with(
        &env,
        Arc::clone(&flaky) as Arc<dyn Journal>,
        Arc::clone(&env.keys) as Arc<dyn KeyVault>,
        Arc::clone(&env.gate),
    );
    assert!(chain1
        .erase(eid(12), doc(12), attest(RetentionCode::NoObligation))
        .await
        .is_err());
    assert_eq!(
        journal_states(&path),
        vec![SagaState::Requested, SagaState::Gated]
    );

    // Neustart: frisches Gate, Recovery aktiviert es sofort vor jedem Lesezugriff.
    let gate2 = Arc::new(SagaGate::new());
    let chain2 = chain_with(
        &env,
        Arc::new(FileJournal::open(&path).expect("reopen")),
        Arc::clone(&env.keys) as Arc<dyn KeyVault>,
        Arc::clone(&gate2),
    );
    let open = chain2.recover().expect("recover");
    assert_eq!(open, vec![eid(12)]);
    let key = TextKeyLayout::document_key(TenantId(1), CollectionId(2), DocId(12)).expect("k");
    assert_eq!(
        gate2.check(&key),
        GateDecision::Erased,
        "Datum bleibt nach Neustart gesperrt"
    );
    assert_eq!(chain2.status(eid(12)).expect("status"), SagaState::Gated);

    // Fortsetzung bis Completed.
    assert_eq!(
        chain2.resume(eid(12)).await.expect("resume").state,
        SagaState::KeyRevoked
    );
    chain2.purge(eid(12), cx()).await.expect("purge");
    assert_eq!(
        chain2.verify(eid(12)).await.expect("verify").state,
        SagaState::Completed
    );
}

#[tokio::test]
async fn t_del_4_nach_neustart_bleibt_completed_datum_erased() {
    let dir = tmp_dir("reboot");
    let path = dir.join("j");
    let env = env();
    env.keys.provision(&doc(13), [5; 32]).expect("provision");
    let chain = chain_with(
        &env,
        Arc::new(FileJournal::open(&path).expect("open")),
        Arc::clone(&env.keys) as Arc<dyn KeyVault>,
        Arc::clone(&env.gate),
    );
    chain
        .erase(eid(13), doc(13), attest(RetentionCode::NoObligation))
        .await
        .expect("erase");
    chain.purge(eid(13), cx()).await.expect("purge");
    chain.verify(eid(13)).await.expect("verify");

    let gate2 = Arc::new(SagaGate::new());
    let chain2 = chain_with(
        &env,
        Arc::new(FileJournal::open(&path).expect("reopen")),
        Arc::clone(&env.keys) as Arc<dyn KeyVault>,
        Arc::clone(&gate2),
    );
    assert!(
        chain2.recover().expect("recover").is_empty(),
        "abgeschlossen = nicht offen"
    );
    let key = TextKeyLayout::document_key(TenantId(1), CollectionId(2), DocId(13)).expect("k");
    assert_eq!(gate2.check(&key), GateDecision::Erased);
}

#[tokio::test]
async fn plattenreserve_niedrig_pausiert_purge_und_setzt_pending_fort() {
    let dir = tmp_dir("disk");
    let path = dir.join("j");
    let env = env();
    env.keys.provision(&doc(14), [5; 32]).expect("provision");
    let chain = chain_with(
        &env,
        Arc::new(FileJournal::open(&path).expect("open")),
        Arc::clone(&env.keys) as Arc<dyn KeyVault>,
        Arc::clone(&env.gate),
    );
    chain
        .erase(eid(14), doc(14), attest(RetentionCode::NoObligation))
        .await
        .expect("erase");
    env.disk.0.store(1024, Ordering::SeqCst);
    assert!(matches!(
        chain.purge(eid(14), cx()).await,
        Err(ErasureError::DiskReserve)
    ));
    assert_eq!(
        chain.status(eid(14)).expect("status"),
        SagaState::Pending(PendingReason::DiskReserveLow)
    );

    env.disk.0.store(2 * 1_073_741_824, Ordering::SeqCst);
    assert_eq!(
        chain.resume(eid(14)).await.expect("resume").state,
        SagaState::Completed
    );
}

#[tokio::test]
async fn rueckstand_in_messung_haelt_zustand_verifying() {
    let dir = tmp_dir("residue");
    let env = env();
    env.keys.provision(&doc(15), [5; 32]).expect("provision");
    let chain = chain_with(
        &env,
        Arc::new(FileJournal::open(&dir.join("j")).expect("open")),
        Arc::clone(&env.keys) as Arc<dyn KeyVault>,
        Arc::clone(&env.gate),
    );
    chain
        .erase(eid(15), doc(15), attest(RetentionCode::NoObligation))
        .await
        .expect("erase");
    chain.purge(eid(15), cx()).await.expect("purge");
    env.layers[1].residue.store(3, Ordering::SeqCst);
    assert!(matches!(
        chain.verify(eid(15)).await,
        Err(ErasureError::Residue(3))
    ));
    assert_eq!(chain.status(eid(15)).expect("status"), SagaState::Verifying);
}

#[tokio::test]
async fn ohne_positivkontrolle_ist_messung_ungueltig() {
    let dir = tmp_dir("canary");
    let env = env();
    env.keys.provision(&doc(16), [5; 32]).expect("provision");
    let chain = chain_with(
        &env,
        Arc::new(FileJournal::open(&dir.join("j")).expect("open")),
        Arc::clone(&env.keys) as Arc<dyn KeyVault>,
        Arc::clone(&env.gate),
    );
    chain
        .erase(eid(16), doc(16), attest(RetentionCode::NoObligation))
        .await
        .expect("erase");
    chain.purge(eid(16), cx()).await.expect("purge");
    env.canary.0.store(false, Ordering::SeqCst);
    assert!(chain.verify(eid(16)).await.is_err());
    assert_eq!(chain.status(eid(16)).expect("status"), SagaState::Verifying);
}

#[tokio::test]
async fn subjekt_ohne_index_scheitert_fail_closed_ohne_zustandswechsel() {
    let dir = tmp_dir("subject");
    let env = env();
    let subject = EraseTarget::Subject {
        tenant: TenantId(1),
        subject: contextra_ids::SubjectRef([4; 16]),
    };
    env.keys.provision(&subject, [5; 32]).expect("provision");
    let chain = chain_with(
        &env,
        Arc::new(FileJournal::open(&dir.join("j")).expect("open")),
        Arc::clone(&env.keys) as Arc<dyn KeyVault>,
        Arc::clone(&env.gate),
    );
    assert!(chain
        .erase(
            eid(17),
            subject.clone(),
            attest(RetentionCode::NoObligation)
        )
        .await
        .is_err());
    assert_eq!(chain.status(eid(17)).expect("status"), SagaState::Requested);
    assert!(
        !env.keys.verify_absent(&subject).expect("verify"),
        "Schlüssel unberührt"
    );
}

#[tokio::test]
async fn mandanten_loeschung_deckt_alle_dokumente_des_mandanten_ab() {
    let dir = tmp_dir("tenant");
    let env = env();
    let tenant = EraseTarget::Tenant {
        tenant: TenantId(1),
    };
    env.keys.provision(&tenant, [5; 32]).expect("provision");
    let chain = chain_with(
        &env,
        Arc::new(FileJournal::open(&dir.join("j")).expect("open")),
        Arc::clone(&env.keys) as Arc<dyn KeyVault>,
        Arc::clone(&env.gate),
    );
    chain
        .erase(eid(18), tenant, attest(RetentionCode::NoObligation))
        .await
        .expect("erase");
    let other_tenant_doc =
        TextKeyLayout::document_key(TenantId(2), CollectionId(2), DocId(1)).expect("k");
    let own_doc = TextKeyLayout::document_key(TenantId(1), CollectionId(9), DocId(77)).expect("k");
    assert_eq!(env.gate.check(&own_doc), GateDecision::Erased);
    assert_eq!(
        env.gate.check(&other_tenant_doc),
        GateDecision::Open,
        "fremder Mandant bleibt offen"
    );
}
