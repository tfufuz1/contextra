//! Deterministischer Pseudozufallsgenerator für `contextra-adapt` (SPEC v4.0 §2 B11).
//!
//! SplitMix64-Generator für reproduzierbare Randomisierung ohne globale Zustände.

/// Trait für Pseudozufallsgeneratoren in Bandit- und Sampling-Algorithmen.
pub trait Rng {
    /// Generiert die nächste vorzeichenlose 64-Bit-Ganzzahl.
    fn next_u64(&mut self) -> u64;

    /// Generiert eine Fließkommazahl im halboffenen Intervall `[0, 1)`.
    fn next_f32(&mut self) -> f32 {
        let v = (self.next_u64() >> 40) as f32;
        v / 16777216.0
    }
}

/// Deterministischer SplitMix64 PRNG.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeededRng {
    state: u64,
}

impl SeededRng {
    /// Erstellt einen neuen `SeededRng` mit dem Startzustand `seed`.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
}

impl Rng for SeededRng {
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}
