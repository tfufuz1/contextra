//! # FaultFs Selbsttests nach SPEC §19.2 und IMPL WP-1.3

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_storage::testing::fault_fs::{CrashMode, FaultConfig, FaultFs};
use contextra_storage::testing::{HookPoint, MemFs, TestHooks};
use contextra_storage_ports::{Fs, OpenMode, Perm};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[test]
fn test_fault_fs_crash_at_op_n() {
    // 1. Absturz an I/O-Operation n: danach zählt nur der dauerhafte Zustand.
    let mem = MemFs::new();
    let cfg = Arc::new(FaultConfig::new(0x42));
    let fs = FaultFs::new_mem(mem.clone(), Arc::clone(&cfg));
    let root = Path::new("/crash_ops");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let p = root.join("journal.dat");
    let f = fs.open(&p, OpenMode::CreateNew(Perm::Private)).unwrap();
    f.write_all_at(b"COMMITTED_DATA", 0).unwrap();
    f.sync_data().unwrap();
    fs.sync_dir(root).unwrap();

    // Setze Absturz genau bei der nächsten Operation (z.B. unvollständiger Write)
    cfg.fail_after_ops.store(cfg.op_counter.load(Ordering::SeqCst) + 1, Ordering::SeqCst);

    // Dieser Schreibvorgang löst den Absturz aus
    let res = f.write_all_at(b"UNCOMMITTED_WRITE", 14);
    assert!(res.is_err(), "Operation n muss fehlschlagen und abstürzen");
    assert!(cfg.crashed.load(Ordering::SeqCst));

    // Nach dem Absturz zählt nur der dauerhafte Zustand
    let reopened = mem.open(&p, OpenMode::Read).unwrap();
    assert_eq!(reopened.len().unwrap(), 14);
    let mut buf = vec![0u8; 14];
    reopened.read_at(&mut buf, 0).unwrap();
    assert_eq!(&buf, b"COMMITTED_DATA");
}

#[test]
fn test_fault_fs_torn_write_sector_sizes() {
    // SPEC §19.2: Zerreißen in Sektorgrößen (512 B und 4096 B).
    let mem = MemFs::new();
    let cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem.clone(), Arc::clone(&cfg));
    let root = Path::new("/torn_root");
    fs.create_dir_all(root, Perm::Private).unwrap();

    // 1. Zerreißen bei 512-Byte-Sektoren
    let p512 = root.join("sec512.dat");
    let f512 = fs.open(&p512, OpenMode::CreateNew(Perm::Private)).unwrap();
    cfg.torn_write_limit.store(512, Ordering::Relaxed);

    let large_buf = vec![0xABu8; 1024];
    let res = f512.write_all_at(&large_buf, 0);
    assert!(res.is_err(), "Torn write muss fehlschlagen");
    cfg.torn_write_limit.store(0, Ordering::Relaxed);

    // Exakt 512 Bytes wurden geschrieben
    assert_eq!(f512.len().unwrap(), 512);
    let mut read_buf = vec![0u8; 512];
    f512.read_at(&mut read_buf, 0).unwrap();
    assert!(read_buf.iter().all(|&b| b == 0xAB));

    // 2. Zerreißen bei 4096-Byte-Sektoren
    let p4k = root.join("sec4k.dat");
    let f4k = fs.open(&p4k, OpenMode::CreateNew(Perm::Private)).unwrap();
    cfg.torn_write_limit.store(4096, Ordering::Relaxed);

    let huge_buf = vec![0xCDu8; 8192];
    let res4k = f4k.write_all_at(&huge_buf, 0);
    assert!(res4k.is_err());
    cfg.torn_write_limit.store(0, Ordering::Relaxed);

    assert_eq!(f4k.len().unwrap(), 4096);

    // 3. crash_torn auf MemFs: Unvollständiger Schreibvorgang von 1200 Bytes wird auf 512-B-Grenze (1024 B) gekappt
    let p_crash = root.join("crash_torn.dat");
    let fc = fs.open(&p_crash, OpenMode::CreateNew(Perm::Private)).unwrap();
    fc.sync_data().unwrap();
    fs.sync_dir(root).unwrap();

    let uncommitted_data = vec![0xEFu8; 1200];
    fc.write_all_at(&uncommitted_data, 0).unwrap();
    assert_eq!(fc.len().unwrap(), 1200);

    // Absturz mit Zerreißen auf 512-Byte-Sektoren
    mem.crash_torn(512);

    let rc = mem.open(&p_crash, OpenMode::Read).unwrap();
    assert_eq!(rc.len().unwrap(), 1024, "Nur die ersten 2 vollen 512-Byte-Sektoren überleben");
}

#[test]
fn test_fault_fs_crash_reordered() {
    let mem = MemFs::new();
    let cfg = Arc::new(FaultConfig::new(0x12345));
    *cfg.crash_mode.lock().unwrap() = CrashMode::Reordered;
    let fs = FaultFs::new_mem(mem.clone(), Arc::clone(&cfg));
    let root = Path::new("/reorder_root");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let p = root.join("reorder.dat");
    let f = fs.open(&p, OpenMode::CreateNew(Perm::Private)).unwrap();
    f.write_all_at(b"BASE", 0).unwrap();
    f.sync_data().unwrap();
    fs.sync_dir(root).unwrap();

    f.write_all_at(b"_EXTRA_UNCOMMITTED_DATA", 4).unwrap();
    fs.crash();

    let reopened = mem.open(&p, OpenMode::Read).unwrap();
    assert!(reopened.len().unwrap() >= 4);
}

#[test]
fn test_fault_fs_lost_write() {
    // 7. Lost Write: quittiert, aber nicht geschrieben.
    let mem = MemFs::new();
    let cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem, Arc::clone(&cfg));
    let root = Path::new("/lost_root");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let p = root.join("lost.dat");
    let f = fs.open(&p, OpenMode::CreateNew(Perm::Private)).unwrap();

    cfg.lost_write.store(true, Ordering::Relaxed);
    let res = f.write_all_at(b"GHOST_DATA_NOT_ON_DISK", 0);
    assert!(res.is_ok(), "Lost write wird quittiert");
    cfg.lost_write.store(false, Ordering::Relaxed);

    assert_eq!(f.len().unwrap(), 0, "Datei muss leer bleiben");
}

#[test]
fn test_fault_fs_misdirected_write() {
    // 8. Misdirected Write: falscher Offset derselben Datei.
    let mem = MemFs::new();
    let cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem, Arc::clone(&cfg));
    let root = Path::new("/misdirected_root");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let p = root.join("misdirected.dat");
    let f = fs.open(&p, OpenMode::CreateNew(Perm::Private)).unwrap();

    cfg.misdirected_write.store(true, Ordering::Relaxed);
    cfg.misdirected_offset_delta.store(50, Ordering::Relaxed);
    f.write_all_at(b"MISDIRECTED_BLOCK", 0).unwrap();
    cfg.misdirected_write.store(false, Ordering::Relaxed);

    // Am Offset 0 liegt nichts
    let mut zero_buf = [0u8; 17];
    f.read_at(&mut zero_buf, 0).unwrap();
    assert_eq!(zero_buf, [0u8; 17]);

    // Am Offset 50 liegt der Block
    let mut shifted_buf = vec![0u8; 17];
    f.read_at(&mut shifted_buf, 50).unwrap();
    assert_eq!(&shifted_buf, b"MISDIRECTED_BLOCK");
}

#[test]
fn test_fault_fs_fail_sync_poison() {
    // 2. fsync und sync_dir schlagen fehl (Poison-Pfad).
    let mem = MemFs::new();
    let cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem, Arc::clone(&cfg));
    let root = Path::new("/sync_root");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let p = root.join("sync.dat");
    let f = fs.open(&p, OpenMode::CreateNew(Perm::Private)).unwrap();
    f.write_all_at(b"DATA", 0).unwrap();

    cfg.fail_sync.store(true, Ordering::Relaxed);
    assert!(f.sync_data().is_err(), "sync_data muss fehlschlagen");
    assert!(f.sync_all().is_err(), "sync_all muss fehlschlagen");
    assert!(fs.sync_dir(root).is_err(), "sync_dir muss fehlschlagen");
    cfg.fail_sync.store(false, Ordering::Relaxed);

    assert!(f.sync_data().is_ok());
    assert!(fs.sync_dir(root).is_ok());
}

#[test]
fn test_fault_fs_enospc_and_short_write() {
    // 3. ENOSPC und Kurzschreibvorgänge mitten im Gruppen-Schreibvorgang (§10.5).
    let mem = MemFs::new();
    let cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem, Arc::clone(&cfg));
    let root = Path::new("/enospc_root");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let p = root.join("enospc.dat");
    let f = fs.open(&p, OpenMode::CreateNew(Perm::Private)).unwrap();

    // ENOSPC schlägt bei Schreibvorgang fehl
    cfg.fail_enospc.store(true, Ordering::Relaxed);
    let res = f.write_all_at(b"OVERFLOW", 0);
    assert_eq!(res.unwrap_err().kind(), std::io::ErrorKind::StorageFull);

    // Lesen und list_dir bleiben trotz ENOSPC erlaubt! (§10.5 Poison-Semantik)
    assert!(fs.list_dir(root).is_ok());
    let mut read_buf = [0u8; 4];
    assert!(f.read_at(&mut read_buf, 0).is_ok());
    cfg.fail_enospc.store(false, Ordering::Relaxed);

    // Kurzschreibvorgang: Nur erste 5 Bytes geschrieben, dann ENOSPC
    cfg.short_write_limit.store(5, Ordering::Relaxed);
    let res_short = f.write_all_at(b"1234567890", 0);
    assert_eq!(res_short.unwrap_err().kind(), std::io::ErrorKind::StorageFull);
    cfg.short_write_limit.store(0, Ordering::Relaxed);

    let mut short_read = [0u8; 10];
    let n = f.read_at(&mut short_read, 0).unwrap();
    assert_eq!(n, 5);
    assert_eq!(&short_read[..5], b"12345");
}

#[test]
fn test_fault_fs_eio_and_bit_rot_at_offset() {
    // 5. & 6. EIO und Bitfäule beim Lesen an gewähltem Offset (§5.1 F-6, F-7).
    let mem = MemFs::new();
    let cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new(mem, Arc::clone(&cfg));
    let root = Path::new("/eio_rot_root");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let p = root.join("test.dat");
    let f = fs.open(&p, OpenMode::CreateNew(Perm::Private)).unwrap();
    f.write_all_at(&[0x00; 32], 0).unwrap();

    // 1. EIO an Offset 10
    cfg.fail_read_eio.store(true, Ordering::Relaxed);
    cfg.fail_read_offset.store(10, Ordering::Relaxed);

    let mut buf = [0u8; 5];
    assert!(f.read_at(&mut buf, 0).is_ok(), "Lesen vor Fehler-Offset gelingt");
    assert!(f.read_at(&mut buf, 8).is_err(), "Lesen über Offset 10 liefert EIO");
    cfg.fail_read_eio.store(false, Ordering::Relaxed);

    // 2. Bitfäule an Offset 7
    cfg.enable_bit_rot.store(true, Ordering::Relaxed);
    cfg.bit_rot_offset.store(7, Ordering::Relaxed);

    let mut rot_buf = [0u8; 16];
    let n = f.read_at(&mut rot_buf, 0).unwrap();
    assert_eq!(n, 16);
    assert_eq!(rot_buf[7], 0x01, "Bit 0 an Offset 7 muss invertiert sein");
    assert_eq!(rot_buf[6], 0x00);
    assert_eq!(rot_buf[8], 0x00);
}

#[test]
fn test_fault_fs_f10_rollback_older_state() {
    // 9. Zurückspielen eines älteren Gesamtstands (F-10).
    let mem = MemFs::new();
    let cfg = Arc::new(FaultConfig::default());
    let fs = FaultFs::new_mem(mem, cfg);
    let root = Path::new("/f10_root");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let p = root.join("state.dat");
    let f = fs.open(&p, OpenMode::CreateNew(Perm::Private)).unwrap();
    f.write_all_at(b"STATE_A", 0).unwrap();
    f.sync_data().unwrap();
    fs.sync_dir(root).unwrap();

    let snap_a = fs.take_snapshot();

    f.write_all_at(b"STATE_B", 0).unwrap();
    f.sync_data().unwrap();

    // Rollback zu Schnappschuss A
    fs.rollback_to_snapshot(snap_a).unwrap();

    let reopened = fs.open(&p, OpenMode::Read).unwrap();
    let mut buf = vec![0u8; 7];
    reopened.read_at(&mut buf, 0).unwrap();
    assert_eq!(&buf, b"STATE_A");
}

#[test]
fn test_fault_fs_hooks_sequence() {
    // 10. Hook-Marken: AfterRedoWrite, AfterRedoSync, BeforeVisible, BeforeCatalogEdit.
    let mem = MemFs::new();
    let cfg = Arc::new(FaultConfig::default());
    let hooks = Arc::new(TestHooks::new());
    cfg.set_hooks(Arc::clone(&hooks) as Arc<dyn contextra_storage_ports::Hooks>);
    let fs = FaultFs::new(mem, cfg);

    fs.config().trigger_hook(HookPoint::AfterRedoWrite);
    fs.config().trigger_hook(HookPoint::AfterRedoSync);
    fs.config().trigger_hook(HookPoint::BeforeVisible);
    fs.config().trigger_hook(HookPoint::BeforeCatalogEdit);

    let seq = hooks.sequence();
    assert_eq!(seq, vec![
        HookPoint::AfterRedoWrite,
        HookPoint::AfterRedoSync,
        HookPoint::BeforeVisible,
        HookPoint::BeforeCatalogEdit,
    ]);
    assert_eq!(hooks.count(HookPoint::AfterRedoWrite), 1);
    assert_eq!(hooks.count(HookPoint::BeforeCatalogEdit), 1);
}
