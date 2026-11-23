use contextra_crypto_ref::mac::Blake3Mac;
use contextra_storage::env::faultfs::{Fault, FaultFs, FaultPlan};
use contextra_storage::env::memfs::MemFs;
use contextra_storage::ports::traits::{FileRole, Fs, IoCtx};
use contextra_storage::ports::{BranchId, CallerMeta, ClusterId, Durability, Key32, Origin, SpaceId};
use contextra_storage::store::redo::frame::{EncodedFrame, FrameHead, FrameKind, OpHead};
use contextra_storage::store::redo::reader::RedoReader;
use contextra_storage::store::redo::writer::RedoWriter;
use std::io::ErrorKind;
use std::path::Path;

const PATH: &str = "/redo/redo-0.log";
const IO: IoCtx = IoCtx { role: FileRole::Redo, purpose: "crash_sweep_test" };

fn frame() -> EncodedFrame {
    let payload = b"payload".to_vec();
    EncodedFrame {
        head: FrameHead {
            stamp: 1, kind: FrameKind::Change, idem: [0; 16], epoch: 0, flags: 0,
            caller: CallerMeta { origin: Origin::User, caller_seq: 1 }, ttl_bucket: 0,
            ops: vec![OpHead { space: SpaceId(2), cluster: ClusterId::from_wire([3; 16]),
                branch: BranchId(0), kind: 1, key_epoch: 0, op_uid: [4; 16], payload_len: payload.len() as u32 }],
        },
        payloads: vec![payload],
    }
}

fn run(fs: &dyn Fs) -> Result<(), contextra_storage::ports::Error> {
    let file = fs.create_new(Path::new(PATH), IO)?;
    file.preallocate(1 << 20, IO)?;
    file.sync_all(IO)?;
    fs.sync_dir(Path::new("/redo"), IO)?;
    let key = Key32::from_array([9; 32]);
    let mac = Blake3Mac;
    let mut writer = RedoWriter::new(file, key, &mac, [0; 32], 0, 1 << 20, 8)?;
    writer.append_group(&[frame()], Durability::Durable)?;
    Ok(())
}

fn read_crash_image(image: &MemFs) -> Result<Vec<u8>, contextra_storage::ports::Error> {
    let file = match image.open(Path::new(PATH), IO) {
        Ok(file) => file,
        Err(contextra_storage::ports::Error::Io { kind: ErrorKind::NotFound, .. }) => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let len = usize::try_from(file.size()?).map_err(|_| contextra_storage::ports::Error::Limit { limit: "redo test file length" })?;
    let mut bytes = vec![0; len];
    let mut offset = 0usize;
    while offset < len {
        let start = u64::try_from(offset).map_err(|_| contextra_storage::ports::Error::Limit { limit: "redo test offset" })?;
        let dst = bytes.get_mut(offset..).ok_or(contextra_storage::ports::Error::Corrupt { place: "redo test buffer" })?;
        let read = file.read_at(start, dst, IO)?;
        if read == 0 { return Err(contextra_storage::ports::Error::Corrupt { place: "redo test short read" }); }
        offset = offset.checked_add(read).ok_or(contextra_storage::ports::Error::Limit { limit: "redo test offset" })?;
    }
    Ok(bytes)
}

fn assert_whole_group_or_empty(image: &MemFs) -> Result<(), contextra_storage::ports::Error> {
    let bytes = read_crash_image(image)?;
    if bytes.is_empty() { return Ok(()); }
    let key = Key32::from_array([9; 32]);
    let result = RedoReader { alignment: 8, max_group_bytes: 1 << 20 }
        .scan(&bytes, &key, &Blake3Mac, [0; 32], 0)?;
    assert!(result.frames.is_empty() || (result.frames.len() == 1 && result.last_stamp == 1));
    if result.frames.is_empty() { assert_eq!(result.valid_bytes, 0); }
    Ok(())
}

#[test]
fn redo_crash_sweep_preserves_only_whole_durable_groups() -> Result<(), contextra_storage::ports::Error> {
    let baseline = FaultFs::new(MemFs::new(), FaultPlan { at_op: u64::MAX, fault: Fault::Eio });
    run(&baseline)?;
    let count = baseline.op_count();
    for fault in [Fault::Eio, Fault::TornWrite { keep: 11 }, Fault::LostFsync, Fault::Enospc] {
        for at_op in 0..count {
            let fs = FaultFs::new(MemFs::new(), FaultPlan { at_op, fault });
            let outcome = run(&fs);
            drop(outcome);
            let image = fs.crash_image()?;
            assert_whole_group_or_empty(&image)?;
        }
    }
    Ok(())
}
