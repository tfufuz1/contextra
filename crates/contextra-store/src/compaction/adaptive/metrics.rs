//! # Operational Workload Metrics (Ring 1)
//!
//! Deterministische Metriken zur Erfassung der Lese- und Schreiblast (ohne Wanduhrzeit).

use std::sync::atomic::{AtomicU64, Ordering};

/// Operational workload metrics captured deterministically without wall-clock time (P28 compliance).
///
/// All trigger decisions are strictly derived from transaction/sequence numbers and operational counts.
#[derive(Debug)]
pub struct WorkloadMetrics {
    read_count: AtomicU64,
    write_count: AtomicU64,
    last_seq_no: AtomicU64,
}

impl Default for WorkloadMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkloadMetrics {
    /// Creates a new WorkloadMetrics counter instance initialized to zero.
    pub fn new() -> Self {
        Self {
            read_count: AtomicU64::new(0),
            write_count: AtomicU64::new(0),
            last_seq_no: AtomicU64::new(0),
        }
    }

    /// Records a read operation atomically.
    #[inline]
    pub fn record_read(&self) {
        self.read_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a write operation and updates the latest sequence number atomically.
    #[inline]
    pub fn record_write(&self, seq_no: u64) {
        self.write_count.fetch_add(1, Ordering::Relaxed);
        self.last_seq_no.fetch_max(seq_no, Ordering::Relaxed);
    }

    /// Takes a deterministic snapshot of current metrics for compaction strategy evaluation.
    pub fn snapshot(&self) -> WorkloadMetricsSnapshot {
        WorkloadMetricsSnapshot {
            read_count: self.read_count.load(Ordering::Relaxed),
            write_count: self.write_count.load(Ordering::Relaxed),
            last_seq_no: self.last_seq_no.load(Ordering::Relaxed),
        }
    }
}

/// Point-in-time deterministic snapshot of operational workload counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkloadMetricsSnapshot {
    /// Total read operations recorded.
    pub read_count: u64,
    /// Total write operations recorded.
    pub write_count: u64,
    /// Highest observed transaction sequence number.
    pub last_seq_no: u64,
}

impl WorkloadMetricsSnapshot {
    /// Computes the read/write operational ratio safely handling zero writes.
    pub fn read_write_ratio(&self) -> f64 {
        if self.write_count == 0 {
            if self.read_count > 0 {
                f64::INFINITY
            } else {
                1.0
            }
        } else {
            self.read_count as f64 / self.write_count as f64
        }
    }
}
