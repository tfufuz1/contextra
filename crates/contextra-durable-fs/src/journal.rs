use crate::error::{DurableFsError, Result};
use crate::health::{AtomicHealthState, HealthEvent, HealthState};
use crate::platform::{sync_file, FlushKind};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Synchronisations-Politik für das Append-only-Journal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalSyncPolicy {
    /// Jeder Record wird sofort synchron per `fsync` persistiert.
    Always,
    /// Batch-Synchronisation nach maximaler Bytezahl oder Zeitintervall.
    GroupCommit {
        /// Maximale ungeflushte Bytezahl vor einem erzwungenen Flush.
        max_bytes: usize,
        /// Maximale Verzögerungsdauer vor einem Flush.
        max_delay: Duration,
    },
    /// Manueller Sync durch den Aufrufer via [`AppendOnlyJournal::flush`].
    Manual,
}

/// Recovery-Verhalten bei beschädigtem Journal-Ende (Torn-Write / Bit-Rot / Zero-Fill).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecordRecoveryPolicy {
    /// Bricht bei jedem Prüfsummen- oder Framing-Fehler sofort ab (Fail-Closed).
    #[default]
    FailClosed,
    /// Schneidet das Journal deterministisch vor dem ersten ungültigen Record ab (Truncate).
    TruncateCorruptTail,
}

/// Magic Bytes für den Header eines Journal-Records: `[0x43, 0x54, 0x58, 0x4A]` ("CTXJ").
pub const JOURNAL_MAGIC: [u8; 4] = [0x43, 0x54, 0x58, 0x4A];

/// Header-Größe für jeden Journal-Record:
/// 4 Bytes Magic + 8 Bytes SeqNo + 4 Bytes Payload-Länge + 4 Bytes CRC32 = 20 Bytes.
pub const RECORD_HEADER_SIZE: usize = 20;

/// Hilfsfunktion zur Berechnung einer CRC32-Prüfsumme.
#[must_use]
pub fn calculate_crc32(data: &[u8]) -> u32 {
    #[cfg(feature = "crc")]
    {
        crc32fast::hash(data)
    }
    #[cfg(not(feature = "crc"))]
    {
        let mut sum: u32 = 0;
        for &b in data {
            sum = sum.wrapping_mul(31).wrapping_add(b as u32);
        }
        sum
    }
}

/// Ein gelesener Journal-Eintrag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalRecord {
    /// Monoton steigende Sequenznummer des Records.
    pub seq_no: u64,
    /// Nutzdaten des Records.
    pub payload: Vec<u8>,
}

/// Universelles Append-only Journal mit Prüfsummen und Sequenznummern.
#[derive(Debug)]
pub struct AppendOnlyJournal {
    file: File,
    path: PathBuf,
    next_seq: u64,
    policy: JournalSyncPolicy,
    unflushed_bytes: usize,
    health: Arc<AtomicHealthState>,
}

impl AppendOnlyJournal {
    /// Öffnet oder erstellt ein Journal mit Standard-Sync-Politik ([`JournalSyncPolicy::Always`]).
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        Self::open_with_policy(
            path,
            JournalSyncPolicy::Always,
            Arc::new(AtomicHealthState::new()),
        )
    }

    /// Öffnet ein Journal ohne automatische Initial-Recovery (nützlich für manuelle Recovery).
    pub fn open_without_recovery(
        path: impl Into<PathBuf>,
        policy: JournalSyncPolicy,
        health: Arc<AtomicHealthState>,
    ) -> Result<Self> {
        let path = path.into();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(DurableFsError::Io)?;

        Ok(Self {
            file,
            path,
            next_seq: 1,
            policy,
            unflushed_bytes: 0,
            health,
        })
    }

    /// Öffnet oder erstellt ein Journal mit benutzerdefinierter Sync-Politik und Health-State.
    pub fn open_with_policy(
        path: impl Into<PathBuf>,
        policy: JournalSyncPolicy,
        health: Arc<AtomicHealthState>,
    ) -> Result<Self> {
        let mut journal = Self::open_without_recovery(path, policy, health)?;
        let records = journal.recover(RecordRecoveryPolicy::FailClosed)?;
        if let Some(last) = records.last() {
            journal.next_seq = last.seq_no.saturating_add(1);
        }
        Ok(journal)
    }

    /// Gibt den Pfad des Journals zurück.
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Hängt einen neuen Payload-Record an das Journal an.
    pub fn append(&mut self, payload: &[u8]) -> Result<u64> {
        if self.health.load().is_poisoned() {
            return Err(DurableFsError::PoisonedHandle);
        }

        let seq = self.next_seq;
        let payload_len = u32::try_from(payload.len()).map_err(|_| {
            DurableFsError::InvalidArgument("payload size exceeds u32::MAX".to_string())
        })?;

        let crc = calculate_crc32(payload);

        let mut header = [0u8; RECORD_HEADER_SIZE];
        header[0..4].copy_from_slice(&JOURNAL_MAGIC);
        header[4..12].copy_from_slice(&seq.to_le_bytes());
        header[12..16].copy_from_slice(&payload_len.to_le_bytes());
        header[16..20].copy_from_slice(&crc.to_le_bytes());

        self.file
            .seek(SeekFrom::End(0))
            .map_err(DurableFsError::Io)?;
        self.file.write_all(&header).map_err(DurableFsError::Io)?;
        self.file.write_all(payload).map_err(DurableFsError::Io)?;

        self.next_seq = self.next_seq.saturating_add(1);
        self.unflushed_bytes = self
            .unflushed_bytes
            .saturating_add(RECORD_HEADER_SIZE + payload.len());

        let needs_sync = match self.policy {
            JournalSyncPolicy::Always => true,
            JournalSyncPolicy::GroupCommit { max_bytes, .. } => self.unflushed_bytes >= max_bytes,
            JournalSyncPolicy::Manual => false,
        };

        if needs_sync {
            self.flush()?;
        }

        Ok(seq)
    }

    /// Flusht ausstehende Journal-Schreibvorgänge synchron auf stabilen Speicher (`INV-FSYNC-1`).
    pub fn flush(&mut self) -> Result<()> {
        if self.health.load().is_poisoned() {
            return Err(DurableFsError::PoisonedHandle);
        }

        if let Err(err) = sync_file(&self.file, FlushKind::DataAndMetadata) {
            self.health.record_event(HealthEvent::FsyncFailure);
            return Err(DurableFsError::Io(err));
        }

        self.unflushed_bytes = 0;
        Ok(())
    }

    /// Führt die Recovery des Journals durch und liest gültige Records ein.
    pub fn recover(&mut self, recovery_policy: RecordRecoveryPolicy) -> Result<Vec<JournalRecord>> {
        self.file
            .seek(SeekFrom::Start(0))
            .map_err(DurableFsError::Io)?;
        let mut records = Vec::new();
        let mut last_valid_offset = 0u64;

        let mut header_buf = [0u8; RECORD_HEADER_SIZE];
        loop {
            let n = match self.file.read_exact(&mut header_buf) {
                Ok(()) => RECORD_HEADER_SIZE,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    let pos = self.file.stream_position().map_err(DurableFsError::Io)?;
                    if pos == last_valid_offset {
                        break;
                    }
                    return self.handle_recovery_corruption(
                        recovery_policy,
                        last_valid_offset,
                        "incomplete record header",
                    );
                }
                Err(e) => return Err(DurableFsError::Io(e)),
            };

            if n == 0 {
                break;
            }

            let mut magic = [0u8; 4];
            magic.copy_from_slice(&header_buf[0..4]);
            if magic != JOURNAL_MAGIC {
                return self.handle_recovery_corruption(
                    recovery_policy,
                    last_valid_offset,
                    "invalid journal magic or zero-fill",
                );
            }

            let mut seq_bytes = [0u8; 8];
            seq_bytes.copy_from_slice(&header_buf[4..12]);
            let seq_no = u64::from_le_bytes(seq_bytes);

            let mut len_bytes = [0u8; 4];
            len_bytes.copy_from_slice(&header_buf[12..16]);
            let payload_len = u32::from_le_bytes(len_bytes) as usize;

            let mut crc_bytes = [0u8; 4];
            crc_bytes.copy_from_slice(&header_buf[16..20]);
            let expected_crc = u32::from_le_bytes(crc_bytes);

            let mut payload = vec![0u8; payload_len];
            if let Err(e) = self.file.read_exact(&mut payload) {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    return self.handle_recovery_corruption(
                        recovery_policy,
                        last_valid_offset,
                        "truncated record payload",
                    );
                }
                return Err(DurableFsError::Io(e));
            }

            let actual_crc = calculate_crc32(&payload);
            if actual_crc != expected_crc {
                return self.handle_recovery_corruption(
                    recovery_policy,
                    last_valid_offset,
                    "record checksum mismatch (bit-rot or torn write)",
                );
            }

            records.push(JournalRecord { seq_no, payload });
            last_valid_offset = self.file.stream_position().map_err(DurableFsError::Io)?;
        }

        if let Some(last) = records.last() {
            self.next_seq = last.seq_no.saturating_add(1);
        }

        Ok(records)
    }

    fn handle_recovery_corruption(
        &mut self,
        policy: RecordRecoveryPolicy,
        last_valid_offset: u64,
        msg: &str,
    ) -> Result<Vec<JournalRecord>> {
        match policy {
            RecordRecoveryPolicy::FailClosed => {
                self.health.record_event(HealthEvent::ReadChecksumWarning);
                Err(DurableFsError::Corruption(msg.to_string()))
            }
            RecordRecoveryPolicy::TruncateCorruptTail => {
                self.file
                    .set_len(last_valid_offset)
                    .map_err(DurableFsError::Io)?;
                self.flush()?;
                self.recover(RecordRecoveryPolicy::FailClosed)
            }
        }
    }

    /// Gibt den aktuellen Gesundheitszustand zurück.
    #[must_use]
    pub fn health_state(&self) -> HealthState {
        self.health.load()
    }
}

/// Journal zur dauerhaften Aufzeichnung von Entfernungsaufträgen über Prozessabstürze hinweg.
#[derive(Debug)]
pub struct PendingDeleteJournal {
    journal_path: PathBuf,
    entries: Vec<PathBuf>,
    health: Arc<AtomicHealthState>,
}

impl PendingDeleteJournal {
    /// Öffnet oder erstellt ein `PendingDeleteJournal` am angegebenen Pfad mit neuem Health-State.
    pub fn open(journal_path: impl Into<PathBuf>) -> Result<Self> {
        Self::open_with_health(journal_path, Arc::new(AtomicHealthState::new()))
    }

    /// Öffnet oder erstellt ein `PendingDeleteJournal` mit einer geteilten [`AtomicHealthState`]-Instanz.
    pub fn open_with_health(
        journal_path: impl Into<PathBuf>,
        health: Arc<AtomicHealthState>,
    ) -> Result<Self> {
        let journal_path = journal_path.into();
        let mut entries = Vec::new();

        if journal_path.exists() {
            let content = std::fs::read_to_string(&journal_path).map_err(DurableFsError::Io)?;
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

    /// Fragt den aktuellen Gesundheitsstatus ([`HealthState`]) des Journals ab.
    #[must_use]
    pub fn health_state(&self) -> HealthState {
        self.health.load()
    }

    /// Fügt einen neuen Dateipfad dauerhaft zum Journal hinzu.
    pub fn record(&mut self, path: &std::path::Path) -> Result<()> {
        self.entries.push(path.to_path_buf());
        self.flush()
    }

    /// Liefert eine unveränderliche Sicht auf alle aktuell ausstehenden Pfade.
    #[must_use]
    pub fn pending(&self) -> &[PathBuf] {
        &self.entries
    }

    /// Verarbeitet alle ausstehenden Einträge unter Verwendung der Entfernungsfunktion `remove_fn`.
    pub fn process<F>(&mut self, mut remove_fn: F) -> Result<usize>
    where
        F: FnMut(&std::path::Path) -> Result<()>,
    {
        self.process_inner(&mut remove_fn)
    }

    fn process_inner<F>(&mut self, remove_fn: &mut F) -> Result<usize>
    where
        F: FnMut(&std::path::Path) -> Result<()>,
    {
        let mut processed_count = 0;
        let mut remaining = Vec::new();

        for (idx, entry) in self.entries.iter().enumerate() {
            match remove_fn(entry) {
                Ok(()) => {
                    processed_count += 1;
                }
                Err(DurableFsError::NotFound { .. }) => {
                    processed_count += 1;
                }
                Err(err) => {
                    if let Some(sub) = self.entries.get(idx..) {
                        remaining.extend(sub.iter().cloned());
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

        if self.entries.is_empty() && self.journal_path.exists() {
            if let Err(err) = crate::durable_remove(&self.journal_path) {
                self.health.record_event(HealthEvent::FsyncFailure);
                return Err(err);
            }
        }

        Ok(processed_count)
    }

    fn flush(&self) -> Result<()> {
        let mut buffer = String::new();
        for path in &self.entries {
            buffer.push_str(&path.to_string_lossy());
            buffer.push('\n');
        }
        match crate::atomic_replace(&self.journal_path, buffer.as_bytes()) {
            Ok(()) => Ok(()),
            Err(err) => {
                self.health.record_event(HealthEvent::FsyncFailure);
                Err(err)
            }
        }
    }
}

impl Drop for PendingDeleteJournal {
    fn drop(&mut self) {
        if !self.entries.is_empty() && self.journal_path.exists() {
            if let Err(_err) = self.flush() {
                self.health.record_event(HealthEvent::FsyncFailure);
            }
        }
    }
}
