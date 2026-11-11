// FILE-CONTEXT
// ZWECK: Gemeinsame Hilfsfunktionen und deterministische Datensatzgeneratoren für Benchmarks
// INVARIANTEN: Deterministische Pseudozufallserzeugung über feste Seeds (kein thread_rng, kein Instant::now im Setup)
// HOTSPOTS: 20-80
// STAND: 2026-11-11

//! Gemeinsame Benchmark-Hilfsmittel für `contextra-vector`.
//!
//! Stellt deterministische Vektor- und ID-Generatoren mit festen Seeds bereit,
//! um reproduzierbare Benchmark-Läufe zu garantieren.

// ANCHOR[contextra-vector:W11-BENCH-001]

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Standard-Seed für reproduzierbare Benchmark-Läufe.
pub const DEFAULT_BENCH_SEED: u64 = 0x5EED_0001_CAFE_BABE;

/// Typische Vektordimensionen für Benchmarks.
pub const DIM_SMALL: usize = 16;
pub const DIM_MEDIUM: usize = 128;
pub const DIM_LARGE: usize = 1536;

/// Generiert einen normalisierten Vektor mit Pseudo-Zufallswerten im Bereich [-1.0, 1.0].
pub fn generate_vector(dim: usize, seed: u64) -> Vec<f32> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut v: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
    v
}

/// Generiert `count` normalisierte Vektoren mit deterministischem PRNG.
pub fn generate_vectors(count: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut vectors = Vec::with_capacity(count);
    for _ in 0..count {
        let mut v: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in v.iter_mut() {
                *x /= norm;
            }
        }
        vectors.push(v);
    }
    vectors
}

/// Generiert unnormalisierte Vektoren für Distanz- und SQ8-Tests.
pub fn generate_raw_vectors(count: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut vectors = Vec::with_capacity(count);
    for _ in 0..count {
        let v: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
        vectors.push(v);
    }
    vectors
}
