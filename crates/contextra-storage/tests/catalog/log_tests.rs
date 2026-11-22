#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_storage::catalog::current::{read_current, write_current};
use contextra_storage::catalog::durable::sync_existing;
use contextra_storage::catalog::edit::CatalogEdit;
use contextra_storage::catalog::log::{catalog_file_name, CatalogHeader, CatalogLog, CATALOG_HEADER_LEN};
use contextra_storage::catalog::rollover::rollover;
use contextra_storage::catalog::state::CatalogState;
use contextra_storage::catalog::types::*;
use contextra_storage::testing::MemFs;
use contextra_storage_ports::{Fs, OpenMode, Perm};
use std::path::Path;

#[test]
fn test_catalog_header_roundtrip() {
    let header = CatalogHeader::new(42);
    let bytes = header.encode();
    assert_eq!(bytes.len(), CATALOG_HEADER_LEN);

    let decoded = CatalogHeader::decode(&bytes).expect("decode header succeeds");
    assert_eq!(decoded.catalog_no, 42);
    assert_eq!(decoded.version, 1);
}

#[test]
fn test_catalog_commit_and_recovery() {
    let fs = MemFs::new();
    let root = Path::new("test_cat");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let cat_name = catalog_file_name(1);
    let cat_path = root.join(&cat_name);
    let file = fs.open(&cat_path, OpenMode::CreateNew(Perm::Private)).unwrap();

    let header = CatalogHeader::new(1);
    file.write_all_at(&header.encode(), 0).unwrap();

    let mut log = CatalogLog::read_and_recover(&*file, 1, 0, None, None, None, None).unwrap().1;

    // Edit 1: Space hinzufügen
    let mut e1 = CatalogEdit::default();
    e1.spaces_add.push((1, b"main".to_vec(), CatalogSpaceOptions {
        header_len: 4,
        weights_k: 1,
        flags: 0,
        separate_threshold: 1024,
        prefix_filter: 0,
        locality_prefix: 0,
    }));

    // M-5: Der Katalog-Commit nimmt ausschließlich Synced entgegen
    let synced_e1 = sync_existing(&fs, &*file, root, e1).unwrap();
    log.commit(&*file, synced_e1, None, None, None, None).unwrap();

    // Edit 2: Run hinzufügen
    let mut e2 = CatalogEdit::default();
    e2.add_files.push(FileMeta {
        kind: CatalogFileKind::Run,
        id: 10,
        space: 1,
        level: 0,
        tier: 0,
        flags: 0,
        min_stamp: 1,
        max_stamp: 5,
        bytes: 2048,
        entries: 10,
        generation: 1,
        segment_kind: 0,
        blob_no_min: 0,
        blob_no_max: 0,
        min_key: b"a".to_vec(),
        max_key: b"b".to_vec(),
    });
    let synced_e2 = sync_existing(&fs, &*file, root, e2).unwrap();
    log.commit(&*file, synced_e2, None, None, None, None).unwrap();

    // Datei neu öffnen und Recovery prüfen
    let (recovered_state, _) = CatalogLog::read_and_recover(&*file, 1, 0, None, None, None, None).unwrap();
    assert!(recovered_state.spaces.contains_key(&1));
    assert!(recovered_state.is_file_live(CatalogFileKind::Run, 10));
}

#[test]
fn test_catalog_torn_write_at_tail_discarded() {
    // INV-CAT-3: Nur der letzte Catalog-Record darf als Torn Write verworfen werden
    let fs = MemFs::new();
    let root = Path::new("test_torn");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let cat_name = catalog_file_name(1);
    let cat_path = root.join(&cat_name);
    let file = fs.open(&cat_path, OpenMode::CreateNew(Perm::Private)).unwrap();

    let header = CatalogHeader::new(1);
    file.write_all_at(&header.encode(), 0).unwrap();

    let mut log = CatalogLog::read_and_recover(&*file, 1, 0, None, None, None, None).unwrap().1;

    // Guter Edit 1
    let mut e1 = CatalogEdit::default();
    e1.space_flushed.push((1, 100));
    let synced_e1 = sync_existing(&fs, &*file, root, e1).unwrap();
    log.commit(&*file, synced_e1, None, None, None, None).unwrap();

    // Zerrissenen letzten Record simulieren: hänge 3 halbe Bytes an
    let cur_len = file.len().unwrap();
    file.write_all_at(&[0x11, 0x22, 0x33], cur_len).unwrap();

    // INV-CAT-3: Recovery verwirft den letzten zerrissenen Record und übernimmt den guten Stand
    let (state, _) = CatalogLog::read_and_recover(&*file, 1, 0, None, None, None, None)
        .expect("torn tail at EOF is discarded gracefully");
    assert_eq!(state.space_flushed.get(&1), Some(&100));
}

#[test]
fn test_catalog_defect_before_tail_is_corrupt() {
    // INV-CAT-3: Jeder frühere Defekt führt zwingend zu Corrupt
    let fs = MemFs::new();
    let root = Path::new("test_corrupt");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let cat_name = catalog_file_name(1);
    let cat_path = root.join(&cat_name);
    let file = fs.open(&cat_path, OpenMode::CreateNew(Perm::Private)).unwrap();

    let header = CatalogHeader::new(1);
    file.write_all_at(&header.encode(), 0).unwrap();

    let mut log = CatalogLog::read_and_recover(&*file, 1, 0, None, None, None, None).unwrap().1;

    // Edit 1 schreiben
    let mut e1 = CatalogEdit::default();
    e1.space_flushed.push((1, 50));
    let synced_e1 = sync_existing(&fs, &*file, root, e1).unwrap();
    log.commit(&*file, synced_e1, None, None, None, None).unwrap();

    // Edit 2 schreiben
    let mut e2 = CatalogEdit::default();
    e2.space_flushed.push((1, 100));
    let synced_e2 = sync_existing(&fs, &*file, root, e2).unwrap();
    log.commit(&*file, synced_e2, None, None, None, None).unwrap();

    // Bitflip in Edit 1 (Offset 25 = mitten in Edit 1)
    let mut bit = [0u8; 1];
    file.read_at(&mut bit, 25).unwrap();
    bit[0] ^= 0x01;
    file.write_all_at(&bit, 25).unwrap();

    // INV-CAT-3: Defekt VOR dem Dateiende (gefolgt von Edit 2) muss zwingend als Corrupt scheitern!
    let res = CatalogLog::read_and_recover(&*file, 1, 0, None, None, None, None);
    assert!(matches!(res, Err(CatalogError::Corrupt(_))));
}

#[test]
fn test_catalog_rollover_lifecycle() {
    let fs = MemFs::new();
    let root = Path::new("test_rollover");
    fs.create_dir_all(root, Perm::Private).unwrap();

    // CURRENT initialisieren
    write_current(&fs, root, 1).unwrap();
    assert_eq!(read_current(&fs, root).unwrap(), 1);

    // Initialer Zustand
    let mut state = CatalogState::default();
    state.space_flushed.insert(1, 42);

    // Rollover ausführen (1 -> 2)
    let new_no = rollover(&fs, root, 1, &state, 0, None, None, None, None).unwrap();
    assert_eq!(new_no, 2);

    // CURRENT muss nun auf 2 zeigen
    assert_eq!(read_current(&fs, root).unwrap(), 2);

    // Alte Datei CATALOG-000001 muss gelöscht sein
    let old_path = root.join(catalog_file_name(1));
    assert!(fs.open(&old_path, OpenMode::Read).is_err());

    // Neue Datei CATALOG-000002 muss existieren und Snapshot enthalten
    let new_path = root.join(catalog_file_name(2));
    let new_file = fs.open(&new_path, OpenMode::Read).unwrap();
    let (rec_state, _) = CatalogLog::read_and_recover(&*new_file, 2, 0, None, None, None, None).unwrap();
    assert_eq!(rec_state.space_flushed.get(&1), Some(&42));
}
