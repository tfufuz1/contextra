use contextra_storage::env::faultfs::{Fault, FaultFs, FaultPlan};
use contextra_storage::env::memfs::MemFs;
use contextra_storage::ports::traits::{FileRole, Fs, IoCtx, ReadRange};
use contextra_storage::ports::Error;
use std::path::Path;

fn ctx() -> IoCtx {
    IoCtx { role: FileRole::Other, purpose: "port contract test" }
}

#[test]
fn read_ranges_preserves_request_order() -> Result<(), Error> {
    let fs = MemFs::new();
    let path = Path::new("data");
    let file = fs.create_new(path, ctx())?;
    file.preallocate(6, ctx())?;
    file.append_prealloc(b"abcdef", ctx())?;

    let mut first = [0; 2];
    let mut second = [0; 3];
    let mut ranges = [
        ReadRange { offset: 3, buffer: &mut first, bytes_read: 0 },
        ReadRange { offset: 0, buffer: &mut second, bytes_read: 0 },
    ];
    file.read_ranges(&mut ranges, ctx())?;
    assert_eq!(ranges[0].bytes_read, 2);
    assert_eq!(ranges[1].bytes_read, 3);
    assert_eq!(first, *b"de");
    assert_eq!(second, *b"abc");
    Ok(())
}

#[test]
fn faultfs_indexes_range_reads_through_read_at() -> Result<(), Error> {
    let base = MemFs::new();
    let path = Path::new("data");
    let file = base.create_new(path, ctx())?;
    file.preallocate(4, ctx())?;
    file.append_prealloc(b"data", ctx())?;

    let fs = FaultFs::new(base, FaultPlan { at_op: 1, fault: Fault::Eio });
    let file = fs.open(path, ctx())?;
    let mut buffer = [0; 4];
    let mut ranges = [ReadRange { offset: 0, buffer: &mut buffer, bytes_read: 0 }];
    assert!(matches!(file.read_ranges(&mut ranges, ctx()), Err(Error::Io { .. })));
    assert_eq!(fs.op_count(), 2);
    Ok(())
}

#[test]
fn memfs_instance_lock_is_released_on_drop() -> Result<(), Error> {
    let fs = MemFs::new();
    let path = Path::new("instance.lock");
    let first = fs.lock(path, ctx())?;
    drop(first);
    let second = fs.lock(path, ctx())?;
    drop(second);
    Ok(())
}

#[test]
fn faultfs_counts_and_injects_lock_operations() {
    let fs = FaultFs::new(MemFs::new(), FaultPlan { at_op: 0, fault: Fault::Eio });
    assert!(matches!(fs.lock(Path::new("instance.lock"), ctx()), Err(Error::Io { .. })));
    assert_eq!(fs.op_count(), 1);
}

#[test]
fn faultfs_injects_each_optional_fs_operation() {
    let operations: [fn(&FaultFs<MemFs>) -> Result<(), Error>; 3] = [
        |fs| fs.link(Path::new("source"), Path::new("target"), ctx()),
        |fs| fs.reflink(Path::new("source"), Path::new("target"), ctx()),
        |fs| fs.punch_hole(Path::new("source"), 0, 1, ctx()),
    ];
    for operation in operations {
        let fs = FaultFs::new(MemFs::new(), FaultPlan { at_op: 0, fault: Fault::Eio });
        assert!(matches!(operation(&fs), Err(Error::Io { .. })));
        assert_eq!(fs.op_count(), 1);
    }
}
