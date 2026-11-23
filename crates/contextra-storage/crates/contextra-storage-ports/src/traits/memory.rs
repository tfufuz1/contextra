//! Shared admission contract for storage and registered extensions.

use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceClass {
    Memtable,
    ClusterMeta,
    Guard,
    BlockCache,
    KeyCache,
    ExtensionDelta,
    Inflight,
    PurgeScratch,
}

/// A single successful reservation. Dropping it returns exactly the reserved
/// amount to the originating budget; permits are intentionally not cloneable.
pub struct ResourcePermit {
    bytes: u64,
    class: ResourceClass,
    release: Option<Box<dyn FnOnce() + Send + Sync + 'static>>,
}

impl ResourcePermit {
    pub fn new(
        class: ResourceClass,
        bytes: u64,
        release: impl FnOnce() + Send + Sync + 'static,
    ) -> Self {
        Self { bytes, class, release: Some(Box::new(release)) }
    }

    pub fn bytes(&self) -> u64 { self.bytes }
    pub fn class(&self) -> ResourceClass { self.class }
}

impl Drop for ResourcePermit {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() { release(); }
    }
}

pub trait MemoryBudget: Send + Sync {
    /// Reserves before the caller allocates. Implementations must reject
    /// overflow and class/total-limit exhaustion without changing accounting.
    fn try_reserve(&self, class: ResourceClass, bytes: u64) -> Result<ResourcePermit, Error>;
}
