//! LinUCB Contextual Bandit für SLM-Profil-Routing (§13.2).
//! Feature `bandit-routing` (Default: off). Kein Default-Wechsel ohne P7-Nachweis.

#![cfg(feature = "bandit-routing")]
#![allow(clippy::needless_range_loop)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Fehlerzustände des Contextual Bandits.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum BanditError {
    /// Dimension des Eingabevektors stimmt nicht mit dem Bandit-Zustand überein.
    #[error("Embedding dimension mismatch: expected {expected}, actual {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
}

/// 64-Byte cache-aligned Wrapper um `Vec<f32>` zur Vermeidung von False Sharing.
///
/// Hinweis: Der `Vec<f32>`-Heap-Buffer selbst folgt zwar nicht zwingend der 64-Byte-Grenze des Structs
/// (abhängig vom globalen Allocator), aber `repr(align(64))` richtet die Struct-Instanz selbst
/// an einer 64-Byte-Grenze aus und der Heap-Buffer liefert bei `Vec::with_capacity` i. d. R.
/// mindestens 16/32-Byte ausgerichtete Zeiger.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(align(64))]
pub struct AlignedF32Vec(pub Vec<f32>);

impl std::ops::Deref for AlignedF32Vec {
    type Target = [f32];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AlignedF32Vec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AlignedF32Vec {
    /// Erstellt einen neuen `AlignedF32Vec` aus einem `Vec<f32>`.
    pub fn new(vec: Vec<f32>) -> Self {
        Self(vec)
    }

    /// Erstellt einen neuen `AlignedF32Vec` mit `len` Nullen.
    pub fn zeros(len: usize) -> Self {
        Self(vec![0.0f32; len])
    }
}

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
    pub inv_a: AlignedF32Vec,
    /// Pre-allocated Buffer für Matrix-Vektor Produkte (d Elemente) zur Vermeidung von Hot-Path Allokationen.
    #[serde(skip, default)]
    pub work_buf: AlignedF32Vec,
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
            inv_a: AlignedF32Vec::new(inv_a),
            work_buf: AlignedF32Vec::zeros(d),
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
            let mut inv_a = vec![0.0f32; d * d];
            for i in 0..d {
                inv_a[i * d + i] = 1.0;
            }
            self.inv_a = AlignedF32Vec::new(inv_a);
        }
    }

    fn ensure_work_buf(&mut self) {
        let d = self.theta.len();
        if self.work_buf.len() != d {
            self.work_buf = AlignedF32Vec::zeros(d);
        }
    }

    /// Gibt die erwartete Dimension der Feature-Vektoren zurück.
    pub fn expected_dim(&self) -> usize {
        self.theta.len()
    }

    /// Berechnet UCB-Score für Kontext-Embedding `x` und Profilkosten `cost`.
    ///
    /// r̂_p(x) = θᵀx + α√(Σ(x)) - λ·cost - μ·is_cloud
    #[allow(clippy::needless_range_loop)]
    pub fn score(
        &self,
        x: &[f32],
        cost: f32,
        is_cloud_transport: bool,
    ) -> Result<f32, BanditError> {
        if x.len() != self.theta.len() {
            return Err(BanditError::DimensionMismatch {
                expected: self.theta.len(),
                actual: x.len(),
            });
        }

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
            BanditImplementation::ShermanMorrison => {
                let d = self.theta.len();
                if self.inv_a.len() == d * d {
                    let var_sum: f32 = self
                        .inv_a
                        .chunks_exact(d)
                        .zip(x.iter())
                        .map(|(row, &xi)| {
                            let row_dot: f32 =
                                row.iter().zip(x.iter()).map(|(&a, &xj)| a * xj).sum();
                            xi * row_dot
                        })
                        .sum();
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

        Ok(dot + self.alpha * variance_term - self.lambda * cost - privacy_penalty)
    }

    /// Aktualisiert θ und σ² (sowie A⁻¹ bei Sherman-Morrison) nach beobachtetem Outcome unter Berücksichtigung von Discounting.
    ///
    /// r_adj = r_outcome - λ·cost - μ·is_cloud
    pub fn update(
        &mut self,
        x: &[f32],
        r_outcome: f32,
        cost: f32,
        is_cloud_transport: bool,
    ) -> Result<(), BanditError> {
        if x.len() != self.theta.len() {
            return Err(BanditError::DimensionMismatch {
                expected: self.theta.len(),
                actual: x.len(),
            });
        }

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

                // 1. Berechne v_disc = γ⁻¹ A⁻¹ x in work_buf und xᵀ v_disc
                let mut xt_v_disc = 0.0f32;
                for (i, row) in self.inv_a.chunks_exact(d).enumerate() {
                    let row_dot: f32 = row.iter().zip(x.iter()).map(|(&a, &xj)| a * xj).sum();
                    let v_disc_i = gamma_inv * row_dot;
                    self.work_buf[i] = v_disc_i;
                    xt_v_disc += x[i] * v_disc_i;
                }

                // 2. Denominator und Gain Vector k
                let denominator = 1.0 + xt_v_disc;
                let denom_safe = denominator.max(1e-8);

                // 3. Parameter Residual: r_adj - θᵀ x
                let pred_theta_x: f32 = self
                    .theta
                    .iter()
                    .zip(x.iter())
                    .map(|(&t, &xi)| t * xi)
                    .sum();
                let residual = r_adj - pred_theta_x;

                // 4. Matrix-Update: A_new⁻¹ = γ⁻¹ A_old⁻¹ - k (v_disc)ᵀ
                // Parameter-Update: θ_new = θ_old + (r_adj - θ_oldᵀ x) * k
                let work_slice = &self.work_buf[..d];
                for (i, row) in self.inv_a.chunks_exact_mut(d).enumerate() {
                    let k_i = work_slice[i] / denom_safe;

                    for (row_j, &v_j) in row.iter_mut().zip(work_slice.iter()) {
                        *row_j = gamma_inv * *row_j - k_i * v_j;
                    }

                    self.theta[i] += residual * k_i;
                    self.sigma_sq[i] = (self.sigma_sq[i] * effective_gamma).max(1.0) + x[i] * x[i];
                }
            }
        }

        Ok(())
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

        let score_before = state.score(&x, 0.0, false).expect("valid score");
        assert!(score_before > 0.0);

        state.update(&x, 1.0, 0.0, false).expect("valid update");
        assert!(state.theta[0] > 0.0);

        let score_after = state.score(&x, 0.0, false).expect("valid score");
        assert_ne!(score_after, score_before);
    }

    #[test]
    #[cfg(feature = "egress-sherman-morrison")]
    fn test_sherman_morrison_correctness_and_precision() {
        let d = 8;
        let mut state = BanditProfileState::cold_start(d, 0.5);
        state.implementation = BanditImplementation::ShermanMorrison;

        let x = vec![0.5f32, -0.2, 0.8, 0.1, -0.4, 0.6, -0.1, 0.3];
        let cost = 0.15f32;
        let is_cloud = false;
        let reward = 0.85f32;

        let score_initial = state.score(&x, cost, is_cloud).expect("valid score");
        // Cold start theta = 0, inv_a = I -> x^T I x = norm(x)^2 = 1.36
        // dot = 0, variance = sqrt(1.36) ≈ 1.16619, alpha = 0.5, cost*0.1 = 0.015
        let expected_var = x.iter().map(|&v| v * v).sum::<f32>().sqrt();
        let expected_score = 0.5 * expected_var - 0.1 * cost;
        assert!(
            (score_initial - expected_score).abs() < 1e-5,
            "Initial UCB score calculation mismatch: expected {}, got {}",
            expected_score,
            score_initial
        );

        // Perform 5 updates and verify numerical consistency and theta convergence
        for _ in 0..5 {
            state
                .update(&x, reward, cost, is_cloud)
                .expect("valid update");
        }

        assert!(
            state.theta[0] > 0.0,
            "Theta[0] must be positive after positive rewards"
        );
        let score_updated = state.score(&x, cost, is_cloud).expect("valid score");
        assert!(
            score_updated > score_initial,
            "Score must increase after positive updates"
        );
    }

    #[test]
    #[ignore]
    #[cfg(feature = "egress-sherman-morrison")]
    fn bench_sherman_morrison_p95_latency() {
        use std::time::Instant;

        let d = 768;
        let iterations = 1000;
        let mut state = BanditProfileState::cold_start(d, 0.5);
        state.implementation = BanditImplementation::ShermanMorrison;

        let x = vec![0.5f32; d];
        let cost = 0.2f32;
        let is_cloud = false;
        let reward = 0.8f32;

        // Warmup
        for _ in 0..10 {
            let _ = state.score(&x, cost, is_cloud);
            let _ = state.update(&x, reward, cost, is_cloud);
        }

        let mut latencies_us = Vec::with_capacity(iterations);
        for _ in 0..iterations {
            let start = Instant::now();
            let _score = state.score(&x, cost, is_cloud);
            let _ = state.update(&x, reward, cost, is_cloud);
            latencies_us.push(start.elapsed().as_micros() as u64);
        }

        latencies_us.sort_unstable();
        let p95_idx = (iterations as f64 * 0.95) as usize;
        let p95_us = latencies_us[p95_idx.min(iterations - 1)];

        println!(
            "\n[BENCHMARK RESULT] Sherman-Morrison (d={}) P95 Decision + Update Latency over {} iterations: {} µs",
            d, iterations, p95_us
        );
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
        let score = state.score(&x, 0.0, false).expect("valid score");
        assert!(
            score > 0.0,
            "Cold-Start Score muss durch Exploration > 0 sein"
        );
    }

    #[test]
    fn test_update_increases_theta_on_positive_reward() {
        let mut state = BanditProfileState::cold_start(2, 0.5);
        let x = vec![1.0f32, 0.0f32];
        state.update(&x, 1.0, 0.0, false).expect("valid update"); // Success = 1.0
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
        let score_local = state.score(&x, 0.0, false).expect("valid score");
        let score_cloud = state.score(&x, 0.0, true).expect("valid score");
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
            state_discounted
                .update(&x, 1.0, 0.0, false)
                .expect("valid update");
            state_stiff
                .update(&x, 1.0, 0.0, false)
                .expect("valid update");
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
            state_discounted
                .update(&x, -1.0, 0.0, false)
                .expect("valid update");
            state_stiff
                .update(&x, -1.0, 0.0, false)
                .expect("valid update");
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
