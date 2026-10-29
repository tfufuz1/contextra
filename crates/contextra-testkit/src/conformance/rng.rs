use super::failure::{ConformanceFailure, ConformanceResult};
use contextra_ports::Rng;

/// Runs the complete conformance test suite for an [`Rng`] implementation.
///
/// # Contract Requirements Tested
/// 1. `next_u64()` produces changing output values (non-constant).
/// 2. `fill_bytes()` fills the destination slice completely.
/// 3. `next_unit_f64()` returns values in the half-open interval $[0.0, 1.0)$.
pub fn run_rng_suite<R: Rng>(rng: &R) -> ConformanceResult {
    let suite = "Rng";

    // Test 1: Non-constant u64 sequence
    let u1 = rng.next_u64();
    let u2 = rng.next_u64();
    let u3 = rng.next_u64();
    if u1 == u2 && u2 == u3 {
        return Err(ConformanceFailure::new(
            suite,
            "next_u64_non_constant",
            format!("3 consecutive draws returned identical u64: {u1}"),
        ));
    }

    // Test 2: fill_bytes
    let mut buf = [0u8; 16];
    rng.fill_bytes(&mut buf);
    if buf == [0u8; 16] {
        return Err(ConformanceFailure::new(
            suite,
            "fill_bytes_non_zero",
            "fill_bytes produced 16 all-zero bytes",
        ));
    }

    // Test 3: next_unit_f64 in [0.0, 1.0)
    for _ in 0..100 {
        let val = rng.next_unit_f64();
        if !(0.0..1.0).contains(&val) {
            return Err(ConformanceFailure::new(
                suite,
                "next_unit_f64_bounds",
                format!("next_unit_f64 returned {val}, outside [0.0, 1.0)"),
            ));
        }
    }

    Ok(())
}
