//! Math utility types for safe arithmetic and saturating casts without silent truncation.

use serde::{Deserialize, Serialize};

/// A branch-free newtype for counters that saturate at `u8::MAX` rather than wrapping around
/// (prevents silent truncation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SaturatingU8(u8);

impl SaturatingU8 {
    /// Creates a new `SaturatingU8` wrapper around a `u8` value.
    #[inline(always)]
    pub const fn new(val: u8) -> Self {
        Self(val)
    }

    /// Unwraps and returns the underlying `u8` value.
    #[inline(always)]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl From<u8> for SaturatingU8 {
    #[inline(always)]
    fn from(val: u8) -> Self {
        Self(val)
    }
}

impl From<SaturatingU8> for u8 {
    #[inline(always)]
    fn from(s: SaturatingU8) -> Self {
        s.0
    }
}

impl From<usize> for SaturatingU8 {
    #[inline(always)]
    #[allow(clippy::cast_possible_truncation)]
    fn from(val: usize) -> Self {
        // Compiles to branchless `cmov` or `vmin` on x86_64/ARM64.
        Self(val.min(u8::MAX as usize) as u8)
    }
}

impl From<u32> for SaturatingU8 {
    #[inline(always)]
    #[allow(clippy::cast_possible_truncation)]
    fn from(val: u32) -> Self {
        Self(val.min(u8::MAX as u32) as u8)
    }
}

impl From<u64> for SaturatingU8 {
    #[inline(always)]
    #[allow(clippy::cast_possible_truncation)]
    fn from(val: u64) -> Self {
        Self(val.min(u8::MAX as u64) as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saturating_u8_from_usize() {
        assert_eq!(SaturatingU8::from(0usize).get(), 0);
        assert_eq!(SaturatingU8::from(100usize).get(), 100);
        assert_eq!(SaturatingU8::from(255usize).get(), 255);
        assert_eq!(SaturatingU8::from(256usize).get(), 255);
        assert_eq!(SaturatingU8::from(10000usize).get(), 255);
        assert_eq!(SaturatingU8::from(usize::MAX).get(), 255);
    }

    #[test]
    fn test_saturating_u8_conversions() {
        let sat = SaturatingU8::new(42);
        assert_eq!(sat.get(), 42);
        let val: u8 = sat.into();
        assert_eq!(val, 42);
        let sat_from_u8 = SaturatingU8::from(42u8);
        assert_eq!(sat_from_u8, sat);
    }
}
