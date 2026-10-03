#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

// FILE-CONTEXT
// ZWECK: Vertikaler Pfad-Test 3 (Teil 14.3 v17), Bezug B-16: Schreiblast parallel zu einem langen offenen Snapshot darf SSI nicht beeinträchtigen oder ausfallen lassen.
// INVARIANTEN: Greift ausschließlich über die öffentliche Facade (contextra::*) zu; Zero-Panic in Produktion; Timeout-geschützt.

mod common;

use common::{create_test_db, dummy_metadata, dummy_vector};
use contextra::ContextraError;
use serde_json::json;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::timeout;

/// Helper function to determine if an error is a documented, expected SSI or OCC conflict.
fn is_expected_ssi_conflict(err: &ContextraError) -> bool {
    if err.is_occ_conflict() {
        return true;
    }
    match err {
        ContextraError::Conflict(_) | ContextraError::StaleRead(_) => true,
        ContextraError::Transaction(msg) => {
            msg.contains("Conflict")
                || msg.contains("Serializable isolation")
                || msg.contains("isolation violation")
        }
        _ => false,
    }
}

/// Vertikaler Pfad-Test 3 (Teil 14.3 v17), Bezug B-16:
/// Schreiblast parallel zu einem langen offenen Snapshot darf SSI nicht unbrauchbar machen.
#[tokio::test]
async fn vertical_path_test_3_ssi_under_write_load_b16() -> Result<(), Box<dyn std::error::Error>> {
    // Timeout-Schutz gegen Deadlocks, Paniks oder unbegrenztes Hängen (max 120 Sekunden)
    let test_execution = timeout(Duration::from_secs(120), async {
        let tmp = TempDir::new()?;
        let dim = 16;
        let db = Arc::new(create_test_db(&tmp, dim).await?);
        let coll = db.collection("ssi_vertical_path_col").await?;

        // 1. Initialzustand herstellen: Basisdokumente einfügen
        let base_doc_count = 10;
        for i in 0..base_doc_count {
            let doc_id = format!("base_doc_{i}");
            coll.insert(
                &doc_id,
                &dummy_vector(dim, 0.1 * (i as f32 + 1.0)),
                Some(json!({ "initial_val": i, "version": 1 })),
            )
            .await?;
        }

        // 2. Lang laufenden Snapshot öffnen (Snapshot-Sequenz sichern)
        let long_snapshot_seq = coll.snapshot_seq().await?;

        // Verifizieren, dass der Snapshot vor Beginn der hohen Schreiblast alle Basisdokumente sieht
        for i in 0..base_doc_count {
            let doc_id = format!("base_doc_{i}");
            let snap_doc = coll.get_at_snapshot(&doc_id, long_snapshot_seq).await?;
            assert!(
                snap_doc.is_some(),
                "Base doc '{doc_id}' must be present at long snapshot seq {long_snapshot_seq}"
            );
            assert_eq!(
                snap_doc
                    .unwrap()
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("version"))
                    .and_then(|v| v.as_u64()),
                Some(1)
            );
        }

        // 3. Parallele Tokio-Tasks starten, um kontinuierliche hohe Schreiblast über tausende Transaktionen zu treiben
        let task_count = 4;
        let txs_per_task = 250; // Total 1.000 Transaktionen
        let total_tx_target = task_count * txs_per_task;

        let successful_commits = Arc::new(AtomicUsize::new(0));
        let expected_conflicts = Arc::new(AtomicUsize::new(0));
        let unexpected_errors = Arc::new(AtomicUsize::new(0));

        let start_time = Instant::now();

        let mut handles = Vec::new();

        for task_idx in 0..task_count {
            let coll_clone = coll.clone();
            let succ_counter = Arc::clone(&successful_commits);
            let conf_counter = Arc::clone(&expected_conflicts);
            let err_counter = Arc::clone(&unexpected_errors);

            let handle = tokio::spawn(async move {
                for tx_i in 0..txs_per_task {
                    let doc_id = format!("doc_task_{task_idx}_{tx_i}");
                    let vec = dummy_vector(dim, (task_idx + 1) as f32 * 0.05);
                    let meta = dummy_metadata("write_load", tx_i);

                    // Wechselweise Inserts und Updates auf denselben/neuen Keys
                    let res = if tx_i % 2 == 0 {
                        coll_clone.insert(&doc_id, &vec, Some(meta)).await
                    } else {
                        // Existenten oder Basis-Key updaten
                        let target_id = format!("base_doc_{}", tx_i % base_doc_count);
                        coll_clone.update(&target_id, &vec, Some(meta)).await
                    };

                    match res {
                        Ok(()) => {
                            succ_counter.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(ref err) if is_expected_ssi_conflict(err) => {
                            // Dokumentierter/erwarteter SSI-Konflikt
                            eprintln!("Expected SSI conflict in task {task_idx}: {err}");
                            conf_counter.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(other_err) => {
                            // Undokumentierter / unerwarteter interner Fehler
                            eprintln!(
                                "Unexpected internal error in task {task_idx}: {other_err:?}"
                            );
                            err_counter.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            });

            handles.push(handle);
        }

        // Paralleler Snapshot-Reader Task: prüft kontinuierlich, dass der lange Snapshot isoliert und stabil bleibt
        let coll_snapshot_reader = coll.clone();
        let snapshot_check_passed = Arc::new(AtomicBool::new(true));
        let snapshot_check_passed_clone = Arc::clone(&snapshot_check_passed);

        let snapshot_reader_handle = tokio::spawn(async move {
            for check_cycle in 0..50 {
                tokio::time::sleep(Duration::from_millis(5)).await;

                // Verifizieren: Die während der Schreiblast neu erzeugten Keys sind im langen Snapshot NICHT sichtbar
                let new_key = format!("doc_task_0_{check_cycle}");
                if let Ok(snap_res) = coll_snapshot_reader
                    .get_at_snapshot(&new_key, long_snapshot_seq)
                    .await
                {
                    if snap_res.is_some() {
                        eprintln!("Snapshot isolation breach: new key '{new_key}' visible at snapshot {long_snapshot_seq}");
                        snapshot_check_passed_clone.store(false, Ordering::Relaxed);
                        break;
                    }
                }

                // Verifizieren: Die Basisdokumente behalten im langen Snapshot stets ihren ursprünglichen Stand (version == 1)
                let base_key = format!("base_doc_{}", check_cycle % base_doc_count);
                if let Ok(Some(doc)) = coll_snapshot_reader
                    .get_at_snapshot(&base_key, long_snapshot_seq)
                    .await
                {
                    let version = doc
                        .metadata
                        .as_ref()
                        .and_then(|m| m.get("version"))
                        .and_then(|v| v.as_u64());
                    if version != Some(1) {
                        eprintln!("Snapshot consistency breach for '{base_key}': version is {:?}, expected 1", version);
                        snapshot_check_passed_clone.store(false, Ordering::Relaxed);
                        break;
                    }
                }
            }
        });

        // Warten auf Abschluss aller Schreibtasks und des Reader-Tasks
        for h in handles {
            h.await?;
        }
        snapshot_reader_handle.await?;

        let elapsed = start_time.elapsed();
        let n_succ = successful_commits.load(Ordering::Relaxed);
        let n_conf = expected_conflicts.load(Ordering::Relaxed);
        let n_err = unexpected_errors.load(Ordering::Relaxed);

        // Telemetrie / Degradations-Protokollierung als Testoutput (für B-16 Metrik-Gegenprüfung)
        println!("============================================================");
        println!("VERTIKALER PFAD-TEST 3 (B-16) TELEMETRIE-PROTOKOLL:");
        println!("  Gesamtzahl versuchter Schreibtransaktionen: {total_tx_target}");
        println!("  Erfolgreiche Commits:                       {n_succ}");
        println!("  Abgelehnte SSI-Konflikte (erwartet):        {n_conf}");
        println!("  Unerwartete/interne Fehler:                 {n_err}");
        println!("  Gesamtdauer:                                {elapsed:.2?}");
        if total_tx_target > 0 {
            println!(
                "  Erfolgsquote:                               {:.2}%",
                (n_succ as f64 / total_tx_target as f64) * 100.0
            );
            println!(
                "  Konfliktrate:                               {:.2}%",
                (n_conf as f64 / total_tx_target as f64) * 100.0
            );
        }
        println!("============================================================");

        // Assert (a): Keine unerwarteten/undokumentierten internen Fehler aufgetreten
        assert_eq!(
            n_err, 0,
            "No write transactions may fail with internal/undocumented errors! Got {n_err} errors."
        );

        // Assert (b): Der lange Snapshot ist bis zum Schluss konsistent und isoliert geblieben
        assert!(
            snapshot_check_passed.load(Ordering::Relaxed),
            "Long snapshot must remain strictly consistent and isolated throughout write load!"
        );

        // Abschließender Check auf den langen Snapshot
        for i in 0..base_doc_count {
            let doc_id = format!("base_doc_{i}");
            let snap_doc = coll.get_at_snapshot(&doc_id, long_snapshot_seq).await?;
            assert!(
                snap_doc.is_some(),
                "Base doc '{doc_id}' must remain present in long snapshot after write load"
            );
            assert_eq!(
                snap_doc
                    .unwrap()
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("version"))
                    .and_then(|v| v.as_u64()),
                Some(1)
            );
        }

        // Assert (c): System-Normalisierung
        // Nach Freigabe/Ende des alten Snapshots müssen neue Schreibvorgänge problemlos und ohne erhöhte Fehlerrate committet werden können.
        let post_load_writes = 50;
        let mut post_load_successes = 0;
        for i in 0..post_load_writes {
            let doc_id = format!("post_load_doc_{i}");
            let res = coll
                .insert(
                    &doc_id,
                    &dummy_vector(dim, 0.99),
                    Some(dummy_metadata("post_load", i)),
                )
                .await;

            if res.is_ok() {
                post_load_successes += 1;
            }
        }

        assert_eq!(
            post_load_successes, post_load_writes,
            "System normalization failed! Expected {post_load_writes} post-load writes to succeed, but only {post_load_successes} succeeded."
        );

        // Gracefully close database instance
        drop(coll);
        if let Ok(owned_db) = Arc::try_unwrap(db) {
            owned_db.close().await?;
        }

        Ok::<(), Box<dyn std::error::Error>>(())
    });

    // Assert (d): Totalausfall-Schutz — Test schlägt fehl if Timeout (60s) überschritten wird (Deadlock/Hang/Panic)
    match test_execution.await {
        Ok(res) => res,
        Err(_) => panic!("Vertical Path Test 3 (B-16) timed out after 120 seconds! Potential deadlock or infinite loop in commit register."),
    }
}
