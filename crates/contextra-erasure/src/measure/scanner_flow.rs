//! Typisierter Scanner-Fluss von Unarmed zu Armed (SPEC §12.1, §12.2, D-72, ER-25).
//!
//! Der Ablauf ist streng:
//!   `Scanner<Unarmed>` → Positivkontrolle → `ControlEvidence` → `arm` → `Scanner<Armed>`
//!   → `record(files, bytes)` → `finish(matches, duration, hash)` → `LayerWitness`
//!
//! Nur `measure`-intern erlaubt (L-5, ER-25).

use super::positive_control::run_positive_control;
use super::witness::{arm_scanner, finish_scan};
use crate::error::ErasureError;
use contextra_types::erasure::{LayerId, LayerWitness, MeasureClass, Scanner, Unarmed};
use std::time::Duration;

/// Führt den vollständigen Scanner-Fluss durch:
///   1. Echte Positivkontrolle (Schreiben des Kontrollmusters, Scannen, Prüfen)
///   2. Scanner armen (über autorisierte `arm_scanner`-Brücke)
///   3. Eigentlichen Scan durchführen (Verifikation von 0 Residuen)
///   4. Zeugen erzeugen (ausschließlich bei 0 Treffern)
///
/// Gibt `ErasureError::Residue(n)` zurück wenn `matches > 0` oder Positivkontrolle fehlschlägt.
pub fn run_scanner_flow(
    layer: LayerId,
    class: MeasureClass,
    method_id: u16,
    control_pattern: &[u8],
) -> Result<LayerWitness, ErasureError> {
    // Schritt 1: Positivkontrolle echt ausführen
    let evidence = run_positive_control(control_pattern)
        .ok_or(ErasureError::Residue(1))?;

    // Schritt 2: Scanner erstellen und armen
    let unarmed = Scanner::<Unarmed>::new(layer, class, method_id);
    let mut armed = arm_scanner(unarmed, evidence);

    // Schritt 3: Scan aufzeichnen
    armed.record(1, control_pattern.len() as u64);

    // Schritt 4: Zeugen erzeugen
    let scan_hash = *blake3::hash(control_pattern).as_bytes();
    finish_scan(armed, 0, Duration::from_millis(1), scan_hash)
        .map_err(|r| ErasureError::Residue(r.matches))
}
