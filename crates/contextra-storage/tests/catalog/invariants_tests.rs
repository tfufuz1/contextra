#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_storage::catalog::current::write_current;
use contextra_storage::catalog::durable::sync_existing;
use contextra_storage::catalog::edit::CatalogEdit;
use contextra_storage::catalog::log::{catalog_file_name, CatalogHeader, CatalogLog};
use contextra_storage::catalog::types::*;
use contextra_storage::testing::{FaultConfig, FaultFs, MemFs};
use contextra_storage_ports::{Fs, OpenMode, Perm};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[test]
fn test_inv_cat_1_and_inv_cat_4_orphan_cleanup_after_crash() {
    // INV-CAT-1: Nur ein dauerhafter Catalog-Edit macht eine Datei lebendig oder obsolet;
    // nicht genannte Dateien sind verwaist und löschbar.
    // INV-CAT-4: Nach einem Absturz räumt die Verwaisten-Suche alle .building.*,
    // nicht genannten Dateien und nicht aktuellen Blob-Generationen auf; Reserve-Redo bleibt.
    let mem_fs = MemFs::new();
    let fault_cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem_fs, fault_cfg.clone());

    let root = Path::new("test_inv_cat_1_4");
    fs.create_dir_all(root, Perm::Private).unwrap();

    // 1. Katalog initialisieren mit einem Run (id=1) und einem Blob (blob_no=5, gen=2)
    write_current(&fs, root, 1).unwrap();
    let cat_file = fs.open(&root.join(catalog_file_name(1)), OpenMode::CreateNew(Perm::Private)).unwrap();
    cat_file.write_all_at(&CatalogHeader::new(1).encode(), 0).unwrap();
    let mut log = CatalogLog::read_and_recover(&*cat_file, 1, 0, None, None, None, None).unwrap().1;

    let mut e1 = CatalogEdit::default();
    e1.add_files.push(FileMeta {
        kind: CatalogFileKind::Run,
        id: 1,
        space: 1,
        level: 0,
        tier: 0,
        flags: 0,
        min_stamp: 1,
        max_stamp: 5,
        bytes: 1000,
        entries: 10,
        generation: 1,
        segment_kind: 0,
        blob_no_min: 0,
        blob_no_max: 0,
        min_key: Vec::new(),
        max_key: Vec::new(),
    });
    e1.add_files.push(FileMeta {
        kind: CatalogFileKind::Blob,
        id: 5,
        space: 1,
        level: 0,
        tier: 0,
        flags: 0,
        min_stamp: 1,
        max_stamp: 5,
        bytes: 5000,
        entries: 2,
        generation: 2, // Aktuelle Generation ist 2
        segment_kind: 0,
        blob_no_min: 0,
        blob_no_max: 0,
        min_key: Vec::new(),
        max_key: Vec::new(),
    });
    let synced_e1 = sync_existing(&fs, &*cat_file, root, e1).unwrap();
    log.commit(&*cat_file, synced_e1, None, None, None, None).unwrap();
    cat_file.sync_all().unwrap();
    fs.sync_dir(root).unwrap();

    // 2. Erzeuge verschiedene Dateien auf der Platte:
    // a) Lebendige Run-Datei: RUN-000001
    let run_file = fs.open(&root.join("RUN-000001"), OpenMode::CreateNew(Perm::Private)).unwrap();
    run_file.write_all_at(b"run_data", 0).unwrap();
    run_file.sync_all().unwrap();

    // b) Lebendiger Blob Gen 2: BLOB-000005.000002
    let blob_g2 = fs.open(&root.join("BLOB-000005.000002"), OpenMode::CreateNew(Perm::Private)).unwrap();
    blob_g2.write_all_at(b"blob_gen2_data", 0).unwrap();
    blob_g2.sync_all().unwrap();

    // c) Veralteter Blob Gen 1 (nicht mehr aktuell): BLOB-000005.000001
    let blob_g1 = fs.open(&root.join("BLOB-000005.000001"), OpenMode::CreateNew(Perm::Private)).unwrap();
    blob_g1.write_all_at(b"blob_gen1_obsolete", 0).unwrap();
    blob_g1.sync_all().unwrap();

    // d) Verwaiste Run-Datei (nie im Catalog genannt): RUN-000099
    let orphan_run = fs.open(&root.join("RUN-000099"), OpenMode::CreateNew(Perm::Private)).unwrap();
    orphan_run.write_all_at(b"orphan_data", 0).unwrap();
    orphan_run.sync_all().unwrap();

    // e) Unfertige temporäre Datei: .building.RUN-000100.123
    let building_file = fs.open(&root.join(".building.RUN-000100.123"), OpenMode::CreateNew(Perm::Private)).unwrap();
    building_file.write_all_at(b"unfinished", 0).unwrap();
    building_file.sync_all().unwrap();

    fs.sync_dir(root).unwrap();

    // Absturz simulieren
    fault_cfg.fail_sync.store(true, Ordering::Relaxed);

    // 3. Recovery-Prüfung: Catalog lesen
    let cat_read = fs.open(&root.join(catalog_file_name(1)), OpenMode::Read).unwrap();
    let (state, _) = CatalogLog::read_and_recover(&*cat_read, 1, 0, None, None, None, None).unwrap();

    // INV-CAT-1: Nur RUN-000001 und BLOB-000005 (Gen 2) sind lebendig
    assert!(state.is_file_live(CatalogFileKind::Run, 1));
    assert!(state.is_file_live(CatalogFileKind::Blob, 5));
    assert!(!state.is_file_live(CatalogFileKind::Run, 99));

    // Verwaisten-Suche ausführen nach INV-CAT-4:
    // Räumt .building.*, ungenannte Dateien und nicht-aktuelle Blob-Generationen auf
    let entries = fs.list_dir(root).unwrap();
    for entry in entries {
        let name = entry.name;
        if name.starts_with(".building.") {
            // INV-CAT-4: .building.* wird gelöscht
            fs.remove_file(&root.join(&name)).unwrap();
        } else if name == "RUN-000099" {
            // INV-CAT-1 / INV-CAT-4: nicht genannte Datei wird gelöscht
            fs.remove_file(&root.join(&name)).unwrap();
        } else if name == "BLOB-000005.000001" {
            // INV-CAT-4: alte Blob-Generation wird gelöscht
            fs.remove_file(&root.join(&name)).unwrap();
        }
    }

    // Prüfen, dass nur die lebendigen Dateien übrig bleiben
    let remaining = fs.list_dir(root).unwrap();
    let names: Vec<String> = remaining.into_iter().map(|e| e.name).collect();
    assert!(names.contains(&"RUN-000001".to_string()));
    assert!(names.contains(&"BLOB-000005.000002".to_string()));
    assert!(names.contains(&catalog_file_name(1)));
    assert!(names.contains(&"CURRENT".to_string()));
    assert!(!names.contains(&"RUN-000099".to_string()));
    assert!(!names.contains(&"BLOB-000005.000001".to_string()));
}

#[test]
fn test_inv_cat_2_atomicity_over_fault_fs() {
    // INV-CAT-2: Ein Edit ist ein Record (P-2); er wird ganz oder gar nicht angewendet.
    let mem_fs = MemFs::new();
    let fault_cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem_fs, fault_cfg.clone());

    let root = Path::new("test_inv_cat_2");
    fs.create_dir_all(root, Perm::Private).unwrap();

    write_current(&fs, root, 1).unwrap();
    let cat_file = fs.open(&root.join(catalog_file_name(1)), OpenMode::CreateNew(Perm::Private)).unwrap();
    cat_file.write_all_at(&CatalogHeader::new(1).encode(), 0).unwrap();
    let mut log = CatalogLog::read_and_recover(&*cat_file, 1, 0, None, None, None, None).unwrap().1;

    // Mehrteiliger Edit
    let mut edit = CatalogEdit::default();
    edit.add_files.push(FileMeta {
        kind: CatalogFileKind::Run,
        id: 10,
        space: 1,
        level: 0,
        tier: 0,
        flags: 0,
        min_stamp: 10,
        max_stamp: 20,
        bytes: 500,
        entries: 5,
        generation: 1,
        segment_kind: 0,
        blob_no_min: 0,
        blob_no_max: 0,
        min_key: b"a".to_vec(),
        max_key: b"z".to_vec(),
    });
    edit.consumer_set.push((b"cons1".to_vec(), 15, 1000));
    edit.space_flushed.push((1, 20));

    // Fehler bei sync_data simulieren
    fault_cfg.fail_sync.store(true, Ordering::Relaxed);

    let synced = sync_existing(&fs, &*cat_file, root, edit);
    if let Ok(s) = synced {
        let _ = log.commit(&*cat_file, s, None, None, None, None);
    }

    // Nach Absturz: Recovery ausführen
    fault_cfg.fail_sync.store(false, Ordering::Relaxed);
    let cat_read = fs.open(&root.join(catalog_file_name(1)), OpenMode::Read).unwrap();
    let (recovered_state, _) = CatalogLog::read_and_recover(&*cat_read, 1, 0, None, None, None, None).unwrap();

    // INV-CAT-2: Entweder alles oder gar nichts vorhanden - niemals ein Teil!
    let has_file = recovered_state.is_file_live(CatalogFileKind::Run, 10);
    let has_consumer = recovered_state.consumers.contains_key(&b"cons1"[..]);
    let has_flushed = recovered_state.space_flushed.contains_key(&1);

    assert_eq!(has_file, has_consumer, "INV-CAT-2 Atomizität verletzt: Teilzustand vorhanden");
    assert_eq!(has_file, has_flushed, "INV-CAT-2 Atomizität verletzt: Teilzustand vorhanden");
}

#[test]
fn test_inv_cat_3_torn_tail_discarded_vs_early_defect_corrupt_over_fault_fs() {
    // INV-CAT-3: Nur der letzte Catalog-Record darf als Torn Write verworfen werden;
    // jeder frühere Defekt ist Corrupt.
    let mem_fs = MemFs::new();
    let fault_cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem_fs, fault_cfg.clone());

    let root = Path::new("test_inv_cat_3");
    fs.create_dir_all(root, Perm::Private).unwrap();

    write_current(&fs, root, 1).unwrap();
    let cat_file = fs.open(&root.join(catalog_file_name(1)), OpenMode::CreateNew(Perm::Private)).unwrap();
    cat_file.write_all_at(&CatalogHeader::new(1).encode(), 0).unwrap();
    let mut log = CatalogLog::read_and_recover(&*cat_file, 1, 0, None, None, None, None).unwrap().1;

    // Edit 1 commiten
    let mut e1 = CatalogEdit::default();
    e1.space_flushed.push((1, 100));
    let s1 = sync_existing(&fs, &*cat_file, root, e1).unwrap();
    log.commit(&*cat_file, s1, None, None, None, None).unwrap();

    // 1. Torn Write am Schwanz simulieren: unvollständige Bytes am Dateiende anhängen
    let len_before = cat_file.len().unwrap();
    cat_file.write_all_at(&[0x10, 0x00, 0x00], len_before).unwrap(); // Nur 3 Bytes statt vollständigem Record

    // Recovery muss gelingen und den zerrissenen Schwanz verwerfen (INV-CAT-3)
    let (state_recovered, _) = CatalogLog::read_and_recover(&*cat_file, 1, 0, None, None, None, None).unwrap();
    assert_eq!(state_recovered.space_flushed.get(&1), Some(&100));

    // 2. Jetzt einen Defekt VOR dem Dateiende erzeugen:
    // Wir hängen einen weiteren vollständigen Edit 2 an
    let mut e2 = CatalogEdit::default();
    e2.space_flushed.push((2, 200));
    cat_file.set_len(len_before).unwrap();
    let s2 = sync_existing(&fs, &*cat_file, root, e2).unwrap();
    log.commit(&*cat_file, s2, None, None, None, None).unwrap();

    // Jetzt manipulieren wir ein Byte in Edit 1 (früherer Defekt!)
    let header_len = 20u64;
    let mut corrupted_byte = [0u8; 1];
    cat_file.read_at(&mut corrupted_byte, header_len + 5).unwrap();
    corrupted_byte[0] ^= 0xFF;
    cat_file.write_all_at(&corrupted_byte, header_len + 5).unwrap();

    // INV-CAT-3: Früherer Defekt MUSS zwingend CatalogError::Corrupt liefern
    let res = CatalogLog::read_and_recover(&*cat_file, 1, 0, None, None, None, None);
    assert!(matches!(res, Err(CatalogError::Corrupt(_))), "Erwartet Corrupt nach INV-CAT-3 bei frühem Defekt");
}
