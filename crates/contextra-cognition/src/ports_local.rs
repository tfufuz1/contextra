//! Local abstractions for ports to decouple contextra-cognition.

pub use contextra_ports as ports;
pub use ports::{
    BoxFuture, Clock, ContextSegment, GroundingValidator, IdGen, InMemoryMetricsSink, LayerScanner, LayerWitness,
    LlmTextGenerator, MetricsSink, NoopMetricsSink, Purger, ResponseGroundingValidator, Rng, SeededRng,
    SegmentSynthesizer, StorageEngine, StorageStats, SystemClock, TextEmbeddingEngine, VectorIndex,
};
