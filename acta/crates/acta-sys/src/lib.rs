//! acta-sys: einzige Stelle mit `unsafe` (G-UNSAFE; jeder Block mit `// SAFETY:`).
//!
//! Level L1 · Budget 1600 LOC · Spezifikation docs/SPEC.md Kap. 4.6, 13.1, 13.2.
//!
//! Zielinhalt: `SecureBuf` (mlock, MADV_DONTDUMP, feste Kapazitaet), Hostpruefungen, Sandbox
//! (Landlock, seccomp), `SystemClock`, `OsCryptoRng`, SIMD-Kerne. Extraktionsarchiv: `archive/acta-sys/MANIFEST.md`.
#![deny(unsafe_op_in_unsafe_fn)]
