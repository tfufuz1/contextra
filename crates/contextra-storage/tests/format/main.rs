#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::collapsible_if,
    clippy::unnecessary_map_or
)]

//! # Format Tests
//!
//! Verifiziert SPEC §9.2, Golden A.4, F-15 Downgrade-Schutz und Feature-Bits.

mod f15_test;
mod features_test;
mod golden_test;
