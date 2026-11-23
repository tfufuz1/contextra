//! Atomic shared admission budget used before bounded allocations.

use contextra_storage_ports::traits::{MemoryBudget, ResourceClass, ResourcePermit};
use contextra_storage_ports::Error;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

const CLASS_COUNT: usize = 8;

#[derive(Debug, Clone, Copy)]
pub struct BudgetLimits {
    pub total_bytes: u64,
    pub class_bytes: [u64; CLASS_COUNT],
}

impl BudgetLimits {
    /// Default shares from SPEC §19. PurgeScratch is separately backed by the
    /// durable disk reserve and therefore receives no RAM share by default.
    pub fn from_total(total_bytes: u64) -> Self {
        let percent = |n: u64| ((u128::from(total_bytes) * u128::from(n)) / 100) as u64;
        Self {
            total_bytes,
            class_bytes: [
                percent(25), percent(15), percent(2), percent(20),
                percent(1), percent(20), percent(10), 0,
            ],
        }
    }

    pub fn with_class(mut self, class: ResourceClass, bytes: u64) -> Self {
        if let Some(limit) = self.class_bytes.get_mut(class_index(class)) { *limit = bytes; }
        self
    }

    fn validate(self) -> Result<Self, Error> {
        let sum = self.class_bytes.iter().try_fold(0u64, |sum, value| sum.checked_add(*value))
            .ok_or(Error::Limit { limit: "memory budget class limits overflow" })?;
        if sum > self.total_bytes {
            return Err(Error::Limit { limit: "memory budget class limits exceed total" });
        }
        Ok(self)
    }
}

struct BudgetInner {
    total_limit: u64,
    total_used: AtomicU64,
    class_limits: [u64; CLASS_COUNT],
    class_used: [AtomicU64; CLASS_COUNT],
}

pub struct Budget { inner: Arc<BudgetInner> }

impl Budget {
    pub fn new(limits: BudgetLimits) -> Result<Self, Error> {
        let limits = limits.validate()?;
        Ok(Self { inner: Arc::new(BudgetInner {
            total_limit: limits.total_bytes,
            total_used: AtomicU64::new(0),
            class_limits: limits.class_bytes,
            class_used: std::array::from_fn(|_| AtomicU64::new(0)),
        }) })
    }

    pub fn from_total(total_bytes: u64) -> Result<Self, Error> {
        Self::new(BudgetLimits::from_total(total_bytes))
    }

    pub fn used(&self) -> u64 { self.inner.total_used.load(Ordering::Acquire) }
    pub fn class_used(&self, class: ResourceClass) -> u64 {
        self.inner.class_used.get(class_index(class)).map(|used| used.load(Ordering::Acquire)).unwrap_or(0)
    }
    pub fn class_limit(&self, class: ResourceClass) -> u64 {
        self.inner.class_limits.get(class_index(class)).copied().unwrap_or(0)
    }

    fn reserve_counter(counter: &AtomicU64, limit: u64, bytes: u64) -> Result<(), Error> {
        let mut current = counter.load(Ordering::Acquire);
        loop {
            let next = current.checked_add(bytes)
                .ok_or(Error::Limit { limit: "memory budget accounting overflow" })?;
            if next > limit { return Err(Error::Limit { limit: "memory budget exhausted" }); }
            match counter.compare_exchange_weak(current, next, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => return Ok(()),
                Err(observed) => current = observed,
            }
        }
    }
}

impl MemoryBudget for Budget {
    fn try_reserve(&self, class: ResourceClass, bytes: u64) -> Result<ResourcePermit, Error> {
        let index = class_index(class);
        let class_counter = self.inner.class_used.get(index).ok_or(Error::Corrupt { place: "memory budget class index" })?;
        let class_limit = self.inner.class_limits.get(index).copied().ok_or(Error::Corrupt { place: "memory budget class index" })?;
        Self::reserve_counter(class_counter, class_limit, bytes)?;
        // The sum of class limits is validated <= total, so this cannot fail
        // unless the accounting invariant was violated internally.
        if let Err(error) = Self::reserve_counter(&self.inner.total_used, self.inner.total_limit, bytes) {
            class_counter.fetch_sub(bytes, Ordering::AcqRel);
            return Err(error);
        }
        let inner = Arc::clone(&self.inner);
        Ok(ResourcePermit::new(class, bytes, move || {
            if let Some(used) = inner.class_used.get(index) { used.fetch_sub(bytes, Ordering::AcqRel); }
            inner.total_used.fetch_sub(bytes, Ordering::AcqRel);
        }))
    }
}

fn class_index(class: ResourceClass) -> usize {
    match class {
        ResourceClass::Memtable => 0,
        ResourceClass::ClusterMeta => 1,
        ResourceClass::Guard => 2,
        ResourceClass::BlockCache => 3,
        ResourceClass::KeyCache => 4,
        ResourceClass::ExtensionDelta => 5,
        ResourceClass::Inflight => 6,
        ResourceClass::PurgeScratch => 7,
    }
}
