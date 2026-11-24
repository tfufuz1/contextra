//! Change-feed consumer and durable offset watermarking (SPEC §6, §12.4, SP-EXT, INV-CONS-1).
#![forbid(unsafe_code)]

use crate::erase::gate::GateIndex;
use contextra_storage_ports::{ClusterId, Error, InstanceId, Stamp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedPayload {
    Active(Vec<u8>),
    Redacted,
    Erased,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedItem {
    pub stamp: Stamp,
    pub cluster: ClusterId,
    pub key: Vec<u8>,
    pub payload: FeedPayload,
}

pub struct ConsumerFeed {
    instance: InstanceId,
    watermark: Stamp,
}

impl ConsumerFeed {
    pub fn new(instance: InstanceId, initial_watermark: Stamp) -> Self {
        Self {
            instance,
            watermark: initial_watermark,
        }
    }

    pub fn instance(&self) -> InstanceId {
        self.instance
    }

    pub fn watermark(&self) -> Stamp {
        self.watermark
    }

    /// Advances watermark up to durable_stamp. Rejects advancing beyond durable position.
    pub fn advance_watermark(&mut self, next_stamp: Stamp, durable_stamp: Stamp) -> Result<(), Error> {
        if next_stamp > durable_stamp {
            return Err(Error::Limit {
                limit: "consumer feed watermark cannot exceed durable stamp",
            });
        }
        if next_stamp < self.watermark {
            return Err(Error::Limit {
                limit: "consumer feed watermark cannot regress",
            });
        }
        self.watermark = next_stamp;
        Ok(())
    }

    /// Projects a raw mutation into a feed item, respecting gate visibility.
    pub fn project_item(
        &self,
        stamp: Stamp,
        cluster: ClusterId,
        key: Vec<u8>,
        raw_payload: Vec<u8>,
        gate_index: &GateIndex,
        is_purged: bool,
    ) -> FeedItem {
        let payload = if is_purged {
            FeedPayload::Erased
        } else if gate_index.check_visible(cluster, stamp).is_err() {
            FeedPayload::Redacted
        } else {
            FeedPayload::Active(raw_payload)
        };

        FeedItem {
            stamp,
            cluster,
            key,
            payload,
        }
    }
}
