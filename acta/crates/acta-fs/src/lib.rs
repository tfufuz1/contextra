//! acta-fs: Vfs-Port, DurableVfs, FaultVfs, Scrub, O_DIRECT-Ruecklesen.
//!
//! Level L1 · Budget 2500 LOC · Spezifikation docs/SPEC.md Kap. 3.7, 5.6
//!
//! Extraktionsarchiv: `archive/acta-fs/MANIFEST.md` (falls vorhanden).
#![forbid(unsafe_code)]

pub mod error;
