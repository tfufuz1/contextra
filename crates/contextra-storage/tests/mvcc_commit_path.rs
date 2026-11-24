use contextra_storage::engine::committer::{CommitMutation, CommitMutationDecoder, CommitPublisher, MemtableCommitPublisher};
use contextra_storage::engine::memtable::Memtable;
use contextra_storage::engine::registry::VersionRegistry;
use contextra_storage::ports::{BranchId, ClusterId, Error, Stamp};
use contextra_storage::store::redo::frame::{EncodedFrame, FrameHead, FrameKind};
use contextra_storage::store::redo::writer::GroupAck;
use std::sync::Arc;

struct Decoder;
impl CommitMutationDecoder for Decoder {
    fn decode(&self, frame: &EncodedFrame) -> Result<Vec<CommitMutation>, Error> {
        Ok(vec![CommitMutation {
            cluster: ClusterId::from_wire([8; 16]), branch: BranchId(0),
            key: vec![frame.head.stamp as u8], kind: 1, payload: vec![frame.head.stamp as u8],
        }])
    }
}

fn frame(stamp: Stamp) -> EncodedFrame {
    EncodedFrame {
        head: FrameHead {
            stamp, kind: FrameKind::Change, idem: [0; 16], epoch: 0, flags: 0,
            caller: contextra_storage::ports::CallerMeta { origin: contextra_storage::ports::Origin::User, caller_seq: stamp },
            ttl_bucket: 0, ops: Vec::new(),
        },
        payloads: Vec::new(),
    }
}

#[test]
fn commit_publication_preserves_historical_and_durable_roots() -> Result<(), Error> {
    let memtable = Arc::new(Memtable::new());
    let versions = Arc::new(VersionRegistry::new());
    let publisher = MemtableCommitPublisher::new(Arc::clone(&memtable), Arc::clone(&versions), Arc::new(Decoder));
    publisher.publish(&[frame(1), frame(2)], GroupAck { last_stamp: 2, durable: true, end_offset: 64 })?;

    let at_one = versions.view_at(1)?;
    let cluster = ClusterId::from_wire([8; 16]);
    assert_eq!(at_one.get(cluster, BranchId(0), &[1], 1).map(|entry| entry.payload.as_slice()), Some(&[1][..]));
    assert!(at_one.get(cluster, BranchId(0), &[2], 1).is_none());
    assert_eq!(versions.view_durable()?.visible_stamp(), 2);

    publisher.publish(&[frame(3)], GroupAck { last_stamp: 3, durable: false, end_offset: 96 })?;
    assert_eq!(versions.snapshot()?.visible_stamp(), 3);
    assert_eq!(versions.view_durable()?.visible_stamp(), 2);
    assert!(versions.view_at(3)?.get(cluster, BranchId(0), &[3], 3).is_some());
    versions.prune_before(2)?;
    assert!(versions.view_at(1).is_err());
    let floor = versions.view_at(2)?;
    assert!(floor.get(cluster, BranchId(0), &[1], 2).is_some());
    assert!(floor.get(cluster, BranchId(0), &[2], 2).is_some());
    assert!(versions.view_at(3)?.get(cluster, BranchId(0), &[3], 3).is_some());
    Ok(())
}
