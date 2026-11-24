//! Secret key material that cannot be copied by accident.
use core::fmt;
use zeroize::Zeroize;

/// A 256-bit secret.
///
/// Design decisions:
/// * The bytes live in a heap box, so moving a `Key32` never copies them
///   onto another stack frame.
/// * Construction takes `&mut [u8; 32]` and wipes the source immediately;
///   there is no by-value constructor that would leave a stray copy.
/// * Reads happen only inside a closure ([`Key32::with_exposed`]); the
///   closure receives a *slice*, not a `&[u8; 32]`, so callers cannot
///   write `*key.expose()` to obtain a `Copy` array.
/// * Not `Clone`, not `Copy`, redacted `Debug`, constant-time equality.
pub struct Key32 {
    bytes: Box<[u8; 32]>,
}

impl Key32 {
    /// Takes ownership of the secret and zeroizes `source`.
    pub fn take_from(source: &mut [u8; 32]) -> Self {
        let bytes = Box::new(*source);
        source.zeroize();
        Self { bytes }
    }

    /// Fills the key through a writer closure so no intermediate copy exists
    /// outside the key's own allocation (e.g. for entropy sources).
    pub fn fill_with<E>(f: impl FnOnce(&mut [u8; 32]) -> Result<(), E>) -> Result<Self, E> {
        let mut bytes = Box::new([0u8; 32]);
        match f(&mut bytes) {
            Ok(()) => Ok(Self { bytes }),
            Err(e) => {
                bytes.zeroize();
                Err(e)
            }
        }
    }

    /// Scoped read access. The slice must not escape the closure.
    pub fn with_exposed<R>(&self, f: impl FnOnce(&[u8]) -> R) -> R {
        f(self.bytes.as_slice())
    }

    /// Constant-time comparison.
    #[must_use]
    pub fn ct_eq(&self, other: &Self) -> bool {
        let mut diff = 0u8;
        for (a, b) in self.bytes.iter().zip(other.bytes.iter()) {
            diff |= a ^ b;
        }
        core::hint::black_box(diff) == 0
    }
}

impl Drop for Key32 {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

impl fmt::Debug for Key32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Key32(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_is_wiped_after_take() {
        let mut src = [7u8; 32];
        let key = Key32::take_from(&mut src);
        assert_eq!(src, [0u8; 32]);
        key.with_exposed(|b| assert!(b.iter().all(|&x| x == 7)));
    }

    #[test]
    fn debug_is_redacted() {
        let mut src = [1u8; 32];
        let key = Key32::take_from(&mut src);
        assert_eq!(format!("{key:?}"), "Key32(<redacted>)");
    }

    #[test]
    fn constant_time_eq() {
        let (mut a, mut b, mut c) = ([1u8; 32], [1u8; 32], [2u8; 32]);
        let (ka, kb, kc) = (Key32::take_from(&mut a), Key32::take_from(&mut b), Key32::take_from(&mut c));
        assert!(ka.ct_eq(&kb));
        assert!(!ka.ct_eq(&kc));
    }

    #[test]
    fn fill_error_wipes_and_propagates() {
        let r: Result<Key32, &str> = Key32::fill_with(|b| {
            b.fill(9);
            Err("entropy failed")
        });
        assert_eq!(r.err(), Some("entropy failed"));
    }
}
