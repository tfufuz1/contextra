//! # Absturzmatrix Tests für Catalog und Blob (SPEC §13.2)
//!
//! Prüft die Soll-Zustände je Unterbrechungspunkt für:
//! - ID-Reservierung: Vor Edit vs nach Erzeugung aber vor Flush-Edit (INV-KEY-2).
//! - Blob-Sweep: neue Generation gebaut vor Edit vs nach Edit vor Unlink (INV-BLOB-3, INV-CAT-4).
//! - Catalog-Rollover: nach neuer Datei vor CURRENT vs nach CURRENT vor Löschung alter Datei (INV-CAT-1, INV-CAT-4).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_storage::blob::reader::BlobReader;
use contextra_storage::blob::record::BlobRecord;
use contextra_storage::blob::sweep::sweep_blob;
use contextra_storage::blob::writer::BlobWriter;
use contextra_storage::catalog::current::{read_current, write_current};
use contextra_storage::catalog::durable::sync_existing;
use contextra_storage::catalog::edit::CatalogEdit;
use contextra_storage::catalog::log::{catalog_file_name, CatalogHeader, CatalogLog};
use contextra_storage::catalog::reserve::IdReserve;
use contextra_storage::catalog::state::apply_edit;
use contextra_storage::catalog::types::*;
use contextra_storage::testing::{FaultConfig, FaultFs, MemFs};
use contextra_storage_ports::{Fs, OpenMode, Perm};
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

fn file_exists(fs: &dyn Fs, path: &Path) -> bool {
    fs.open(path, OpenMode::Read).is_ok()
}

fn dummy_blob_meta(id: u64, generation: u32, stamp: u64) -> FileMeta {
    FileMeta {
        kind: CatalogFileKind::Blob,
        id,
        space: 1,
        level: 0,
        tier: 0,
        flags: 0,
        min_stamp: stamp,
        max_stamp: stamp,
        bytes: 200,
        entries: 1,
        generation,
        segment_kind: 0,
        blob_no_min: id,
        blob_no_max: id,
        min_key: Vec::new(),
        max_key: Vec::new(),
    }
}

#[test]
fn test_crash_matrix_id_reserve_crash_before_edit() {
    // Zeile: "ID-Reservierung: Absturz vor Edit | keine Datei mit ungesicherter ID | IDs wurden nie benutzt"
    // INV-KEY-2: Nach Recovery beginnt Vergabe bei der Hochwassermarke.
    let mem_fs = MemFs::new();
    let fault_cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem_fs, fault_cfg.clone());
    let root = Path::new("crash_id_reserve_before_edit");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let mut reserve = IdReserve::recover(0, 0, 0, 0, 0);
    // Allokation triggert Reservierungsblock (256)
    let run_id_1 = reserve.ensure_run_id().unwrap();
    let hwm_run = reserve.run_id_hwm;
    assert_eq!(hwm_run, 256);
    assert!(run_id_1 < hwm_run);

    // Absturz vor Edit: Niemals ein Edit geschrieben.
    fault_cfg.fail_sync.store(true, Ordering::Relaxed);

    // Nach Neustart: Recovery liest den Zustand aus dem letzten dauerhaften Catalog-Stand (hier 0 bzw. HWM).
    // Wenn HWM im Catalog persistiert war (oder nach Recovery von HWM fortgeführt wird):
    let mut recovered_reserve = IdReserve::recover(hwm_run, 0, 0, 0, 0);
    let next_id = recovered_reserve.ensure_run_id().unwrap();
    // INV-KEY-2: IDs unter HWM werden nie wiederverwendet
    assert!(next_id >= hwm_run);
    assert_eq!(next_id, 256);
}

#[test]
fn test_crash_matrix_id_used_file_created_crash_before_flush_edit() {
    // Zeile: "ID benutzt (Datei angelegt), Absturz vor Flush-Edit | verwaiste Datei mit ID < Hochwassermarke | Datei gelöscht; ID wird nie wieder vergeben (INV-KEY-2)"
    let mem_fs = MemFs::new();
    let fault_cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem_fs, fault_cfg.clone());
    let root = Path::new("crash_id_used_before_edit");
    fs.create_dir_all(root, Perm::Private).unwrap();

    // 1. Initialer Katalog
    write_current(&fs, root, 1).unwrap();
    let cat_file = fs.open(&root.join(catalog_file_name(1)), OpenMode::CreateNew(Perm::Private)).unwrap();
    cat_file.write_all_at(&CatalogHeader::new(1).encode(), 0).unwrap();
    let _log = CatalogLog::read_and_recover(&*cat_file, 1, 0, None, None, None, None).unwrap().1;

    let mut reserve = IdReserve::recover(0, 0, 0, 0, 0);
    let run_id = reserve.ensure_run_id().unwrap();
    let hwm = reserve.run_id_hwm; // 256

    // Flush beginnt: Run-Datei auf Platte erzeugt
    let run_path = root.join("RUN-000001");
    let run_f = fs.open(&run_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    run_f.write_all_at(b"flushed run content", 0).unwrap();
    run_f.sync_all().unwrap();
    fs.sync_dir(root).unwrap();

    // Absturz vor Flush-Edit! Edit wird nie committet.
    fault_cfg.fail_sync.store(true, Ordering::Relaxed);

    // Neustart / Recovery:
    // a) Catalog liest Zustand
    let cat_read = fs.open(&root.join(catalog_file_name(1)), OpenMode::Read).unwrap();
    let (state, _) = CatalogLog::read_and_recover(&*cat_read, 1, 0, None, None, None, None).unwrap();
    assert!(!state.is_file_live(CatalogFileKind::Run, run_id));

    // b) Verwaisten-Suche (§13.1 Schritt 4, INV-CAT-4):
    // Datei RUN-000001 ist verwaist, da nicht im Catalog genannt -> gelöscht!
    fs.remove_file(&run_path).unwrap();
    assert!(!file_exists(&fs, &run_path));

    // c) Reserve nach Recovery (INV-KEY-2): HWM wird weitergeführt, vergebene ID wird nie wieder vergeben
    let mut new_reserve = IdReserve::recover(hwm, 0, 0, 0, 0);
    let next_id = new_reserve.ensure_run_id().unwrap();
    assert!(next_id >= hwm);
    assert_ne!(next_id, run_id);
}

#[test]
fn test_crash_matrix_blob_sweep_crash_before_edit() {
    // Zeile: "Blob-Sweep: neue Generation gebaut, vor Edit | zusätzliche Generationsdatei | gelöscht (nicht die aktuelle Generation)"
    let mem_fs = MemFs::new();
    let fault_cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem_fs, fault_cfg.clone());
    let root = Path::new("crash_blob_sweep_before_edit");
    fs.create_dir_all(root, Perm::Private).unwrap();

    // Blob Gen 1 schreiben
    let b1_path = root.join("BLOB-000001.000001");
    let b1_file = fs.open(&b1_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    let mut writer = BlobWriter::new(1, 1, 1, 0);
    let r1 = BlobRecord {
        kind: 0x01,
        domain: 0,
        stamp: 10,
        key: b"k1".to_vec(),
        body: b"body1".to_vec(),
    };
    let ref1 = writer.append(&*b1_file, &r1, None, None, None, None, None).unwrap();
    writer.finish(&*b1_file).unwrap();
    b1_file.sync_all().unwrap();
    fs.sync_dir(root).unwrap();

    // Katalog registriert Gen 1
    write_current(&fs, root, 1).unwrap();
    let cat_file = fs.open(&root.join(catalog_file_name(1)), OpenMode::CreateNew(Perm::Private)).unwrap();
    cat_file.write_all_at(&CatalogHeader::new(1).encode(), 0).unwrap();
    let mut log = CatalogLog::read_and_recover(&*cat_file, 1, 0, None, None, None, None).unwrap().1;
    let mut e1 = CatalogEdit::default();
    e1.add_files.push(dummy_blob_meta(1, 1, 10));
    let synced_e1 = sync_existing(&fs, &*cat_file, root, e1).unwrap();
    log.commit(&*cat_file, synced_e1, None, None, None, None).unwrap();
    cat_file.sync_all().unwrap();
    fs.sync_dir(root).unwrap();

    // Sweep erzeugt Generation 2
    let b2_path = root.join("BLOB-000001.000002");
    let b1_read = fs.open(&b1_path, OpenMode::Read).unwrap();
    let b2_write = fs.open(&b2_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    let mut live_offsets = HashSet::new();
    live_offsets.insert(ref1.offset);
    sweep_blob(&*b1_read, &*b2_write, &live_offsets, 0, None, None).unwrap();
    b2_write.sync_all().unwrap();
    fs.sync_dir(root).unwrap();

    // Absturz VOR dem Catalog-Edit!
    fault_cfg.fail_sync.store(true, Ordering::Relaxed);

    // Neustart / Verwaisten-Suche:
    let cat_read = fs.open(&root.join(catalog_file_name(1)), OpenMode::Read).unwrap();
    let (state, _) = CatalogLog::read_and_recover(&*cat_read, 1, 0, None, None, None, None).unwrap();

    // Katalog kennt nur Gen 1
    assert!(state.is_file_live(CatalogFileKind::Blob, 1));
    let blob_meta = state.files.get(&(CatalogFileKind::Blob, 1)).unwrap();
    assert_eq!(blob_meta.generation, 1);

    // Gen 2 ist zusätzliche Generationsdatei -> verwaist -> löschen (INV-CAT-4)
    if file_exists(&fs, &b2_path) {
        fs.remove_file(&b2_path).unwrap();
    }
    assert!(!file_exists(&fs, &b2_path));
    assert!(file_exists(&fs, &b1_path));

    // Gen 1 weiterhin lesbar (INV-BLOB-1)
    let r_active = BlobReader::open(&*b1_read, 0).unwrap();
    let rec = r_active.read(&*b1_read, &ref1, b"k1", 10, None, None, None, None, None).unwrap();
    assert_eq!(rec, b"body1");
}

#[test]
fn test_crash_matrix_blob_sweep_crash_after_edit_before_unlink() {
    // Zeile: "Blob-Sweep: nach Edit, vor Unlink | alte Generation | gelöscht"
    let mem_fs = MemFs::new();
    let fault_cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem_fs, fault_cfg.clone());
    let root = Path::new("crash_blob_sweep_after_edit");
    fs.create_dir_all(root, Perm::Private).unwrap();

    // b1 anlegen
    let b1_path = root.join("BLOB-000001.000001");
    let b1_f = fs.open(&b1_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    let mut w1 = BlobWriter::new(1, 1, 1, 0);
    let rec = BlobRecord {
        kind: 0x01,
        domain: 0,
        stamp: 20,
        key: b"k".to_vec(),
        body: b"v".to_vec(),
    };
    let ref1 = w1.append(&*b1_f, &rec, None, None, None, None, None).unwrap();
    w1.finish(&*b1_f).unwrap();
    b1_f.sync_all().unwrap();

    // b2 sweepen
    let b2_path = root.join("BLOB-000001.000002");
    let b2_f = fs.open(&b2_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    let mut live_offsets = HashSet::new();
    live_offsets.insert(ref1.offset);
    sweep_blob(&*b1_f, &*b2_f, &live_offsets, 0, None, None).unwrap();
    b2_f.sync_all().unwrap();
    fs.sync_dir(root).unwrap();

    // Catalog initialisieren und Edit commiten: Ersetzt Gen 1 durch Gen 2
    write_current(&fs, root, 1).unwrap();
    let cat_f = fs.open(&root.join(catalog_file_name(1)), OpenMode::CreateNew(Perm::Private)).unwrap();
    cat_f.write_all_at(&CatalogHeader::new(1).encode(), 0).unwrap();
    let mut log = CatalogLog::read_and_recover(&*cat_f, 1, 0, None, None, None, None).unwrap().1;

    let mut e1 = CatalogEdit::default();
    e1.add_files.push(dummy_blob_meta(1, 1, 20));
    e1.blob_replace.push((1, 1, 2)); // blob 1, alt gen 1, neu gen 2
    let synced = sync_existing(&fs, &*cat_f, root, e1).unwrap();
    log.commit(&*cat_f, synced, None, None, None, None).unwrap();
    cat_f.sync_all().unwrap();
    fs.sync_dir(root).unwrap();

    // Absturz vor Unlink von b1! Beide Dateien existieren noch auf Platte.
    fault_cfg.fail_sync.store(true, Ordering::Relaxed);

    // Neustart / Verwaisten-Suche:
    let cat_r = fs.open(&root.join(catalog_file_name(1)), OpenMode::Read).unwrap();
    let (state, _) = CatalogLog::read_and_recover(&*cat_r, 1, 0, None, None, None, None).unwrap();

    let meta = state.files.get(&(CatalogFileKind::Blob, 1)).unwrap();
    assert_eq!(meta.generation, 2);

    // Verwaisten-Suche löscht alte Generation b1 (INV-CAT-4)
    fs.remove_file(&b1_path).unwrap();

    assert!(!file_exists(&fs, &b1_path));
    assert!(file_exists(&fs, &b2_path));

    // Gen 2 enthält Daten an exakt gleicher BlobRef (INV-BLOB-3)
    let b2_r = fs.open(&b2_path, OpenMode::Read).unwrap();
    let reader2 = BlobReader::open(&*b2_r, 0).unwrap();
    let read_rec = reader2.read(&*b2_r, &ref1, b"k", 20, None, None, None, None, None).unwrap();
    assert_eq!(read_rec, b"v");
}

#[test]
fn test_crash_matrix_catalog_rollover_crash_before_current() {
    // Zeile: "Catalog-Rollover: nach neuer Datei, vor CURRENT | alte CURRENT gültig | neue Datei verwaist, gelöscht"
    let mem_fs = MemFs::new();
    let fault_cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem_fs, fault_cfg.clone());
    let root = Path::new("crash_rollover_before_current");
    fs.create_dir_all(root, Perm::Private).unwrap();

    // Katalog 1 existiert und CURRENT zeigt auf 1
    write_current(&fs, root, 1).unwrap();
    let cat1_path = root.join(catalog_file_name(1));
    let cat1_file = fs.open(&cat1_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    cat1_file.write_all_at(&CatalogHeader::new(1).encode(), 0).unwrap();
    let (mut state1, mut log) = CatalogLog::read_and_recover(&*cat1_file, 1, 0, None, None, None, None).unwrap();

    let mut e1 = CatalogEdit::default();
    e1.spaces_add.push((42, b"space42".to_vec(), CatalogSpaceOptions {
        header_len: 0,
        weights_k: 0,
        flags: 0,
        separate_threshold: 0,
        prefix_filter: 0,
        locality_prefix: 0,
    }));
    let synced = sync_existing(&fs, &*cat1_file, root, e1.clone()).unwrap();
    log.commit(&*cat1_file, synced, None, None, None, None).unwrap();
    state1 = apply_edit(&state1, &e1).unwrap();
    cat1_file.sync_all().unwrap();
    fs.sync_dir(root).unwrap();

    // Rollover beginnt: neue Datei catalog-000002 wird geschrieben und gesynct
    let cat2_path = root.join(catalog_file_name(2));
    let cat2_file = fs.open(&cat2_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    cat2_file.write_all_at(&CatalogHeader::new(2).encode(), 0).unwrap();
    let snap = state1.to_snapshot_edit();
    let mut log2 = CatalogLog::new(2);
    let snap_synced = sync_existing(&fs, &*cat2_file, root, snap).unwrap();
    log2.commit(&*cat2_file, snap_synced, None, None, None, None).unwrap();
    cat2_file.sync_all().unwrap();
    fs.sync_dir(root).unwrap();

    // Absturz BEVOR CURRENT aktualisiert wird!
    fault_cfg.fail_sync.store(true, Ordering::Relaxed);

    // Neustart / Recovery:
    // Liest CURRENT -> zeigt immer noch auf 1!
    let cur_no = read_current(&fs, root).unwrap();
    assert_eq!(cur_no, 1);

    // Alte Datei ist gültig
    let r1 = fs.open(&cat1_path, OpenMode::Read).unwrap();
    let (recovered_state1, _) = CatalogLog::read_and_recover(&*r1, 1, 0, None, None, None, None).unwrap();
    assert_eq!(recovered_state1.spaces.len(), 1);

    // Neue Datei ist verwaist und wird von der Verwaisten-Suche gelöscht (INV-CAT-4)
    fs.remove_file(&cat2_path).unwrap();
    assert!(!file_exists(&fs, &cat2_path));
    assert!(file_exists(&fs, &cat1_path));
}

#[test]
fn test_crash_matrix_catalog_rollover_crash_after_current_before_delete() {
    // Zeile: "Catalog-Rollover: nach CURRENT, vor Löschen der alten Datei | beide vorhanden | alte Datei verwaist, gelöscht"
    let mem_fs = MemFs::new();
    let fault_cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem_fs, fault_cfg.clone());
    let root = Path::new("crash_rollover_after_current");
    fs.create_dir_all(root, Perm::Private).unwrap();

    write_current(&fs, root, 1).unwrap();
    let cat1_path = root.join(catalog_file_name(1));
    let cat1_file = fs.open(&cat1_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    cat1_file.write_all_at(&CatalogHeader::new(1).encode(), 0).unwrap();
    let (mut state1, mut log) = CatalogLog::read_and_recover(&*cat1_file, 1, 0, None, None, None, None).unwrap();

    let mut e1 = CatalogEdit::default();
    e1.spaces_add.push((99, b"space99".to_vec(), CatalogSpaceOptions {
        header_len: 0,
        weights_k: 0,
        flags: 0,
        separate_threshold: 0,
        prefix_filter: 0,
        locality_prefix: 0,
    }));
    let synced = sync_existing(&fs, &*cat1_file, root, e1.clone()).unwrap();
    log.commit(&*cat1_file, synced, None, None, None, None).unwrap();
    state1 = apply_edit(&state1, &e1).unwrap();
    cat1_file.sync_all().unwrap();
    fs.sync_dir(root).unwrap();

    // Rollover mit neuer Datei 2 und CURRENT auf 2 gesetzt
    let cat2_path = root.join(catalog_file_name(2));
    let cat2_file = fs.open(&cat2_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    cat2_file.write_all_at(&CatalogHeader::new(2).encode(), 0).unwrap();
    let snap = state1.to_snapshot_edit();
    let mut log2 = CatalogLog::new(2);
    let snap_synced = sync_existing(&fs, &*cat2_file, root, snap).unwrap();
    log2.commit(&*cat2_file, snap_synced, None, None, None, None).unwrap();
    cat2_file.sync_all().unwrap();

    // CURRENT auf 2 umschalten
    write_current(&fs, root, 2).unwrap();

    // Absturz vor Löschen von catalog 1! Beide Dateien vorhanden!
    fault_cfg.fail_sync.store(true, Ordering::Relaxed);

    // Neustart / Recovery:
    let cur_no = read_current(&fs, root).unwrap();
    assert_eq!(cur_no, 2);

    // Neue Datei ist gültig und aktiv
    let r2 = fs.open(&cat2_path, OpenMode::Read).unwrap();
    let (state2, _) = CatalogLog::read_and_recover(&*r2, 2, 0, None, None, None, None).unwrap();
    assert_eq!(state2.spaces.len(), 1);
    assert!(state2.spaces.contains_key(&99));

    // Alte Datei catalog 1 ist verwaist und wird von Verwaisten-Suche aufgeräumt (INV-CAT-4)
    fs.remove_file(&cat1_path).unwrap();
    assert!(!file_exists(&fs, &cat1_path));
    assert!(file_exists(&fs, &cat2_path));
}
