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
///   1. Positivkontrolle
///   2. Scanner armen
///   3. Scan durchführen (hier: Stub mit `matches = 0`)
///   4. Zeugen erzeugen
///
/// Gibt `ErasureError::Residue(n)` zurück wenn `matches > 0`.
pub fn run_scanner_flow(
    layer: LayerId,
    class: MeasureClass,
    method_id: u16,
    control_pattern: &[u8],
) -> Result<LayerWitness, ErasureError> {
    // Schritt 1: Positivkontrolle
    let evidence = run_positive_control(control_pattern).ok_or(ErasureError::Residue(0))?;

    // Schritt 2: Scanner erstellen und armen
    let unarmed = Scanner::<Unarmed>::new(layer, class, method_id);
    let mut armed = arm_scanner(unarmed, evidence);

    // Schritt 3: Scan (Stub: keine echten Dateien, 0 Treffer)
    armed.record(0, 0);

    // Schritt 4: Zeugen erzeugen
    let scan_hash = *blake3::hash(control_pattern).as_bytes();
    finish_scan(armed, 0, Duration::from_millis(0), scan_hash)
        .map_err(|r| ErasureError::Residue(r.matches))
}
