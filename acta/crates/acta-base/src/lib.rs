//! acta-base: Newtypes, Fehlerklassen, Ports, Festkomma.
//!
//! Level L0 · Budget 2500 LOC · Spezifikation docs/SPEC.md Kap. 2.2, 2.4, 3.5, 3.7, 4.6, 5.1.
//!
//! Extraktionsarchiv: `archive/acta-base/MANIFEST.md`.
#![forbid(unsafe_code)]

pub mod class;
pub mod error;
pub mod fx;
pub mod ids;
pub mod ports;

pub use class::DataClass;
pub use error::{ClassifiedError, ErrorClass};
pub use ports::{Clock, CryptoRng, EntropyError};
