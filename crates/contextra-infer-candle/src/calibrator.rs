//! Lokales Trait `ConfidenceCalibrator` zur Entkopplung von `contextra-rank` (SPEC-v2 §6.2).

use contextra_types::ConfigFingerprint;

/// Abstraktion für Vertrauens-/Kalibrierungs-Engines.
pub trait ConfidenceCalibrator: Send + Sync {
    /// Zeichnet ein Ground-Truth-Signal auf.
    fn record_outcome(&mut self, raw_score: f32, ground_truth: bool);
    /// Berechnet die kalibrierte Wahrscheinlichkeit.
    fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32>;
    /// Invalidiert gesammelte Beobachtungen bei Änderung des Fingerabdrucks (P8 / INV-CAL-2).
    fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint);
    /// Gibt die Anzahl der aktuell gesammelten Beobachtungen zurück.
    fn observation_count(&self) -> usize;
}

/// Standard-Identitätskalibrator.
#[derive(Debug, Default, Clone)]
pub struct IdentityCalibrator {
    fingerprint: Option<ConfigFingerprint>,
}

impl ConfidenceCalibrator for IdentityCalibrator {
    fn record_outcome(&mut self, _raw_score: f32, _ground_truth: bool) {}
    fn calibrated_probability(&mut self, raw_score: f32) -> Option<f32> {
        Some(raw_score)
    }
    fn invalidate_on_config_change(&mut self, new_fingerprint: ConfigFingerprint) {
        self.fingerprint = Some(new_fingerprint);
    }
    fn observation_count(&self) -> usize {
        0
    }
}
