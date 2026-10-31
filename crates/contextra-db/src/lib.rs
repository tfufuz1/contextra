// FILE-CONTEXT
// ZWECK: Contextra Database Facade & Integration Hub (Ring 3).
// INVARIANTEN: Abwärtskompatibilität für bestehende Pfade via prelude/advanced; Forbid unsafe code.

#![forbid(unsafe_code)]

//! # Contextra DB — Facade & Integrationstest-Hub
//!
//! `contextra-db` bildet die benutzerfreundliche Hauptfassade (Ring 3) für Anwendungen,
//! welche die Contextra-Datenbankbibliothek als eingebettete Engine nutzen.
//!
//! ## Architekturübersicht: Welche Schicht gehört wozu?
//!
//! Contextra ist als geschichteter Baukasten (Layered Toolkit) konzipiert:
//!
//! - **Ring 0 — Fundament & Hardware-Isolation (`contextra-store`, `contextra-sys`, `contextra-wire`)**:
//!   Verwaltet LSM-Storage (Log-Structured Merge-Tree), WAL (Write-Ahead-Log), Speicher-Locks,
//!   Zero-Copy-IPC sowie direkte I/O- und Durabilitätssicherungen (fsync/fdatasync).
//! - **Ring 1 — Fachspezifische Indizes (`contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-crypto`)**:
//!   Implementiert Vektorsuche (HNSW / Hierarchical Navigable Small World), BM25-Textsuche,
//!   Graph-CSR-Strukturen (Compressed Sparse Row) und kryptographische DSGVO-Art.-17-Löschbeweise.
//! - **Ring 2 — Adaption & Kognitive Orchestrierung (`contextra-engine`, `contextra-adapt`, `contextra-cognition`)**:
//!   Kombiniert Multi-Signal-Fusion (RRF / Reciprocal Rank Fusion), dynamische Schwellenwert-Anpassung,
//!   Memory Consolidation (Speicher-Konsolidierung), Turn-Segmentierung und Schlafzyklen-Synthese.
//! - **Ring 3 — Fassade & Haupt-API (`contextra-db`)**:
//!   Bündelt alle Subsysteme in eine konsistente, typ- und mandanten-sichere API ([`Contextra`], [`Collection`]).
//!
//! ## Einbindung und Abhängigkeiten
//! - **Eingehende Einbindungen**: `contextra` (Top-Level Facade), `contextra-agent`, `contextra-py` (Python-Bindings), `contextra-bench`.
//! - **Ausgehende Abhängigkeiten**: `contextra-engine`, `contextra-types`, `contextra-ports`, `contextra-store`,
//!   `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-cognition`, `contextra-adapt`, `contextra-rank`,
//!   `contextra-checkpoint`, `contextra-crypto`, `contextra-sys`.
//!
//! ## API-Strukturierung: `prelude` vs. `advanced`
//!
//! - **[`prelude`]**: Die empfohlene, stabile Kern-API für reguläre Datenbankoperationen (Einfügen, Suchen, Löschen, Multi-Step).
//! - **[`advanced`]**: Erweiterte Steuerungselemente und fortgeschrittene Subsystem-Schnittstellen aus `adapt`, `cognition` und `graph`.

// ANCHOR[contextra-db:FACADE-001]
/// Stabile Kern-API für den alltäglichen Einsatz.
///
/// Enthält alle wesentlichen Typen für das Öffnen der Datenbank, Dokumentenoperationen,
/// Hybrid-Abfragen und Multi-Step Retrieval.
pub mod prelude {
    pub use contextra_engine::collection::{self, Collection, CollectionConfig};
    pub use contextra_engine::{
        Contextra, ContextraConfig, ContextraStats, DbStats, Document, EmbeddingBackend,
        HybridQueryBuilder, Language, ProvenanceRecord, SearchResult, SearchStrategy,
        SignalContribution, SignalWeights,
    };
    pub use contextra_types::{DistanceMetric, DocId, RequestContext, TenantId, TxId};

    pub use crate::multistep::{MultiStepConfig, MultiStepEngine, MultiStepResult, QueryRewriter};

    #[cfg(feature = "volatile-vault")]
    pub use crate::volatile_vault::{
        CommitReceipt, PurgeReceipt, SignalModality, VaultChunk, VaultChunkMetadata, VaultConfig,
        VaultError, VolatileContextVault,
    };
}

// ANCHOR[contextra-db:FACADE-002]
/// Erweiterte Experten-Schnittstellen aus Adaption, Kognition und Graph-Maintenance.
///
/// Bietet tiefere Eingriffsmöglichkeiten für Hintergrundwartung, Latenzkontrolle (PID),
/// Decay-Steuerung und Schlafzyklen-Konsolidierung.
pub mod advanced {
    pub use crate::decay_controller::{
        AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs,
    };
    pub use crate::homeostat::RerankDeadline;
    #[allow(deprecated)]
    pub use crate::homeostat::RerankPidController;
    pub use crate::pid_latency_controller::{
        LatencyBudgetGuard, PidLatencyController, DEFAULT_TARGET_LATENCY_MS, MAX_SCALING_FACTOR,
        MIN_SCALING_FACTOR,
    };

    pub use contextra_cognition as cognition;
    pub use contextra_cognition::{
        cleanup_orphaned_consolidation_intents, compact_segment_via_context_compactor,
        compute_community_hash, detect_near_duplicates, execute_background_consolidation,
        execute_consolidation_pass, group_turns_into_segments, run_consolidation_pass,
        run_synthesis_pass, CommunityStabilityTracker, CompactedContext, CompactionStrategy,
        ConsolidationConfig, ConsolidationEngine, ConsolidationNodesGuard,
        ConsolidationPhaseResult, ConsolidationSession, ContextCompactor, ContextManager,
        MaintenanceConfig, MaintenanceScheduler, MetaChunk, SpatialFence, StatusToken,
        SynthesisConfig, SynthesisPhaseResult, TurnSegment,
    };
    pub use contextra_engine as engine;
    pub use contextra_engine::{
        background_workers, chunker, export, filter, import, temporal_filter, transaction,
        CommunityDetectionConfig, ExportCollectionV1, ExportDocumentV1, ExportMemoryV1,
        ExportRelationV1, ImportSummary, MAX_SCAN_RESULTS, SCHEMA_VERSION_V1,
    };

    #[cfg(feature = "graph-connectivity-health")]
    pub use contextra_engine::collection::maintenance::PercolationResult;
    #[cfg(feature = "sandbox")]
    pub use contextra_engine::SandboxBridge;
}

// Abwärtskompatible Re-Exports auf Root-Ebene (bestehender Code kompiliert nahtlos weiter)
pub use advanced::*;
pub use prelude::*;

// ANCHOR[contextra-db:FACADE-003]
/// Kontext-Verwaltung und Konversationshistorie.
pub mod context {
    pub use contextra_cognition::context::*;
}

// ANCHOR[contextra-db:FACADE-004]
/// Kontext-Kompaktierung und Chunk-Zusammenfassung.
pub mod context_compaction {
    pub use contextra_cognition::context_compaction::*;
}

// ANCHOR[contextra-db:FACADE-005]
/// Speicher-Konsolidierung und Wissens-Synthese.
pub mod memory_consolidation {
    pub use contextra_cognition::memory_consolidation::*;
}

// ANCHOR[contextra-db:FACADE-006]
/// Synthese-Phase für konsolidierte Wissenssegmente.
pub mod synthesis_phase {
    pub use contextra_cognition::synthesis_phase::*;
}

// ANCHOR[contextra-db:FACADE-007]
/// Ausführung von Konsolidierungs-Passes.
pub mod consolidation_executor {
    pub use contextra_cognition::consolidation_executor::*;
}

// ANCHOR[contextra-db:FACADE-008]
/// Locks und Synchronisation für Konsolidierung.
pub mod consolidation_locks {
    pub use contextra_cognition::consolidation_locks::*;
}

// ANCHOR[contextra-db:FACADE-009]
/// Scheduler für periodische Hintergrundwartung.
pub mod maintenance_scheduler {
    pub use contextra_cognition::maintenance_scheduler::*;
}

// ANCHOR[contextra-db:FACADE-010]
/// Konfiguration für Wartungszyklen.
pub mod maintenance_config {
    pub use contextra_cognition::maintenance_config::*;
}

// ANCHOR[contextra-db:FACADE-011]
/// Veraltetes Reaper-Modul (bitte stattdessen `background_workers` verwenden).
#[deprecated(note = "use background_workers instead")]
pub mod reaper {
    pub use contextra_engine::background_workers::*;
}
#[deprecated(note = "use start_consolidation_worker instead")]
#[allow(deprecated)]
pub use contextra_cognition::start_consolidation_reaper;

// mod Collection is used via pub mod collection
// ANCHOR[contextra-db:FACADE-012]
/// Adaptiver Decay-Controller für zeitliche Relevanz-Abwertung.
pub mod decay_controller {
    pub use contextra_adapt::decay_controller::*;
}
pub use contextra_engine::fusion;
// ANCHOR[contextra-db:FACADE-013]
/// Homeostat-Modul für PID-Kandidatenpool-Regulierung.
pub mod homeostat;
// ANCHOR[contextra-db:FACADE-014]
/// Iteratives Multi-Step Retrieval mit Query Rewriting.
pub mod multistep;
// ANCHOR[contextra-db:FACADE-015]
/// PID-Regler zur Einhaltung von Latenzbudgets.
pub mod pid_latency_controller {
    pub use contextra_adapt::pid_latency_controller::*;
}

#[cfg(feature = "volatile-vault")]
pub mod volatile_vault;
#[cfg(feature = "volatile-vault")]
pub use volatile_vault::{
    CommitReceipt, PurgeReceipt, SignalModality, VaultChunk, VaultChunkMetadata, VaultConfig,
    VaultError, VolatileContextVault,
};

pub use decay_controller::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs};
pub use homeostat::RerankDeadline;
#[allow(deprecated)]
pub use homeostat::RerankPidController;
pub use multistep::{MultiStepConfig, MultiStepEngine, MultiStepResult, QueryRewriter};
pub use pid_latency_controller::{
    LatencyBudgetGuard, PidLatencyController, DEFAULT_TARGET_LATENCY_MS, MAX_SCALING_FACTOR,
    MIN_SCALING_FACTOR,
};

pub use collection::{Collection, CollectionConfig};
pub use contextra_checkpoint;
#[cfg(feature = "graph-connectivity-health")]
pub use contextra_graph::percolation::PercolationConfig;

pub use contextra_ports::{DriftStatusProvider, SegmentSynthesizer, TextEmbeddingEngine};
pub use contextra_types::DistanceMetric;
pub use serde_json::json;

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use contextra_types::TenantId;
    use std::sync::Arc;
    use tempfile::TempDir;

    // Erzeugt eine temporäre Test-Datenbankinstanz mit angegebener Vektordimension.
    async fn test_db(dim: usize) -> (Contextra, TempDir) {
        let tmp = TempDir::new().expect("temp dir"); // expect
        let config = ContextraConfig {
            dimension: dim,
            max_elements: 10_000,
            distance_metric: DistanceMetric::Cosine,
            ..Default::default()
        };
        let db = Contextra::open_with_config(tmp.path(), config)
            .await
            .expect("open db"); // expect
        (db, tmp)
    }

    // Prüft Einfügen und Vektorsuche im Roundtrip.
    #[tokio::test]
    async fn test_insert_search_roundtrip() {
        let (db, _tmp) = test_db(4).await;

        db.insert(
            "doc-1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"topic": "rust"})),
        )
        .await
        .expect("insert"); // expect

        db.insert(
            "doc-2",
            &[0.0, 1.0, 0.0, 0.0],
            Some(json!({"topic": "python"})),
        )
        .await
        .expect("insert"); // expect

        db.insert("doc-3", &[0.9, 0.1, 0.0, 0.0], None)
            .await
            .expect("insert"); // expect

        let results = db.search(&[1.0, 0.0, 0.0, 0.0], 2).await.expect("search"); // expect
        assert_eq!(results.len(), 2);
        // doc-1 should be closest
        assert!(results[0].score > results[1].score);
    }

    // Prüft, dass Suchergebnisse gespeicherte Metadaten korrekt zurückliefern.
    #[tokio::test]
    async fn test_insert_search_returns_metadata() {
        let (db, _tmp) = test_db(4).await;

        db.insert(
            "doc-1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"topic": "rust", "priority": 1})),
        )
        .await
        .expect("insert"); // expect

        let results = db.search(&[1.0, 0.0, 0.0, 0.0], 1).await.expect("search"); // expect
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc-1");
        let meta = results[0].metadata.as_ref().expect("metadata should exist"); // expect
        assert_eq!(meta["topic"], "rust");
        assert_eq!(meta["priority"], 1);
    }

    // Prüft das Auslesen eines Dokuments anhand seines Primärschlüssels.
    #[tokio::test]
    async fn test_get_by_key() {
        let (db, _tmp) = test_db(4).await;

        db.insert(
            "doc-1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"topic": "rust"})),
        )
        .await
        .expect("insert"); // expect

        let doc = db.get("doc-1").await.expect("get").expect("should exist"); // expect
        assert_eq!(doc.id, "doc-1");
        assert_eq!(doc.metadata.expect("valid")["topic"], "rust"); // expect

        let none = db.get("nonexistent").await.expect("get"); // expect
        assert!(none.is_none());
    }

    // Prüft das Aktualisieren von Vektor und Metadaten eines Dokuments.
    #[tokio::test]
    async fn test_update() {
        let (db, _tmp) = test_db(4).await;

        db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], Some(json!({"v": 1})))
            .await
            .expect("insert"); // expect

        db.update("doc-1", &[0.0, 1.0, 0.0, 0.0], Some(json!({"v": 2})))
            .await
            .expect("update"); // expect

        // Metadata should be updated
        let doc = db.get("doc-1").await.expect("get").expect("exists"); // expect
        assert_eq!(doc.metadata.expect("valid")["v"], 2); // expect

        // Vector should be updated — search for new vector should find it
        let results = db.search(&[0.0, 1.0, 0.0, 0.0], 1).await.expect("search"); // expect
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc-1");
    }

    // Prüft das Löschen eines Dokuments und Konsistenz von db.len().
    #[tokio::test]
    async fn test_delete() {
        let (db, _tmp) = test_db(4).await;

        db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("insert"); // expect
        assert_eq!(db.len().await.expect("len"), 1); // expect

        db.delete("doc-1").await.expect("delete"); // expect
        assert_eq!(db.len().await.expect("len"), 0); // expect

        // get should return None after delete
        let doc = db.get("doc-1").await.expect("get"); // expect
        assert!(doc.is_none());
    }

    // Prüft das Anlegen gerichteter Wissensgraph-Kanten zwischen Dokumenten.
    #[tokio::test]
    async fn test_relate() {
        let (db, _tmp) = test_db(4).await;

        db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("insert"); // expect
        db.insert("doc-2", &[0.0, 1.0, 0.0, 0.0], None)
            .await
            .expect("insert"); // expect

        // Should not error
        db.relate("doc-1", "doc-2", "references")
            .await
            .expect("relate"); // expect
    }

    // Prüft, dass Vektoren mit inkompatibler Dimension abgewiesen werden (Fail-Closed).
    #[tokio::test]
    async fn test_dimension_mismatch() {
        let (db, _tmp) = test_db(4).await;
        let result = db.insert("doc-1", &[1.0, 0.0], None).await;
        assert!(result.is_err());
    }

    // Prüft, dass eine Suche auf leerer Datenbank eine leere Ergebnismenge liefert.
    #[tokio::test]
    async fn test_empty_search() {
        let (db, _tmp) = test_db(4).await;
        let results = db.search(&[1.0, 0.0, 0.0, 0.0], 5).await.expect("search"); // expect
        assert!(results.is_empty());
    }

    // Prüft bidirektionale Relationen und Prefix-Scans im Wissensgraphen.
    #[tokio::test]
    async fn test_relate_and_scan_prefix() {
        let (db, _tmp) = test_db(4).await;

        db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("insert"); // expect
        db.insert("doc-2", &[0.0, 1.0, 0.0, 0.0], None)
            .await
            .expect("insert"); // expect
        db.insert("doc-3", &[0.0, 0.0, 1.0, 0.0], None)
            .await
            .expect("insert"); // expect

        db.relate_bidirectional("doc-1", "doc-2", "references")
            .await
            .expect("relate"); // expect
        db.relate("doc-1", "doc-3", "references")
            .await
            .expect("relate"); // expect

        // Scan for relations of doc-1
        let results = db
            .scan_prefix("__rel:doc-1:references:", None)
            .await
            .expect("scan"); // expect
        assert_eq!(results.len(), 2);

        let related_ids: Vec<String> = results
            .into_iter()
            .map(|(_, v)| v["to"].as_str().expect("valid").to_string()) // expect
            .collect();
        assert!(related_ids.contains(&"doc-2".to_string()));
        assert!(related_ids.contains(&"doc-3".to_string()));

        // Check backward edge setup automatically
        let backward_results = db
            .scan_prefix("__rel:doc-2:references:", None)
            .await
            .expect("scan bwd"); // expect
        assert_eq!(backward_results.len(), 1);
        assert_eq!(backward_results[0].1["to"], "doc-1");
    }

    // Prüft die Aggregation von Statistiken über Speicher und Indizes.
    #[tokio::test]
    async fn test_stats_aggregation() {
        let (db, _tmp) = test_db(4).await;

        db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("insert"); // expect

        let stats = db.stats().await.expect("stats"); // expect
        assert_eq!(stats.index_stats.num_vectors, 1);
        assert_eq!(stats.active_memory_count, 1);
        assert_eq!(stats.drift_status, "nicht verfügbar");
        assert!(stats.storage_stats.memtable_size_bytes > 0);
    }

    // Prüft den Lebenszyklus von Weak-References bei Drift-Providern und Kalibratoren in den Statistiken.
    #[tokio::test]
    async fn test_stats_live_data_and_weak_reference_lifecycle() {
        let (db, _tmp) = test_db(4).await;

        // 1. Unattached state
        let stats_unattached = db.stats().await.expect("stats");
        assert_eq!(stats_unattached.drift_status, "nicht verfügbar");
        assert_eq!(stats_unattached.calibration_ece, None);
        assert_eq!(stats_unattached.last_calibration_at, None);
        assert_eq!(stats_unattached.pid_pool_size, None);

        // 2. Attached state with live components
        struct DummyRouter;
        impl DriftStatusProvider for DummyRouter {
            fn overall_drift_status(&self) -> String {
                "stabil".to_string()
            }
        }

        let router_arc: Arc<dyn DriftStatusProvider> = Arc::new(DummyRouter);
        let calibrator_arc = Arc::new(parking_lot::Mutex::new(
            contextra_rank::IsotonicCalibrator::new(5, 100),
        ));
        // Warmup calibrator so ECE is populated
        {
            let mut cal = calibrator_arc.lock();
            for i in 0..10 {
                cal.record_outcome(i as f32 / 10.0, i > 5);
            }
            cal.force_rebuild();
        }

        db.set_router(Arc::downgrade(&router_arc));
        db.set_calibrator(Arc::downgrade(&calibrator_arc));

        let stats_attached = db.stats().await.expect("stats");
        assert_eq!(stats_attached.drift_status, "stabil");
        assert!(stats_attached.calibration_ece.is_some());
        assert!(stats_attached.last_calibration_at.is_some());

        // 3. Drop strong references (simulating shutdown) -> stats() must not panic and fallback gracefully
        drop(router_arc);
        drop(calibrator_arc);

        let stats_dropped = db.stats().await.expect("stats after drop");
        assert_eq!(stats_dropped.drift_status, "nicht verfügbar");
        assert_eq!(stats_dropped.calibration_ece, None);
        assert_eq!(stats_dropped.last_calibration_at, None);
        assert_eq!(stats_dropped.pid_pool_size, None);
    }

    // Prüft die End-to-End-Integration von Insert, Relate, Search, Update und Delete.
    #[tokio::test]
    async fn test_integration_end_to_end() {
        let (db, _tmp) = test_db(4).await;

        // 1. Insert
        db.insert(
            "agent-1",
            &[1.0, 0.5, 0.0, 0.0],
            Some(json!({"type": "agent"})),
        )
        .await
        .expect("insert agent"); // expect
        db.insert(
            "task-1",
            &[0.9, 0.6, 0.0, 0.0],
            Some(json!({"type": "task"})),
        )
        .await
        .expect("insert task"); // expect
        db.insert(
            "task-2",
            &[0.0, 0.0, 1.0, 0.5],
            Some(json!({"type": "task"})),
        )
        .await
        .expect("insert task 2"); // expect

        // 2. Relate
        db.relate("agent-1", "task-1", "assigned_to")
            .await
            .expect("relate 1"); // expect
        db.relate("agent-1", "task-2", "assigned_to")
            .await
            .expect("relate 2"); // expect

        // 3. Search
        let results = db.search(&[1.0, 0.5, 0.0, 0.0], 2).await.expect("search"); // expect
        assert_eq!(results[0].id, "agent-1"); // Exactly matches
        assert_eq!(results[1].id, "task-1"); // Close match

        // 4. Update
        db.update(
            "task-1",
            &[0.1, 0.1, 0.9, 0.9],
            Some(json!({"type": "task", "status": "done"})),
        )
        .await
        .expect("update task"); // expect

        // 5. Scan prefix
        let edges = db
            .scan_prefix("__rel:agent-1:assigned_to:", None)
            .await
            .expect("scan"); // expect
        assert_eq!(edges.len(), 2);

        // 6. Delete
        db.delete("agent-1").await.expect("delete"); // expect

        // 7. Verify empty search and missing doc
        let get_agent = db.get("agent-1").await.expect("get"); // expect
        assert!(get_agent.is_none());
        assert_eq!(db.len().await.expect("len"), 2); // 3 inserted, 1 deleted // expect
    }

    // Prüft, dass nebenläufige Aufrufe von db.collection() dieselbe Collection-Instanz (Arc) liefern (Idempotenz).
    #[tokio::test]
    async fn test_concurrent_collection_idempotency() {
        let (db, _tmp) = test_db(4).await;
        let db = Arc::new(db);
        let mut handles = Vec::new();

        for _ in 0..10 {
            let db_clone = db.clone();
            handles.push(tokio::spawn(async move {
                db_clone.collection("c").await.expect("collection c") // expect
            }));
        }

        let mut cols = Vec::new();
        for handle in handles {
            cols.push(handle.await.expect("join handle")); // expect
        }

        let first = &cols[0];
        for col in &cols[1..] {
            assert!(
                Arc::ptr_eq(first, col),
                "collection(\"c\") must return the exact same Arc instance"
            );
        }
    }

    // Prüft, dass Kollektionen vollständig voneinander isoliert sind.
    #[tokio::test]
    async fn collections_are_isolated() {
        let (db, _tmp) = test_db(4).await;
        let vec = vec![1.0, 0.0, 0.0, 0.0];
        let col_a = db.collection("alpha").await.unwrap(); // unwrap
        let col_b = db.collection("beta").await.unwrap(); // unwrap
        col_a.insert("doc1", &vec, None).await.unwrap(); // unwrap
        let results = col_b.search(&vec, 10).await.unwrap(); // unwrap
        assert!(
            results.is_empty(),
            "Collection B must not see Collection A's data"
        );
    }

    // Prüft Daten- und Schlüsselraum-Isolation über separate Kollektionen hinweg.
    #[tokio::test]
    async fn test_collections_are_isolated() {
        let (db, _tmp) = test_db(4).await;
        let col_a = db.collection("a").await.expect("col a"); // expect
        let col_b = db.collection("b").await.expect("col b"); // expect

        col_a
            .insert("k1", &[1.0, 0.0, 0.0, 0.0], Some(json!({"val": "a"})))
            .await
            .expect("ins a"); // expect
        col_b
            .insert("k1", &[0.0, 1.0, 0.0, 0.0], Some(json!({"val": "b"})))
            .await
            .expect("ins b"); // expect

        let res_a = col_a.get("k1").await.expect("get a").expect("exists"); // expect
        let res_b = col_b.get("k1").await.expect("get b").expect("exists"); // expect

        assert_eq!(res_a.metadata.expect("test")["val"], "a"); // expect
        assert_eq!(res_b.metadata.expect("test")["val"], "b"); // expect

        let search_a = col_a
            .search(&[1.0, 0.0, 0.0, 0.0], 1)
            .await
            .expect("search a"); // expect
        assert_eq!(search_a.len(), 1);
        assert_eq!(search_a[0].id, "k1");
        assert_eq!(search_a[0].metadata.as_ref().expect("test")["val"], "a"); // expect
    }

    // Prüft Persistenz und Wiederherstellung nach Schließen und erneutem Öffnen mit 100 Dokumenten.
    #[tokio::test]
    async fn test_close_and_reopen_100_docs() {
        let tmp = TempDir::new().expect("temp dir"); // expect
        let path = tmp.path().to_path_buf();
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };

        {
            let db = Contextra::open_with_config(&path, config.clone())
                .await
                .expect("open 1"); // expect
            for i in 0..100 {
                let id = format!("doc-{}", i);
                let val = (i as f32) / 100.0;
                db.insert(&id, &[val, 1.0 - val, 0.0, 0.0], Some(json!({"idx": i})))
                    .await
                    .expect("insert"); // expect
            }
            db.close().await.expect("close"); // expect
        }

        {
            let db = Contextra::open_with_config(&path, config)
                .await
                .expect("open 2"); // expect
            assert_eq!(db.len().await.expect("len"), 100); // expect
            for i in 0..100 {
                let id = format!("doc-{}", i);
                let doc = db.get(&id).await.expect("get").expect("exists"); // expect
                assert_eq!(doc.id, id);
                assert_eq!(doc.metadata.expect("valid")["idx"], i); // expect
            }
            let results = db.search(&[0.5, 0.5, 0.0, 0.0], 10).await.expect("search"); // expect
            assert_eq!(results.len(), 10);
        }
    }

    // Prüft Schließen und Wiederöffnen mit Einzelwerten.
    #[tokio::test]
    async fn test_close_and_reopen() {
        let tmp = TempDir::new().expect("temp dir"); // expect
        let path = tmp.path().to_path_buf();
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };

        {
            let db = Contextra::open_with_config(&path, config.clone())
                .await
                .expect("open 1"); // expect
            db.insert("doc-1", &[1.0, 0.0, 0.0, 0.0], Some(json!({"v": 1})))
                .await
                .expect("insert"); // expect
            db.close().await.expect("close"); // expect
        }

        {
            let db = Contextra::open_with_config(&path, config)
                .await
                .expect("open 2"); // expect
            let doc = db.get("doc-1").await.expect("get").expect("exists"); // expect
            assert_eq!(doc.id, "doc-1");
            assert_eq!(doc.metadata.expect("valid")["v"], 1); // expect
        }
    }

    // Prüft, dass drop_collection alle Daten und Indizes der Kollektion vollständig entfernt.
    #[tokio::test]
    async fn test_drop_removes_all_data() {
        let (db, _tmp) = test_db(4).await;
        let tenant_id = TenantId::try_new(1).expect("tenant_id"); // expect
        let col = db
            .collection_for_tenant("drop-me", tenant_id)
            .await
            .expect("col"); // expect
        col.insert("k1", &[1.0, 0.0, 0.0, 0.0], None)
            .await
            .expect("ins"); // expect

        db.drop_collection("drop-me", tenant_id, &[0u8; 32])
            .await
            .expect("drop"); // expect

        let col2 = db
            .collection_for_tenant("drop-me", tenant_id)
            .await
            .expect("re-create"); // expect
        assert_eq!(col2.len().await, 0);
        assert!(col2.get("k1").await.expect("get").is_none()); // expect
    }

    // Prüft Abwärtskompatibilität der Standard-Kollektion ("default").
    #[tokio::test]
    async fn test_default_collection_compat() {
        let (db, _tmp) = test_db(4).await;
        db.insert("k", &[1.0, 0.0, 0.0, 0.0], Some(json!({"v": 1})))
            .await
            .expect("ins"); // expect

        let doc = db.get("k").await.expect("get").expect("exists"); // expect
        assert_eq!(doc.id, "k");

        let results = db.search(&[1.0, 0.0, 0.0, 0.0], 1).await.expect("search"); // expect
        assert_eq!(results[0].id, "k");
    }

    // Prüft das Auflisten aller vorhandenen Kollektionsnamen.
    #[tokio::test]
    async fn test_list_collections() {
        let (db, _tmp) = test_db(4).await;
        db.collection("c1").await.expect("c1"); // expect
        db.collection("c2").await.expect("c2"); // expect
        db.collection("c3").await.expect("c3"); // expect

        let list = db.list_collections().await.expect("list"); // expect
        assert!(list.contains(&"default".to_string()));
        assert!(list.contains(&"c1".to_string()));
        assert!(list.contains(&"c2".to_string()));
        assert!(list.contains(&"c3".to_string()));
        assert_eq!(list.len(), 4);
    }

    // Prüft Standardwerte von ContextraConfig (ONNX-Modell und 768 Dimensionen).
    #[test]
    fn test_contextra_config_defaults_to_onnx_backend() {
        let config = ContextraConfig::default();
        assert_eq!(config.dimension, 768);
        assert_eq!(
            config.embedding_backend,
            EmbeddingBackend::Onnx {
                model_name: "nomic-embed-text".to_string(),
                cache_dir: None,
            }
        );
        assert_eq!(
            EmbeddingBackend::default(),
            EmbeddingBackend::Onnx {
                model_name: "nomic-embed-text".to_string(),
                cache_dir: None,
            }
        );
    }

    // Prüft ONNX-Embedding-Integration End-to-End mit Modell-Fixtures.
    #[tokio::test]
    #[cfg(feature = "onnx")]
    async fn test_onnx_default_backend_e2e_with_fixture() {
        let tmp = TempDir::new().expect("temp dir");
        let cache_tmp = TempDir::new().expect("cache temp dir");
        let model_dir = cache_tmp.path().join("nomic-embed-text");
        std::fs::create_dir_all(&model_dir).expect("create model dir");

        let fixture_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("contextra-embed")
            .join("tests")
            .join("fixtures");

        std::fs::copy(fixture_dir.join("model.onnx"), model_dir.join("model.onnx"))
            .expect("copy model");
        std::fs::copy(
            fixture_dir.join("tokenizer.json"),
            model_dir.join("tokenizer.json"),
        )
        .expect("copy tokenizer");

        let config = ContextraConfig {
            dimension: 32, // Fixture BERT model output dim is 32
            embedding_backend: EmbeddingBackend::Onnx {
                model_name: "nomic-embed-text".to_string(),
                cache_dir: Some(cache_tmp.path().to_path_buf()),
            },
            ..Default::default()
        };

        let db = Contextra::open_with_config(tmp.path(), config)
            .await
            .expect("open_with_config");

        assert!(db.embedder.read().is_some());
        let embedder = db.embedder.read().as_ref().cloned().expect("embedder");
        let vec = embedder.embed("test document").await.expect("embed test");
        assert_eq!(vec.len(), 32);
    }

    // Prüft, dass Wiederöffnen mit abweichender Vektordimension fehlschlägt.
    #[tokio::test]
    async fn test_open_dimension_mismatch_fails() -> contextra_types::Result<()> {
        let dir = tempfile::tempdir()
            .map_err(|e| contextra_types::ContextraError::InvalidInput(e.to_string()))?;
        let config_768 = ContextraConfig {
            dimension: 768,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config_768).await?;
        db.close().await?;

        // Zweites Öffnen mit falscher Dimension muss früh fehlschlagen
        let config_1536 = ContextraConfig {
            dimension: 1536,
            ..Default::default()
        };
        let result = Contextra::open_with_config(dir.path(), config_1536).await;
        assert!(result.is_err());
        let err_msg = match result {
            Err(e) => e.to_string(),
            Ok(_) => panic!("expected error"),
        };
        assert!(err_msg.contains("Dimension mismatch"));
        Ok(())
    }

    // Prüft Serialisierung und Deserialisierung von ProvenanceRecord im JSON-Roundtrip.
    #[test]
    fn test_provenance_record_serialization_roundtrip() {
        let p = ProvenanceRecord {
            vector_distance: Some(0.42),
            bm25_score: Some(1.23),
            graph_score: None,
            rerank_score: None,
            signal_ranks: {
                let mut m = ahash::AHashMap::new();
                m.insert("vector".to_string(), 1u32);
                m.insert("bm25".to_string(), 3u32);
                m
            },
            source_collection: Some("test_col".to_string()),
            index_type: Some("hnsw".to_string()),
            signal_contributions: ahash::AHashMap::new(),
            coherence_bonus: 0.0,
        };
        let json = serde_json::to_string(&p).expect("serialize");
        let back: ProvenanceRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.vector_distance, Some(0.42));
        assert_eq!(back.bm25_score, Some(1.23));
        assert_eq!(back.graph_score, None);
        assert_eq!(back.source_collection.as_deref(), Some("test_col"));
    }

    // Prüft Serialisierung von SearchResult mit befülltem ProvenanceRecord.
    #[test]
    fn test_search_result_with_provenance_serialization() {
        let sr = SearchResult {
            id: "doc1".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec!["vector".to_string()],
            provenance: Some(ProvenanceRecord {
                source_collection: Some("my_col".to_string()),
                ..Default::default()
            }),
        };
        let json = serde_json::to_string(&sr).expect("serialize");
        assert!(json.contains("provenance"));
        assert!(json.contains("my_col"));
    }

    // Prüft, dass SearchResult ohne Provenance das entsprechende Feld im JSON auslässt.
    #[test]
    fn test_search_result_without_provenance_serialization_omits_field() {
        let sr = SearchResult {
            id: "doc2".to_string(),
            score: 0.5,
            metadata: None,
            matched_signals: vec![],
            provenance: None,
        };
        let json = serde_json::to_string(&sr).expect("serialize");
        // provenance: None → Feld soll komplett fehlen (skip_serializing_if)
        assert!(
            !json.contains("provenance"),
            "provenance=None soll im JSON weggelassen werden: {json}"
        );
    }

    // Prüft die Invariante INV-PROV-1: Summe der Signalbeiträge entspricht dem RRF-Score.
    #[tokio::test]
    async fn test_provenance_rrf_sum_invariant() {
        let (db, _tmp) = test_db(4).await;
        let col = db.collection("prov_test").await.expect("collection");

        for i in 0..10 {
            let id = format!("doc-{}", i);
            let val = (i as f32) / 10.0;
            col.insert(
                &id,
                &[val, 1.0 - val, 0.0, 0.0],
                Some(json!({ "text": format!("rust memory system {}", i) })),
            )
            .await
            .expect("insert");
        }

        let results = col
            .query()
            .text("rust memory")
            .embedding([0.5, 0.5, 0.0, 0.0])
            .include_provenance(true)
            .k(5)
            .execute()
            .await
            .expect("search");

        assert!(!results.is_empty(), "Results must not be empty");

        for result in &results {
            let prov = result
                .provenance
                .as_ref()
                .expect("Provenance must be present when include_provenance=true");
            let sum_contrib: f32 = prov
                .signal_contributions
                .values()
                .map(|c| c.rrf_contribution)
                .sum();
            assert!(
                (sum_contrib - result.score).abs() < 1e-6,
                "INV-PROV-1 verletzt: Summe der Signal-Beiträge ({}) ≠ RRF-Score ({})",
                sum_contrib,
                result.score
            );
        }
    }

    // Prüft, dass Suchergebnisse bei aktivierter Provenance ein befülltes Provenance-Objekt enthalten.
    #[tokio::test]
    async fn test_search_results_have_provenance_is_some() {
        let (db, _tmp) = test_db(4).await;
        let col = db.collection("search_prov_test").await.expect("collection");

        col.insert(
            "doc-1",
            &[1.0, 0.0, 0.0, 0.0],
            Some(json!({"text": "rust search provenance test"})),
        )
        .await
        .expect("insert");

        let vec_results = col
            .search(&[1.0, 0.0, 0.0, 0.0], 1)
            .await
            .expect("vector search");
        assert_eq!(vec_results.len(), 1);
        assert!(
            vec_results[0].provenance.is_some(),
            "Vector search results must contain provenance record"
        );
        let prov = vec_results[0].provenance.as_ref().expect("provenance");
        assert_eq!(prov.source_collection.as_deref(), Some("search_prov_test"));
        assert_eq!(prov.index_type.as_deref(), Some("hnsw"));

        let text_results = col
            .query()
            .text("rust search")
            .embedding([1.0, 0.0, 0.0, 0.0])
            .include_provenance(true)
            .k(1)
            .execute()
            .await
            .expect("text search");
        assert_eq!(text_results.len(), 1);
        assert!(
            text_results[0].provenance.is_some(),
            "Text search results must contain provenance record"
        );
    }
}
