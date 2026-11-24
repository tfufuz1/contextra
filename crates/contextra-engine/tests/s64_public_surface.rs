//! # Konsumenten-Anker (S-64A)
//!
//! Reiner Compile-Time-Pin-Test, der sicherstellt, dass alle öffentlichen Re-Exporte
//! von `contextra-engine` sowie alle Schnittstellen, die von `contextra-db` und `contextra-cognition`
//! konsumiert werden, vorhanden, unverändert typisiert und quellkompatibel bleiben.

#![allow(dead_code, unused_imports)]

use std::sync::Arc;

// 1. Konsumenten-Pfade von contextra-db
fn _pin_db_consumer_surface() {
    use contextra_engine::collection::{self, Collection, CollectionConfig};
    use contextra_engine::{
        Contextra, ContextraConfig, ContextraStats, DbStats, Document, EmbeddingBackend,
        HybridQueryBuilder, Language, ProvenanceRecord, SearchResult, SearchStrategy,
        SignalContribution, SignalWeights,
    };
    use contextra_engine::DurabilityMode;
    use contextra_engine::fusion;
    use contextra_engine::decay_controller::{
        AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs,
    };
    use contextra_engine::{
        background_workers, chunker, export, filter, import, temporal_filter, transaction,
        CommunityDetectionConfig, ExportCollectionV1, ExportDocumentV1, ExportMemoryV1,
        ExportRelationV1, ImportSummary, MAX_SCAN_RESULTS, SCHEMA_VERSION_V1,
    };

    #[cfg(feature = "sandbox")]
    use contextra_engine::SandboxBridge;
}

// 2. Konsumenten-Pfade von contextra-cognition
fn _pin_cognition_consumer_surface() {
    use contextra_engine::collection::Collection;
    use contextra_engine::Language;
    let _lang = Language::English;
    let _lang_de = Language::German;
}

// 3. Öffentliche Re-Exporte der Engine Facade (src/lib.rs)
fn _pin_engine_public_surface() {
    use contextra_engine::{
        Backends, Collection, CollectionConfig, CommunityDetectionConfig, Contextra,
        ContextraConfig, ContextraStats, DbStats, Document, DurabilityMode, EmbeddingBackend,
        HybridQueryBuilder, Language, ProvenanceRecord, SearchResult, SearchStrategy,
        SignalContribution, SignalWeights, TenantCollectionMap, TenantPolicy,
        TextEmbeddingEngine, VectorDeleteMode, MAX_SCAN_RESULTS,
    };

    #[cfg(feature = "graph-connectivity-health")]
    use contextra_engine::{PercolationConfig, PercolationResult};

    #[cfg(feature = "background-maintenance")]
    use contextra_engine::start_decay_cleanup_worker;

    use contextra_engine::{start_expiry_cleanup_worker, start_orphan_cleanup_worker};
}

#[test]
fn test_public_surface_compile_time_anchor() {
    // Reiner Compile-Time-Check, benötigt keine Laufzeitlogik
}
