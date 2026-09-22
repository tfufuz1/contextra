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

/// Randomisierte Logging-Policy für unverzerrte Datensammlung zur Off-Policy-Evaluation (§B.5.2.4 & §8.5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RandomizedLoggingPolicy {
    /// Exploration-Wahrscheinlichkeit epsilon (ε in [0.0, 1.0]).
    pub epsilon: f32,
    /// Anzahl verfügbarer Aktionen K (K >= 1).
    pub num_actions: u32,
}

impl RandomizedLoggingPolicy {
    /// Erstellt eine neue `RandomizedLoggingPolicy`.
    pub fn new(epsilon: f32, num_actions: u32) -> Self {
        Self {
            epsilon: epsilon.clamp(0.0, 1.0),
            num_actions: num_actions.max(1),
        }
    }

    /// Berechnet die Propensity P(a | x) für jede Aktion 0..K-1 bei gegebener greedy/target Aktion.
    pub fn compute_propensities(&self, greedy_action: u32) -> Vec<f32> {
        let k = self.num_actions as f32;
        let uniform_p = self.epsilon / k;
        let mut propensities = vec![uniform_p; self.num_actions as usize];

        if (greedy_action as usize) < propensities.len() {
            propensities[greedy_action as usize] += 1.0 - self.epsilon;
        }

        propensities
    }

    /// Wählt eine Aktion aus und gibt den (Aktionsindex, Propensity) Vektor/Tupel zurück.
    /// `random_sample` muss im Intervall [0.0, 1.0) liegen.
    pub fn select_action(&self, greedy_action: u32, random_sample: f32) -> (u32, f32) {
        let sample = random_sample.clamp(0.0, 0.999_999_9);
        let propensities = self.compute_propensities(greedy_action);

        if sample >= self.epsilon {
            let action = if (greedy_action as usize) < propensities.len() {
                greedy_action
            } else {
                0
            };
            (action, propensities[action as usize])
        } else {
            let action = ((sample / self.epsilon) * self.num_actions as f32) as u32;
            let clamped_action = action.min(self.num_actions - 1);
            (clamped_action, propensities[clamped_action as usize])
        }
    }
}

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
    fn test_randomized_logging_policy_propensities() {
        let policy = RandomizedLoggingPolicy::new(0.2, 4);
        let propensities = policy.compute_propensities(1);

        assert_eq!(propensities.len(), 4);
        // non-greedy actions get epsilon / 4 = 0.05
        assert!((propensities[0] - 0.05).abs() < 1e-6);
        // greedy action 1 gets 0.05 + 0.8 = 0.85
        assert!((propensities[1] - 0.85).abs() < 1e-6);
        assert!((propensities[2] - 0.05).abs() < 1e-6);
        assert!((propensities[3] - 0.05).abs() < 1e-6);

        // Sum of propensities must equal 1.0
        let sum: f32 = propensities.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_randomized_logging_policy_select_action() {
        let policy = RandomizedLoggingPolicy::new(0.2, 4);

        // Exploitation sample (>= epsilon 0.2)
        let (action_exp, p_exp) = policy.select_action(2, 0.5);
        assert_eq!(action_exp, 2);
        assert!((p_exp - 0.85).abs() < 1e-6);

        // Exploration sample (< epsilon 0.2)
        let (action_exp2, p_exp2) = policy.select_action(2, 0.05); // sample 0.05 / 0.2 * 4 = 1.0 -> action 1
        assert_eq!(action_exp2, 1);
        assert!((p_exp2 - 0.05).abs() < 1e-6);
    }

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
                if val < 0.8 {
                    1.0
                } else {
                    0.0
                }
            } else {
                if val < 0.2 {
                    1.0
                } else {
                    0.0
                }
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
