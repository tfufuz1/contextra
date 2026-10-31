//! Integrationstests für `contextra-core`.
//!
//! # Zweck dieser Test-Suite
//! Überprüft das Zusammenspiel der re-exportierten Kernkomponenten:
//! - [`contextra_mvcc::tx_buffer::TxBuffer`]: Transaktions-Staging und Isolation uncommitted Schreiboperationen.
//! - [`contextra_mvcc::snapshot::SnapshotRegistry`]: Erfassung aktiver Leseschnappschüsse und GC-Wasserzeichen (`min_active_seqno`).
//! - [`contextra_types::types::DistanceMetric`]: Berechnung von Vektordistanzen (z. B. Kosinus-Distanz).
//!
//! # Getestete Invarianten
//! 1. Transaktionsisolation: Operationen in Tx 1 sind erst nach dem Drainen sichtbar und beeinflussen Tx 2 nicht.
//! 2. Snapshot-Pinning: Solange ein `SnapshotGuard` gehalten wird, kann die minimale aktive Sequenznummer nicht vorrücken.
//! 3. RAII-Bereinigung: Nach dem Verwerfen (`drop`) des Guards kehrt `min_active_seqno` auf `u64::MAX` zurück.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{
    snapshot::SnapshotRegistry,
    tx_buffer::{IndexOp, TxBuffer},
    types::{DocId, TxId},
};
use std::sync::Arc;
use std::time::Duration;

// ANCHOR[contextra-core:TEST-INT-001]
/// Prüft die isolierte Ausführung zweier Transaktionen im `TxBuffer` sowie das Zusammenspiel mit der `SnapshotRegistry`.
///
/// # Ablauf von `test_integration_tx_buffer_and_snapshots`:
/// 1. `TxBuffer` und `SnapshotRegistry` initialisieren.
/// 2. Transaktion 1 (`tx1`) starten und eine Dokumenteinfügung stagen.
/// 3. Leseschnappschuss bei Sequenznummer 101 registrieren (`guard`).
/// 4. Transaktion 2 (`tx2`) starten und eine separate Einfügung stagen.
/// 5. Prüfen, dass beide Transaktionen isoliert im Puffer liegen (`len == 2`).
/// 6. `tx1` entleeren (`drain`) und verifizieren, dass nur Dokument 1 zurückgegeben wird.
/// 7. Prüfen, dass `min_active_seqno` durch `guard` bei 101 gehalten wird.
/// 8. `guard` verwerfen (`drop`) und prüfen, dass `min_active_seqno` auf `u64::MAX` springt.
/// 9. `tx2` verwerfen (`discard`) und sicherstellen, dass der Puffer leer ist.
#[test]
fn test_integration_tx_buffer_and_snapshots() {
    // Phase 1: Set up TX Buffer and Snapshot Registry
    let buffer = TxBuffer::<String>::new_with_config(8, Duration::from_secs(60));
    let registry = Arc::new(SnapshotRegistry::new());

    // Transaction 1
    let tx1 = TxId::new(100);
    buffer.begin(tx1);
    let _ = buffer.stage(
        tx1,
        IndexOp::Insert {
            doc_id: DocId::new(1),
            data: "first_doc".to_string(),
        },
    );

    // Take a snapshot
    let guard = registry.register(101);

    // Transaction 2
    let tx2 = TxId::new(102);
    buffer.begin(tx2);
    let _ = buffer.stage(
        tx2,
        IndexOp::Insert {
            doc_id: DocId::new(2),
            data: "second_doc".to_string(),
        },
    );

    // Assert isolated tx
    assert_eq!(buffer.len(), 2);
    let drained_tx1 = buffer.drain(tx1);
    assert_eq!(drained_tx1.len(), 1);
    // NOTE: Indexierung `drained_tx1[0]` löst clippy::indexing_slicing aus; Code wird laut Regel 1 nicht verändert.
    assert_eq!(drained_tx1[0].doc_id(), DocId::new(1));

    // Snapshot guard is still holding 101 as minimum sequence number
    assert_eq!(registry.min_active_seqno(), 101);
    drop(guard);
    assert_eq!(registry.min_active_seqno(), u64::MAX);

    // Clear tx2
    buffer.discard(tx2);
    assert!(buffer.is_empty());
}

// ANCHOR[contextra-core:TEST-INT-002]
/// Prüft die korrekte Berechnung der Kosinus-Distanz zweier identischer Vektoren über den `DistanceMetric`-Port.
///
/// # Ablauf von `test_domain_metrics_integration`:
/// 1. Zwei identische 3D-Vektoren definieren.
/// 2. Kosinus-Distanz über `DistanceMetric::Cosine.compute()` berechnen.
/// 3. Verifizieren, dass das Ergebnis innerhalb der Fließkomma-Toleranz (< 0.0001) exakt 0.0 entspricht.
#[test]
fn test_domain_metrics_integration() {
    use contextra_core::types::DistanceMetric;

    let a = [1.0, 0.0, 0.0];
    let b = [1.0, 0.0, 0.0];

    // Exact match in Cosine means distance is 0.0 ideally
    let dist = DistanceMetric::Cosine.compute(&a, &b).unwrap();
    assert!(dist < 0.0001); // floating point tolerance
}
