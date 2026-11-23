use contextra_storage::env::faultfs::{Fault, FaultFs, FaultPlan};
use contextra_storage::env::memfs::MemFs;
use contextra_storage_ports::traits::{FileRole, Fs, IoCtx};
use std::path::Path;

#[test]
fn faultfs_tracks_every_file_role_without_out_of_bounds_access() {
    let base = MemFs::new();
    let ctx = IoCtx { role: FileRole::Other, purpose: "role_counter_regression" };
    let created = base.create_new(Path::new("/other/data"), ctx).unwrap();
    created.preallocate(8, ctx).unwrap();
    created.append_prealloc(b"x", ctx).unwrap();
    created.sync_all(ctx).unwrap();

    let fs = FaultFs::new(base, FaultPlan { at_op: u64::MAX, fault: Fault::Eio });
    let file = fs.open(Path::new("/other/data"), ctx).unwrap();
    let mut byte = [0];
    assert_eq!(file.read_at(0, &mut byte, ctx).unwrap(), 1);
    assert_eq!(byte, [b'x']);
    assert_eq!(fs.bytes_read(FileRole::Other), 1);
}
