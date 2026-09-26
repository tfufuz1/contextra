//! Regularization-Induced Exploration (RIE) Greedy Mechanism (§10.4.1).
//!
//! Archetyp "Der faule Nutzer": Personalisierung ohne stochastische Exploration,
//! gesteuert durch Regularisierung und zeitliches Abklingen.

#![allow(clippy::needless_range_loop)]

use thiserror::Error;

/// Fehlerzustände für den RIE-Greedy-Personalismus-Mechanismus (§10.4.1).
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum RieGreedyError {
    /// Dimension des Kontext-Vektors stimmt nicht mit der konfigurierten Profil-Dimension überein.
    #[error("Embedding dimension mismatch: expected {expected}, actual {actual}")]
    DimensionMismatch {
        /// Erwartete Dimension.
        expected: usize,
        /// Tatsächliche Dimension.
        actual: usize,
    },
    /// Ein Wert ist NaN oder unendlich (non-finite).
    #[error("Non-finite numerical value encountered")]
    NonFinite,
    /// Ungültige Konfiguration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

impl From<RieGreedyError> for contextra_types::ContextraError {
    fn from(err: RieGreedyError) -> Self {
        match err {
            RieGreedyError::DimensionMismatch { expected, actual } => {
                contextra_types::ContextraError::InvalidInput(format!(
                    "Embedding dimension mismatch: expected {expected}, actual {actual}"
                ))
            }
            RieGreedyError::NonFinite => contextra_types::ContextraError::InvalidInput(
                "Non-finite numerical value encountered".to_string(),
            ),
            RieGreedyError::InvalidConfig(msg) => {
                contextra_types::ContextraError::InvalidInput(format!("Invalid configuration: {msg}"))
            }
        }
    }
}

/// Laufzeitzustand eines RIE-Greedy-Profils (§10.4.1).
///
/// Implementiert die deterministisch-gierige Personalisierung mit
/// zeitlich diskontierter Präzisionsmatrix $\Lambda_t$ und Informationsvektor $\eta_t$.
#[derive(Debug, Clone, PartialEq)]
pub struct RieGreedyProfile {
    /// Inverse Präzisionsmatrix $\Lambda^{-1}$ ($d \times d$, row-major).
    pub precision_inv: Vec<f32>,
    /// Informationsvektor $\eta$ ($d$ Elemente).
    pub info_vector: Vec<f32>,
    /// Feature-Dimension $d$.
    pub dim: usize,
    /// Regularisierungsparameter $\lambda > 0$.
    pub lambda: f32,
    /// Temporal-Decay-Faktor $\gamma \in (0, 1]$.
    pub gamma: f32,
}

impl RieGreedyProfile {
    /// Erstellt ein neues RIE-Greedy-Profil mit initialer Präzisionsmatrix $\Lambda_0 = \lambda I_d$.
    pub fn new(dim: usize, lambda: f32, gamma: f32) -> Self {
        let dim = dim.max(1);
        let lambda = if lambda.is_finite() && lambda > 0.0 {
            lambda
        } else {
            1.0
        };
        let gamma = if gamma.is_finite() {
            gamma.clamp(1e-4, 1.0)
        } else {
            1.0
        };

        let inv_lambda = 1.0 / lambda;
        let mut precision_inv = vec![0.0f32; dim * dim];
        for i in 0..dim {
            precision_inv[i * dim + i] = inv_lambda;
        }

        Self {
            precision_inv,
            info_vector: vec![0.0f32; dim],
            dim,
            lambda,
            gamma,
        }
    }

    /// Berechnet die Parameter-Schätzung $\hat{\mu}_t = \Lambda_t^{-1} \eta_t$.
    pub fn mu_hat(&self) -> Result<Vec<f32>, RieGreedyError> {
        let d = self.dim;
        let mut mu = vec![0.0f32; d];
        for i in 0..d {
            let row = &self.precision_inv[i * d..(i + 1) * d];
            let mut sum = 0.0f32;
            for j in 0..d {
                sum += row[j] * self.info_vector[j];
            }
            if !sum.is_finite() {
                return Err(RieGreedyError::NonFinite);
            }
            mu[i] = sum;
        }
        Ok(mu)
    }

    /// Berechnet die deterministisch-gierige Erwartungswert-Vorhersage $\hat{\mu}_t \cdot x$.
    pub fn predict(&self, context: &[f32]) -> Result<f32, RieGreedyError> {
        if context.len() != self.dim {
            return Err(RieGreedyError::DimensionMismatch {
                expected: self.dim,
                actual: context.len(),
            });
        }
        if context.iter().any(|&v| !v.is_finite()) {
            return Err(RieGreedyError::NonFinite);
        }

        let mu = self.mu_hat()?;
        let mut score = 0.0f32;
        for i in 0..self.dim {
            score += mu[i] * context[i];
        }

        if !score.is_finite() {
            return Err(RieGreedyError::NonFinite);
        }

        Ok(score)
    }

    /// Führt die Diskontierung VOR dem Sherman-Morrison Rang-1 Update durch (§10.4.1, Spec §9.2).
    ///
    /// 1. Diskontierung: $\Lambda^{-1} \leftarrow \gamma^{-1} \Lambda^{-1}$, $\eta \leftarrow \gamma \eta$
    /// 2. Rang-1-Update: Sherman-Morrison-Formel für $x x^T$, $\eta \leftarrow \eta + r x$
    pub fn update(&mut self, context: &[f32], reward: f32) -> Result<(), RieGreedyError> {
        if context.len() != self.dim {
            return Err(RieGreedyError::DimensionMismatch {
                expected: self.dim,
                actual: context.len(),
            });
        }
        if !reward.is_finite() || context.iter().any(|&v| !v.is_finite()) {
            return Err(RieGreedyError::NonFinite);
        }

        let d = self.dim;
        let gamma_inv = 1.0 / self.gamma;

        // Sicherungspunkt für atomaren Rollback im Fehlerfall
        let backup_precision_inv = self.precision_inv.clone();
        let backup_info_vector = self.info_vector.clone();

        // Schritt 1: Diskontierung VOR Update (Spec §9.2 / §10.4.1)
        for val in self.precision_inv.iter_mut() {
            *val *= gamma_inv;
        }
        for val in self.info_vector.iter_mut() {
            *val *= self.gamma;
        }

        // Schritt 2: Sherman-Morrison Update auf der diskontierten Matrix
        // v = \Lambda^{-1} x
        let mut v = vec![0.0f32; d];
        let mut xt_v = 0.0f32;
        for i in 0..d {
            let row = &self.precision_inv[i * d..(i + 1) * d];
            let mut row_dot = 0.0f32;
            for j in 0..d {
                row_dot += row[j] * context[j];
            }
            v[i] = row_dot;
            xt_v += context[i] * row_dot;
        }

        let denom = 1.0 + xt_v;
        if !denom.is_finite() || denom <= 1e-8 {
            self.precision_inv = backup_precision_inv;
            self.info_vector = backup_info_vector;
            return Err(RieGreedyError::NonFinite);
        }

        for i in 0..d {
            let vi = v[i];
            for j in 0..d {
                let vj = v[j];
                self.precision_inv[i * d + j] -= (vi * vj) / denom;
            }
        }

        for i in 0..d {
            self.info_vector[i] += reward * context[i];
        }

        // Finitheits-Prüfung aller aktualisierten Werte
        let valid = self.precision_inv.iter().all(|&val| val.is_finite())
            && self.info_vector.iter().all(|&val| val.is_finite());

        if !valid {
            self.precision_inv = backup_precision_inv;
            self.info_vector = backup_info_vector;
            return Err(RieGreedyError::NonFinite);
        }

        Ok(())
    }

    /// Berechnet die Spur der inversen Präzisionsmatrix $\text{Tr}(\Lambda^{-1})$.
    pub fn trace_precision_inv(&self) -> f32 {
        let d = self.dim;
        let mut trace = 0.0f32;
        for i in 0..d {
            trace += self.precision_inv[i * d + i];
        }
        trace
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rie_greedy_initialization() {
        let profile = RieGreedyProfile::new(4, 1.0, 0.9);
        assert_eq!(profile.dim, 4);
        assert_eq!(profile.lambda, 1.0);
        assert_eq!(profile.gamma, 0.9);
        assert_eq!(profile.precision_inv.len(), 16);
        assert_eq!(profile.info_vector.len(), 4);
        assert_eq!(profile.trace_precision_inv(), 4.0);
    }
}
