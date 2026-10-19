// FILE-CONTEXT
// ZWECK: Kernel-Modul-Deklarationen für SIMD-Distanzfunktionen.

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_sign_loss)]
#![allow(unused_unsafe)]

pub mod avx2;
pub mod avx512;
pub mod neon;
pub mod scalar;

pub use scalar::*;
