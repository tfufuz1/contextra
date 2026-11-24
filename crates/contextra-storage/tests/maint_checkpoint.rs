//! Integration tests for Maintenance tasks and Checkpoint/Restore (SPEC §10, §16).

use contextra_storage::checkpoint::create::{
    CheckpointCreator, CheckpointFileEntry, CheckpointManifest,
};
use contextra_storage::checkpoint::restore::{CheckpointRestorer, RestoreOptions};
use contextra_storage::env::anchor::MemAnchor;
use contextra_storage::env::memfs::MemFs;
use contextra_storage::maint::compaction::Compactor;
use contextra_storage::maint::flush::{FlushJob, FlushRunner};
use contextra_storage::maint::purge::PurgeRunner;
use contextra_storage::maint::scheduler::{MaintPriority, MaintScheduler};
use contextra_storage::maint::scrub::Scrubber;
use contextra_storage_ports::traits::AnchorValue;
use contextra_storage_ports::{ClusterId, Error, InstanceId, TombId};
use std::path::{Path, PathBuf};

#[test]
fn test_maint_scheduler_and_gc_floor() {
    let mut sched = MaintScheduler::new();
    let id_idle = sched.schedule(MaintPriority::Idle, "idle_task");
    let id_flush = sched.schedule(MaintPriority::Flush, "flush_task");
    let id_urgent = sched.schedule(MaintPriority::Urgent, "urgent_task");

    assert_eq!(sched.len(), 3);
    let next1 = sched.pop_next().unwrap();
    assert_eq!(next1.id, id_urgent);
    assert_eq!(next1.priority, MaintPriority::Urgent);

    let next2 = sched.pop_next().unwrap();
    assert_eq!(next2.id, id_flush);

    let next3 = sched.pop_next().unwrap();
    assert_eq!(next3.id, id_idle);
    assert!(sched.is_empty());

    let floor = MaintScheduler::compute_gc_floor(&[100, 200], &[150], &[120], 300);
    assert_eq!(floor, 100);
}

#[test]
fn test_flush_and_compaction_installation() {
    let fs = MemFs::new();
    let root = Path::new("test_runs");

    let job = FlushJob {
        file_id: 1,
        min_stamp: 10,
        max_stamp: 20,
        raw_data: b"run_data_payload_flush".to_vec(),
    };
    let path = FlushRunner::install_run(&fs, root, &job).unwrap();
    assert_eq!(path, root.join(format!("{:016x}.run", 1)));

    let compact_path = Compactor::install_compacted_run(
        &fs,
        root,
        2,
        b"compacted_run_data",
    )
    .unwrap();
    assert_eq!(compact_path, root.join(format!("{:016x}.run", 2)));
}

#[test]
fn test_purge_planning() {
    let c1 = ClusterId::from_wire([1; 16]);
    let c2 = ClusterId::from_wire([2; 16]);
    let c3 = ClusterId::from_wire([3; 16]);

    let file_clusters = vec![
        (101, vec![c1]),         // dead
        (102, vec![c1, c2, c3]), // survivor
        (103, vec![c2, c3]),     // unaffected
    ];

    let plan = PurgeRunner::plan_purge(&[c1], &file_clusters);
    assert_eq!(plan.dead_files, vec![101]);
    assert_eq!(plan.survivor_files, vec![102]);
}

#[test]
fn test_scrub_block_integrity() {
    use contextra_storage::codec::checksums::crc32c;

    let payload = b"critical_block_data_content";
    let crc = crc32c(payload);

    let mut valid_block = payload.to_vec();
    valid_block.extend_from_slice(&crc.to_be_bytes());

    assert!(Scrubber::scrub_block(&valid_block).unwrap());

    let mut corrupted = valid_block.clone();
    corrupted[0] ^= 0x01;
    assert!(!Scrubber::scrub_block(&corrupted).unwrap());
}

#[test]
fn test_checkpoint_create_and_restore_validation() {
    let fs = MemFs::new();
    let root = Path::new("ckpt_dir");

    let manifest = CheckpointManifest {
        instance_id: InstanceId::from_wire([5u8; 16]),
        checkpoint_id: 42,
        created_at_utc: 1234567890,
        catalog_epoch: 5,
        durable_stamp: 1000,
        erase_epoch: 10,
        tomb_epoch: 1,
        files: vec![CheckpointFileEntry {
            relative_path: PathBuf::from("0000000000000001.run"),
            size_bytes: 1024,
            checksum: 0xabcdef,
        }],
    };

    let manifest_path = CheckpointCreator::write_manifest(&fs, root, &manifest).unwrap();
    assert_eq!(manifest_path, root.join("manifest.ckpt"));
    let decoded_manifest = CheckpointCreator::read_manifest(&fs, root).unwrap();
    assert_eq!(decoded_manifest, manifest);

    let anchor = MemAnchor::new(AnchorValue {
        erase_epoch: 10,
        ledger_head: 20,
        catalog_epoch: 5,
    });

    // Case 1: Valid restore
    let valid_opts = RestoreOptions {
        external_fencing_attested: true,
        restore_floor_erase_epoch: 10,
    };
    assert!(CheckpointRestorer::validate_for_restore(&manifest, &valid_opts, &anchor).is_ok());

    // Case 2: Missing fencing fails closed (E-73)
    let no_fencing = RestoreOptions {
        external_fencing_attested: false,
        restore_floor_erase_epoch: 10,
    };
    assert!(matches!(
        CheckpointRestorer::validate_for_restore(&manifest, &no_fencing, &anchor),
        Err(Error::Restricted)
    ));

    // Case 3: Rollback detection against floor
    let rollback_opts = RestoreOptions {
        external_fencing_attested: true,
        restore_floor_erase_epoch: 15,
    };
    assert!(matches!(
        CheckpointRestorer::validate_for_restore(&manifest, &rollback_opts, &anchor),
        Err(Error::Rollback)
    ));

    let tombs = vec![TombId::from_bytes([0x88; 32])];
    let reconciled = CheckpointRestorer::reconcile_tombstones(&manifest, &tombs).unwrap();
    assert_eq!(reconciled, 1);
}
