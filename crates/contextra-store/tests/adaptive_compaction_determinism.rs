use contextra_ports::StorageStats;
use contextra_store::{
    AdaptiveCompactionPlanner, CostBasedAdaptivePlanner, WorkloadMetrics,
};

#[test]
fn test_adaptive_compaction_determinism() {
    let metrics1 = WorkloadMetrics::new();
    let metrics2 = WorkloadMetrics::new();

    // Replay identical workload sequence (100 reads, 10 writes, seq_no 1 to 10)
    for seq in 1..=10 {
        metrics1.record_write(seq);
        metrics2.record_write(seq);
    }
    for _ in 0..100 {
        metrics1.record_read();
        metrics2.record_read();
    }

    let snap1 = metrics1.snapshot();
    let snap2 = metrics2.snapshot();

    assert_eq!(snap1, snap2);

    let planner = CostBasedAdaptivePlanner::new(0.70, 2, 4.0);
    let stats = StorageStats {
        num_segments: 3,
        total_size_bytes: 4096,
        memtable_size_bytes: 512,
    };

    let plan1 = planner.plan_compaction(&stats, &snap1, &[], 0).unwrap();
    let plan2 = planner.plan_compaction(&stats, &snap2, &[], 0).unwrap();

    assert_eq!(plan1, plan2);

    // Deterministic strategy calculation check directly on metrics
    let total_ops1 = snap1.read_count + snap1.write_count;
    let read_ratio1 = snap1.read_count as f64 / total_ops1 as f64;

    let total_ops2 = snap2.read_count + snap2.write_count;
    let read_ratio2 = snap2.read_count as f64 / total_ops2 as f64;

    assert_eq!(read_ratio1, read_ratio2);
    assert!(read_ratio1 >= 0.70);
}
