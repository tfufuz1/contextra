// Persistenz-Subsystem für DiskANN.
// Invarianten: Modulare Aufteilung in Teildomänen; re-exportiert alle bisherigen Pfade.

pub(crate) mod delta;
pub(crate) mod fallback;
pub(crate) mod loader;
pub(crate) mod node;
pub(crate) mod writer;
