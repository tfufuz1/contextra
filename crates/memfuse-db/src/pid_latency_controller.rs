// FILE-CONTEXT
// ZWECK: PID-Latenzregler und LatencyBudgetGuard für dynamische Multi-Step- retrieval Parameter-Skalierung.
// INVARIANTEN: Anti-Windup Clamping; Ausgabewerte strikt geklemmt [min_scale, max_scale] (nie <= 0, default min 0.3); NaN/Inf-Sicherheit.
// NICHT-OFFENSICHTLICH: Konservativer Target-Latenz Default ist 100.0ms (konfigurierbar). BudgetGuard liefert Zero-Panic Graceful Stop.
// STAND: TS:2026-09-13T20:00:00Z (SESSION: pid_latency_controller)

use std::time::{Duration, Instant};

/// Konservativer Default für die Ziel-P95-Latenz in Millisekunden.
/// Gemäß Architekturentscheidung in `implementation_plan-db2.md` wird 100.0ms als konservativer Default
/// gewählt und über `PidConfig` bzw. Konstruktor-Argumente frei anpassbar gemacht.
pub const DEFAULT_TARGET_LATENCY_MS: f64 = 100.0;

/// Konfiguration für den PID-Latenzregler.
#[derive(Debug, Clone)]
pub struct PidConfig {
    /// Ziel-Latenz in Millisekunden (Default: 100.0ms).
    pub target_latency_ms: f64,
    /// Proportional-Verstärkung (Kp). Default: 0.005.
    pub kp: f64,
    /// Integral-Verstärkung (Ki). Default: 0.0005.
    pub ki: f64,
    /// Differential-Verstärkung (Kd). Default: 0.001.
    pub kd: f64,
    /// Maximaler absoluter Wert für den akkumulierten Integrator-Fehler (Anti-Windup). Default: 50.0.
    pub max_integral: f64,
    /// Minimaler Skalierungsfaktor (APM-RECALL-COLLAPSE Protection). Default: 0.3.
    pub min_scale: f64,
    /// Maximaler Skalierungsfaktor. Default: 1.0.
    pub max_scale: f64,
    /// Glättungsfaktor (EMA) für Latenzmessungen in [0.0, 1.0]. Default: 0.3.
    pub ema_alpha: f64,
}

impl Default for PidConfig {
    fn default() -> Self {
        Self {
            target_latency_ms: DEFAULT_TARGET_LATENCY_MS,
            kp: 0.005,
            ki: 0.0005,
            kd: 0.001,
            max_integral: 50.0,
            min_scale: 0.3,
            max_scale: 1.0,
            ema_alpha: 0.3,
        }
    }
}

/// Klassischer PID-Latenzregler für Retrieval-Parameter (`k_pool`, `max_hops`).
#[derive(Debug, Clone)]
pub struct PidLatencyController {
    config: PidConfig,
    integral: f64,
    previous_error: f64,
    ema_latency: Option<f64>,
}

impl PidLatencyController {
    /// Erstellt einen neuen PID-Latenzregler mit Standard- oder benutzerdefinierter Konfiguration.
    pub fn new(config: PidConfig) -> Self {
        Self {
            config,
            integral: 0.0,
            previous_error: 0.0,
            ema_latency: None,
        }
    }

    /// Erstellt einen Regler mit Angabe einer spezifischen Ziel-Latenz in Millisekunden.
    pub fn with_target_latency(target_latency_ms: f64) -> Self {
        let config = PidConfig {
            target_latency_ms: target_latency_ms.max(1.0),
            ..Default::default()
        };
        Self::new(config)
    }

    /// Berechnet den Skalierungsfaktor basierend auf der beobachteten Latenz in Millisekunden.
    ///
    /// Gibt einen Wert in `[min_scale, max_scale]` zurück (Standard: `[0.3, 1.0]`).
    /// - Wenn `observed_latency_ms > target_latency_ms`, sinkt der Faktor (Reduktion von k_pool/max_hops).
    /// - Wenn `observed_latency_ms < target_latency_ms`, steigt der Faktor in Richtung 1.0.
    ///
    /// APM-INTEGRATOR-WINDUP: Der Integrator-Anteil ist strikt auf `[-max_integral, max_integral]` geklemmt.
    /// APM-RECALL-COLLAPSE: Der Rückgabewert ist strikt auf `[min_scale, max_scale]` geklemmt (nie < min_scale, nie <= 0).
    pub fn compute_adjustment(&mut self, observed_latency_ms: f64) -> f64 {
        // Safe handling of NaN / Inf / non-positive inputs
        let latency = if observed_latency_ms.is_nan() || observed_latency_ms.is_infinite() {
            self.config.target_latency_ms
        } else {
            observed_latency_ms.max(0.0)
        };

        // Update Exponential Moving Average (EMA)
        let smoothed_latency = match self.ema_latency {
            Some(prev_ema) => {
                let alpha = self.config.ema_alpha.clamp(0.01, 1.0);
                alpha * latency + (1.0 - alpha) * prev_ema
            }
            None => latency,
        };
        self.ema_latency = Some(smoothed_latency);

        // Error = Target - Smoothed Observed Latency
        // Positive error -> latency below budget -> can increase scale up to max_scale (1.0)
        // Negative error -> latency over budget -> decrease scale down to min_scale (0.3)
        let error = self.config.target_latency_ms - smoothed_latency;

        // Anti-Windup Clamping on Integrator
        let max_int = self.config.max_integral.abs().max(1.0);
        self.integral = (self.integral + error).clamp(-max_int, max_int);

        // Derivative term
        let derivative = error - self.previous_error;
        self.previous_error = error;

        // Calculate control signal delta around baseline 1.0
        let p_term = self.config.kp * error;
        let i_term = self.config.ki * self.integral;
        let d_term = self.config.kd * derivative;
        let control_delta = p_term + i_term + d_term;

        let raw_scale = 1.0 + control_delta;

        // Strict Clamping for Output Scale (APM-RECALL-COLLAPSE & Zero-Panic)
        let min_s = self.config.min_scale.clamp(0.01, 1.0);
        let max_s = self.config.max_scale.clamp(min_s, 2.0);

        raw_scale.clamp(min_s, max_s)
    }

    /// Setzt den Zustand des Reglers zurück (z. B. bei Konfigurationsänderung).
    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.previous_error = 0.0;
        self.ema_latency = None;
    }
}

/// Simple, typsichere Struktur zur Latenz-Budget-Überwachung während einer Multi-Step Execution.
///
/// Garantiert Zero-Panic Graceful Stop: Bei Budget-Überschreitung gibt die Engine `Ok(PartialResult)` zurück.
#[derive(Debug, Clone)]
pub struct LatencyBudgetGuard {
    start_time: Instant,
    budget: Option<Duration>,
}

impl LatencyBudgetGuard {
    /// Erstellt einen neuen Budget Guard mit optionaler Latenz-Budget Dauer.
    pub fn new(budget_ms: Option<f64>) -> Self {
        let budget = budget_ms
            .filter(|b| !b.is_nan() && *b > 0.0)
            .map(|b| Duration::from_secs_f64(b / 1000.0));
        Self {
            start_time: Instant::now(),
            budget,
        }
    }

    /// Prüft ob das gesetzte Latenz-Budget überschritten wurde.
    pub fn is_exceeded(&self) -> bool {
        match self.budget {
            Some(b) => self.start_time.elapsed() >= b,
            None => false,
        }
    }

    /// Liefert die bisher verstrichene Ausführungszeit in Millisekunden.
    pub fn elapsed_ms(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64() * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_latency_controller_increasing_latency() {
        let mut controller = PidLatencyController::with_target_latency(100.0);

        // Standard 100ms baseline should stay near 1.0
        let scale_100 = controller.compute_adjustment(100.0);
        assert!((scale_100 - 1.0).abs() < 0.05);

        // High latency (200ms) over target (100ms) -> scale must decrease monotonically
        let scale_150 = controller.compute_adjustment(150.0);
        let scale_200 = controller.compute_adjustment(200.0);
        let scale_250 = controller.compute_adjustment(250.0);

        assert!(scale_150 <= scale_100);
        assert!(scale_200 <= scale_150);
        assert!(scale_250 <= scale_200);

        // Must never drop below min_scale (default 0.3)
        for _ in 0..20 {
            let scale = controller.compute_adjustment(500.0);
            assert!(scale >= 0.3);
            assert!(scale <= 1.0);
        }
    }

    #[test]
    fn test_pid_latency_controller_decreasing_latency() {
        let mut controller = PidLatencyController::with_target_latency(100.0);

        // Drive controller into low-scale state with high latency
        for _ in 0..10 {
            controller.compute_adjustment(200.0);
        }
        let scale_low = controller.compute_adjustment(200.0);
        assert!(scale_low < 0.7);

        // Recovery with low latency (30ms) -> scale must increase towards 1.0
        let scale_recover1 = controller.compute_adjustment(30.0);
        let scale_recover2 = controller.compute_adjustment(30.0);

        assert!(scale_recover1 >= scale_low);
        assert!(scale_recover2 >= scale_recover1);
    }

    #[test]
    fn test_pid_anti_windup_clamping() {
        let mut controller = PidLatencyController::with_target_latency(100.0);

        // Extreme latency for 1000 iterations
        for _ in 0..1000 {
            controller.compute_adjustment(1000.0);
        }

        assert!(controller.integral >= -50.0 && controller.integral <= 50.0);

        // Immediate drop back to normal latency should recover within a few steps (no infinite windup delay)
        let scale1 = controller.compute_adjustment(50.0);
        let scale2 = controller.compute_adjustment(50.0);
        assert!(scale2 > scale1);
    }

    #[test]
    fn test_pid_nan_and_inf_safety() {
        let mut controller = PidLatencyController::with_target_latency(100.0);

        let scale_nan = controller.compute_adjustment(f64::NAN);
        let scale_inf = controller.compute_adjustment(f64::INFINITY);
        let scale_neg = controller.compute_adjustment(-50.0);

        assert!(scale_nan >= 0.3 && scale_nan <= 1.0);
        assert!(scale_inf >= 0.3 && scale_inf <= 1.0);
        assert!(scale_neg >= 0.3 && scale_neg <= 1.0);
    }

    #[test]
    fn test_latency_budget_guard() {
        let guard = LatencyBudgetGuard::new(Some(10.0));
        assert!(!guard.is_exceeded());
        assert!(guard.elapsed_ms() >= 0.0);

        std::thread::sleep(Duration::from_millis(15));
        assert!(guard.is_exceeded());

        let unconstrained_guard = LatencyBudgetGuard::new(None);
        assert!(!unconstrained_guard.is_exceeded());
    }
}
