use contextra_testkit::ManualClock;
use std::time::{Duration, UNIX_EPOCH};

#[test]
fn test_j37_manual_clock_methods_integration() {
    let clock = ManualClock::new(1_000_000_000);

    // Initial state: 1 second since epoch
    assert_eq!(clock.now_nanos(), 1_000_000_000);
    assert_eq!(clock.now_secs_f64(), 1.0);
    assert_eq!(clock.now_system_time(), UNIX_EPOCH + Duration::from_secs(1));

    // Explicit set_nanos
    clock.set_nanos(5_500_000_000);
    assert_eq!(clock.now_nanos(), 5_500_000_000);
    assert_eq!(clock.now_secs_f64(), 5.5);
    assert_eq!(
        clock.now_system_time(),
        UNIX_EPOCH + Duration::from_millis(5_500)
    );
}
