//! # Crash-sicheres Pending-Delete-Journal
//!
//! Zweck: Dauerhafte Protokollierung und Nachverfolgung von Dateien, die zur physischen Löschung
//! vorgemerkt sind, über Prozessabstürze und Systemneustarts hinweg (Crash-Resumption).
//!
//! Ring-Zugehörigkeit: Ring 0 (Dauerhafte Dateisystem-Hilfsmittel).
//! Wird genutzt von: Bereinigungs- und Shredding-Routinen (z. B. WAL- und SSTable-Löschungen).
//! Wichtigste Typen:
//! - [`PendingDeleteJournal`]: Das Journal-Objekt zur Registrierung und schrittweisen Abarbeitung ausstehender Löschpfade.
//!
//! Invarianten:
//! - Atomare Persistenz: Änderungen am Journal werden stets über [`crate::atomic_replace`] geschrieben,
//!   sodass niemals halb geschriebene Journal-Zustände auf der Festplatte verbleiben.
//! - Fail-Closed: Schlägt das Schreiben des Journals oder ein Löschversuch fehl, wird das Ereignis
//!   an den verknüpften [`crate::AtomicHealthState`] gemeldet (`FsyncFailure` oder `Enospc`).
//! - Crash-Sicherheit: Nicht gelöschte Dateien verbleiben im Journal und werden beim nächsten Start erneut abgearbeitet.

// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Durable Pending-Delete Journal zur Aufzeichnung und Nachverfolgung ausstehender Löschungen.
// INVARIANTEN: Atomare Persistenz via atomic_replace, Fehler werden propagiert, HealthState Tracking.

use crate::{atomic_replace, durable_remove, AtomicHealthState, HealthEvent, HealthState};
use std::path::{Path, PathBuf};
use std::sync::Arc;

// ANCHOR[contextra-durable-fs:JOURNAL-STRUCT-001]
/// Journal zur dauerhaften Aufzeichnung von Löschaufträgen über Prozessabstürze hinweg.
///
/// Funktionsweise:
/// Bevor eine Datei physisch gelöscht wird, wird ihr Pfad im Journal vermerkt und synchronisiert.
/// Sollte das System während des Löschvorgangs abstürzen, kann ein neu gestarteter Prozess
/// das Journal öffnen und alle noch anhängigen Löschungen zuverlässig nachholen (Idempotenz).
#[derive(Debug)]
pub struct PendingDeleteJournal {
    journal_path: PathBuf,
    entries: Vec<PathBuf>,
    health: Arc<AtomicHealthState>,
}

impl PendingDeleteJournal {
    // ANCHOR[contextra-durable-fs:JOURNAL-OPEN]
    /// Öffnet oder erstellt ein `PendingDeleteJournal` am angegebenen Pfad mit neuem Standard-Health-State.
    ///
    /// Vorbedingungen:
    /// - Der übergeordnete Ordner von `journal_path` muss existieren und schreibbar sein.
    ///
    /// Nachbedingungen:
    /// - Bereits existierende Einträge im Journal werden vollständig in den Arbeitsspeicher geladen.
    ///
    /// Fehlerfälle:
    /// - Liefert `Err(std::io::Error)`, wenn die Journaldatei nicht gelesen werden kann.
    pub fn open(journal_path: impl Into<PathBuf>) -> std::io::Result<Self> {
        Self::open_with_health(journal_path, Arc::new(AtomicHealthState::new()))
    }

    // ANCHOR[contextra-durable-fs:JOURNAL-OPEN-WITH-HEALTH]
    /// Öffnet oder erstellt ein `PendingDeleteJournal` mit einer geteilten [`AtomicHealthState`]-Instanz.
    ///
    /// Ablauf:
    /// 1. Pfad konvertieren (`journal_path.into()`).
    /// 2. Prüfen, ob die Journaldatei bereits existiert.
    /// 3. Falls vorhanden, zeilenweise einlesen und Pfade bereinigt in `entries` sammeln.
    /// 4. Journal-Instanz zurückgeben.
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

    // ANCHOR[contextra-durable-fs:JOURNAL-HEALTH-STATE]
    /// Fragt den aktuellen Gesundheitsstatus ([`HealthState`]) des Journals ab.
    pub fn health_state(&self) -> HealthState {
        self.health.load()
    }

    // ANCHOR[contextra-durable-fs:JOURNAL-RECORD]
    /// Fügt einen neuen Dateipfad dauerhaft zum Journal hinzu.
    ///
    /// Ablauf:
    /// 1. Pfad an die interne Liste anhängen.
    /// 2. Journaldatei via [`PendingDeleteJournal::flush`] atomar auf Festplatte synchronisieren.
    ///
    /// Fehlerfälle:
    /// - Liefert `Err(std::io::Error)` bei Schreib- oder `fsync`-Fehlern.
    pub fn record(&mut self, path: &Path) -> std::io::Result<()> {
        self.entries.push(path.to_path_buf());
        self.flush()
    }

    // ANCHOR[contextra-durable-fs:JOURNAL-PENDING]
    /// Liefert eine unveränderliche Sicht auf alle aktuell ausstehenden Pfade.
    pub fn pending(&self) -> &[PathBuf] {
        &self.entries
    }

    // ANCHOR[contextra-durable-fs:JOURNAL-PROCESS]
    /// Verarbeitet alle ausstehenden Einträge unter Verwendung der Löschfunktion `remove_fn`.
    ///
    /// Erfolgreich gelöschte Dateien werden aus dem Journal entfernt.
    /// Tritt ein Fehler auf, bricht die Verarbeitung ab und verbleibende Einträge bleiben erhalten.
    ///
    /// Rückgabewert:
    /// - Anzahl der erfolgreich verarbeiteten Einträge als `Ok(usize)`.
    pub fn process<F>(&mut self, mut remove_fn: F) -> std::io::Result<usize>
    where
        F: FnMut(&Path) -> std::io::Result<()>,
    {
        self.process_inner(&mut remove_fn)
    }

    // NOTE: Trennung in process_inner verhindert Code-Duplizierung bei generischen Closures.
    fn process_inner<F>(&mut self, remove_fn: &mut F) -> std::io::Result<usize>
    where
        F: FnMut(&Path) -> std::io::Result<()>,
    {
        let mut processed_count = 0;
        let mut remaining = Vec::new();

        for (idx, entry) in self.entries.iter().enumerate() {
            match remove_fn(entry) {
                Ok(()) => {
                    processed_count += 1;
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    // NOTE: Datei existiert nicht mehr (z. B. bereits vor Crash gelöscht) -> gilt als erfolgreich bereinigt.
                    processed_count += 1;
                }
                Err(err) => {
                    // NOTE: Fehlgeschlagenen Eintrag und alle nachfolgenden unberührten Einträge im Journal behalten.
                    remaining.extend(self.entries[idx..].iter().cloned());
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

        // NOTE: Ist das Journal vollständig abgearbeitet, wird die Journaldatei selbst dauerhaft entfernt.
        if self.entries.is_empty() && self.journal_path.exists() {
            if let Err(err) = durable_remove(&self.journal_path) {
                self.health.record_event(HealthEvent::FsyncFailure);
                return Err(err);
            }
        }

        Ok(processed_count)
    }

    // ANCHOR[contextra-durable-fs:JOURNAL-FLUSH]
    /// Persistiert den aktuellen Journalinhalt atomar auf die Festplatte.
    ///
    /// Verwendet [`atomic_replace`], um eine temporäre Datei zu schreiben,
    /// diese per `fsync` abzusichern und atomar auf `journal_path` umzubenennen.
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
    // NOTE: Drop darf niemals panicken (INV-DURABLE-FS-NO-PANIC). Fehler beim automatischen Flush werden im HealthState protokolliert.
    fn drop(&mut self) {
        if !self.entries.is_empty() && self.journal_path.exists() {
            if let Err(_err) = self.flush() {
                self.health.record_event(HealthEvent::FsyncFailure);
            }
        }
    }
}
