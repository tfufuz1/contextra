//! Robustheits- und Nebenläufigkeitstests für `contextra-core`.
//!
//! # Zweck dieser Test-Suite
//! Validiert die Ausfallsicherheit und Invarianten der Kernkomponenten unter extremen
//! Bedingungen und Nebenläufigkeit (Multithreading):
//! - Schutz vor ungültigen Fließkommawerten (`NaN`, `Infinity`) in `FusionWeights` und `DistanceMetric`.
//! - Thread-Sicherheit der `SnapshotRegistry` bei paralleler Registrierung und Deregistrierung.
//! - Grenzwertverhalten des `ResourceTracker` bei Speicherüberschreitung und Unterlauf.
//! - Nebenläufiges Reaping verwaister Transaktionen im `TxBuffer` unter Last.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_core::{
    snapshot::SnapshotRegistry,
    tx_buffer::{IndexOp, TxBuffer},
    types::{DistanceMetric, DocId, FusionWeights, ResourceBudget, ResourceTracker, TxId},
};
use std::sync::Arc;
use std::time::Duration;

// ANCHOR[contextra-core:TEST-ROB-001]
/// Prüft, dass `FusionWeights` bei der Initialisierung Fließkomma-Anomalien (`NaN`, `Infinity`) ablehnt (Fail-Closed).
///
/// # Ablauf von `test_fusion_weights_nan_and_inf_prevention`:
/// 1. Versuch, `FusionWeights` mit `f32::NAN` zu erzeugen -> muss fehlschlagen (`Err`).
/// 2. Versuch, `FusionWeights` mit `-f32::NAN` zu erzeugen -> muss fehlschlagen (`Err`).
/// 3. Versuch, `FusionWeights` mit `f32::INFINITY` zu erzeugen -> muss fehlschlagen (`Err`).
#[test]
fn test_fusion_weights_nan_and_inf_prevention() {
    // Expose the NaN vulnerability: FusionWeights should not allow NaN weights.
    let res_nan = FusionWeights::new(f32::NAN, 0.5, 0.5);
    assert!(res_nan.is_err(), "FusionWeights should reject NaN values");

    let res_neg_nan = FusionWeights::new(-f32::NAN, 0.5, 0.5);
    assert!(
        res_neg_nan.is_err(),
        "FusionWeights should reject negative NaN values"
    );

    // Expose the Inf vulnerability: FusionWeights should not allow Inf weights.
    let res_inf = FusionWeights::new(f32::INFINITY, 0.5, 0.5);
    assert!(
        res_inf.is_err(),
        "FusionWeights should reject Infinity values"
    );
}

// ANCHOR[contextra-core:TEST-ROB-002]
/// Prüft, dass Vektordistanzberechnungen (`Cosine`, `Euclidean`, `DotProduct`) bei ungültigen Eingaben (`NaN`, `Infinity`) abbrechen.
///
/// # Ablauf von `test_distance_metric_nan_inf_prevention`:
/// 1. `DistanceMetric::Cosine` mit `NaN` in Eingabevektor ausführen -> muss `Err` liefern.
/// 2. `DistanceMetric::Cosine` mit `Infinity` in Eingabevektor ausführen -> muss `Err` liefern.
/// 3. `DistanceMetric::Euclidean` mit `NaN` ausführen -> muss `Err` liefern.
/// 4. `DistanceMetric::DotProduct` mit `NaN` ausführen -> muss `Err` liefern.
#[test]
fn test_distance_metric_nan_inf_prevention() {
    let metric = DistanceMetric::Cosine;

    // Cosine with NaN in first vector
    let a = [f32::NAN, 0.0];
    let b = [1.0, 1.0];
    let res = metric.compute(&a, &b);
    assert!(
        res.is_err(),
        "Distance computation should fail if inputs contain NaN"
    );

    // Cosine with Inf in first vector
    let a = [f32::INFINITY, 0.0];
    let b = [1.0, 1.0];
    let res = metric.compute(&a, &b);
    assert!(
        res.is_err(),
        "Distance computation should fail if inputs contain Infinity"
    );

    // Euclidean with NaN
    let metric_e = DistanceMetric::Euclidean;
    let a = [0.0, f32::NAN];
    let b = [1.0, 1.0];
    let res = metric_e.compute(&a, &b);
    assert!(
        res.is_err(),
        "Euclidean distance computation should fail if inputs contain NaN"
    );

    // DotProduct with NaN
    let metric_dp = DistanceMetric::DotProduct;
    let a = [f32::NAN, 0.0];
    let b = [1.0, 1.0];
    let res = metric_dp.compute(&a, &b);
    assert!(
        res.is_err(),
        "DotProduct distance computation should fail if inputs contain NaN"
    );
}

// ANCHOR[contextra-core:TEST-ROB-003]
/// Prüft die Thread-Sicherheit und Konsistenz von `SnapshotRegistry` unter hoher paralleler Last.
///
/// # Ablauf von `test_snapshot_registry_robustness_and_concurrency`:
/// 1. `SnapshotRegistry` in einem `Arc` initialisieren.
/// 2. 10 parallele Threads starten, die jeweils 1.000 Snapshots mit monoton steigenden Sequenznummern registrieren und sofort freigeben.
/// 3. Während der Ausführung validieren, dass `min_active_seqno()` stets kleiner oder gleich der registrierten Sequenznummer ist.
/// 4. Auf Beendigung aller Threads warten (`join`).
/// 5. Verifizieren, dass nach Beendigung aller Guards das Wasserzeichen wieder auf `u64::MAX` zurückgesetzt ist.
#[test]
fn test_snapshot_registry_robustness_and_concurrency() {
    let registry = Arc::new(SnapshotRegistry::new());
    let thread_count = 10;
    let iterations = 1000;
    let mut handles = Vec::new();

    for i in 0..thread_count {
        let r = registry.clone();
        handles.push(std::thread::spawn(move || {
            for j in 0..iterations {
                let seq = (i * iterations + j) as u64;
                let guard = r.register(seq);
                assert!(r.min_active_seqno() <= seq);
                drop(guard);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(registry.min_active_seqno(), u64::MAX);
}

// ANCHOR[contextra-core:TEST-ROB-004]
/// Prüft das Verhalten von `ResourceTracker` bei extremen Werten und Budgetüberschreitungen.
///
/// # Ablauf von `test_resource_tracker_edge_cases`:
/// 1. Tracker mit Speicherbudget von 1000 Bytes erstellen.
/// 2. Versuch, `u64::MAX` Bytes zu allozieren -> muss mit Fehler abgewiesen werden (Überlaufschutz).
/// 3. Versuch, 1001 Bytes (Budgetüberschreitung) zu allozieren -> muss mit Fehler abgewiesen werden.
/// 4. Freigabe von `u64::MAX` Bytes ausführen -> saturierender Schutz verhindert Unterlauf, Verbrauch bleibt 0.
#[test]
fn test_resource_tracker_edge_cases() {
    let budget = ResourceBudget { memory_limit: 1000 };
    let tracker = ResourceTracker::new(budget);

    // consume extreme values
    assert!(tracker.consume_memory(u64::MAX).is_err());
    assert!(tracker.consume_memory(1001).is_err());

    // release extreme values
    tracker.release_memory(u64::MAX);
    assert_eq!(tracker.memory_used(), 0);
}

// ANCHOR[contextra-core:TEST-ROB-005]
/// Prüft das nebenläufige Bereinigen verwaister Transaktionen (`reap_orphans`) im `TxBuffer` unter Schreiblast.
///
/// # Ablauf von `test_tx_buffer_orphan_reaper_concurrency`:
/// 1. `TxBuffer` mit kurzem Timeout (5 ms) konfigurieren.
/// 2. 5 Writer-Threads starten, die Transaktionen beginnen und Index-Operationen stagen, aber nicht committen.
/// 3. Parallel einen Reaper-Thread starten, der periodisch `reap_orphans()` aufruft.
/// 4. Auf Beendigung aller Writer- und Reaper-Threads warten.
/// 5. Nach einer Ruhepause einen finalen Reaper-Lauf ausführen und prüfen, dass alle verwaisten Transaktionen bereinigt wurden (`is_empty() == true`).
#[test]
fn test_tx_buffer_orphan_reaper_concurrency() {
    let buffer = Arc::new(TxBuffer::<String>::new_with_config(
        8,
        Duration::from_millis(5),
    ));
    let thread_count = 5;
    let iterations = 100;
    let mut handles = Vec::new();

    // Spawn writers staging transactions
    for t in 0..thread_count {
        let b = buffer.clone();
        handles.push(std::thread::spawn(move || {
            for i in 0..iterations {
                let tx = TxId::new((t * iterations + i) as u64);
                b.begin(tx);
                let _ = b.stage(
                    tx,
                    IndexOp::Insert {
                        doc_id: DocId::from(i as u64),
                        data: "test".to_string(),
                    },
                );
                std::thread::sleep(Duration::from_micros(100));
            }
        }));
    }

    // Spawn reaper
    let b_reap = buffer.clone();
    let reaper_handle = std::thread::spawn(move || {
        for _ in 0..50 {
            let _reaped = b_reap.reap_orphans();
            std::thread::sleep(Duration::from_millis(2));
        }
    });

    for h in handles {
        h.join().unwrap();
    }
    reaper_handle.join().unwrap();

    // After letting it rest and reaping one last time, it should be clean
    std::thread::sleep(Duration::from_millis(10));
    buffer.reap_orphans();
    assert!(
        buffer.is_empty(),
        "Orphan reaper should eventually clean everything"
    );
}
