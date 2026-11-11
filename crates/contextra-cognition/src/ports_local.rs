//! Local abstractions for ports to decouple contextra-cognition.
//
// FILE-CONTEXT
// ZWECK: Lokale Re-Exports und Typ-Aliase für contextra-ports Port-Definitionen.
// INVARIANTEN: Zero-Panic; Dyn-Safety; Send + Sync für asynchrone Schnittstellen.
// STAND: 2026-11-11

// SECTION: Ports Re-Exports
// ANCHOR[contextra-cognition:LOC-PRT-001]
pub use contextra_ports as ports;
pub use ports::{
    BoxFuture, Clock, ContextSegment, GroundingValidator, IdGen, InMemoryMetricsSink, LayerScanner,
    LayerWitness, LlmTextGenerator, MetricsSink, NoopMetricsSink, Purger,
    ResponseGroundingValidator, Rng, SeededRng, SegmentSynthesizer, StorageEngine, StorageStats,
    SystemClock, TextEmbeddingEngine, VectorIndex,
};
// !SECTION: Ports Re-Exports
