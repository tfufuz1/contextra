// Testmodul-Reorganisation für persistence (thematische Untermodule).
// Invarianten: Behaelt alle bisherigen Tests bei und fuehrt atomic_tests ein.

pub mod atomic_tests;
pub mod header_tests;
#[cfg(any(feature = "mmap", feature = "workspace"))]
pub mod mmap_tests;
pub mod node_tests;
