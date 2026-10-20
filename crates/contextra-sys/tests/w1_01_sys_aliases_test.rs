#[allow(deprecated)]
use contextra_sys::{SeededRng, SplitMix64, SystemClock, SystemRng};

#[test]
#[allow(deprecated)]
fn sys_deprecated_aliases_still_compile() {
    let clock = SystemClock::new();
    let _nanos = clock.now_unix_nanos();

    let rng = SystemRng::new();
    let mut buf = [0u8; 8];
    rng.fill_bytes(&mut buf);

    let seeded_alias: SeededRng = SeededRng::new(42);
    let val1 = seeded_alias.next_u64();

    let split_mix = SplitMix64::new(42);
    let val2 = split_mix.next_u64();

    assert_eq!(
        val1, val2,
        "SeededRng alias and SplitMix64 must have identical behavior"
    );
}
