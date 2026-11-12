//! Local abstractions for engine components to decouple contextra-cognition (SPEC S-63).
//
// FILE-CONTEXT
// ZWECK: Lokale Re-Exports und Typ-Aliase vor der Invertierung (S-63).
// INVARIANTEN: Keine Abhängigkeit mehr zu contextra-engine!

pub use crate::collection::{Collection, DbTransaction};
pub use crate::collection_types::{StoredDocument, StoredDocumentMeta};
pub use crate::decay::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs};
pub use crate::ports::{CollectionOps, CommitIntent};
pub use contextra_rank::fusion::{ProvenanceRecord, SearchResult};
pub use contextra_text::Language;

pub mod collection {
    pub use crate::collection::{Collection, DbTransaction};
    pub use crate::collection_types::{StoredDocument, StoredDocumentMeta};
}

pub mod decay_controller {
    pub use crate::decay::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs};
}

pub mod transaction {
    pub use crate::ports::CommitIntent;
}
