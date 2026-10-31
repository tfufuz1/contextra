// FILE-CONTEXT
// ZWECK: Kernel-Modul-Deklarationen für SIMD-Distanzfunktionen.

//! # Modul `kernels`
//!
//! **Ring-Zugehörigkeit:** Ring 0 (Unsafe-Insel).
//!
//! **Zweck:**
//! Kapselt die konkreten mathematischen Implementierungen zur Vektordistanzberechnung.
//! Es unterteilt sich in plattformspezifische SIMD-Backends (AVX2, AVX-512, ARM NEON)
//! sowie eine portable Skalar-Referenzimplementierung (`scalar`), die auf allen Zielarchitekturen
//! ohne spezielle CPU-Instruktionen lauffähig ist.
//!
//! **Sichtbarkeit:**
//! Die hardwarenahen Kernel (`avx2`, `avx512`, `neon`) sind `pub(crate)` und werden ausschließlich
//! über das Laufzeit-Dispatching in `dispatch.rs` nach Prüfung der CPU-Features aufgerufen.
//! Das Modul `scalar` ist öffentlich zugänglich und dient als Fallback und Referenz.

// ANCHOR[contextra-simd:KERN-MOD-AVX2]
// x86_64 AVX2 / FMA Instruktionen (256-Bit Vektoren).
pub(crate) mod avx2;

// ANCHOR[contextra-simd:KERN-MOD-AVX512]
// x86_64 AVX-512F / AVX-512BW / AVX-512VNNI Instruktionen (512-Bit Vektoren).
pub(crate) mod avx512;

// ANCHOR[contextra-simd:KERN-MOD-NEON]
// AArch64 ARM NEON Instruktionen (128-Bit Vektoren).
pub(crate) mod neon;

// ANCHOR[contextra-simd:KERN-MOD-SCALAR]
// Plattformunabhängige Skalar-Berechnungen (Referenzpfad).
pub mod scalar;

pub use scalar::*;
