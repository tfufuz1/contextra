// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Deterministic Simulation Testing (DST) runner and fault injector.
// INVARIANTEN: Bit-identical operation generation for equal seeds, reproducible fault scenarios.

use crate::dst::engine::DstEngine;
use crate::dst::invariants::{DstModel, InvariantChecker};
use crate::dst::op::DstOp;
use crate::dst::rng::DstRng;
use crate::fault_vfs::{FaultConfig, FaultVfs};
use std::path::PathBuf;

/// Fault mode for DST simulation runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FaultMode {
    /// No faults injected.
    None,
    /// Simulates a torn write after N successful writes.
    TornWrite { fail_writes_after: usize },
    /// Simulates an fsync error after N successful syncs.
    FsyncError { fail_syncs_after: usize },
    /// Simulates out-of-disk-space (ENOSPC) after N successful writes.
    Enospc { fail_writes_after: usize },
}

/// Configuration for a DST runner execution.
#[derive(Debug, Clone)]
pub struct DstConfig {
    /// Seed determining operation stream and crash points.
    pub seed: u64,
    /// Number of operations to generate.
    pub num_ops: usize,
    /// Active fault injection mode.
    pub fault_mode: FaultMode,
}

impl Default for DstConfig {
    fn default() -> Self {
        Self {
            seed: 12345,
            num_ops: 50,
            fault_mode: FaultMode::None,
        }
    }
}

/// Result of a DST simulation run.
#[derive(Debug, Clone)]
pub struct DstResult {
    /// The seed used for the run.
    pub seed: u64,
    /// Generated sequence of operations.
    pub ops: Vec<DstOp>,
    /// Number of successfully acknowledged commits.
    pub acked_commits: usize,
    /// Sequence number recovered after crash.
    pub recovered_seq: u64,
    /// Whether all post-recovery invariants passed.
    pub success: bool,
    /// Error message if invariants were violated or run failed.
    pub error_message: Option<String>,
}

/// Deterministic Simulation Testing runner.
pub struct DstRunner;

impl DstRunner {
    /// Generates a bit-identical operation sequence from `seed` and `num_ops`.
    pub fn generate_ops(seed: u64, num_ops: usize) -> Vec<DstOp> {
        let mut rng = DstRng::new(seed);
        let mut ops = Vec::with_capacity(num_ops);
        let mut staged_count = 0;

        for i in 0..num_ops {
            let op_type = rng.next_range(1, 100);
            let key_idx = rng.next_range(1, 10);
            let key = format!("key_{}", key_idx).into_bytes();
            let val = format!("val_{}_{}", key_idx, i).into_bytes();

            if op_type <= 50 {
                ops.push(DstOp::Put { key, value: val });
                staged_count += 1;
            } else if op_type <= 65 {
                ops.push(DstOp::Delete { key });
                staged_count += 1;
            } else if op_type <= 75 {
                ops.push(DstOp::Forget { key });
            } else if op_type <= 90 || staged_count > 5 {
                ops.push(DstOp::Commit);
                staged_count = 0;
            } else if op_type <= 95 {
                ops.push(DstOp::Sync);
            } else {
                ops.push(DstOp::Rollback);
                staged_count = 0;
            }
        }

        // Always end with a Commit if ops were staged
        if staged_count > 0 {
            ops.push(DstOp::Commit);
        }

        ops
    }

    /// Runs a DST scenario using the given configuration.
    pub fn run(config: &DstConfig) -> DstResult {
        let ops = Self::generate_ops(config.seed, config.num_ops);

        let vfs = FaultVfs::new();
        match config.fault_mode {
            FaultMode::None => {}
            FaultMode::TornWrite { fail_writes_after }
            | FaultMode::Enospc { fail_writes_after } => {
                vfs.set_config(FaultConfig {
                    fail_writes_after: Some(fail_writes_after),
                    ..Default::default()
                });
            }
            FaultMode::FsyncError { fail_syncs_after } => {
                vfs.set_config(FaultConfig {
                    fail_syncs_after: Some(fail_syncs_after),
                    ..Default::default()
                });
            }
        }

        let db_path = PathBuf::from("dst_store.wal");
        let mut engine = DstEngine::new(vfs.clone(), db_path.clone());
        let mut model = DstModel::new();

        let mut acked_commits = 0;

        for op in &ops {
            match op {
                DstOp::Put { key, value } => {
                    engine.put(key.clone(), value.clone());
                    model.stage_put(key.clone(), value.clone());
                }
                DstOp::Delete { key } => {
                    engine.delete(key.clone());
                    model.stage_delete(key.clone());
                }
                DstOp::Forget { key } => {
                    if let Ok(()) = engine.forget(key.clone()) {
                        model.ack_forget(key.clone());
                    } else {
                        // Fault injected during forget -> stop simulation
                        break;
                    }
                }
                DstOp::Commit => {
                    match engine.commit() {
                        Ok(seq) => {
                            model.ack_commit(seq);
                            acked_commits += 1;
                        }
                        Err(_) => {
                            // Fault injected during commit -> stop simulation and rollback un-acked ops
                            engine.rollback();
                            model.rollback();
                            break;
                        }
                    }
                }
                DstOp::Rollback => {
                    engine.rollback();
                    model.rollback();
                }
                DstOp::Sync => {
                    if engine.sync().is_err() {
                        break;
                    }
                }
            }
        }

        // Post-crash recovery pass: reset fault counters while keeping file data
        vfs.reset_counters();
        let recovered_engine = match DstEngine::recover(&vfs, &db_path) {
            Ok(eng) => eng,
            Err(e) => {
                return DstResult {
                    seed: config.seed,
                    ops,
                    acked_commits,
                    recovered_seq: 0,
                    success: false,
                    error_message: Some(format!("Recovery failed with I/O error: {}", e)),
                };
            }
        };

        // Enforce invariants
        match InvariantChecker::verify(&recovered_engine, &model) {
            Ok(()) => DstResult {
                seed: config.seed,
                ops,
                acked_commits,
                recovered_seq: recovered_engine.last_seq(),
                success: true,
                error_message: None,
            },
            Err(msg) => DstResult {
                seed: config.seed,
                ops,
                acked_commits,
                recovered_seq: recovered_engine.last_seq(),
                success: false,
                error_message: Some(format!("DST Seed {}: {}", config.seed, msg)),
            },
        }
    }
}
