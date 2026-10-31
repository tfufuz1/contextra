#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! # Umfassende Transaktionsisolations- und Nebenläufigkeitstests
//!
//! **Ring:** Ring 0 (Transaktionsisolation)
//!
//! Prüft alle zentralen Isolationsgarantien deterministisch:
//! 1. Write-Skew-Schutz (klassisches Bankkonto-Szenario).
//! 2. Lost-Update-Schutz (Verhinderung verlorener Updates bei Zählern).
//! 3. Phantomschutz über Bereichsscan-Präfixe.
//! 4. Unabbrechbarkeit reiner Lese-Snapshots (Read-Only Snapshots).
//! 5. Lease- und Pin-Ablaufprüfung mit `ManualClock`.
//! 6. SnapshotRegistry-Lebenszyklus, Drop-Reihenfolgen und Min-Active-Tracking.
//! 7. SequenceLog-Grenzwerte, Tombstone-Maskierung und Kompaktierung.
//! 8. Multithread-Nebenläufigkeit mit Barriers.
//! 9. TxBuffer-Budgets (Operations- und Byte-Limits) sowie Orphan-Reaping.
//!
//! # Verified Behaviors:
//! 1. Write-Skew Protection (classic bank balance invariant).
//! 2. Lost-Update Protection.
//! 3. Phantom Protection via Range Prefix Reads.
//! 4. Read-Only Snapshot Isolation Preservation (never aborted).
//! 5. Pin Expiration & Reaping with `ManualClock`.
//! 6. `SnapshotRegistry` Lifecycle, Out-of-order Guard Dropping & Min-Active Calculation.
//! 7. `SequenceLog` Boundary Sequence Handling, Tombstone Masking & Compaction.
//! 8. Multi-Threaded Concurrency with `std::thread` and Barrier Synchronization.
//! 9. `TxBuffer` Staging, Capacity, Byte Budget Limits & Orphan Reaping.

use contextra_mvcc::seq_log::SequenceLog;
use contextra_mvcc::snapshot::SnapshotRegistry;
use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator};
use contextra_mvcc::tx_buffer::{IndexOp, TxBuffer, TxBufferConfig};
use contextra_mvcc::SsiValidator;
use contextra_testkit::ManualClock;
use contextra_types::{ContextraError, DocId, TxId, TOMBSTONE_BIT};
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::Barrier;
use std::time::Duration;

// --- 1. Write-Skew Protection Tests ---

// ANCHOR[contextra-mvcc:TEST-ISO-001]
/// Klassischer Write-Skew-Test: Zwei Transaktionen lesen Guthaben beider Konten und buchen gegenseitig ab.
#[test]
fn test_isolation_write_skew_classic_bank_account() {
    // Invariant: Account_A + Account_B >= 0. Initial balances: A = 100, B = 100.
    let validator = SequenceLogSsiValidator::new();
    let snap_seq = 10;

    // Tx1 attempts to withdraw 150 from A (checking B = 100 at snap_seq 10)
    let tx1 = TxId::new(101);
    let mut rs1 = ReadSet::new();
    rs1.record_read(b"account:A", snap_seq);
    rs1.record_read(b"account:B", snap_seq);

    // Tx2 attempts to withdraw 150 from B (checking A = 100 at snap_seq 10)
    let tx2 = TxId::new(102);
    let mut rs2 = ReadSet::new();
    rs2.record_read(b"account:A", snap_seq);
    rs2.record_read(b"account:B", snap_seq);

    // Tx1 validates and records its commit to A at seq 11
    let keys1 = vec![b"account:A".as_slice()];
    assert!(validator.validate_and_record(tx1, &rs1, keys1, 11).is_ok());

    // Tx2 attempts to validate its read set at seq 12 -> MUST fail write-skew validation because A was mutated at seq 11 > 10
    let res2 = validator.validate(tx2, &rs2);
    assert!(
        matches!(res2, Err(ContextraError::Conflict(_))),
        "Tx2 must fail validation due to write-skew on account:A"
    );
}

// ANCHOR[contextra-mvcc:TEST-ISO-002]
/// Prüft Write-Skew bei überlappenden ReadSets und disjunkten Schreibzielen.
#[test]
fn test_isolation_write_skew_overlapping_reads_disjoint_writes() {
    let validator = SequenceLogSsiValidator::new();
    let snap_seq = 20;

    // Tx1 reads keys 1 and 2, writes key 1
    let tx1 = TxId::new(1);
    let mut rs1 = ReadSet::new();
    rs1.record_read(b"key1", snap_seq);
    rs1.record_read(b"key2", snap_seq);

    // Tx2 reads keys 1 and 2, writes key 2
    let tx2 = TxId::new(2);
    let mut rs2 = ReadSet::new();
    rs2.record_read(b"key1", snap_seq);
    rs2.record_read(b"key2", snap_seq);

    // Tx1 validates read set and records key1 at seq 21
    let keys1 = vec![b"key1".as_slice()];
    assert!(validator.validate_and_record(tx1, &rs1, keys1, 21).is_ok());

    // Tx2 attempts to validate at seq 22 -> conflicts because key1 was modified after snap_seq 20
    let res2 = validator.validate(tx2, &rs2);
    assert!(matches!(res2, Err(ContextraError::Conflict(_))));
}

// ANCHOR[contextra-mvcc:TEST-ISO-003]
/// Verifiziert, dass Transaktionen mit völlig disjunkten ReadSets gleichzeitig erfolgreich committen.
#[test]
fn test_isolation_write_skew_non_overlapping_read_sets_succeed() {
    let validator = SequenceLogSsiValidator::new();

    let tx1 = TxId::new(10);
    let mut rs1 = ReadSet::new();
    rs1.record_read(b"alpha", 100);

    let tx2 = TxId::new(20);
    let mut rs2 = ReadSet::new();
    rs2.record_read(b"beta", 100);

    // Tx1 validates and commits "alpha" at seq 101
    let keys1 = vec![b"alpha".as_slice()];
    assert!(validator.validate_and_record(tx1, &rs1, keys1, 101).is_ok());

    // Tx2 validates and commits "beta" at seq 102 -> non-overlapping read/write set succeeds
    let keys2 = vec![b"beta".as_slice()];
    assert!(validator.validate_and_record(tx2, &rs2, keys2, 102).is_ok());
}

// --- 2. Lost-Update Protection Tests ---

// ANCHOR[contextra-mvcc:TEST-ISO-004]
/// Prüft Lost-Update-Erkennung bei konkurrierenden Inkrementierungen desselben Zählerschlüssels.
#[test]
fn test_isolation_lost_update_concurrent_counter_increment() {
    let validator = SequenceLogSsiValidator::new();
    let snap_seq = 50;

    // Concurrent transactions read counter key at snap_seq 50
    let tx1 = TxId::new(1);
    let mut rs1 = ReadSet::new();
    rs1.record_read(b"global_counter", snap_seq);

    let tx2 = TxId::new(2);
    let mut rs2 = ReadSet::new();
    rs2.record_read(b"global_counter", snap_seq);

    // Tx1 updates global_counter and commits at seq 51
    let keys1 = vec![b"global_counter".as_slice()];
    assert!(validator.validate_and_record(tx1, &rs1, keys1, 51).is_ok());

    // Tx2 attempts to commit global_counter -> lost-update detected
    let res2 = validator.validate(tx2, &rs2);
    assert!(
        matches!(res2, Err(ContextraError::Conflict(_))),
        "Lost-update on global_counter must fail validation"
    );
}

// ANCHOR[contextra-mvcc:TEST-ISO-005]
/// Testet die atomare Validierung und Registrierung (`validate_and_record`) gegen Race Conditions.
#[test]
fn test_isolation_lost_update_atomic_validate_and_record() {
    let validator = SequenceLogSsiValidator::new();
    let snap_seq = 10;

    let tx1 = TxId::new(100);
    let mut rs1 = ReadSet::new();
    rs1.record_read(b"shared_resource", snap_seq);

    let tx2 = TxId::new(200);
    let mut rs2 = ReadSet::new();
    rs2.record_read(b"shared_resource", snap_seq);

    // Atomic validate_and_record for Tx1
    let keys1 = vec![b"shared_resource".as_slice()];
    assert!(validator.validate_and_record(tx1, &rs1, keys1, 11).is_ok());

    // Tx2 attempting validate_and_record fails and leaves no write keys recorded
    let keys2 = vec![b"tx2_side_effect".as_slice()];
    let res2 = validator.validate_and_record(tx2, &rs2, keys2, 12);
    assert!(matches!(res2, Err(ContextraError::Conflict(_))));

    // Verify side effect key from failed tx2 was NOT recorded
    let mut rs_check = ReadSet::new();
    rs_check.record_read(b"tx2_side_effect", 10);
    assert!(validator.validate(TxId::new(300), &rs_check).is_ok());
}

// --- 3. Phantom Protection Tests ---

// ANCHOR[contextra-mvcc:TEST-ISO-006]
/// Verifiziert Phantomschutz: Einfügen eines Schlüssels unter ein gescanntes Präfix bricht Transaktion ab.
#[test]
fn test_isolation_phantom_read_prefix_match_conflicts() {
    let validator = SequenceLogSsiValidator::new();
    let snap_seq = 30;

    let tx_scan = TxId::new(1);
    let mut rs_scan = ReadSet::new();
    // Range scan over "user:tenant_1:"
    rs_scan.record_prefix(b"user:tenant_1:".to_vec(), snap_seq);

    // Concurrent insertion matching prefix committed at seq 35
    validator.record_commit_key(b"user:tenant_1:doc_999", 35);

    // Range scan validation must detect phantom insertion
    let res = validator.validate(tx_scan, &rs_scan);
    assert!(
        matches!(res, Err(ContextraError::Conflict(_))),
        "Phantom insertion matching range prefix must cause conflict"
    );
}

// ANCHOR[contextra-mvcc:TEST-ISO-007]
/// Bestätigt, dass Einfügungen außerhalb des gescannten Präfixes keinen Phantomkonflikt auslösen.
#[test]
fn test_isolation_phantom_read_unrelated_prefix_succeeds() {
    let validator = SequenceLogSsiValidator::new();
    let snap_seq = 30;

    let tx_scan = TxId::new(1);
    let mut rs_scan = ReadSet::new();
    rs_scan.record_prefix(b"user:tenant_1:".to_vec(), snap_seq);

    // Concurrent insertion under DIFFERENT prefix "user:tenant_2:"
    validator.record_commit_key(b"user:tenant_2:doc_001", 35);

    // Range scan validation under tenant_1 must succeed
    assert!(validator.validate(tx_scan, &rs_scan).is_ok());
}

// ANCHOR[contextra-mvcc:TEST-ISO-008]
/// Prüft Phantomschutz bei mehreren registrierten Präfixen in einer einzigen Transaktion.
#[test]
fn test_isolation_phantom_read_multiple_range_prefixes() {
    let validator = SequenceLogSsiValidator::new();
    let snap_seq = 40;

    let tx = TxId::new(10);
    let mut rs = ReadSet::new();
    rs.record_prefix(b"cat:A:".to_vec(), snap_seq);
    rs.record_prefix(b"cat:B:".to_vec(), snap_seq);

    // Insertion under cat:B: at seq 45
    validator.record_commit_key(b"cat:B:item_12", 45);

    let res = validator.validate(tx, &rs);
    assert!(matches!(res, Err(ContextraError::Conflict(_))));
}

// --- 4. Read-Only Snapshot Isolation Preservation Tests ---

// ANCHOR[contextra-mvcc:TEST-ISO-009]
/// Garantiert, dass reine Lese-Transaktionen niemals durch spätere Schreibtransaktionen abgebrochen werden.
#[test]
fn test_isolation_read_only_snapshot_never_aborted_by_writes() {
    let validator = SequenceLogSsiValidator::new();
    let snap_seq = 100;

    // Read-only transaction reads keys at snap_seq 100
    let tx_ro = TxId::new(500);
    let mut rs_ro = ReadSet::new();
    rs_ro.record_read(b"metric:cpu", snap_seq);
    rs_ro.record_read(b"metric:mem", snap_seq);

    // Multiple write transactions mutate other keys at seq 101, 102, 103
    validator.record_commit_key(b"metric:disk", 101);
    validator.record_commit_key(b"metric:net", 102);

    // Read-only transaction validation against non-mutated read set passes
    assert!(validator.validate(tx_ro, &rs_ro).is_ok());
}

// ANCHOR[contextra-mvcc:TEST-ISO-010]
/// Prüft, dass ein leeres ReadSet stets konfliktfrei validiert wird.
#[test]
fn test_isolation_read_only_snapshot_empty_read_set_always_valid() {
    let validator = SequenceLogSsiValidator::new();

    // Commit many writes
    for i in 1..=50 {
        validator.record_commit_key(
            format!("key_{i}").as_bytes(),
            u64::try_from(i).expect("valid u64"),
        );
    }

    let empty_rs = ReadSet::new();
    assert!(validator.validate(TxId::new(99), &empty_rs).is_ok());
}

// --- 5. Pin Expiration & Reaping with ManualClock Tests ---

// ANCHOR[contextra-mvcc:TEST-ISO-011]
/// Verifiziert das Erkennen abgelaufener Pins im SequenceLog mittels ManualClock.
#[test]
fn test_isolation_pin_expiration_sequence_log_manual_clock() {
    let clock = Arc::new(ManualClock::new(1_000_000_000));
    let mut log =
        SequenceLog::with_clock(clock.clone()).with_max_pin_duration(Duration::from_secs(300));

    // Pin snapshot at seq 100
    log.pin_snapshot(100);
    assert_eq!(log.min_retention_seq(), Some(100));
    assert!(log.expired_pins().is_empty());

    // Advance time by 200 seconds (not expired yet)
    clock.advance(Duration::from_secs(200));
    assert!(log.expired_pins().is_empty());

    // Advance time by 105 more seconds (total 305s > 300s max_pin_duration)
    clock.advance(Duration::from_secs(105));
    assert_eq!(log.expired_pins(), vec![100]);

    // Explicitly reap expired pins
    let reaped = log.reap_expired_pins();
    assert_eq!(reaped, vec![100]);
    assert_eq!(log.min_retention_seq(), None);
}

// ANCHOR[contextra-mvcc:TEST-ISO-012]
/// Verifiziert das automatische Reaping abgelaufener Pins in der SnapshotRegistry.
#[test]
fn test_isolation_pin_expiration_snapshot_registry_manual_clock() {
    let clock = Arc::new(ManualClock::new(10_000_000_000));
    let registry = Arc::new(SnapshotRegistry::with_clock(clock.clone()));

    registry.pin(50);
    registry.pin(150);
    assert_eq!(registry.min_active_seqno(), 50);

    // Advance time by 310 seconds
    clock.advance(Duration::from_secs(310));

    // Both pins expired relative to max_pin_duration 300s
    let reaped = registry.reap_expired_pins(Duration::from_secs(300));
    assert_eq!(reaped, vec![50, 150]);
    assert_eq!(registry.min_active_seqno(), u64::MAX);
}

// ANCHOR[contextra-mvcc:TEST-ISO-013]
/// Prüft die Diagnose des ältesten aktiven Snapshot-Pins über `longest_active_pin_at`.
#[test]
fn test_isolation_longest_active_pin_diagnostics_manual_clock() {
    let clock = Arc::new(ManualClock::new(1_000_000_000));
    let registry = Arc::new(SnapshotRegistry::with_clock(clock.clone()));

    let g1 = registry.register(200);
    clock.advance(Duration::from_secs(50));
    let _g2 = registry.register(300);

    // Longest pin should be g1 at seq 200 with 50s duration
    let (seq, dur) = registry.longest_active_pin().expect("active pin");
    assert_eq!(seq, 200);
    assert_eq!(dur, Duration::from_secs(50));

    drop(g1);
    // Now longest pin should be g2 at seq 300 with 0s duration
    let (seq2, dur2) = registry.longest_active_pin().expect("active pin");
    assert_eq!(seq2, 300);
    assert_eq!(dur2, Duration::from_secs(0));
}

// --- 6. SnapshotRegistry Lifecycle Tests ---

// ANCHOR[contextra-mvcc:TEST-ISO-014]
/// Verifiziert korrekte Neuberechnung von `min_active_seqno` beim Droppen von Guards in beliebiger Reihenfolge.
#[test]
fn test_isolation_snapshot_registry_out_of_order_guard_drop() {
    let registry = Arc::new(SnapshotRegistry::new());

    let g30 = registry.register(30);
    let g10 = registry.register(10);
    let g20 = registry.register(20);

    assert_eq!(registry.min_active_seqno(), 10);

    // Drop lowest guard (10) -> min updates to 20
    drop(g10);
    assert_eq!(registry.min_active_seqno(), 20);

    // Drop highest guard (30) -> min remains 20
    drop(g30);
    assert_eq!(registry.min_active_seqno(), 20);

    // Drop last guard (20) -> min resets to u64::MAX
    drop(g20);
    assert_eq!(registry.min_active_seqno(), u64::MAX);
}

// ANCHOR[contextra-mvcc:TEST-ISO-015]
/// Prüft das Reference-Counting bei mehrfacher Registrierung derselben Sequenznummer.
#[test]
fn test_isolation_snapshot_registry_ref_counting_same_seq() {
    let registry = Arc::new(SnapshotRegistry::new());

    let g1 = registry.register(100);
    let g2 = registry.register(100);
    let g3 = registry.register(100);

    assert_eq!(registry.min_active_seqno(), 100);

    drop(g1);
    assert_eq!(registry.min_active_seqno(), 100);
    drop(g2);
    assert_eq!(registry.min_active_seqno(), 100);
    drop(g3);
    assert_eq!(registry.min_active_seqno(), u64::MAX);
}

// --- 7. SequenceLog Boundary, Tombstone & Compaction Tests ---

// ANCHOR[contextra-mvcc:TEST-ISO-016]
/// Verifiziert das Maskieren des TOMBSTONE_BIT bei Einfügungen und Sichtbarkeitsprüfungen im SequenceLog.
#[test]
fn test_isolation_seq_log_tombstone_bit_masking() {
    let mut log = SequenceLog::new();
    let doc = DocId::from(42);

    let raw_seq = 100 | TOMBSTONE_BIT;
    log.record_insert(doc, raw_seq);

    // Visible at unmasked sequence 100
    assert!(log.is_visible(doc, 100));
    assert!(log.is_visible(doc, 100 | TOMBSTONE_BIT));
    assert!(!log.is_visible(doc, 99));
}

// ANCHOR[contextra-mvcc:TEST-ISO-017]
/// Testet Randfälle von Sequenznummern (z. B. 0 und u64::MAX) im SequenceLog.
#[test]
fn test_isolation_seq_log_boundary_seq_numbers() {
    let mut log = SequenceLog::new();
    let doc = DocId::from(999);

    let near_max = u64::MAX - 2;
    log.record_insert(doc, near_max);
    assert!(log.is_visible(doc, near_max));
    assert!(log.is_visible(doc, u64::MAX - 1));

    log.record_delete(doc, u64::MAX - 1);
    assert!(!log.is_visible(doc, u64::MAX - 1));
}

// ANCHOR[contextra-mvcc:TEST-ISO-018]
/// Prüft, dass Kompaktierung aktive Versionen oberhalb von `min_active_seqno` sicher bewahrt.
#[test]
fn test_isolation_seq_log_compaction_retains_active_versions() {
    let mut log = SequenceLog::new();
    let doc1 = DocId::from(1);
    let doc2 = DocId::from(2);

    log.record_insert(doc1, 10);
    log.record_delete(doc1, 20);

    log.record_insert(doc2, 15);
    log.record_delete(doc2, 40);

    // Compact below min_active_seqno = 30
    log.compact(30);

    // doc1 (deleted at 20 < 30) is purged
    assert_eq!(log.deletion_seq(doc1), None);
    // doc2 (deleted at 40 >= 30) is retained
    assert_eq!(log.deletion_seq(doc2), Some(40));
}

// --- 8. Multi-Threaded Concurrency Tests ---

// ANCHOR[contextra-mvcc:TEST-ISO-019]
/// Multithread-Stresstest für parallele Registrierung und Freigabe in der SnapshotRegistry.
#[test]
fn test_isolation_multithreaded_snapshot_registry_concurrency() {
    let registry = Arc::new(SnapshotRegistry::new());
    let num_threads = 8;
    let barrier = Arc::new(Barrier::new(num_threads));

    let mut handles = Vec::new();
    for i in 0..num_threads {
        let reg = registry.clone();
        let bar = barrier.clone();
        let seq = u64::try_from(i + 1).expect("valid u64") * 10;

        handles.push(std::thread::spawn(move || {
            bar.wait();
            let g = reg.register(seq);
            let min = reg.min_active_seqno();
            assert!(min <= seq);
            std::thread::yield_now();
            drop(g);
        }));
    }

    for h in handles {
        h.join().expect("thread panicked");
    }

    assert_eq!(registry.min_active_seqno(), u64::MAX);
}

// ANCHOR[contextra-mvcc:TEST-ISO-020]
/// Multithread-Test für gleichzeitige Validierungen und Commits im SSI-Validator.
#[test]
fn test_isolation_multithreaded_ssi_validator_concurrent_commits() {
    let validator = Arc::new(SequenceLogSsiValidator::new());
    let num_threads = 8;
    let barrier = Arc::new(Barrier::new(num_threads));

    let mut handles = Vec::new();
    for i in 0..num_threads {
        let val = validator.clone();
        let bar = barrier.clone();

        handles.push(std::thread::spawn(move || {
            bar.wait();
            let key = format!("thread_key_{i}").into_bytes();
            let seq = u64::try_from(i + 1).expect("valid u64") * 10;
            val.record_commit_key(&key, seq);
        }));
    }

    for h in handles {
        h.join().expect("thread panicked");
    }

    assert_eq!(validator.tracked_commit_keys(), num_threads);
}

// ANCHOR[contextra-mvcc:TEST-ISO-021]
/// Prüft paralleles Staging über mehrere Shards hinweg ohne Datenverlust oder Deadlocks.
#[test]
fn test_isolation_multithreaded_tx_buffer_sharded_concurrency() {
    let buffer = Arc::new(TxBuffer::<usize>::new_with_config(
        16,
        Duration::from_secs(30),
    ));
    let num_threads = 16;
    let barrier = Arc::new(Barrier::new(num_threads));

    let mut handles = Vec::new();
    for i in 0..num_threads {
        let buf = buffer.clone();
        let bar = barrier.clone();
        let tx = TxId::new(u64::try_from(i + 1).expect("valid u64"));

        handles.push(std::thread::spawn(move || {
            bar.wait();
            buf.begin(tx);
            let _ = buf.stage(
                tx,
                IndexOp::Insert {
                    doc_id: DocId::from(tx.inner()),
                    data: i,
                },
            );
            assert!(buf.has_tx(tx));
            let ops = buf.drain(tx);
            assert_eq!(ops.len(), 1);
        }));
    }

    for h in handles {
        h.join().expect("thread panicked");
    }

    assert!(buffer.is_empty());
}

// --- 9. TxBuffer Capacity, Budget & Orphan Reaping Tests ---

// ANCHOR[contextra-mvcc:TEST-ISO-022]
/// Verifiziert das Durchsetzen der maximalen Operationsgrenze pro Transaktion im TxBuffer.
#[test]
fn test_isolation_tx_buffer_bounded_ops_limit() {
    let config = TxBufferConfig {
        max_ops_per_tx: 3,
        ..Default::default()
    };
    let buffer = TxBuffer::<String>::new_with_config_ext(4, Duration::from_secs(10), config);
    let tx = TxId::new(1);
    buffer.begin(tx);

    for i in 0..3 {
        assert!(buffer
            .stage(
                tx,
                IndexOp::Insert {
                    doc_id: DocId::from(u64::try_from(i).expect("valid u64")),
                    data: format!("d{i}"),
                }
            )
            .is_ok());
    }

    // 4th stage exceeds max_ops_per_tx
    let res = buffer.stage(
        tx,
        IndexOp::Insert {
            doc_id: DocId::from(99),
            data: "overflow".to_string(),
        },
    );
    assert!(matches!(res, Err(ContextraError::Transaction(_))));
}

// ANCHOR[contextra-mvcc:TEST-ISO-023]
/// Verifiziert die Durchsetzung des Byte-Budgets pro Transaktion im TxBuffer.
#[test]
fn test_isolation_tx_buffer_tx_staged_bytes_budget() {
    let config = TxBufferConfig {
        max_tx_staged_bytes: 100,
        ..Default::default()
    };
    let buffer = TxBuffer::<String>::new_with_config_ext(4, Duration::from_secs(10), config);
    let tx = TxId::new(1);
    buffer.begin(tx);

    // Large payload that exceeds 100 bytes limit
    let large_payload = "A".repeat(200);
    let res = buffer.stage(
        tx,
        IndexOp::Insert {
            doc_id: DocId::from(1),
            data: large_payload,
        },
    );
    assert!(matches!(res, Err(ContextraError::Storage(_))));
}

// ANCHOR[contextra-mvcc:TEST-ISO-024]
/// Prüft zeitgesteuertes Orphan-Reaping im TxBuffer via ManualClock.
#[test]
fn test_isolation_tx_buffer_orphan_reaping_manual_clock() {
    let clock = Arc::new(ManualClock::new(10_000_000_000));
    let config = TxBufferConfig {
        tx_timeout: Duration::from_secs(10),
        ..Default::default()
    };
    let buffer = TxBuffer::<String>::new_with_clock_and_config(clock.clone(), 4, config);

    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);

    buffer.begin(tx1);
    clock.advance(Duration::from_secs(5));
    buffer.begin(tx2);

    // Advance clock past tx1 timeout (total elapsed for tx1: 12s > 10s)
    clock.advance(Duration::from_secs(7));

    let reaped = buffer.reap_orphans();
    assert_eq!(reaped, vec![tx1]);
    assert!(buffer.has_tx(tx2));
    assert!(!buffer.has_tx(tx1));
}

// ANCHOR[contextra-mvcc:TEST-ISO-025]
/// Verifiziert die Nachverfolgung der minimalen Snapshot-Sequenznummer im ReadSet.
#[test]
fn test_isolation_read_set_min_snapshot_seq_tracking() {
    let mut rs = ReadSet::new();
    assert_eq!(rs.min_snapshot_seq(), None);

    rs.record_read(b"k1", 100);
    rs.record_read(b"k2", 50);
    rs.record_read(b"k3", 200);

    assert_eq!(rs.min_snapshot_seq(), Some(50));

    rs.clear();
    assert_eq!(rs.min_snapshot_seq(), None);
}

// ANCHOR[contextra-mvcc:TEST-ISO-026]
/// Prüft Konflikterkennung über das angebundene SequenceLog im SSI-Validator.
#[test]
fn test_isolation_ssi_validator_with_sequence_log_conflict() {
    let seq_log = Arc::new(RwLock::new(SequenceLog::new()));
    let validator = SequenceLogSsiValidator::with_sequence_log(seq_log.clone());

    let doc_key = "doc_test_100";
    let doc_id = DocId::from_key(doc_key).expect("valid doc_id");

    let mut rs = ReadSet::new();
    rs.record_read(doc_key.as_bytes(), 10);

    // Valid before changes
    assert!(validator.validate(TxId::new(1), &rs).is_ok());

    // Record insert in sequence_log at seq 15 (> snapshot_seq 10)
    seq_log.write().record_insert(doc_id, 15);

    let res = validator.validate(TxId::new(1), &rs);
    assert!(matches!(res, Err(ContextraError::Conflict(_))));
}
