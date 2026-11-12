// FILE-CONTEXT
// ZWECK: Testmodul-Reorganisation für persistence (thematische Untermodule).
// INVARIANTEN: Behaelt alle bisherigen Tests bei und fuehrt atomic_tests ein.
// STAND: TS:2026-11-12T00:00:00Z

pub mod atomic_tests;
pub mod header_tests;
#[cfg(any(feature = "mmap", feature = "workspace"))]
pub mod mmap_tests;
pub mod node_tests;
