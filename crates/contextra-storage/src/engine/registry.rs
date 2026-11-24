//! Arc-published immutable roots with historical and durable snapshots.

use super::version::Version;
use contextra_storage_ports::{Error, Stamp};
use std::collections::{BTreeMap, Bound};
use std::sync::{Arc, RwLock};

struct Roots {
    visible: Arc<Version>,
    durable: Arc<Version>,
    history: BTreeMap<Stamp, Arc<Version>>,
}

pub struct VersionRegistry { roots: RwLock<Roots> }

impl Default for VersionRegistry {
    fn default() -> Self { Self::new() }
}

impl VersionRegistry {
    pub fn new() -> Self {
        let initial = Arc::new(Version::empty());
        let mut history = BTreeMap::new();
        history.insert(0, Arc::clone(&initial));
        Self { roots: RwLock::new(Roots { visible: Arc::clone(&initial), durable: initial, history }) }
    }

    pub fn snapshot(&self) -> Result<Arc<Version>, Error> {
        self.roots.read().map(|roots| Arc::clone(&roots.visible))
            .map_err(|_| Error::Poisoned { cause: "version registry poisoned" })
    }

    /// Publishes immutable roots and visibility/durability watermarks as one
    /// coherent state. Durable contents are supplied independently so Relaxed
    /// writes can never leak into `view_durable`.
    pub fn publish(&self, version: Arc<Version>) -> Result<(), Error> {
        self.publish_group(&[version])
    }

    pub fn publish_group(&self, versions: &[Arc<Version>]) -> Result<(), Error> {
        if versions.is_empty() { return Err(Error::Limit { limit: "version publication group must not be empty" }); }
        let mut roots = self.roots.write().map_err(|_| Error::Poisoned { cause: "version registry poisoned" })?;
        let mut previous_visible = roots.visible.visible_stamp();
        let mut previous_durable = roots.durable.durable_stamp();
        for version in versions {
            if version.visible_stamp() <= previous_visible
                || version.durable_stamp() < previous_durable
                || version.durable_stamp() > version.visible_stamp()
            {
                return Err(Error::Limit { limit: "version publication must be monotone" });
            }
            previous_visible = version.visible_stamp();
            previous_durable = version.durable_stamp();
        }
        for version in versions {
            roots.history.insert(version.visible_stamp(), Arc::clone(version));
        }
        let latest = versions.last().ok_or(Error::Limit { limit: "version publication group" })?;
        if latest.durable_stamp() > roots.durable.durable_stamp() {
            let (_, durable) = roots.history.range(..=latest.durable_stamp()).next_back()
                .ok_or(Error::Limit { limit: "durable version root unavailable" })?;
            roots.durable = Arc::clone(durable);
        }
        roots.visible = Arc::clone(latest);
        Ok(())
    }

    pub fn view_at(&self, stamp: Stamp) -> Result<Arc<Version>, Error> {
        let roots = self.roots.read().map_err(|_| Error::Poisoned { cause: "version registry poisoned" })?;
        if stamp < roots.visible.gc_floor() { return Err(Error::Limit { limit: "view stamp below gc_floor" }); }
        if stamp > roots.visible.visible_stamp() { return Err(Error::Limit { limit: "view stamp beyond visible stamp" }); }
        let (_, root) = roots.history.range(..=stamp).next_back()
            .ok_or(Error::Limit { limit: "historical version unavailable" })?;
        Ok(Arc::new(root.at_stamp(stamp)?))
    }

    pub fn view_durable(&self) -> Result<Arc<Version>, Error> {
        let roots = self.roots.read().map_err(|_| Error::Poisoned { cause: "version registry poisoned" })?;
        Ok(Arc::clone(&roots.durable))
    }

    pub fn advance_recovered(&self, stamp: Stamp) -> Result<(), Error> {
        let parent = self.snapshot()?;
        if stamp <= parent.visible_stamp() { return Ok(()); }
        let durable_stamp = stamp.max(parent.durable_stamp());
        let next = Arc::new(Version::commit(Arc::clone(&parent), stamp, durable_stamp, Vec::new())?);
        self.publish(next)
    }

    /// Drops versions strictly older than the durable history floor. Active
    /// readers retain their roots through Arc references.
    pub fn prune_before(&self, floor: Stamp) -> Result<(), Error> {
        let mut roots = self.roots.write().map_err(|_| Error::Poisoned { cause: "version registry poisoned" })?;
        if floor > roots.durable.durable_stamp() || floor < roots.visible.gc_floor() {
            return Err(Error::Limit { limit: "version gc floor" });
        }
        let predecessor = roots.history.range(..=floor).next_back().map(|(_, root)| Arc::clone(root))
            .ok_or(Error::Limit { limit: "version gc predecessor unavailable" })?;
        let floor_root = Arc::new(predecessor.compact_history(floor)?.at_stamp(floor)?);
        let mut compacted = BTreeMap::new();
        compacted.insert(floor, Arc::clone(&floor_root));
        for (stamp, root) in roots.history.range((Bound::Excluded(floor), Bound::Unbounded)) {
            compacted.insert(*stamp, Arc::new(root.compact_history(floor)?));
        }
        roots.visible = Arc::new(roots.visible.compact_history(floor)?);
        roots.durable = compacted.range(..=roots.durable.durable_stamp()).next_back()
            .map(|(_, root)| Arc::clone(root))
            .ok_or(Error::Limit { limit: "durable version below gc floor" })?;
        roots.history = compacted;
        Ok(())
    }
}
