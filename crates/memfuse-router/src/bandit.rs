//! LinUCB Contextual Bandit für SLM-Profil-Routing (§13.2).
//! Feature `bandit-routing` (Default: off). Kein Default-Wechsel ohne P7-Nachweis.

#![cfg(feature = "bandit-routing")]
#![allow(clippy::needless_range_loop)]

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

fn default_alpha_base() -> f32 {
    0.5
}

fn default_alpha_max_multiplier() -> f32 {
    4.0
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
    /// Inverser Kovarianzmatrix-Speicher A⁻¹ (d x d, Row-Major) für Sherman-Morrison O(d²).
    #[serde(default)]
    pub inv_a: Vec<f32>,
    /// Pre-allocated Buffer für Matrix-Vektor Produkte (d Elemente) zur Vermeidung von Hot-Path Allokationen.
    #[serde(skip, default)]
    pub work_buf: Vec<f32>,
    /// Exploration-Parameter α_p.
    pub alpha: f32,
    /// Cold-Start Baseline Exploration Parameter α_base (Default: 0.5).
    #[serde(default = "default_alpha_base")]
    pub alpha_base: f32,
    /// Maximale Eskalation von α als Multiplikator von α_base (Default: 4.0).
    #[serde(default = "default_alpha_max_multiplier")]
    pub alpha_max_multiplier: f32,
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
        let mut inv_a = vec![0.0f32; d * d];
        for i in 0..d {
            inv_a[i * d + i] = 1.0;
        }
        Self {
            theta: vec![0.0f32; d],
            sigma_sq: vec![1.0f32; d], // Uninformative Prior
            inv_a,
            work_buf: vec![0.0f32; d],
            alpha: alpha_base,
            alpha_base,
            alpha_max_multiplier: default_alpha_max_multiplier(),
            lambda: 0.1,
            mu: 0.2,
            gamma: default_gamma(),
            drift_gamma: default_drift_gamma(),
            drift_steps_remaining: 0,
            drift_decay_window: default_drift_decay_window(),
            implementation: BanditImplementation::default(),
        }
    }

    fn ensure_inv_a(&mut self) {
        let d = self.theta.len();
        if self.inv_a.len() != d * d {
            self.inv_a = vec![0.0f32; d * d];
            for i in 0..d {
                self.inv_a[i * d + i] = 1.0;
            }
        }
    }

    fn ensure_work_buf(&mut self) {
        let d = self.theta.len();
        if self.work_buf.len() != d {
            self.work_buf.resize(d, 0.0);
        }
    }

    /// Berechnet UCB-Score für Kontext-Embedding `x` und Profilkosten `cost`.
    ///
    /// r̂_p(x) = θᵀx + α√(Σ(x)) - λ·cost - μ·is_cloud
    #[allow(clippy::needless_range_loop)]
    pub fn score(&self, x: &[f32], cost: f32, is_cloud_transport: bool) -> f32 {
        debug_assert_eq!(
            x.len(),
            self.theta.len(),
            "Embedding-Dimension muss übereinstimmen"
        );

        let dot: f32 = self.theta.iter().zip(x.iter()).map(|(t, xi)| t * xi).sum();

        let variance_term = match self.implementation {
            BanditImplementation::DiagonalApproximation => self
                .sigma_sq
                .iter()
                .zip(x.iter())
                .map(|(s, xi)| xi * xi / s.max(1e-8))
                .sum::<f32>()
                .sqrt(),
            #[cfg(feature = "egress-sherman-morrison")]
            #[allow(clippy::needless_range_loop)]
            BanditImplementation::ShermanMorrison => {
                let d = self.theta.len();
                if self.inv_a.len() == d * d {
                    let mut var_sum = 0.0f32;
                    for i in 0..d {
                        let row_offset = i * d;
                        let mut row_dot = 0.0f32;
                        for j in 0..d {
                            row_dot += self.inv_a[row_offset + j] * x[j];
                        }
                        var_sum += x[i] * row_dot;
                    }
                    var_sum.max(0.0).sqrt()
                } else {
                    self.sigma_sq
                        .iter()
                        .zip(x.iter())
                        .map(|(s, xi)| xi * xi / s.max(1e-8))
                        .sum::<f32>()
                        .sqrt()
                }
            }
        };

        let privacy_penalty = if is_cloud_transport { self.mu } else { 0.0 };

        dot + self.alpha * variance_term - self.lambda * cost - privacy_penalty
    }

    /// Aktualisiert θ und σ² (sowie A⁻¹ bei Sherman-Morrison) nach beobachtetem Outcome unter Berücksichtigung von Discounting.
    ///
    /// r_adj = r_outcome - λ·cost - μ·is_cloud
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

        match self.implementation {
            BanditImplementation::DiagonalApproximation => {
                for (i, &xi) in x.iter().enumerate().take(self.theta.len()) {
                    self.sigma_sq[i] = (self.sigma_sq[i] * effective_gamma).max(1.0);
                    self.theta[i] += r_adj * xi / self.sigma_sq[i].max(1e-8);
                    self.sigma_sq[i] += xi * xi;
                }
            }
            #[cfg(feature = "egress-sherman-morrison")]
            BanditImplementation::ShermanMorrison => {
                let d = self.theta.len();
                self.ensure_inv_a();
                self.ensure_work_buf();

                let gamma_inv = 1.0 / effective_gamma.max(1e-5);

                // 1. Berechne v_x = A⁻¹ x in work_buf
                let mut xt_vx = 0.0f32;
                for i in 0..d {
                    let row_offset = i * d;
                    let mut sum = 0.0f32;
                    for j in 0..d {
                        sum += self.inv_a[row_offset + j] * x[j];
                    }
                    self.work_buf[i] = sum;
                    xt_vx += x[i] * sum;
                }

                // 2. Diskonterte Zwischenwerte
                let denominator = 1.0 + gamma_inv * xt_vx;
                let denom_safe = denominator.max(1e-8);

                // 3. Gain Vector k = (γ⁻¹ A⁻¹ x) / (1 + γ⁻¹ xᵀ A⁻¹ x)
                // Matrix-Update: A_new⁻¹ = γ⁻¹ A_old⁻¹ - k (γ⁻¹ A_old⁻¹ x)ᵀ
                // Parameter-Update: θ_new = θ_old + (r_adj - θ_oldᵀ x) * k
                let mut pred_theta_x = 0.0f32;
                for i in 0..d {
                    pred_theta_x += self.theta[i] * x[i];
                }
                let residual = r_adj - pred_theta_x;

                // Aktualisiere inv_a und theta
                for i in 0..d {
                    let v_disc_i = gamma_inv * self.work_buf[i];
                    let k_i = v_disc_i / denom_safe;

                    let row_offset = i * d;
                    for j in 0..d {
                        let v_disc_j = gamma_inv * self.work_buf[j];
                        self.inv_a[row_offset + j] =
                            gamma_inv * self.inv_a[row_offset + j] - k_i * v_disc_j;
                    }

                    self.theta[i] += residual * k_i;
                    self.sigma_sq[i] = (self.sigma_sq[i] * effective_gamma).max(1.0) + x[i] * x[i];
                }
            }
        }
    }

    /// Drift-Kopplung: Erhöhe α temporär und aktiviere beschleunigten Decay bei erkannter Drift (§13.2).
    ///
    /// α_p ← min(α_p · k_drift, α_base · alpha_max_multiplier)
    /// drift_steps_remaining ← drift_decay_window (Default: 50)
    pub fn on_drift_detected(&mut self, k_drift: f32) {
        self.alpha = (self.alpha * k_drift).min(self.alpha_base * self.alpha_max_multiplier);
        self.drift_steps_remaining = self.drift_decay_window;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "egress-sherman-morrison")]
    fn test_sherman_morrison_score_and_update() {
        let mut state = BanditProfileState::cold_start(4, 0.5);
        state.implementation = BanditImplementation::ShermanMorrison;
        let x = vec![1.0f32, 0.0, 0.0, 0.0];

        let score_before = state.score(&x, 0.0, false);
        assert!(score_before > 0.0);

        state.update(&x, 1.0, 0.0, false);
        assert!(state.theta[0] > 0.0);

        let score_after = state.score(&x, 0.0, false);
        assert_ne!(score_after, score_before);
    }

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
    fn test_repeated_drift_caps_at_max_multiplier() {
        let mut state = BanditProfileState::cold_start(2, 0.5);
        let max_expected_alpha = state.alpha_base * state.alpha_max_multiplier; // 0.5 * 4.0 = 2.0
        for _ in 0..10 {
            state.on_drift_detected(2.0);
        }
        assert!((state.alpha - max_expected_alpha).abs() < 1e-6);
    }

    #[test]
    fn test_serde_backward_compatibility_for_alpha_fields() {
        // Simuliert einen alten JSON-Snapshot ohne alpha_base und alpha_max_multiplier
        let legacy_json = r#"{
            "theta": [0.0, 0.0],
            "sigma_sq": [1.0, 1.0],
            "inv_a": [],
            "alpha": 0.5,
            "lambda": 0.1,
            "mu": 0.2,
            "gamma": 0.999,
            "drift_gamma": 0.95,
            "drift_steps_remaining": 0,
            "drift_decay_window": 50,
            "implementation": "DiagonalApproximation"
        }"#;

        let state: BanditProfileState = serde_json::from_str(legacy_json)
            .unwrap_or_else(|e| panic!("Deserialisierung alter Snapshots fehlgeschlagen: {e}"));
        assert_eq!(state.alpha_base, 0.5);
        assert_eq!(state.alpha_max_multiplier, 4.0);
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
