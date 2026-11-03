//! Ledger-Eintragsformat und kryptografische Verkettung (SPEC §11.8, Anhang F.9, ER-30).
//!
//! Eintrag: `(commitment, range_set, gated_seq, completed_at)`, Hash-Kette + `ledger_mac`.
//! Lebensdauer = `backup_retention`.

use serde::{Deserialize, Serialize};

/// Ein Ledger-Eintrag (SPEC §11.8, ER-30).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Commitment (BLAKE3-keyed über Ziel + Mandant).
    pub commitment: [u8; 32],
    /// Bereich-Tags (opak, kein Klartext).
    pub range_tags: Vec<u8>,
    /// Gate-Sequenznummer.
    pub gated_seq: u64,
    /// Abschlusszeitstempel (Mikrosekunden UTC, 0 = noch offen).
    pub completed_at: u64,
    /// Hash-Kette: BLAKE3(prev_hash ‖ commitment ‖ gated_seq_le).
    pub chain_hash: [u8; 32],
    /// Ledger-MAC: BLAKE3-keyed(K_ledger, chain_hash).
    pub ledger_mac: [u8; 16],
}

impl LedgerEntry {
    /// Erstellt einen neuen Ledger-Eintrag mit Hash-Kette.
    pub fn new(
        commitment: [u8; 32],
        range_tags: Vec<u8>,
        gated_seq: u64,
        completed_at: u64,
        prev_hash: &[u8; 32],
        ledger_key: &[u8; 32],
    ) -> Self {
        let chain_hash = compute_chain_hash(prev_hash, &commitment, gated_seq);
        let ledger_mac = compute_ledger_mac(ledger_key, &chain_hash);
        Self {
            commitment,
            range_tags,
            gated_seq,
            completed_at,
            chain_hash,
            ledger_mac,
        }
    }

    /// Prüft die Integrität dieses Eintrags (MAC-Verifikation).
    pub fn verify(&self, prev_hash: &[u8; 32], ledger_key: &[u8; 32]) -> bool {
        let expected_chain = compute_chain_hash(prev_hash, &self.commitment, self.gated_seq);
        if expected_chain != self.chain_hash {
            return false;
        }
        let expected_mac = compute_ledger_mac(ledger_key, &self.chain_hash);
        expected_mac == self.ledger_mac
    }
}

/// Berechnet den Ketten-Hash: BLAKE3(prev_hash ‖ commitment ‖ gated_seq_le).
fn compute_chain_hash(prev_hash: &[u8; 32], commitment: &[u8; 32], gated_seq: u64) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(prev_hash);
    hasher.update(commitment);
    hasher.update(&gated_seq.to_le_bytes());
    *hasher.finalize().as_bytes()
}

/// Berechnet den Ledger-MAC: BLAKE3-keyed(K_ledger, chain_hash)[0..16].
fn compute_ledger_mac(ledger_key: &[u8; 32], chain_hash: &[u8; 32]) -> [u8; 16] {
    let hash = blake3::keyed_hash(ledger_key, chain_hash);
    let mut out = [0u8; 16];
    out.copy_from_slice(&hash.as_bytes()[..16]);
    out
}

/// Genesis-Hash für die erste Kette (Nullvektor).
pub const GENESIS_HASH: [u8; 32] = [0u8; 32];
