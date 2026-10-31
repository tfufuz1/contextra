//! # Testsuite: scrub_journal_tests.rs
//!
//! Zweck: Integrationstests für `scrub_if_safe` mit Hardlink-Verweigerung und `PendingDeleteJournal`.

use contextra_durable_fs::{
    durable_remove, scrub_if_safe, PendingDeleteJournal, ScrubOutcome, ScrubPolicy, ScrubRefusal,
};
use tempfile::tempdir;

// ANCHOR[contextra-durable-fs:TEST-SCRUB-JOURNAL-HARDLINK]
/// Prüft die Verweigerung des Nullen-Überschreibens bei mehreren Hardlinks.
#[test]
fn test_scrub_if_safe_hardlink_refusal() -> std::io::Result<()> {
    let dir = tempdir()?;
    let original = dir.path().join("original.txt");
    let link = dir.path().join("link.txt");

    std::fs::write(&original, b"sensitive data")?;
    std::fs::hard_link(&original, &link)?;

    let outcome = scrub_if_safe(&original, ScrubPolicy::OverwriteZeros, false, false)?;
    assert_eq!(
        outcome,
        ScrubOutcome::Refused(ScrubRefusal::MultipleHardlinks)
    );

    // Hardlink contents must remain intact!
    assert!(original.exists());
    assert_eq!(std::fs::read(&link)?, b"sensitive data");

    // Remove link and re-test
    durable_remove(&link)?;
    let outcome2 = scrub_if_safe(&original, ScrubPolicy::OverwriteZeros, false, false)?;

    #[cfg(target_os = "linux")]
    {
        // On ext4 or tmpfs, should scrub or return CoW refusal depending on filesystem
        match outcome2 {
            ScrubOutcome::Scrubbed(receipt) => {
                assert_eq!(receipt.bytes_scrubbed, 14);
                assert!(!original.exists());
            }
            ScrubOutcome::Refused(ScrubRefusal::CopyOnWriteFilesystem)
            | ScrubOutcome::Refused(ScrubRefusal::Unsupported) => {
                assert!(original.exists());
            }
            other => {
                assert_eq!(other, ScrubOutcome::RemovedUnscrubbed);
            }
        }
    }

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-SCRUB-JOURNAL-DISABLED]
/// Prüft die direkte Entfernung ohne Überschreiben bei deaktivierter Richtlinie.
#[test]
fn test_scrub_if_safe_policy_disabled() -> std::io::Result<()> {
    let dir = tempdir()?;
    let file = dir.path().join("file.txt");
    std::fs::write(&file, b"data")?;

    let outcome = scrub_if_safe(&file, ScrubPolicy::Disabled, false, false)?;
    assert_eq!(outcome, ScrubOutcome::RemovedUnscrubbed);
    assert!(!file.exists());

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-SCRUB-JOURNAL-RECOVERY]
/// Prüft die Crash-Recovery des PendingDeleteJournals bei unvollständiger Bereinigung.
#[test]
fn test_pending_delete_journal_crash_recovery() -> std::io::Result<()> {
    let dir = tempdir()?;
    let journal_path = dir.path().join("pending_deletes.journal");
    let file1 = dir.path().join("file1.sst");
    let file2 = dir.path().join("file2.sst");

    std::fs::write(&file1, b"sst data 1")?;
    std::fs::write(&file2, b"sst data 2")?;

    {
        let mut journal = PendingDeleteJournal::open(&journal_path)?;
        journal.record(&file1)?;
        journal.record(&file2)?;
        assert_eq!(journal.pending().len(), 2);
    }

    // Simulate process crash and re-opening journal
    {
        let mut recovered_journal = PendingDeleteJournal::open(&journal_path)?;
        assert_eq!(recovered_journal.pending().len(), 2);

        // Process deletion
        let processed = recovered_journal.process(durable_remove)?;
        assert_eq!(processed, 2);
        assert!(!file1.exists());
        assert!(!file2.exists());
    }

    // Re-open journal after successful processing
    {
        let journal_after = PendingDeleteJournal::open(&journal_path)?;
        assert_eq!(journal_after.pending().len(), 0);
    }

    Ok(())
}
