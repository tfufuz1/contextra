//! Standalone-Beispiel für contextra-adapt (deterministische Regelung & Bandits).

use contextra_adapt::pid::PidController;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut pid = PidController::new(100.0, 50, 500, Some(100));
    let next_pool = pid.update(Duration::from_millis(50), 120.0);
    assert!(next_pool >= 50);

    #[cfg(feature = "bandit-routing")]
    {
        use contextra_adapt::bandit::{BanditImplementation, BanditProfileState};
        let mut bandit = BanditProfileState::cold_start(4, 0.5);
        bandit.implementation = BanditImplementation::ShermanMorrison;
        let ctx = [0.1f32, 0.2, 0.3, 0.4];
        let score = bandit.score(&ctx, 0.05, false)?;
        bandit.update(&ctx, 1.0, 0.05, false)?;
        println!("LinUCB Score: {score:.4}, Pool: {next_pool}");
    }
    #[cfg(not(feature = "bandit-routing"))]
    println!("PID Pool: {next_pool}");

    Ok(())
}
