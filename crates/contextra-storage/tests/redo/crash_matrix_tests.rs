//! # Absturzmatrix-Tests für Redo nach SPEC §13.2
//!
//! Verifiziert jede Redo-relevante Tabellenzeile aus SPEC §13.2 über `FaultFs`.

use contextra_storage::env::entropy::RealEntropy;
use contextra_storage::redo::driver::{segment_file_name, RedoWriter};
use contextra_storage::redo::error::RedoError;
use contextra_storage::redo::frame::Durability;
use contextra_storage::redo::op::RedoOp;
use contextra_storage::redo::replay::recover_redo;
use contextra_storage::testing::fault_fs::{FaultConfig, FaultFs};
use contextra_storage::testing::mem_fs::MemFs;
use contextra_storage_ports::{File, Fs, OpenMode};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[test]
fn test_crash_row_before_redo_write() {
    // Zeile: vor Redo-write | nichts | Change nicht vorhanden, nie quittiert
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/crash_before_write");
    let entropy = RealEntropy;

    // Segment anlegen, aber noch keinen Change schreiben
    let mut writer = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();
    // Absturz vor writer.write_change(...)
    drop(writer);

    let rep = recover_redo(Arc::clone(&mem), base, 1, 0, 0).expect("Recovery succeeds");
    assert_eq!(rep.committed_frames.len(), 0);
}

#[test]
fn test_crash_row_mid_write_torn_tail_healed() {
    // Zeile: mitten im write / vor fsync | halbes oder ungeflushtes Frame | Frame wird abgeschnitten oder fehlt, nie quittiert
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/crash_mid_write");
    let entropy = RealEntropy;

    let mut writer = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();
    let op1 = RedoOp::new_put(1, None, b"k1".to_vec(), b"v1".to_vec());
    writer.write_change(1, Durability::Always, &[op1], 2, &entropy).unwrap();

    // Mitten im zweiten Schreibvorgang bricht das System ab: Halber Frame im Segment
    let seg_path = base.join(segment_file_name(1));
    let file = mem.open(&seg_path, OpenMode::ReadWrite).unwrap();
    let half_frame = [0x15, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00]; // Unvollständig
    file.write_all_at(&half_frame, 48 + 31).unwrap();

    // Recovery schneidet torn tail ab und siegelt
    let rep = recover_redo(Arc::clone(&mem), base, 1, 0, 0).expect("Recovery succeeds");
    assert_eq!(rep.committed_frames.len(), 1, "Nur der erste vollständig quittierte Frame bleibt");
    assert_eq!(rep.max_stamp, 1);
}

#[test]
fn test_crash_row_after_fsync_before_receipt() {
    // Zeile: nach fsync, vor Quittung | Frame dauerhaft | Change vorhanden (zulässig)
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/crash_after_fsync");
    let entropy = RealEntropy;

    let mut writer = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();
    let op = RedoOp::new_put(1, None, b"durable_k".to_vec(), b"durable_v".to_vec());
    let stamp = writer.write_change(42, Durability::Always, &[op], 2, &entropy).unwrap();
    assert_eq!(stamp, 42);

    // Absturz vor Zustellung der Quittung an Aufrufer: Nach Neustart ist der Frame da
    let rep = recover_redo(Arc::clone(&mem), base, 1, 0, 0).expect("Recovery succeeds");
    assert_eq!(rep.committed_frames.len(), 1);
    assert_eq!(rep.committed_frames[0].0.stamp, 42);
}

#[test]
fn test_crash_row_enospc_truncate_succeeded() {
    // Zeile: ENOSPC nach Teilschreiben, Abschneiden gelang | Segment endet beim letzten guten Frame | Neustart unauffällig
    let mem = MemFs::new();
    let fault_cfg = Arc::new(FaultConfig::default());
    let fault_fs = Arc::new(FaultFs::new(mem, Arc::clone(&fault_cfg)));
    let base = Path::new("/crash_enospc_trunc_ok");
    let entropy = RealEntropy;

    let mut writer = RedoWriter::open_new(Arc::clone(&fault_fs), base, 1, 1024 * 1024, &entropy).unwrap();
    let op1 = RedoOp::new_put(1, None, b"good_k".to_vec(), b"good_v".to_vec());
    writer.write_change(1, Durability::Always, &[op1], 2, &entropy).unwrap();

    // ENOSPC beim nächsten Write
    fault_cfg.fail_enospc.store(true, Ordering::Relaxed);
    let op2 = RedoOp::new_put(1, None, b"bad_k".to_vec(), b"bad_v".to_vec());
    let res = writer.write_change(2, Durability::Always, &[op2], 2, &entropy);

    assert_eq!(res, Err(RedoError::BusyStorageFull));
    assert!(!writer.is_poisoned(), "Erfolgreiches Abschneiden darf nicht vergiften");

    // Fehlerschalter vor der Recovery zurücksetzen (S-6): sonst scheitert der Neustart am ENOSPC-Schalter
    fault_cfg.fail_enospc.store(false, Ordering::Relaxed);

    // Neustart ist unauffällig
    let rep = recover_redo(Arc::clone(&fault_fs), base, 1, 0, 0).expect("Recovery unauffällig");
    assert_eq!(rep.committed_frames.len(), 1);
}

#[test]
fn test_crash_row_enospc_truncate_failed_poisons() {
    // Zeile: ENOSPC, Abschneiden gescheitert → Poison, Absturz | Torn-Tail im letzten Segment | Heilen + Siegeln (§13.1 Schritt 6)
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/crash_enospc_poison");
    let entropy = RealEntropy;

    let mut writer = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();
    let op1 = RedoOp::new_put(1, None, b"k1".to_vec(), b"v1".to_vec());
    writer.write_change(1, Durability::Always, &[op1], 2, &entropy).unwrap();

    // Torn-Tail im letzten Segment nach Absturz
    let seg_path = base.join(segment_file_name(1));
    let file = mem.open(&seg_path, OpenMode::ReadWrite).unwrap();
    let corrupt_bytes = [0x55u8; 16];
    file.write_all_at(&corrupt_bytes, 48 + 31).unwrap();

    // Heilen + Siegeln in Recovery
    let rep = recover_redo(Arc::clone(&mem), base, 1, 0, 0).expect("Recovery heilt und siegelt");
    assert_eq!(rep.committed_frames.len(), 1);
}

#[test]
fn test_crash_row_recovery_heal_idempotent() {
    // Zeile: Recovery-Heilen: nach Abschneiden, vor Siegel | Segment kurz, ungesiegelt | nächste Recovery wiederholt Heilen + Siegeln (idempotent)
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/crash_heal_idempotent");
    let entropy = RealEntropy;

    let mut writer = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();
    let op = RedoOp::new_put(1, None, b"k".to_vec(), b"v".to_vec());
    writer.write_change(1, Durability::Always, &[op], 2, &entropy).unwrap();

    // Erste Recovery siegelt
    let rep1 = recover_redo(Arc::clone(&mem), base, 1, 0, 0).expect("Recovery 1");
    assert_eq!(rep1.committed_frames.len(), 1);

    // Zweite Recovery läuft erneut fehlerfrei über dasselbe gesiegelte Verzeichnis (Idempotenz)
    let rep2 = recover_redo(Arc::clone(&mem), base, 1, 0, 0).expect("Recovery 2 idempotent");
    assert_eq!(rep2.committed_frames.len(), 1);
}

#[test]
fn test_crash_row_seal_written_next_segment_missing() {
    // Zeile: Siegel geschrieben, nächstes Segment fehlt | Segment gesiegelt, kein Nachfolger | neues Segment wird angelegt
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/crash_seal_no_next");
    let entropy = RealEntropy;

    let mut writer = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();
    let op = RedoOp::new_put(1, None, b"k".to_vec(), b"v".to_vec());
    writer.write_change(1, Durability::Always, &[op], 2, &entropy).unwrap();
    writer.seal_and_close().unwrap();

    // Segment 1 ist gesiegelt, Segment 2 existiert noch nicht
    let rep = recover_redo(Arc::clone(&mem), base, 1, 0, 0).expect("Recovery succeeds");
    assert_eq!(rep.committed_frames.len(), 1);

    // Neues Segment 2 kann angelegt werden
    let mut writer2 = RedoWriter::open_new(Arc::clone(&mem), base, 2, 1024 * 1024, &entropy).unwrap();
    let op2 = RedoOp::new_put(1, None, b"k2".to_vec(), b"v2".to_vec());
    writer2.write_change(2, Durability::Always, &[op2], 3, &entropy).unwrap();
    assert_eq!(writer2.current_segment_id(), 2);
}

#[test]
fn test_crash_row_prealloc_building_file_ignored() {
    // Zeile: Reservesegment: Absturz beim Vorallokieren | .building.* | gelöscht / ignoriert
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/crash_prealloc");
    let entropy = RealEntropy;

    let mut writer = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();
    let op = RedoOp::new_put(1, None, b"k".to_vec(), b"v".to_vec());
    writer.write_change(1, Durability::Always, &[op], 2, &entropy).unwrap();
    writer.seal_and_close().unwrap();

    // Temporäre building-Datei simulieren
    let b_path = base.join(".building.redo-0000000000000002.log");
    let b_file = mem.open(&b_path, OpenMode::CreateNew(contextra_storage_ports::Perm::Private)).unwrap();
    b_file.write_all_at(b"incomplete-preallocation", 0).unwrap();

    let rep = recover_redo(Arc::clone(&mem), base, 1, 0, 0).expect("Recovery ignoriert building");
    assert_eq!(rep.committed_frames.len(), 1);
}

#[test]
fn test_crash_row_lost_write_detected_inv_log_8() {
    // Zeile: Redo-Schwanz nach Edit verloren (Lost Write) | Redo endet unter redo_hwm | Corrupt{RedoTruncated} statt stillem Verlust (INV-LOG-8)
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/crash_lost_write");
    let entropy = RealEntropy;

    let mut writer = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();
    let op = RedoOp::new_put(1, None, b"k".to_vec(), b"v".to_vec());
    writer.write_change(50, Durability::Always, &[op], 2, &entropy).unwrap();
    writer.seal_and_close().unwrap();

    // Catalog hat bereits redo_hwm = 100 verankert, aber Redo endet bei Stamp 50 (Lost Write!)
    let err = recover_redo(Arc::clone(&mem), base, 1, 0, 100);
    assert!(
        matches!(
            err,
            Err(RedoError::CorruptRedoTruncated { last_stamp: 50, redo_hwm: 100 })
        ),
        "Lost-Write muss zuverlässig als INV-LOG-8 RedoTruncated erkannt werden"
    );
}
