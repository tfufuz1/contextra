//! Eigenständiger Verifizierer für Löschberichte und Feldscans.
//!
//! Ring: dev/bin
//! Erlaubte Abhängigkeiten: keine Produktions-Crates (AR-4)
//! SPEC: §8.17, §11.7, Anhang D, F.9, F.10

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod canonical_json;
pub mod cli;
pub mod codes;
pub mod journal;
pub mod report;
pub mod residual_risk;
pub mod scan;
