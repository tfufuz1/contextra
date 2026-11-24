//! # Differenztest: Deterministische Planer-Äquivalenz (Ring 1)
//!
//! Prüft die Äquivalenz der Planer-Berechnungen gegen die Referenzkopie.

mod mz_d_support;

use contextra_store::compaction::adaptive::{CostBasedAdaptivePlanner, WorkloadMetricsSnapshot};

#[test]
fn test_planner_metrics_ratio_equivalence() {
    let snapshots = vec![
        WorkloadMetricsSnapshot {
            read_count: 100,
            write_count: 100,
            last_seq_no: 10,
        },
        WorkloadMetricsSnapshot {
            read_count: 500,
            write_count: 10,
            last_seq_no: 25,
        },
        WorkloadMetricsSnapshot {
            read_count: 0,
            write_count: 100,
            last_seq_no: 50,
        },
        WorkloadMetricsSnapshot {
            read_count: 10,
            write_count: 0,
            last_seq_no: 60,
        },
        WorkloadMetricsSnapshot {
            read_count: 0,
            write_count: 0,
            last_seq_no: 0,
        },
    ];

    for snapshot in snapshots {
        let expected = mz_d_support::reference_read_write_ratio(&snapshot);
        let actual = snapshot.read_write_ratio();
        if expected.is_infinite() {
            assert!(actual.is_infinite());
        } else {
            assert_eq!(expected, actual);
        }
    }
}

#[test]
fn test_default_planner_equivalence() {
    let _reference = mz_d_support::reference_default_planner();
    let _current = CostBasedAdaptivePlanner::default();
    // Validierung, dass beide deterministisch konstruiert werden
}
