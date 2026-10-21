// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Testet DisposePolicy-Verhalten (RequireScrub vs AllowRemoveUnscrubbed) und Resume mit Journal-Dateien.
// INVARIANTEN: ScrubRefusal liefert Fehler unter RequireScrub, verbleibt im Journal. AllowRemoveUnscrubbed verzeichnet residual_risk und entfernt Datei.

use contextra_durable_fs::PendingDeleteJournal;
use contextra_store::lsm::purge::dispose::{dispose_obsolete_files_with_policy, DisposePolicy};
use contextra_store::lsm::purge::resume::resume_pending_with_policy;
use contextra_store::lsm::purge::OrxHooks;
use tempfile::tempdir;

#[tokio::test]
async fn test_dispose_policy_refusal_require_scrub() {
    let dir = tempdir().expect("tempdir");
    let storage_dir = dir.path();

    let target_file = storage_dir.join("data.sst");
    let link_file = storage_dir.join("data_link.sst");

    std::fs::write(&target_file, b"sensitive sstable content").expect("write target");
    std::fs::hard_link(&target_file, &link_file).expect("hard link");

    // secure_delete = true triggers ScrubPolicy::OverwriteZeros, which gets ScrubRefusal::MultipleHardlinks
    let res = dispose_obsolete_files_with_policy(
        storage_dir,
        vec![target_file.clone()],
        DisposePolicy::RequireScrub,
    )
    .await;

    assert!(res.is_err(), "RequireScrub must fail on ScrubRefusal");

    // Target file and hardlink must still exist
    assert!(
        target_file.exists(),
        "Target file must remain intact on refusal under RequireScrub"
    );
    assert!(link_file.exists(), "Hardlink file must remain intact");

    // Journal must still contain the pending delete entry
    let journal_path = storage_dir.join("pending_delete.journal");
    let journal = PendingDeleteJournal::open(&journal_path).expect("open journal");
    assert_eq!(journal.pending(), &[target_file]);
}

#[tokio::test]
async fn test_dispose_policy_refusal_allow_remove_unscrubbed() {
    let dir = tempdir().expect("tempdir");
    let storage_dir = dir.path();

    let target_file = storage_dir.join("data.sst");
    let link_file = storage_dir.join("data_link.sst");

    std::fs::write(&target_file, b"sensitive sstable content").expect("write target");
    std::fs::hard_link(&target_file, &link_file).expect("hard link");

    let report = dispose_obsolete_files_with_policy(
        storage_dir,
        vec![target_file.clone()],
        DisposePolicy::AllowRemoveUnscrubbed,
    )
    .await
    .expect("dispose succeeds with AllowRemoveUnscrubbed");

    assert_eq!(report.files_removed, 1);
    assert_eq!(report.files_scrubbed, 0);
    assert_eq!(report.removed_unscrubbed, 1);
    assert_eq!(report.refusals.len(), 1);
    assert_eq!(report.residual_risk.len(), 1);
    assert_eq!(report.residual_risk[0].0, target_file);

    assert!(!target_file.exists(), "Target file must be removed");
    assert!(link_file.exists(), "Other hardlink remains on disk");
}

#[tokio::test]
async fn test_resume_pending_disposes_journal_obsolete_files() {
    let dir = tempdir().expect("tempdir");
    let storage_dir = dir.path();

    let file1 = storage_dir.join("obsolete1.sst");
    let file2 = storage_dir.join("obsolete2.sst");

    std::fs::write(&file1, b"obsolete content 1").expect("write file1");
    std::fs::write(&file2, b"obsolete content 2").expect("write file2");

    let journal_path = storage_dir.join("pending_delete.journal");
    let mut journal = PendingDeleteJournal::open(&journal_path).expect("create journal");
    journal.record(&file1).expect("record file1");
    journal.record(&file2).expect("record file2");

    let hooks = OrxHooks::default();
    let res = resume_pending_with_policy(
        storage_dir,
        &hooks,
        None,
        DisposePolicy::AllowRemoveUnscrubbed,
    )
    .await
    .expect("resume_pending_with_policy succeeds");

    assert!(res.is_some(), "Must resume pending disposal");
    let (_purge_rep, disp_rep) = res.unwrap();

    assert_eq!(disp_rep.files_removed, 2);
    assert!(!file1.exists(), "file1 removed");
    assert!(!file2.exists(), "file2 removed");
    assert!(
        !journal_path.exists(),
        "journal file cleaned up after complete removal"
    );
}
