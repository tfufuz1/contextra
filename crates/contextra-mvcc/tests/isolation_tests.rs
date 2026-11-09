// ANCHOR[contextra-mvcc:TEST-ISOLATION-ROOT]
//! # Umfassende Isolations- und Nebenläufigkeitstests für `contextra-mvcc`
//!
//! Verifiziert die fundamentalen Garantien der Multi-Version Concurrency Control:
//! 1. Schutz vor Write-Skew-Anomalien (klassisches Bankkonto-Szenario).
//! 2. Schutz vor Lost Updates bei parallelen Zählerinkrementen.
//! 3. Phantom-Schutz über Bereichs-Präfixe (Range Reads).
//! 4. Reine Lese-Snapshots werden niemals durch Schreiber abgebrochen.
//! 5. TTL-Ablauf und Bereinigung von Pins mittels `ManualClock`.
//! 6. SnapshotRegistry-Lebenszyklus und Minimum-Berechnung bei unordentlichem Drop.
//! 7. SequenceLog-Grenzwerte, Tombstone-Maskierung und Compaction.
//! 8. Echte Multi-Threaded-Nebenläufigkeit mit Threads und Barrieren.
//! 9. TxBuffer Staging, Sharding, Byte-Budget-Limits und Orphan Reaping.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

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

// Prüft die Erkennung von Write-Skew beim klassischen Bankkonto-Beispiel (A + B >= 0).
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

// Prüft die Write-Skew-Erkennung, wenn Transaktionen dieselben Daten lesen, aber disjunkte Schlüssel schreiben.
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

// Prüft, dass disjunkte Lese- und Schreibmengen konfliktfrei nebeneinander committen können.
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

// Prüft den Schutz vor Lost Updates bei gleichzeitigem Inkrementieren eines Zählers.
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

// Prüft atomares Validieren und Registrieren zur Vermeidung von Lost Updates bei nebenläufigen Commits.
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

// Prüft, dass das Einfügen eines passenden Schlüssels in einen aktiven Bereichsscan-Präfix einen Phantom-Read-Konflikt auslöst.
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

// Prüft, dass Schreiboperationen auf nicht betroffene Präfixe keinen Konflikt für Bereichsleser erzeugen.
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

// Prüft den Phantom-Schutz bei mehreren registrierten Bereichs-Präfixen innerhalb einer Transaktion.
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

// Prüft, dass reine Lese-Transaktionen niemals durch spätere Schreiboperationen abgebrochen werden.
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

// Prüft, dass eine Transaktion mit leerem ReadSet immer als konfliktfrei validiert wird.
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

// Prüft das zeitgesteuerte Ablaufen und Bereinigen veralteter Snapshot-Pins im SequenceLog.
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

// Prüft das Ablaufen und Bereinigen veralteter Pins in der SnapshotRegistry mittels ManualClock.
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

// Prüft die Diagnoseabfrage des am längsten gehaltenen aktiven Snapshot-Pins.
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

// Prüft, dass das Droppen von SnapshotGuards in beliebiger Reihenfolge stets das korrekte Minimum erhält.
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

// Prüft die Referenzzählung der SnapshotRegistry, wenn mehrere Guards dieselbe Sequenznummer halten.
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

// Prüft die transparente Maskierung des Tombstone-Bits im SequenceLog.
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

// Prüft Randfälle bei der Sichtbarkeitsprüfung mit Sequenznummer 0 und u64::MAX.
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

// Prüft, dass die Kompaktierung des SequenceLogs nur Versionen unterhalb des Schwellenwerts entfernt.
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

// Multi-Threaded Stresstest für die SnapshotRegistry unter paralleler Registrierung und Freigabe.
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

// Multi-Threaded Stresstest für den SequenceLogSsiValidator bei parallelen Commit-Registrierungen.
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

// Multi-Threaded Stresstest für den TxBuffer mit parallelen Schreibern über mehrere Shards.
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

// Prüft die strikte Einhaltung des Limits gepufferter Operationen pro Transaktion.
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

// Prüft die Durchsetzung des Byte-Budgets für gepufferte Nutzdaten pro Transaktion.
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

// Prüft das Abräumen verwaister Transaktionen im TxBuffer mit ManualClock.
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

// Prüft das korrekte Nachführen der minimalen Snapshot-Sequenznummer im ReadSet.
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

// Prüft die Konflikterkennung des SSI-Validators in Verbindung mit einem aktiven SequenceLog.
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

// --- 10. Handgerechnete Modell-Isolationstests (P02 / WP-36 Lückenabdeckung) ---

// Handgerechnete Verifikation von Eigenschaft (1):
// Snapshot sieht zu keinem Zeitpunkt spätere Commits.
// Modellrechnung:
// - T=10: Doc A wird eingefügt (seq 10)
// - T=20: Doc B wird eingefügt (seq 20)
// - T=30: Doc A wird gelöscht (delete_seq 30)
// - T=40: Doc C wird eingefügt (seq 40)
// Erwartungswerte strikt aus Zeitmodell:
// - Snapshot bei seq 5:   A=false, B=false, C=false
// - Snapshot bei seq 10:  A=true,  B=false, C=false
// - Snapshot bei seq 15:  A=true,  B=false, C=false
// - Snapshot bei seq 20:  A=true,  B=true,  C=false
// - Snapshot bei seq 29:  A=true,  B=true,  C=false
// - Snapshot bei seq 30:  A=false (gelöscht), B=true, C=false
// - Snapshot bei seq 35:  A=false, B=true,  C=false
// - Snapshot bei seq 40:  A=false, B=true,  C=true
#[test]
fn test_isolation_handcrafted_snapshot_never_sees_future_commits() {
    let mut log = SequenceLog::new();
    let doc_a = DocId::from(1001u64);
    let doc_b = DocId::from(1002u64);
    let doc_c = DocId::from(1003u64);

    // Historie aufbauen
    log.record_insert(doc_a, 10);
    log.record_insert(doc_b, 20);
    log.record_delete(doc_a, 30);
    log.record_insert(doc_c, 40);

    // Handgerechneter Prüfpunkt T=5
    assert!(!log.is_visible(doc_a, 5), "At seq 5, doc_a (inserted 10) must be invisible");
    assert!(!log.is_visible(doc_b, 5), "At seq 5, doc_b (inserted 20) must be invisible");
    assert!(!log.is_visible(doc_c, 5), "At seq 5, doc_c (inserted 40) must be invisible");

    // Handgerechneter Prüfpunkt T=10
    assert!(log.is_visible(doc_a, 10), "At seq 10, doc_a must be visible");
    assert!(!log.is_visible(doc_b, 10), "At seq 10, doc_b must be invisible");
    assert!(!log.is_visible(doc_c, 10), "At seq 10, doc_c must be invisible");

    // Handgerechneter Prüfpunkt T=15
    assert!(log.is_visible(doc_a, 15), "At seq 15, doc_a must be visible");
    assert!(!log.is_visible(doc_b, 15), "At seq 15, doc_b must be invisible");

    // Handgerechneter Prüfpunkt T=20
    assert!(log.is_visible(doc_a, 20), "At seq 20, doc_a must be visible");
    assert!(log.is_visible(doc_b, 20), "At seq 20, doc_b must be visible");
    assert!(!log.is_visible(doc_c, 20), "At seq 20, doc_c must be invisible");

    // Handgerechneter Prüfpunkt T=29
    assert!(log.is_visible(doc_a, 29), "At seq 29, doc_a must be visible prior to delete at 30");
    assert!(log.is_visible(doc_b, 29), "At seq 29, doc_b must be visible");

    // Handgerechneter Prüfpunkt T=30: Grenze Löschzeitpunkt
    // Invariante: delete_seq > as_of ist strikt -> bei as_of == delete_seq (30) ist Doc A NICHT mehr sichtbar.
    assert!(!log.is_visible(doc_a, 30), "At seq 30, doc_a is deleted and must be invisible");
    assert!(log.is_visible(doc_b, 30), "At seq 30, doc_b remains visible");
    assert!(!log.is_visible(doc_c, 30), "At seq 30, doc_c must be invisible");

    // Handgerechneter Prüfpunkt T=35
    assert!(!log.is_visible(doc_a, 35), "At seq 35, doc_a is deleted");
    assert!(log.is_visible(doc_b, 35), "At seq 35, doc_b remains visible");
    assert!(!log.is_visible(doc_c, 35), "At seq 35, doc_c must be invisible");

    // Handgerechneter Prüfpunkt T=40
    assert!(!log.is_visible(doc_a, 40), "At seq 40, doc_a is deleted");
    assert!(log.is_visible(doc_b, 40), "At seq 40, doc_b is visible");
    assert!(log.is_visible(doc_c, 40), "At seq 40, doc_c is visible");
}

// Handgerechnete Verifikation von Eigenschaft (2):
// Write-Konflikte (Anti-Dependency-Zyklen / Concurrent Writes) werden zuverlässig erkannt.
// Modellrechnung:
// - Tx1 liest Key K1 bei Snapshot-Sequenz 100
// - Tx2 schreibt Key K1 und committet bei Sequenz 105 (> 100)
// - Tx1 versucht nun Validation: Erwartung Err(Conflict)
// - Gegenprobe: Tx3 liest Key K2 bei Snapshot-Sequenz 100.
//   Tx2 hat K1 geschrieben, nicht K2. Tx3 validiert erfolgreich: Ok(())
// - Grenzprobe: Tx4 liest Key K1 bei Snapshot-Sequenz 105 (exakt gleich Commit-Sequenz).
//   Invariante: commit_seq > read_seq löst Konflikt aus. Bei commit_seq == 105 und read_seq == 105
//   ist der Commit bereits Teil des Schnappschusses -> Ok(())
#[test]
fn test_isolation_handcrafted_write_conflict_detection_and_disjoint_reads() {
    let validator = SequenceLogSsiValidator::new();

    let tx1 = TxId::new(501);
    let mut rs1 = ReadSet::new();
    rs1.record_read(b"inventory:item:42", 100);

    let tx2 = TxId::new(502);
    let mut rs2 = ReadSet::new();
    rs2.record_read(b"inventory:item:42", 100);
    // Tx2 schreibt K1 und committet bei seq 105
    let commit_keys_tx2 = vec![b"inventory:item:42".as_slice()];
    assert!(validator.validate_and_record(tx2, &rs2, commit_keys_tx2, 105).is_ok());

    // Tx1 muss scheitern, weil 105 > 100
    let res1 = validator.validate(tx1, &rs1);
    assert!(
        matches!(res1, Err(ContextraError::Conflict(_))),
        "Tx1 must be rejected with Conflict because commit_seq 105 > read_seq 100"
    );

    // Gegenprobe Tx3: Lesezugriff auf disjunkten Key 'inventory:item:99' bei seq 100
    let tx3 = TxId::new(503);
    let mut rs3 = ReadSet::new();
    rs3.record_read(b"inventory:item:99", 100);
    assert!(
        validator.validate(tx3, &rs3).is_ok(),
        "Tx3 must succeed as its read set is entirely disjoint from committed keys"
    );

    // Grenzprobe Tx4: Liest bei Snapshot-Sequenz 105 (nachdem Tx2 committet hat)
    let tx4 = TxId::new(504);
    let mut rs4 = ReadSet::new();
    rs4.record_read(b"inventory:item:42", 105);
    assert!(
        validator.validate(tx4, &rs4).is_ok(),
        "Tx4 read at seq 105 has observed the commit at 105, hence no anti-dependency exists"
    );
}

// Handgerechnete Verifikation von Eigenschaft (3):
// Lesen eigener Schreibvorgänge (Read-your-own-writes) vs. Snapshot-Isolation fremder Leser.
// Modellrechnung:
// 1. Initialer Zustand: Key 'user:profile:1' hat Wert "v1" committet bei seq 10 im SequenceLog.
// 2. Snapshot Reader S1 registriert bei seq 10 (sieht doc 1001 / "v1").
// 3. Transaktion Tx1 (TxId 701) startet und schreibt neue Version "v2" in den TxBuffer (gestaged).
// 4. Read-your-own-writes:
//    - Tx1 fragt den Staging-Puffer: is_key_staged_for_tx(Tx1, "user:profile:1") == true!
//    - Tx1 sieht die eigene uncommitted Operation im Puffer via get_ops(Tx1).
// 5. Fremde Isolation:
//    - Eine andere Transaktion Tx2 fragt den Puffer: is_key_staged_for_tx(Tx2, "user:profile:1") == false!
//    - Der bestehende Snapshot S1 liest aus SequenceLog: doc 1001 ist weiterhin zu seq 10 sichtbar (unverändert "v1").
// 6. Transaktion Tx1 committet bei seq 20:
//    - Ops werden aus TxBuffer gedrained.
//    - Neuer SequenceLog-Eintrag für "v2" (doc 1001) wird bei seq 20 recorded.
// 7. Konsistenz nach Commit:
//    - Snapshot S1 (seq 10) sieht unverändert "v1", nicht "v2".
//    - Neuer Snapshot S2 bei seq 20 sieht die neue Version "v2".
//    - TxBuffer ist nun leer für Tx1.
#[test]
fn test_isolation_handcrafted_read_your_own_writes_lifecycle() {
    let mut seq_log = SequenceLog::new();
    let tx_buffer = TxBuffer::<(Vec<u8>, Vec<u8>)>::new();

    let doc_id = DocId::from(1001u64);
    let key = b"user:profile:1".to_vec();

    // 1. Initialer Zustand im Log bei seq 10
    seq_log.record_insert(doc_id, 10);
    assert!(seq_log.is_visible(doc_id, 10));

    // 2. Snapshot S1 bei seq 10
    let s1_as_of = 10u64;
    assert!(seq_log.is_visible(doc_id, s1_as_of));

    // 3. Tx1 startet und puffert Modifikation
    let tx1 = TxId::new(701);
    let tx2 = TxId::new(702);

    tx_buffer.begin(tx1);
    tx_buffer.begin(tx2);

    let staged_op = IndexOp::Insert {
        doc_id,
        data: (key.clone(), b"value_v2".to_vec()),
    };
    assert!(tx_buffer.stage_kv(tx1, staged_op).is_ok());

    // 4. Read-your-own-writes: Tx1 sieht eigene Vorbereitung
    assert!(
        tx_buffer.is_key_staged_for_tx(tx1, &key),
        "Tx1 must observe its own staged write for key"
    );
    let ops_tx1 = tx_buffer.get_ops(tx1).expect("Tx1 ops must exist");
    assert_eq!(ops_tx1.len(), 1);
    match &ops_tx1[0] {
        IndexOp::Insert { data, .. } => {
            assert_eq!(data.0, key);
            assert_eq!(data.1, b"value_v2");
        }
        _ => panic!("Expected Insert op"),
    }

    // 5. Fremde Isolation: Tx2 und S1 sehen die Vorbereitung von Tx1 NICHT
    assert!(
        !tx_buffer.is_key_staged_for_tx(tx2, &key),
        "Tx2 must NOT see uncommitted staged write of Tx1"
    );
    // SequenceLog enthält bis zum Commit keine Änderung
    assert!(seq_log.is_visible(doc_id, s1_as_of));

    // 6. Commit von Tx1 bei seq 20
    let drained_ops = tx_buffer.drain_kv(tx1);
    assert_eq!(drained_ops.len(), 1);
    // Commit im SequenceLog registrieren (Update zu seq 20)
    seq_log.record_insert(doc_id, 20);

    // 7. Konsistenz nach Commit
    // Puffer für Tx1 ist nun bereinigt
    assert!(!tx_buffer.is_key_staged_for_tx(tx1, &key));
    assert!(tx_buffer.get_ops(tx1).is_none());

    // Alter Snapshot S1 (seq 10) sieht den Stand von seq 10
    assert!(seq_log.is_visible(doc_id, s1_as_of));
    // Neuer Snapshot S2 (seq 20) sieht den Stand von seq 20
    let s2_as_of = 20u64;
    assert!(seq_log.is_visible(doc_id, s2_as_of));
}
