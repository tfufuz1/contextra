//! # MemFs Selbsttests nach SPEC §19.2 und IMPL WP-1.3

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_storage::testing::conformance::run_fs_conformance_suite;
use contextra_storage::testing::MemFs;
use contextra_storage_ports::{Fs, OpenMode, Perm};
use std::path::Path;

#[test]
fn test_mem_fs_conformance_suite() {
    let mem = MemFs::new();
    let root = Path::new("/conformance_root");
    mem.create_dir_all(root, Perm::Private).expect("create_dir_all");
    run_fs_conformance_suite(&mem, root);
}

#[test]
fn test_mem_fs_unsynced_data_lost_after_crash() {
    // 1. Jede Datei hat zwei Zustände: Seitencache und dauerhaft.
    let fs = MemFs::new();
    let root = Path::new("/data");
    fs.create_dir_all(root, Perm::Private).unwrap();
    fs.sync_dir(root).unwrap();

    let file_path = root.join("test.dat");
    let file = fs.open(&file_path, OpenMode::CreateNew(Perm::Private)).unwrap();

    // Initiale persistierte Daten
    file.write_all_at(b"INITIAL_PERSISTED", 0).unwrap();
    file.sync_data().unwrap();
    fs.sync_dir(root).unwrap();

    // Neuer Schreibvorgang im Seitencache OHNE sync_data
    file.write_all_at(b"_DIRTY_PAGE_CACHE", 17).unwrap();
    assert_eq!(file.len().unwrap(), 34);

    // Absturz simulieren
    fs.crash();

    // Nach Absturz: Nur initial persistierter Stand existiert
    let reopened = fs.open(&file_path, OpenMode::Read).unwrap();
    assert_eq!(reopened.len().unwrap(), 17);
    let mut buf = vec![0u8; 17];
    reopened.read_at(&mut buf, 0).unwrap();
    assert_eq!(&buf, b"INITIAL_PERSISTED");
}

#[test]
fn test_mem_fs_synced_data_survives_crash() {
    let fs = MemFs::new();
    let root = Path::new("/durable_data");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let file_path = root.join("durable.log");
    let file = fs.open(&file_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    file.write_all_at(b"EXACT_DURABLE_PAYLOAD", 0).unwrap();
    file.sync_data().unwrap();
    fs.sync_dir(root).unwrap();

    fs.crash();

    let reopened = fs.open(&file_path, OpenMode::Read).unwrap();
    assert_eq!(reopened.len().unwrap(), 21);
    let mut buf = vec![0u8; 21];
    reopened.read_at(&mut buf, 0).unwrap();
    assert_eq!(&buf, b"EXACT_DURABLE_PAYLOAD");
}

#[test]
fn test_mem_fs_rename_without_sync_dir_reverted() {
    // SPEC §19.2: rename ohne sync_dir wird beim Absturz rückgängig gemacht.
    let fs = MemFs::new();
    let root = Path::new("/rename_dir");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let from = root.join("source.dat");
    let to = root.join("target.dat");

    let file = fs.open(&from, OpenMode::CreateNew(Perm::Private)).unwrap();
    file.write_all_at(b"RENAME_ME", 0).unwrap();
    file.sync_data().unwrap();
    fs.sync_dir(root).unwrap();

    // Rename ausführen, aber sync_dir NICHT aufrufen
    fs.rename(&from, &to).unwrap();
    assert!(fs.open(&to, OpenMode::Read).is_ok());
    assert!(fs.open(&from, OpenMode::Read).is_err());

    // Absturz simulieren
    fs.crash();

    // Nach Absturz muss Quelldatei wieder da sein und Zieldatei nicht existieren
    assert!(fs.open(&from, OpenMode::Read).is_ok(), "Quelldatei muss nach Crash wiederhergestellt sein");
    assert!(fs.open(&to, OpenMode::Read).is_err(), "Zieldatei darf nach Crash ohne sync_dir nicht existieren");

    let recovered = fs.open(&from, OpenMode::Read).unwrap();
    let mut buf = vec![0u8; 9];
    recovered.read_at(&mut buf, 0).unwrap();
    assert_eq!(&buf, b"RENAME_ME");
}

#[test]
fn test_mem_fs_remove_without_sync_dir_reverted() {
    // Verzeichniseinträge (remove) sind erst nach sync_dir dauerhaft.
    let fs = MemFs::new();
    let root = Path::new("/remove_dir");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let file_path = root.join("deleteme.dat");
    let file = fs.open(&file_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    file.write_all_at(b"KEEP_ALIVE", 0).unwrap();
    file.sync_all().unwrap();
    fs.sync_dir(root).unwrap();

    // Datei löschen, aber Verzeichnis NICHT synchronisieren
    fs.remove_file(&file_path).unwrap();
    assert!(fs.open(&file_path, OpenMode::Read).is_err());

    // Absturz
    fs.crash();

    // Datei muss wieder vorhanden sein
    let recovered = fs.open(&file_path, OpenMode::Read).expect("Datei muss nach Crash ohne sync_dir überleben");
    let mut buf = vec![0u8; 10];
    recovered.read_at(&mut buf, 0).unwrap();
    assert_eq!(&buf, b"KEEP_ALIVE");
}

#[test]
fn test_mem_fs_sync_dir_granularity() {
    // 2. Verzeichniseinträge sind erst nach sync_dir dauerhaft (getrennt nach Verzeichnissen).
    let fs = MemFs::new();
    let dir_a = Path::new("/dir_a");
    let dir_b = Path::new("/dir_b");
    fs.create_dir_all(dir_a, Perm::Private).unwrap();
    fs.create_dir_all(dir_b, Perm::Private).unwrap();

    let fa_path = dir_a.join("file_a.dat");
    let fb_path = dir_b.join("file_b.dat");

    let fa = fs.open(&fa_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    fa.write_all_at(b"AAA", 0).unwrap();
    fa.sync_data().unwrap();

    let fb = fs.open(&fb_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    fb.write_all_at(b"BBB", 0).unwrap();
    fb.sync_data().unwrap();

    // Nur dir_a synchronisieren
    fs.sync_dir(dir_a).unwrap();

    fs.crash();

    assert!(fs.open(&fa_path, OpenMode::Read).is_ok());
    assert!(fs.open(&fb_path, OpenMode::Read).is_err());
}

#[test]
fn test_mem_fs_snapshot_f10_rollback() {
    // SPEC §19.2 / §5.1 F-10: Zurückspielen eines älteren Gesamtstands für den Anker-Test.
    let fs = MemFs::new();
    let root = Path::new("/snap_root");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let f1_path = root.join("f1.dat");
    let f1 = fs.open(&f1_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    f1.write_all_at(b"ORIGINAL_1", 0).unwrap();
    f1.sync_all().unwrap();
    fs.sync_dir(root).unwrap();

    // Snapshot 1 erstellen
    let snap_id = fs.take_snapshot();

    // Änderungen vornehmen
    f1.write_all_at(b"MODIFIED_1", 0).unwrap();
    f1.sync_all().unwrap();

    let f2_path = root.join("f2.dat");
    let f2 = fs.open(&f2_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    f2.write_all_at(b"NEW_FILE_2", 0).unwrap();
    f2.sync_all().unwrap();
    fs.sync_dir(root).unwrap();

    // Prüfen, dass geänderter Stand aktiv ist
    let cur2 = fs.open(&f2_path, OpenMode::Read).unwrap();
    assert_eq!(cur2.len().unwrap(), 10);

    // Rollback auf älteren Gesamtstand (F-10)
    fs.restore_snapshot(snap_id).expect("restore_snapshot succeeds");

    // f1 muss wieder ORIGINAL_1 sein, f2 darf nicht mehr existieren
    assert!(fs.open(&f2_path, OpenMode::Read).is_err(), "f2 darf nach Rollback nicht existieren");
    let r1 = fs.open(&f1_path, OpenMode::Read).unwrap();
    let mut buf = vec![0u8; 10];
    r1.read_at(&mut buf, 0).unwrap();
    assert_eq!(&buf, b"ORIGINAL_1");
}
