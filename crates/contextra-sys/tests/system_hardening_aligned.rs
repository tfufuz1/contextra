//! Integration tests for contextra-sys determinism, hardening, and aligned buffers.

use contextra_sys::{harden_process, AlignedBuf, SystemClock, SystemIdGen, SystemRng, ALIGNMENT};

#[test]
fn test_integration_system_clock() {
    let clock = SystemClock::new();
    let wall = clock.now_unix_nanos();
    let mono1 = clock.monotonic_nanos();

    assert!(
        wall > 0,
        "now_unix_nanos should return positive nanoseconds"
    );
    std::thread::sleep(std::time::Duration::from_millis(1));
    let mono2 = clock.monotonic_nanos();
    assert!(
        mono2 > mono1,
        "monotonic_nanos should increase monotonically"
    );
}

#[test]
fn test_integration_system_rng_isolation() {
    let rng1 = SystemRng::new();
    let rng2 = SystemRng::new();

    let v1 = rng1.next_u64();
    let v2 = rng2.next_u64();
    assert_ne!(
        v1, v2,
        "Separate SystemRng instances should yield distinct random numbers"
    );

    let f = rng1.next_unit_f64();
    assert!(
        (0.0..1.0).contains(&f),
        "f64 should be in half-open interval [0, 1)"
    );

    let mut bytes = [0u8; 32];
    rng1.fill_bytes(&mut bytes);
    assert_ne!(bytes, [0u8; 32]);
}

#[test]
fn test_integration_system_id_gen() {
    let gen = SystemIdGen::new();
    let id1 = gen.next_id();
    let id2 = gen.next_id();
    assert_eq!(id2, id1 + 1, "IDs should increment monotonically");
}

#[test]
fn test_integration_harden_process() {
    let res = harden_process();
    assert!(res.is_ok(), "harden_process should return Ok result");
    let status = res.expect("status");
    if cfg!(target_os = "linux") {
        assert!(status.platform_supported);
        assert!(status.core_dump_disabled);
        assert!(status.dumpable_disabled);
    }
}

#[test]
fn test_integration_aligned_buf() {
    let mut buf = AlignedBuf::zeroed(128);
    assert_eq!(buf.len(), 128);
    assert_eq!(
        buf.as_raw_ptr() as usize % ALIGNMENT,
        0,
        "Memory address must be 64-byte aligned"
    );

    let payload = b"Contextra KV-Cache Block Data";
    buf.clear();
    buf.extend_from_slice(payload);
    assert_eq!(&buf[..], payload);
    assert_eq!(buf.as_raw_ptr() as usize % ALIGNMENT, 0);
}
