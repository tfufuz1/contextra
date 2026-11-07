//! Local abstractions for engine components to decouple `contextra-cognition`.

pub use contextra_engine as engine;
pub use engine::collection::Collection;
pub use engine::decay_controller::AdaptiveDecayController;
pub use engine::decay_controller::DecayControllerConfig;
pub use engine::transaction::CommitIntent;
pub use engine::ProvenanceRecord;
pub use engine::SearchResult;
