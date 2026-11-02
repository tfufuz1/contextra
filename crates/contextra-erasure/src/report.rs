//! Berichts-Modul (SPEC §11.7).
//!
//! Zusammenstellung, Kanonisierung und Signierung des Löschberichts.
pub mod build;
pub mod risk;
pub mod canonical;
pub mod signing;

pub use build::{
    build_report, canonicalize_report, ErasureReport, FieldMeasurementSection,
    LayerKeyCoverage, ProcessHardeningStatus, ReachStatus, WitnessEntry,
};
pub use signing::SignedReport;
pub use canonical::{canonicalize_value, to_canonical_json, to_canonical_json_string};
pub use risk::{evaluate_residual_risks, ResidualRiskEntry};
