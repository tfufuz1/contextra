//! Erzeugung und Validierung von LayerWitness-Beweisen (SPEC §12.2, ER-25).
//!
//! `LayerWitness` wird ausschließlich bei matches == 0 erzeugt (I-19, TY-10).

use crate::types_local::{Armed, ControlEvidence, LayerWitness, Scanner, Unarmed};
use std::time::Duration;

/// Erstellt einen bewaffneten Scanner nach Positivkontrolle (ER-25).
///
/// Nur aus `src/measure` aufzurufen (L-5).
pub fn arm_scanner(scanner: Scanner<Unarmed>, evidence: ControlEvidence) -> Scanner<Armed> {
    scanner.arm(evidence)
}

/// Erstellt einen Zeugen nach erfolgreichem Scan (ER-25).
///
/// `matches == 0` → Ok(LayerWitness)
/// `matches != 0` → Err(Residue)
pub fn finish_scan(
    scanner: Scanner<Armed>,
    matches: u64,
    duration: Duration,
    scan_hash: [u8; 32],
) -> Result<LayerWitness, crate::types_local::Residue> {
    scanner.finish(matches, duration, scan_hash)
}
