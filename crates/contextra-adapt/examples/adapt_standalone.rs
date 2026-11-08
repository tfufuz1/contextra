//! Autarkes Minimalbeispiel für `contextra-adapt`.
//!
//! Demonstriert:
//! 1. PID-Latenzregelung mit Anti-Windup zur dynamischen Anpassung eines Suchfensters.
//! 2. LinUCB-Bandit-State für Scoring und Parameteraktualisierung.

use contextra_adapt::bandit::{BanditImplementation, BanditProfileState};
use contextra_adapt::pid::PidController;
use std::time::Duration;

fn main() {
    println!("=== contextra-adapt: Autarkes Standalone-Beispiel ===");

    // 1. PID-Regler zur Latenzregelung initialisieren
    // Ziel: Latenz von 25.0 ms stabil halten, Pool-Größe zwischen 50 und 500, Startwert 100
    let mut pid = PidController::new(25.0, 50, 500, Some(100));

    println!(
        "[PID] Initialer Candidate-Pool: {:?}",
        pid.current_pool_size()
    );

    // Simulation: Latenz steigt über den Zielwert (z.B. 45.0 ms bei dt = 100ms)
    let adjusted_pool = pid.update(Duration::from_millis(100), 45.0);
    println!(
        "[PID] Nach Überlast (45.0 ms gemessen): Pool angepasst auf {}",
        adjusted_pool
    );
    assert!(adjusted_pool <= 100);

    // 2. LinUCB-Bandit zur Pfadauswahl initialisieren
    let feature_dim = 4;
    let mut bandit = BanditProfileState::cold_start(feature_dim, 0.2);
    bandit.implementation = BanditImplementation::ShermanMorrison;

    let features = vec![0.5, 0.1, 0.8, 0.2];
    let cost = 0.05;
    let is_cloud = false;

    let initial_score = bandit
        .score(&features, cost, is_cloud)
        .expect("Bandit scoring must succeed with valid features");
    println!("[Bandit] Initialer Score: {:.4}", initial_score);

    // Feedback-Update mit positivem Reward (z.B. 1.0)
    let reward = 1.0;
    bandit
        .update(&features, reward, cost, is_cloud)
        .expect("Bandit update must succeed");

    let updated_score = bandit
        .score(&features, cost, is_cloud)
        .expect("Bandit scoring must succeed");
    println!("[Bandit] Score nach positivem Reward: {:.4}", updated_score);
    assert!(updated_score > initial_score);

    println!("=== Beispiel erfolgreich beendet ===");
}
