//! Redo barrier cut (ER4) and segment retirement eligibility (SPEC §9.5, SP-REDO WP-05.4, INV-LOG-9).
#![forbid(unsafe_code)]

use super::frame::{EncodedFrame, FrameHead, FrameKind};
use super::writer::{GroupAck, RedoWriter};
use contextra_storage_ports::{CallerMeta, Durability, Error, Origin, Stamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BarrierCut {
    pub cut_stamp: Stamp,
    pub sealed_segment_id: u64,
    pub next_segment_id: u64,
    pub end_offset: u64,
}

/// Writes an aligned durable barrier marker onto `writer` so the current redo
/// segment can be sealed and rotated for ER4 plain-cluster purge.
pub fn cut_segment(
    writer: &mut RedoWriter<'_>,
    cut_stamp: Stamp,
    epoch: u64,
    current_segment_id: u64,
    next_segment_id: u64,
) -> Result<BarrierCut, Error> {
    if next_segment_id <= current_segment_id {
        return Err(Error::Limit {
            limit: "redo next segment id must advance monotonically",
        });
    }
    let frame = EncodedFrame {
        head: FrameHead {
            stamp: cut_stamp,
            kind: FrameKind::Control,
            idem: [0u8; 16],
            epoch,
            flags: 0,
            caller: CallerMeta {
                origin: Origin::Maintenance,
                caller_seq: current_segment_id,
            },
            ttl_bucket: 0,
            ops: Vec::new(),
        },
        payloads: Vec::new(),
    };
    let ack: GroupAck = writer.append_group(&[frame], Durability::Durable)?;
    Ok(BarrierCut {
        cut_stamp: ack.last_stamp,
        sealed_segment_id: current_segment_id,
        next_segment_id,
        end_offset: ack.end_offset,
    })
}

/// Returns true once both `replay_floor` and consumer `retain_floor` have moved
/// strictly past the cut stamp, making the sealed segment safe to retire (INV-LOG-9).
pub fn can_retire_segment(
    cut: &BarrierCut,
    replay_floor_stamp: Stamp,
    retain_floor_stamp: Stamp,
) -> bool {
    replay_floor_stamp > cut.cut_stamp && retain_floor_stamp > cut.cut_stamp
}
