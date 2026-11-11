#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

//! P6 Integrationstests: Ledger-Durability, Hash-Verkettung, Verankerung und Fail-Closed (WP-8, INV-ERA-4).

use contextra_erasure::ledger::durable::{BatchRecordItem, ErasureLedger, LedgerAnchorProof};
use contextra_erasure::ledger::entry::GENESIS_HASH;
use contextra_erasure::ledger::error::LedgerError;
use contextra_erasure::ledger::record::{FramedRecord, LedgerFileHeader, RecordType};
use contextra_erasure::ledger::sink::InMemoryLedger;
use contextra_erasure::ledger::storage::{FileLedgerStorage, LedgerStorage, MemLedgerStorage};
use contextra_erasure::types_local::ErasureId;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Mock für Saga-Orchestrierung zur Überprüfung von INV-ERA-4:
/// "Jede Schlüsselvernichtung ist im Ledger vor dem Ende der Saga verankert."
struct MockSagaDriver {
    ledger: InMemoryLedger,
    key_destroyed: bool,
    saga_ended: bool,
    anchored_proof: Option<LedgerAnchorProof>,
}

impl MockSagaDriver {
    fn new(storage: Box<dyn LedgerStorage>, ledger_key: [u8; 32]) -> Self {
        let ledger = InMemoryLedger::with_storage(storage, ledger_key, true).unwrap();
        Self {
            ledger,
            key_destroyed: false,
            saga_ended: false,
            anchored_proof: None,
        }
    }

    /// Schritt 1: Schlüsselvernichtung (z.B. im Vault)
    fn step_destroy_key(&mut self) {
        assert!(!self.saga_ended, "Saga already ended");
        self.key_destroyed = true;
    }

    /// Schritt 2: Verankerung im Ledger vor Saga-Ende
    fn step_anchor_in_ledger(&mut self, erasure_id: ErasureId, gated_seq: u64, completed_at: u64) {
        assert!(
            self.key_destroyed,
            "Key must be destroyed before anchoring completion"
        );
        let mut commitment = [0u8; 32];
        if let Some(dst) = commitment.get_mut(0..16) {
            dst.copy_from_slice(erasure_id.as_bytes());
        }

        let item = BatchRecordItem {
            record_type: RecordType::KeyDestroyed,
            commitment,
            range_tags: vec![0x42; 8],
            gated_seq,
            completed_at,
        };
        let proof = self.ledger.append_batch(&[item]).unwrap();
        self.anchored_proof = Some(proof);
    }

    /// Schritt 3: Saga beenden
    fn step_end_saga(&mut self) -> Result<(), &'static str> {
        if !self.key_destroyed {
            return Err("Cannot end saga: Key was not destroyed");
        }
        if self.anchored_proof.is_none() {
            return Err(
                "INV-ERA-4 VIOLATION: Cannot end saga without anchoring key destruction in ledger",
            );
        }
        // Verifikation der Ledger-Kette
        if !self.ledger.verify_chain() {
            return Err("INV-ERA-4 VIOLATION: Ledger chain verification failed");
        }
        self.saga_ended = true;
        Ok(())
    }
}

/// Invariante INV-ERA-4: Jede Schlüsselvernichtung ist vor dem Ende der Saga im Ledger verankert.
#[test]
fn inv_era_4_key_destruction_is_anchored_before_saga_end() {
    let storage = Box::new(MemLedgerStorage::new());
    let ledger_key = [0x55; 32];
    let mut saga = MockSagaDriver::new(storage, ledger_key);

    let id = ErasureId::new([0x11; 16]);
    saga.step_destroy_key();

    // Versuch die Saga ohne Ledger-Verankerung zu beenden -> MUSS fehlschlagen!
    let early_end = saga.step_end_saga();
    assert!(
        early_end.is_err(),
        "INV-ERA-4: Saga must refuse to end if key destruction is not anchored in ledger"
    );

    // Jetzt im Ledger verankern
    saga.step_anchor_in_ledger(id, 100, 1_000_000);
    assert!(saga.anchored_proof.is_some());
    let proof = saga.anchored_proof.as_ref().unwrap();
    assert_eq!(proof.last_seq, 100);
    assert_ne!(proof.chain_hash, GENESIS_HASH);

    // Nun muss das Beenden der Saga gelingen
    assert!(saga.step_end_saga().is_ok());
    assert!(saga.saga_ended);
}

/// Test: Öffnen eines nicht existierenden Ledgers mit create_if_missing: false schlägt fail-closed fehl.
#[test]
fn open_missing_ledger_fails_closed() {
    let temp_dir = tempfile::tempdir().unwrap();
    let ledger_path = temp_dir.path().join("missing.ledger");
    let storage = Box::new(FileLedgerStorage::new(&ledger_path));

    let res = ErasureLedger::open(storage, [0xaa; 32], false);
    assert!(res.is_err());
    match res.unwrap_err() {
        LedgerError::NotFound => {}
        other => panic!("Expected LedgerError::NotFound, got {:?}", other),
    }

    // Auch über InMemoryLedger::with_storage
    let storage2 = Box::new(FileLedgerStorage::new(&ledger_path));
    let res2 = InMemoryLedger::with_storage(storage2, [0xaa; 32], false);
    assert!(res2.is_err());
    match res2.unwrap_err() {
        LedgerError::NotFound => {}
        other => panic!("Expected LedgerError::NotFound, got {:?}", other),
    }
}

/// Test: Öffnen eines beschädigten Ledgers (defekter Header oder CRC) schlägt fail-closed fehl.
#[test]
fn open_corrupted_ledger_fails_closed() {
    let storage = Box::new(MemLedgerStorage::new());
    let ledger_key = [0x33; 32];

    // 1. Defekte Magic-Bytes im Header
    let mut corrupt_header = [0u8; 16];
    if let Some(dst) = corrupt_header.get_mut(0..8) {
        dst.copy_from_slice(b"BADMAGIC");
    }
    storage.append_atomic(&corrupt_header).unwrap();

    let res = ErasureLedger::open(storage.clone(), ledger_key, false);
    assert!(res.is_err());
    match res.unwrap_err() {
        LedgerError::Corrupt(msg) => assert!(msg.contains("Invalid ledger magic")),
        other => panic!("Expected Corrupt, got {:?}", other),
    }

    // 2. Gültiger Header, aber abgerissener / unvollständiger Record
    let valid_header = LedgerFileHeader::new().to_bytes();
    let mut data = valid_header.to_vec();
    data.extend_from_slice(&[0x01, 0x02, 0x03]); // Nur 3 Bytes statt vollständigem Record

    let storage2 = Box::new(MemLedgerStorage::with_bytes(data));
    let res2 = ErasureLedger::open(storage2, ledger_key, false);
    assert!(res2.is_err());
    match res2.unwrap_err() {
        LedgerError::Corrupt(_) => {}
        other => panic!("Expected Corrupt on truncated slice, got {:?}", other),
    }

    // 3. Korrumpierter CRC32 in einem Record
    let storage3 = Box::new(MemLedgerStorage::new());
    let mut ledger = ErasureLedger::open(storage3.clone(), ledger_key, true).unwrap();
    ledger
        .append_batch(&[BatchRecordItem {
            record_type: RecordType::Completed,
            commitment: [0x12; 32],
            range_tags: vec![1, 2, 3],
            gated_seq: 42,
            completed_at: 100,
        }])
        .unwrap();

    // Manipuliere 1 Bit in den Daten
    let mut raw = storage3.read_all().unwrap();
    let last_idx = raw.len().saturating_sub(1);
    if let Some(byte) = raw.get_mut(last_idx) {
        *byte ^= 0xFF; // Bitflip in der Prüfsumme
    }
    let storage_tampered = Box::new(MemLedgerStorage::with_bytes(raw));

    let res3 = ErasureLedger::open(storage_tampered, ledger_key, false);
    assert!(res3.is_err());
    match res3.unwrap_err() {
        LedgerError::Corrupt(msg) => assert!(msg.contains("CRC mismatch")),
        other => panic!("Expected CRC Corrupt error, got {:?}", other),
    }
}

/// Test: Erkennung einer gebrochenen Hash-Kette.
#[test]
fn broken_hash_chain_detected() {
    let storage = Box::new(MemLedgerStorage::new());
    let ledger_key = [0x77; 32];
    let mut ledger = ErasureLedger::open(storage.clone(), ledger_key, true).unwrap();

    ledger
        .append_batch(&[
            BatchRecordItem {
                record_type: RecordType::Completed,
                commitment: [0x01; 32],
                range_tags: vec![0x10],
                gated_seq: 10,
                completed_at: 1000,
            },
            BatchRecordItem {
                record_type: RecordType::Completed,
                commitment: [0x02; 32],
                range_tags: vec![0x20],
                gated_seq: 20,
                completed_at: 2000,
            },
            BatchRecordItem {
                record_type: RecordType::Completed,
                commitment: [0x03; 32],
                range_tags: vec![0x30],
                gated_seq: 30,
                completed_at: 3000,
            },
        ])
        .unwrap();

    assert!(ledger.verify_chain().is_ok());

    // Manipuliere Commitment von Record 2 und passe CRC neu an (sodass CRC stimmt, aber Kette bricht)
    let raw = storage.read_all().unwrap();
    let mut parsed_records = Vec::new();
    let mut offset = LedgerFileHeader::SIZE;
    while offset < raw.len() {
        let slice = raw.get(offset..).unwrap();
        let (rec, consumed) = FramedRecord::parse(slice).unwrap();
        parsed_records.push(rec);
        offset += consumed;
    }
    assert_eq!(parsed_records.len(), 3);

    // Manipulation: Commitment von Record 2 ändern, ohne chain_hash neu zu berechnen
    if let Some(rec) = parsed_records.get_mut(1) {
        if let Some(b) = rec.commitment.get_mut(0) {
            *b ^= 0xEE;
        }
    }

    let mut tampered_bytes = LedgerFileHeader::new().to_bytes().to_vec();
    for r in &parsed_records {
        tampered_bytes.extend_from_slice(&r.serialize());
    }

    let storage_broken = Box::new(MemLedgerStorage::with_bytes(tampered_bytes));
    let open_res = ErasureLedger::open(storage_broken, ledger_key, false);
    assert!(open_res.is_err());
    match open_res.unwrap_err() {
        LedgerError::BrokenChain { seq, .. } => {
            assert_eq!(seq, 20);
        }
        other => panic!("Expected BrokenChain error at seq 20, got {:?}", other),
    }
}

/// Speicher-Wrapper zur Simulation von E/A-Fehlern während des Schreibens oder Fsync.
#[derive(Clone)]
struct FaultStorage {
    inner: MemLedgerStorage,
    fail_append: Arc<AtomicBool>,
    fail_sync: Arc<AtomicBool>,
}

impl FaultStorage {
    fn new() -> Self {
        Self {
            inner: MemLedgerStorage::new(),
            fail_append: Arc::new(AtomicBool::new(false)),
            fail_sync: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl LedgerStorage for FaultStorage {
    fn exists(&self) -> bool {
        self.inner.exists()
    }

    fn read_all(&self) -> Result<Vec<u8>, LedgerError> {
        self.inner.read_all()
    }

    fn append_atomic(&self, data: &[u8]) -> Result<(), LedgerError> {
        if self.fail_append.load(Ordering::SeqCst) {
            return Err(LedgerError::Storage(
                std::io::ErrorKind::BrokenPipe,
                "Simulated append disk failure".to_string(),
            ));
        }
        self.inner.append_atomic(data)
    }

    fn sync(&self) -> Result<(), LedgerError> {
        if self.fail_sync.load(Ordering::SeqCst) {
            return Err(LedgerError::Storage(
                std::io::ErrorKind::Other,
                "Simulated fsync flush failure".to_string(),
            ));
        }
        self.inner.sync()
    }

    fn truncate(&self, len: usize) -> Result<(), LedgerError> {
        self.inner.truncate(len)
    }

    fn len(&self) -> Result<usize, LedgerError> {
        self.inner.len()
    }
}

/// Test: Ein simulierter Absturz / E/A-Fehler während des Batch-Appends hinterlässt keinen partiellen Batch.
#[test]
fn atomic_batch_append_crash_leaves_no_partial_batch() {
    let fault_storage = FaultStorage::new();
    let ledger_key = [0x99; 32];
    let mut ledger =
        ErasureLedger::open(Box::new(fault_storage.clone()), ledger_key, true).unwrap();

    // Erster initialer Batch erfolgreich
    ledger
        .append_batch(&[BatchRecordItem {
            record_type: RecordType::Completed,
            commitment: [0x11; 32],
            range_tags: vec![1],
            gated_seq: 10,
            completed_at: 1000,
        }])
        .unwrap();

    let initial_len = fault_storage.len().unwrap();
    let initial_hash = *ledger.last_hash();
    let initial_records_count = ledger.records().len();

    // 1. Simulierter Fehler bei append_atomic
    fault_storage.fail_append.store(true, Ordering::SeqCst);
    let fail_res1 = ledger.append_batch(&[
        BatchRecordItem {
            record_type: RecordType::Completed,
            commitment: [0x22; 32],
            range_tags: vec![2],
            gated_seq: 20,
            completed_at: 2000,
        },
        BatchRecordItem {
            record_type: RecordType::Completed,
            commitment: [0x33; 32],
            range_tags: vec![3],
            gated_seq: 30,
            completed_at: 3000,
        },
    ]);
    assert!(fail_res1.is_err());

    // Zustand darf weder auf Platte noch im RAM verändert worden sein!
    assert_eq!(fault_storage.len().unwrap(), initial_len);
    assert_eq!(ledger.records().len(), initial_records_count);
    assert_eq!(ledger.last_hash(), &initial_hash);

    // 2. Simulierter Fehler bei sync (durabler Sync schlägt fehl)
    fault_storage.fail_append.store(false, Ordering::SeqCst);
    fault_storage.fail_sync.store(true, Ordering::SeqCst);

    let fail_res2 = ledger.append_batch(&[BatchRecordItem {
        record_type: RecordType::Completed,
        commitment: [0x44; 32],
        range_tags: vec![4],
        gated_seq: 40,
        completed_at: 4000,
    }]);
    assert!(fail_res2.is_err());

    // Durch Rollback / Truncate verbleibt kein partieller Batch
    assert_eq!(fault_storage.len().unwrap(), initial_len);
    assert_eq!(ledger.records().len(), initial_records_count);
    assert_eq!(ledger.last_hash(), &initial_hash);

    // Nach Behebung des Fehlers gelingt der Batch und die Kette ist konsistent
    fault_storage.fail_sync.store(false, Ordering::SeqCst);
    let success = ledger.append_batch(&[BatchRecordItem {
        record_type: RecordType::Completed,
        commitment: [0x55; 32],
        range_tags: vec![5],
        gated_seq: 50,
        completed_at: 5000,
    }]);
    assert!(success.is_ok());
    assert_eq!(ledger.records().len(), 2);
    assert!(ledger.verify_chain().is_ok());
}
