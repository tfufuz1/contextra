//! LinUCB Contextual Bandit für SLM-Profil-Routing (§13.2).
//! Feature `bandit-routing` (Default: off). Kein Default-Wechsel ohne P7-Nachweis.

#![cfg(feature = "bandit-routing")]

use serde::{Deserialize, Serialize};

/// LinUCB-Implementierungsvariante (§13.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BanditImplementation {
    /// Diagonal-Approximation O(d): kein BLAS, Default.
    #[default]
    DiagonalApproximation,
    /// Sherman-Morrison O(d²): opt-in via Feature `egress-sherman-morrison`.
    #[cfg(feature = "egress-sherman-morrison")]
    ShermanMorrison,
}

fn default_gamma() -> f32 {
    0.999
}

fn default_drift_gamma() -> f32 {
    0.95
}

fn default_drift_decay_window() -> usize {
    50
}

/// Laufzeitzustand eines LinUCB-Bandits pro Profil.
///
/// # Formeln (§13.2)
/// Score: r̂_p(x) = θ_pᵀ x + α_p √(Σ_p(x)) - λ·c_p - μ·1[transport=HttpCloud]
/// Reward: r_adj = r_outcome - λ·c_p - μ·1[transport=HttpCloud]
/// Update (Discounted-Diagonal, Garivier & Moulines 2011):
///   σ²_p[i] ← max(γ · σ²_p[i], 1.0)
///   θ_p[i] += r_adj · x[i] / σ²_p[i]
///   σ²_p[i] += x[i]²
///
/// # Cold-Start (§13.2)
/// θ_p = 0-Vektor, σ²_p[i] = 1.0 (uninformative Prior), α_p = α_base (Default: 0.5)
/// gamma = 0.999 (1000 Schritte effektives Gedächtnisfenster 1/(1-γ))
/// drift_gamma = 0.95 (beschleunigter Decay nach Drift-Erkennung)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BanditProfileState {
    /// Gewichtsvektor θ_p (Dimension = Embedding-Dim).
    pub theta: Vec<f32>,
    /// Diagonale Kovarianz-Terme σ²_p (Diagonal-Approximation).
    pub sigma_sq: Vec<f32>,
    /// Exploration-Parameter α_p.
    pub alpha: f32,
    /// Kostensensitivität λ (Default: 0.1).
    pub lambda: f32,
    /// Privacy-Malus für Cloud-Transport μ (Default: 0.2).
    pub mu: f32,
    /// Diskontfaktor γ für stationären Betrieb (Default: 0.999).
    /// Korrespondiert mit einem effektiven Beobachtungshorizont von ~1000 Schritten (1 / (1 - γ)).
    #[serde(default = "default_gamma")]
    pub gamma: f32,
    /// Accelerated Diskontfaktor γ_drift nach erkannter Drift (Default: 0.95).
    #[serde(default = "default_drift_gamma")]
    pub drift_gamma: f32,
    /// Verbleibende Update-Schritte mit beschleunigtem Post-Drift Decay (Default: 0).
    #[serde(default)]
    pub drift_steps_remaining: usize,
    /// Dauer des Post-Drift Decay-Fensters in Update-Schritten (Default: 50).
    #[serde(default = "default_drift_decay_window")]
    pub drift_decay_window: usize,
    /// Implementierungsvariante.
    pub implementation: BanditImplementation,
}

impl BanditProfileState {
    /// Erstellt einen neuen Cold-Start-Zustand für Dimension `d`.
    pub fn cold_start(d: usize, alpha_base: f32) -> Self {
        Self {
            theta: vec![0.0f32; d],
            sigma_sq: vec![1.0f32; d], // Uninformative Prior
            alpha: alpha_base,
            lambda: 0.1,
            mu: 0.2,
            gamma: default_gamma(),
            drift_gamma: default_drift_gamma(),
            drift_steps_remaining: 0,
            drift_decay_window: default_drift_decay_window(),
            implementation: BanditImplementation::default(),
        }
    }

    /// Berechnet UCB-Score für Kontext-Embedding `x` und Profilkosten `cost`.
    ///
    /// r̂_p(x) = θᵀx + α√(Σ(x)) - λ·cost - μ·is_cloud
    pub fn score(&self, x: &[f32], cost: f32, is_cloud_transport: bool) -> f32 {
        debug_assert_eq!(
            x.len(),
            self.theta.len(),
            "Embedding-Dimension muss übereinstimmen"
        );

        let dot: f32 = self.theta.iter().zip(x.iter()).map(|(t, xi)| t * xi).sum();

        // Diagonal-Approximation: Σ(x) = Σ_i x_i² / σ²_i
        let variance_term: f32 = self
            .sigma_sq
            .iter()
            .zip(x.iter())
            .map(|(s, xi)| xi * xi / s.max(1e-8))
            .sum::<f32>()
            .sqrt();

        let privacy_penalty = if is_cloud_transport { self.mu } else { 0.0 };

        dot + self.alpha * variance_term - self.lambda * cost - privacy_penalty
    }

    /// Aktualisiert θ und σ² nach beobachtetem Outcome unter Berücksichtigung von Discounting.
    ///
    /// r_adj = r_outcome - λ·cost - μ·is_cloud
    /// σ²[i] ← max(γ_eff · σ²[i], 1.0)
    /// θ[i] += r_adj · x[i] / σ²[i]
    /// σ²[i] += x[i]²
    pub fn update(&mut self, x: &[f32], r_outcome: f32, cost: f32, is_cloud_transport: bool) {
        debug_assert_eq!(x.len(), self.theta.len());

        let privacy_penalty = if is_cloud_transport { self.mu } else { 0.0 };
        let r_adj = r_outcome - self.lambda * cost - privacy_penalty;

        // Effektiver Diskontfaktor γ (Post-Drift beschleunigter Decay vs. Normalbetrieb)
        let effective_gamma = if self.drift_steps_remaining > 0 {
            self.drift_steps_remaining -= 1;
            self.drift_gamma
        } else {
            self.gamma
        };

        for (i, &xi) in x.iter().enumerate().take(self.theta.len()) {
            // Discounted-Update: Ältere Beobachtungen dämpfen, floored bei Cold-Start Prior 1.0
            self.sigma_sq[i] = (self.sigma_sq[i] * effective_gamma).max(1.0);
            self.theta[i] += r_adj * xi / self.sigma_sq[i].max(1e-8);
            self.sigma_sq[i] += xi * xi;
        }
    }

    /// Drift-Kopplung: Erhöhe α temporär und aktiviere beschleunigten Decay bei erkannter Drift (§13.2).
    ///
    /// α_p ← α_p · k_drift (Default k_drift = 2.0)
    /// drift_steps_remaining ← drift_decay_window (Default: 50)
    pub fn on_drift_detected(&mut self, k_drift: f32) {
        self.alpha *= k_drift;
        self.drift_steps_remaining = self.drift_decay_window;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cold_start_zero_theta_unit_sigma() {
        let state = BanditProfileState::cold_start(4, 0.5);
        assert!(state.theta.iter().all(|&v| v == 0.0));
        assert!(state.sigma_sq.iter().all(|&v| v == 1.0));
        assert_eq!(state.alpha, 0.5);
    }

    #[test]
    fn test_score_cold_start_is_exploration_dominated() {
        let state = BanditProfileState::cold_start(4, 0.5);
        let x = vec![1.0f32; 4];
        // Cold-Start: θᵀx = 0, Varianzterm > 0 → Score > 0
        let score = state.score(&x, 0.0, false);
        assert!(
            score > 0.0,
            "Cold-Start Score muss durch Exploration > 0 sein"
        );
    }

    #[test]
    fn test_update_increases_theta_on_positive_reward() {
        let mut state = BanditProfileState::cold_start(2, 0.5);
        let x = vec![1.0f32, 0.0f32];
        state.update(&x, 1.0, 0.0, false); // Success = 1.0
        assert!(
            state.theta[0] > 0.0,
            "θ[0] muss nach positivem Reward steigen"
        );
        assert_eq!(state.theta[1], 0.0, "θ[1] bleibt 0 da x[1]=0");
    }

    #[test]
    fn test_cloud_transport_penalty_reduces_score() {
        let state = BanditProfileState::cold_start(2, 0.5);
        let x = vec![1.0f32, 1.0f32];
        let score_local = state.score(&x, 0.0, false);
        let score_cloud = state.score(&x, 0.0, true);
        assert!(
            score_local > score_cloud,
            "Cloud-Transport-Penalty muss Score reduzieren"
        );
    }

    #[test]
    fn test_drift_increases_alpha() {
        let mut state = BanditProfileState::cold_start(2, 0.5);
        let alpha_before = state.alpha;
        state.on_drift_detected(2.0);
        assert!((state.alpha - alpha_before * 2.0).abs() < 1e-6);
        assert_eq!(state.drift_steps_remaining, state.drift_decay_window);
    }

    #[test]
    fn test_bandit_forgetting() {
        // Testet Erholung der Lernfähigkeit nach Drift (Nicht-Stationarität)
        let d = 2;
        let mut state_discounted = BanditProfileState::cold_start(d, 0.5);

        let mut state_stiff = BanditProfileState::cold_start(d, 0.5);
        state_stiff.gamma = 1.0;
        state_stiff.drift_gamma = 1.0;

        let x = vec![1.0f32, 0.0f32];

        // Phase 1: 500 stationäre Schritte mit hohem Reward (1.0)
        for _ in 0..500 {
            state_discounted.update(&x, 1.0, 0.0, false);
            state_stiff.update(&x, 1.0, 0.0, false);
        }

        let theta_disc_peak = state_discounted.theta[0];
        let theta_stiff_peak = state_stiff.theta[0];

        assert!(theta_disc_peak > 0.5);
        assert!(theta_stiff_peak > 0.5);

        // Phase 2: Sprunghafte Drift (Reward fällt dauerhaft auf 0.0)
        state_discounted.on_drift_detected(2.0);
        state_stiff.on_drift_detected(2.0);
        state_stiff.drift_steps_remaining = 0; // Kein beschleunigter Decay für den starren Banditen

        // Phase 3: 100 Schritte nach Drift mit neuem negativem Reward / Misserfolg (-1.0)
        for _ in 0..100 {
            state_discounted.update(&x, -1.0, 0.0, false);
            state_stiff.update(&x, -1.0, 0.0, false);
        }

        let drop_discounted = theta_disc_peak - state_discounted.theta[0];
        let drop_stiff = theta_stiff_peak - state_stiff.theta[0];

        println!(
            "Theta[0] Peak -> Post-Drift: Discounted ({:.4} -> {:.4}, Drop={:.4}), Stiff ({:.4} -> {:.4}, Drop={:.4})",
            theta_disc_peak, state_discounted.theta[0], drop_discounted,
            theta_stiff_peak, state_stiff.theta[0], drop_stiff
        );

        assert!(
            state_discounted.theta[0] < state_stiff.theta[0],
            "Diskontierter Bandit muss theta[0] nach Drift schneller korrigieren als starrer Bandit"
        );
        assert!(
            drop_discounted > 3.0 * drop_stiff,
            "Drop des diskontierten Banditen ({:.4}) muss mindestens 3x größer sein als beim starren Banditen ({:.4})",
            drop_discounted, drop_stiff
        );
    }
}
