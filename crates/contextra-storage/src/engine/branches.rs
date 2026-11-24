//! Cluster-local branch metadata and visibility-chain primitives.
//!
//! Durable allocation and publication still belong to the committer: callers
//! must reserve the returned ID and persist the corresponding SYS_BRANCH op
//! before exposing a fork. This table is the validated in-memory projection.

use contextra_storage_ports::{BranchId, Error, Stamp, STAMP_MAX};
use std::collections::BTreeMap;
use std::time::Duration;
use zeroize::Zeroizing;

pub const MAIN_BRANCH: BranchId = BranchId(0);
pub const DEFAULT_MAX_BRANCHES: usize = 1024;
pub const DEFAULT_MAX_DEPTH: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForkPoint {
    Head,
    At(Stamp),
}

#[derive(Clone, PartialEq, Eq)]
pub struct BranchOptions {
    /// Opaque application label. The core never interprets or logs it.
    pub name: Vec<u8>,
    pub ttl: Option<Duration>,
}

impl Default for BranchOptions {
    fn default() -> Self { Self { name: Vec::new(), ttl: None } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchState {
    Live,
    Merged { at: Stamp },
    Dropped { at: Stamp },
}

#[derive(Clone)]
pub struct BranchMeta {
    id: BranchId,
    parent: Option<BranchId>,
    fork_stamp: Stamp,
    head_stamp: Stamp,
    depth: u8,
    state: BranchState,
    name: Zeroizing<Vec<u8>>,
    ttl: Option<Duration>,
}

impl BranchMeta {
    pub fn id(&self) -> BranchId { self.id }
    pub fn parent(&self) -> Option<BranchId> { self.parent }
    pub fn fork_stamp(&self) -> Stamp { self.fork_stamp }
    pub fn head_stamp(&self) -> Stamp { self.head_stamp }
    pub fn depth(&self) -> u8 { self.depth }
    pub fn state(&self) -> BranchState { self.state }
    pub fn name(&self) -> &[u8] { self.name.as_slice() }
    pub fn ttl(&self) -> Option<Duration> { self.ttl }
    pub fn is_live(&self) -> bool { self.state == BranchState::Live }
}

/// Per-cluster branch projection. IDs are monotone and deliberately not
/// recycled after drop. The owning storage layer serializes mutations through
/// its single writer and persists each mutation in the same Redo frame.
pub struct BranchTable {
    branches: BTreeMap<BranchId, BranchMeta>,
    next_id: u32,
    max_branches: usize,
    max_depth: u8,
    history_floor: Stamp,
}

impl BranchTable {
    pub fn new(history_floor: Stamp, main_head: Stamp, max_branches: usize, max_depth: u8) -> Result<Self, Error> {
        if history_floor > main_head || main_head > STAMP_MAX {
            return Err(Error::Limit { limit: "branch initial stamp bounds" });
        }
        if max_branches == 0 || max_depth == 0 {
            return Err(Error::Limit { limit: "branch limits must be nonzero" });
        }
        let mut branches = BTreeMap::new();
        branches.insert(MAIN_BRANCH, BranchMeta {
            id: MAIN_BRANCH,
            parent: None,
            fork_stamp: history_floor,
            head_stamp: main_head,
            depth: 0,
            state: BranchState::Live,
            name: Zeroizing::new(Vec::new()),
            ttl: None,
        });
        Ok(Self { branches, next_id: 1, max_branches, max_depth, history_floor })
    }

    pub fn history_floor(&self) -> Stamp { self.history_floor }
    pub fn len(&self) -> usize { self.branches.values().filter(|branch| !matches!(branch.state, BranchState::Dropped { .. })).count() }
    pub fn is_empty(&self) -> bool { self.branches.is_empty() }
    pub fn get(&self, id: BranchId) -> Option<&BranchMeta> { self.branches.get(&id) }
    pub fn iter(&self) -> impl Iterator<Item = &BranchMeta> { self.branches.values() }

    /// Creates an O(1) child descriptor. `id` is consumed monotonically; in a
    /// durable integration it must come from a completed durable reservation.
    /// `commit_stamp` is the outer stamp of the SYS_BRANCH record. A `Head`
    /// fork is pinned there, as required by the on-disk branch contract.
    pub fn fork(&mut self, id: BranchId, parent: BranchId, point: ForkPoint, options: BranchOptions, parent_head: Stamp, commit_stamp: Stamp) -> Result<Stamp, Error> {
        if id.0 != self.next_id { return Err(Error::Limit { limit: "branch id must be the next reserved id" }); }
        if self.branches.contains_key(&id) { return Err(Error::Corrupt { place: "branch id reused" }); }
        if options.name.len() > 64 { return Err(Error::Limit { limit: "branch name exceeds 64 bytes" }); }
        if self.len() >= self.max_branches { return Err(Error::Limit { limit: "branch count limit" }); }
        let parent_meta = self.branches.get(&parent).ok_or(Error::BranchClosed)?;
        if !parent_meta.is_live() || parent_meta.head_stamp != parent_head { return Err(Error::BranchClosed); }
        if commit_stamp <= parent_head || commit_stamp > STAMP_MAX {
            return Err(Error::Limit { limit: "branch fork record stamp must follow parent head" });
        }
        let depth = parent_meta.depth.checked_add(1).ok_or(Error::Limit { limit: "branch depth overflow" })?;
        if depth > self.max_depth { return Err(Error::Limit { limit: "branch depth limit" }); }
        let fork_stamp = match point {
            ForkPoint::Head => commit_stamp,
            ForkPoint::At(stamp) if stamp < self.history_floor => return Err(Error::BelowFloor),
            ForkPoint::At(stamp) if stamp > parent_head => return Err(Error::Limit { limit: "fork point beyond parent head" }),
            ForkPoint::At(stamp) => stamp,
        };
        let next_id = self.next_id.checked_add(1).ok_or(Error::Exhausted)?;
        self.branches.insert(id, BranchMeta {
            id, parent: Some(parent), fork_stamp, head_stamp: fork_stamp, depth,
            state: BranchState::Live, name: Zeroizing::new(options.name), ttl: options.ttl,
        });
        self.next_id = next_id;
        Ok(fork_stamp)
    }

    /// Advances a live branch head after its commit has been durably accepted.
    pub fn advance_head(&mut self, id: BranchId, expected: Stamp, next: Stamp) -> Result<(), Error> {
        if next <= expected || next > STAMP_MAX { return Err(Error::Limit { limit: "branch head stamps must increase" }); }
        let branch = self.branches.get_mut(&id).ok_or(Error::BranchClosed)?;
        if !branch.is_live() { return Err(Error::BranchClosed); }
        if branch.head_stamp != expected { return Err(Error::Conflict); }
        branch.head_stamp = next;
        Ok(())
    }

    /// Merge is terminal for the source and valid only for a direct child.
    /// The caller must atomically persist target mutations and this transition.
    pub fn mark_merged(&mut self, source: BranchId, target: BranchId, expected_source_head: Stamp, expected_target_head: Stamp, at: Stamp) -> Result<(), Error> {
        if at <= expected_source_head.max(expected_target_head) || at > STAMP_MAX {
            return Err(Error::Limit { limit: "merge stamp must follow both heads" });
        }
        let source_meta = self.branches.get(&source).ok_or(Error::BranchClosed)?;
        let target_meta = self.branches.get(&target).ok_or(Error::BranchClosed)?;
        if !source_meta.is_live() || !target_meta.is_live() { return Err(Error::BranchClosed); }
        if source_meta.parent != Some(target) { return Err(Error::Limit { limit: "merge requires a direct child branch" }); }
        if source_meta.head_stamp != expected_source_head || target_meta.head_stamp != expected_target_head {
            return Err(Error::Conflict);
        }
        if self.has_live_children(source) { return Err(Error::BranchHasChildren); }
        self.branches.get_mut(&source).ok_or(Error::BranchClosed)?.state = BranchState::Merged { at };
        self.branches.get_mut(&target).ok_or(Error::BranchClosed)?.head_stamp = at;
        Ok(())
    }

    /// Drops only a leaf. Merged branches remain readable until explicitly dropped.
    pub fn drop_branch(&mut self, id: BranchId, at: Stamp) -> Result<(), Error> {
        if id == MAIN_BRANCH { return Err(Error::Limit { limit: "main branch cannot be dropped" }); }
        let branch = self.branches.get(&id).ok_or(Error::BranchClosed)?;
        if matches!(branch.state, BranchState::Dropped { .. }) { return Ok(()); }
        if self.has_live_children(id) { return Err(Error::BranchHasChildren); }
        if at < branch.head_stamp { return Err(Error::Limit { limit: "branch drop stamp precedes branch head" }); }
        self.branches.get_mut(&id).ok_or(Error::BranchClosed)?.state = BranchState::Dropped { at };
        Ok(())
    }

    pub fn set_history_floor(&mut self, floor: Stamp) -> Result<(), Error> {
        if floor < self.history_floor { return Err(Error::Limit { limit: "history floor cannot move backward" }); }
        if self.branches.values().any(|branch| branch.is_live() && branch.parent.is_some() && branch.fork_stamp < floor) {
            return Err(Error::Limit { limit: "history floor exceeds a live branch fork point" });
        }
        self.history_floor = floor;
        Ok(())
    }

    /// Returns source-to-root visibility caps, including the requested branch.
    pub fn visibility_chain(&self, branch: BranchId, view_stamp: Stamp) -> Result<Vec<(BranchId, Stamp)>, Error> {
        let mut chain = Vec::new();
        let mut current = self.branches.get(&branch).ok_or(Error::BranchClosed)?;
        if matches!(current.state, BranchState::Dropped { .. }) { return Err(Error::BranchClosed); }
        let mut cap = view_stamp.min(current.head_stamp);
        loop {
            chain.push((current.id, cap));
            match current.parent {
                Some(parent) => {
                    cap = cap.min(current.fork_stamp);
                    current = self.branches.get(&parent).ok_or(Error::Corrupt { place: "branch parent missing" })?;
                }
                None => break,
            }
        }
        Ok(chain)
    }

    fn has_live_children(&self, parent: BranchId) -> bool {
        self.branches.values().any(|branch| branch.parent == Some(parent) && !matches!(branch.state, BranchState::Dropped { .. }))
    }
}
