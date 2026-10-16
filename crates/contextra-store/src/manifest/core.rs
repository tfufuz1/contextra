// FILE-CONTEXT
// STAND: 2026-09-29 (S-01 / T-04)
// ZWECK: Kernimplementierung des Manifests zum Lesen, Schreiben, Rollover und Reconstruct aktiver SSTables.
// INVARIANTEN: Atomare Schreiboperationen via Buffer & Sync; Tail-Truncation Recovery bei unvollständigem Frame am EOF.
// HOTSPOTS: load, reconstruct_valid_sstables, append_batch
// FORMATABKÜNDIGUNG (S-01 / T-04): `rank` in `ManifestEntry::Replace` ist abgekündigt. `reconstruct_valid_sstables` nutzt ausschließlich die Manifest-Operationsreihenfolge (`next_rank`). `recovery.rs` verwirft den Wert explizit.

use contextra_ports::erasure::ErasureMode;
use contextra_ports::rng::EntropyRng;
use contextra_types::{ContextraError, Result};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use super::entry::ManifestEntry;
use super::recovery::is_valid_tail_truncation_candidate;
use super::MAX_MANIFEST_ENTRY_SIZE;
use crate::format::{check_openable, FileKind, FormatError, FormatHeader};
use crate::util::DirLock;

pub const MANIFEST_HEADER_MAGIC: &[u8; 4] = b"MFMN";
pub const CURRENT_MANIFEST_VERSION: u8 = 2;

/// Tracks highest recorded format versions per file kind from manifest entries.
#[derive(Debug, Clone, Default)]
pub struct ManifestFormatTracker {
    pub max_sstable_version: u16,
    pub max_blob_version: u16,
    pub max_manifest_version: u16,
    pub max_wal_version: u16,
    pub max_ledger_version: u16,
    pub id_width: Option<u8>,
    pub allowed_sstable_migrated_from: std::collections::HashSet<u16>,
    pub allowed_blob_migrated_from: std::collections::HashSet<u16>,
    pub allowed_manifest_migrated_from: std::collections::HashSet<u16>,
    pub allowed_wal_migrated_from: std::collections::HashSet<u16>,
    pub allowed_ledger_migrated_from: std::collections::HashSet<u16>,
}

impl ManifestFormatTracker {
    pub fn from_entries(entries: &[ManifestEntry]) -> Self {
        let mut tracker = Self::default();
        for entry in entries {
            if let ManifestEntry::FormatVersionRecord {
                file_kind,
                max_version,
                migrated_from,
            } = entry
            {
                match file_kind {
                    0 => {
                        tracker.max_sstable_version = tracker.max_sstable_version.max(*max_version);
                        if *migrated_from > 0 {
                            tracker.allowed_sstable_migrated_from.insert(*migrated_from);
                        }
                    }
                    1 => {
                        tracker.max_blob_version = tracker.max_blob_version.max(*max_version);
                        if *migrated_from > 0 {
                            tracker.allowed_blob_migrated_from.insert(*migrated_from);
                        }
                    }
                    2 => {
                        tracker.max_manifest_version =
                            tracker.max_manifest_version.max(*max_version);
                        if *migrated_from > 0 {
                            tracker
                                .allowed_manifest_migrated_from
                                .insert(*migrated_from);
                        }
                    }
                    3 => {
                        tracker.max_wal_version = tracker.max_wal_version.max(*max_version);
                        if *migrated_from > 0 {
                            tracker.allowed_wal_migrated_from.insert(*migrated_from);
                        }
                    }
                    4 => {
                        tracker.max_ledger_version = tracker.max_ledger_version.max(*max_version);
                        if *migrated_from > 0 {
                            tracker.allowed_ledger_migrated_from.insert(*migrated_from);
                        }
                    }
                    _ => {}
                }
            }
        }
        tracker
    }

    pub fn verify_file_version(
        &self,
        kind: FileKind,
        file_version: u16,
    ) -> std::result::Result<(), FormatError> {
        let (recorded_max, allowed_migrated) = match kind {
            FileKind::Sstable => (
                self.max_sstable_version,
                &self.allowed_sstable_migrated_from,
            ),
            FileKind::Blob => (self.max_blob_version, &self.allowed_blob_migrated_from),
            FileKind::Manifest => (
                self.max_manifest_version,
                &self.allowed_manifest_migrated_from,
            ),
            FileKind::Wal => (self.max_wal_version, &self.allowed_wal_migrated_from),
            FileKind::Ledger => (self.max_ledger_version, &self.allowed_ledger_migrated_from),
        };

        if recorded_max > 0
            && file_version < recorded_max
            && !allowed_migrated.contains(&file_version)
        {
            return Err(FormatError::DowngradeAttempt {
                kind,
                file_version,
                recorded_version: recorded_max,
            });
        }
        Ok(())
    }

    pub fn verify_id_width(&self, expected_width: u8) -> std::result::Result<(), FormatError> {
        if let Some(recorded_width) = self.id_width {
            if recorded_width != expected_width {
                return Err(FormatError::IdWidthMismatch {
                    expected: expected_width,
                    found: recorded_width,
                });
            }
        }
        Ok(())
    }
}

/// Append-only Manifest file handle for recording active SSTable set transitions.
pub struct Manifest {
    pub(super) path: PathBuf,
    pub(super) file: tokio::sync::Mutex<tokio::fs::File>,
    pub(super) erasure_high_water_mark: AtomicU8,
    pub(super) ledger_head: tokio::sync::RwLock<(u64, [u8; 32])>,
    pub(super) backend_id: AtomicU64,
    pub(super) _dir_lock: Option<DirLock>,
}

impl std::fmt::Debug for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Manifest")
            .field("path", &self.path)
            .finish()
    }
}

impl Manifest {
    /// Opens or creates an append-only Manifest file at `path`.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_dir_lock(path, None).await
    }

    /// Opens or creates an append-only Manifest file at `path`, attaching `dir_lock`.
    pub(crate) async fn open_with_dir_lock(
        path: impl AsRef<Path>,
        dir_lock: Option<DirLock>,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let (mut file, is_new) = match tokio::fs::OpenOptions::new()
            .create_new(true)
            .append(true)
            .read(true)
            .open(&path)
            .await
        {
            Ok(file) => (file, true),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let file = tokio::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .read(true)
                    .open(&path)
                    .await
                    .map_err(|e| {
                        ContextraError::Storage(format!("Failed to open MANIFEST: {}", e))
                    })?;
                (file, false)
            }
            Err(e) => {
                return Err(ContextraError::Storage(format!(
                    "Failed to create MANIFEST: {}",
                    e
                )));
            }
        };

        if is_new {
            let mut header = Vec::with_capacity(5);
            header.extend_from_slice(MANIFEST_HEADER_MAGIC);
            header.push(CURRENT_MANIFEST_VERSION);
            file.write_all(&header).await.map_err(|e| {
                ContextraError::Storage(format!("Failed to write MANIFEST header: {}", e))
            })?;
            file.sync_all().await.map_err(|e| {
                ContextraError::Storage(format!(
                    "MANIFEST file fsync failed for {}: {}",
                    path.display(),
                    e
                ))
            })?;
            crate::util::fsync_parent_dir(&path).await?;
        }

        // Load existing entries or parse populated state
        let entries = Self::load(&path).await?;

        let mut max_watermark = 0u8; // ErasureMode::Off
        let mut last_ledger_seq = 0u64;
        let mut last_ledger_mac = [0u8; 32];
        let mut found_backend_id = None;

        for entry in &entries {
            match entry {
                ManifestEntry::ErasureWatermarkRecord { mode } => {
                    max_watermark = max_watermark.max(*mode);
                }
                ManifestEntry::LedgerHeadRecord { seq, mac } => {
                    last_ledger_seq = *seq;
                    last_ledger_mac = *mac;
                }
                ManifestEntry::BackendIdRecord { backend_id } => {
                    found_backend_id = Some(*backend_id);
                }
                _ => {}
            }
        }

        let backend_id = match found_backend_id {
            Some(id) => id,
            None => {
                let rng = contextra_ports::SystemRng::new();
                let mut gen_id = rng.next_u64();
                while gen_id == 0 {
                    gen_id = rng.next_u64();
                }
                gen_id
            }
        };

        let manifest = Self {
            path,
            file: tokio::sync::Mutex::new(file),
            erasure_high_water_mark: AtomicU8::new(max_watermark),
            ledger_head: tokio::sync::RwLock::new((last_ledger_seq, last_ledger_mac)),
            backend_id: AtomicU64::new(backend_id),
            _dir_lock: dir_lock,
        };

        if found_backend_id.is_none() {
            manifest
                .append(&ManifestEntry::BackendIdRecord { backend_id })
                .await?;
        }

        Ok(manifest)
    }

    /// Appends a new entry to the manifest and performs flush + fsync.
    pub async fn append(&self, entry: &ManifestEntry) -> Result<()> {
        self.append_batch(std::slice::from_ref(entry)).await
    }

    /// Appends multiple entries to the manifest in a single atomic batch with flush + fsync.
    pub async fn append_batch(&self, entries: &[ManifestEntry]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }
        let mut batch_bytes = Vec::new();
        for entry in entries {
            batch_bytes.extend(entry.to_bytes()?);
        }

        let mut file = self.file.lock().await;
        file.write_all(&batch_bytes).await.map_err(|e| {
            ContextraError::Storage(format!(
                "MANIFEST write failed for {}: {}",
                self.path.display(),
                e
            ))
        })?;
        file.flush().await.map_err(|e| {
            ContextraError::Storage(format!(
                "MANIFEST flush failed for {}: {}",
                self.path.display(),
                e
            ))
        })?;
        file.sync_all().await.map_err(|e| {
            ContextraError::Storage(format!(
                "MANIFEST fsync failed for {}: {}",
                self.path.display(),
                e
            ))
        })?;
        Ok(())
    }

    /// Loads all valid manifest entries from `path`.
    ///
    /// If the file does not exist, returns an empty vector.
    /// Tail-truncation (an incomplete frame at EOF caused by interrupted write / power cut) is recovered
    /// safely by returning valid entries read up to the point of truncation.
    /// Mid-file corruption or CRC mismatches on complete frames return `Err`.
    pub async fn load(path: &Path) -> Result<Vec<ManifestEntry>> {
        if !tokio::fs::try_exists(path).await.unwrap_or(false) {
            return Ok(Vec::new());
        }

        let mut file = match tokio::fs::File::open(path).await {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(ContextraError::Storage(format!(
                    "Failed to open MANIFEST for reading: {}",
                    e
                )))
            }
        };

        let file_size = file
            .metadata()
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?
            .len();

        let mut reader = tokio::io::BufReader::new(&mut file);
        let mut entries = Vec::new();
        let mut pos = 0u64;

        if file_size >= 5 {
            let mut header = [0u8; 5];
            if reader.read_exact(&mut header).await.is_ok() {
                if &header[0..4] == MANIFEST_HEADER_MAGIC {
                    let version = header[4] as u16;
                    let format_header = FormatHeader::new(*MANIFEST_HEADER_MAGIC, version);
                    check_openable(
                        FileKind::Manifest,
                        &format_header,
                        CURRENT_MANIFEST_VERSION as u16,
                    )?;
                    pos = 5;
                } else {
                    // Legacy version 0 manifest without header magic
                    reader
                        .seek(tokio::io::SeekFrom::Start(0))
                        .await
                        .map_err(|e| {
                            ContextraError::Storage(format!("Failed to seek MANIFEST: {}", e))
                        })?;
                }
            }
        }

        loop {
            if pos == file_size {
                break;
            }

            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes).await {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    tracing::warn!(
                        "MANIFEST tail truncation detected (incomplete length header) at offset {}",
                        pos
                    );
                    break;
                }
                Err(e) => {
                    return Err(ContextraError::Storage(format!(
                        "MANIFEST read error at offset {}: {}",
                        pos, e
                    )));
                }
            }

            let len = u32::from_le_bytes(len_bytes) as usize;
            if len < 5 || len > MAX_MANIFEST_ENTRY_SIZE as usize {
                return Err(ContextraError::Storage(format!(
                    "MANIFEST entry length ({}) invalid or exceeds max size ({}) at offset {}",
                    len, MAX_MANIFEST_ENTRY_SIZE, pos
                )));
            }

            let entry_pos = pos;
            pos += 4 + len as u64;

            if pos <= file_size {
                let mut entry_raw = vec![0u8; len];
                if let Err(e) = reader.read_exact(&mut entry_raw).await {
                    return Err(ContextraError::Storage(format!(
                        "MANIFEST read error at offset {}: {}",
                        entry_pos, e
                    )));
                }

                match ManifestEntry::from_bytes(&entry_raw) {
                    Ok(entry) => entries.push(entry),
                    Err(e) => {
                        return Err(ContextraError::Storage(format!(
                            "MANIFEST entry corruption at offset {}: {}",
                            entry_pos, e
                        )));
                    }
                }
            } else {
                let avail = (file_size - (entry_pos + 4)) as usize;
                let mut partial_raw = vec![0u8; avail];
                if let Err(e) = reader.read_exact(&mut partial_raw).await {
                    return Err(ContextraError::Storage(format!(
                        "MANIFEST read error for partial payload at offset {}: {}",
                        entry_pos, e
                    )));
                }

                if is_valid_tail_truncation_candidate(len, &partial_raw) {
                    tracing::warn!(
                        "MANIFEST tail truncation detected (incomplete payload) at offset {}",
                        entry_pos
                    );
                    break;
                } else {
                    return Err(ContextraError::Storage(format!(
                        "MANIFEST mid-file corruption at offset {}: invalid frame length {} or corrupt frame structure (possible SSTable resurrection risk)",
                        entry_pos, len
                    )));
                }
            }
        }

        Ok(entries)
    }

    /// Reconstructs the ordered list of currently valid SSTable file names (or path components)
    /// and their rank (shadowing order: smaller rank = older) from a sequence of manifest entries.
    pub fn reconstruct_valid_sstables(entries: &[ManifestEntry]) -> Vec<(PathBuf, u64)> {
        let mut valid_map = std::collections::HashMap::new();
        // `next_rank` führt eine reine Monotonie-über-die-Add-Historie und ist NICHT identisch mit dem
        // zur Kompaktierungszeit lokal berechneten `insertion_point` (siehe `lsm/ops/compaction.rs`).
        // Diese Diskrepanz zwischen den Zähldomänen ist bewusst folgenlos, weil `rank` bei der Recovery
        // an keiner Stelle für Sortier- oder Sichtbarkeitsentscheidungen konsultiert wird:
        // `recovery.rs` verwirft den Wert explizit via `.map(|(path, _rank)| path)`. Die Lese-Sichtbarkeit
        // richtet sich ausschließlich nach `SstableMetadata::max_seq`.
        let mut next_rank = 0u64;

        for entry in entries {
            match entry {
                ManifestEntry::Add { path, .. } => {
                    let key = path
                        .file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| path.clone());
                    valid_map.insert(key, next_rank);
                    next_rank += 1;
                }
                ManifestEntry::Remove { path } => {
                    let key = path
                        .file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| path.clone());
                    valid_map.remove(&key);
                }
                ManifestEntry::RollbackComplete { .. }
                | ManifestEntry::WalCheckpoint { .. }
                | ManifestEntry::FormatVersionRecord { .. }
                | ManifestEntry::ErasureWatermarkRecord { .. }
                | ManifestEntry::LedgerHeadRecord { .. }
                | ManifestEntry::BackendIdRecord { .. } => {}
                ManifestEntry::Replace { removed, added, .. } => {
                    for p in removed {
                        let key = p
                            .file_name()
                            .map(PathBuf::from)
                            .unwrap_or_else(|| p.clone());
                        valid_map.remove(&key);
                    }
                    let key = added
                        .file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| added.clone());
                    valid_map.insert(key, next_rank);
                    next_rank += 1;
                }
            }
        }

        let mut sorted: Vec<(PathBuf, u64)> = valid_map.into_iter().collect();
        sorted.sort_by_key(|(_, rank)| *rank);
        sorted
    }

    /// Reconstructs the set of SSTable file names (or path components) that are proven dead
    /// (explicitly removed via `Remove` or `Replace.removed`) and are not in the valid set.
    pub fn reconstruct_dead_sstables(
        entries: &[ManifestEntry],
    ) -> std::collections::HashSet<PathBuf> {
        let valid = Self::reconstruct_valid_sstables(entries);
        let valid_set: std::collections::HashSet<PathBuf> =
            valid.into_iter().map(|(p, _)| p).collect();
        let mut dead_set = std::collections::HashSet::new();

        for entry in entries {
            match entry {
                ManifestEntry::Remove { path } => {
                    let key = path
                        .file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| path.clone());
                    if !valid_set.contains(&key) {
                        dead_set.insert(key);
                    }
                }
                ManifestEntry::Replace { removed, .. } => {
                    for p in removed {
                        let key = p
                            .file_name()
                            .map(PathBuf::from)
                            .unwrap_or_else(|| p.clone());
                        if !valid_set.contains(&key) {
                            dead_set.insert(key);
                        }
                    }
                }
                ManifestEntry::Add { .. }
                | ManifestEntry::RollbackComplete { .. }
                | ManifestEntry::WalCheckpoint { .. }
                | ManifestEntry::FormatVersionRecord { .. }
                | ManifestEntry::ErasureWatermarkRecord { .. }
                | ManifestEntry::LedgerHeadRecord { .. }
                | ManifestEntry::BackendIdRecord { .. } => {}
            }
        }

        dead_set
    }

    /// Extracts the most recent WAL high-water-mark HMAC from a list of manifest entries.
    pub fn extract_high_water_mark(entries: &[ManifestEntry]) -> Option<[u8; 32]> {
        for entry in entries.iter().rev() {
            if let ManifestEntry::WalCheckpoint { hmac } = entry {
                return Some(*hmac);
            }
        }
        None
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Reads the current highest ErasureMode watermark recorded in the manifest.
    pub fn erasure_high_water_mark(&self) -> ErasureMode {
        match self.erasure_high_water_mark.load(Ordering::SeqCst) {
            1 => ErasureMode::Assured,
            2 => ErasureMode::Attested,
            _ => ErasureMode::Off,
        }
    }

    /// Monotonically raises the erasure high water mark in the manifest.
    ///
    /// If `mode` is less than or equal to the currently stored watermark, this call is a no-op and returns `Ok(())`.
    /// The mark is strictly non-decreasing and cannot be lowered.
    pub async fn set_erasure_high_water_mark(&self, mode: ErasureMode) -> Result<()> {
        let mode_u8 = mode as u8;
        let mut current = self.erasure_high_water_mark.load(Ordering::SeqCst);

        while mode_u8 > current {
            match self.erasure_high_water_mark.compare_exchange_weak(
                current,
                mode_u8,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    self.append(&ManifestEntry::ErasureWatermarkRecord { mode: mode_u8 })
                        .await?;
                    break;
                }
                Err(actual) => {
                    current = actual;
                }
            }
        }

        Ok(())
    }

    /// Reads the current ledger head state `(ledger_head_seq, ledger_head_mac)`.
    pub async fn ledger_head(&self) -> (u64, [u8; 32]) {
        *self.ledger_head.read().await
    }

    /// Updates and persists the ledger head state.
    pub async fn set_ledger_head(&self, seq: u64, mac: [u8; 32]) -> Result<()> {
        let mut guard = self.ledger_head.write().await;
        self.append(&ManifestEntry::LedgerHeadRecord { seq, mac })
            .await?;
        *guard = (seq, mac);
        Ok(())
    }

    /// Reads the persistent non-zero backend identifier.
    pub fn backend_id(&self) -> u64 {
        self.backend_id.load(Ordering::SeqCst)
    }
}
