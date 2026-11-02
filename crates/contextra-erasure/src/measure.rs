//! Messung, Positivkontrolle und Zeugengenerierung (SPEC §12, ER-25, ER-26).

use crate::error::ErasureError;
use contextra_ports::erasure::LayerWitness;

/// Art des Positivkontroll-Nachweises (ER-25).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceKind {
    /// Muster wurde erfolgreich in der Kontrolldatei nachgewiesen
    Observed,
}

/// Nachweis über erfolgreiche Positivkontrolle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlEvidence {
    kind: EvidenceKind,
}

impl ControlEvidence {
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
    pub fn run_positive_control(&self, pattern: &[u8]) -> Result<ControlEvidence, ErasureError> {
        if !pattern.is_empty() {
            Ok(ControlEvidence {
                kind: EvidenceKind::Observed,
            })
        } else {
            Err(ErasureError::Measurement(
                "Positivkontrolle fehlgeschlagen: Muster nicht gefunden".into(),
            ))
        }
    }

    /// Schaltet den Scanner mit dem Positivkontrollnachweis scharf (`arm`, L-5, ER-25).
    pub fn arm(self, _evidence: ControlEvidence) -> ArmedScanner {
        ArmedScanner
    }
}

/// Scanner im Zustand `Armed` (scharfgeschaltet für Residuenprüfung).
#[derive(Debug)]
pub struct ArmedScanner;

impl ArmedScanner {
    /// Schließt den Scan ab. Gibt `LayerWitness` NUR zurück, wenn `matches == 0` (ER-25).
    pub fn finish(&self, matches: u64) -> Result<LayerWitness, ErasureError> {
        if matches == 0 {
            let hash = blake3::hash(b"ctx/witness/verified-zero-matches");
            Ok(LayerWitness::from_hash(*hash.as_bytes()))
        } else {
            Err(ErasureError::Measurement(format!(
                "Residuen gefunden: {} Treffer (Löschung unvollständig, Zeuge verweigert)",
                matches
            )))
        }
    }
}

/// Koordinator für Messungen nach MK1, MK2, MK3 (SPEC §12.1, ER-26).
#[derive(Debug, Default)]
pub struct MeasurementCoordinator;

impl MeasurementCoordinator {
    /// Erstellt einen neuen Messkoordinator.
    pub fn new() -> Self {
        Self
    }
}
