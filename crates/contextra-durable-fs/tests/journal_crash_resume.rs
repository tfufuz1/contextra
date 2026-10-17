// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Integrationstests für PendingDeleteJournal Crash-Resumption und scrub_if_safe Hardlink-Sicherheitsprüfungen (AP-12, ADR-10).

use contextra_durable_fs::{
    durable_remove, scrub_if_safe, PendingDeleteJournal, ScrubOutcome, ScrubPolicy, ScrubRefusal,
};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_journal_process_crash_resume_idempotency() -> std::io::Result<()> {
    let dir = tempdir()?;
    let journal_path = dir.path().join("pending_delete.journal");

    let file1 = dir.path().join("file1.sst");
    let file2 = dir.path().join("file2.sst");
    let file3 = dir.path().join("file3.sst");

    File::create(&file1)?.write_all(b"data1")?;
    File::create(&file2)?.write_all(b"data2")?;
    File::create(&file3)?.write_all(b"data3")?;

    let mut journal = PendingDeleteJournal::open(&journal_path)?;
    journal.record(&file1)?;
    journal.record(&file2)?;
    journal.record(&file3)?;

    let mut count = 0;
    // Simulate crash after deleting file1
    let _ = journal.process(|path| {
        count += 1;
        if count == 1 {
            durable_remove(path)?;
            Ok(())
        } else {
            Err(std::io::Error::other("simulated crash mid-process"))
        }
    });

    assert!(!file1.exists(), "file1 should have been removed");
    assert!(file2.exists(), "file2 should still exist");
    assert!(file3.exists(), "file3 should still exist");

    // Re-open journal on restart and resume
    let mut resumed_journal = PendingDeleteJournal::open(&journal_path)?;
    assert_eq!(
        resumed_journal.pending().len(),
        2,
        "Only file2 and file3 should remain in journal"
    );

    let processed = resumed_journal.process(durable_remove)?;
    assert_eq!(processed, 2);

    assert!(!file2.exists());
    assert!(!file3.exists());
    assert!(
        !journal_path.exists(),
        "Journal file should be deleted when empty"
    );

    Ok(())
}

#[test]
fn test_scrub_if_safe_hardlink_refusal_and_safety() -> std::io::Result<()> {
    let dir = tempdir()?;
    let target = dir.path().join("original.sst");
    let link = dir.path().join("hardlink.sst");

    File::create(&target)?.write_all(b"sensitive_data")?;
    std::fs::hard_link(&target, &link)?;

    // Attempt scrub_if_safe on target with nlink == 2
    let outcome = scrub_if_safe(&target, ScrubPolicy::OverwriteZeros, false, false)?;

    #[cfg(unix)]
    {
        assert_eq!(
            outcome,
            ScrubOutcome::Refused(ScrubRefusal::MultipleHardlinks),
            "Scrubbing must be refused when nlink > 1"
        );
        // Original content must be intact and not zeroed out
        let content = std::fs::read_to_string(&link)?;
        assert_eq!(content, "sensitive_data");
    }

    Ok(())
}

#[test]
fn test_durable_remove_parent_dir_fsync() -> std::io::Result<()> {
    let dir = tempdir()?;
    let file_path = dir.path().join("test_file.tmp");

    File::create(&file_path)?.write_all(b"content")?;
    assert!(file_path.exists());

    durable_remove(&file_path)?;
    assert!(!file_path.exists());

    Ok(())
}
