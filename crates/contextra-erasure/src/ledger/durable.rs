//! Persistente, hash-verkettete ErasureLedger-Implementierung (WP-8, SPEC §11.8, ER-30).
//! Herkunft: e0a7395f1:crates/contextra-store/src/erasure_ledger/mod.rs:540

use super::entry::GENESIS_HASH;
use super::error::LedgerError;
use super::record::{FramedRecord, LedgerFileHeader, RecordType};
use super::storage::LedgerStorage;
use std::fmt;

/// Bestätigung für einen atomaren Batch-Append als Verankerungsnachweis (WP-8, INV-ERA-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerAnchorProof {
    /// Höchste Sequenznummer nach diesem Batch.
    pub last_seq: u64,
    /// Aktueller Ketten-Hash nach diesem Batch.
    pub chain_hash: [u8; 32],
    /// Anzahl der im Batch verankerten Einträge.
    pub count: usize,
}

/// Eingabedaten für einen Batch-Append im Ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchRecordItem {
    /// Record-Typ (z. B. Completed oder KeyDestroyed)
    pub record_type: RecordType,
    /// Commitment-Hash
    pub commitment: [u8; 32],
    /// Bereichs-Tags (opak)
    pub range_tags: Vec<u8>,
    /// Gate-Sequenznummer
    pub gated_seq: u64,
    /// Zeitstempel des Abschlusses (Mikrosekunden UTC)
    pub completed_at: u64,
}

/// Persistentes, hash-verkettetes Ledger (WP-8, CTRALEDG v1).
pub struct ErasureLedger {
    storage: Box<dyn LedgerStorage>,
    ledger_key: [u8; 32],
    last_hash: [u8; 32],
    max_gated_seq: u64,
    records: Vec<FramedRecord>,
}

impl fmt::Debug for ErasureLedger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErasureLedger")
            .field("last_hash", &self.last_hash)
            .field("max_gated_seq", &self.max_gated_seq)
            .field("records_count", &self.records.len())
            .finish()
    }
}

impl ErasureLedger {
    /// Öffnet oder erstellt ein ErasureLedger über den angegebenen Speicher.
    ///
    /// Fail-Closed Verhalten:
    /// - `create_if_missing == false` und Speicher leer/nicht vorhanden: `LedgerError::NotFound`.
    /// - Beschädigte Header/Records, CRC-Fehler: `LedgerError::Corrupt`.
    /// - Nicht unterstützte Version: `LedgerError::UnsupportedVersion`.
    /// - Gebrochene Hash-Kette oder falscher MAC: `LedgerError::BrokenChain` / `LedgerError::Corrupt`.
    pub fn open(
        storage: Box<dyn LedgerStorage>,
        ledger_key: [u8; 32],
        create_if_missing: bool,
    ) -> Result<Self, LedgerError> {
        let exists = storage.exists();
        if !exists {
            if !create_if_missing {
                return Err(LedgerError::NotFound);
            }
            // Neues Ledger initialisieren
            let header = LedgerFileHeader::new();
            let header_bytes = header.to_bytes();
            storage.append_atomic(&header_bytes)?;
            storage.sync()?;

            return Ok(Self {
                storage,
                ledger_key,
                last_hash: GENESIS_HASH,
                max_gated_seq: 0,
                records: Vec::new(),
            });
        }

        let raw_data = storage.read_all()?;
        if raw_data.is_empty() {
            if !create_if_missing {
                return Err(LedgerError::NotFound);
            }
            let header = LedgerFileHeader::new();
            let header_bytes = header.to_bytes();
            storage.append_atomic(&header_bytes)?;
            storage.sync()?;

            return Ok(Self {
                storage,
                ledger_key,
                last_hash: GENESIS_HASH,
                max_gated_seq: 0,
                records: Vec::new(),
            });
        }

        // Header validieren
        let _header = LedgerFileHeader::parse(&raw_data)?;

        // Records parsen und Kette verifizieren
        let mut offset = LedgerFileHeader::SIZE;
        let mut records = Vec::new();
        let mut prev_hash = GENESIS_HASH;
        let mut max_seq = 0u64;

        while offset < raw_data.len() {
            let slice = raw_data.get(offset..).ok_or_else(|| {
                LedgerError::Corrupt("Out of bounds while parsing records".to_string())
            })?;
            let (rec, consumed) = FramedRecord::parse(slice)?;

            // Hash-Kette prüfen
            let expected_chain = compute_chain_hash(&prev_hash, &rec.commitment, rec.seq);
            if expected_chain != rec.chain_hash {
                return Err(LedgerError::BrokenChain {
                    seq: rec.seq,
                    expected: Box::new(expected_chain),
                    actual: Box::new(rec.chain_hash),
                });
            }

            // MAC prüfen
            let expected_mac = compute_ledger_mac(&ledger_key, &rec.chain_hash);
            if expected_mac != rec.ledger_mac {
                return Err(LedgerError::Corrupt(format!(
                    "Ledger MAC mismatch at sequence {}",
                    rec.seq
                )));
            }

            prev_hash = rec.chain_hash;
            if rec.seq > max_seq {
                max_seq = rec.seq;
            }
            records.push(rec);
            offset = match offset.checked_add(consumed) {
                Some(next) => next,
                None => {
                    return Err(LedgerError::Corrupt(
                        "Offset overflow while advancing record iterator".to_string(),
                    ))
                }
            };
        }

        Ok(Self {
            storage,
            ledger_key,
            last_hash: prev_hash,
            max_gated_seq: max_seq,
            records,
        })
    }

    /// Führt einen atomaren Batch-Append von Einträgen durch (INV-ERA-4).
    ///
    /// Synchronisiert vor Rückgabe (`sync`) und liefert einen `LedgerAnchorProof`.
    /// Bei Fehlern vor dem Abschluss wird kein Zustand verändert (Fail-Closed & Crash-Safety).
    pub fn append_batch(
        &mut self,
        items: &[BatchRecordItem],
    ) -> Result<LedgerAnchorProof, LedgerError> {
        if items.is_empty() {
            return Err(LedgerError::EmptyBatch);
        }

        let initial_storage_len = self.storage.len()?;
        let mut current_hash = self.last_hash;
        let mut new_records = Vec::with_capacity(items.len());
        let mut batch_bytes = Vec::new();
        let mut new_max_seq = self.max_gated_seq;

        for item in items {
            let chain_hash = compute_chain_hash(&current_hash, &item.commitment, item.gated_seq);
            let ledger_mac = compute_ledger_mac(&self.ledger_key, &chain_hash);

            let rec = FramedRecord {
                record_type: item.record_type,
                seq: item.gated_seq,
                timestamp: item.completed_at,
                commitment: item.commitment,
                range_tags: item.range_tags.clone(),
                prev_hash: current_hash,
                chain_hash,
                ledger_mac,
            };

            batch_bytes.extend_from_slice(&rec.serialize());
            current_hash = chain_hash;
            if item.gated_seq > new_max_seq {
                new_max_seq = item.gated_seq;
            }
            new_records.push(rec);
        }

        // Atomares Schreiben in den Speicher
        if let Err(e) = self.storage.append_atomic(&batch_bytes) {
            let _ = self.storage.truncate(initial_storage_len);
            return Err(e);
        }

        // Durables Fsync vor Bestätigung
        if let Err(e) = self.storage.sync() {
            let _ = self.storage.truncate(initial_storage_len);
            return Err(e);
        }

        // Erfolgreich: State im Speicher aktualisieren
        self.last_hash = current_hash;
        self.max_gated_seq = new_max_seq;
        let count = new_records.len();
        self.records.extend(new_records);

        Ok(LedgerAnchorProof {
            last_seq: self.max_gated_seq,
            chain_hash: self.last_hash,
            count,
        })
    }

    /// Verifiziert die gesamte Hash-Kette aller Records im Ledger.
    pub fn verify_chain(&self) -> Result<(), LedgerError> {
        let mut prev = GENESIS_HASH;
        for rec in &self.records {
            let expected_chain = compute_chain_hash(&prev, &rec.commitment, rec.seq);
            if expected_chain != rec.chain_hash {
                return Err(LedgerError::BrokenChain {
                    seq: rec.seq,
                    expected: Box::new(expected_chain),
                    actual: Box::new(rec.chain_hash),
                });
            }
            let expected_mac = compute_ledger_mac(&self.ledger_key, &rec.chain_hash);
            if expected_mac != rec.ledger_mac {
                return Err(LedgerError::Corrupt(format!(
                    "Ledger MAC mismatch at sequence {}",
                    rec.seq
                )));
            }
            prev = rec.chain_hash;
        }
        Ok(())
    }

    /// Liefert den aktuellen Ketten-Hash.
    pub fn last_hash(&self) -> &[u8; 32] {
        &self.last_hash
    }

    /// Liefert den ledger_floor: `max(gated_seq)` (ER-30).
    pub fn ledger_floor(&self) -> u64 {
        self.max_gated_seq
    }

    /// Prüft auf Zeitregression: `store_current_seq < max(gated_seq)` (ER-30).
    pub fn detect_time_regression(&self, store_current_seq: u64) -> bool {
        store_current_seq < self.max_gated_seq
    }

    /// Alle geladenen Records.
    pub fn records(&self) -> &[FramedRecord] {
        &self.records
    }
}

/// Hilfsfunktion für Ketten-Hash (BLAKE3).
fn compute_chain_hash(prev_hash: &[u8; 32], commitment: &[u8; 32], gated_seq: u64) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(prev_hash);
    hasher.update(commitment);
    hasher.update(&gated_seq.to_le_bytes());
    *hasher.finalize().as_bytes()
}

/// Hilfsfunktion für Keyed-MAC (BLAKE3).
fn compute_ledger_mac(ledger_key: &[u8; 32], chain_hash: &[u8; 32]) -> [u8; 16] {
    let hash = blake3::keyed_hash(ledger_key, chain_hash);
    let mut out = [0u8; 16];
    out.copy_from_slice(&hash.as_bytes()[..16]);
    out
}
