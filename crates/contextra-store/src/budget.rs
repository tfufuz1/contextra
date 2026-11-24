//! ResourceTracker and ResourceBudget for contextra-store memory budgeting.

use std::sync::atomic::{AtomicU64, Ordering};
use crate::error::{StoreError, StoreResult};

/// Resource budget configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct ResourceBudget {
    /// Memory limit in bytes (0 = unlimited).
    pub memory_limit: usize,
}

/// Thread-safe tracker for memory allocations.
#[derive(Debug)]
pub struct ResourceTracker {
    budget: ResourceBudget,
    memory_used: AtomicU64,
}

impl Default for ResourceTracker {
    fn default() -> Self {
        Self::new(ResourceBudget::default())
    }
}

impl ResourceTracker {
    /// Creates a new ResourceTracker with the specified budget.
    pub fn new(budget: ResourceBudget) -> Self {
        Self {
            budget,
            memory_used: AtomicU64::new(0),
        }
    }

    /// Consumes memory, failing if limit is exceeded.
    pub fn consume_memory(&self, bytes: u64) -> StoreResult<()> {
        if self.budget.memory_limit == 0 {
            self.memory_used.fetch_add(bytes, Ordering::Relaxed);
            return Ok(());
        }
        let limit = self.budget.memory_limit as u64;
        let mut current = self.memory_used.load(Ordering::Relaxed);
        loop {
            if current.saturating_add(bytes) > limit {
                return Err(StoreError::MemoryBudgetExceeded {
                    used_mb: (current + bytes) / (1024 * 1024),
                    limit_mb: limit / (1024 * 1024),
                });
            }
            match self.memory_used.compare_exchange_weak(
                current,
                current + bytes,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(()),
                Err(actual) => current = actual,
            }
        }
    }

    /// Releases previously consumed memory.
    pub fn release_memory(&self, bytes: u64) {
        let mut current = self.memory_used.load(Ordering::Relaxed);
        loop {
            let next = current.saturating_sub(bytes);
            match self.memory_used.compare_exchange_weak(
                current,
                next,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(actual) => current = actual,
            }
        }
    }

    /// Returns the current memory used in bytes.
    pub fn memory_used(&self) -> u64 {
        self.memory_used.load(Ordering::Relaxed)
    }

    /// Returns true if memory limit is not reached.
    pub fn has_memory_capacity(&self) -> bool {
        if self.budget.memory_limit == 0 {
            return true;
        }
        self.memory_used.load(Ordering::Relaxed) < self.budget.memory_limit as u64
    }
}
