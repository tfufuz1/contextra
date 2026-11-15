// FILE-CONTEXT
// ZWECK: Persistenz-Subsystem für DiskANN.
// INVARIANTEN: Modulare Aufteilung in Teildomänen; re-exportiert alle bisherigen Pfade.
// STAND: 2026-11-11

pub(crate) mod delta;
pub(crate) mod fallback;
pub(crate) mod loader;
pub(crate) mod node;
pub(crate) mod writer;
