// FILE-CONTEXT
// ZWECK: Kernel-Modul-Deklarationen für SIMD-Distanzfunktionen.

pub(crate) mod avx2;
pub(crate) mod avx512;
pub(crate) mod neon;
pub mod scalar;

pub use scalar::*;
