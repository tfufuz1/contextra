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

/// Prüft, ob ein `LayerWitness` die strengen Invarianten für die Berichterstellung erfüllt (INV-CLASS-MEASURED-1).
///
/// Ein Zeuge ist nur dann gültig, wenn:
/// 1. `matches == 0` (keine Restspuren)
/// 2. `witness_hash` nicht nur aus Nullen besteht (Scanlauf tatsächlich ausgeführt).
pub fn validate_witness(witness: &LayerWitness) -> Result<(), crate::error::ErasureError> {
    if witness.matches() != 0 {
        return Err(crate::error::ErasureError::Residue(witness.matches()));
    }
    if *witness.hash() == [0u8; 32] {
        return Err(crate::error::ErasureError::Internal(
            "INV-CLASS-MEASURED-1 violation: witness has empty zero scan hash".to_string(),
        ));
    }
    Ok(())
}

/// Berechnet einen kryptografischen BLAKE3-Digest eines `LayerWitness` (SPEC §12.2, INV-ERA-7).
///
/// Bindet Schicht-ID, Messklasse, Methoden-ID, Dateien, Bytes, Scandauer und den Scan-Hash.
/// // Herkunft: d835c682d:crates/contextra-crypto/src/deletion_proof/issuer.rs:112
pub fn compute_witness_digest(witness: &LayerWitness) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"ctx/witness/v1\n");
    hasher.update(witness.layer().as_bytes());
    hasher.update(&[witness.class() as u8]);
    hasher.update(&witness.method_id().to_le_bytes());
    hasher.update(&witness.files().to_le_bytes());
    hasher.update(&witness.bytes().to_le_bytes());
    let micros = u64::try_from(witness.duration().as_micros()).unwrap_or(u64::MAX);
    hasher.update(&micros.to_le_bytes());
    hasher.update(witness.hash());
    *hasher.finalize().as_bytes()
}
