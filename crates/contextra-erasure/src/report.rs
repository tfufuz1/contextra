//! Berichts-Modul (SPEC §11.7, §12.5, Anhang D, Anhang F.10).
//!
//! Zusammenstellung, Kanonisierung und Signierung des Löschberichts.

pub mod build;
pub mod canonical;
pub mod risk;
pub mod signing;

pub use build::{
    build_report, build_report_with_options, canonicalize_report, ErasureReport,
    FieldMeasurementSection, LayerKeyCoverage, LayerWitnessReport, ProcessHardeningStatus,
    ReachStatus, ReportBuildOptions, WitnessEntry,
};
pub use canonical::{
    canonicalize_json, canonicalize_value, to_canonical_json, to_canonical_json_string,
    verify_canonical_nfc,
};
pub use risk::{
    evaluate_residual_risks, evaluate_residual_risks_with_context, ResidualRiskContext,
    ResidualRiskEntry, ResidualRiskId,
};
pub use signing::{base64url_encode, DerivedReportSigner, ReportSigner, SignedReport};
