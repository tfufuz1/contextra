// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Durable Pending-Delete Journal zur Aufzeichnung und Nachverfolgung ausstehender Löschungen.
// INVARIANTEN: Atomare Persistenz via atomic_replace, Fehler werden propagiert, keine let _ Verwürfe.

use crate::{atomic_replace, durable_remove, sync_dir};
use std::path::{Path, PathBuf};

/// Journal tracking files scheduled for deletion across process restarts or crashes.
#[derive(Debug)]
pub struct PendingDeleteJournal {
    journal_path: PathBuf,
    entries: Vec<PathBuf>,
}

impl PendingDeleteJournal {
    /// Opens or creates a `PendingDeleteJournal` at `journal_path`.
    pub fn open(journal_path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let journal_path = journal_path.into();
        let mut entries = Vec::new();

        if journal_path.exists() {
            let content = std::fs::read_to_string(&journal_path)?;
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    entries.push(PathBuf::from(trimmed));
                }
            }
        }

        Ok(Self {
            journal_path,
            entries,
        })
    }

    /// Appends a new file path to the pending delete journal durably.
    pub fn record(&mut self, path: &Path) -> std::io::Result<()> {
        self.entries.push(path.to_path_buf());
        self.flush()
    }

    /// Returns a slice of currently recorded pending file paths.
    pub fn pending(&self) -> &[PathBuf] {
        &self.entries
    }

    /// Processes all pending entries using the supplied removal closure `remove_fn`.
    ///
    /// Removes successfully processed paths from the journal and updates journal file on disk
    /// after each item to guarantee crash consistency. Missing files are treated as processed.
    pub fn process<F>(&mut self, mut remove_fn: F) -> std::io::Result<usize>
    where
        F: FnMut(&Path) -> std::io::Result<()>,
    {
        let mut processed_count = 0;
        let mut remaining = Vec::new();
        let original_entries = self.entries.clone();

        for (idx, entry) in original_entries.iter().enumerate() {
            let res = remove_fn(entry);
            let is_success = match res {
                Ok(()) => true,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => true,
                Err(err) => {
                    remaining.push(entry.clone());
                    self.entries = remaining.clone();
                    self.entries.extend(original_entries[idx + 1..].iter().cloned());
                    self.flush()?;
                    return Err(err);
                }
            };

            if is_success {
                if let Some(parent) = entry.parent() {
                    if parent.exists() {
                        sync_dir(parent)?;
                    }
                }
                processed_count += 1;
                // Persist updated remaining list to disk immediately
                self.entries = original_entries[idx + 1..].to_vec();
                self.entries.extend(remaining.clone());
                self.flush()?;
            }
        }

        // If journal is empty, attempt to durably remove the journal file itself
        if self.entries.is_empty() && self.journal_path.exists() {
            durable_remove(&self.journal_path)?;
        }

        Ok(processed_count)
    }

    /// Persists current entries to `journal_path` via `atomic_replace`.
    fn flush(&self) -> std::io::Result<()> {
        let mut buffer = String::new();
        for path in &self.entries {
            buffer.push_str(&path.to_string_lossy());
            buffer.push('\n');
        }
        atomic_replace(&self.journal_path, buffer.as_bytes())
    }
}
