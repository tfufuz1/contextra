// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Pre-configured DST scenarios for testing system invariants.
// INVARIANTEN: All scenarios must be reproducible given a seed.

use crate::dst::runner::{DstConfig, DstResult, DstRunner, FaultMode};

/// Pre-configured scenario runner for DST simulations.
pub struct DstScenario;

impl DstScenario {
    /// Runs the Smoke scenario.
    pub fn smoke(seed: u64) -> DstResult {
        DstRunner::run(&DstConfig {
            seed,
            num_ops: 60,
            fault_mode: FaultMode::None,
        })
    }

    /// Runs the Torn Write scenario (write fault injected midway).
    pub fn torn_write(seed: u64, fail_writes_after: usize) -> DstResult {
        DstRunner::run(&DstConfig {
            seed,
            num_ops: 80,
            fault_mode: FaultMode::TornWrite { fail_writes_after },
        })
    }

    /// Runs the fsync error scenario (sync fault injected midway).
    pub fn fsync_error(seed: u64, fail_syncs_after: usize) -> DstResult {
        DstRunner::run(&DstConfig {
            seed,
            num_ops: 80,
            fault_mode: FaultMode::FsyncError { fail_syncs_after },
        })
    }

    /// Runs the ENOSPC scenario (storage full fault injected midway).
    pub fn enospc(seed: u64, fail_writes_after: usize) -> DstResult {
        DstRunner::run(&DstConfig {
            seed,
            num_ops: 80,
            fault_mode: FaultMode::Enospc { fail_writes_after },
        })
    }
}
