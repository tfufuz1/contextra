// FILE-CONTEXT
// STAND: 2026-10-29
// ZWECK: SLM-Profildefinitionen und konformale Quantil-Kalibrierung (Gibbs & Candès).
// INVARIANTEN: INV-ROUTER-1 (quantile_threshold in [0.0, 1.0]), INV-P8-1 (Fingerprint-Invalidation).
// SIEHE AUCH: docs/decisions/ADR-020-contextra-brain.md

//! # SLM-Profildefinitionen und konformale Quantil-Kalibrierung (Gibbs & Candès)
//!
//! **Ring:** Ring 3 (Applikations- und Koordinationsschicht).
//! **Zweck:** Definition von Modelleigenschaften ([`SlmProfile`]), Schwellwertkalibrierungen
//! ([`ProfileCalibrator`], [`ConformalCalibrator`]) und Nachverfolgung dynamischer Zustände ([`ProfileCalibrationState`]).
//!
//! **Hintergrund:**
//! Jedes Modell verfügt über spezifische Begrenzungen (Token-Budget, Domänen-Tags) und ein Fehlerniveau.
//! Mittels adaptiver Quantil-Kalibrierung nach Gibbs & Candès (Conformal Prediction) passt der Router
//! die Akzeptanzschwellen dynamisch an, um statistische Fehlergarantien (z. B. Fehlerrate <= alpha) einzuhalten.
//!
//! **Kritische Invarianten:**
//! - `INV-ROUTER-1`: `quantile_threshold` liegt stets in `[0.0, 1.0]`.
//! - `INV-P8-1`: Ändert sich der Konfigurations-Fingerprint eines Modells, MUSS die Kalibrierung invalidiert werden.
//!
//! **Abhängigkeiten:**
//! - Upstream: `contextra_types::{ConfigFingerprint, ContextraError, Result, TokenBudget}`, `serde`.
//! - Downstream: [`crate::router::RouterEngine`], [`crate::dispatch`].

use contextra_types::{ConfigFingerprint, ContextraError, Result, TokenBudget};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Quantization level of the underlying model execution path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[allow(non_camel_case_types)]
// ANCHOR[contextra-router:PRF-001]
/// Quantisierungsstufe des Modells (z. B. F16, Q8_0, Q4_K_M).
pub enum QuantizationLevel {
    F16,
    Q8_0,
    Q4_K_M,
    #[default]
    Unknown,
}

/// Represents a Small Language Model (SLM) target and its domain expertise parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// ANCHOR[contextra-router:PRF-002]
/// Repräsentiert das Profil eines Small Language Models (SLM) mit Token-Budget und Domänen-Zuständigkeiten.
pub struct SlmProfile {
    /// Identifier name of the SLM (e.g. "coding-slm", "docs-slm").
    pub name: String,
    /// MCP endpoint address or URI for client communication.
    pub mcp_endpoint: String,
    /// Set of graph community IDs this SLM is domain-responsible for.
    #[serde(with = "serde_sorted_u64_set")]
    pub domain_communities: BTreeSet<u64>,
    /// Token budget configuration for prompt context trimming.
    pub token_budget: TokenBudget,
    /// Minimum relevance threshold score required for routing candidates.
    pub min_relevance_score: f32,
    /// Estimated resource/execution cost of selecting this profile.
    /// 0.0 indicates default/unspecified, in which case `token_budget.limit` is used.
    #[serde(default)]
    pub resource_cost_estimate: f32,
    /// P8-Pflicht: Fingerprint der LLM-Konfiguration bei der Kalibrierung.
    /// None = noch nicht kalibriert / Fingerprint noch nicht gesetzt.
    /// INVARIANTE INV-P8-1: Bei Fingerprint-Wechsel MUSS invalidate() aufgerufen werden.
    #[serde(default)]
    pub fingerprint: Option<ConfigFingerprint>,

    /// MCP-Transport-Kanal (Default: StdioMcp).
    /// Feature `cloud-egress-guard`: Aktiviert HttpCloud-Variante.
    #[cfg(feature = "cloud-egress-guard")]
    #[serde(default)]
    pub transport: crate::transport::Transport,

    /// LinUCB-Bandit-Zustand (None = Cascade-Routing aktiv).
    /// Feature `bandit-routing`: Aktiviert Contextual-Bandit-Routing.
    #[cfg(feature = "bandit-routing")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bandit_state: Option<crate::bandit::BanditProfileState>,
}

impl SlmProfile {
    /// Creates a new `SlmProfile`.
    // ANCHOR[contextra-router:PRF-003]
    /// Erzeugt ein neues `SlmProfile` mit Standardwerten.
    ///
    /// # Ablauf
    /// 1. Belegung aller Pflichtfelder (Name, Endpunkt, Communities, Budget, Mindestscore).
    /// 2. Initialisierung von Fingerprint und Bandit-Zustand auf `None`.
    ///
    /// # Vorbedingungen
    /// Gültige Parameter (Name nicht leer, Score in [0.0, 1.0]).
    pub fn new(
        name: impl Into<String>,
        mcp_endpoint: impl Into<String>,
        domain_communities: impl IntoIterator<Item = u64>,
        token_budget: TokenBudget,
        min_relevance_score: f32,
    ) -> Self {
        Self {
            name: name.into(),
            mcp_endpoint: mcp_endpoint.into(),
            domain_communities: domain_communities.into_iter().collect(),
            token_budget,
            min_relevance_score,
            resource_cost_estimate: 0.0,
            fingerprint: None,
            #[cfg(feature = "cloud-egress-guard")]
            transport: crate::transport::Transport::default(),
            #[cfg(feature = "bandit-routing")]
            bandit_state: None,
        }
    }

    /// Builder method to attach a specific `ConfigFingerprint` to this profile.
    // ANCHOR[contextra-router:PRF-004]
    /// Weist dem Profil einen Konfigurations-Fingerprint zu (P8-Pflicht).
    pub fn with_fingerprint(mut self, fingerprint: ConfigFingerprint) -> Self {
        self.fingerprint = Some(fingerprint);
        self
    }

    /// Builder method to attach an explicit resource cost estimate to this profile.
    // ANCHOR[contextra-router:PRF-005]
    /// Weist dem Profil geschätzte Ressourcenkosten zu.
    pub fn with_resource_cost_estimate(mut self, resource_cost_estimate: f32) -> Self {
        self.resource_cost_estimate = resource_cost_estimate;
        self
    }

    /// Returns the effective estimated cost for resource-aware fallback routing.
    /// If `resource_cost_estimate` is > 0.0, returns it; otherwise falls back to `token_budget.limit as f32`.
    // ANCHOR[contextra-router:PRF-006]
    /// Liefert die geschätzten Kosten des Profils (Fallback auf `token_budget.limit`).
    pub fn estimated_cost(&self) -> f32 {
        if self.resource_cost_estimate > 0.0 {
            self.resource_cost_estimate
        } else {
            self.token_budget.limit as f32
        }
    }

    /// Validates `SlmProfile` parameters.
    // ANCHOR[contextra-router:PRF-007]
    /// Validiert alle Invarianten des Profils.
    ///
    /// # Invarianten
    /// - Name darf nicht leer sein.
    /// - MCP-Endpunkt darf nicht leer sein.
    /// - `min_relevance_score` muss in `[0.0, 1.0]` liegen.
    /// - `token_budget` muss gültig sein (`limit > 0`, `reserve < limit`).
    /// - `resource_cost_estimate` darf nicht negativ oder NaN sein.
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(ContextraError::InvalidInput(
                "SLM profile name cannot be empty".to_string(),
            ));
        }
        if self.mcp_endpoint.trim().is_empty() {
            return Err(ContextraError::InvalidInput(
                "MCP endpoint cannot be empty".to_string(),
            ));
        }
        if !self.min_relevance_score.is_finite() || self.min_relevance_score < 0.0 {
            return Err(ContextraError::InvalidInput(
                "min_relevance_score must be finite and non-negative".to_string(),
            ));
        }
        if !self.resource_cost_estimate.is_finite() || self.resource_cost_estimate < 0.0 {
            return Err(ContextraError::InvalidInput(
                "resource_cost_estimate must be finite and non-negative".to_string(),
            ));
        }
        Ok(())
    }

    /// Creates and validates a new `SlmProfile`.
    // ANCHOR[contextra-router:PRF-008]
    /// Erzeugt ein neues `SlmProfile` und validiert es sofort strikt.
    pub fn try_new(
        name: impl Into<String>,
        mcp_endpoint: impl Into<String>,
        domain_communities: impl IntoIterator<Item = u64>,
        token_budget: TokenBudget,
        min_relevance_score: f32,
    ) -> Result<Self> {
        let profile = Self::new(
            name,
            mcp_endpoint,
            domain_communities,
            token_budget,
            min_relevance_score,
        );
        profile.validate()?;
        Ok(profile)
    }
}

/// Profile-Kalibrierung (Coverage-Garantie erst mit record_outcome()).
/// Basierend auf quantile-basierter Schwellenwert-Adaption (Gibbs & Candès, 2021).
///
/// Abgrenzung: Umbenannt von `ConformalCalibrator` zu `ProfileCalibrator`, um eine
/// Kollision mit `contextra_rank::ConformalCalibrator` zu vermeiden.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// ANCHOR[contextra-router:PRF-009]
/// Konformaler Kalibrator nach Gibbs & Candès zur dynamischen Schwellwertanpassung.
pub struct ProfileCalibrator {
    /// Target error rate (e.g., 0.05 for 95% coverage).
    pub alpha: f32,
    /// Adaptation rate for online quantile updates.
    pub gamma: f32,
    /// Current calibrated confidence threshold q_t.
    /// INV-ROUTER-1: Always clamped to [0.0, 1.0].
    pub quantile_threshold: f32,
    /// Number of errors observed in the current window.
    pub window_errors: u64,
    /// Total observations in the current window.
    pub window_total: u64,
}

#[deprecated(
    since = "0.1.0",
    note = "Renamed to ProfileCalibrator to avoid namespace collision with contextra_rank::ConformalCalibrator"
)]
// ANCHOR[contextra-router:PRF-010]
/// Typ-Alias für `ProfileCalibrator` gemäß Spec-Namenskonvention.
pub type ConformalCalibrator = ProfileCalibrator;

// ANCHOR[contextra-router:PRF-011]
impl Default for ProfileCalibrator {
    fn default() -> Self {
        Self {
            alpha: 0.05,
            gamma: 0.01,
            quantile_threshold: 0.5,
            window_errors: 0,
            window_total: 0,
        }
    }
}

impl ProfileCalibrator {
    /// Creates a new `ProfileCalibrator` with specified error tolerance and adaptation rate.
    // ANCHOR[contextra-router:PRF-012]
    /// Erstellt einen neuen `ProfileCalibrator` mit Fehlertoleranz `alpha`, Schrittweite `gamma` und Startschwelle.
    pub fn new(alpha: f32, gamma: f32, initial_threshold: f32) -> Self {
        Self {
            alpha: alpha.clamp(0.001, 0.5),
            gamma: gamma.clamp(0.001, 0.1),
            quantile_threshold: initial_threshold.clamp(0.0, 1.0),
            window_errors: 0,
            window_total: 0,
        }
    }

    /// Updates the conformal threshold based on a non-conformity score.
    // ANCHOR[contextra-router:PRF-013]
    /// Aktualisiert den Quantils-Schwellwert basierend auf einem beobachteten Non-Conformity-Score.
    ///
    /// # Ablauf
    /// 1. Fehlerprüfung: War der Score größer als die bisherige Schwelle?
    /// 2. Anpassung der Schwelle mit Schrittweite `gamma` und Fehlerniveau `alpha`.
    /// 3. Clamping der neuen Schwelle auf das Intervall `[0.0, 1.0]` (INV-ROUTER-1).
    ///
    /// # Rückgabe
    /// `true`, wenn die Vorhersage fehlerhaft war (Score > Schwelle).
    pub fn update(&mut self, non_conformity_score: f32) -> bool {
        self.window_total += 1;

        let indicator = if non_conformity_score > self.quantile_threshold {
            self.window_errors += 1;
            1.0f32
        } else {
            0.0f32
        };

        let old_threshold = self.quantile_threshold;
        self.quantile_threshold += self.gamma * (indicator - self.alpha);
        self.quantile_threshold = self.quantile_threshold.clamp(0.0, 1.0);

        (self.quantile_threshold - old_threshold).abs() > f32::EPSILON
    }

    /// Returns the current empirical error rate.
    // ANCHOR[contextra-router:PRF-014]
    /// Berechnet die empirische Fehlerrate im aktuellen Beobachtungsfenster.
    pub fn empirical_error_rate(&self) -> f32 {
        if self.window_total == 0 {
            return 0.0;
        }
        self.window_errors as f32 / self.window_total as f32
    }

    /// Resets statistics while preserving the current calibrated threshold.
    // ANCHOR[contextra-router:PRF-015]
    /// Setzt die Fensterzähler für Fehler und Gesamtzahl zurück.
    pub fn reset_window(&mut self) {
        self.window_errors = 0;
        self.window_total = 0;
    }
}

/// Laufzeit-Kalibrierungsstatistik für ein SLM-Profil.
/// Wird von RouterEngine verwaltet und nicht vom Aufrufer gesetzt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// ANCHOR[contextra-router:PRF-016]
/// Laufzeit-Zustand der Kalibrierung eines Profils inklusive Auswahlhäufigkeit und Konfidenz.
pub struct ProfileCalibrationState {
    /// Gesamtzahl der Routing-Entscheidungen für dieses Profil.
    pub times_selected: u64,
    /// Kumulierte Routing-Konfidenz (best_score / second_best_score).
    /// Höher = distinktiver. Startwert: 1.0
    pub cumulative_confidence: f64,
    /// Aktuell kalibrierter min_relevance_score-Wert
    /// (kann von original_min_relevance_score abweichen).
    pub calibrated_min_score: f32,
    /// Ursprünglicher min_relevance_score aus der Konfiguration.
    pub original_min_score: f32,
    /// Conformal calibrator for distribution-free threshold adaptation.
    pub conformal: ProfileCalibrator,
    /// Configuration fingerprint under which calibration statistics were gathered.
    #[serde(default)]
    pub last_calibrated_fingerprint: Option<ConfigFingerprint>,
}

impl Default for ProfileCalibrationState {
    fn default() -> Self {
        Self {
            times_selected: 0,
            cumulative_confidence: 1.0,
            calibrated_min_score: 0.5,
            original_min_score: 0.5,
            conformal: ProfileCalibrator::default(),
            last_calibrated_fingerprint: None,
        }
    }
}

impl ProfileCalibrationState {
    // ANCHOR[contextra-router:PRF-017]
    /// Erzeugt einen neuen Initialzustand für die Kalibrierung eines Profils.
    pub fn new(original_min_score: f32) -> Self {
        Self {
            times_selected: 0,
            cumulative_confidence: 1.0,
            calibrated_min_score: original_min_score,
            original_min_score,
            conformal: ProfileCalibrator::new(0.05, 0.01, original_min_score),
            last_calibrated_fingerprint: None,
        }
    }

    /// Checks active fingerprint against recorded calibration fingerprint.
    // ANCHOR[contextra-router:PRF-018]
    /// Prüft, ob der aktive Konfigurations-Fingerprint abweicht, und setzt bei Diskrepanz die Kalibrierung zurück (INV-P8-1).
    pub fn check_and_invalidate_fingerprint(&mut self, active_fp: Option<&ConfigFingerprint>) {
        let Some(active_fp) = active_fp else {
            if self.last_calibrated_fingerprint.is_some() || self.conformal.window_total > 0 {
                self.reset();
            }
            self.last_calibrated_fingerprint = None;
            return;
        };

        match &self.last_calibrated_fingerprint {
            Some(cal_fp) if cal_fp == active_fp => {
                // Fingerprint matches active execution configuration
            }
            _ => {
                // Configuration shift detected
                self.reset();
                self.last_calibrated_fingerprint = Some(active_fp.clone());
            }
        }
    }

    /// Returns whether calibration is valid and active for the given active fingerprint.
    // ANCHOR[contextra-router:PRF-019]
    /// Prüft, ob das Profil kalibriert und der Fingerprint noch valide ist.
    pub fn is_calibrated(&self, active_fp: Option<&ConfigFingerprint>) -> bool {
        let Some(active_fp) = active_fp else {
            return false;
        };
        if self.conformal.window_total < crate::router::CALIBRATION_WARMUP_WINDOW as u64 {
            return false;
        }
        self.last_calibrated_fingerprint.as_ref() == Some(active_fp)
    }

    /// Durchschnittliche Konfidenz über alle bisherigen Entscheidungen.
    // ANCHOR[contextra-router:PRF-020]
    /// Liefert die durchschnittliche Konfidenz über alle Auswahlen.
    pub fn average_confidence(&self) -> f64 {
        if self.times_selected == 0 {
            return 1.0;
        }
        self.cumulative_confidence / self.times_selected as f64
    }

    /// Recalibrates using the conformal calibrator.
    // ANCHOR[contextra-router:PRF-021]
    /// Führt eine Neukalibrierung anhand des beobachteten Non-Conformity-Scores durch.
    pub fn recalibrate_conformal(&mut self, non_conformity_score: f32) -> bool {
        let adjusted = self.conformal.update(non_conformity_score);
        if adjusted {
            let lower = self.original_min_score * 0.5;
            let upper = self.original_min_score * 2.0;
            self.calibrated_min_score = self.conformal.quantile_threshold.clamp(lower, upper);
        }
        adjusted
    }

    /// Setzt Kalibrierungsstate vollständig zurück.
    // ANCHOR[contextra-router:PRF-022]
    /// Setzt alle Kalibrierungsstatistiken auf die Ausgangswerte zurück.
    pub fn reset(&mut self) {
        self.times_selected = 0;
        self.cumulative_confidence = 1.0;
        self.calibrated_min_score = self.original_min_score;
        self.conformal = ProfileCalibrator::new(0.05, 0.01, self.original_min_score);
        self.last_calibrated_fingerprint = None;
    }
}

mod serde_sorted_u64_set {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::BTreeSet;

    pub fn serialize<S: Serializer>(set: &BTreeSet<u64>, s: S) -> Result<S::Ok, S::Error> {
        let v: Vec<u64> = set.iter().copied().collect();
        v.serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeSet<u64>, D::Error> {
        Vec::<u64>::deserialize(d).map(|v| v.into_iter().collect())
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_calibrator_invariants() {
        let mut cal = ProfileCalibrator::new(0.05, 0.05, 0.5);

        for _ in 0..100 {
            cal.update(1.0);
            assert!(cal.quantile_threshold >= 0.0 && cal.quantile_threshold <= 1.0);
        }

        for _ in 0..100 {
            cal.update(0.0);
            assert!(cal.quantile_threshold >= 0.0 && cal.quantile_threshold <= 1.0);
        }
    }

    #[test]
    fn test_profile_calibrator_coverage_guarantee() {
        let alpha = 0.10;
        let gamma = 0.01;
        let mut cal = ProfileCalibrator::new(alpha, gamma, 0.5);

        let mut lcg = 123456789u64;
        for _ in 0..10_000 {
            lcg = lcg
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let rand_val = ((lcg >> 32) as u32 as f32) / (u32::MAX as f32);
            cal.update(rand_val);
        }

        let empirical = cal.empirical_error_rate();
        assert!((empirical - alpha).abs() < 0.05);
    }

    #[test]
    fn test_profile_calibration_state_conformal_recalibrate() {
        let mut state = ProfileCalibrationState::new(0.6);
        assert_eq!(state.calibrated_min_score, 0.6);

        let adjusted = state.recalibrate_conformal(0.8);
        assert!(adjusted);
        assert!(state.calibrated_min_score >= 0.3 && state.calibrated_min_score <= 1.2);
    }

    #[test]
    fn test_domain_communities_contains_is_o1(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let communities: BTreeSet<u64> = (1..=1000).collect();
        let profile = SlmProfile::new(
            "test-slm",
            "http://localhost:8000/mcp",
            communities,
            TokenBudget::new(1000, 100),
            0.1,
        );

        assert!(profile.domain_communities.contains(&500));
        assert!(profile.domain_communities.contains(&1));
        assert!(profile.domain_communities.contains(&1000));
        assert!(!profile.domain_communities.contains(&0));
        assert!(!profile.domain_communities.contains(&1001));

        let json_str = serde_json::to_string(&profile)?;
        let json_val: serde_json::Value = serde_json::from_str(&json_str)?;

        let arr = json_val["domain_communities"]
            .as_array()
            .ok_or("domain_communities is not an array")?;

        assert_eq!(arr.len(), 1000);
        let expected_sorted: Vec<u64> = (1..=1000).collect();
        let actual_arr: Vec<u64> = arr.iter().filter_map(|v| v.as_u64()).collect();
        assert_eq!(actual_arr, expected_sorted);

        let deserialized: SlmProfile = serde_json::from_str(&json_str)?;
        assert_eq!(deserialized, profile);

        Ok(())
    }

    #[test]
    fn test_deserialize_legacy_json_array() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let profile_orig = SlmProfile::new(
            "test-slm",
            "http://localhost:8000/mcp",
            vec![1, 2, 3],
            TokenBudget::new(1000, 100),
            0.5,
        );
        let raw_json = serde_json::to_string(&profile_orig)?;

        let profile: SlmProfile = serde_json::from_str(&raw_json)?;
        let expected_set: BTreeSet<u64> = [1, 2, 3].into_iter().collect();
        assert_eq!(profile.domain_communities, expected_set);

        Ok(())
    }
}
