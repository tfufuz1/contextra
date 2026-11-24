//! In-process storage for Volatile Spaces (SPEC §4.5, §6.1, INV-VOL-1).
//!
//! This state is intentionally independent of Redo, Runs, and Blobs. Dropping
//! the store clears its owned buffers on a best-effort basis. Callers must not
//! report durable success for a change that targets a Volatile Space.

use crate::spaces::SpaceClass;
use crate::sync::ranked::Ranked;
use contextra_storage_ports::{BranchId, Change, ClusterId, Durability, Error, SpaceId};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use zeroize::Zeroizing;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChangeDisposition {
    /// True only when every target Space is Durable and the request asks for
    /// Durable semantics. Volatile writes are never sent to Redo.
    pub durable: bool,
    pub has_volatile: bool,
    pub has_durable: bool,
}

/// Validates a Change's Space mix before mutation or I/O.
pub fn validate_change(
    change: &Change,
    mut class_of: impl FnMut(SpaceId) -> Result<SpaceClass, Error>,
) -> Result<ChangeDisposition, Error> {
    if change.ops.is_empty() {
        return Err(Error::Limit { limit: "change must contain at least one operation" });
    }
    let mut spaces = Vec::new();
    for op in &change.ops {
        let space = match op {
            contextra_storage_ports::Op::Put { space, .. }
            | contextra_storage_ports::Op::Delete { space, .. }
            | contextra_storage_ports::Op::RangeDelete { space, .. } => *space,
        };
        if !spaces.contains(&space) { spaces.push(space); }
    }

    let mut has_volatile = false;
    let mut has_durable = false;
    for space in spaces {
        match class_of(space)? {
            SpaceClass::Durable => has_durable = true,
            SpaceClass::Volatile => has_volatile = true,
        }
    }
    if has_volatile && has_durable && !change.allow_mixed {
        return Err(Error::Limit { limit: "mixed volatile/durable change requires allow_mixed" });
    }
    if has_volatile && change.durability == Durability::Durable {
        return Err(Error::Limit { limit: "change with volatile target cannot request durable success" });
    }
    Ok(ChangeDisposition {
        durable: has_durable && !has_volatile && change.durability == Durability::Durable,
        has_volatile,
        has_durable,
    })
}

struct VolatileKey {
    space: SpaceId,
    cluster: ClusterId,
    branch: BranchId,
    key: Zeroizing<Vec<u8>>,
}

impl PartialEq for VolatileKey {
    fn eq(&self, other: &Self) -> bool { self.cmp(other) == Ordering::Equal }
}
impl Eq for VolatileKey {}
impl PartialOrd for VolatileKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for VolatileKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.space.cmp(&other.space)
            .then_with(|| self.cluster.cmp(&other.cluster))
            .then_with(|| self.branch.cmp(&other.branch))
            .then_with(|| self.key.as_slice().cmp(other.key.as_slice()))
    }
}

/// A volatile value owns zeroizing copies of the header and body.
pub struct VolatileValue {
    pub header: Zeroizing<Vec<u8>>,
    pub body: Zeroizing<Vec<u8>>,
    pub weights: [u32; 4],
}

#[derive(Default)]
struct State(BTreeMap<VolatileKey, VolatileValue>);

/// Process-local point-value store. The rank-5 lock protects only its own
/// map; methods do not perform I/O or invoke caller-provided callbacks while
/// holding it. No read API retains a lock after returning.
pub struct VolatileStore {
    state: Ranked<State, 5>,
}

impl Default for VolatileStore {
    fn default() -> Self { Self::new() }
}

impl VolatileStore {
    pub fn new() -> Self { Self { state: Ranked::new(State::default()) } }

    pub fn put(
        &self,
        space: SpaceId,
        cluster: ClusterId,
        branch: BranchId,
        key: &[u8],
        header: &[u8],
        body: &[u8],
        weights: [u32; 4],
    ) -> Result<(), Error> {
        let entry = VolatileKey {
            space, cluster, branch, key: Zeroizing::new(key.to_vec()),
        };
        self.state.lock()?.0.insert(entry, VolatileValue {
            header: Zeroizing::new(header.to_vec()),
            body: Zeroizing::new(body.to_vec()),
            weights,
        });
        Ok(())
    }

    /// Returns an owned, zeroizing copy so callers cannot pin the store lock.
    pub fn get(
        &self,
        space: SpaceId,
        cluster: ClusterId,
        branch: BranchId,
        key: &[u8],
    ) -> Result<Option<VolatileValue>, Error> {
        let state = self.state.lock()?;
        let lookup = VolatileKey {
            space, cluster, branch, key: Zeroizing::new(key.to_vec()),
        };
        Ok(state.0.get(&lookup).map(|value| VolatileValue {
            header: Zeroizing::new(value.header.to_vec()),
            body: Zeroizing::new(value.body.to_vec()),
            weights: value.weights,
        }))
    }

    pub fn delete(
        &self,
        space: SpaceId,
        cluster: ClusterId,
        branch: BranchId,
        key: &[u8],
    ) -> Result<bool, Error> {
        let lookup = VolatileKey {
            space, cluster, branch, key: Zeroizing::new(key.to_vec()),
        };
        Ok(self.state.lock()?.0.remove(&lookup).is_some())
    }

    pub fn erase_space(&self, space: SpaceId) -> Result<usize, Error> {
        let mut state = self.state.lock()?;
        let before = state.0.len();
        state.0.retain(|key, _| key.space != space);
        Ok(before - state.0.len())
    }

    pub fn erase_cluster(&self, cluster: ClusterId) -> Result<usize, Error> {
        let mut state = self.state.lock()?;
        let before = state.0.len();
        state.0.retain(|key, _| key.cluster != cluster);
        Ok(before - state.0.len())
    }

    pub fn clear(&self) -> Result<usize, Error> {
        let mut state = self.state.lock()?;
        let count = state.0.len();
        state.0.clear();
        Ok(count)
    }

    pub fn len(&self) -> Result<usize, Error> { Ok(self.state.lock()?.0.len()) }
    pub fn is_empty(&self) -> Result<bool, Error> { Ok(self.state.lock()?.0.is_empty()) }
}
