//! Nonblocking admission decisions for disk pressure and maintenance I/O.
//!
//! The governor contains no filesystem calls: adapters sample free space and
//! pass the value to `observe_free_space` outside storage locks. Urgent erase
//! work remains admissible while ordinary writes are stopped.

use std::sync::atomic::{AtomicBool, Ordering};

/// Work classes use the SPEC priority order. The urgent class is reserved for
/// erase gate, key destruction, and purge progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkClass {
    UserCommit,
    Flush,
    Compaction,
    BlobGc,
    UrgentErase,
    Read,
}

/// Disk-full hysteresis and maintenance I/O pacing configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GovernorConfig {
    /// Enter disk-full mode below this protected reserve.
    pub reserve_bytes: u64,
    /// Additional free space required to leave disk-full mode.
    pub recovery_headroom_bytes: u64,
    /// Maximum fraction of observed device bandwidth assigned to maintenance,
    /// in basis points (0..=10_000).
    pub maintenance_io_basis_points: u16,
}

impl GovernorConfig {
    pub fn validate(self) -> bool {
        self.recovery_headroom_bytes > 0 && self.maintenance_io_basis_points <= 10_000
    }
}

/// Lock-free disk pressure state. User commits and ordinary maintenance are
/// rejected while full; reads and urgent erasure remain available.
pub struct Governor {
    config: GovernorConfig,
    disk_full: AtomicBool,
}

impl Governor {
    pub fn new(config: GovernorConfig) -> Option<Self> {
        config.validate().then_some(Self {
            config,
            disk_full: AtomicBool::new(false),
        })
    }

    /// Update the pressure state from an adapter's free-space observation.
    /// Entry is immediate; exit requires reserve plus configured headroom.
    pub fn observe_free_space(&self, free_bytes: u64) -> bool {
        let recovery_at = self.config.reserve_bytes.saturating_add(self.config.recovery_headroom_bytes);
        let mut current = self.disk_full.load(Ordering::Acquire);
        loop {
            let next = if current {
                free_bytes < recovery_at
            } else {
                free_bytes < self.config.reserve_bytes
            };
            match self.disk_full.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return next,
                Err(observed) => current = observed,
            }
        }
    }

    pub fn is_disk_full(&self) -> bool {
        self.disk_full.load(Ordering::Acquire)
    }

    /// Whether work can start now. This is an admission decision only; callers
    /// perform any backpressure wait before acquiring a core lock.
    pub fn admits(&self, work: WorkClass) -> bool {
        if !self.is_disk_full() {
            return true;
        }
        matches!(work, WorkClass::Read | WorkClass::UrgentErase)
    }

    /// Configured cap for the maintenance scheduler, expressed in basis
    /// points of the adapter-measured available I/O rate.
    pub const fn maintenance_io_basis_points(&self) -> u16 {
        self.config.maintenance_io_basis_points
    }
}
