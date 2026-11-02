//! GateLog nach SPEC F.9, ER-12: kind = 7, Appendlog mit Hash-Kette.

use contextra_ports::gate::{KeyBuf, Seq};
use serde::{Deserialize, Serialize};

/// Art des Eintrags im Gate-Log.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GateLogEntry {
    /// Hinzufügen eines Tor-Intervalls
    Add {
        /// Eindeutige Gate-ID
        gate_id: [u8; 16],
        /// Variante: 0 = Erased, 1 = Restricted
        variant: u8,
        /// Untere Grenze
        start: KeyBuf,
        /// Obere Grenze
        end: KeyBuf,
        /// Gated Sequence
        gated_seq: Seq,
        /// Löschungs-ID
        erasure_id: [u8; 16],
    },
    /// Entfernen eines Tors
    Remove {
        /// Eindeutige Gate-ID
        gate_id: [u8; 16],
    },
}

/// In-Memory GateLog-Abstraktion mit Hash-Kette.
#[derive(Debug, Default)]
pub struct GateLog {
    entries: Vec<GateLogEntry>,
    chain_hash: [u8; 32],
}

impl GateLog {
    /// Erstellt ein neues Gate-Log.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            chain_hash: [0u8; 32],
        }
    }

    /// Fügt einen Eintrag an und aktualisiert die Hash-Kette.
    pub fn append(&mut self, entry: GateLogEntry) {
        let serialized = serde_json::to_vec(&entry).unwrap_or_default();
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.chain_hash);
        hasher.update(&serialized);
        self.chain_hash = *hasher.finalize().as_bytes();
        self.entries.push(entry);
    }

    /// Gibt alle Einträge zurück (z. B. für Replay beim Start, ER-14).
    pub fn entries(&self) -> &[GateLogEntry] {
        &self.entries
    }
}
