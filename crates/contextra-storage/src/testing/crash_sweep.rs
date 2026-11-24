//! FaultFs crash-sweep driver across every I/O boundary (SP-ENV, SP-VERIFY §13.1).

use crate::env::faultfs::{Fault, FaultFs, FaultPlan};
use crate::env::memfs::MemFs;
use contextra_storage_ports::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrashSweepReport {
    pub total_ops: u64,
    pub crash_points_checked: u64,
}

/// Runs `workload` once cleanly to count total I/O ops `N`, then replays
/// `workload` with `FaultFs` injecting `fault` at every `at_op in 0..N` and
/// invokes `verify_crash_image` on each resulting `MemFs::crash_image()`.
pub fn sweep_write_ops<W, V>(
    fault: Fault,
    mut workload: W,
    mut verify_crash_image: V,
) -> Result<CrashSweepReport, Error>
where
    W: FnMut(&FaultFs<MemFs>) -> Result<(), Error>,
    V: FnMut(u64, MemFs) -> Result<(), Error>,
{
    let baseline = FaultFs::new(
        MemFs::new(),
        FaultPlan {
            at_op: u64::MAX,
            fault,
        },
    );
    workload(&baseline)?;
    let total_ops = baseline.op_count();

    for at_op in 0..total_ops {
        let fs = FaultFs::new(MemFs::new(), FaultPlan { at_op, fault });
        let _ = workload(&fs);
        let crashed = fs.crash_image()?;
        verify_crash_image(at_op, crashed)?;
    }

    Ok(CrashSweepReport {
        total_ops,
        crash_points_checked: total_ops,
    })
}
