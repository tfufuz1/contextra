//! Local abstractions for engine components to decouple contextra-cognition.
//
// FILE-CONTEXT
// ZWECK: Lokale Re-Exports und Typ-Aliase für engine-Subsysteme vor der Invertierung (S-63).
// INVARIANTEN: Strikt temporär bis zur Delegation an contextra-db; keine zirkulären Pfade.
// STAND: 2026-11-11

// SECTION: Engine Re-Exports
// ANCHOR[contextra-cognition:LOC-ENG-001]
pub use contextra_engine as engine;
pub use engine::collection::{self, Collection, StoredDocument, StoredDocumentMeta};
pub use engine::decay_controller::{self, AdaptiveDecayController, DecayControllerConfig};
pub use engine::transaction::{self, CommitIntent};
pub use engine::ProvenanceRecord;
pub use engine::SearchResult;
// !SECTION: Engine Re-Exports
