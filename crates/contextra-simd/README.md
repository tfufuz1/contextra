# contextra-simd

`contextra-simd` ist eine Unsafe-Insel für SIMD-Distanzkernel und Laufzeit-Hardware-Dispatch (Ring 0).

## Zweck

Stellt hochoptimierte Vektor-Distanzfunktionen (L2, Cosine, Dot Product) bereit. Generiert Hardware-Dispatch auf AVX2, AVX-512, NEON oder skalare Fallbacks.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 0 (Unsafe-Insel)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![allow(unsafe_code)]` — Isoliert in `contextra-simd`, Safe Public API Boundary.

## Öffentliche API-Übersicht

- **Dispatch & Validation:** `compute_distance`, `validate_vector`, `compute_distance_trusted`
- **Kernels:** `avx2`, `avx512`, `neon`, `scalar`

## Architektur & Verweise

Details zu den Unsafe-Inseln (§0.4) und SIMD-Distanzberechnungen finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) sowie `README.md` §7.4.
