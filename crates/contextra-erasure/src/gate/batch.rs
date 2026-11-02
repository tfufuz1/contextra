//! Gate-Gruppen-Commit für kooperative Barrieren-Bündelung (SPEC §11.3, D-57, D-78, ER-13).
//!
//! `add_batch` bündelt gleichzeitige Anfragen in einer einzigen exklusiven
//! Barrieren-Phase mit einem gemeinsamen fsync.

use crate::error::ErasureError;
use contextra_ports::gate::{KeyBuf, Seq};

/// Anfrage für einen Gruppen-Commit.
pub struct BatchGateRequest {
    /// Gate-Kennung.
    pub gate_id: [u8; 16],
    /// Untere Grenze.
    pub lo: KeyBuf,
    /// Obere Grenze.
    pub hi: KeyBuf,
    /// Gate-Variante.
    pub is_erased: bool,
    /// Lösch-ID.
    pub erasure_id: [u8; 16],
}

/// Ergebnis eines Gruppen-Commits.
pub struct BatchGateOutcome {
    /// Sequenznummer, unter der die Gates gesetzt wurden.
    pub gated_seq: Seq,
}

/// Bündelt mehrere Gate-Anfragen in einer Barrieren-Phase (ER-13).
///
/// In der Produktionsimplementierung (WP-4.2) wird dies über `CommitBarrier::exclusive`
/// koordiniert.
pub fn add_batch(
    requests: Vec<BatchGateRequest>,
    current_seq: Seq,
) -> Result<BatchGateOutcome, ErasureError> {
    // Stub: Alle Anfragen teilen sich eine Sequenznummer
    let _ = requests;
    Ok(BatchGateOutcome {
        gated_seq: current_seq,
    })
}
