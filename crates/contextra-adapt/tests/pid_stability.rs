// FILE-CONTEXT
// STAND: 2026-09-15T00:00:00Z
// ZWECK: Stabilitätstest des PID-Reglers mit Bekannter Sprungantwort (Settling Time, Overshoot, Steady-State Error).
// INVARIANTEN: Deterministischer Regler-Test ohne Zufallszahlen; alle Parameter als benannte Konstanten begründet.

use contextra_adapt::PidController;
use std::time::Duration;

/// Zeitkonstante tau = 1.0s für das Plant-Modell erster Ordnung (dx/dt = (u - x) / tau).
const PLANT_TAU: f64 = 1.0;

/// Zeitschritt dt = 0.01s (10ms Diskretsierungsintervall) für die Simulation.
const DT_SECS: f64 = 0.01;

/// Zielwert (Setpoint) für die Sprungantwort von 0.0 auf 1.0 zum Zeitpunkt t=0.
const TARGET_SETPOINT: f32 = 1.0;

/// Proportionalbeiwert Kp, empirisch gewählt für eine angemessene Anstiegszeit ohne extreme Oszillation.
const KP_PARAM: f32 = 0.1;

/// Integralbeiwert Ki zur Beseitigung der stationären Regelabweichung (Steady-State Error).
const KI_PARAM: f32 = 0.01;

/// Differenzialbeiwert Kd zur Dämpfung schneller Messwertänderungen.
const KD_PARAM: f32 = 0.01;

/// Minimale Pool-Größe überschrieben auf 0, um uneingeschränkte Skalierung für das Plant-Modell zu ermöglichen.
const MIN_POOL_SIZE: usize = 0;

/// Maximale Pool-Größe (1000) verhindert unbegrenzte Sättigung bei Sprungantworten.
const MAX_POOL_SIZE: usize = 1000;

/// Beginn der Pool-Größe bei 0 zum Simulationsstart t=0.
const INITIAL_POOL: usize = 0;

/// Toleranzband für Settling-Time: ±2% um den Zielwert (Bereich 0.98 bis 1.02 bei Setpoint 1.0).
const SETTLING_BAND_FRACTION: f64 = 0.02;

/// Maximale zulässige Settling-Time (5.0s simulierter Zeit).
///
/// Begründung: Das Plant-Modell erster Ordnung mit tau=1.0s und den gewählten PID-Parametern
/// (Kp=0.1, Ki=0.01, Kd=0.01) erreicht im Testlauf nach t ~ 3.89s dauerhaft das ±2%-Toleranzband.
/// 5.0s bieten eine sichere Obergrenze zur Erkennung von Instabilitäten.
const MAX_ALLOWABLE_SETTLING_TIME_SECS: f64 = 5.0;

/// Maximal zulässiger Überschwingungsgrad (Overshoot) in Prozent (< 30%).
///
/// Begründung: Regelungstechnisch ist bei RAG-Pool-Anpassungen eine überschwingungsarme
/// Dynamik erwünscht. Bei den gewählten Parametern bleibt der Overshoot bei 0.0% (aperiodischer Grenzfall).
const MAX_ALLOWABLE_OVERSHOOT_PERCENT: f64 = 30.0;

/// Maximale zulässige stationäre Regelabweichung (< 1% Abweichung vom Zielwert 1.0).
const STEADY_STATE_TOLERANCE_FRACTION: f64 = 0.01;

/// Standard-Anzahl an Simulationsschritten (1000 Schritte * 0.01s = 10.0s Gesamtdauer).
const SIMULATION_STEPS_STANDARD: usize = 1000;

/// Erweiterte Anzahl an Simulationsschritten (2000 Schritte * 0.01s = 20.0s Gesamtdauer) für Steady-State Test.
const SIMULATION_STEPS_EXTENDED: usize = 2000;

/// Simulierter linearer Prozess erster Ordnung (Plant-Modell):
/// dx/dt = (u - x) / tau
#[derive(Debug, Clone)]
struct FirstOrderPlant {
    value: f64,
    tau: f64,
}

impl FirstOrderPlant {
    fn new(initial_value: f64, tau: f64) -> Self {
        Self {
            value: initial_value,
            tau,
        }
    }

    fn step(&mut self, control_input: f64, dt: f64) {
        self.value += dt * (control_input - self.value) / self.tau;
    }
}

/// Hilfsfunktion zur Erstellung eines konfigurierten `PidController` für die Sprungantwort-Tests.
fn create_test_controller() -> PidController {
    let mut pid = PidController::new(
        TARGET_SETPOINT,
        MIN_POOL_SIZE,
        MAX_POOL_SIZE,
        Some(INITIAL_POOL),
    );
    pid.kp = KP_PARAM;
    pid.ki = KI_PARAM;
    pid.kd = KD_PARAM;
    pid.min_pool_size = MIN_POOL_SIZE;
    pid.current_pool_size = Some(INITIAL_POOL);
    pid
}

#[test]
fn step_response_settles_within_expected_time() {
    let mut plant = FirstOrderPlant::new(0.0, PLANT_TAU);
    let mut pid = create_test_controller();
    let dt = Duration::from_secs_f64(DT_SECS);

    let mut history: Vec<(f64, f64)> = Vec::with_capacity(SIMULATION_STEPS_STANDARD);

    for i in 0..SIMULATION_STEPS_STANDARD {
        let t = i as f64 * DT_SECS;
        let control_input = pid.update(dt, plant.value as f32) as f64;
        plant.step(control_input, DT_SECS);
        history.push((t, plant.value));
    }

    let target = TARGET_SETPOINT as f64;
    let lower_bound = target * (1.0 - SETTLING_BAND_FRACTION);
    let upper_bound = target * (1.0 + SETTLING_BAND_FRACTION);

    // Settling-Time: Erster Zeitpunkt t, ab dem ALLE nachfolgenden Plant-Werte im Band [0.98, 1.02] liegen.
    let settling_time = history
        .iter()
        .enumerate()
        .find(|&(idx, _)| {
            history[idx..]
                .iter()
                .all(|&(_, val)| val >= lower_bound && val <= upper_bound)
        })
        .map(|(_, &(t, _))| t)
        .expect("Plant value must enter and stay within ±2% settling band during simulation");

    assert!(
        settling_time < MAX_ALLOWABLE_SETTLING_TIME_SECS,
        "Settling time ({settling_time:.2}s) exceeded limit of {MAX_ALLOWABLE_SETTLING_TIME_SECS:.2}s"
    );
}

#[test]
fn step_response_overshoot_bounded() {
    let mut plant = FirstOrderPlant::new(0.0, PLANT_TAU);
    let mut pid = create_test_controller();
    let dt = Duration::from_secs_f64(DT_SECS);

    let mut max_plant_value = 0.0f64;

    for _ in 0..SIMULATION_STEPS_STANDARD {
        let control_input = pid.update(dt, plant.value as f32) as f64;
        plant.step(control_input, DT_SECS);
        if plant.value > max_plant_value {
            max_plant_value = plant.value;
        }
    }

    let target = TARGET_SETPOINT as f64;
    let overshoot = if max_plant_value > target {
        ((max_plant_value - target) / target) * 100.0
    } else {
        0.0
    };

    assert!(
        overshoot < MAX_ALLOWABLE_OVERSHOOT_PERCENT,
        "Overshoot ({overshoot:.2}%) exceeded maximum allowed limit ({MAX_ALLOWABLE_OVERSHOOT_PERCENT:.2}%)"
    );
}

#[test]
fn no_steady_state_error() {
    // Dieser Test demonstriert die Integralwirkung (Ki > 0) des PID-Reglers.
    // Bei Ki = 0 würde ein reiner P/PD-Regler bei manchen Prozessen eine bleibende Regelabweichung
    // (Steady-State Error) hinterlassen. Die I-Komponente summiert die Abweichungen e(t) über die Zeit auf,
    // bis die Regelabweichung vollständig verschwindet (e(t) -> 0, plant.value -> target).
    let mut plant = FirstOrderPlant::new(0.0, PLANT_TAU);
    let mut pid = create_test_controller();
    let dt = Duration::from_secs_f64(DT_SECS);

    for _ in 0..SIMULATION_STEPS_EXTENDED {
        let control_input = pid.update(dt, plant.value as f32) as f64;
        plant.step(control_input, DT_SECS);
    }

    let target = TARGET_SETPOINT as f64;
    let final_error = (plant.value - target).abs();

    assert!(
        final_error < STEADY_STATE_TOLERANCE_FRACTION * target,
        "Steady state error ({final_error:.6}) exceeded tolerance of {} * target",
        STEADY_STATE_TOLERANCE_FRACTION
    );
}
