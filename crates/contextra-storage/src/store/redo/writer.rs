//! Sequenzierter Gruppenwriter: Frames vorab kodieren, genau ein Append und Sync.

use super::frame::{encode_frame, EncodedFrame};
use super::segment::{group_padding, GROUP_MAGIC};
use contextra_storage_ports::limits::MAX_GROUP_BYTES;
use contextra_storage_ports::traits::{File, FileRole, IoCtx, Mac};
use contextra_storage_ports::{Durability, Error, Key32, Stamp};

const GROUP_LENGTH_BYTES: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GroupAck {
    pub last_stamp: Stamp,
    pub durable: bool,
    pub end_offset: u64,
}

pub struct RedoWriter<'a> {
    file: Box<dyn File>,
    key: Key32,
    mac: &'a dyn Mac,
    previous_mac: [u8; 32],
    durable_stamp: Stamp,
    appended_stamp: Stamp,
    segment_bytes: u64,
    alignment: u64,
    poisoned: bool,
}

impl<'a> RedoWriter<'a> {
    pub fn new(
        file: Box<dyn File>,
        key: Key32,
        mac: &'a dyn Mac,
        previous_mac: [u8; 32],
        durable_stamp: Stamp,
        segment_bytes: u64,
        alignment: u64,
    ) -> Result<Self, Error> {
        if segment_bytes == 0 || alignment == 0 || !alignment.is_power_of_two() {
            return Err(Error::Limit { limit: "redo segment geometry" });
        }
        Ok(Self { file, key, mac, previous_mac, durable_stamp, appended_stamp: durable_stamp, segment_bytes, alignment, poisoned: false })
    }

    pub fn durable_stamp(&self) -> Stamp { self.durable_stamp }

    /// True only when an append or sync error made the writer unsafe to reuse.
    /// Validation/rotation errors before I/O leave the writer healthy.
    pub fn is_poisoned(&self) -> bool { self.poisoned }

    pub fn append_group(&mut self, frames: &[EncodedFrame], durability: Durability) -> Result<GroupAck, Error> {
        if self.poisoned { return Err(Error::Poisoned { cause: "redo writer is poisoned" }); }
        if frames.is_empty() { return Err(Error::Limit { limit: "redo group must contain a frame" }); }
        let current_offset = self.file.size()?;
        let padding = group_padding(current_offset, self.alignment)?;
        let mut frame_bytes = Vec::new();
        let mut chain = self.previous_mac;
        let mut last_stamp = self.appended_stamp;
        for frame in frames {
            if frame.head.stamp <= last_stamp {
                return Err(Error::Limit { limit: "redo stamps must increase" });
            }
            let encoded = encode_frame(frame, &chain, &self.key, self.mac)?;
            let mac_start = encoded.len().checked_sub(32).ok_or(Error::Corrupt { place: "encoded redo frame" })?;
            let next_chain: [u8; 32] = encoded.get(mac_start..)
                .ok_or(Error::Corrupt { place: "encoded redo MAC" })?
                .try_into().map_err(|_| Error::Corrupt { place: "encoded redo MAC" })?;
            frame_bytes.extend_from_slice(&encoded);
            chain = next_chain;
            last_stamp = frame.head.stamp;
        }

        let count = u16::try_from(frames.len()).map_err(|_| Error::Limit { limit: "redo frames per group" })?;
        let mut group_body = Vec::new();
        group_body.extend_from_slice(GROUP_MAGIC);
        let encoded_frames_len = u32::try_from(frame_bytes.len()).map_err(|_| Error::Limit { limit: "redo frames length" })?;
        group_body.extend_from_slice(&encoded_frames_len.to_le_bytes());
        group_body.extend_from_slice(&count.to_le_bytes());
        group_body.extend_from_slice(&frame_bytes);
        let group_crc = crate::codec::checksums::crc32c(&group_body);
        group_body.extend_from_slice(&group_crc.to_le_bytes());
        let group_len = u32::try_from(group_body.len()).map_err(|_| Error::Limit { limit: "redo group length" })?;
        let group_bytes = group_body.len().checked_add(4).ok_or(Error::Limit { limit: "redo group length" })?;
        if frames.len() > 1 && group_bytes > MAX_GROUP_BYTES {
            return Err(Error::Limit { limit: "redo group <= max_group_bytes" });
        }
        if group_bytes as u64 > self.segment_bytes / 8 {
            return Err(Error::Limit { limit: "redo group <= segment / 8" });
        }
        if current_offset.checked_add(padding as u64).and_then(|v| v.checked_add(group_bytes as u64))
            .is_none_or(|end| end > self.segment_bytes)
        {
            return Err(Error::Limit { limit: "redo segment rotation required" });
        }

        let mut append = Vec::with_capacity(padding + GROUP_LENGTH_BYTES + group_body.len());
        append.resize(padding, 0);
        append.extend_from_slice(&group_len.to_le_bytes());
        append.extend_from_slice(&group_body);
        let io = IoCtx { role: FileRole::Redo, purpose: "append_group" };
        if let Err(error) = self.file.append_prealloc(&append, io) {
            if matches!(error, Error::Io { .. } | Error::Poisoned { .. }) {
                self.poisoned = true;
                return Err(Error::Poisoned { cause: "redo append failed" });
            }
            return Err(error);
        }
        let end_offset = current_offset.checked_add(append.len() as u64)
            .ok_or(Error::Limit { limit: "redo segment offset" })?;
        self.previous_mac = chain;
        self.appended_stamp = last_stamp;
        let durable = durability == Durability::Durable;
        if durable {
            if self.file.sync_data(io).is_err() {
                self.poisoned = true;
                return Err(Error::Poisoned { cause: "redo group sync failed" });
            }
            self.durable_stamp = last_stamp;
        }
        Ok(GroupAck { last_stamp, durable, end_offset })
    }

    pub fn needs_rotation(&self, additional_bytes: u64) -> Result<bool, Error> {
        let offset = self.file.size()?;
        Ok(offset.checked_add(additional_bytes).is_none_or(|end| end > self.segment_bytes))
    }
}
