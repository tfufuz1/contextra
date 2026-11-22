//! # Loom-Modell 2: Sichtbarkeit mit Rotation
//!
//! Normative Referenzen: SPEC §10.3, §16.3 (INV-VIS-1, INV-VIS-2, INV-VIS-3), §16.7.
//! Verifiziert das Sichtbarkeitsprotokoll und belegt das Gegenbeispiel mit vertauschter
//! Leseordnung als erwarteten Fehler.

use contextra_storage::sync::{Arc, AtomicU64, Ordering};

#[cfg(loom)]
use loom::thread;
#[cfg(not(loom))]
use std::thread;

use contextra_storage::codec::tag::{InternalKey, Tag, TagKind};
use contextra_storage::engine::memtable::Slot;
use contextra_storage::engine::version::{Version, VersionState};
use contextra_storage::engine::views::create_view;

#[test]
fn test_loom_vis_rotation_inv_vis_3_correct_order() {
    // Beweis und Test für INV-VIS-3:
    // Schreiber rotiert und veröffentlicht Stamps; Leser lädt ZUERST visible_stamp,
    // DANN die Version. Für jeden sichtbaren Stamp v muss die View den Eintrag enthalten!
    let run_test = || {
        let mut initial_version = Version::new();
        initial_version.add_space(1);

        let state = Arc::new(VersionState::new(Arc::new(initial_version)));
        let visible_stamp = Arc::new(AtomicU64::new(0));

        let writer_state = Arc::clone(&state);
        let writer_vis = Arc::clone(&visible_stamp);

        let reader_state = Arc::clone(&state);
        let reader_vis = Arc::clone(&visible_stamp);

        let writer_handle = thread::spawn(move || {
            // Schritt 1: Eintrag 1 in aktive Memtable M1 einfügen
            {
                let v = Arc::clone(&*writer_state.read());
                let sp = v.get_space(1).unwrap();
                let tag = Tag::new(1, TagKind::Put);
                let k = InternalKey::new(b"key".to_vec(), tag);
                sp.active.insert_batch(&[(k, Slot::put(b"h1".to_vec(), b"v1".to_vec()))]);
            }
            writer_vis.store(1, Ordering::Release);

            // Schritt 2: Rotation (M1 -> frozen, M2 neu aktiv)
            {
                let mut v_guard = writer_state.write();
                let mut new_ver = (**v_guard).clone();
                if let Some(sp) = new_ver.spaces.get_mut(&1) {
                    sp.rotate();
                }
                *v_guard = Arc::new(new_ver);
            }

            // Schritt 3: Eintrag 2 in aktive Memtable M2 einfügen
            {
                let v = Arc::clone(&*writer_state.read());
                let sp = v.get_space(1).unwrap();
                let tag = Tag::new(2, TagKind::Put);
                let k = InternalKey::new(b"key".to_vec(), tag);
                sp.active.insert_batch(&[(k, Slot::put(b"h2".to_vec(), b"v2".to_vec()))]);
            }
            writer_vis.store(2, Ordering::Release);
        });

        let reader_handle = thread::spawn(move || {
            // INV-VIS-3: Leser lädt ZUERST visible_stamp, DANN Version
            let view = create_view(&reader_vis, &reader_state);

            if view.stamp == 0 {
                // Noch nichts sichtbar
            } else if view.stamp == 1 {
                // Muss v1 sehen
                let val = view.get(1, b"key");
                assert_eq!(
                    val,
                    Some((b"h1".to_vec(), b"v1".to_vec())),
                    "INV-VIS-3: View mit Stamp 1 muss v1 sehen"
                );
            } else if view.stamp == 2 {
                // Muss v2 sehen
                let val = view.get(1, b"key");
                assert_eq!(
                    val,
                    Some((b"h2".to_vec(), b"v2".to_vec())),
                    "INV-VIS-3: View mit Stamp 2 muss v2 sehen"
                );
            }
        });

        writer_handle.join().unwrap();
        reader_handle.join().unwrap();
    };

    #[cfg(loom)]
    {
        loom::model(run_test);
    }
    #[cfg(not(loom))]
    {
        run_test();
    }
}

#[test]
fn test_loom_vis_counterexample_inverted_order_fails() {
    // Gegenbeispiel nach SPEC §16.3:
    // Vertauschte Leseordnung (Version VOR visible_stamp) führt bei paralleler
    // Rotation zu Datenverlust: Der Leser liest Version V0 (nur M1), der Schreiber
    // rotiert auf M2, fügt Stamp 2 in M2 ein und setzt visible_stamp = 2.
    // Liest der Leser danach visible_stamp = 2, besitzt seine View nur V0 (kennt M2 nicht)
    // und verliert den Eintrag bei Stamp 2!

    let mut initial_version = Version::new();
    initial_version.add_space(1);

    let state = Arc::new(VersionState::new(Arc::new(initial_version)));
    let visible_stamp = Arc::new(AtomicU64::new(0));

    // Simulation des fehlerhaften Ablaufs:
    // 1. Leser liest ZUERST Version V0
    let v0 = Arc::clone(&*state.read());

    // 2. Schreiber rotiert Version auf M2 und fügt Stamp 2 in M2 ein
    {
        let mut v_guard = state.write();
        let mut new_ver = (**v_guard).clone();
        let sp = new_ver.spaces.get_mut(&1).unwrap();
        sp.rotate();
        let tag = Tag::new(2, TagKind::Put);
        let k = InternalKey::new(b"key".to_vec(), tag);
        sp.active.insert_batch(&[(k, Slot::put(b"h2".to_vec(), b"v2".to_vec()))]);
        *v_guard = Arc::new(new_ver);
    }
    visible_stamp.store(2, Ordering::Release);

    // 3. Leser lädt DANACH visible_stamp = 2
    let stamp = visible_stamp.load(Ordering::Acquire);
    let buggy_view = contextra_storage::engine::views::View::new(v0, stamp);

    // Erwarteter Fehler: Die buggy View hat Stamp 2, sieht aber NICHT den Wert v2 aus M2!
    let found = buggy_view.get(1, b"key");
    assert_eq!(
        found, None,
        "Gegenbeispiel belegt: Vertauschte Reihenfolge verliert Einträge (erwarteter Fehler)!"
    );
}
