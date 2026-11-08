//! Local abstractions for engine components to decouple contextra-cognition.

pub use contextra_engine as engine;
pub use engine::collection::{self, Collection, StoredDocument, StoredDocumentMeta};
pub use engine::decay_controller::{self, AdaptiveDecayController, DecayControllerConfig};
pub use engine::transaction::{self, CommitIntent};
pub use engine::ProvenanceRecord;
pub use engine::SearchResult;
