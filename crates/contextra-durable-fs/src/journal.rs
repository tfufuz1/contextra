// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Durable Pending-Delete Journal zur Aufzeichnung und Nachverfolgung ausstehender Löschungen.
// INVARIANTEN: Atomare Persistenz via atomic_replace, Fehler werden propagiert, HealthState Tracking.

use crate::{atomic_replace, durable_remove, AtomicHealthState, HealthEvent, HealthState};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Journal tracking files scheduled for deletion across process restarts or crashes.
#[derive(Debug)]
pub struct PendingDeleteJournal {
    journal_path: PathBuf,
    entries: Vec<PathBuf>,
    health: Arc<AtomicHealthState>,
}

impl PendingDeleteJournal {
    /// Opens or creates a `PendingDeleteJournal` at `journal_path`.
    pub fn open(journal_path: impl Into<PathBuf>) -> std::io::Result<Self> {
        Self::open_with_health(journal_path, Arc::new(AtomicHealthState::new()))
    }

    /// Opens or creates a `PendingDeleteJournal` with a custom [`AtomicHealthState`].
    pub fn open_with_health(
        journal_path: impl Into<PathBuf>,
        health: Arc<AtomicHealthState>,
    ) -> std::io::Result<Self> {
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
            health,
        })
    }

    /// Returns the current health state of the journal's underlying storage.
    pub fn health_state(&self) -> HealthState {
        self.health.load()
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
    /// Removes successfully processed paths from the journal and updates journal file on disk.
    pub fn process<F>(&mut self, mut remove_fn: F) -> std::io::Result<usize>
    where
        F: FnMut(&Path) -> std::io::Result<()>,
    {
        self.process_inner(&mut remove_fn)
    }

    fn process_inner<F>(&mut self, remove_fn: &mut F) -> std::io::Result<usize>
    where
        F: FnMut(&Path) -> std::io::Result<()>,
    {
        let mut processed_count: usize = 0;
        let mut remaining = Vec::new();

        for (idx, entry) in self.entries.iter().enumerate() {
            match remove_fn(entry) {
                Ok(()) => {
                    processed_count = processed_count.saturating_add(1);
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    // File already deleted/missing, treat as processed
                    processed_count = processed_count.saturating_add(1);
                }
                Err(err) => {
                    // Keep entry that failed AND all subsequent unprocessed entries
                    if let Some(unprocessed) = self.entries.get(idx..) {
                        remaining.extend(unprocessed.iter().cloned());
                    }
                    self.entries = remaining;
                    if let Err(flush_err) = self.flush() {
                        self.health.record_event(HealthEvent::FsyncFailure);
                        return Err(flush_err);
                    }
                    return Err(err);
                }
            }
        }

        self.entries = remaining;
        self.flush()?;

        // If journal is empty, attempt to durably remove the journal file itself
        if self.entries.is_empty() && self.journal_path.exists() {
            if let Err(err) = durable_remove(&self.journal_path) {
                self.health.record_event(HealthEvent::FsyncFailure);
                return Err(err);
            }
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
        match atomic_replace(&self.journal_path, buffer.as_bytes()) {
            Ok(()) => Ok(()),
            Err(err) => {
                if err.kind() == std::io::ErrorKind::StorageFull {
                    self.health.record_event(HealthEvent::Enospc);
                } else {
                    self.health.record_event(HealthEvent::FsyncFailure);
                }
                Err(err)
            }
        }
    }
}

impl Drop for PendingDeleteJournal {
    fn drop(&mut self) {
        // Drop must not panic. If journal flushing or cleanup on drop fails, record health event.
        if !self.entries.is_empty() && self.journal_path.exists() {
            if let Err(_err) = self.flush() {
                self.health.record_event(HealthEvent::FsyncFailure);
            }
        }
    }
}
