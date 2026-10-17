//! Alias/Re-export-Modul für Off-Policy-Evaluation (§B.5.2.4 & §8.5).
//!
//! DEPRECATED: Use [`crate::off_policy`] instead.

#[deprecated(
    since = "0.1.0",
    note = "Use `contextra_adapt::off_policy` instead; removal 2027-01-31"
)]
pub use crate::off_policy::*;
