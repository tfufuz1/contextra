//! Storage engines module.
//!
//! NICHT-PRODUKTIONSPFAD:
//! Reines In-Memory StorageEngine (`memory_only`) dient ausschließlich
//! zu Test- und Entwicklungszwecken (`test-utils`/Dev) und führt keine persistente
//! oder verschlüsselte Datenspeicherung durch (Klartextpfad).

#[cfg(all(feature = "memory-only-storage", any(test, feature = "test-utils")))]
pub mod memory_only;
