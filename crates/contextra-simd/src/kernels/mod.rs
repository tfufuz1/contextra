// FILE-CONTEXT
// ZWECK: Kernel-Modul-Deklarationen für SIMD-Distanzfunktionen.

pub mod avx2;
pub mod avx512;
pub mod neon;
pub mod scalar;

pub use scalar::*;
