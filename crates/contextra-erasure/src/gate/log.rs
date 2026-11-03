//! Persistentes Append-Log aktiver Gates gates.log (SPEC §11.3, D-58, Anhang F.9, ER-12).
//!
//! Das Log enthält `Add` und `Remove`-Einträge für Gates.
//! Format: AEAD-geschützt, Hash-Kette, ein fsync je Gruppe.

use serde::{Deserialize, Serialize};

/// Mögliche Eintragsarten im Gate-Log.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GateLogEntry {
    /// Ein neues Gate hinzufügen.
    Add {
        /// Gate-Kennung.
        gate_id: [u8; 16],
        /// Gate-Variante: true = Erased, false = Restricted.
        is_erased: bool,
        /// Untere Grenze.
        lo: [u8; 16],
        /// Obere Grenze.
        hi: [u8; 16],
        /// Gate-Sequenznummer.
        gated_seq: u64,
        /// Zugehörige Lösch-ID.
        erasure_id: [u8; 16],
    },
    /// Ein Gate entfernen.
    Remove {
        /// Gate-Kennung.
        gate_id: [u8; 16],
    },
}

/// In-Memory-Append-Log für Gates (vollständige Implementierung in WP-4.2).
pub struct GateLog {
    entries: Vec<GateLogEntry>,
}

impl GateLog {
    /// Erstellt ein leeres Gate-Log.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Fügt einen Eintrag hinzu.
    pub fn append(&mut self, entry: GateLogEntry) {
        self.entries.push(entry);
    }

    /// Liefert alle Einträge.
    pub fn entries(&self) -> &[GateLogEntry] {
        &self.entries
    }

    /// Anzahl der Einträge.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Prüft ob das Log leer ist.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for GateLog {
    fn default() -> Self {
        Self::new()
    }
}
