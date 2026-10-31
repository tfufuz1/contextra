//! # Latenz- und Performance-Tests für Routing-Pfade
//!
//! **Ring:** Ring 3 (Applikations- und Testschicht).
//! **Zweck:** Latenz- und Performance-Tests für Routing-Pfade.

// ANCHOR[contextra-router:TST-LAT-001]
#![allow(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    deprecated
)]
// Latency-related tests are located in external files or covered by integration test suite.
