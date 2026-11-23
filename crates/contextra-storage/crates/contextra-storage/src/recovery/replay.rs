//! Geordnetes, fail-closed Replay nach Abschluss des Integritäts- und Gate-Scans.

use super::scan::RecoveryScan;
use crate::store::redo::frame::{DecodedFrame, FrameKind};
use contextra_storage_ports::{ClusterId, Error, Stamp};
use std::collections::BTreeSet;

/// Engine adapter for recovery. Implementations decrypt and apply Change
/// payloads; this layer controls ordering and suppresses retired-cluster data.
pub trait ReplaySink {
    /// Apply all live ops from this committed frame as one recovery unit.
    /// Retired ops are omitted by the visibility barrier, while live ops retain
    /// the original frame's atomic application semantics.
    fn apply_change(&mut self, frame: &DecodedFrame<'_>, live_ops: &[usize]) -> Result<(), Error>;
    fn apply_control(&mut self, frame: &DecodedFrame<'_>) -> Result<(), Error>;
    fn finish(&mut self, last_stamp: Stamp) -> Result<(), Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplaySummary {
    pub applied_frames: u64,
    pub skipped_retired_ops: u64,
    pub last_stamp: Stamp,
}

/// Replays only after the complete scan has established the final retired set.
/// A mixed frame is passed intact to the sink only when all its ops are live;
/// sinks must implement atomic per-frame application for partial retirement.
pub fn replay(scan: &RecoveryScan<'_>, sink: &mut dyn ReplaySink) -> Result<ReplaySummary, Error> {
    replay_with_retired(scan, &scan.retired_clusters, sink)
}

pub fn replay_with_retired(
    scan: &RecoveryScan<'_>,
    retired: &BTreeSet<ClusterId>,
    sink: &mut dyn ReplaySink,
) -> Result<ReplaySummary, Error> {
    let mut summary = ReplaySummary { applied_frames: 0, skipped_retired_ops: 0, last_stamp: 0 };
    for frame in &scan.frames {
        summary.last_stamp = frame.head.stamp;
        match frame.head.kind {
            FrameKind::Change => {
                let live_ops: Vec<usize> = frame.payloads.iter().enumerate()
                    .filter_map(|(index, op)| (!retired.contains(&op.head.cluster)).then_some(index))
                    .collect();
                summary.skipped_retired_ops = summary.skipped_retired_ops
                    .checked_add((frame.payloads.len() - live_ops.len()) as u64)
                    .ok_or(Error::Limit { limit: "recovery skipped op count" })?;
                if !live_ops.is_empty() {
                    sink.apply_change(frame, &live_ops)?;
                    summary.applied_frames = summary.applied_frames.checked_add(1)
                        .ok_or(Error::Limit { limit: "recovery frame count" })?;
                }
            }
            FrameKind::Gate | FrameKind::Control | FrameKind::Reserve => {
                sink.apply_control(frame)?;
                summary.applied_frames = summary.applied_frames.checked_add(1)
                    .ok_or(Error::Limit { limit: "recovery frame count" })?;
            }
        }
    }
    sink.finish(scan.last_stamp)?;
    summary.last_stamp = scan.last_stamp;
    Ok(summary)
}
