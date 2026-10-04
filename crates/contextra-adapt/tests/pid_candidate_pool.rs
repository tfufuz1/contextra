use contextra_adapt::{pid_regulated_candidate_pool, PidController};

#[test]
fn test_pid_regulated_candidate_pool_high_latency_monotonic_decrease() {
    let mut controller = PidController::new(150.0, 50, 200, Some(100));
    let mut prev_pool = controller.current_pool_size().unwrap_or(controller.min_pool_size);

    for _ in 0..10 {
        let new_pool = pid_regulated_candidate_pool(&mut controller, 300.0);
        assert!(
            new_pool <= prev_pool,
            "Pool size must monotonically decrease under high latency: prev={prev_pool}, new={new_pool}"
        );
        assert!(
            new_pool >= controller.min_pool_size,
            "Pool size must never fall below min_pool_size (>= 50): got {new_pool}"
        );
        assert!(
            new_pool >= 50,
            "Pool size must never fall below hard floor 50: got {new_pool}"
        );
        prev_pool = new_pool;
    }

    assert_eq!(
        pid_regulated_candidate_pool(&mut controller, 300.0),
        50,
        "Pool size should settle at hard minimum floor 50"
    );
}

#[test]
fn test_pid_regulated_candidate_pool_low_latency_monotonic_increase() {
    let mut controller = PidController::new(150.0, 50, 200, Some(100));
    let mut prev_pool = controller.current_pool_size().unwrap_or(controller.min_pool_size);

    for _ in 0..10 {
        let new_pool = pid_regulated_candidate_pool(&mut controller, 50.0);
        assert!(
            new_pool >= prev_pool,
            "Pool size must monotonically increase under low latency: prev={prev_pool}, new={new_pool}"
        );
        assert!(
            new_pool <= controller.max_pool_size,
            "Pool size must never exceed max_pool_size 200: got {new_pool}"
        );
        prev_pool = new_pool;
    }

    assert_eq!(
        pid_regulated_candidate_pool(&mut controller, 50.0),
        200,
        "Pool size should settle at maximum pool size 200"
    );
}

#[test]
fn test_pid_regulated_candidate_pool_non_finite_and_negative_latency() {
    let mut controller = PidController::new(150.0, 50, 200, Some(100));

    // First update with valid latency
    let initial_pool = pid_regulated_candidate_pool(&mut controller, 250.0);
    assert!(initial_pool >= 50 && initial_pool <= 200);

    // Non-finite inputs: NaN, INFINITY, NEG_INFINITY
    let nan_pool = pid_regulated_candidate_pool(&mut controller, f32::NAN);
    assert_eq!(nan_pool, initial_pool);

    let inf_pool = pid_regulated_candidate_pool(&mut controller, f32::INFINITY);
    assert_eq!(inf_pool, initial_pool);

    let neg_inf_pool = pid_regulated_candidate_pool(&mut controller, f32::NEG_INFINITY);
    assert_eq!(neg_inf_pool, initial_pool);

    // Negative latency input: should be handled safely within bounds without panic
    let neg_latency_pool = pid_regulated_candidate_pool(&mut controller, -50.0);
    assert!(
        neg_latency_pool >= controller.min_pool_size,
        "Negative latency result must be >= min_pool_size: got {neg_latency_pool}"
    );
    assert!(
        neg_latency_pool <= controller.max_pool_size,
        "Negative latency result must be <= max_pool_size: got {neg_latency_pool}"
    );
}

#[test]
fn test_pid_regulated_candidate_pool_determinism() {
    let mut controller_a = PidController::new(150.0, 50, 200, Some(100));
    let mut controller_b = PidController::new(150.0, 50, 200, Some(100));

    let observation_sequence = vec![
        150.0, 200.0, 350.0, 300.0, f32::NAN, 100.0, 50.0, -20.0, f32::INFINITY, 120.0,
    ];

    let results_a: Vec<usize> = observation_sequence
        .iter()
        .map(|&lat| pid_regulated_candidate_pool(&mut controller_a, lat))
        .collect();

    let results_b: Vec<usize> = observation_sequence
        .iter()
        .map(|&lat| pid_regulated_candidate_pool(&mut controller_b, lat))
        .collect();

    assert_eq!(
        results_a, results_b,
        "Identical observation sequences must yield identical candidate pool sequences"
    );
}
