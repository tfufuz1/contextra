// FILE-CONTEXT
// STAND: 2026-11-10
// ZWECK: Interne Testsuite für contextra-router (misc_tests.rs).
// INVARIANTEN: Deterministische Ausführung, Abdeckung von Routing- & Kalibrierungslogik.

//! # Verschiedene zusätzliche Tests und Hilfstests
//!
//! **Ring:** Ring 3 (Applikations- und Testschicht).
//! **Zweck:** Verschiedene zusätzliche Tests und Hilfstests.

// ANCHOR[contextra-router:TST-MSC-001]
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
mod part1;
mod part2;
