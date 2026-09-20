//! Off-Policy-Evaluation (IPS) für Bandit-Routing (§B.5.2.4 & §8.5).
//!
//! # Mathematische Spezifikation
//! Die kontrafaktische Evaluation einer neuen Policy π_new aus geloggten Daten der Policy π_old
//! erfolgt über Inverse Propensity Scoring (IPS):
//!
//! V_IPS(π_new) = (1 / t) * Σ_{i=1}^t [ r_i * I(π_new(x_i) == a_i) / P_{π_old}(a_i | x_i) ]
//!
//! Der Nenner wird auf `max(p, 0.01)` geklemmt, um Varianz-Explosionen zu dämpfen.
//! Komplexität pro `observe`-Aufruf: O(1).
//!
//! # Nebenläufigkeitsmodell
//! Das Lock-Freiheits-Prinzip ("lock-freier Akkumulator") wird hier durch alleinigen Besitz
//! (`&mut self`, kein internes Locking) realisiert. Es werden keine mutablen Zustandssperren
//! (`Mutex`/`RwLock`) innerhalb des Evaluators gehalten. Synchronisation bei nebenläufigem Zugriff
//! obliegt dem Aufrufer.

use serde::{Deserialize, Serialize};

/// Statistiken über verworrene/ungültige Samples der Off-Policy-Evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct OffPolicyStats {
    /// Anzahl verworfener Samples aufgrund ungültiger Propensity-Werte (NaN, < 0.0, > 1.0).
    pub discarded: u64,
}

/// Evaluator für Inverse Propensity Scoring (IPS) zur kontrafaktischen Off-Policy-Schätzung.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct OffPolicyEvaluator {
    cumulative_ips: f64,
    samples: u64,
    discarded: u64,
}

impl OffPolicyEvaluator {
    /// Erstellt einen neuen `OffPolicyEvaluator` mit leeren Akkumulatoren.
    ///
    /// *Hinweis zur Spezifikation:* Von der Spezifikation normativ nicht explizit genannt,
    /// jedoch für die Instanziierung des Typs erforderlich.
    pub fn new() -> Self {
        Self {
            cumulative_ips: 0.0,
            samples: 0,
            discarded: 0,
        }
    }

    /// Beobachtet ein geloggtes Event und akkumuliert den IPS-Wert.
    ///
    /// - `target_action`: Die Aktion, die von der zu evaluierenden Ziel-Policy gewählt worden wäre.
    /// - `logged_action`: Die von der Logging-Policy tatsächlich gewählte und im Log aufgezeichnete Aktion.
    /// - `propensity`: Wahrscheinlichkeit P(a_i | x_i) der Logging-Policy.
    /// - `reward`: Der beobachtete Reward r_i.
    ///
    /// # Verhalten bei ungültiger Propensity
    /// Falls `propensity` NaN, negativ oder größer 1.0 ist, wird der Sample verworfen
    /// (`discarded` inkrementiert, `samples` inkrementiert, `cumulative_ips` bleibt unverändert).
    pub fn observe(
        &mut self,
        target_action: u32,
        logged_action: u32,
        propensity: f32,
        reward: f32,
    ) {
        if propensity.is_nan() || propensity < 0.0 || propensity > 1.0 {
            self.discarded += 1;
            self.samples += 1;
            return;
        }

        if target_action == logged_action {
            let p = propensity.max(0.01);
            self.cumulative_ips += (reward / p) as f64;
        }

        self.samples += 1;
    }

    /// Liefert den aktuellen IPS-Schätzwert V_IPS = cumulative_ips / samples.
    ///
    /// Falls noch keine Samples beobachtet wurden (`samples == 0`), wird 0.0 zurückgegeben.
    pub fn estimate(&self) -> f64 {
        self.cumulative_ips / self.samples.max(1) as f64
    }

    /// Liefert die kumulative IPS-Summe Σ (r_i / max(p_i, 0.01)).
    pub fn cumulative_ips(&self) -> f64 {
        self.cumulative_ips
    }

    /// Liefert die Gesamtanzahl beobachteter Samples (inklusive verworfener).
    pub fn samples(&self) -> u64 {
        self.samples
    }

    /// Liefert die Anzahl verworfener Ungültig-Propensity-Samples.
    pub fn discarded(&self) -> u64 {
        self.discarded
    }

    /// Liefert die Off-Policy-Evaluator-Statistiken.
    pub fn stats(&self) -> OffPolicyStats {
        OffPolicyStats {
            discarded: self.discarded,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_case() {
        let mut evaluator = OffPolicyEvaluator::new();
        evaluator.observe(1, 1, 0.5, 1.0);

        // reward / propensity = 1.0 / 0.5 = 2.0
        assert!((evaluator.cumulative_ips() - 2.0).abs() < 1e-9);
        assert_eq!(evaluator.samples(), 1);
        assert_eq!(evaluator.discarded(), 0);
        assert!((evaluator.estimate() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_miss_case() {
        let mut evaluator = OffPolicyEvaluator::new();
        evaluator.observe(1, 2, 0.5, 1.0);

        assert!((evaluator.cumulative_ips() - 0.0).abs() < 1e-9);
        assert_eq!(evaluator.samples(), 1);
        assert_eq!(evaluator.discarded(), 0);
        assert!((evaluator.estimate() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_propensity_clamping() {
        let mut evaluator = OffPolicyEvaluator::new();
        // Propensity 0.001 muss auf 0.01 geklemmt werden
        evaluator.observe(1, 1, 0.001, 1.0);

        // reward / max(0.001, 0.01) = 1.0 / 0.01 = 100.0
        assert!((evaluator.cumulative_ips() - 100.0).abs() < 1e-9);
        assert_eq!(evaluator.samples(), 1);
        assert_eq!(evaluator.discarded(), 0);
        assert!((evaluator.estimate() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_invalid_propensity_discarded() {
        let mut evaluator = OffPolicyEvaluator::new();

        // NaN Propensity
        evaluator.observe(1, 1, f32::NAN, 1.0);
        // Negativ Propensity
        evaluator.observe(1, 1, -0.1, 1.0);
        // Propensity > 1.0
        evaluator.observe(1, 1, 1.5, 1.0);

        assert!((evaluator.cumulative_ips() - 0.0).abs() < 1e-9);
        assert_eq!(evaluator.samples(), 3);
        assert_eq!(evaluator.discarded(), 3);
        assert_eq!(evaluator.stats().discarded, 3);
        assert!((evaluator.estimate() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_convergence_on_synthetic_dataset() {
        // Synthetischer Datensatz:
        // logging policy: 2 actions (0 und 1) mit Propensity p = 0.5 zufällig gewählt.
        // target policy: wählt immer action 0.
        // True value of target policy (action 0): Mean reward = 0.8
        // Action 1: Mean reward = 0.2

        let mut evaluator = OffPolicyEvaluator::new();

        let n = 10_000;
        let mut rng_state: u64 = 42;

        for _ in 0..n {
            // LCG PRNG
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let logged_action = ((rng_state >> 33) & 1) as u32;
            let propensity = 0.5f32;

            // Simple reward generator
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = ((rng_state >> 33) & 0xFF) as f32 / 255.0;

            let reward = if logged_action == 0 {
                if val < 0.8 { 1.0 } else { 0.0 }
            } else {
                if val < 0.2 { 1.0 } else { 0.0 }
            };

            // Target policy always chooses action 0
            evaluator.observe(0, logged_action, propensity, reward);
        }

        let estimated = evaluator.estimate();
        // Target policy expected value is 0.8. IPS estimate should converge close to 0.8.
        assert!(
            (estimated - 0.8).abs() < 0.05,
            "Expected convergence near 0.8, got {estimated}"
        );
        assert_eq!(evaluator.samples(), n);
        assert_eq!(evaluator.discarded(), 0);
    }
}
