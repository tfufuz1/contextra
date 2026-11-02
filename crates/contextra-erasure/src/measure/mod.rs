//! Messung und Typestate-Scanner (SPEC §12, D-72, I-19, ER-25, ER-26).
//!
//! Ablauf: `Scanner<Unarmed>` → Positivkontrolle → `ControlEvidence::observed()` →
//! `arm` → `Scanner<Armed>` → `record(files, bytes)` → `finish(matches, duration, hash)` → `LayerWitness`.
//!
//! `arm` und `ControlEvidence::observed` dürfen nur hier aufgerufen werden (L-5, ER-25).

mod positive_control;
mod witness;
pub mod field;
pub mod scanner_flow;

pub use field::{Mk1Result, Mk2Result, Mk3Result};
pub use scanner_flow::run_scanner_flow;

// Re-Export der Typen aus `types` für Aufrufer innerhalb dieses Moduls
pub use contextra_types::erasure::{
    Armed, ControlEvidence, LayerWitness, MeasureClass, Scanner, Unarmed,
};

/// Koordinator für Feldmessungen (MK1, MK2, MK3, ER-26).
pub struct MeasurementCoordinator {
    _private: (),
}

impl MeasurementCoordinator {
    /// Erstellt einen neuen Koordinator.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for MeasurementCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
