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

/// Laufzeitzustand eines LinUCB-Bandits pro Profil.
///
/// # Formeln (§13.2)
/// Score: r̂_p(x) = θ_pᵀ x + α_p √(Σ_p(x)) - λ·c_p - μ·1[transport=HttpCloud]
/// Reward: r_adj = r_outcome - λ·c_p - μ·1[transport=HttpCloud]
/// Update (Diagonal): θ_p[i] += r_adj · x[i] / σ²_p[i]; σ²_p[i] += x[i]²
///
/// # Cold-Start (§13.2)
/// θ_p = 0-Vektor, σ²_p[i] = 1.0 (uninformative Prior), α_p = α_base (Default: 0.5)
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

    /// Aktualisiert θ und σ² nach beobachtetem Outcome.
    ///
    /// r_adj = r_outcome - λ·cost - μ·is_cloud
    /// θ[i] += r_adj · x[i] / σ²[i]
    /// σ²[i] += x[i]²
    pub fn update(&mut self, x: &[f32], r_outcome: f32, cost: f32, is_cloud_transport: bool) {
        debug_assert_eq!(x.len(), self.theta.len());

        let privacy_penalty = if is_cloud_transport { self.mu } else { 0.0 };
        let r_adj = r_outcome - self.lambda * cost - privacy_penalty;

        for (i, &xi) in x.iter().enumerate().take(self.theta.len()) {
            self.theta[i] += r_adj * xi / self.sigma_sq[i].max(1e-8);
            self.sigma_sq[i] += xi * xi;
        }
    }

    /// Drift-Kopplung: Erhöhe α temporär bei erkanntem Drift (§13.2).
    ///
    /// α_p ← α_p · k_drift (Default k_drift = 2.0)
    pub fn on_drift_detected(&mut self, k_drift: f32) {
        self.alpha *= k_drift;
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
    }
}
