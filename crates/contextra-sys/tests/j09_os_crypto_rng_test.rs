//! J09 Security tests: OsCryptoRng and Clock/Rng bridge traits in contextra-sys.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_sys::{OsCryptoRng, SystemClock};
#[cfg(feature = "ports")]
use contextra_sys::{SeededRng, SystemRng};
use std::sync::Arc;

#[test]
fn test_os_crypto_rng_entropy_and_uniqueness() {
    let rng1 = OsCryptoRng::new();
    let rng2 = OsCryptoRng::new();

    let mut buf1 = [0u8; 32];
    let mut buf2 = [0u8; 32];

    rng1.fill_bytes(&mut buf1);
    rng2.fill_bytes(&mut buf2);

    assert_ne!(buf1, [0u8; 32]);
    assert_ne!(buf2, [0u8; 32]);
    assert_ne!(buf1, buf2, "Two OsCryptoRng draws MUST produce distinct entropy");

    let val1 = rng1.next_u64();
    let val2 = rng1.next_u64();
    assert_ne!(val1, val2);
}

#[cfg(feature = "ports")]
#[test]
fn test_clock_and_rng_ports_bridge_implementations() {
    use contextra_ports::{Clock, CryptoRng, EntropyRng, SeededRng as SeededRngPort};

    let sys_clock = SystemClock::new();
    let port_clock: &dyn Clock = &sys_clock;
    let ts1 = port_clock.now();
    let t1 = port_clock.monotonic_nanos();
    assert!(ts1.wall_ms > 0);
    let t2 = port_clock.monotonic_nanos();
    assert!(t2 >= t1);

    let os_rng = OsCryptoRng::new();
    let _crypto_ref: &dyn CryptoRng = &os_rng;
    let entropy_ref: &dyn EntropyRng = &os_rng;
    let mut buf = [0u8; 16];
    entropy_ref.fill_bytes(&mut buf);
    assert_ne!(buf, [0u8; 16]);

    let seeded1 = SeededRng::new(12345);
    let seeded2 = SeededRng::new(12345);
    let seeded_port1: &dyn SeededRngPort = &seeded1;
    let seeded_port2: &dyn SeededRngPort = &seeded2;
    assert_eq!(
        seeded_port1.next_u64(),
        seeded_port2.next_u64(),
        "Identical seeds MUST produce identical sequences"
    );

    let sys_rng = SystemRng::new();
    let sys_rng_port: &dyn SeededRngPort = &sys_rng;
    let _ = sys_rng_port.next_u64();
}

#[test]
fn test_clock_monotonicity_across_threads() {
    let clock = Arc::new(SystemClock::new());
    let mut handles = Vec::new();

    for _ in 0..8 {
        let clk = Arc::clone(&clock);
        handles.push(std::thread::spawn(move || {
            let mut prev = clk.monotonic_nanos();
            for _ in 0..1000 {
                let curr = clk.monotonic_nanos();
                assert!(curr >= prev, "SystemClock monotonic time went backwards");
                prev = curr;
            }
        }));
    }

    for h in handles {
        let _ = h.join();
    }
}
