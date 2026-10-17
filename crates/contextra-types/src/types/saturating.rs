//! Branchless saturating newtype wrappers for safe narrowing integer conversions.

/// A saturating wrapper around `u16` designed to safely convert `usize` values into `u16`
/// without silent integer-truncation wraparound.
///
/// This type exists specifically to satisfy `clippy::cast_possible_truncation` at call sites
/// that require a saturating (non-panicking, non-wrapping) narrowing conversion, utilizing
/// branchless `cmov`/`min` CPU instructions for zero overhead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SaturatingU16(u16);

impl SaturatingU16 {
    /// Creates a new `SaturatingU16` wrapping the provided `u16` value.
    #[inline]
    pub const fn new(val: u16) -> Self {
        Self(val)
    }

    /// Returns the underlying inner `u16` value.
    #[inline]
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl From<usize> for SaturatingU16 {
    #[inline]
    fn from(val: usize) -> Self {
        // Safe narrowing conversion: val is bounded to at most u16::MAX by .min() before casting.
        #[allow(clippy::cast_possible_truncation)]
        Self(val.min(u16::MAX as usize) as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saturating_u16_from_usize() {
        assert_eq!(SaturatingU16::from(0usize).get(), 0);
        assert_eq!(SaturatingU16::from(255usize).get(), 255);
        assert_eq!(SaturatingU16::from(65535usize).get(), 65535);
        assert_eq!(SaturatingU16::from(65536usize).get(), 65535);
        assert_eq!(SaturatingU16::from(usize::MAX).get(), 65535);
    }
}
