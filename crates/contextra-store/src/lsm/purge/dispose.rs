// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Sichere Entsorgung obsoleter SSTable-Dateien via PendingDeleteJournal und Scrubbing (Spezifikation C5.1, ADR-10).
// INVARIANTEN: Atomare Aufzeichnung vor Entsorgung, RequireScrub-Fail-Closed bei ScrubRefusal, Propagierung aller Fehler.

use contextra_durable_fs::{
    durable_remove, scrub_if_safe, PendingDeleteJournal, ScrubOutcome, ScrubPolicy, ScrubRefusal,
};
use std::path::{Path, PathBuf};

/// Policy controlling disposal behavior when file scrubbing is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisposePolicy {
    /// Refusal during scrubbing is treated as a hard error and keeps the file in the journal.
    #[default]
    RequireScrub,
    /// Refusal permits falling back to durable removal without zero-overwriting.
    AllowRemoveUnscrubbed,
    /// Scrubbing disabled, fall back to standard durable removal.
    Disabled,
}

/// Report summarizing file disposal outcome.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrxDisposeReport {
    /// Number of files successfully removed.
    pub files_removed: usize,
    /// Number of files overwritten with zeros before removal.
    pub files_scrubbed: usize,
    /// Number of files removed without zero-overwriting.
    pub removed_unscrubbed: usize,
    /// List of scrub refusals encountered during disposal.
    pub refusals: Vec<(PathBuf, ScrubRefusal)>,
    /// List of files where scrubbing was refused resulting in residual risk.
    pub residual_risk: Vec<(PathBuf, ScrubRefusal)>,
}

/// Disposes obsolete files durably using a specified [`DisposePolicy`].
///
/// Records paths in `PendingDeleteJournal` before removal.
/// Attempts `scrub_if_safe` if policy is [`DisposePolicy::RequireScrub`] or [`DisposePolicy::AllowRemoveUnscrubbed`].
/// Under [`DisposePolicy::RequireScrub`], if scrubbing is refused (e.g. multiple hardlinks or CoW filesystem),
/// records refusal in `residual_risk` and returns an error, ensuring the file remains in the journal.
/// Under [`DisposePolicy::AllowRemoveUnscrubbed`], falls back to standard durable removal without zero-overwriting,
/// recording refusal in `residual_risk` and counting in `removed_unscrubbed`.
pub async fn dispose_obsolete_files_with_policy(
    storage_dir: &Path,
    obsolete_files: Vec<PathBuf>,
    policy: DisposePolicy,
) -> std::io::Result<OrxDisposeReport> {
    let dir = storage_dir.to_path_buf();

    tokio::task::spawn_blocking(move || {
        let journal_path = dir.join("pending_delete.journal");
        let mut journal = PendingDeleteJournal::open(&journal_path)?;

        // Record obsolete files in journal before starting physical removal (skip duplicates)
        for file in &obsolete_files {
            if !journal.pending().contains(file) {
                journal.record(file)?;
            }
        }

        let mut report = OrxDisposeReport::default();

        let scrub_policy = match policy {
            DisposePolicy::RequireScrub | DisposePolicy::AllowRemoveUnscrubbed => {
                ScrubPolicy::OverwriteZeros
            }
            DisposePolicy::Disabled => ScrubPolicy::Disabled,
        };

        journal.process(|path| {
            if !path.exists() {
                report.files_removed += 1;
                return Ok(());
            }

            match scrub_if_safe(path, scrub_policy, false, false) {
                Ok(ScrubOutcome::Scrubbed(_)) => {
                    report.files_removed += 1;
                    report.files_scrubbed += 1;
                    Ok(())
                }
                Ok(ScrubOutcome::RemovedUnscrubbed) => {
                    report.files_removed += 1;
                    report.removed_unscrubbed += 1;
                    Ok(())
                }
                Ok(ScrubOutcome::Refused(refusal)) => {
                    report.refusals.push((path.to_path_buf(), refusal));
                    report.residual_risk.push((path.to_path_buf(), refusal));
                    match policy {
                        DisposePolicy::RequireScrub => Err(std::io::Error::new(
                            std::io::ErrorKind::PermissionDenied,
                            format!("Scrubbing refused for {:?}: {:?}", path, refusal),
                        )),
                        DisposePolicy::AllowRemoveUnscrubbed | DisposePolicy::Disabled => {
                            durable_remove(path)?;
                            report.files_removed += 1;
                            report.removed_unscrubbed += 1;
                            Ok(())
                        }
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    report.files_removed += 1;
                    Ok(())
                }
                Err(err) => Err(err),
            }
        })?;

        Ok(report)
    })
    .await
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
}

/// Disposes obsolete files durably.
///
/// Records paths in `PendingDeleteJournal` before removal.
/// Wraps [`dispose_obsolete_files_with_policy`] with default policy [`DisposePolicy::RequireScrub`]
/// when `secure_delete` is `true`, or [`DisposePolicy::AllowRemoveUnscrubbed`] when `false`.
pub async fn dispose_obsolete_files(
    storage_dir: &Path,
    obsolete_files: Vec<PathBuf>,
    secure_delete: bool,
) -> std::io::Result<OrxDisposeReport> {
    let dispose_policy = if secure_delete {
        DisposePolicy::RequireScrub
    } else {
        DisposePolicy::AllowRemoveUnscrubbed
    };
    dispose_obsolete_files_with_policy(storage_dir, obsolete_files, dispose_policy).await
}
