// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Deterministic Simulation Testing (DST) runner, oracle verification, shrinking, and report writer.
// INVARIANTEN: Bit-identical sequence per seed, zero panic in production runner, JSON report generation.

use crate::dst::op::DstOp;
use crate::dst::p07_sut::SystemUnderTest;
use crate::dst::rng::DstRng;
use crate::reference_model::ReferenceModel;
use crate::sim_fs::{SimFs, SimFsConfig};
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::time::Instant;

/// Summary report of a DST test campaign run.
#[derive(Debug, Clone)]
pub struct P07DstReport {
    /// Number of distinct seeds tested.
    pub seeds_tested: usize,
    /// Total execution passes performed.
    pub executions: usize,
    /// Duration of campaign in milliseconds.
    pub duration_ms: u128,
    /// Throughput in seeds per second.
    pub seeds_per_sec: f64,
    /// List of divergence records found during run.
    pub divergences: Vec<P07DivergenceRecord>,
}

impl P07DstReport {
    /// Serializes report to JSON string.
    pub fn to_json(&self) -> String {
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str(&format!("  \"seeds_tested\": {},\n", self.seeds_tested));
        json.push_str(&format!("  \"executions\": {},\n", self.executions));
        json.push_str(&format!("  \"duration_ms\": {},\n", self.duration_ms));
        json.push_str(&format!(
            "  \"seeds_per_sec\": {:.2},\n",
            self.seeds_per_sec
        ));
        json.push_str("  \"divergences\": [\n");

        for (i, d) in self.divergences.iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!("      \"seed\": {},\n", d.seed));
            json.push_str(&format!("      \"op_count\": {},\n", d.op_count));
            json.push_str(&format!(
                "      \"error\": \"{}\"\n",
                d.error.replace('"', "\\\"")
            ));
            json.push_str("    }");
            if i + 1 < self.divergences.len() {
                json.push(',');
            }
            json.push('\n');
        }

        json.push_str("  ]\n");
        json.push_str("}\n");
        json
    }
}

/// Record of a single seed divergence failure.
#[derive(Debug, Clone)]
pub struct P07DivergenceRecord {
    /// Seed that failed.
    pub seed: u64,
    /// Total operations generated.
    pub op_count: usize,
    /// Divergence error message.
    pub error: String,
    /// Minimized list of operations after delta-debugging shrinking.
    pub shrunken_ops: Vec<DstOp>,
}

/// Operation sequence generator derived deterministically from a 64-bit seed.
pub fn generate_p07_ops(seed: u64, count: usize) -> Vec<DstOp> {
    let mut rng = DstRng::new(seed);
    let mut ops = Vec::with_capacity(count);

    for i in 0..count {
        let op_type = rng.next_range(1, 100);
        let k_idx = rng.next_range(1, 8);
        let key = format!("k_{}", k_idx).into_bytes();
        let val = format!("v_{}_{}", k_idx, i).into_bytes();

        if op_type <= 45 {
            ops.push(DstOp::Put { key, value: val });
        } else if op_type <= 60 {
            ops.push(DstOp::Delete { key });
        } else if op_type <= 70 {
            ops.push(DstOp::Forget { key });
        } else if op_type <= 85 {
            ops.push(DstOp::Commit);
        } else if op_type <= 92 {
            ops.push(DstOp::Sync);
        } else {
            ops.push(DstOp::Rollback);
        }
    }

    ops.push(DstOp::Commit);
    ops
}

/// DST Runner executing workloads against a SystemUnderTest and ReferenceModel oracle.
pub struct P07DstRunner;

impl P07DstRunner {
    /// Runs a single seed test execution for `SUT`.
    pub fn run_seed<F, S>(seed: u64, num_ops: usize, mut sut_factory: F) -> Result<(), String>
    where
        F: FnMut(&SimFs) -> S,
        S: SystemUnderTest,
    {
        let ops = generate_p07_ops(seed, num_ops);
        let sim_fs = SimFs::new();

        // Configure deterministic fault mode based on seed
        let mut rng = DstRng::new(seed);
        if rng.next_bool() {
            let limit = usize::try_from(rng.next_range(2, 10)).unwrap_or(2);
            sim_fs.set_config(SimFsConfig {
                fail_syncs_after: Some(limit),
                ..Default::default()
            });
        }

        let mut oracle = ReferenceModel::new();
        let mut sut = sut_factory(&sim_fs);

        for op in &ops {
            match op {
                DstOp::Put { key, value } => {
                    let _ = sut.apply(op);
                    oracle.put(key, value);
                }
                DstOp::Delete { key } => {
                    let _ = sut.apply(op);
                    oracle.delete(key);
                }
                DstOp::Forget { key } => {
                    let _ = sut.apply(op);
                    oracle.forget(key);
                }
                DstOp::Commit => {
                    let sut_res = sut.commit();
                    if sut_res.is_ok() {
                        oracle.commit();
                    } else {
                        oracle.commit_unacked();
                    }
                }
                DstOp::Rollback => {
                    let _ = sut.apply(op);
                    oracle.rollback();
                }
                DstOp::Sync => {
                    let _ = sut.apply(op);
                }
            }
        }

        // Trigger simulated crash
        sim_fs.crash(seed);

        // Reset fault counters for recovery pass
        sim_fs.reset_counters();

        // Perform recovery pass
        if let Err(e) = sut.crash_and_recover(&sim_fs) {
            return Err(format!("Crash recovery failed: {}", e));
        }

        // Validate recovered snapshot against oracle invariants
        let recovered_snapshot = sut.snapshot();
        oracle.verify_recovery(&recovered_snapshot).map(|_| ())
    }

    /// Delta-debugging shrinker: minimizes a failing op sequence.
    pub fn shrink_failing_ops<F, S>(seed: u64, mut sut_factory: F) -> Vec<DstOp>
    where
        F: FnMut(&SimFs) -> S,
        S: SystemUnderTest,
    {
        let full_ops = generate_p07_ops(seed, 60);
        let mut current_ops = full_ops.clone();

        let mut idx = 0;
        while idx < current_ops.len() {
            if current_ops.len() <= 2 {
                break;
            }
            let candidate: Vec<DstOp> = current_ops
                .iter()
                .enumerate()
                .filter(|&(i, _)| i != idx)
                .map(|(_, op)| op.clone())
                .collect();

            let sim_fs = SimFs::new();
            let mut oracle = ReferenceModel::new();
            let mut sut = sut_factory(&sim_fs);

            for op in &candidate {
                match op {
                    DstOp::Put { key, value } => {
                        let _ = sut.apply(op);
                        oracle.put(key, value);
                    }
                    DstOp::Delete { key } => {
                        let _ = sut.apply(op);
                        oracle.delete(key);
                    }
                    DstOp::Forget { key } => {
                        let _ = sut.apply(op);
                        oracle.forget(key);
                    }
                    DstOp::Commit => {
                        if sut.commit().is_ok() {
                            oracle.commit();
                        } else {
                            oracle.commit_unacked();
                        }
                    }
                    DstOp::Rollback => {
                        let _ = sut.apply(op);
                        oracle.rollback();
                    }
                    DstOp::Sync => {
                        let _ = sut.apply(op);
                    }
                }
            }

            sim_fs.crash(seed);
            sim_fs.reset_counters();
            let failed = sut.crash_and_recover(&sim_fs).is_err()
                || oracle.verify_recovery(&sut.snapshot()).is_err();

            if failed {
                current_ops = candidate;
            } else {
                idx += 1;
            }
        }

        current_ops
    }

    /// Executes a DST campaign using env configuration (`CONTEXTRA_DST_SEEDS`, `CONTEXTRA_DST_SEED`, `CONTEXTRA_DST_REPORT`).
    pub fn run_campaign<F, S>(sut_factory: F) -> io::Result<P07DstReport>
    where
        F: Fn(&SimFs) -> S + Copy,
        S: SystemUnderTest,
    {
        let start_time = Instant::now();

        let num_seeds: usize = env::var("CONTEXTRA_DST_SEEDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(200);

        let single_seed: Option<u64> = env::var("CONTEXTRA_DST_SEED")
            .ok()
            .and_then(|s| s.parse().ok());

        let report_path = env::var("CONTEXTRA_DST_REPORT")
            .unwrap_or_else(|_| "target/dst/report.json".to_string());

        let seeds_to_run: Vec<u64> = match single_seed {
            Some(s) => vec![s],
            None => (1000..1000 + num_seeds as u64).collect(),
        };

        let mut divergences = Vec::new();
        let total_seeds = seeds_to_run.len();

        for &s in &seeds_to_run {
            if let Err(err_msg) = Self::run_seed(s, 50, sut_factory) {
                let shrunken = Self::shrink_failing_ops(s, sut_factory);
                divergences.push(P07DivergenceRecord {
                    seed: s,
                    op_count: 50,
                    error: err_msg,
                    shrunken_ops: shrunken,
                });
            }
        }

        let elapsed = start_time.elapsed();
        let duration_ms = elapsed.as_millis();
        let seeds_per_sec = if duration_ms > 0 {
            (total_seeds as f64 * 1000.0) / duration_ms as f64
        } else {
            total_seeds as f64 * 1000.0
        };

        let report = P07DstReport {
            seeds_tested: total_seeds,
            executions: total_seeds,
            duration_ms,
            seeds_per_sec,
            divergences,
        };

        // Write report file
        let path = Path::new(&report_path);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent); // INTENTIONAL-DROP
        }
        let _ = fs::write(path, report.to_json()); // INTENTIONAL-DROP

        Ok(report)
    }
}
