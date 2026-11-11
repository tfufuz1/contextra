// FILE-CONTEXT
// STAND: 2026-11-15T12:00:00Z
// ZWECK: Personalized PageRank (Forward-Push und dichte Matrix-Berechnung).
// INVARIANTEN: Massenerhaltung der Residuen, Konvergenzgarantie.
// HOTSPOTS: Keine kritischen Bottlenecks identifiziert
// SIEHE AUCH: docs/SPEC.md, crates/contextra-graph/SPEC.md

// ANCHOR[contextra-graph:SRC-PPR-TESTS-001]
//! Haupt-Testsuite für personalisierte PageRank-Verfahren.
//!
//! # Zweck
//! Prüft Konvergenzkriterien, Teleport-Wahrscheinlichkeiten und Vektornormalisierung.


// SECTION Tests Core Implementation
mod algo_tests;
mod context_tests;
mod golden_tests;

// !SECTION Tests Core Implementation
