use contextra_durable_fs::journal::PendingDeleteJournal;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_journal_crash_resume_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let journal_path = dir.path().join("crash_resume.journal");
    let file1 = dir.path().join("crash1.bin");
    let file2 = dir.path().join("crash2.bin");

    fs::write(&file1, b"c1")?;
    fs::write(&file2, b"c2")?;

    {
        let mut journal = PendingDeleteJournal::open(&journal_path)?;
        journal.record(&file1)?;
        journal.record(&file2)?;
    }

    // Neuer Prozess öffnet das Journal
    {
        let mut journal = PendingDeleteJournal::open(&journal_path)?;
        assert_eq!(journal.pending().len(), 2);

        // Erstes File löschen
        let processed = journal.process(|p| {
            if p == file1 {
                fs::remove_file(p)?;
                Ok(())
            } else {
                Err(contextra_durable_fs::DurableFsError::Io(
                    std::io::Error::other("simulated crash mid-process"),
                ))
            }
        });
        assert!(processed.is_err());
    }

    // Erneuter Prozessstart nach Crash
    {
        let mut journal = PendingDeleteJournal::open(&journal_path)?;
        assert_eq!(journal.pending().len(), 1);
        assert_eq!(journal.pending()[0], file2);

        let processed = journal.process(|p| {
            fs::remove_file(p)?;
            Ok(())
        })?;
        assert_eq!(processed, 1);
    }

    assert!(!file1.exists());
    assert!(!file2.exists());
    assert!(!journal_path.exists());

    Ok(())
}
