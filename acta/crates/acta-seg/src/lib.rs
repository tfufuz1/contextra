//! acta-seg: Versiegelter schluesselfreier Segmentspeicher: Journal, Segmente, Manifest.
//!
//! Level L2 · Budget 6500 LOC · Spezifikation docs/SPEC.md Kap. 5.1
//!
//! Extraktionsarchiv: `archive/acta-seg/MANIFEST.md` (falls vorhanden).
#![forbid(unsafe_code)]

pub mod error;
