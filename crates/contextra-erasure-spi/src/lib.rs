//! # Service Provider Interface (SPI) für die modulare Löschung und Restdaten-Messung (Ring 0)
//!
//! Gemäß Architektur-Entscheidung D-E1 kapselt diese Crate alle Schnittstellen und
//! Typen, die für physisches Purging und forensische Restdaten-Messung erforderlich sind.
//!
//! ## Architekturregeln (D-E1 & Baukasten-Vertrag)
//! - Hängt ausschließlich an `contextra-types` (Ring 0).
//! - Datenhalter (`store`, `vector`, `text`, `graph`, `mvcc`) hängen **nicht** an `spi`.
//! - Satelliten (`contextra-erasure-<schicht>`) implementieren `Purger` und `LayerScanner`
//!   und rufen die generische Wartungs-API der Schicht auf.
//! - Strenges Zero-Panic-Gebot, keine rechtlichen Versprechungen im Core (SPEC 2.6).

#![forbid(unsafe_code)]

pub mod error;
pub mod layer;
pub mod purger;
pub mod scanner;
pub mod scope;
pub mod session;
pub mod witness;

pub use error::{PurgeError, ScanError, SpiError, SpiResult};
pub use layer::{PurgeStats, ScanTechnique, WitnessLayer};
pub use purger::Purger;
pub use scanner::LayerScanner;
pub use scope::{KeyScope, ScanTarget};
pub use session::ScanSession;
pub use witness::LayerWitness;

#[cfg(any(test, feature = "testkit"))]
pub use witness::mint_witness_for_testkit;
