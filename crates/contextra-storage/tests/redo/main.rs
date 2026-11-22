#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

//! # Redo-Modul Integrationstests
//!
//! SPEC §9.4, §10.2, §10.5, §13.1, §13.2, §16.6 und §18 (INV-LOG-1 bis INV-LOG-8).

mod crash_matrix_tests;
mod crypto_tests;
mod golden_tests;
mod invariants_tests;
mod property_tests;
