//! acta-fs: Vfs-Port, `DurableVfs`, `FaultVfs`, Scrub, `O_DIRECT`-Rücklesen.
//!
//! Level L1 · Budget 2500 LOC · Spezifikation docs/SPEC.md Kap. 3.7, 5.6, 16.2, 16.4.
//!
//! Extraktionsarchiv: `archive/acta-fs/MANIFEST.md`. Die Archiv-Ports nutzen eine andere
//! API und wurden nicht übernommen; Verhalten und Fehlerbilder sind neu nach Spec gebaut.
#![forbid(unsafe_code)]

pub mod error;
pub mod vfs;

#[cfg(unix)]
pub mod durable;

#[cfg(unix)]
pub mod fault;
