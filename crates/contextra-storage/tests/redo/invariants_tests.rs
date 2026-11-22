//! # Redo-Invarianten-Tests (INV-LOG-1 bis INV-LOG-8)
//!
//! SPEC §18 Invariantenkatalog:
//! Jede Invariante wird per ID zitiert und systematisch verifiziert.

use contextra_storage::env::entropy::RealEntropy;
use contextra_storage::redo::driver::{segment_file_name, RedoWriter};
use contextra_storage::redo::error::{check_redo_hwm, RedoError};
use contextra_storage::redo::frame::{Durability, RedoFrame};
use contextra_storage::redo::header::RedoHeader;
use contextra_storage::redo::op::RedoOp;
use contextra_storage::redo::replay::recover_redo;
use contextra_storage::redo::replay_sm::{ReplayEffect, ReplayEvent, ReplayStateMachine};
use contextra_storage::testing::fault_fs::{FaultConfig, FaultFs};
use contextra_storage::testing::mem_fs::MemFs;
use contextra_storage_ports::{Fs, OpenMode, Perm};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[test]
fn test_inv_log_1_one_frame_one_change() {
    // // INV-LOG-1: Ein Frame = ein Change.
    let mem = MemFs::new();
    let base = Path::new("/redo_inv1");
    let entropy = RealEntropy;
    let mut writer = RedoWriter::open_new(Arc::new(mem), base, 1, 1024 * 1024, &entropy).unwrap();

    let op1 = RedoOp::new_put(1, None, b"k1".to_vec(), b"v1".to_vec());
    let op2 = RedoOp::new_put(1, None, b"k2".to_vec(), b"v2".to_vec());

    // Jeder Change entspricht genau einem Frame
    let stamp1 = writer.write_change(10, Durability::Always, &[op1], 2, &entropy).unwrap();
    let stamp2 = writer.write_change(11, Durability::Always, &[op2], 2, &entropy).unwrap();

    assert_eq!(stamp1, 10);
    assert_eq!(stamp2, 11);
}

#[test]
fn test_inv_log_2_replay_valid_or_ends_log() {
    // // INV-LOG-2: Beim Replay ist ein Frame entweder vollständig gültig oder er beendet den Log.
    let chain_seed = 0x12345678u32;
    let header = RedoHeader::new(1, 1, chain_seed, 1024 * 1024);
    let mut data = Vec::new();
    data.extend_from_slice(&header.to_bytes());

    let op1 = RedoOp::new_put(1, None, b"k1".to_vec(), b"v1".to_vec());
    let (f1, chk1) = RedoFrame::encode_plain(1, Durability::Always, &[op1], chain_seed).unwrap();
    data.extend_from_slice(&f1.to_bytes());

    // Unvollständiges Frame am Ende (abgeschnitten)
    let op2 = RedoOp::new_put(1, None, b"k2".to_vec(), b"v2".to_vec());
    let (f2, _) = RedoFrame::encode_plain(2, Durability::Always, &[op2], chk1).unwrap();
    let f2_bytes = f2.to_bytes();
    data.extend_from_slice(&f2_bytes[..f2_bytes.len() / 2]);

    let mut rsm = ReplayStateMachine::new();
    let effects = rsm.step(ReplayEvent::Chunk(&data));
    let eof_effects = rsm.step(ReplayEvent::Eof);

    let mut valid_frames = 0;
    let mut found_torn_tail = false;

    for eff in effects.into_iter().chain(eof_effects.into_iter()) {
        match eff {
            ReplayEffect::ValidFrame { .. } => valid_frames += 1,
            ReplayEffect::TornTail { .. } => found_torn_tail = true,
            _ => {}
        }
    }

    assert_eq!(valid_frames, 1, "Nur das erste vollständige Frame darf angewendet werden");
    assert!(found_torn_tail, "Unvollständiger Frame beendet den Log als TornTail");
}

#[test]
fn test_inv_log_3_strictly_increasing_stamps() {
    // // INV-LOG-3: Stamps im Log streng aufsteigend (nicht lückenlos).
    let chain_seed = 0x87654321u32;
    let header = RedoHeader::new(1, 1, chain_seed, 1024 * 1024);
    let mut data = Vec::new();
    data.extend_from_slice(&header.to_bytes());

    let op1 = RedoOp::new_put(1, None, b"k1".to_vec(), b"v1".to_vec());
    let (f1, chk1) = RedoFrame::encode_plain(5, Durability::Always, &[op1], chain_seed).unwrap();
    data.extend_from_slice(&f1.to_bytes());

    // Lücke im Stamp ist erlaubt (z.B. von 5 auf 10)
    let op2 = RedoOp::new_put(1, None, b"k2".to_vec(), b"v2".to_vec());
    let (f2, chk2) = RedoFrame::encode_plain(10, Durability::Always, &[op2], chk1).unwrap();
    data.extend_from_slice(&f2.to_bytes());

    // Nicht streng aufsteigend (Stamp 10 erneut oder kleiner) ist ein Defekt
    let op3 = RedoOp::new_put(1, None, b"k3".to_vec(), b"v3".to_vec());
    let (f3, _) = RedoFrame::encode_plain(10, Durability::Always, &[op3], chk2).unwrap();
    data.extend_from_slice(&f3.to_bytes());

    let mut rsm = ReplayStateMachine::new();
    let effects = rsm.step(ReplayEvent::Chunk(&data));

    let mut corrupt_found = false;
    for eff in effects {
        if let ReplayEffect::CorruptDefect { cause, .. } = eff {
            if cause.contains("INV-LOG-3") {
                corrupt_found = true;
            }
        }
    }
    assert!(corrupt_found, "Nicht streng aufsteigender Stamp muss als INV-LOG-3 Defekt erkannt werden");
}

#[test]
fn test_inv_log_4_new_instance_starts_new_segment() {
    // // INV-LOG-4: Jede Prozessinstanz beginnt ein neues Segment; nie anhängen.
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/redo_inv4");
    let entropy = RealEntropy;

    // Instanz 1 beginnt bei Segment 1
    let mut writer1 = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();
    let op = RedoOp::new_put(1, None, b"k".to_vec(), b"v".to_vec());
    writer1.write_change(1, Durability::Always, &[op], 2, &entropy).unwrap();
    writer1.seal_and_close().unwrap();

    // Neue Instanz 2 beginnt Segment 2 (nie anhängen an Segment 1)
    let writer2 = RedoWriter::open_new(Arc::clone(&mem), base, 2, 1024 * 1024, &entropy).unwrap();
    assert_eq!(writer2.current_segment_id(), 2);
}

#[test]
fn test_inv_log_5_fsync_failure_poisons() {
    // // INV-LOG-5: fsync-Fehler => Poison, nie Retry, nichts in die Memtable.
    let mem = MemFs::new();
    let fault_cfg = Arc::new(FaultConfig::default());
    let fault_fs = Arc::new(FaultFs::new(mem, Arc::clone(&fault_cfg)));
    let base = Path::new("/redo_inv5");
    let entropy = RealEntropy;

    let mut writer = RedoWriter::open_new(Arc::clone(&fault_fs), base, 1, 1024 * 1024, &entropy).unwrap();

    // Aktiviere Sync-Fehler
    fault_cfg.fail_sync.store(true, Ordering::Relaxed);

    let op = RedoOp::new_put(1, None, b"k".to_vec(), b"v".to_vec());
    let res = writer.write_change(1, Durability::Always, &[op], 2, &entropy);

    assert!(matches!(res, Err(RedoError::Poisoned { .. })), "Fsync-Fehler muss Poisoned auslösen");
    assert!(writer.is_poisoned(), "Instanz muss dauerhaft vergiftet bleiben (INV-LOG-5)");

    // Zweiter Versuch muss sofort mit Poisoned abgewiesen werden
    let res2 = writer.write_change(2, Durability::Always, &[], 2, &entropy);
    assert!(matches!(res2, Err(RedoError::Poisoned { .. })));
}

#[test]
fn test_inv_log_6_sealed_segments_and_frame_count() {
    // // INV-LOG-6: Jedes Segment außer dem neuesten mit Frames ist gesiegelt; Defekt in gesiegeltem Segment => Corrupt; frame_count passt.
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/redo_inv6");
    let entropy = RealEntropy;

    // Segment 1 schreiben und siegeln
    let mut writer1 = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();
    let op = RedoOp::new_put(1, None, b"k1".to_vec(), b"v1".to_vec());
    writer1.write_change(1, Durability::Always, &[op], 2, &entropy).unwrap();
    writer1.seal_and_close().unwrap();

    // Segment 2 ungesiegelt lassen
    let mut writer2 = RedoWriter::open_new(Arc::clone(&mem), base, 2, 1024 * 1024, &entropy).unwrap();
    let op2 = RedoOp::new_put(1, None, b"k2".to_vec(), b"v2".to_vec());
    writer2.write_change(2, Durability::Always, &[op2], 3, &entropy).unwrap();
    // Kein Siegel

    // Recovery muss ungesiegeltes Segment 2 heilen und siegeln
    let rep = recover_redo(Arc::clone(&mem), base, 1, 0, 0).expect("Recovery succeeds");
    assert_eq!(rep.committed_frames.len(), 2);
}

#[test]
fn test_inv_log_6_corrupt_in_sealed_segment() {
    // // INV-LOG-6: Defekt in gesiegeltem Segment => Corrupt
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/redo_inv6_corrupt");
    let entropy = RealEntropy;

    let mut writer = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();
    let op = RedoOp::new_put(1, None, b"k".to_vec(), b"v".to_vec());
    writer.write_change(1, Durability::Always, &[op], 2, &entropy).unwrap();
    writer.seal_and_close().unwrap();

    // Bitflip in Segment 1 manipulieren
    let seg1_path = base.join(segment_file_name(1));
    let file = mem.open(&seg1_path, OpenMode::ReadWrite).unwrap();
    let mut b = [0u8; 1];
    file.read_at(&mut b, 55).unwrap();
    b[0] ^= 0xFF;
    file.write_all_at(&b, 55).unwrap();

    let err = recover_redo(Arc::clone(&mem), base, 1, 0, 0);
    assert!(matches!(err, Err(RedoError::Corrupt(_))), "Defekt in gesiegeltem Segment muss Corrupt sein");
}

#[test]
fn test_inv_log_7_reserve_segment_alive() {
    // // INV-LOG-7: Ein Segment ohne Frames mit gültigem Header und id >= redo_floor ist lebendig (Reserve).
    let mem = Arc::new(MemFs::new());
    let base = Path::new("/redo_inv7");
    let entropy = RealEntropy;

    // Segment 1 schreiben und siegeln
    let mut writer1 = RedoWriter::open_new(Arc::clone(&mem), base, 1, 1024 * 1024, &entropy).unwrap();
    let op = RedoOp::new_put(1, None, b"k1".to_vec(), b"v1".to_vec());
    writer1.write_change(1, Durability::Always, &[op], 2, &entropy).unwrap();
    writer1.seal_and_close().unwrap();

    // Reservesegment 2 vorallokieren
    RedoWriter::preallocate_reserve(&*mem, base, 2, 1024 * 1024, &entropy).unwrap();

    // Recovery muss Reservesegment 2 erkennen und übernehmen (INV-LOG-7)
    let rep = recover_redo(Arc::clone(&mem), base, 1, 0, 0).expect("Recovery succeeds");
    assert_eq!(rep.reserve_segment_id, Some(2), "Reservesegment 2 muss erhalten bleiben");
}

#[test]
fn test_inv_log_8_redo_hwm_truncation_detection() {
    // // INV-LOG-8: Letzter gültiger Redo-Stamp < redo_hwm (ohne redo_floor_stamp-Erklärung) => Corrupt{RedoTruncated}.
    // Fall 1: Redo endet unter redo_hwm
    let res = check_redo_hwm(Some(50), 0, 100);
    assert!(matches!(
        res,
        Err(RedoError::CorruptRedoTruncated { last_stamp: 50, redo_hwm: 100 })
    ));

    // Fall 2: Redo reicht bis oder über redo_hwm
    let ok1 = check_redo_hwm(Some(100), 0, 100);
    assert!(ok1.is_ok());

    let ok2 = check_redo_hwm(Some(120), 0, 100);
    assert!(ok2.is_ok());

    // Fall 3: Redo-Floor-Stamp erklärt die Lücke (z.B. nach Flush aller Segmente bis 100)
    // // INV-FLUSH-1: Flush-Barriere synchronisiert Redo vor dem Catalog-Edit
    let ok3 = check_redo_hwm(None, 100, 100);
    assert!(ok3.is_ok());
}
