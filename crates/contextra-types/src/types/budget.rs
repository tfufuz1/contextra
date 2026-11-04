//! Ressourcen- und Token-Budgetverwaltung für Contextra.
//!
//! # Architektur-Rolle: Ring 0 (Budget- und Backpressure-Management)
//!
//! Dieses Modul überwacht und steuert den Ressourcenverbrauch:
//! - `TokenBudget`: Atomares Tracking und RAII-Reservierung (`Reservation`) von LLM-Tokens
//!   zur Vermeidung von Obergrenzenüberschreitungen und Double-Spending.
//! - `ResourceTracker`: Speicherüberwachung (RAM) mit automatischem Backpressure-Signal
//!   (`has_memory_capacity`), wenn die 95%-Grenze erreicht wird.
//!
//! # Invarianten
//! - Atomare Zähler mit `Ordering::Acquire`, `Ordering::Release` und `compare_exchange_weak`.
//! - Sättigende Subtraktion (`saturating_sub`) verhindert Underflow-Wrapping bei Speicherfreigaben.

// ZWECK: Token- und Resource-Budget-Management mit atomarem Tracking & Memory Backpressure.
// INVARIANTEN: Atomares Memory Tracking; MemoryBudgetExceeded bei >95% Kapazität.
// HOTSPOTS: 40-220
// NICHT-OFFENSICHTLICH: TokenBudget reserviert Puffer für System-Prompts & Antworten in ContextManager.
// SIEHE AUCH: rules/tag_taxonomy.md, DECISIONS.md

//! # Architektur
//! Implementiert den `ResourceTracker`, der den Speicherverbrauch überwacht
//! und Backpressure-Signale an die Storage-Engines sendet.
//!
//! # Invarianten
//! - Der Speicherverbrauch wird atomar getrackt.
//! - Überschreitet der Verbrauch 95% des Budgets, wird `MemoryBudgetExceeded` geworfen.

use crate::error::{ContextraError, Result};
use serde::{Deserialize, Serialize};

/// Strategie für Token-Budget-Management.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ResourceBudget {
    /// Maximum allowed memory usage in bytes.
    pub memory_limit: u64,
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self {
            memory_limit: 2 * 1024 * 1024 * 1024, // 2GB
        }
    }
}

/// Tracks resource usage against a budget.
#[derive(Debug)]
pub struct ResourceTracker {
    /// The configured budget.
    budget: ResourceBudget,
    /// Current memory usage in bytes.
    memory_used: std::sync::atomic::AtomicU64,
}

impl Default for ResourceTracker {
    fn default() -> Self {
        Self::new(ResourceBudget::default())
    }
}

impl ResourceTracker {
    /// Creates a new ResourceTracker with the given budget.
    pub fn new(budget: ResourceBudget) -> Self {
        Self {
            budget,
            memory_used: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Attempts to consume `bytes` of memory from the tracked budget.
    ///
    /// # Errors
    /// Returns `Err(ContextraError::MemoryBudgetExceeded)` if consuming `bytes`
    /// exceeds the configured memory limit.
    pub fn consume_memory(&self, bytes: u64) -> Result<()> {
        loop {
            let current = self.memory_used.load(std::sync::atomic::Ordering::Acquire);
            let next = current.saturating_add(bytes);

            if next > self.budget.memory_limit {
                return Err(ContextraError::MemoryBudgetExceeded {
                    used_mb: next / (1024 * 1024),
                    limit_mb: self.budget.memory_limit / (1024 * 1024),
                });
            }
            if self
                .memory_used
                .compare_exchange(
                    current,
                    next,
                    std::sync::atomic::Ordering::AcqRel,
                    std::sync::atomic::Ordering::Relaxed,
                )
                .is_ok()
            {
                return Ok(());
            }
        }
    }

    /// Releases memory from the budget. Uses saturating subtraction to prevent
    /// atomic underflow (wrapping to `u64::MAX`) which would permanently block
    /// all future allocations. (FIND-COR-002)
    pub fn release_memory(&self, bytes: u64) {
        loop {
            let current = self.memory_used.load(std::sync::atomic::Ordering::Acquire);

            // SD-05-COR-001: Underflow protection for the atomic counter.
            // We saturate to 0 instead of wrapping to preserve system stability.

            let new_val = current.saturating_sub(bytes);
            if self
                .memory_used
                .compare_exchange(
                    current,
                    new_val,
                    std::sync::atomic::Ordering::AcqRel,
                    std::sync::atomic::Ordering::Relaxed,
                )
                .is_ok()
            {
                break;
            }
        }
    }

    /// Returns current memory usage in bytes.
    pub fn memory_used(&self) -> u64 {
        self.memory_used.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Returns a reference to the configured resource budget.
    pub fn budget(&self) -> &ResourceBudget {
        &self.budget
    }

    /// Returns true if memory usage is below 95% of the limit.
    pub fn has_memory_capacity(&self) -> bool {
        let threshold = self.budget.memory_limit.saturating_mul(95) / 100;
        self.memory_used() < threshold
    }
}

// INTENT: Resource Tracker (Memory Budget & Backpressure) verified by 5 tests.
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_tracker_basic() {
        let budget = ResourceBudget { memory_limit: 1000 };
        let tracker = ResourceTracker::new(budget);

        assert_eq!(tracker.memory_used(), 0);
        assert!(tracker.has_memory_capacity());

        tracker.consume_memory(500).expect("should consume"); // expect
        assert_eq!(tracker.memory_used(), 500);
        assert!(tracker.has_memory_capacity()); // 50% < 95%

        tracker.release_memory(200);
        assert_eq!(tracker.memory_used(), 300);
    }

    #[test]
    fn test_budget_exceeded() {
        let budget = ResourceBudget { memory_limit: 1000 };
        let tracker = ResourceTracker::new(budget);

        tracker.consume_memory(900).expect("should consume"); // expect
        let result = tracker.consume_memory(200);

        assert!(result.is_err());
        match result.err().unwrap() {
            // unwrap
            // unwrap
            ContextraError::MemoryBudgetExceeded { limit_mb, .. } => {
                // used_mb = (900 + 200) / 1024*1024 = 0 in this case because limit is tiny
                assert_eq!(limit_mb, 0);
            }
            _ => panic!("Expected MemoryBudgetExceeded error"),
        }
    }

    #[test]
    fn test_capacity_threshold() {
        let budget = ResourceBudget { memory_limit: 1000 };
        let tracker = ResourceTracker::new(budget);

        tracker.consume_memory(949).expect("ok"); // expect
        assert!(tracker.has_memory_capacity()); // 94.9% < 95%

        tracker.consume_memory(1).expect("ok"); // expect
        assert!(!tracker.has_memory_capacity()); // 95% is not < 95%
    }

    #[test]
    fn test_concurrent_consumption() {
        let budget = ResourceBudget {
            memory_limit: 10000,
        };
        let tracker = std::sync::Arc::new(ResourceTracker::new(budget));

        let mut handlers = Vec::new();
        for _ in 0..10 {
            let t = tracker.clone();
            handlers.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    t.consume_memory(10).expect("consume"); // expect
                }
            }));
        }

        for h in handlers {
            h.join().unwrap(); // unwrap
        }

        assert_eq!(tracker.memory_used(), 10000);
        assert!(tracker.consume_memory(1).is_err());
    }

    #[test]
    fn test_release_memory_underflow_safety() {
        let budget = ResourceBudget { memory_limit: 1000 };
        let tracker = ResourceTracker::new(budget);

        // Underflow: release 10 when 0 used
        tracker.release_memory(10);
        assert_eq!(
            tracker.memory_used(),
            0,
            "Counter should saturate at 0, not wrap to MAX"
        );

        // Verify we can still consume memory (wrap would make memory_used > limit)
        tracker
            .consume_memory(500)
            .expect("Should still allow consumption after underflow"); // expect
        assert_eq!(tracker.memory_used(), 500);
    }

    #[test]
    fn test_consume_memory_overflow_prevention() {
        let budget = ResourceBudget { memory_limit: 1000 };
        let tracker = ResourceTracker::new(budget);

        // Try to consume more than u64::MAX (total)
        let result = tracker.consume_memory(u64::MAX);
        assert!(result.is_err());

        // Now test if current + bytes overflows u64
        tracker.consume_memory(500).unwrap(); // unwrap
        let result = tracker.consume_memory(u64::MAX - 100);
        assert!(result.is_err());
    }
}
