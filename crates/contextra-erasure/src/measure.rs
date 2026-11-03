//! Messung und Typestate-Scanner (SPEC §12, D-72, I-19, ER-25, ER-26).
//!
//! Ablauf: `Scanner<Unarmed>` → Positivkontrolle → `ControlEvidence::observed()` →
//! `arm` → `Scanner<Armed>` → `record(files, bytes)` → `finish(matches, duration, hash)` → `LayerWitness`.
//!
//! `arm` und `ControlEvidence::observed` dürfen nur hier aufgerufen werden (L-5, ER-25).

pub mod positive_control;
pub mod witness;
pub mod field;
pub mod scanner_flow;

pub use field::{Mk1Result, Mk2Result, Mk3Result};
pub use scanner_flow::run_scanner_flow;
pub use positive_control::run_positive_control;
pub use witness::{arm_scanner, finish_scan};

// Re-Export der Typen aus `types` für Aufrufer innerhalb dieses Moduls
pub use contextra_types::erasure::{
    Armed, ControlEvidence, LayerWitness, MeasureClass, Scanner, Unarmed,
};

/// Art des Positivkontroll-Nachweises (ER-25).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceKind {
    /// Muster wurde erfolgreich in der Kontrolldatei nachgewiesen
    Observed,
}

/// Scanner im Zustand `Unarmed` (noch nicht scharfgeschaltet) für Abwärtskompatibilität.
#[derive(Debug, Default)]
pub struct UnarmedScanner;

impl UnarmedScanner {
    /// Erstellt einen neuen unbewaffneten Scanner.
    pub fn new() -> Self {
        Self
    }

    /// Führt die Positivkontrolle auf einem bekannten Muster aus (ER-25).
    pub fn run_positive_control(&self, pattern: &[u8]) -> Result<ControlEvidenceWrapper, crate::error::ErasureError> {
        if let Some(ev) = positive_control::run_positive_control(pattern) {
            Ok(ControlEvidenceWrapper {
                inner: ev,
                kind: EvidenceKind::Observed,
            })
        } else {
            Err(crate::error::ErasureError::Internal(
                "Positivkontrolle fehlgeschlagen: Muster nicht gefunden".into(),
            ))
        }
    }

    /// Schaltet den Scanner mit dem Positivkontrollnachweis scharf (`arm`, L-5, ER-25).
    pub fn arm(self, evidence: ControlEvidenceWrapper) -> ArmedScanner {
        ArmedScanner {
            _evidence: evidence.inner,
        }
    }
}

/// Wrapper für ControlEvidence
#[derive(Debug)]
pub struct ControlEvidenceWrapper {
    inner: ControlEvidence,
    kind: EvidenceKind,
}

impl ControlEvidenceWrapper {
    /// Gibt die Art des Nachweises zurück.
    pub fn kind(&self) -> EvidenceKind {
        self.kind
    }
}

/// Scanner im Zustand `Armed` für Abwärtskompatibilität.
#[derive(Debug)]
pub struct ArmedScanner {
    _evidence: ControlEvidence,
}

impl ArmedScanner {
    /// Schließt den Scan ab. Gibt `LayerWitness` NUR zurück, wenn `matches == 0` (ER-25).
    pub fn finish(&self, matches: u64) -> Result<contextra_ports::erasure::LayerWitness, crate::error::ErasureError> {
        if matches == 0 {
            let hash = blake3::hash(b"ctx/witness/verified-zero-matches");
            Ok(contextra_ports::erasure::LayerWitness::from_hash(*hash.as_bytes()))
        } else {
            Err(crate::error::ErasureError::Internal(format!(
                "Residuen gefunden: {} Treffer (Löschung unvollständig, Zeuge verweigert)",
                matches
            )))
        }
    }
}

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
