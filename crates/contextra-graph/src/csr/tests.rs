// FILE-CONTEXT
// STAND: 2026-11-15T12:00:00Z
// ZWECK: Compressed Sparse Row (CSR) Graph-Engine Kern.
// INVARIANTEN: Bitemporale Sichtbarkeit, lock-freie Lesepfade.
// HOTSPOTS: Keine kritischen Bottlenecks identifiziert
// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

// ANCHOR[contextra-graph:SRC-CSR-TESTS-001]
//! Haupt-Testsuite für elementare CSR-Graphfunktionen.
//!
//! # Zweck
//! Überprüft Adjazenzabbildungen, Knoteneinfügungen und Konsistenzprüfungen.


// SECTION Tests Core Implementation
mod basic_tests;
mod bitemporal_tests;
mod graph_index_tests;
mod mutation_sequence_tests;
mod persistence_tests;

// !SECTION Tests Core Implementation
