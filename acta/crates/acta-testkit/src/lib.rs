//! acta-testkit: `ManualClock`, `SeededRng`, Fehlerinjektion, Konformitaetstests (Spec 16.1, 16.2).
//!
//! Extraktionsarchiv: `archive/acta-testkit/MANIFEST.md`.
#![forbid(unsafe_code)]

use acta_base::{Clock, CryptoRng, EntropyError};
use std::sync::atomic::{AtomicU64, Ordering};

/// Von Hand gestellte Uhr.
#[derive(Debug, Default)]
pub struct ManualClock(AtomicU64);

impl ManualClock {
    /// Uhr auf `ms` setzen.
    #[must_use]
    pub fn new(ms: u64) -> Self {
        Self(AtomicU64::new(ms))
    }
    /// Uhr stellen (auch rueckwaerts, fuer `ClockRegression`-Tests).
    pub fn set(&self, ms: u64) {
        self.0.store(ms, Ordering::SeqCst);
    }
    /// Uhr vorstellen.
    pub fn advance(&self, ms: u64) {
        self.0.fetch_add(ms, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now_ms(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// Deterministischer Zufall (SplitMix64). Nur fuer Tests; liefert nie Fehler.
#[derive(Debug)]
pub struct SeededRng(AtomicU64);

impl SeededRng {
    /// Neuer Generator mit Startwert.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self(AtomicU64::new(seed))
    }
    fn next(&self) -> u64 {
        let mut z = self
            .0
            .fetch_add(0x9E37_79B9_7F4A_7C15, Ordering::SeqCst)
            .wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

impl CryptoRng for SeededRng {
    fn try_fill(&self, dst: &mut [u8]) -> Result<(), EntropyError> {
        for chunk in dst.chunks_mut(8) {
            let v = self.next().to_be_bytes();
            let n = chunk.len();
            chunk.copy_from_slice(v.get(..n).ok_or(EntropyError)?);
        }
        Ok(())
    }
}

/// Zufallsquelle, die immer scheitert (Test `T-RNG-*`: Entropiefehler erzeugt nie Umschlag oder Schluessel).
#[derive(Debug, Default)]
pub struct FailingRng;

impl CryptoRng for FailingRng {
    fn try_fill(&self, _dst: &mut [u8]) -> Result<(), EntropyError> {
        Err(EntropyError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_rng_is_deterministic() {
        let (a, b) = (SeededRng::new(1), SeededRng::new(1));
        let (mut x, mut y) = ([0_u8; 24], [0_u8; 24]);
        assert!(a.try_fill(&mut x).is_ok() && b.try_fill(&mut y).is_ok());
        assert_eq!(x, y);
    }

    #[test]
    fn failing_rng_fails() {
        assert!(FailingRng.try_fill(&mut [0_u8; 4]).is_err());
    }

    #[test]
    fn manual_clock_can_go_back() {
        let c = ManualClock::new(10);
        c.set(5);
        assert_eq!(c.now_ms(), 5);
    }
}
