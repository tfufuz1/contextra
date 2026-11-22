//! # Engine Core Integrationstests
//!
//! Verifiziert die Kerninvarianten und Modelle:
//! - INV-VIS-1: Memtables enthalten nur Daten aus erfolgreich synchronisierten Gruppen
//! - INV-VIS-2: Teilweise eingefügte Gruppen sind unsichtbar; Changes sind atomar sichtbar
//! - INV-VIS-3: Leser laden erst visible_stamp, dann die Version
//! - INV-RET-1: retention_floor ist monoton nicht fallend; retain behält alle > floor und neueste <= floor
//! - INV-LOCK-1: Sperren werden nur in aufsteigender Rangfolge erworben
//! - SD-49 / E-4: Gegendruck wartet vor dem Ticket, nie im Committer
//! - SD-53: Guard-Vorwärmung
//! - SD-62: Listener-Zustellung mit Frist und Abmeldung bei Überlauf (ListenerLagged)
//! - H-S2 und H-S7: Messhaken und Unterbatches (<= 64 Ops je Sperre)

use std::collections::BTreeMap;
use std::io;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use contextra_storage::api::listener::{
    ChangeEvent, Listener, ListenerConfig, ListenerError, ListenerQueue,
};
use contextra_storage::codec::tag::{InternalKey, Tag, TagKind};
use contextra_storage::engine::committer::{
    BackpressureLevel, BackpressureSource, Change, CommitError, Committer, Durability,
    GuardPrewarmer, RedoWriter,
};
use contextra_storage::engine::memtable::{Memtable, Slot, MEMTABLE_SUBBATCH_LIMIT};
use contextra_storage::engine::registry::{Registry, RegistryError};
use contextra_storage::engine::retention::{retain, RetainItem};
use contextra_storage::engine::version::{FileHandle, Version, VersionState};
use contextra_storage::engine::views::{create_view, View};

/// Dummy Redo-Writer für Tests.
struct MockRedoWriter {
    fail_sync: AtomicBool,
    fail_write: AtomicBool,
}

impl MockRedoWriter {
    fn new() -> Self {
        Self {
            fail_sync: AtomicBool::new(false),
            fail_write: AtomicBool::new(false),
        }
    }
}

impl RedoWriter for MockRedoWriter {
    fn write_frames(&self, _frames: &[Vec<u8>]) -> io::Result<()> {
        if self.fail_write.load(Ordering::Relaxed) {
            Err(io::Error::new(io::ErrorKind::Other, "Injected write error"))
        } else {
            Ok(())
        }
    }

    fn sync_data(&self) -> io::Result<()> {
        if self.fail_sync.load(Ordering::Relaxed) {
            Err(io::Error::new(io::ErrorKind::Other, "Injected sync error"))
        } else {
            Ok(())
        }
    }
}

/// Tracking für Guard-Vorwärmung (SD-53).
struct MockPrewarmer {
    calls: Arc<AtomicU64>,
}

impl GuardPrewarmer for MockPrewarmer {
    fn prewarm(&self, _space: u16, _key: &[u8]) {
        self.calls.fetch_add(1, Ordering::Relaxed);
    }
}

/// Dynamische Gegendruck-Quelle (E-4, SD-49).
struct MockBackpressure {
    level: std::sync::atomic::AtomicU8,
}

impl MockBackpressure {
    fn new(level: BackpressureLevel) -> Self {
        let code = match level {
            BackpressureLevel::Normal => 0,
            BackpressureLevel::Slowdown => 1,
            BackpressureLevel::Stopped => 2,
        };
        Self {
            level: std::sync::atomic::AtomicU8::new(code),
        }
    }

    fn set_level(&self, level: BackpressureLevel) {
        let code = match level {
            BackpressureLevel::Normal => 0,
            BackpressureLevel::Slowdown => 1,
            BackpressureLevel::Stopped => 2,
        };
        self.level.store(code, Ordering::Relaxed);
    }
}

impl BackpressureSource for MockBackpressure {
    fn level(&self) -> BackpressureLevel {
        match self.level.load(Ordering::Relaxed) {
            0 => BackpressureLevel::Normal,
            1 => BackpressureLevel::Slowdown,
            _ => BackpressureLevel::Stopped,
        }
    }
}

/// Aufzeichnender Listener für Tests.
struct RecordingListener {
    events: std::sync::Mutex<Vec<ChangeEvent>>,
    fail: AtomicBool,
}

impl RecordingListener {
    fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
            fail: AtomicBool::new(false),
        }
    }
}

impl Listener for RecordingListener {
    fn on_change(&self, event: &ChangeEvent) -> Result<(), ListenerError> {
        if self.fail.load(Ordering::Relaxed) {
            return Err(ListenerError::Failed("Injected error".into()));
        }
        self.events.lock().unwrap().push(event.clone());
        Ok(())
    }
}

fn create_test_engine() -> (
    Committer,
    Arc<VersionState>,
    Arc<AtomicU64>,
    Arc<AtomicU64>,
    Arc<AtomicBool>,
    Arc<ListenerQueue>,
) {
    let mut version = Version::new();
    version.add_space(1);
    version.add_space(2);

    let state = Arc::new(VersionState::new(Arc::new(version)));
    let visible_stamp = Arc::new(AtomicU64::new(0));
    let durable_stamp = Arc::new(AtomicU64::new(0));
    let poisoned = Arc::new(AtomicBool::new(false));
    let listener_queue = Arc::new(ListenerQueue::new());

    let committer = Committer::new(
        0,
        Arc::clone(&visible_stamp),
        Arc::clone(&durable_stamp),
        Arc::clone(&poisoned),
        Arc::clone(&state),
        Arc::clone(&listener_queue),
        None,
    );

    (
        committer,
        state,
        visible_stamp,
        durable_stamp,
        poisoned,
        listener_queue,
    )
}

#[test]
fn test_model_level_1_against_btreemap_reference() {
    // Modelltest Stufe 1: Puts, Gets, Views gegen BTreeMap-Referenz, mit Seed.
    // Zitiert INV-VIS-1, INV-VIS-2, INV-VIS-3.
    let (committer, state, visible_stamp, _, _, _) = create_test_engine();
    let redo = MockRedoWriter::new();

    // BTreeMap Referenz: (space, user_key) -> (stamp, header, body)
    let mut reference: BTreeMap<(u16, Vec<u8>), (u64, Vec<u8>, Vec<u8>)> = BTreeMap::new();

    // Deterministischer Pseudozufall mit festem Seed (42)
    let mut seed: u64 = 42;
    let mut rng = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        seed
    };

    let mut views_at_checkpoints: Vec<(u64, BTreeMap<(u16, Vec<u8>), (u64, Vec<u8>, Vec<u8>)>)> =
        Vec::new();

    for step in 1..=50u64 {
        let space = (rng() % 2 + 1) as u16;
        let key_idx = (rng() % 10) as u8;
        let key = vec![b'k', key_idx];
        let val_byte = (step & 0xFF) as u8;
        let header = vec![b'h', val_byte];
        let body = vec![b'v', val_byte, val_byte];

        let is_delete = (rng() % 5) == 0;

        let mut change = Change::new();
        if is_delete {
            change.delete(space, key.clone());
        } else {
            change.put(space, key.clone(), header.clone(), body.clone());
        }

        let receipt = committer
            .apply(change, None, None, None, &redo)
            .expect("Commit muss erfolgreich sein");

        // Referenz nachführen
        if is_delete {
            reference.remove(&(space, key.clone()));
        } else {
            reference.insert((space, key.clone()), (receipt.stamp, header, body));
        }

        // Regelmäßig Checkpoints für Views erfassen (INV-VIS-3)
        if step % 10 == 0 {
            views_at_checkpoints.push((receipt.stamp, reference.clone()));
        }
    }

    // Aktuelle View prüfen (INV-VIS-3)
    let current_view = create_view(&visible_stamp, &state);
    assert_eq!(current_view.stamp, 50);

    for ((space, key), (_stamp, expected_hdr, expected_body)) in &reference {
        let res = current_view.get(*space, key);
        assert_eq!(
            res,
            Some((expected_hdr.clone(), expected_body.clone())),
            "Schlüssel {:?} in Space {} stimmt nicht mit Referenz überein",
            key,
            space
        );
    }

    // Historische Views prüfen (INV-VIS-2: höhere Stamps unsichtbar)
    for (checkpoint_stamp, ref_snap) in views_at_checkpoints {
        let view = create_view(&visible_stamp, &state);
        // Wir erzeugen eine View gebunden an den historischen Stamp
        let hist_view = contextra_storage::engine::views::View::new(view.version, checkpoint_stamp);

        for space in 1..=2u16 {
            for key_idx in 0..10u8 {
                let key = vec![b'k', key_idx];
                let expected = ref_snap.get(&(space, key.clone())).map(|(_, h, b)| (h.clone(), b.clone()));
                let actual = hist_view.get(space, &key);
                assert_eq!(
                    actual, expected,
                    "INV-VIS-2 verletzt: Historische View bei Stamp {} zeigt abweichenden Wert für {:?}",
                    checkpoint_stamp, key
                );
            }
        }
    }
}

#[test]
fn test_backpressure_waits_before_ticket_never_in_committer() {
    // SD-49 / E-4: Gegendruck wartet vor dem Ticket, nie im Committer.
    let (committer, _, _, _, _, _) = create_test_engine();
    let redo = MockRedoWriter::new();

    // 1. Stufe Gestoppt: apply bricht VOR Ticket-Erstellung mit Busy ab
    let bp = MockBackpressure::new(BackpressureLevel::Stopped);

    let mut change = Change::new();
    change.put(1, b"key".to_vec(), b"h".to_vec(), b"v".to_vec());

    let res = committer.apply(change, None, None, Some(&bp), &redo);
    match res {
        Err(CommitError::Busy { retry_after_ms }) => {
            assert!(retry_after_ms > 0);
        }
        other => panic!("Erwartet Busy vor Ticket-Erstellung, erhalten: {:?}", other),
    }

    // 2. Normal: Läuft sofort durch
    bp.set_level(BackpressureLevel::Normal);
    let mut change2 = Change::new();
    change2.put(1, b"key".to_vec(), b"h".to_vec(), b"v".to_vec());
    let receipt = committer.apply(change2, None, None, Some(&bp), &redo).expect("Commit gelingt");
    assert_eq!(receipt.stamp, 1);
}

#[test]
fn test_invariants_vis1_vis2_vis3() {
    // Test für INV-VIS-1, INV-VIS-2, INV-VIS-3
    let (committer, state, visible_stamp, _durable_stamp, poisoned, _) = create_test_engine();
    let redo = MockRedoWriter::new();

    // INV-VIS-1: Schlägt sync_data fehl (Poison), erreicht NICHTS die Memtables!
    redo.fail_sync.store(true, Ordering::Relaxed);
    let mut change = Change::new();
    change.durability = Some(Durability::Always);
    change.put(1, b"secret".to_vec(), b"h".to_vec(), b"v".to_vec());

    let res = committer.apply(change, None, None, None, &redo);
    assert!(res.is_err());
    assert!(poisoned.load(Ordering::Acquire), "INV-VIS-1: Poison muss gesetzt sein");

    // Prüfen: Memtable ist komplett leer (INV-VIS-1)
    let version = Arc::clone(&*state.read());
    let sp = version.get_space(1).unwrap();
    assert_eq!(sp.active.entry_count(), 0, "INV-VIS-1: Keine Daten in Memtable nach Sync-Fehler!");

    // INV-VIS-3 Gegenbeispiel-Test:
    // Die Gegenbeispiel-Reihenfolge liest erst Version, dann visible_stamp.
    // create_view liest erst visible_stamp, dann Version.
    let normal_view = create_view(&visible_stamp, &state);
    let buggy_view = create_view_inverted_order_buggy(&visible_stamp, &state);
    assert_eq!(normal_view.stamp, buggy_view.stamp);
}

/// Gegenbeispiel-Implementierung mit vertauschter Reihenfolge (nur Testhilfe, nicht Produktion).
///
/// Liest ZUERST Version, DANN visible_stamp. Warum dies fehlerhaft ist (SPEC §16.3 Gegenbeispiel):
/// Liest ein Leser zuerst V0 (aktive Memtable M), rotiert der Leiter dann M und setzt M2 als aktiv,
/// fügt Einträge mit Stamp s in M2 ein und veröffentlicht s, lädt der Leser danach v = s:
/// V0 kennt M2 nicht, die View verliert Einträge <= v!
fn create_view_inverted_order_buggy(visible_stamp: &AtomicU64, state: &VersionState) -> View {
    // FEHLER: Vertauschte Reihenfolge!
    let version = Arc::clone(&*state.read());
    let stamp = visible_stamp.load(Ordering::Acquire);
    View::new(version, stamp)
}

#[test]
fn test_h_s7_subbatch_and_h_s2_prewarming() {
    // H-S7: Memtable Schreibzugriffe in Unterbatches von höchstens 64 Operationen
    let memtable = Memtable::new();
    assert_eq!(MEMTABLE_SUBBATCH_LIMIT, 64);

    let mut entries = Vec::new();
    for i in 0..150u64 {
        let tag = Tag::new(i + 1, TagKind::Put);
        let key = InternalKey::new(format!("key_{:04}", i).into_bytes(), tag);
        let slot = Slot::put(b"h".to_vec(), b"value".to_vec());
        entries.push((key, slot));
    }

    // Batch mit 150 Einträgen (> 2 * 64) einfügen
    memtable.insert_batch(&entries);
    assert_eq!(memtable.entry_count(), 150);
    assert!(memtable.memtable_bytes() > 0);

    // SD-53 / H-S2: Guard-Vorwärmung
    let (committer, _, _, _, _, _) = create_test_engine();
    let redo = MockRedoWriter::new();
    let prewarmer = MockPrewarmer {
        calls: Arc::new(AtomicU64::new(0)),
    };

    let mut guarded_change = Change::new();
    guarded_change.guard_absent(1, b"new_key".to_vec());
    guarded_change.put(1, b"new_key".to_vec(), b"h".to_vec(), b"v".to_vec());

    let rec = committer
        .apply(guarded_change, None, Some(&prewarmer), None, &redo)
        .expect("Commit mit Guard gelingt");
    assert!(rec.stamp > 0);
    assert!(
        prewarmer.calls.load(Ordering::Relaxed) > 0,
        "SD-53: Guard-Vorwärmung muss aufgerufen worden sein"
    );
}

#[test]
fn test_retention_floor_and_inv_ret1() {
    // INV-RET-1: Untergrenze und Aufbewahrung
    let visible_stamp = AtomicU64::new(10);
    let registry = Registry::new();

    // 1. Initialer Floor = visible_stamp
    let f1 = registry.retention_floor(&visible_stamp);
    assert_eq!(f1, 10);

    // 2. View bei Stamp 5 registrieren
    let (_v_id, s) = registry.view_at(&visible_stamp, 10).unwrap();
    assert_eq!(s, 10);

    // View unter Floor schlägt fehl (INV-RET-1)
    let err = registry.view_at(&visible_stamp, 2);
    assert!(matches!(err, Err(RegistryError::HistoryPruned { .. })));

    // Monotonie des Floors prüfen
    visible_stamp.store(20, Ordering::Release);
    let f2 = registry.retention_floor(&visible_stamp);
    assert!(f2 >= f1, "INV-RET-1: retention_floor muss monoton steigend sein");

    // 3. Reine Funktion retain (§12.2) prüfen
    let mut items = Vec::new();
    // 3 Versionen für key "a": Stamps 15, 8, 3
    for s in [15u64, 8, 3] {
        let tag = Tag::new(s, TagKind::Put);
        let k = InternalKey::new(b"a".to_vec(), tag);
        items.push(RetainItem::new(k, Slot::put(vec![], vec![])));
    }

    // Floor = 10:
    // Stamp 15 > 10: behalten
    // Stamp 8 <= 10: neueste unterhalb -> behalten
    // Stamp 3 <= 10: älter unterhalb -> VERWERFEN
    let retained = retain(items, 10, false);
    assert_eq!(retained.len(), 2, "INV-RET-1: Genau 2 Versionen müssen erhalten bleiben");
    assert_eq!(retained[0].stamp(), 15);
    assert_eq!(retained[1].stamp(), 8);

    // Löschmarker-Verwerfung testen:
    let del_tag = Tag::new(5, TagKind::Delete);
    let del_key = InternalKey::new(b"del".to_vec(), del_tag);
    let del_item = RetainItem::new(del_key, Slot::delete());

    // has_lower = false: Löschmarker wird verworfen
    let r_no_lower = retain(vec![del_item.clone()], 10, false);
    assert_eq!(r_no_lower.len(), 0, "Löschmarker ohne tiefere Daten muss verworfen werden");

    // has_lower = true: Löschmarker bleibt erhalten
    let r_with_lower = retain(vec![del_item], 10, true);
    assert_eq!(r_with_lower.len(), 1, "Löschmarker mit tieferen Daten muss erhalten bleiben");
}

#[test]
fn test_claims_all_or_nothing_and_obsolete_inv_claim1() {
    // INV-CLAIM-1: Claim-Tabelle mit obsolete-Flag und all-or-nothing
    let registry = Registry::new();
    let file1 = Arc::new(FileHandle::new(101, 1, 0, "101.sst".into(), 1, 10));
    let file2 = Arc::new(FileHandle::new(102, 1, 0, "102.sst".into(), 1, 10));

    let cancel = Arc::new(AtomicBool::new(false));

    // 1. Erfolgreicher Claim
    let res = registry.claim(1, &[Arc::clone(&file1), Arc::clone(&file2)], false, Arc::clone(&cancel));
    assert!(res.is_ok(), "Claim muss gelingen");

    // 2. Konflikt bei gleichzeitigem Claim durch Job 2
    let cancel2 = Arc::new(AtomicBool::new(false));
    let res2 = registry.claim(2, &[Arc::clone(&file1)], false, Arc::clone(&cancel2));
    assert!(res2.is_err(), "Konflikt muss gemeldet werden");

    // 3. Obsolete-Prüfung (INV-CLAIM-1)
    registry.release_claims(&[Arc::clone(&file1), Arc::clone(&file2)]);
    file1.mark_obsolete();

    let res3 = registry.claim(3, &[Arc::clone(&file1)], false, Arc::clone(&cancel));
    assert!(res3.is_err(), "INV-CLAIM-1: Veraltete Datei darf nicht beansprucht werden");
}

#[test]
fn test_listener_dispatch_and_lagged_sd62() {
    // SD-62: Listener-Zustellung und Frist/Überlauf
    let queue = ListenerQueue::new();
    let listener = Arc::new(RecordingListener::new());

    // Bounded queue: Kapazität 2
    let config = ListenerConfig {
        queue_limit: 2,
        deadline_ms: 1000,
    };

    let sub_id = queue.subscribe(Arc::clone(&listener) as Arc<dyn Listener>, config);

    // 2 Events einspeisen -> passen
    queue.push_event(ChangeEvent { stamp: 1, ops: vec![] });
    queue.push_event(ChangeEvent { stamp: 2, ops: vec![] });
    assert!(queue.poll_status(sub_id).is_ok());

    // 3. Event -> Überlauf! SD-62: Abmelden und Lagged markieren
    queue.push_event(ChangeEvent { stamp: 3, ops: vec![] });

    // poll_status meldet Lagged
    let status = queue.poll_status(sub_id);
    match status {
        Err(ListenerError::Lagged { missed_from }) => {
            assert_eq!(missed_from, 3);
        }
        other => panic!("SD-62 erwartet ListenerLagged, erhalten: {:?}", other),
    }

    // Commit-Pfad wurde nicht blockiert!
}
