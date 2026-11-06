//! Dateibasierte LedgerSink-Implementierung außerhalb des Datenverzeichnisses (SPEC §11.8, ER-30).
//!
//! Das Ledger liegt außerhalb des Datenverzeichnisses (D-74).
//! Lebensdauer = `backup_retention`. Einträge jenseits davon werden entfernt.

use super::entry::{LedgerEntry, GENESIS_HASH};
use crate::error::ErasureError;

/// In-Memory-Ledger-Implementierung (für Tests und WP-4.3 als Basis).
///
/// Produktiv: Datei außerhalb `data_dir`, AEAD-Appendlog (F.9).
pub struct InMemoryLedger {
    entries: Vec<LedgerEntry>,
    ledger_key: [u8; 32],
    last_hash: [u8; 32],
    /// Höchste gesehene gated_seq.
    max_gated_seq: u64,
}

impl InMemoryLedger {
    /// Erstellt ein leeres Ledger mit dem gegebenen MAC-Schlüssel.
    pub fn new(ledger_key: [u8; 32]) -> Self {
        Self {
            entries: Vec::new(),
            ledger_key,
            last_hash: GENESIS_HASH,
            max_gated_seq: 0,
        }
    }

    /// Fügt einen Eintrag hinzu (ER-30).
    pub fn append(
        &mut self,
        commitment: [u8; 32],
        range_tags: Vec<u8>,
        gated_seq: u64,
        completed_at: u64,
    ) -> Result<(), ErasureError> {
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

    /// Prüft alle Einträge auf Integrität (Ketten-Verifikation).
    pub fn verify_chain(&self) -> bool {
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
}
