// FILE-CONTEXT
// STAND: 2026-11-10
// ZWECK: Interne Testsuite für contextra-router (routing_tests.rs).
// INVARIANTEN: Deterministische Ausführung, Abdeckung von Routing- & Kalibrierungslogik.

//! # Routing-Test-Suiten und Untermodul-Deklarationen
//!
//! **Ring:** Ring 3 (Applikations- und Testschicht).
//! **Zweck:** Routing-Test-Suiten und Untermodul-Deklarationen.

// ANCHOR[contextra-router:TST-RTG-001]
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
#[cfg(feature = "subprocess-dispatch")]
mod dispatch;
mod engine_part1;
mod engine_part2;
