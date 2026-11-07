//! ScanSession für den gesperrten Mint-Pfad von LayerWitness (SPEC §12.7.4, INV-WITNESS-MINT-1).
//!
//! # Invarianten
//! - `finish` verweigert `scanned == 0`, `control_matches == 0` und `NotMeasured` ohne Begründung.

#![forbid(unsafe_code)]

use crate::error::{SpiError, SpiResult};
use crate::layer::{ScanTechnique, WitnessLayer};
use crate::scope::ScanTarget;
use crate::witness::LayerWitness;

/// Laufende Scan-Session zur kontrollierten Erzeugung eines `LayerWitness`.
#[derive(Debug)]
pub struct ScanSession {
    layer: WitnessLayer,
    target: ScanTarget,
    technique: ScanTechnique,
    scanned: u64,
    remaining: u64,
    control_matches: u64,
    unit_hashes: Vec<[u8; 32]>,
}

impl ScanSession {
    /// Startet eine neue ScanSession (modul-intern oder für Tests).
    #[doc(hidden)]
    pub fn begin(layer: WitnessLayer, target: ScanTarget, technique: ScanTechnique) -> Self {
        Self {
            layer,
            target,
            technique,
            scanned: 0,
            remaining: 0,
            control_matches: 0,
            unit_hashes: Vec::new(),
        }
    }

    /// Gibt die aktuelle Schicht zurück.
    pub fn layer(&self) -> &WitnessLayer {
        &self.layer
    }

    /// Gibt das Ziel zurück.
    pub fn target(&self) -> &ScanTarget {
        &self.target
    }

    /// Gibt die Technik zurück.
    pub fn technique(&self) -> &ScanTechnique {
        &self.technique
    }

    /// Registriert eine gescannte physische Einheit.
    pub fn record_unit(&mut self, unit_hash: [u8; 32], is_live_residual: bool) {
        self.scanned = self.scanned.saturating_add(1);
        if is_live_residual {
            self.remaining = self.remaining.saturating_add(1);
        }
        self.unit_hashes.push(unit_hash);
    }

    /// Registriert eine bestandene Positivkontrolle (Control Match / Canary Probe).
    pub fn record_control_match(&mut self) {
        self.control_matches = self.control_matches.saturating_add(1);
    }

    /// Schließt die Session ab und erzeugt den `LayerWitness` (gesperrter Mint-Pfad).
    ///
    /// # Fehlerbedingungen (INV-WITNESS-MINT-1)
    /// - `scanned == 0` (außer bei `FeatureDisabled` oder `NotMeasured`)
    /// - `control_matches == 0` (Positivkontrolle nicht bestanden)
    /// - `ScanTechnique::NotMeasured(grund)` mit leerem Grund
    #[doc(hidden)]
    pub fn finish(mut self, now_unix_nanos: u64) -> SpiResult<LayerWitness> {
        match &self.technique {
            ScanTechnique::FeatureDisabled => {}
            ScanTechnique::NotMeasured(reason) => {
                if reason.trim().is_empty() {
                    return Err(SpiError::WitnessMintViolation(
                        "INV-WITNESS-MINT-1: NotMeasured technique requires non-empty reason"
                            .to_string(),
                    ));
                }
            }
            _ => {
                if self.scanned == 0 {
                    return Err(SpiError::WitnessMintViolation(
                        "INV-WITNESS-MINT-1: scanned count cannot be 0 for active scan technique"
                            .to_string(),
                    ));
                }
                if self.control_matches == 0 {
                    return Err(SpiError::ControlMatchFailed(
                        "INV-WITNESS-MINT-1: positive control check failed (control_matches == 0)"
                            .to_string(),
                    ));
                }
            }
        }

        // Sortiere Unit-Hashes für deterministischen Digest ohne Klartext
        self.unit_hashes.sort();

        let mut hasher = blake3::Hasher::new();
        hasher.update(self.layer.as_str().as_bytes());
        hasher.update(format!("{:?}", self.technique).as_bytes());
        hasher.update(&self.target.target_hash);
        for uh in &self.unit_hashes {
            hasher.update(uh);
        }
        let digest: [u8; 32] = *hasher.finalize().as_bytes();

        Ok(LayerWitness::new(
            self.layer,
            self.scanned,
            self.remaining,
            self.technique,
            digest,
            now_unix_nanos,
        ))
    }
}
