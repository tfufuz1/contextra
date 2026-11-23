//! Single-writer commit path: serialize → append/group-sync → publish.

use crate::store::redo::frame::EncodedFrame;
use crate::store::redo::writer::{GroupAck, RedoWriter};
use super::memtable::{MemKey, Memtable};
use super::registry::VersionRegistry;
use super::version::{Version, VersionEntry};
use contextra_storage_ports::{BranchId, ClusterId, Durability, Error, Stamp};
use std::sync::Arc;

pub struct CommitMutation {
    pub cluster: ClusterId,
    pub branch: BranchId,
    pub key: Vec<u8>,
    pub kind: u8,
    pub payload: Vec<u8>,
}

/// Decodes already-authenticated frame operations. A sealed implementation
/// must verify the cluster generation before and after unsealing.
pub trait CommitMutationDecoder: Send + Sync {
    fn decode(&self, frame: &EncodedFrame) -> Result<Vec<CommitMutation>, Error>;
}

/// Adapter applies committed frames to the memtable and publishes a new MVCC
/// root. It is called only after the append/sync result is known and without
/// storage-internal locks held by this module.
pub trait CommitPublisher: Send + Sync {
    fn publish(&self, frames: &[EncodedFrame], ack: GroupAck) -> Result<(), Error>;
}

/// Concrete commit-to-memtable/MVCC adapter. The decoder owns operation
/// format and key-scope policy; this publisher owns stamp ordering and
/// post-durability visibility.
pub struct MemtableCommitPublisher {
    memtable: Arc<Memtable>,
    versions: Arc<VersionRegistry>,
    decoder: Arc<dyn CommitMutationDecoder>,
}

impl MemtableCommitPublisher {
    pub fn new(memtable: Arc<Memtable>, versions: Arc<VersionRegistry>, decoder: Arc<dyn CommitMutationDecoder>) -> Self {
        Self { memtable, versions, decoder }
    }
}

impl CommitPublisher for MemtableCommitPublisher {
    fn publish(&self, frames: &[EncodedFrame], ack: GroupAck) -> Result<(), Error> {
        let mut parent = self.versions.snapshot()?;
        let mut durable_stamp = parent.durable_stamp();
        let mut roots = Vec::with_capacity(frames.len());
        let mut staged = Vec::with_capacity(frames.len());

        // Decode and validate the complete group before touching the mutable
        // table. A malformed later frame must not leave earlier mutations
        // behind without a corresponding published version root.
        for frame in frames {
            if frame.head.stamp <= parent.visible_stamp() || frame.head.stamp > ack.last_stamp {
                return Err(Error::Corrupt { place: "commit publisher stamp order" });
            }
            let mutations = self.decoder.decode(frame)?;
            let mut entries = Vec::with_capacity(mutations.len());
            for mutation in &mutations {
                entries.push(VersionEntry::new(
                    mutation.cluster,
                    mutation.branch,
                    mutation.key.clone(),
                    frame.head.stamp,
                    mutation.kind,
                    mutation.payload.clone(),
                ));
            }
            if ack.durable { durable_stamp = frame.head.stamp; }
            parent = Arc::new(Version::commit(parent, frame.head.stamp, durable_stamp, entries)?);
            roots.push(Arc::clone(&parent));
            staged.push((frame.head.stamp, mutations));
        }
        if parent.visible_stamp() != ack.last_stamp { return Err(Error::Corrupt { place: "commit publisher group stamp mismatch" }); }

        // All fallible frame decoding and MVCC validation has completed. Only
        // now apply the prepared mutations, then make the matching roots
        // visible as one publication.
        for (stamp, mutations) in staged {
            for mutation in mutations {
                let key = MemKey::new(mutation.branch, mutation.key, stamp << 8 | u64::from(mutation.kind));
                self.memtable.insert(mutation.cluster, key, mutation.kind, &mutation.payload)?;
            }
        }
        self.versions.publish_group(&roots)
    }
}

pub struct Committer<'a> {
    writer: RedoWriter<'a>,
    publisher: Box<dyn CommitPublisher>,
    last_published: Stamp,
    poisoned: bool,
}

impl<'a> Committer<'a> {
    pub fn new(writer: RedoWriter<'a>, publisher: Box<dyn CommitPublisher>) -> Self {
        let initial = writer.durable_stamp();
        Self {
            writer,
            publisher,
            last_published: initial,
            poisoned: false,
        }
    }

    /// Synchronous sequencer entry point. Group formation is intentionally left
    /// to the intake layer; this method guarantees one append and one sync per
    /// supplied group, then publishes only the returned committed range.
    pub fn commit(&mut self, frames: &[EncodedFrame], durability: Durability) -> Result<GroupAck, Error> {
        if self.poisoned {
            return Err(Error::Poisoned { cause: "committer is poisoned" });
        }
        if frames.is_empty() { return Err(Error::Limit { limit: "commit group must not be empty" }); }
        if frames.first().is_some_and(|frame| frame.head.stamp <= self.last_published)
            || frames.windows(2).any(|pair| {
                pair.first().zip(pair.get(1))
                    .is_some_and(|(first, second)| first.head.stamp >= second.head.stamp)
            })
        {
            return Err(Error::Limit { limit: "commit stamps must increase" });
        }

        let ack = match self.writer.append_group(frames, durability) {
            Ok(ack) => ack,
            Err(error) => {
                if self.writer.is_poisoned() {
                    self.poisoned = true;
                    let first_stamp = frames.first().map(|frame| frame.head.stamp)
                        .ok_or(Error::Corrupt { place: "empty commit group after validation" })?;
                    let last_stamp = frames.last().map(|frame| frame.head.stamp)
                        .ok_or(Error::Corrupt { place: "empty commit group after validation" })?;
                    return Err(Error::Indeterminate { first_stamp, last_stamp });
                }
                return Err(error);
            }
        };
        if self.publisher.publish(frames, ack).is_err() {
            self.poisoned = true;
            let first_stamp = frames.first().map(|frame| frame.head.stamp)
                .ok_or(Error::Corrupt { place: "empty commit group after validation" })?;
            let last_stamp = frames.last().map(|frame| frame.head.stamp)
                .ok_or(Error::Corrupt { place: "empty commit group after validation" })?;
            return Err(Error::Indeterminate { first_stamp, last_stamp });
        }
        self.last_published = ack.last_stamp;
        Ok(ack)
    }

    pub fn last_published_stamp(&self) -> Stamp { self.last_published }

    pub fn is_poisoned(&self) -> bool { self.poisoned }
}
