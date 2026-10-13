// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Entsorgung obsoleter SSTables / Dateien über PendingDeleteJournal mit optionaler Scrub-Prüfung (ADR-10).
// INVARIANTEN: Aufzeichnung in PendingDeleteJournal erst nach Manifest-Commit, Fehler werden propagiert.

use contextra_durable_fs::{
    durable_remove, scrub_if_safe, PendingDeleteJournal, ScrubOutcome, ScrubPolicy, ScrubRefusal,
};
use std::path::PathBuf;

/// Summary report of the file dispose phase.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrxDisposeReport {
    /// Number of files successfully removed from disk.
    pub files_removed: usize,
    /// Total bytes zero-scrubbed prior to deletion.
    pub bytes_scrubbed: u64,
    /// Refusal reasons encountered when secure scrubbing was skipped or unsafe.
    pub refusals: Vec<ScrubRefusal>,
}

/// Disposes obsolete files durably by recording them in `PendingDeleteJournal` and processing them.
///
/// Runs file I/O operations inside `tokio::task::spawn_blocking`.
pub async fn execute_dispose(
    journal_path: PathBuf,
    obsolete_files: Vec<PathBuf>,
    secure_delete: bool,
) -> std::io::Result<OrxDisposeReport> {
    tokio::task::spawn_blocking(move || {
        let mut journal = PendingDeleteJournal::open(&journal_path)?;

        // Record all obsolete files to journal first
        for file in &obsolete_files {
            journal.record(file)?;
        }

        let mut report = OrxDisposeReport::default();

        let process_res = journal.process(|file_path| {
            if secure_delete {
                // Check if scrubbing is safe
                let outcome = scrub_if_safe(
                    file_path,
                    ScrubPolicy::OverwriteZeros,
                    false, // reader_alive
                    false, // not_obsolete
                )?;

                match outcome {
                    ScrubOutcome::Scrubbed(receipt) => {
                        report.bytes_scrubbed += receipt.bytes_scrubbed;
                        report.files_removed += 1;
                        Ok(())
                    }
                    ScrubOutcome::RemovedUnscrubbed => {
                        report.files_removed += 1;
                        Ok(())
                    }
                    ScrubOutcome::Refused(refusal) => {
                        report.refusals.push(refusal);
                        // Fallback to standard durable removal
                        durable_remove(file_path)?;
                        report.files_removed += 1;
                        Ok(())
                    }
                }
            } else {
                durable_remove(file_path)?;
                report.files_removed += 1;
                Ok(())
            }
        });

        match process_res {
            Ok(_) => Ok(report),
            Err(err) => Err(err),
        }
    })
    .await
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
}
