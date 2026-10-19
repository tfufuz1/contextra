// FILE-CONTEXT
// STAND: 2026-10-06
// ZWECK: Ghost-Pointer Verification Types für HNSW Graph.

/// Reason why a ghost pointer scan could not be completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncompleteReason {
    /// The scan exceeded its allocated iteration/node cost budget.
    BudgetExhausted,
    /// An error occurred while reading index structures (e.g., mmap connection lookup).
    ReadError,
}

/// Result of a ghost-pointer verification scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhostScan {
    /// Verification completed fully with 0 residual ghost pointers (or `n` = 0).
    Complete(u64),
    /// Verification found `n` > 0 ghost pointers.
    Violated(u64),
    /// Verification was incomplete due to budget limits or read errors.
    Incomplete(IncompleteReason),
}

impl GhostScan {
    /// Returns true if and only if the scan completed fully and verified 0 ghost pointers remain.
    #[inline]
    pub fn is_verified_no_ghost_pointers(&self) -> bool {
        matches!(self, GhostScan::Complete(0))
    }
}
