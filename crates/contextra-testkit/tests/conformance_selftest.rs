//! # Konformitäts-Selbsttests (`conformance_selftest`)
//!
//! Diese Testdatei validiert die Konformitätsprüfsuiten selbst über Mutation-Testing
//! (Reference-Green & Mutant-Red).
//!
//! ## Testprinzip
//! 1. **Referenzimplementierungen (Green):** Standard-Mocks wie `InMemoryStorageEngine`,
//!    `InMemoryVectorIndex` etc. müssen die Suiten fehlerfrei bestehen (`assert!(res.is_ok())`).
//! 2. **Mutanten mit bekannten Fehlern (Red):** Absichtlich fehlerhafte Implementierungen
//!    (z. B. No-Op-Delete, nicht-monotone Clock, NaN-RNG) müssen zuverlässig abgefangen
//!    und mit den exakten Testfall-Namen abgewiesen werden (`assert_eq!(failure.test_case, ...)`).

#![allow(clippy::panic, clippy::unwrap_used)]

use contextra_ports::{
    BoxFuture, Clock, GraphIndex, GraphIndexStats, MetricsSink, Rng, SeededRng, StorageEngine,
    StorageStats, TextIndex, TextIndexStats, VectorIndex, VectorIndexStats,
};
use contextra_testkit::conformance::*;
use contextra_testkit::{
    InMemoryGraphIndex, InMemoryStorageEngine, InMemoryTextIndex, InMemoryVectorIndex, ManualClock,
    TestMetricsSink,
};
use contextra_types::{DocId, Edge, Entity, EntityId, Result, ScoredDocument, TxId};
use std::sync::atomic::{AtomicU64, Ordering};

// ============================================================================
// MUTANT IMPLEMENTATIONS FOR MUTATION PROOF TESTS
// ============================================================================

// ANCHOR[contextra-testkit:SELFTEST-MUTANT-STORE-001]
/// Mutante StorageEngine, bei der Löschoperationen absichtlich ignoriert werden (No-Op).
#[derive(Clone, Debug)]
struct BuggyDeleteStorageEngine(InMemoryStorageEngine);

impl StorageEngine for BuggyDeleteStorageEngine {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        self.0.get(key)
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        self.0.get_at_seq(key, seq)
    }

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        self.0.put(tx_id, key, value)
    }

    fn delete<'a>(&'a self, _tx_id: TxId, _key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        // BUG: Delete is a no-op!
        Box::pin(async move { Ok(()) })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.0.commit(tx_id)
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.0.rollback(tx_id)
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.0.rollback_to_tx(tx_id)
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        self.0.flush()
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        self.0.stats()
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        self.0.last_seq_no()
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        self.0.last_tx_id()
    }

    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        self.0.pin_checkpoint(seq_no)
    }

    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, Result<()>> {
        self.0.unpin_checkpoint(seq_no)
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.0.scan_prefix(prefix)
    }

    fn scan<'a>(
        &'a self,
        start: std::ops::Bound<&'a [u8]>,
        end: std::ops::Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        self.0.scan(start, end, limit)
    }
}

// ANCHOR[contextra-testkit:SELFTEST-MUTANT-VECTOR-001]
/// Mutante VectorIndex, bei dem Löschen eine No-Op ist.
#[derive(Clone, Debug)]
struct BuggyDeleteVectorIndex(InMemoryVectorIndex);

impl VectorIndex for BuggyDeleteVectorIndex {
    async fn insert(&self, tx: TxId, id: DocId, embedding: &[f32]) -> Result<()> {
        self.0.insert(tx, id, embedding).await
    }

    async fn delete(&self, _tx: TxId, _id: DocId) -> Result<()> {
        // BUG: Delete is a no-op!
        Ok(())
    }

    async fn commit(&self, tx: TxId) -> Result<()> {
        self.0.commit(tx).await
    }

    async fn rollback(&self, tx: TxId) -> Result<()> {
        self.0.rollback(tx).await
    }

    async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()> {
        self.0.rollback_to_tx(tx_id).await
    }

    async fn last_tx_id(&self) -> Result<TxId> {
        self.0.last_tx_id().await
    }

    async fn all_doc_ids(&self) -> Result<Vec<DocId>> {
        self.0.all_doc_ids().await
    }

    async fn search(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>> {
        self.0.search(query, k).await
    }

    async fn len(&self) -> usize {
        self.0.len().await
    }

    async fn stats(&self) -> Result<VectorIndexStats> {
        self.0.stats().await
    }
}

// ANCHOR[contextra-testkit:SELFTEST-MUTANT-TEXT-001]
/// Mutante TextIndex, bei dem Löschen eine No-Op ist.
#[derive(Clone, Debug)]
struct BuggyDeleteTextIndex(InMemoryTextIndex);

impl TextIndex for BuggyDeleteTextIndex {
    async fn insert(&self, tx: TxId, id: DocId, text: &str) -> Result<()> {
        self.0.insert(tx, id, text).await
    }

    async fn delete(&self, _tx: TxId, _id: DocId) -> Result<()> {
        // BUG: Delete is a no-op!
        Ok(())
    }

    async fn commit(&self, tx: TxId) -> Result<()> {
        self.0.commit(tx).await
    }

    async fn rollback(&self, tx: TxId) -> Result<()> {
        self.0.rollback(tx).await
    }

    async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()> {
        self.0.rollback_to_tx(tx_id).await
    }

    async fn last_tx_id(&self) -> Result<TxId> {
        self.0.last_tx_id().await
    }

    async fn search(&self, query: &str, k: usize) -> Result<Vec<ScoredDocument>> {
        self.0.search(query, k).await
    }

    async fn len(&self) -> usize {
        self.0.len().await
    }

    async fn stats(&self) -> Result<TextIndexStats> {
        self.0.stats().await
    }
}

// ANCHOR[contextra-testkit:SELFTEST-MUTANT-GRAPH-001]
/// Mutante GraphIndex, bei dem `remove_edge` eine No-Op ist.
#[derive(Clone, Debug)]
struct BuggyRemoveEdgeGraphIndex(InMemoryGraphIndex);

impl GraphIndex for BuggyRemoveEdgeGraphIndex {
    fn add_entity<'a>(&'a self, tx: TxId, entity: Entity) -> BoxFuture<'a, Result<()>> {
        self.0.add_entity(tx, entity)
    }

    fn add_edge<'a>(&'a self, tx: TxId, edge: Edge) -> BoxFuture<'a, Result<()>> {
        self.0.add_edge(tx, edge)
    }

    fn remove_edge<'a>(
        &'a self,
        _tx: TxId,
        _from: EntityId,
        _to: EntityId,
    ) -> BoxFuture<'a, Result<()>> {
        // BUG: remove_edge is a no-op!
        Box::pin(async move { Ok(()) })
    }

    fn commit<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>> {
        self.0.commit(tx)
    }

    fn rollback<'a>(&'a self, tx: TxId) -> BoxFuture<'a, Result<()>> {
        self.0.rollback(tx)
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        self.0.rollback_to_tx(tx_id)
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        self.0.last_tx_id()
    }

    fn neighbors<'a>(&'a self, start_node: EntityId) -> BoxFuture<'a, Result<Vec<EntityId>>> {
        self.0.neighbors(start_node)
    }

    fn traverse<'a>(
        &'a self,
        start_node: EntityId,
        max_hops: usize,
    ) -> BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        self.0.traverse(start_node, max_hops)
    }

    fn len<'a>(&'a self) -> BoxFuture<'a, usize> {
        self.0.len()
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<GraphIndexStats>> {
        self.0.stats()
    }
}

// ANCHOR[contextra-testkit:SELFTEST-MUTANT-CLOCK-001]
/// Mutante Uhr, deren monotone Zeit bei jedem Abruf sinkt.
#[derive(Debug)]
struct DecreasingClock(AtomicU64);

impl Clock for DecreasingClock {
    fn now_unix_nanos(&self) -> u64 {
        1_700_000_000_000_000_000
    }

    fn monotonic_nanos(&self) -> u64 {
        // BUG: Decreases by 100 on every call!
        self.0.fetch_sub(100, Ordering::SeqCst)
    }
}

// ANCHOR[contextra-testkit:SELFTEST-MUTANT-RNG-001]
/// Mutanter RNG, dessen `next_unit_f64` Werte außerhalb von [0.0, 1.0) liefert.
#[derive(Debug)]
struct OutOfBoundsRng(AtomicU64);

impl Rng for OutOfBoundsRng {
    fn next_u64(&self) -> u64 {
        self.0.fetch_add(1, Ordering::SeqCst)
    }

    fn fill_bytes(&self, dest: &mut [u8]) {
        dest.fill(0xAA);
    }

    fn next_unit_f64(&self) -> f64 {
        // BUG: Returns 2.5 (outside [0.0, 1.0))!
        2.5
    }
}

// ANCHOR[contextra-testkit:SELFTEST-MUTANT-METRICS-001]
/// Mutante MetricsSink, die beim Zählen absichtlich panickt.
#[derive(Debug)]
struct PanickingMetricsSink;

impl MetricsSink for PanickingMetricsSink {
    fn record_counter(&self, _name: &str, _value: u64, _labels: &[(&str, &str)]) {
        panic!("PanickingMetricsSink intentionally panicked");
    }

    fn record_gauge(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}

    fn record_histogram(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}
}

// ============================================================================
// SELF-TESTS: REFERENCE GREEN & MUTANT RED
// ============================================================================

// ANCHOR[contextra-testkit:SELFTEST-RUN-STO-GREEN-001]
// Prüft, dass die In-Memory-Speicher-Engine alle Verträge der StorageEngine-Suite besteht.
#[tokio::test]
async fn test_storage_engine_conformance_reference_green() {
    let res = run_storage_engine_suite(|| async { InMemoryStorageEngine::new() }).await;
    assert!(
        res.is_ok(),
        "InMemoryStorageEngine must pass conformance: {res:?}"
    );
}

// ANCHOR[contextra-testkit:SELFTEST-RUN-STO-RED-001]
// Prüft, dass die Mutante mit defektem Delete zuverlässig als Fehler gemeldet wird.
#[tokio::test]
async fn test_storage_engine_conformance_mutant_red() {
    let res = run_storage_engine_suite(|| async {
        BuggyDeleteStorageEngine(InMemoryStorageEngine::new())
    })
    .await;
    match res {
        Err(failure) => {
            assert_eq!(failure.suite_name, "StorageEngine");
            assert_eq!(failure.test_case, "put_get_delete");
        }
        Ok(_) => panic!("BuggyDeleteStorageEngine must fail conformance test suite"),
    }
}

// ANCHOR[contextra-testkit:SELFTEST-RUN-VEC-GREEN-001]
// Prüft, dass der In-Memory-Vektorindex alle Verträge der VectorIndex-Suite besteht.
#[tokio::test]
async fn test_vector_index_conformance_reference_green() {
    let res = run_vector_index_suite(|| async { InMemoryVectorIndex::new(3) }).await;
    assert!(
        res.is_ok(),
        "InMemoryVectorIndex must pass conformance: {res:?}"
    );
}

// ANCHOR[contextra-testkit:SELFTEST-RUN-VEC-RED-001]
// Prüft, dass der Vektor-Mutant mit No-Op-Delete als ConformanceFailure erkannt wird.
#[tokio::test]
async fn test_vector_index_conformance_mutant_red() {
    let res =
        run_vector_index_suite(|| async { BuggyDeleteVectorIndex(InMemoryVectorIndex::new(3)) })
            .await;
    match res {
        Err(failure) => {
            assert_eq!(failure.suite_name, "VectorIndex");
            assert_eq!(failure.test_case, "delete_tombstone");
        }
        Ok(_) => panic!("BuggyDeleteVectorIndex must fail conformance test suite"),
    }
}

// ANCHOR[contextra-testkit:SELFTEST-RUN-TXT-GREEN-001]
// Prüft, dass der In-Memory-Textindex alle Verträge der TextIndex-Suite erfüllt.
#[tokio::test]
async fn test_text_index_conformance_reference_green() {
    let res = run_text_index_suite(|| async { InMemoryTextIndex::new() }).await;
    assert!(
        res.is_ok(),
        "InMemoryTextIndex must pass conformance: {res:?}"
    );
}

// ANCHOR[contextra-testkit:SELFTEST-RUN-TXT-RED-001]
// Prüft, dass der Text-Mutant mit No-Op-Delete als ConformanceFailure erkannt wird.
#[tokio::test]
async fn test_text_index_conformance_mutant_red() {
    let res =
        run_text_index_suite(|| async { BuggyDeleteTextIndex(InMemoryTextIndex::new()) }).await;
    match res {
        Err(failure) => {
            assert_eq!(failure.suite_name, "TextIndex");
            assert_eq!(failure.test_case, "delete_never_retrievable");
        }
        Ok(_) => panic!("BuggyDeleteTextIndex must fail conformance test suite"),
    }
}

// ANCHOR[contextra-testkit:SELFTEST-RUN-GRP-GREEN-001]
// Prüft, dass der In-Memory-Graphindex alle Verträge der GraphIndex-Suite erfüllt.
#[tokio::test]
async fn test_graph_index_conformance_reference_green() {
    let res = run_graph_index_suite(|| async { InMemoryGraphIndex::new() }).await;
    assert!(
        res.is_ok(),
        "InMemoryGraphIndex must pass conformance: {res:?}"
    );
}

// ANCHOR[contextra-testkit:SELFTEST-RUN-GRP-RED-001]
// Prüft, dass der Graph-Mutant mit fehlendem remove_edge als ConformanceFailure erkannt wird.
#[tokio::test]
async fn test_graph_index_conformance_mutant_red() {
    let res =
        run_graph_index_suite(|| async { BuggyRemoveEdgeGraphIndex(InMemoryGraphIndex::new()) })
            .await;
    match res {
        Err(failure) => {
            assert_eq!(failure.suite_name, "GraphIndex");
            assert_eq!(failure.test_case, "remove_edge");
        }
        Ok(_) => panic!("BuggyRemoveEdgeGraphIndex must fail conformance test suite"),
    }
}

// ANCHOR[contextra-testkit:SELFTEST-RUN-CLK-GREEN-001]
// Prüft, dass die ManualClock alle Anforderungen der Clock-Suite besteht.
#[test]
fn test_clock_conformance_reference_green() {
    let clock = ManualClock::new(1000);
    let res = run_clock_suite(&clock);
    assert!(
        res.is_ok(),
        "ManualClock must pass clock conformance: {res:?}"
    );
}

// ANCHOR[contextra-testkit:SELFTEST-RUN-CLK-RED-001]
// Prüft, dass eine Uhr mit rückläufiger Zeit die Suite fehlschlagen lässt.
#[test]
fn test_clock_conformance_mutant_red() {
    let mutant_clock = DecreasingClock(AtomicU64::new(5000));
    let res = run_clock_suite(&mutant_clock);
    match res {
        Err(failure) => {
            assert_eq!(failure.suite_name, "Clock");
            assert_eq!(failure.test_case, "monotonic_non_decreasing");
        }
        Ok(_) => panic!("DecreasingClock must fail clock conformance"),
    }
}

// ANCHOR[contextra-testkit:SELFTEST-RUN-RNG-GREEN-001]
// Prüft, dass SeededRng alle Konformitätsprüfungen für Pseudozufall besteht.
#[test]
fn test_rng_conformance_reference_green() {
    let rng = SeededRng::new(12345);
    let res = run_rng_suite(&rng);
    assert!(res.is_ok(), "SeededRng must pass rng conformance: {res:?}");
}

// ANCHOR[contextra-testkit:SELFTEST-RUN-RNG-RED-001]
// Prüft, dass RNGs mit Werten außerhalb von [0, 1) als fehlerhaft erkannt werden.
#[test]
fn test_rng_conformance_mutant_red() {
    let mutant_rng = OutOfBoundsRng(AtomicU64::new(1));
    let res = run_rng_suite(&mutant_rng);
    match res {
        Err(failure) => {
            assert_eq!(failure.suite_name, "Rng");
            assert_eq!(failure.test_case, "next_unit_f64_bounds");
        }
        Ok(_) => panic!("OutOfBoundsRng must fail rng conformance"),
    }
}

// ANCHOR[contextra-testkit:SELFTEST-RUN-MET-GREEN-001]
// Prüft, dass TestMetricsSink die Konformitätsprüfung für Metriksenken besteht.
#[test]
fn test_metrics_sink_conformance_reference_green() {
    let sink = TestMetricsSink::new();
    let res = run_metrics_sink_suite(&sink);
    assert!(
        res.is_ok(),
        "TestMetricsSink must pass metrics conformance: {res:?}"
    );
}

// ANCHOR[contextra-testkit:SELFTEST-RUN-MET-RED-001]
// Prüft, dass panickende Metriksenken als ConformanceFailure mit 'record_panic' diagnostiziert werden.
#[test]
fn test_metrics_sink_conformance_mutant_red() {
    let panicking_sink = PanickingMetricsSink;
    let res = run_metrics_sink_suite(&panicking_sink);
    match res {
        Err(failure) => {
            assert_eq!(failure.suite_name, "MetricsSink");
            assert_eq!(failure.test_case, "record_panic");
        }
        Ok(_) => panic!("PanickingMetricsSink must fail metrics conformance"),
    }
}
