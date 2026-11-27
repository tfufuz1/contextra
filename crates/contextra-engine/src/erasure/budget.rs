//! Engine-side I/O budget adapter (Spezifikation §8, "Budget").
//!
//! Wraps two [`StoreIoBudget`]s: a normal one and one with a doubled erase share that is
//! selected once a job has used 80 % of its SLA. Pressure coupling (`Critical` pauses,
//! `Elevated` halves) is inherited from the store budget; G1 never goes through this type.

use contextra_ports::erasure::{IoBudget, IoClass, PressureHint};
use contextra_ports::{BoxFuture, Clock};
use contextra_store::io_budget::StoreIoBudget;
use contextra_types::ErasureError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// `IoBudget` used by the erasure worker.
pub struct EngineIoBudget {
    normal: StoreIoBudget,
    boosted: StoreIoBudget,
    boost: AtomicBool,
}

impl EngineIoBudget {
    /// Creates the budget. `max_io_bytes_per_second == 0` means unlimited rate (pressure only).
    pub fn new(
        max_io_bytes_per_second: u64,
        erase_io_share: f32,
        clock: Arc<dyn Clock>,
        pressure_rx: tokio::sync::watch::Receiver<contextra_store::SystemPressure>,
    ) -> Self {
        Self {
            normal: StoreIoBudget::new(
                max_io_bytes_per_second,
                erase_io_share,
                Arc::clone(&clock),
                pressure_rx.clone(),
            ),
            boosted: StoreIoBudget::new(
                max_io_bytes_per_second,
                erase_io_share * 2.0,
                clock,
                pressure_rx,
            ),
            boost: AtomicBool::new(false),
        }
    }

    /// Switches to the doubled erase share (SLA at 80 %). `Critical` pressure is still honoured.
    pub fn set_sla_boost(&self, on: bool) {
        self.boost.store(on, Ordering::Release);
    }

    /// Returns whether the SLA boost is active.
    pub fn sla_boost(&self) -> bool {
        self.boost.load(Ordering::Acquire)
    }

    fn active(&self) -> &StoreIoBudget {
        if self.sla_boost() {
            &self.boosted
        } else {
            &self.normal
        }
    }
}

impl IoBudget for EngineIoBudget {
    fn acquire<'a>(
        &'a self,
        class: IoClass,
        bytes: u64,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        self.active().acquire(class, bytes)
    }

    fn pressure(&self) -> PressureHint {
        self.active().pressure()
    }
}
