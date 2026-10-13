// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integrationstests für DisposePolicy (RequireScrub, AllowRemoveUnscrubbed, Disabled) und Resume-Verhalten.
// INVARIANTEN: Scrubbing-Refusal führt bei RequireScrub zum Fehler (fail-closed, Datei bleibt im Journal), AllowRemoveUnscrubbed entfernt unzerstört und zählt separat.

use contextra_durable_fs::{PendingDeleteJournal, ScrubRefusal};
use contextra_store::lsm::purge::dispose::{dispose_obsolete_files_with_policy, DisposePolicy};
use contextra_store::lsm::purge::resume::resume_pending_with_policy;
use contextra_store::lsm::purge::OrxHooks;
use tempfile::tempdir;

#[tokio::test]
async fn test_dispose_policy_require_scrub_refusal_fails() -> std::io::Result<()> {
    let dir = tempdir()?;
    let file1 = dir.path().join("sstable1.sst");
    let link1 = dir.path().join("sstable1_link.sst");

    std::fs::write(&file1, b"sstable data content 123456789")?;
    std::fs::hard_link(&file1, &link1)?;

    let obsolete_files = vec![file1.clone()];

    // Attempt disposal with DisposePolicy::RequireScrub on a file with multiple hardlinks
    let res =
        dispose_obsolete_files_with_policy(dir.path(), obsolete_files, DisposePolicy::RequireScrub)
            .await;

    assert!(
        res.is_err(),
        "RequireScrub must fail with an error when scrubbing is refused"
    );

    // Verify file still exists on disk
    assert!(
        file1.exists(),
        "File must remain intact when scrubbing is refused under RequireScrub"
    );

    // Verify pending_delete.journal still contains the file
    let journal_path = dir.path().join("pending_delete.journal");
    let journal = PendingDeleteJournal::open(&journal_path)?;
    assert_eq!(
        journal.pending(),
        &[file1],
        "File must remain recorded in pending delete journal"
    );

    Ok(())
}

#[tokio::test]
async fn test_dispose_policy_allow_remove_unscrubbed_removes_file() -> std::io::Result<()> {
    let dir = tempdir()?;
    let file1 = dir.path().join("sstable1.sst");
    let link1 = dir.path().join("sstable1_link.sst");

    std::fs::write(&file1, b"sstable data content 123456789")?;
    std::fs::hard_link(&file1, &link1)?;

    let obsolete_files = vec![file1.clone()];

    // Attempt disposal with DisposePolicy::AllowRemoveUnscrubbed on a file with multiple hardlinks
    let report = dispose_obsolete_files_with_policy(
        dir.path(),
        obsolete_files,
        DisposePolicy::AllowRemoveUnscrubbed,
    )
    .await?;

    assert_eq!(report.files_removed, 1);
    assert_eq!(report.files_scrubbed, 0);
    assert_eq!(report.removed_unscrubbed, 1);
    assert_eq!(
        report.residual_risk,
        vec![(file1.clone(), ScrubRefusal::MultipleHardlinks)]
    );

    // Verify file1 was removed, while link1 still exists
    assert!(!file1.exists(), "Target file must be removed");
    assert!(link1.exists(), "Hardlink must still exist");

    // Verify journal was cleaned up on successful disposal
    let journal_path = dir.path().join("pending_delete.journal");
    let journal = PendingDeleteJournal::open(&journal_path)?;
    assert!(
        journal.pending().is_empty(),
        "Journal must be empty after completion"
    );

    Ok(())
}

#[tokio::test]
async fn test_resume_pending_cleans_orphaned_journal_entries() -> std::io::Result<()> {
    let dir = tempdir()?;
    let journal_path = dir.path().join("pending_delete.journal");
    let file1 = dir.path().join("sstable_orphaned.sst");

    std::fs::write(&file1, b"orphaned sstable data")?;

    // Record file1 into pending_delete.journal
    let mut journal = PendingDeleteJournal::open(&journal_path)?;
    journal.record(&file1)?;
    drop(journal);

    assert!(file1.exists());
    assert!(journal_path.exists());

    let hooks = OrxHooks::default();

    // Resume with AllowRemoveUnscrubbed policy
    let report_opt = resume_pending_with_policy(
        dir.path(),
        &hooks,
        None,
        DisposePolicy::AllowRemoveUnscrubbed,
    )
    .await
    .expect("Resume should succeed");

    assert!(report_opt.is_some());
    assert!(!file1.exists(), "Orphaned file should be removed on resume");

    Ok(())
}
