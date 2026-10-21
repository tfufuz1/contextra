// FILE-CONTEXT
// STAND: 2026-10-22
// ZWECK: Integration tests for CrashMatrix generic failpoint runner.
// INVARIANTEN: Deterministic execution, dead failpoint detection, zero panic.

use contextra_testkit::crash_matrix::{CrashMatrix, CrashMatrixError};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn crash_matrix_reports_dead_failpoint() {
    let failpoints = &[
        "store_wal_write_fail",
        "store_manifest_append_fail",
        "never_triggered_fp",
    ];

    let matrix = CrashMatrix::new(failpoints);

    // Scenario runner hits the first two failpoints, but never hits "never_triggered_fp"
    let report_res = matrix
        .run(|fp| async move {
            if fp == "store_wal_write_fail" || fp == "store_manifest_append_fail" {
                // Simulate hitting failpoint
                CrashMatrix::record_hit(fp);
            }
            Ok::<(), String>(())
        })
        .await;

    assert!(
        report_res.is_err(),
        "CrashMatrix should fail when a dead failpoint is present"
    );

    match report_res {
        Err(CrashMatrixError::DeadFailpoints(dead_points)) => {
            assert_eq!(dead_points, vec!["never_triggered_fp"]);
        }
        other => panic!("Expected DeadFailpoints error, got: {:?}", other),
    }
}

#[tokio::test]
async fn crash_matrix_is_deterministic() {
    let failpoints = &["fp_a", "fp_b"];

    let run_count_a = Arc::new(AtomicU32::new(0));
    let run_count_b = Arc::new(AtomicU32::new(0));

    let matrix1 = CrashMatrix::new(failpoints).with_seed(12345);
    let counter1 = run_count_a.clone();
    let rep1 = matrix1
        .run(|fp| {
            let c = counter1.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                CrashMatrix::record_hit(fp);
                Ok::<(), String>(())
            }
        })
        .await
        .expect("matrix1 run failed");

    let matrix2 = CrashMatrix::new(failpoints).with_seed(12345);
    let counter2 = run_count_b.clone();
    let rep2 = matrix2
        .run(|fp| {
            let c = counter2.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                CrashMatrix::record_hit(fp);
                Ok::<(), String>(())
            }
        })
        .await
        .expect("matrix2 run failed");

    assert_eq!(
        rep1, rep2,
        "CrashMatrix reports should be identical for same seed"
    );
    assert_eq!(
        run_count_a.load(Ordering::SeqCst),
        run_count_b.load(Ordering::SeqCst)
    );
}
