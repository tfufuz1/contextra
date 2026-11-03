//! Typisierter Scanner.

use crate::error::LabError;

/// Messklasse laut SPEC §12.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementClass {
    /// MK1: Strukturscan
    Mk1,
    /// MK2: Rohscan
    Mk2,
    /// MK3: Schlüsselprobe
    Mk3,
}

/// Nachweis einer erfolgreichen Positivkontrolle.
#[derive(Debug, Clone)]
pub struct ControlEvidence {
    /// Das geprüfte Kontrollmuster.
    pub needle: Vec<u8>,
    /// Anzahl gefundener Treffer.
    pub matches: usize,
}

impl ControlEvidence {
    /// Erzeugt ein Kontrollbeweisobjekt, wenn mindestens ein Treffer auf das Testmuster vorliegt.
    pub fn try_new(needle: &[u8], matches: usize) -> Result<Self, LabError> {
        if matches == 0 {
            return Err(LabError::PositiveControlFailed);
        }
        Ok(Self {
            needle: needle.to_vec(),
            matches,
        })
    }
}

/// Zustand Unarmed.
pub struct Unarmed;

/// Zustand Armed.
pub struct Armed {
    evidence: ControlEvidence,
}

/// Unabhängiger Scanner mit Typzustand (SPEC §12.2).
pub struct Scanner<State> {
    layer: &'static str,
    class: MeasurementClass,
    scanned_files: u64,
    scanned_bytes: u64,
    state: State,
}

/// Ergebnis der Messung (Zeuge, SPEC §12.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerWitness {
    /// Betroffene Schicht.
    pub layer: &'static str,
    /// Messklasse.
    pub class: MeasurementClass,
    /// Anzahl geprüfter Dateien.
    pub scanned_files: u64,
    /// Anzahl geprüfter Bytes.
    pub scanned_bytes: u64,
    /// Anzahl gefundener Zieltreffer (muss 0 sein für gültigen Zeugen).
    pub matches: usize,
    /// Positivkontrolle erfolgreich bestanden.
    pub control_ok: bool,
}

impl Scanner<Unarmed> {
    /// Erzeugt einen neuen unbewaffneten Scanner.
    pub fn new(layer: &'static str, class: MeasurementClass) -> Self {
        Self {
            layer,
            class,
            scanned_files: 0,
            scanned_bytes: 0,
            state: Unarmed,
        }
    }

    /// Schärft den Scanner nach erbrachter Positivkontrolle.
    #[must_use]
    pub fn arm(self, evidence: ControlEvidence) -> Scanner<Armed> {
        Scanner {
            layer: self.layer,
            class: self.class,
            scanned_files: self.scanned_files,
            scanned_bytes: self.scanned_bytes,
            state: Armed { evidence },
        }
    }
}

impl Scanner<Armed> {
    /// Registriert gescannte Dateien und Bytes.
    pub fn record_progress(&mut self, files: u64, bytes: u64) {
        self.scanned_files = self.scanned_files.saturating_add(files);
        self.scanned_bytes = self.scanned_bytes.saturating_add(bytes);
    }

    /// Schließt die Messung ab. Erzeugt `LayerWitness` nur wenn `matches == 0`.
    pub fn finish(self, matches: usize) -> Result<LayerWitness, LabError> {
        if matches > 0 {
            return Err(LabError::VerificationFailed(
                "Zielreste gefunden: matches > 0",
            ));
        }
        Ok(LayerWitness {
            layer: self.layer,
            class: self.class,
            scanned_files: self.scanned_files,
            scanned_bytes: self.scanned_bytes,
            matches,
            control_ok: self.state.evidence.matches > 0,
        })
    }
}
