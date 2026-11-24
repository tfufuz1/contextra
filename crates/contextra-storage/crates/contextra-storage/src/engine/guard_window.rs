//! Stamp-sortierter, begrenzter Konfliktring für Guard-Stufe B (SPEC §6.3).
//!
//! Die Prüfung ist absichtlich konservativ: Punkt-Guards vergleichen nur
//! Fingerprints; Bereichs-Guards reagieren auf jede Mutation derselben
//! `(space, cluster, branch)`-Partition. Schlüsselvergleiche gehören in
//! Guard-Stufe A außerhalb des Sequencers.

use contextra_storage_ports::{BranchId, ClusterId, Error, SpaceId, Stamp};
use std::collections::VecDeque;

pub const MAX_GUARD_WINDOW_ENTRIES: usize = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardScope {
    pub space: SpaceId,
    pub cluster: ClusterId,
    pub branch: BranchId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardMutation {
    pub stamp: Stamp,
    /// Keyed process-local fingerprint of `(space, cluster, branch, key)`.
    pub fingerprint: u64,
    pub scope: GuardScope,
}

/// A bounded suffix of sequenced mutations, including mutations whose group
/// has not reached its durability point yet. It must be advanced in exact
/// stamp order by the single sequencer.
pub struct GuardWindow {
    entries: VecDeque<GuardMutation>,
    capacity: usize,
    last_stamp: Stamp,
}

impl GuardWindow {
    pub fn new(capacity: usize) -> Result<Self, Error> {
        if capacity == 0 || capacity > MAX_GUARD_WINDOW_ENTRIES {
            return Err(Error::Limit { limit: "guard window capacity" });
        }
        Ok(Self { entries: VecDeque::with_capacity(capacity), capacity, last_stamp: 0 })
    }

    /// Records one accepted mutation. Stamps must strictly increase; this
    /// prevents an out-of-order caller from making an old mutation invisible.
    pub fn record(&mut self, mutation: GuardMutation) -> Result<(), Error> {
        if mutation.stamp == 0 || mutation.stamp <= self.last_stamp {
            return Err(Error::Limit { limit: "guard window stamp order" });
        }
        self.last_stamp = mutation.stamp;
        if self.entries.len() == self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(mutation);
        Ok(())
    }

    /// Returns `GuardRetry` if the proof predates retained history or the
    /// same key changed after the caller's snapshot.
    pub fn check_point(&self, snapshot: Stamp, fingerprint: u64) -> Result<(), Error> {
        self.check_floor(snapshot)?;
        if self.entries.iter().any(|entry| entry.stamp > snapshot && entry.fingerprint == fingerprint) {
            return Err(Error::GuardRetry);
        }
        Ok(())
    }

    /// Conservative range validation. Any mutation in the same partition
    /// after the snapshot requires exact revalidation by the caller.
    pub fn check_range(&self, snapshot: Stamp, scope: GuardScope) -> Result<(), Error> {
        self.check_floor(snapshot)?;
        if self.entries.iter().any(|entry| entry.stamp > snapshot && entry.scope == scope) {
            return Err(Error::GuardRetry);
        }
        Ok(())
    }

    /// Earliest snapshot stamp that can still be checked exactly by this ring.
    /// A snapshot equal to the floor is valid; an older one must retry Stufe A.
    pub fn floor(&self) -> Stamp {
        self.entries.front().map_or(self.last_stamp, |entry| entry.stamp.saturating_sub(1))
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }

    fn check_floor(&self, snapshot: Stamp) -> Result<(), Error> {
        if snapshot > self.last_stamp {
            return Err(Error::Limit { limit: "guard snapshot beyond sequenced stamp" });
        }
        if snapshot < self.floor() { return Err(Error::GuardRetry); }
        Ok(())
    }
}
