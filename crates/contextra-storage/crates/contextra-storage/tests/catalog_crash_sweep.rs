use contextra_storage::catalog::file::{CatalogFile, OpenMode};
use contextra_storage::catalog::rollover::{publish_current, read_current, CurrentPointer};
use contextra_storage::catalog::edit::{CatalogEdit, PurgeBatchEdit, PurgeCompleteEdit, UnlinkAckEdit};
use contextra_storage::catalog::ops::FileEntry;
use contextra_storage::catalog::{CatalogIntegrity, CatalogOp, FileKind};
use contextra_storage::env::faultfs::{Fault, FaultFs, FaultPlan};
use contextra_storage::env::memfs::MemFs;
use contextra_storage_ports::traits::Fs;
use contextra_storage_ports::{Protection, SpaceId};
use std::path::Path;

const PATH: &str = "/store/CATALOG-0";

fn append_one(fs: &dyn Fs) -> Result<(), contextra_storage_ports::Error> {
    let mut catalog = CatalogFile::open(
        fs, Path::new(PATH), OpenMode::ReadWrite, CatalogIntegrity::Plain, None, None,
    )?;
    catalog.append_durable(
        &[CatalogOp::SpaceAdd { space: SpaceId(7), protection: Protection::Sealed }],
        CatalogIntegrity::Plain,
        None,
        None,
    )?;
    Ok(())
}

fn assert_valid_prefix(fs: &MemFs) {
    let opened = CatalogFile::open(
        fs, Path::new(PATH), OpenMode::ReadOnly, CatalogIntegrity::Plain, None, None,
    );
    match opened {
        Ok(catalog) => {
            let space_exists = catalog.state().spaces().contains_key(&SpaceId(7));
            assert!(space_exists || catalog.state().spaces().is_empty());
            assert!(catalog.is_writable() == false);
        }
        Err(contextra_storage_ports::Error::Io { kind: std::io::ErrorKind::NotFound, .. }) => {}
        Err(error) => panic!("unexpected post-crash catalog state: {error}"),
    }
}

#[test]
fn catalog_edit_is_old_or_new_after_crash_at_each_io() {
    let baseline = FaultFs::new(
        MemFs::new(),
        FaultPlan { at_op: u64::MAX, fault: Fault::Eio },
    );
    append_one(&baseline).expect("baseline append");
    let operation_count = baseline.op_count();

    for at_op in 0..operation_count {
        let faulted = FaultFs::new(
            MemFs::new(),
            FaultPlan { at_op, fault: Fault::Eio },
        );
        let _ = append_one(&faulted);
        let crashed = faulted.crash_image().expect("crash image");
        assert_valid_prefix(&crashed);
    }
}

#[test]
fn catalog_torn_append_and_lost_fsync_recover_old_or_new() {
    for fault in [Fault::TornWrite { keep: 7 }, Fault::LostFsync, Fault::Enospc] {
        let baseline = FaultFs::new(
            MemFs::new(),
            FaultPlan { at_op: u64::MAX, fault: Fault::Eio },
        );
        append_one(&baseline).expect("baseline append");
        let operation_count = baseline.op_count();
        for at_op in 0..operation_count {
            let faulted = FaultFs::new(MemFs::new(), FaultPlan { at_op, fault });
            let _ = append_one(&faulted);
            let crashed = faulted.crash_image().expect("crash image");
            assert_valid_prefix(&crashed);
        }
    }
}

#[test]
fn catalog_reopens_durable_edit() {
    let fs = MemFs::new();
    append_one(&fs).expect("durable append");
    let crashed = fs.crash_image().expect("crash image");
    let catalog = CatalogFile::open(
        &crashed, Path::new(PATH), OpenMode::ReadOnly, CatalogIntegrity::Plain, None, None,
    ).expect("reopen durable catalog");
    assert!(catalog.state().spaces().contains_key(&SpaceId(7)));
}

#[test]
fn catalog_group_edit_is_one_durability_record() {
    let fs = MemFs::new();
    let mut catalog = CatalogFile::open(
        &fs, Path::new(PATH), OpenMode::ReadWrite, CatalogIntegrity::Plain, None, None,
    ).expect("create catalog");
    let epochs = catalog.append_group_durable(
        &[
            vec![CatalogOp::SpaceAdd { space: SpaceId(7), protection: Protection::Sealed }],
            vec![CatalogOp::SpaceAdd { space: SpaceId(8), protection: Protection::Plain }],
        ],
        CatalogIntegrity::Plain, None, None,
    ).expect("append grouped edit");
    assert_eq!(epochs.len(), 2);
    assert!(epochs.iter().all(|epoch| *epoch == epochs[0]));
    assert_eq!(catalog.next_sequence(), 2);
    let reopened = CatalogFile::open(
        &fs, Path::new(PATH), OpenMode::ReadOnly, CatalogIntegrity::Plain, None, None,
    ).expect("reopen grouped edit");
    assert_eq!(reopened.next_sequence(), 2);
    assert_eq!(reopened.state().spaces().len(), 2);
}

#[test]
fn purge_batch_requires_durable_unlink_ack_before_completion() {
    let fs = MemFs::new();
    let mut catalog = CatalogFile::open(
        &fs, Path::new(PATH), OpenMode::ReadWrite, CatalogIntegrity::Plain, None, None,
    ).expect("create catalog");
    let cluster = contextra_storage_ports::ClusterId::from_wire([21; 16]);
    catalog.append_durable(
        &[
            CatalogOp::SpaceAdd { space: SpaceId(7), protection: Protection::Sealed },
            CatalogOp::ClusterRegister { cluster, protection: Protection::Sealed },
            CatalogOp::FilesAdd { file_id: 10, space: SpaceId(7), cluster: Some(cluster), kind: FileKind::Run,
                level: 0, bytes: 128, min_stamp: 1, max_stamp: 4 },
        ],
        CatalogIntegrity::Plain, None, None,
    ).expect("seed file catalog");
    let ticket = [42; 16];
    let replacement = FileEntry { file_id: 11, space: SpaceId(7), cluster: Some(cluster), kind: FileKind::Run,
        level: 0, bytes: 96, min_stamp: 1, max_stamp: 4 };
    catalog.append_edits_durable(
        &[CatalogEdit::PurgeBatch(PurgeBatchEdit::replace(ticket, 10, replacement).expect("new incarnation"))],
        CatalogIntegrity::Plain, None, None,
    ).expect("install purge batch");
    assert!(!catalog.state().files().contains_key(&10));
    assert!(catalog.state().files().contains_key(&11));
    assert_eq!(catalog.state().graveyard().len(), 1);
    assert!(catalog.append_edits_durable(
        &[CatalogEdit::PurgeComplete(PurgeCompleteEdit::after_owner_attestations(ticket))],
        CatalogIntegrity::Plain, None, None,
    ).is_err());
    catalog.append_edits_durable(
        &[CatalogEdit::UnlinkAck(UnlinkAckEdit::after_durable_unlink(ticket, 10))],
        CatalogIntegrity::Plain, None, None,
    ).expect("ack durable unlink");
    catalog.append_edits_durable(
        &[CatalogEdit::PurgeComplete(PurgeCompleteEdit::after_owner_attestations(ticket))],
        CatalogIntegrity::Plain, None, None,
    ).expect("complete purge");
    let reopened = CatalogFile::open(
        &fs, Path::new(PATH), OpenMode::ReadOnly, CatalogIntegrity::Plain, None, None,
    ).expect("reopen purge state");
    assert!(reopened.state().graveyard().is_empty());
    assert!(reopened.state().purge_is_complete(&ticket));
}

#[test]
fn current_pointer_replace_is_old_or_new_at_each_io_boundary() {
    let root = Path::new("/store");
    let initial = MemFs::new();
    publish_current(&initial, root, CurrentPointer { generation: 4 }).expect("seed CURRENT");
    let baseline = FaultFs::new(initial.clone(), FaultPlan { at_op: u64::MAX, fault: Fault::Eio });
    publish_current(&baseline, root, CurrentPointer { generation: 5 }).expect("baseline CURRENT replace");
    let operation_count = baseline.op_count();
    for at_op in 0..operation_count {
        let fs = FaultFs::new(initial.clone(), FaultPlan { at_op, fault: Fault::Eio });
        let result = publish_current(&fs, root, CurrentPointer { generation: 5 });
        drop(result);
        let crashed = fs.crash_image().expect("CURRENT crash image");
        let generation = read_current(&crashed, root).expect("durable CURRENT after crash").generation;
        assert!(generation == 4 || generation == 5);
    }
}
