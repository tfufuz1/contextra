//! # MemFs Konformitäts- und Crash-Tests

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_storage::testing::conformance::run_fs_conformance_suite;
use contextra_storage::testing::MemFs;
use contextra_storage_ports::{Fs, OpenMode, Perm};
use std::path::Path;

#[test]
fn test_mem_fs_conformance() {
    let mem_fs = MemFs::new();
    let root = Path::new("/test_root");
    mem_fs
        .create_dir_all(root, Perm::Private)
        .expect("create root dir");
    run_fs_conformance_suite(&mem_fs, root);
}

#[test]
fn test_mem_fs_crash_reverts_uncommitted_data() {
    let fs = MemFs::new();
    let root = Path::new("/data");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let file_path = root.join("journal.log");
    let file = fs
        .open(&file_path, OpenMode::CreateNew(Perm::Private))
        .unwrap();

    file.write_all_at(b"COMMITTED", 0).unwrap();
    file.sync_data().unwrap();
    fs.sync_dir(root).unwrap();

    // Schreiben ohne Sync
    file.write_all_at(b"UNCOMMITTED_DATA", 9).unwrap();

    // Absturz simulieren
    fs.crash();

    let reopened = fs.open(&file_path, OpenMode::Read).unwrap();
    assert_eq!(reopened.len().unwrap(), 9);

    let mut buf = vec![0u8; 9];
    reopened.read_at(&mut buf, 0).unwrap();
    assert_eq!(&buf, b"COMMITTED");
}

#[test]
fn test_mem_fs_sync_dir_only_syncs_target_directory() {
    let fs = MemFs::new();
    let dir1 = Path::new("/dir1");
    let dir2 = Path::new("/dir2");
    fs.create_dir_all(dir1, Perm::Private).unwrap();
    fs.create_dir_all(dir2, Perm::Private).unwrap();

    let f1_path = dir1.join("file1.dat");
    let f2_path = dir2.join("file2.dat");

    let f1 = fs.open(&f1_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    f1.write_all_at(b"data1", 0).unwrap();
    f1.sync_all().unwrap();

    let f2 = fs.open(&f2_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    f2.write_all_at(b"data2", 0).unwrap();
    f2.sync_all().unwrap();

    // Nur dir1 synchronisieren!
    fs.sync_dir(dir1).unwrap();

    // Absturz: dir1/file1.dat muss überleben, dir2/file2.dat muss verschwinden (SPEC §19.2)
    fs.crash();

    assert!(fs.open(&f1_path, OpenMode::Read).is_ok());
    assert!(fs.open(&f2_path, OpenMode::Read).is_err());
}

#[test]
fn test_fault_fs_spec_19_2_fault_injection() {
    use contextra_storage::testing::fault_fs::{FaultConfig, FaultFs};
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    let mem = MemFs::new();
    let cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem, Arc::clone(&cfg));
    let root = Path::new("/fault_root");
    fs.create_dir_all(root, Perm::Private).unwrap();

    // 1. Zerreißen von Schreibvorgängen (Torn Write)
    let p_torn = root.join("torn.dat");
    let f_torn = fs.open(&p_torn, OpenMode::CreateNew(Perm::Private)).unwrap();
    cfg.torn_write_limit.store(4, Ordering::Relaxed);
    let res = f_torn.write_all_at(b"12345678", 0);
    assert!(res.is_err(), "Torn write muss fehlschlagen");
    cfg.torn_write_limit.store(0, Ordering::Relaxed);
    let mut buf = [0u8; 8];
    let n = f_torn.read_at(&mut buf, 0).unwrap();
    assert_eq!(n, 4);
    assert_eq!(&buf[..4], b"1234");

    // 2. Verlorener Schreibvorgang (Lost Write)
    let p_lost = root.join("lost.dat");
    let f_lost = fs.open(&p_lost, OpenMode::CreateNew(Perm::Private)).unwrap();
    cfg.lost_write.store(true, Ordering::Relaxed);
    f_lost.write_all_at(b"lost_content", 0).unwrap();
    cfg.lost_write.store(false, Ordering::Relaxed);
    assert_eq!(f_lost.len().unwrap(), 0);

    // 3. Fehlgeleiteter Schreibvorgang (Misdirected Write)
    let p_mis = root.join("misdirected.dat");
    let f_mis = fs.open(&p_mis, OpenMode::CreateNew(Perm::Private)).unwrap();
    cfg.misdirected_write.store(true, Ordering::Relaxed);
    cfg.misdirected_offset_delta.store(10, Ordering::Relaxed);
    f_mis.write_all_at(b"misdirected", 0).unwrap();
    cfg.misdirected_write.store(false, Ordering::Relaxed);
    let mut mis_buf = [0u8; 11];
    f_mis.read_at(&mut mis_buf, 10).unwrap();
    assert_eq!(&mis_buf, b"misdirected");

    // 4. Bitfäule beim Lesen (Bit Rot)
    let p_rot = root.join("rot.dat");
    let f_rot = fs.open(&p_rot, OpenMode::CreateNew(Perm::Private)).unwrap();
    f_rot.write_all_at(&[0x00, 0x00, 0x00, 0x00], 0).unwrap();
    cfg.enable_bit_rot.store(true, Ordering::Relaxed);
    cfg.bit_rot_offset.store(2, Ordering::Relaxed);
    let mut rot_buf = [0u8; 4];
    f_rot.read_at(&mut rot_buf, 0).unwrap();
    assert_eq!(rot_buf, [0x00, 0x00, 0x01, 0x00]);
}
