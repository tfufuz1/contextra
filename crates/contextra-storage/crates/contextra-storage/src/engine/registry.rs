//! Arc-published immutable roots with historical and durable snapshots.

use super::version::Version;
use contextra_storage_ports::{Error, Stamp};
use std::collections::BTreeMap;
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
        let mut roots = self.roots.write().map_err(|_| Error::Poisoned { cause: "version registry poisoned" })?;
        if version.visible_stamp() <= roots.visible.visible_stamp()
            || version.durable_stamp() < roots.durable.durable_stamp()
            || version.durable_stamp() > version.visible_stamp()
        {
            return Err(Error::Limit { limit: "version publication must be monotone" });
        }
        if version.durable_stamp() > roots.durable.durable_stamp() {
            roots.durable = Arc::new(version.durable_snapshot()?);
        }
        roots.history.insert(version.visible_stamp(), Arc::clone(&version));
        roots.visible = version;
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

    /// Drops versions strictly older than the durable history floor. Active
    /// readers retain their roots through Arc references.
    pub fn prune_before(&self, floor: Stamp) -> Result<(), Error> {
        let mut roots = self.roots.write().map_err(|_| Error::Poisoned { cause: "version registry poisoned" })?;
        if floor > roots.durable.durable_stamp() || floor < roots.visible.gc_floor() {
            return Err(Error::Limit { limit: "version gc floor" });
        }
        let predecessor = roots.history.range(..=floor).next_back().map(|(_, root)| Arc::clone(root))
            .ok_or(Error::Limit { limit: "version gc predecessor unavailable" })?;
        let floor_root = Arc::new(predecessor.at_stamp(floor)?);
        roots.history.retain(|stamp, _| *stamp >= floor);
        roots.history.insert(floor, floor_root);
        roots.visible = Arc::new(roots.visible.with_gc_floor(floor)?);
        roots.durable = Arc::new(roots.durable.with_gc_floor(floor)?);
        Ok(())
    }
}
