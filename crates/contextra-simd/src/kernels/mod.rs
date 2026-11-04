//! # Modul `kernels`
//! Kapselt konkrete mathematische Implementierungen (AVX2, AVX-512, ARM NEON, Skalar-Referenz).
//!

// x86_64 AVX2 / FMA Instruktionen (256-Bit Vektoren).
pub(crate) mod avx2;

// x86_64 AVX-512F Instruktionen (512-Bit Vektoren).
pub(crate) mod avx512;

// AArch64 ARM NEON Instruktionen (128-Bit Vektoren).
pub(crate) mod neon;

// Plattformunabhängige Skalar-Berechnungen (Referenzpfad).
pub mod scalar;

pub use scalar::*;
