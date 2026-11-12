//! # Lizenz-Naht (S-64B)
//!
//! Kapselt alle Verwendungen von `contextra_ports::license::*` an einer einzigen Nahtstelle.
//! Ermöglicht die spätere Übergabe an `contextra-license`, sobald die Lizenz-Traits
//! von `contextra-ports` nach `contextra-license` migriert werden (Entscheidung S-50 / S-64).

pub use contextra_ports::license::{AuthorizedRing, FeatureRing, LicenseGate, OpenFastGate};
