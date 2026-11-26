//! Hybrid Logical Clock (SPEC-02 §2.7; IMPL-01 §3 Regel 2).
//!
//! Layout of the 64-bit value: 48 bits of wall-clock milliseconds in the
//! high part, 16 bits of logical counter in the low part. Ordering of the raw
//! `u64` equals ordering of `(ms, logical)`.
//!
//! The clock never reads the system time itself. Callers pass `wall_ms`
//! obtained from the `Clock` port (`contextra-ports`), which keeps the
//! type deterministic under simulation.

use core::fmt;

/// Largest representable wall-clock value (48 bits). About 8 900 years.
pub const HLC_MAX_MS: u64 = (1 << 48) - 1;

/// Largest accepted lead of a remote timestamp over the local wall clock.
pub const HLC_MAX_DRIFT_MS: u64 = 60_000;

const LOGICAL_BITS: u32 = 16;
const LOGICAL_MASK: u64 = 0xFFFF;

/// Hybrid logical clock timestamp.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hlc(pub u64);

/// A remote timestamp lies further in the future than [`HLC_MAX_DRIFT_MS`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HlcDrift {
    /// Wall-clock part of the rejected remote timestamp.
    pub remote_ms: u64,
    /// Local wall clock used for the comparison.
    pub local_wall_ms: u64,
    /// Allowed drift at the time of the check.
    pub max_drift_ms: u64,
}

impl fmt::Display for HlcDrift {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote HLC {} ms exceeds local wall clock {} ms by more than {} ms",
            self.remote_ms, self.local_wall_ms, self.max_drift_ms
        )
    }
}

impl std::error::Error for HlcDrift {}

impl Hlc {
    /// Smallest timestamp.
    pub const ZERO: Hlc = Hlc(0);

    /// Builds a timestamp from its parts. Returns `None` if `ms` exceeds 48 bits.
    #[inline]
    pub const fn from_parts(ms: u64, logical: u16) -> Option<Hlc> {
        if ms > HLC_MAX_MS {
            None
        } else {
            Some(Hlc((ms << LOGICAL_BITS) | logical as u64))
        }
    }

    /// Wraps a raw packed value.
    #[inline]
    pub const fn from_raw(raw: u64) -> Hlc {
        Hlc(raw)
    }

    /// Wall-clock milliseconds part.
    #[inline]
    pub const fn ms(self) -> u64 {
        self.0 >> LOGICAL_BITS
    }

    /// Logical counter part.
    #[inline]
    pub const fn logical(self) -> u16 {
        (self.0 & LOGICAL_MASK) as u16
    }

    /// Canonical little-endian bytes.
    #[inline]
    pub const fn to_le_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    /// Parses canonical little-endian bytes.
    #[inline]
    pub const fn from_le_bytes(bytes: [u8; 8]) -> Hlc {
        Hlc(u64::from_le_bytes(bytes))
    }

    /// Packs `(ms, logical)`. A logical overflow carries into `ms`; `ms` saturates
    /// at [`HLC_MAX_MS`]. Saturation is unreachable in practice and never panics.
    #[inline]
    fn pack(ms: u64, logical: u32) -> Hlc {
        let (ms, logical) = if logical > u32::from(u16::MAX) {
            (ms.saturating_add(1), 0u32)
        } else {
            (ms, logical)
        };
        let ms = ms.min(HLC_MAX_MS);
        let logical = u16::try_from(logical).unwrap_or(u16::MAX);
        Hlc((ms << LOGICAL_BITS) | u64::from(logical))
    }

    /// Local event: advances the clock and returns the new timestamp.
    pub fn tick(&mut self, wall_ms: u64) -> Hlc {
        let wall = wall_ms.min(HLC_MAX_MS);
        let (ms, logical) = (self.ms(), u32::from(self.logical()));
        let next = if wall > ms {
            Hlc::pack(wall, 0)
        } else {
            Hlc::pack(ms, logical + 1)
        };
        *self = next;
        next
    }

    /// Receive event: merges a remote timestamp and returns the new timestamp.
    ///
    /// Rejects remote timestamps that lie more than [`HLC_MAX_DRIFT_MS`] ahead
    /// of `wall_ms`. On rejection the clock state is left unchanged.
    pub fn observe(&mut self, remote: Hlc, wall_ms: u64) -> Result<Hlc, HlcDrift> {
        let wall = wall_ms.min(HLC_MAX_MS);
        if remote.ms() > wall.saturating_add(HLC_MAX_DRIFT_MS) {
            return Err(HlcDrift {
                remote_ms: remote.ms(),
                local_wall_ms: wall,
                max_drift_ms: HLC_MAX_DRIFT_MS,
            });
        }
        let (l, lc) = (self.ms(), u32::from(self.logical()));
        let (r, rc) = (remote.ms(), u32::from(remote.logical()));
        let max_ms = l.max(r).max(wall);
        let next = if max_ms == l && max_ms == r {
            Hlc::pack(max_ms, lc.max(rc) + 1)
        } else if max_ms == l {
            Hlc::pack(max_ms, lc + 1)
        } else if max_ms == r {
            Hlc::pack(max_ms, rc + 1)
        } else {
            Hlc::pack(max_ms, 0)
        };
        *self = next;
        Ok(next)
    }
}

impl fmt::Debug for Hlc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hlc({}:{})", self.ms(), self.logical())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn parts_roundtrip_and_reject_oversized_ms() {
        let h = Hlc::from_parts(1_700_000_000_000, 7).unwrap_or(Hlc::ZERO);
        assert_eq!(h.ms(), 1_700_000_000_000);
        assert_eq!(h.logical(), 7);
        assert!(Hlc::from_parts(HLC_MAX_MS + 1, 0).is_none());
        assert!(Hlc::from_parts(HLC_MAX_MS, u16::MAX).is_some());
    }

    #[test]
    fn tick_same_wall_clock_increments_logical() {
        let mut c = Hlc::ZERO;
        let a = c.tick(100);
        let b = c.tick(100);
        assert_eq!((a.ms(), a.logical()), (100, 0));
        assert_eq!((b.ms(), b.logical()), (100, 1));
        assert!(b > a);
    }

    #[test]
    fn tick_advancing_wall_clock_resets_logical() {
        let mut c = Hlc::ZERO;
        c.tick(100);
        c.tick(100);
        let d = c.tick(101);
        assert_eq!((d.ms(), d.logical()), (101, 0));
    }

    #[test]
    fn tick_going_backwards_stays_monotonic() {
        let mut c = Hlc::ZERO;
        let a = c.tick(500);
        let b = c.tick(400);
        assert!(b > a);
        assert_eq!(b.ms(), 500);
    }

    #[test]
    fn logical_overflow_carries_into_ms_without_panic() {
        let mut c = Hlc::from_parts(100, u16::MAX).unwrap_or(Hlc::ZERO);
        let next = c.tick(100);
        assert_eq!((next.ms(), next.logical()), (101, 0));
    }

    #[test]
    fn observe_merges_remote_ahead_of_local() {
        let mut local = Hlc::ZERO;
        local.tick(100);
        let remote = Hlc::from_parts(120, 4).unwrap_or(Hlc::ZERO);
        let merged = local.observe(remote, 110);
        assert!(matches!(merged, Ok(h) if h.ms() == 120 && h.logical() == 5));
        assert_eq!(local, merged.unwrap_or(Hlc::ZERO));
    }

    #[test]
    fn observe_rejects_excessive_drift_and_keeps_state() {
        let mut local = Hlc::ZERO;
        local.tick(100);
        let before = local;
        let far = Hlc::from_parts(100 + HLC_MAX_DRIFT_MS + 1, 0).unwrap_or(Hlc::ZERO);
        let err = local.observe(far, 100);
        assert!(
            matches!(err, Err(HlcDrift { max_drift_ms, .. }) if max_drift_ms == HLC_MAX_DRIFT_MS)
        );
        assert_eq!(local, before);
    }

    #[test]
    fn le_bytes_roundtrip() {
        let h = Hlc::from_parts(42, 3).unwrap_or(Hlc::ZERO);
        assert_eq!(Hlc::from_le_bytes(h.to_le_bytes()), h);
    }

    proptest! {
        #[test]
        fn tick_is_strictly_monotonic(walls in proptest::collection::vec(0u64..1_000_000, 1..200)) {
            let mut c = Hlc::ZERO;
            let mut prev = Hlc::ZERO;
            for w in walls {
                let next = c.tick(w);
                prop_assert!(next > prev);
                prev = next;
            }
        }

        #[test]
        fn observe_result_dominates_both_inputs(
            local_wall in 0u64..1_000_000,
            remote_ms in 0u64..1_000_000,
            remote_l in any::<u16>(),
        ) {
            let mut c = Hlc::ZERO;
            c.tick(local_wall);
            let before = c;
            let remote = Hlc::from_parts(remote_ms, remote_l).unwrap_or(Hlc::ZERO);
            if let Ok(next) = c.observe(remote, local_wall) {
                prop_assert!(next > before);
                prop_assert!(next > remote);
            }
        }
    }
}
