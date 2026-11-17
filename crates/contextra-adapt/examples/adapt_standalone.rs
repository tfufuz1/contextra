fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "bandit-routing")]
    {
        use contextra_adapt::bandit::{BanditImplementation, BanditProfileState};
        use contextra_adapt::pid::PidController;
        use std::time::Duration;

        let mut pid = PidController::new(100.0, 50, 500, Some(100));
        let dt = Duration::from_millis(50);
        let next_pool = pid.update(dt, 120.0);
        assert!(next_pool >= 50);

        let dim = 16;
        let mut bandit = BanditProfileState::cold_start(dim, 0.5);
        bandit.implementation = BanditImplementation::ShermanMorrison;

        let x = vec![0.1f32; dim];
        let score = bandit.score(&x, 0.05, false)?;
        println!("Bandit score: {score}");

        bandit.update(&x, 1.0, 0.05, false)?;
    }
    #[cfg(not(feature = "bandit-routing"))]
    {
        use contextra_adapt::pid::PidController;
        use std::time::Duration;

        let mut pid = PidController::new(100.0, 50, 500, Some(100));
        let dt = Duration::from_millis(50);
        let next_pool = pid.update(dt, 120.0);
        assert!(next_pool >= 50);
        println!("PID next pool: {next_pool}");
    }

    Ok(())
}
