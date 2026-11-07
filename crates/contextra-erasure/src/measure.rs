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

pub use field::{
    collect_all_files, read_file_verified, scan_mk1_structures, scan_mk2_raw, scan_mk3_keys,
    Mk1Result, Mk2Result, Mk3Result, ScanPlan, StructureDecoder, VerifiedVia,
};
pub use scanner_flow::{run_scanner_flow, scan_with_control};
pub use witness::{arm_scanner, finish_scan};

// Re-Export der Typen aus `types` für Aufrufer innerhalb dieses Moduls
pub use crate::types_local::{
    Armed, ControlEvidence, LayerWitness, MeasureClass, Scanner, Unarmed,
};

use crate::error::ErasureError;

/// Art des Positivkontroll-Nachweises (ER-25).
#[deprecated(note = "Verwende crate::types_local::ControlEvidence über die Positivkontrolle")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceKind {
    /// Muster wurde erfolgreich in der Kontrolldatei nachgewiesen
    Observed,
}

/// Nachweis über erfolgreiche Positivkontrolle für Test-Scanner.
#[deprecated(note = "Verwende crate::types_local::ControlEvidence über die Positivkontrolle")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalControlEvidence {
    #[allow(deprecated)]
    kind: EvidenceKind,
}

#[allow(deprecated)]
impl LocalControlEvidence {
    /// Gibt die Art des Nachweises zurück.
    pub fn kind(&self) -> EvidenceKind {
        self.kind
    }
}

/// Scanner im Zustand `Unarmed` (noch nicht scharfgeschaltet).
#[deprecated(note = "Verwende crate::types_local::Scanner<Unarmed>")]
#[derive(Debug, Default)]
pub struct UnarmedScanner;

#[allow(deprecated)]
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
        if positive_control::run_positive_control(pattern).is_some() {
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
#[deprecated(note = "Verwende crate::types_local::Scanner<Armed>")]
#[derive(Debug)]
pub struct ArmedScanner;

#[allow(deprecated)]
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
#[deprecated(note = "Verwende crate::types_local::LayerWitness")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestWitness {
    hash: [u8; 32],
}

#[allow(deprecated)]
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
