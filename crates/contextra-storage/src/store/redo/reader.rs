//! Redo-Gruppenparser. Entsiegeln bleibt ausschließlich SP-RECOVERY.

use super::frame::{decode_frame, DecodedFrame};
use super::segment::{align_up, GROUP_MAGIC};
use super::torn_tail::TailState;
use contextra_storage_ports::{Error, Key32, Stamp};
use contextra_storage_ports::traits::Mac;

const GROUP_PREFIX: usize = 4;
const GROUP_HEADER: usize = 4 + 4 + 2;
const GROUP_CRC: usize = 4;

pub struct ScanResult<'a> {
    pub frames: Vec<DecodedFrame<'a>>,
    pub tail: TailState,
    pub valid_bytes: usize,
    pub last_stamp: Stamp,
    pub last_mac: [u8; 32],
}

pub struct RedoReader {
    pub alignment: u64,
    pub max_group_bytes: usize,
}

impl RedoReader {
    pub fn scan<'a>(
        &self,
        segment: &'a [u8],
        key: &Key32,
        mac: &dyn Mac,
        previous_mac: [u8; 32],
        previous_stamp: Stamp,
    ) -> Result<ScanResult<'a>, Error> {
        if self.alignment == 0 || !self.alignment.is_power_of_two() {
            return Err(Error::Limit { limit: "redo alignment" });
        }
        let mut offset = 0usize;
        let mut frames = Vec::new();
        let mut chain = previous_mac;
        let mut last_stamp = previous_stamp;
        loop {
            let aligned = usize::try_from(align_up(offset as u64, self.alignment)?)
                .map_err(|_| Error::Limit { limit: "redo scan offset" })?;
            if aligned >= segment.len() {
                let suffix = segment.get(offset..).ok_or(Error::Corrupt { place: "redo segment tail" })?;
                if suffix.iter().all(|byte| *byte == 0) {
                    return Ok(ScanResult { frames, tail: TailState::Clean, valid_bytes: offset, last_stamp, last_mac: chain });
                }
                let tail = super::torn_tail::tail_state(has_later_group(segment, aligned, self.alignment));
                return Ok(ScanResult { frames, tail, valid_bytes: offset, last_stamp, last_mac: chain });
            }
            let padding = segment.get(offset..aligned).ok_or(Error::Corrupt { place: "redo group padding" })?;
            if padding.iter().any(|byte| *byte != 0) { return Err(Error::Corrupt { place: "redo group padding" }); }
            let prefix_end = aligned.checked_add(GROUP_PREFIX).ok_or(Error::Corrupt { place: "redo group offset" })?;
            if prefix_end > segment.len() {
                let tail = super::torn_tail::tail_state(has_later_group(segment, aligned, self.alignment));
                return Ok(ScanResult { frames, tail, valid_bytes: offset, last_stamp, last_mac: chain });
            }
            let len_bytes: [u8; 4] = segment.get(aligned..prefix_end).ok_or(Error::Corrupt { place: "redo group length" })?
                .try_into().map_err(|_| Error::Corrupt { place: "redo group length" })?;
            let group_len = u32::from_le_bytes(len_bytes) as usize;
            if group_len == 0 {
                let suffix = segment.get(aligned..).ok_or(Error::Corrupt { place: "redo segment tail" })?;
                if suffix.iter().all(|byte| *byte == 0) {
                    return Ok(ScanResult { frames, tail: TailState::Clean, valid_bytes: offset, last_stamp, last_mac: chain });
                }
                let tail = super::torn_tail::tail_state(has_later_group(segment, aligned, self.alignment));
                return Ok(ScanResult { frames, tail, valid_bytes: offset, last_stamp, last_mac: chain });
            }
            let group_end = prefix_end.checked_add(group_len).ok_or(Error::Corrupt { place: "redo group length" })?;
            if group_len > (contextra_storage_ports::limits::REDO_SEGMENT_BYTES / 8) as usize
                || group_len < GROUP_HEADER + GROUP_CRC
            {
                if has_later_group(segment, aligned, self.alignment) { return Err(Error::Corrupt { place: "redo mid-log group size" }); }
                return Ok(ScanResult { frames, tail: TailState::Torn, valid_bytes: offset, last_stamp, last_mac: chain });
            }
            if group_end > segment.len() {
                let tail = super::torn_tail::tail_state(has_later_group(segment, aligned, self.alignment));
                return Ok(ScanResult { frames, tail, valid_bytes: offset, last_stamp, last_mac: chain });
            }
            let group = segment.get(prefix_end..group_end).ok_or(Error::Corrupt { place: "redo group body" })?;
            let group_without_crc_len = group.len().checked_sub(GROUP_CRC).ok_or(Error::Corrupt { place: "redo group CRC" })?;
            let body = group.get(..group_without_crc_len).ok_or(Error::Corrupt { place: "redo group body" })?;
            let crc_bytes: [u8; 4] = group.get(group_without_crc_len..).ok_or(Error::Corrupt { place: "redo group CRC" })?
                .try_into().map_err(|_| Error::Corrupt { place: "redo group CRC" })?;
            if crate::codec::checksums::crc32c(body) != u32::from_le_bytes(crc_bytes) {
                if has_later_group(segment, aligned, self.alignment) { return Err(Error::Corrupt { place: "redo mid-log group CRC" }); }
                return Ok(ScanResult { frames, tail: TailState::Torn, valid_bytes: offset, last_stamp, last_mac: chain });
            }
            if body.get(..4) != Some(GROUP_MAGIC.as_slice()) { return Err(Error::Unsupported { what: "redo group magic/version" }); }
            let frame_bytes_len = read_u32(body.get(4..8).ok_or(Error::Corrupt { place: "redo group header" })?)? as usize;
            let count = read_u16(body.get(8..10).ok_or(Error::Corrupt { place: "redo group header" })?)? as usize;
            if count > 1 && group_len.saturating_add(GROUP_PREFIX) > self.max_group_bytes {
                return Err(Error::Corrupt { place: "redo group exceeds configured limit" });
            }
            let frame_bytes = body.get(GROUP_HEADER..).ok_or(Error::Corrupt { place: "redo group frames" })?;
            if frame_bytes.len() != frame_bytes_len || count == 0 { return Err(Error::Corrupt { place: "redo group frame count" }); }
            let mut frame_offset = 0usize;
            for _ in 0..count {
                let length_end = frame_offset.checked_add(4).ok_or(Error::Corrupt { place: "redo frame offset" })?;
                let frame_len_bytes = frame_bytes.get(frame_offset..length_end).ok_or(Error::Corrupt { place: "redo frame length" })?;
                let frame_len = read_u32(frame_len_bytes)? as usize;
                let frame_end = length_end.checked_add(frame_len).ok_or(Error::Corrupt { place: "redo frame length" })?;
                let encoded = frame_bytes.get(frame_offset..frame_end).ok_or(Error::Corrupt { place: "redo frame truncated" })?;
                let decoded = match decode_frame(encoded, &chain, key, mac) {
                    Ok(decoded) => decoded,
                    Err(Error::Corrupt { .. }) if !has_later_group(segment, aligned, self.alignment) => {
                        return Ok(ScanResult { frames, tail: TailState::Torn, valid_bytes: offset, last_stamp, last_mac: chain });
                    }
                    Err(error) => return Err(error),
                };
                if decoded.head.stamp <= last_stamp { return Err(Error::Corrupt { place: "redo stamp order" }); }
                chain = decoded.chain_mac;
                last_stamp = decoded.head.stamp;
                frames.push(decoded);
                frame_offset = frame_end;
            }
            if frame_offset != frame_bytes.len() { return Err(Error::Corrupt { place: "redo group trailing frame bytes" }); }
            offset = group_end;
        }
    }

    pub fn heads_only<'a>(&self, result: &'a ScanResult<'a>) -> impl Iterator<Item = (&'a super::frame::FrameHead, Stamp)> + 'a {
        result.frames.iter().map(|frame| (&frame.head, frame.head.stamp))
    }
}

fn has_later_group(segment: &[u8], current: usize, alignment: u64) -> bool {
    let Ok(alignment) = usize::try_from(alignment) else { return false; };
    let Some(mut candidate) = current.checked_add(alignment) else { return false; };
    while candidate.checked_add(GROUP_PREFIX).is_some_and(|end| end <= segment.len()) {
        let Some(length_bytes) = segment.get(candidate..candidate + GROUP_PREFIX) else { return false; };
        let Ok(length_bytes) = <[u8; 4]>::try_from(length_bytes) else { return false; };
        let len = u32::from_le_bytes(length_bytes) as usize;
        let Some(end) = candidate.checked_add(GROUP_PREFIX).and_then(|start| start.checked_add(len)) else { return false; };
        if len >= GROUP_HEADER + GROUP_CRC && len <= segment.len() && end <= segment.len() {
            if let Some(group) = segment.get(candidate + GROUP_PREFIX..end) {
                let body_len = group.len().saturating_sub(GROUP_CRC);
                if group.get(..4) == Some(GROUP_MAGIC.as_slice()) {
                    if let (Some(body), Some(crc)) = (group.get(..body_len), group.get(body_len..)) {
                        if let Ok(crc) = <[u8; 4]>::try_from(crc) {
                            if crate::codec::checksums::crc32c(body) == u32::from_le_bytes(crc) { return true; }
                        }
                    }
                }
            }
        }
        let Some(next) = candidate.checked_add(alignment) else { return false; };
        candidate = next;
    }
    false
}

fn read_u16(bytes: &[u8]) -> Result<u16, Error> {
    let raw: [u8; 2] = bytes.try_into().map_err(|_| Error::Corrupt { place: "redo u16" })?;
    Ok(u16::from_le_bytes(raw))
}
fn read_u32(bytes: &[u8]) -> Result<u32, Error> {
    let raw: [u8; 4] = bytes.try_into().map_err(|_| Error::Corrupt { place: "redo u32" })?;
    Ok(u32::from_le_bytes(raw))
}
