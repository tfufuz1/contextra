//! System determinism ports and process-level default sources for contextra-sys (Ring 0).
//!
//! Provides default system implementations for time, pseudorandom number generation,
//! and unique ID generation without external crate dependencies.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static ENTROPY_COUNTER: AtomicU64 = AtomicU64::new(1);
const GOLDEN_RATIO_64: u64 = 0x9e37_79b9_7f4a_7c15;

// TODO(WP-0 / PR-1 Schritt 1 / B0-d): Bridge-Trait implementieren: `impl Clock for SystemClock`.
// Befund B0-d: `SystemClock` existiert doppelt in `contextra-ports` und `contextra-sys`. Die Version
// in `contextra-sys` implementiert bislang nicht den `Clock`-Trait aus `contextra-ports`.
// WICHTIG (Ring-DAG-Hierarchie): Da `contextra-sys` (Ring 0) nicht unkontrolliert von `contextra-ports`
// (Ring 1) abhängen darf, soll die Trait-Implementierung direkt in `contextra-sys` über ein optionales
// Feature `ports` bereitgestellt werden (`#[cfg(feature = "ports")] impl contextra_ports::Clock for SystemClock`).
// Anschließend Consumer-Dateien von `contextra_ports::SystemClock` auf `contextra_sys::SystemClock` migrieren.
/// Production system clock implementation backed by [`SystemTime`] and [`Instant`].
#[derive(Debug, Clone, Copy)]
pub struct SystemClock {
    start_instant: Instant,
}

impl SystemClock {
    /// Creates a new [`SystemClock`] with current instant as monotonic baseline.
    // DETERMINISM-EXCEPTION: SystemClock port implementation in contextra-sys
    #[allow(clippy::disallowed_methods)]
    pub fn new() -> Self {
        Self {
            start_instant: Instant::now(),
        }
    }

    /// Returns the current wall-clock time as nanoseconds since the Unix epoch.
    ///
    /// Returns `0` if system time is before the Unix epoch.
    // DETERMINISM-EXCEPTION: SystemClock port implementation in contextra-sys
    #[allow(clippy::cast_possible_truncation, clippy::disallowed_methods)]
    pub fn now_unix_nanos(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
            .unwrap_or(0)
    }

    /// Returns nanoseconds elapsed since monotonic baseline reference point.
    // DETERMINISM-EXCEPTION: SystemClock port implementation in contextra-sys
    #[allow(clippy::cast_possible_truncation, clippy::disallowed_methods)]
    pub fn monotonic_nanos(&self) -> u64 {
        u64::try_from(
            Instant::now()
                .saturating_duration_since(self.start_instant)
                .as_nanos(),
        )
        .unwrap_or(u64::MAX)
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "ports")]
impl contextra_ports::Clock for SystemClock {
    fn now(&self) -> contextra_ports::DeterministicTimestamp {
        let wall_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
            .unwrap_or(0);
        let mono_ctr = u64::try_from(
            Instant::now()
                .saturating_duration_since(self.start_instant)
                .as_nanos(),
        )
        .unwrap_or(u64::MAX);
        contextra_ports::DeterministicTimestamp::new(wall_ms, mono_ctr)
    }

    fn monotonic_ticks(&self) -> u64 {
        self.monotonic_nanos()
    }

    fn now_unix_nanos(&self) -> u64 {
        SystemClock::now_unix_nanos(self)
    }

    fn monotonic_nanos(&self) -> u64 {
        SystemClock::monotonic_nanos(self)
    }
}

/// A fast, thread-safe SplitMix64 pseudorandom generator seeded with process entropy.
#[derive(Debug)]
pub struct SeededRng {
    state: AtomicU64,
}

impl SeededRng {
    /// Creates a new [`SeededRng`] initialized with the given 64-bit seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: AtomicU64::new(seed),
        }
    }

    /// Creates a new [`SeededRng`] seeded from process/runtime entropy.
    pub fn from_entropy() -> Self {
        let counter = ENTROPY_COUNTER.fetch_add(1, Ordering::Relaxed);

        let mut hasher1 = RandomState::new().build_hasher();
        hasher1.write_u64(counter);
        let h1 = hasher1.finish();

        let mut hasher2 = RandomState::new().build_hasher();
        hasher2.write_u64(h1);
        let h2 = hasher2.finish();

        let seed = h1 ^ h2.rotate_left(32) ^ GOLDEN_RATIO_64.wrapping_mul(counter);
        Self::new(seed)
    }

    /// Generates the next pseudorandom 64-bit unsigned integer.
    pub fn next_u64(&self) -> u64 {
        let z = self
            .state
            .fetch_add(GOLDEN_RATIO_64, Ordering::SeqCst)
            .wrapping_add(GOLDEN_RATIO_64);
        let mut z = z;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Fills `dest` with pseudorandom bytes.
    pub fn fill_bytes(&self, mut dest: &mut [u8]) {
        while !dest.is_empty() {
            let val = self.next_u64();
            let bytes = val.to_le_bytes();
            let len = dest.len().min(8);
            dest[..len].copy_from_slice(&bytes[..len]);
            dest = &mut dest[len..];
        }
    }

    /// Returns a pseudorandom `f64` in `[0.0, 1.0)`.
    pub fn next_unit_f64(&self) -> f64 {
        let mantissa = self.next_u64() >> 11;
        (mantissa as f64) * (1.0 / (1u64 << 53) as f64)
    }
}

// TODO(WP-0 / PR-1 Schritt 2): Bridge-Trait implementieren: `impl Rng for SystemRng`.
// `SystemRng` soll das RNG-Port-Interface (z.B. `contextra_ports::Rng` bzw. `EntropyRng`/`SeededRng`)
// unter `#[cfg(feature = "ports")]` implementieren, um Vereinheitlichung und Port-Konsolidierung
// unter Einhaltung der Ring-DAG-Hierarchie zu sichern.
/// Standard non-cryptographic system RNG seeded from process entropy.
#[derive(Debug)]
pub struct SystemRng {
    inner: SeededRng,
}

impl SystemRng {
    /// Creates a new [`SystemRng`].
    pub fn new() -> Self {
        Self {
            inner: SeededRng::from_entropy(),
        }
    }

    /// Generates the next pseudorandom 64-bit unsigned integer.
    pub fn next_u64(&self) -> u64 {
        self.inner.next_u64()
    }

    /// Fills `dest` with pseudorandom bytes.
    pub fn fill_bytes(&self, dest: &mut [u8]) {
        self.inner.fill_bytes(dest);
    }

    /// Returns a pseudorandom `f64` in `[0.0, 1.0)`.
    pub fn next_unit_f64(&self) -> f64 {
        self.inner.next_unit_f64()
    }
}

impl Default for SystemRng {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "ports")]
impl contextra_ports::SeededRng for SeededRng {
    fn next_u64(&self) -> u64 {
        SeededRng::next_u64(self)
    }

    fn fill_bytes(&self, dest: &mut [u8]) {
        SeededRng::fill_bytes(self, dest);
    }

    fn next_unit_f64(&self) -> f64 {
        SeededRng::next_unit_f64(self)
    }
}

#[cfg(feature = "ports")]
impl contextra_ports::SeededRng for SystemRng {
    fn next_u64(&self) -> u64 {
        SystemRng::next_u64(self)
    }

    fn fill_bytes(&self, dest: &mut [u8]) {
        SystemRng::fill_bytes(self, dest);
    }

    fn next_unit_f64(&self) -> f64 {
        SystemRng::next_unit_f64(self)
    }
}

/// Atomic unique 64-bit ID generator initialized from system wall-clock nanoseconds.
#[derive(Debug)]
pub struct SystemIdGen {
    next: AtomicU64,
}

impl SystemIdGen {
    /// Creates a new [`SystemIdGen`] starting at wall-clock nanoseconds.
    // DETERMINISM-EXCEPTION: SystemIdGen port implementation in contextra-sys
    #[allow(clippy::disallowed_methods)]
    pub fn new() -> Self {
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
            .unwrap_or(1);
        Self {
            next: AtomicU64::new(start),
        }
    }

    /// Returns the next unique 64-bit ID.
    pub fn next_id(&self) -> u64 {
        self.next.fetch_add(1, Ordering::SeqCst)
    }
}

impl Default for SystemIdGen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_clock_monotonic_and_wall() {
        let clock = SystemClock::new();
        let t1 = clock.monotonic_nanos();
        let wall1 = clock.now_unix_nanos();
        assert!(wall1 > 0);
        std::thread::sleep(std::time::Duration::from_millis(1));
        let t2 = clock.monotonic_nanos();
        assert!(t2 >= t1);
    }

    #[test]
    fn test_system_rng_isolation() {
        let rng1 = SystemRng::new();
        let rng2 = SystemRng::new();

        let vals1: Vec<u64> = (0..4).map(|_| rng1.next_u64()).collect();
        let vals2: Vec<u64> = (0..4).map(|_| rng2.next_u64()).collect();

        assert_ne!(
            vals1, vals2,
            "SystemRng instances generated identical numbers"
        );

        let f = rng1.next_unit_f64();
        assert!((0.0..1.0).contains(&f));

        let mut buf = [0u8; 16];
        rng1.fill_bytes(&mut buf);
        assert_ne!(buf, [0u8; 16]);
    }

    #[test]
    fn test_system_id_gen_uniqueness() {
        let gen = SystemIdGen::new();
        let id1 = gen.next_id();
        let id2 = gen.next_id();
        assert!(id1 > 0);
        assert_eq!(id2, id1 + 1);
    }
}
