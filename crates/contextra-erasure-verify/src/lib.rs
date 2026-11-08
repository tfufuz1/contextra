//! Unabhaengige Verifikationsinstanz fuer Loeschberichte und Feldscans.
//!
//! Entworfen als eigenstaendiger Baukasten-Baustein gemaess SPEC §8.17, §11.7, Anhang D, F.9, F.10.
//! Ring: Tooling.
//! Keine Abhaengigkeit zum Loeschkern `contextra-erasure`.

#![forbid(unsafe_code)]

pub mod canonical_json;
pub mod codes;
pub mod report;
pub mod residual_risk;
pub mod scan;

pub use codes::{InvalidCode, VerifyOutcome, EXIT_INVALID, EXIT_SUCCESS};
pub use report::{verify_report_bytes, verify_report_file, ErasureReport, LayerWitnessReport};
pub use scan::{execute_field_scan, FieldScanOptions};
