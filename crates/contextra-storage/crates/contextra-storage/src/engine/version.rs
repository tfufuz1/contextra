//! Immutable read root (SP-MEMVER). Payload/run roots are added by flush phases.

use super::memtable::DetachedPartition;
use contextra_storage_ports::{Error, Stamp};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct Version {
    visible_stamp: Stamp,
    durable_stamp: Stamp,
    partitions: Arc<[Arc<DetachedPartition>]>,
    durable_partitions: Arc<[Arc<DetachedPartition>]>,
    gc_floor: Stamp,
}

impl Version {
    pub fn empty() -> Self { Self::default() }

    pub fn new(
        visible_stamp: Stamp,
        durable_stamp: Stamp,
        gc_floor: Stamp,
        partitions: Vec<Arc<DetachedPartition>>,
        durable_partitions: Vec<Arc<DetachedPartition>>,
    ) -> Result<Self, Error> {
        if durable_stamp > visible_stamp || gc_floor > visible_stamp {
            return Err(Error::Limit { limit: "version stamp ordering" });
        }
        Ok(Self { visible_stamp, durable_stamp, partitions: partitions.into(), durable_partitions: durable_partitions.into(), gc_floor })
    }

    pub fn visible_stamp(&self) -> Stamp { self.visible_stamp }
    pub fn durable_stamp(&self) -> Stamp { self.durable_stamp }
    pub fn gc_floor(&self) -> Stamp { self.gc_floor }
    pub fn partitions(&self) -> &[Arc<DetachedPartition>] { &self.partitions }
    pub fn durable_partitions(&self) -> &[Arc<DetachedPartition>] { &self.durable_partitions }

    pub fn at_stamp(&self, stamp: Stamp) -> Result<Self, Error> {
        if stamp < self.gc_floor {
            return Err(Error::Limit { limit: "view stamp outside version bounds" });
        }
        Ok(Self {
            visible_stamp: stamp,
            durable_stamp: self.durable_stamp.min(stamp),
            partitions: Arc::clone(&self.partitions),
            durable_partitions: Arc::clone(&self.durable_partitions),
            gc_floor: self.gc_floor,
        })
    }

    pub fn with_gc_floor(&self, gc_floor: Stamp) -> Result<Self, Error> {
        if gc_floor < self.gc_floor || gc_floor > self.visible_stamp {
            return Err(Error::Limit { limit: "version gc floor" });
        }
        Ok(Self {
            visible_stamp: self.visible_stamp,
            durable_stamp: self.durable_stamp,
            partitions: Arc::clone(&self.partitions),
            durable_partitions: Arc::clone(&self.durable_partitions),
            gc_floor,
        })
    }

    pub fn durable_snapshot(&self) -> Result<Self, Error> {
        Self::new(
            self.durable_stamp,
            self.durable_stamp,
            self.gc_floor.min(self.durable_stamp),
            self.durable_partitions.to_vec(),
            self.durable_partitions.to_vec(),
        )
    }
}
