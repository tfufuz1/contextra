//! Single-writer commit path: serialize → append/group-sync → publish.

use crate::store::redo::frame::EncodedFrame;
use crate::store::redo::writer::{GroupAck, RedoWriter};
use contextra_storage_ports::{Durability, Error, Stamp};

/// Adapter applies committed frames to the memtable and publishes a new MVCC
/// root. It is called only after the append/sync result is known and without
/// storage-internal locks held by this module.
pub trait CommitPublisher: Send + Sync {
    fn publish(&self, frames: &[EncodedFrame], ack: GroupAck) -> Result<(), Error>;
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
            || frames.windows(2).any(|pair| pair[0].head.stamp >= pair[1].head.stamp)
        {
            return Err(Error::Limit { limit: "commit stamps must increase" });
        }

        let ack = match self.writer.append_group(frames, durability) {
            Ok(ack) => ack,
            Err(error) => {
                self.poisoned = true;
                return Err(error);
            }
        };
        if self.publisher.publish(frames, ack).is_err() {
            self.poisoned = true;
            return Err(Error::Poisoned { cause: "durable commit could not be published" });
        }
        self.last_published = ack.last_stamp;
        Ok(ack)
    }

    pub fn last_published_stamp(&self) -> Stamp { self.last_published }

    pub fn is_poisoned(&self) -> bool { self.poisoned }
}
