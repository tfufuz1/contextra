//! Dateibasierte LedgerSink-Implementierung außerhalb des Datenverzeichnisses (SPEC §11.8, ER-30).
//!
//! Das Ledger liegt außerhalb des Datenverzeichnisses (D-74).
//! Lebensdauer = `backup_retention`. Einträge jenseits davon werden entfernt.

use super::durable::{BatchRecordItem, ErasureLedger, LedgerAnchorProof};
use super::entry::{LedgerEntry, GENESIS_HASH};
use super::error::LedgerError;
use super::record::RecordType;
use super::storage::LedgerStorage;
use crate::error::ErasureError;
use std::fmt;

/// In-Memory-Ledger-Implementierung (für Tests und WP-4.3 als Basis).
///
/// Ausgebaut um optionales persistentes Backend (`ErasureLedger`),
/// behält aber alle bestehenden öffentlichen Signaturen exakt bei (Entscheidung E3).
pub struct InMemoryLedger {
    entries: Vec<LedgerEntry>,
    ledger_key: [u8; 32],
    last_hash: [u8; 32],
    /// Höchste gesehene gated_seq.
    max_gated_seq: u64,
    /// Optionales zugrundeliegendes ErasureLedger für Durability.
    durable: Option<ErasureLedger>,
}

impl fmt::Debug for InMemoryLedger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemoryLedger")
            .field("last_hash", &self.last_hash)
            .field("max_gated_seq", &self.max_gated_seq)
            .field("entries_count", &self.entries.len())
            .field("durable", &self.durable)
            .finish()
    }
}

impl InMemoryLedger {
    /// Erstellt ein leeres Ledger mit dem gegebenen MAC-Schlüssel.
    pub fn new(ledger_key: [u8; 32]) -> Self {
        Self {
            entries: Vec::new(),
            ledger_key,
            last_hash: GENESIS_HASH,
            max_gated_seq: 0,
            durable: None,
        }
    }

    /// Erstellt ein Ledger mit persistentem `LedgerStorage`-Backend (WP-8).
    pub fn with_storage(
        storage: Box<dyn LedgerStorage>,
        ledger_key: [u8; 32],
        create_if_missing: bool,
    ) -> Result<Self, LedgerError> {
        let durable = ErasureLedger::open(storage, ledger_key, create_if_missing)?;
        let mut entries = Vec::new();
        for r in durable.records() {
            entries.push(LedgerEntry {
                commitment: r.commitment,
                range_tags: r.range_tags.clone(),
                gated_seq: r.seq,
                completed_at: r.timestamp,
                chain_hash: r.chain_hash,
                ledger_mac: r.ledger_mac,
            });
        }
        let last_hash = *durable.last_hash();
        let max_gated_seq = durable.ledger_floor();

        Ok(Self {
            entries,
            ledger_key,
            last_hash,
            max_gated_seq,
            durable: Some(durable),
        })
    }

    /// Fügt einen Eintrag hinzu (ER-30).
    pub fn append(
        &mut self,
        commitment: [u8; 32],
        range_tags: Vec<u8>,
        gated_seq: u64,
        completed_at: u64,
    ) -> Result<(), ErasureError> {
        if let Some(ref mut d) = self.durable {
            let item = BatchRecordItem {
                record_type: RecordType::Completed,
                commitment,
                range_tags: range_tags.clone(),
                gated_seq,
                completed_at,
            };
            d.append_batch(&[item])
                .map_err(|e| ErasureError::Storage(e.to_string()))?;
        }

        let entry = LedgerEntry::new(
            commitment,
            range_tags,
            gated_seq,
            completed_at,
            &self.last_hash,
            &self.ledger_key,
        );
        self.last_hash = entry.chain_hash;
        if gated_seq > self.max_gated_seq {
            self.max_gated_seq = gated_seq;
        }
        self.entries.push(entry);
        Ok(())
    }

    /// Führt einen atomaren Batch-Append mit Verankerungsnachweis durch (WP-8, INV-ERA-4).
    pub fn append_batch(
        &mut self,
        items: &[BatchRecordItem],
    ) -> Result<LedgerAnchorProof, LedgerError> {
        if items.is_empty() {
            return Err(LedgerError::EmptyBatch);
        }

        if let Some(ref mut d) = self.durable {
            let proof = d.append_batch(items)?;
            for item in items {
                let entry = LedgerEntry::new(
                    item.commitment,
                    item.range_tags.clone(),
                    item.gated_seq,
                    item.completed_at,
                    &self.last_hash,
                    &self.ledger_key,
                );
                self.last_hash = entry.chain_hash;
                if item.gated_seq > self.max_gated_seq {
                    self.max_gated_seq = item.gated_seq;
                }
                self.entries.push(entry);
            }
            return Ok(proof);
        }

        // Reine In-Memory-Verarbeitung
        let mut cur_hash = self.last_hash;
        let mut cur_max = self.max_gated_seq;
        let mut new_entries = Vec::with_capacity(items.len());

        for item in items {
            let entry = LedgerEntry::new(
                item.commitment,
                item.range_tags.clone(),
                item.gated_seq,
                item.completed_at,
                &cur_hash,
                &self.ledger_key,
            );
            cur_hash = entry.chain_hash;
            if item.gated_seq > cur_max {
                cur_max = item.gated_seq;
            }
            new_entries.push(entry);
        }

        self.last_hash = cur_hash;
        self.max_gated_seq = cur_max;
        let count = new_entries.len();
        self.entries.extend(new_entries);

        Ok(LedgerAnchorProof {
            last_seq: self.max_gated_seq,
            chain_hash: self.last_hash,
            count,
        })
    }

    /// Prüft alle Einträge auf Integrität (Ketten-Verifikation).
    pub fn verify_chain(&self) -> bool {
        if let Some(ref d) = self.durable {
            if d.verify_chain().is_err() {
                return false;
            }
        }
        let mut prev = GENESIS_HASH;
        for entry in &self.entries {
            if !entry.verify(&prev, &self.ledger_key) {
                return false;
            }
            prev = entry.chain_hash;
        }
        true
    }

    /// Prüft auf Zeitregression: `store_seq < max(gated_seq)` (ER-30).
    pub fn detect_time_regression(&self, store_current_seq: u64) -> bool {
        store_current_seq < self.max_gated_seq
    }

    /// Gibt den ledger_floor zurück: `max(gated_seq)` (ER-30).
    pub fn ledger_floor(&self) -> u64 {
        self.max_gated_seq
    }

    /// Alle Einträge.
    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }

    /// Entfernt Einträge jenseits von `backup_retention` (ER-30).
    ///
    /// `cutoff_us`: Mikrosekunden UTC, Einträge mit `completed_at < cutoff_us` werden entfernt.
    pub fn prune(&mut self, cutoff_us: u64) {
        self.entries
            .retain(|e| e.completed_at >= cutoff_us || e.completed_at == 0);
    }

    /// Gibt die höchste gesehene gate_seq zurück.
    pub fn max_gated_seq(&self) -> u64 {
        self.max_gated_seq
    }

    /// Gibt den aktuellen Ketten-Hash zurück.
    pub fn last_hash(&self) -> &[u8; 32] {
        &self.last_hash
    }
}
