//! # Lizenz-Naht (S-64B)
//!
//! Kapselt alle Verwendungen von `contextra_ports::license::*` an einer einzigen Nahtstelle.
//! Lizenz-Seam; das Crate `contextra-license` wurde entfernt, die Lizenz-Traits
//! bleiben vorerst in `contextra-ports` (Entscheidung S-50 / S-64 offen).

pub use contextra_ports::license::{AuthorizedRing, FeatureRing, LicenseGate, OpenFastGate};
