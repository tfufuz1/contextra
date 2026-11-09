//! # Modul: mod.rs
//!
//! **Zweck:** Modulübersicht über Collection-Integrationstests.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, strikte Mandanten- und Indexisolation.

mod fixtures;

mod concurrency_tests;
mod hybrid_search_part1_tests;
mod hybrid_search_part2_tests;
mod insert_tests;
mod link_tests;
mod misc_tests;
mod read_tests;
mod search_tests;
mod update_delete_tests;
