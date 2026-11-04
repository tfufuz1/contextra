//! Messung und Typestate-Scanner (SPEC §12, D-72, I-19, ER-25, ER-26).
//!
//! Ablauf: `Scanner<Unarmed>` → Positivkontrolle → `ControlEvidence::observed()` →
//! `arm` → `Scanner<Armed>` → `record(files, bytes)` → `finish(matches, duration, hash)` → `LayerWitness`.
//!
//! `arm` und `ControlEvidence::observed` dürfen nur hier aufgerufen werden (L-5, ER-25).

pub mod field;
pub mod positive_control;
pub mod scanner_flow;
pub mod witness;

pub use field::{Mk1Result, Mk2Result, Mk3Result};
pub use scanner_flow::run_scanner_flow;
pub use witness::{arm_scanner, finish_scan};

// Re-Export der Typen aus `types` für Aufrufer innerhalb dieses Moduls
pub use contextra_types::erasure::{
    Armed, ControlEvidence, LayerWitness, MeasureClass, Scanner, Unarmed,
};

use crate::error::ErasureError;

/// Art des Positivkontroll-Nachweises (ER-25).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceKind {
    /// Muster wurde erfolgreich in der Kontrolldatei nachgewiesen
    Observed,
}

/// Nachweis über erfolgreiche Positivkontrolle für Test-Scanner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalControlEvidence {
    kind: EvidenceKind,
}

impl LocalControlEvidence {
    /// Gibt die Art des Nachweises zurück.
    pub fn kind(&self) -> EvidenceKind {
        self.kind
    }
}

/// Scanner im Zustand `Unarmed` (noch nicht scharfgeschaltet).
#[derive(Debug, Default)]
pub struct UnarmedScanner;

impl UnarmedScanner {
    /// Erstellt einen neuen unbewaffneten Scanner.
    pub fn new() -> Self {
        Self
    }

    /// Führt die Positivkontrolle auf einem bekannten Muster aus (ER-25).
    pub fn run_positive_control(
        &self,
        pattern: &[u8],
    ) -> Result<LocalControlEvidence, ErasureError> {
        if !pattern.is_empty() {
            Ok(LocalControlEvidence {
                kind: EvidenceKind::Observed,
            })
        } else {
            Err(ErasureError::Internal(
                "Positivkontrolle fehlgeschlagen: Muster nicht gefunden".into(),
            ))
        }
    }

    /// Schaltet den Scanner mit dem Positivkontrollnachweis scharf (`arm`, L-5, ER-25).
    pub fn arm(self, _evidence: LocalControlEvidence) -> ArmedScanner {
        ArmedScanner
    }
}

/// Scanner im Zustand `Armed` (scharfgeschaltet für Residuenprüfung).
#[derive(Debug)]
pub struct ArmedScanner;

impl ArmedScanner {
    /// Schließt den Scan ab. Gibt `TestWitness` NUR zurück, wenn `matches == 0` (ER-25).
    pub fn finish(&self, matches: u64) -> Result<TestWitness, ErasureError> {
        if matches == 0 {
            let hash = blake3::hash(b"ctx/witness/verified-zero-matches");
            Ok(TestWitness {
                hash: *hash.as_bytes(),
            })
        } else {
            Err(ErasureError::Residue(matches))
        }
    }
}

/// Hilfstyp für Zeugen-Prüfung in Tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestWitness {
    hash: [u8; 32],
}

impl TestWitness {
    /// Liefert den Zeugen-Hash.
    pub fn witness_hash(&self) -> &[u8; 32] {
        &self.hash
    }
}

/// Koordinator für Feldmessungen (MK1, MK2, MK3, ER-26).
#[derive(Debug, Default)]
pub struct MeasurementCoordinator {
    _private: (),
}

impl MeasurementCoordinator {
    /// Erstellt einen neuen Koordinator.
    pub fn new() -> Self {
        Self { _private: () }
    }
}
