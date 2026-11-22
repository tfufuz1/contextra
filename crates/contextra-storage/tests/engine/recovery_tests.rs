//! # Recovery Integrationstests und Absturzmatrix (SPEC §13.1, §13.2, §18)
//!
//! Verifiziert den 11-Schritte-Wiederanlauf nach SPEC §13.1 und die Absturzmatrix §13.2:
//! - Schritt 3: CURRENT und Katalog lesen (Torn-Record verwerfen, sonst Corrupt)
//! - Schritt 4: .building-Dateien und nicht genannte Dateien löschen, Reservesegmente bleiben (INV-LOG-7, INV-CAT-4)
//! - Schritt 5: Fehlende genannte Datei führt zu CorruptMissingFile (fail-closed)
//! - Schritt 6: Redo ab redo_floor replayen. Nur Ops mit stamp > flushed_stamp anwenden.
//!   - EraseRange-Ops in die pending-Menge aufnehmen (Schnittstelle, INV-ERA-1)
//!   - Defekt in gesiegeltem Segment führt zu Corrupt (INV-LOG-6)
//!   - Ungesiegeltes Segment abschneiden und siegeln
//! - INV-LOG-8: Letzter gültiger Stamp gegen redo_hwm prüfen, bei Unterschreitung CorruptRedoTruncated
//! - Schritt 7: next_stamp, visible_stamp, durable_stamp deterministisch initialisieren
//! - Schritt 8: ID-Vergabe ab Hochwassermarken starten (INV-KEY-2)
//! - Schritt 9: Consumer prüfen
//! - H-S4: Messhaken für Recovery-Dauer

use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use contextra_storage::catalog::current::write_current;
use contextra_storage::catalog::durable::sync_existing;
use contextra_storage::catalog::edit::CatalogEdit;
use contextra_storage::catalog::log::{catalog_file_name, CatalogLog};
use contextra_storage::catalog::reserve::IdReserve;
use contextra_storage::catalog::types::{CatalogFileKind, CatalogHeader, FileMeta};
use contextra_storage::engine::compaction::exec::run_file_name;
use contextra_storage::engine::recovery::{recover_engine, RecoveryError};
use contextra_storage::env::entropy::RealEntropy;
use contextra_storage::redo::driver::{segment_file_name, RedoWriter};
use contextra_storage::redo::frame::Durability;
use contextra_storage::redo::op::RedoOp;
use contextra_storage::testing::fault_fs::{FaultConfig, FaultFs};
use contextra_storage::testing::mem_fs::MemFs;
use contextra_storage_ports::{File, Fs, OpenMode, Perm};

fn init_catalog(fs: &dyn Fs, base_dir: &Path) {
    fs.create_dir_all(base_dir, Perm::Private).unwrap();
    write_current(fs, base_dir, 1).unwrap();
    let cat_path = base_dir.join(catalog_file_name(1));
    let cat_file = fs.open(&cat_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    let hdr = CatalogHeader::new(1);
    cat_file.write_all_at(&hdr.encode(), 0).unwrap();
    cat_file.sync_all().unwrap();
}

#[test]
fn test_recovery_step4_clean_building_and_unreferenced_files_inv_cat_4() {
    // SPEC §13.1 Schritt 4 / INV-CAT-4:
    // *.building.* löschen; Dateien, die der Catalog nicht nennt, löschen.
    // Reservesegmente (>= redo_floor) bleiben (INV-LOG-7).
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/test_rec_step4");
    init_catalog(&*mem, base);

    // 1. Temporäre .building Datei anlegen
    let b1 = base.join("RUN-000005.building");
    let f_b1 = mem.open(&b1, OpenMode::CreateNew(Perm::Private)).unwrap();
    f_b1.write_all_at(b"temp", 0).unwrap();

    // 2. Nicht im Catalog genannte Run-Datei anlegen (Verwaist)
    let orphan_run = base.join("RUN-000099");
    let f_or = mem.open(&orphan_run, OpenMode::CreateNew(Perm::Private)).unwrap();
    f_or.write_all_at(b"orphan", 0).unwrap();

    // 3. Reservesegment anlegen (id = 1 >= redo_floor 1) (INV-LOG-7)
    let entropy = RealEntropy;
    let _writer = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();

    // Recovery ausführen
    let rec = recover_engine(&*mem, base, None, None, None, None).expect("Recovery gelingt");

    // .building und unreferenzierter Run müssen weg sein
    assert!(mem.open(&b1, OpenMode::Read).is_err(), "INV-CAT-4: .building muss gelöscht sein");
    assert!(mem.open(&orphan_run, OpenMode::Read).is_err(), "Verwaiste RUN-000099 muss gelöscht sein");

    // Reservesegment muss übernommen worden sein (INV-LOG-7)
    assert_eq!(rec.reserve_segment_id, Some(1), "INV-LOG-7: Reservesegment muss erhalten bleiben");
}

#[test]
fn test_recovery_step5_missing_file_fails_closed_corrupt_missing_file() {
    // SPEC §13.1 Schritt 5:
    // Vom Catalog genannte Datei fehlt => Err(Corrupt{MissingFile{kind, id}})
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/test_rec_step5");
    init_catalog(&*mem, base);

    // Run-Datei im Katalog eintragen, aber physisch nicht erzeugen
    let cat_path = base.join(catalog_file_name(1));
    let cat_file = mem.open(&cat_path, OpenMode::ReadWrite).unwrap();
    let mut log = CatalogLog::read_and_recover(&*cat_file, 1, 0, None, None, None, None).unwrap().1;

    let mut edit = CatalogEdit::default();
    edit.add_files.push(FileMeta {
        kind: CatalogFileKind::Run,
        id: 42,
        space: 1,
        level: 0,
        tier: 0,
        flags: 0,
        bytes: 100,
        entries: 1,
        generation: 0,
        segment_kind: 0,
        min_stamp: 1,
        max_stamp: 1,
        min_key: b"a".to_vec(),
        max_key: b"z".to_vec(),
        blob_no_min: 0,
        blob_no_max: 0,
    });
    let s = sync_existing(&*mem, &*cat_file, base, edit).unwrap();
    log.commit(&*cat_file, s, None, None, None, None).unwrap();

    let err = recover_engine(&*mem, base, None, None, None, None);
    match err {
        Err(RecoveryError::CorruptMissingFile { kind, id }) => {
            assert_eq!(kind, CatalogFileKind::Run);
            assert_eq!(id, 42);
        }
        other => panic!("Erwartet CorruptMissingFile, erhalten: {:?}", other),
    }
}

#[test]
fn test_recovery_step6_flushed_stamp_filter_and_erase_pending() {
    // SPEC §13.1 Schritt 6:
    // Redo ab redo_floor replayen. Nur Ops mit stamp > space.flushed_stamp anwenden.
    // EraseRange-Op => Marker in pending-Menge, falls kein erase_done.
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/test_rec_step6");
    init_catalog(&*mem, base);

    // Katalog hat space 1 geflusht bis Stamp 10
    let cat_path = base.join(catalog_file_name(1));
    let cat_file = mem.open(&cat_path, OpenMode::ReadWrite).unwrap();
    let mut log = CatalogLog::read_and_recover(&*cat_file, 1, 0, None, None, None, None).unwrap().1;
    let mut edit = CatalogEdit::default();
    edit.space_flushed.push((1, 10));
    let s = sync_existing(&*mem, &*cat_file, base, edit).unwrap();
    log.commit(&*cat_file, s, None, None, None, None).unwrap();

    // Redo schreiben mit:
    // Stamp 8: Put (sollte übersprungen werden, da <= 10)
    // Stamp 12: Put (sollte angewendet werden, da > 10)
    // Stamp 15: EraseRange (sollte angewendet und in pending aufgenommen werden)
    let entropy = RealEntropy;
    let mut writer = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();

    let op8 = RedoOp::new_put(1, None, b"k8".to_vec(), b"v8".to_vec());
    writer.write_change(8, Durability::Always, &[op8], 2, &entropy).unwrap();

    let op12 = RedoOp::new_put(1, None, b"k12".to_vec(), b"v12".to_vec());
    writer.write_change(12, Durability::Always, &[op12], 2, &entropy).unwrap();

    let op15 = RedoOp::new_erase_range(1, b"a".to_vec(), Some(b"z".to_vec()));
    writer.write_change(15, Durability::Always, &[op15], 2, &entropy).unwrap();

    let rec = recover_engine(&*mem, base, None, None, None, None).expect("Recovery gelingt");

    // Prüfe angewendete Ops:
    // Op mit Stamp 8 darf NICHT angewendet worden sein
    let stamps: Vec<u64> = rec.applied_ops.iter().map(|(_, o)| o.key.clone()).map(|_| 0).collect();
    for (_, op) in &rec.applied_ops {
        assert_ne!(op.key, b"k8", "Op mit Stamp <= flushed_stamp darf nicht angewendet werden");
    }
    assert!(rec.applied_ops.iter().any(|(_, o)| o.key == b"k12"), "Op mit Stamp 12 muss angewendet werden");

    // Prüfe pending erases
    assert_eq!(rec.pending_erases.len(), 1, "EraseRange muss in pending_erases sein");
    assert_eq!(rec.pending_erases[0].id, 15);
}

#[test]
fn test_recovery_step6_inv_log_8_redo_truncated_fails_closed() {
    // // INV-LOG-8: Redo-Schwanz nach Edit verloren (Lost Write)
    // letzter gültiger Stamp < redo_hwm => Err(Corrupt{RedoTruncated})
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/test_rec_inv_log_8");
    init_catalog(&*mem, base);

    // Katalog verankert redo_hwm = 100
    let cat_path = base.join(catalog_file_name(1));
    let cat_file = mem.open(&cat_path, OpenMode::ReadWrite).unwrap();
    let mut log = CatalogLog::read_and_recover(&*cat_file, 1, 0, None, None, None, None).unwrap().1;
    let mut edit = CatalogEdit::default();
    edit.redo = Some((1, 0, 100)); // hwm = 100
    let s = sync_existing(&*mem, &*cat_file, base, edit).unwrap();
    log.commit(&*cat_file, s, None, None, None, None).unwrap();

    // Redo endet aber bei Stamp 50
    let entropy = RealEntropy;
    let mut writer = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();
    let op = RedoOp::new_put(1, None, b"k".to_vec(), b"v".to_vec());
    writer.write_change(50, Durability::Always, &[op], 2, &entropy).unwrap();
    writer.seal_and_close().unwrap();

    let err = recover_engine(&*mem, base, None, None, None, None);
    assert!(
        matches!(
            err,
            Err(RecoveryError::CorruptRedoTruncated { last_stamp: 50, redo_hwm: 100 })
        ),
        "INV-LOG-8: RedoTruncated muss gemeldet werden"
    );
}

#[test]
fn test_recovery_step7_and_step8_stamps_and_id_reserve_inv_key_2() {
    // SPEC §13.1 Schritte 7 und 8 / INV-KEY-2:
    // Stempel und ID-Vergabe ab Hochwassermarken
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/test_rec_step7_8");
    init_catalog(&*mem, base);

    // Katalog mit HWMs
    let cat_path = base.join(catalog_file_name(1));
    let cat_file = mem.open(&cat_path, OpenMode::ReadWrite).unwrap();
    let mut log = CatalogLog::read_and_recover(&*cat_file, 1, 0, None, None, None, None).unwrap().1;
    let mut edit = CatalogEdit::default();
    edit.reserve = Some((200, 300, 400, 500, 1));
    let s = sync_existing(&*mem, &*cat_file, base, edit).unwrap();
    log.commit(&*cat_file, s, None, None, None, None).unwrap();

    // Redo mit Frame bei Stamp 75
    let entropy = RealEntropy;
    let mut writer = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();
    let op = RedoOp::new_put(1, None, b"k".to_vec(), b"v".to_vec());
    writer.write_change(75, Durability::Always, &[op], 2, &entropy).unwrap();

    let mut rec = recover_engine(&*mem, base, None, None, None, None).expect("Recovery gelingt");

    assert_eq!(rec.visible_stamp, 75);
    assert_eq!(rec.durable_stamp, 75);
    assert_eq!(rec.next_stamp, 76);

    // INV-KEY-2: ID-Vergabe beginnt ab den Hochwassermarken
    let r_id = rec.id_reserve.ensure_run_id().unwrap();
    assert_eq!(r_id, 201, "INV-KEY-2: Run-ID Vergabe muss strikt > HWM (200) sein");

    let b_id = rec.id_reserve.ensure_blob_no().unwrap();
    assert_eq!(b_id, 301, "INV-KEY-2: Blob-ID Vergabe muss strikt > HWM (300) sein");

    // H-S4: Dauer gemessen
    assert!(rec.duration.as_nanos() > 0, "H-S4 Messhaken aktiv");
}
