//! # Manifest-Kernlogik (Ring 1)
//!
//! Serialisiert und deserialisiert Manifest-Eintraege fuer Dateiaustausch und Compaction.

// FILE-CONTEXT
// STAND: 2026-09-29 (S-01 / T-04)
// ZWECK: Kernimplementierung des Manifests zum Lesen, Schreiben, Rollover und Reconstruct aktiver SSTables.
// INVARIANTEN: Atomare Schreiboperationen via Buffer & Sync; Tail-Truncation Recovery bei unvollständigem Frame am EOF.
// HOTSPOTS: load, reconstruct_valid_sstables, append_batch
// FORMATABKÜNDIGUNG (S-01 / T-04): `rank` in `ManifestEntry::Replace` ist abgekündigt. `reconstruct_valid_sstables` nutzt ausschließlich die Manifest-Operationsreihenfolge (`next_rank`). `recovery.rs` verwirft den Wert explizit.

use contextra_types::{ContextraError, Result};
use crate::io_seam::append_file::{AppendFile, IoFault};
use contextra_ports::VfsOpenOptions;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::entry::ManifestEntry;
use super::recovery::is_valid_tail_truncation_candidate;
use super::state::ManifestState;
use super::MAX_MANIFEST_ENTRY_SIZE;
use crate::io_seam::StoreIo;
use crate::util::DirLock;

pub const MANIFEST_HEADER_MAGIC: &[u8; 4] = b"MFMN";
pub const CURRENT_MANIFEST_VERSION: u8 = 1;

async fn append_durable(
    file: Arc<parking_lot::Mutex<AppendFile>>,
    bytes: Vec<u8>,
) -> std::result::Result<(), IoFault> {
    tokio::task::spawn_blocking(move || file.lock().append_durable(&bytes))
        .await
        .map_err(|error| IoFault::TaskFailed(error.to_string()))?
}

async fn truncate_durable(
    file: Arc<parking_lot::Mutex<AppendFile>>,
    len: u64,
) -> Result<()> {
    tokio::task::spawn_blocking(move || file.lock().truncate_durable(len))
        .await
        .map_err(|error| ContextraError::Storage(format!("Manifest truncate task failed: {error}")))?
        .map_err(ContextraError::Io)
}

async fn read_append_file(file: Arc<parking_lot::Mutex<AppendFile>>) -> Result<Vec<u8>> {
    tokio::task::spawn_blocking(move || file.lock().read_all())
        .await
        .map_err(|error| ContextraError::Storage(format!("Manifest read task failed: {error}")))?
        .map_err(ContextraError::Io)
}

async fn sync_all_file(file: Arc<parking_lot::Mutex<AppendFile>>) -> Result<()> {
    tokio::task::spawn_blocking(move || file.lock().sync_all())
        .await
        .map_err(|error| ContextraError::Storage(format!("Manifest sync task failed: {error}")))?
        .map_err(|error| ContextraError::Storage(error.to_string()))
}

/// Append-only Manifest file handle for recording active SSTable set transitions.
// ANCHOR[contextra-store:MAN-CORE-001]
/// Verwaltet die VersionSet-Manifestdatei zur dauerhaften Erfassung aller SSTables.
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Manifest {
    pub(super) path: PathBuf,
    pub(super) io: StoreIo,
    pub(super) file: Arc<parking_lot::Mutex<AppendFile>>,
    pub(super) operation_lock: tokio::sync::Mutex<()>,
    pub(super) state: tokio::sync::Mutex<ManifestState>,
    pub(super) _dir_lock: Option<DirLock>,
    pub(super) is_poisoned: AtomicBool,
}

impl std::fmt::Debug for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Manifest")
            .field("path", &self.path)
            .finish()
    }
}

impl Manifest {
    pub(crate) async fn state_snapshot(&self) -> ManifestState {
        self.state.lock().await.clone()
    }

    /// Opens or creates an append-only Manifest file at `path`.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_io(path, None, StoreIo::default()).await
    }

    /// Opens or creates an append-only Manifest file with the selected I/O port.
    pub(crate) async fn open_with_io(
        path: impl AsRef<Path>,
        dir_lock: Option<DirLock>,
        io: StoreIo,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let create_new = VfsOpenOptions {
            read: true,
            write: true,
            create_new: true,
            ..Default::default()
        };
        let (file, is_new) = match io.open_async(&path, create_new).await {
            Ok(file) => (file, true),
            Err(ContextraError::Io(error))
                if error.kind() == std::io::ErrorKind::AlreadyExists =>
            {
                let options = VfsOpenOptions {
                    read: true,
                    write: true,
                    ..Default::default()
                };
                (io.open_async(&path, options).await?, false)
            }
            Err(error) => return Err(error),
        };
        let (file, file_len) = tokio::task::spawn_blocking(move || {
            let len = file.len().map_err(ContextraError::Io)?;
            Ok::<_, ContextraError>((file, len))
        })
        .await
        .map_err(|error| ContextraError::Storage(format!("Manifest metadata task failed: {error}")))??;
        let file = Arc::new(parking_lot::Mutex::new(AppendFile::new(file, file_len)));

        if !is_new && file_len < 5 {
            let prefix = read_append_file(Arc::clone(&file)).await?;
            if !prefix.is_empty()
                && !MANIFEST_HEADER_MAGIC
                    .get(..prefix.len().min(MANIFEST_HEADER_MAGIC.len()))
                    .is_some_and(|expected| expected == prefix.as_slice())
            {
                return Err(ContextraError::Storage(
                    "Unknown or incomplete MANIFEST format header".into(),
                ));
            }
            truncate_durable(Arc::clone(&file), 0).await?;
            let mut header = Vec::with_capacity(5);
            header.extend_from_slice(MANIFEST_HEADER_MAGIC);
            header.push(CURRENT_MANIFEST_VERSION);
            append_durable(Arc::clone(&file), header)
                .await
                .map_err(|error| ContextraError::Storage(error.to_string()))?;
            sync_all_file(Arc::clone(&file)).await?;
        }

        if is_new {
            let mut header = Vec::with_capacity(5);
            header.extend_from_slice(MANIFEST_HEADER_MAGIC);
            header.push(CURRENT_MANIFEST_VERSION);
            append_durable(Arc::clone(&file), header)
                .await
                .map_err(|error| ContextraError::Storage(error.to_string()))?;
            sync_all_file(Arc::clone(&file)).await?;
            if let Some(parent) = path.parent() {
                io.sync_dir(parent).await?;
            }
        }

        let entries = Self::load_with_io(&path, io.clone()).await?;
        let state = ManifestState::from_entries(&entries)?;
        let current_bytes = read_append_file(Arc::clone(&file)).await?;
        let has_header = current_bytes
            .get(..4)
            .is_some_and(|magic| magic == MANIFEST_HEADER_MAGIC.as_slice());
        let mut valid_len = if has_header { 5u64 } else { 0u64 };
        for entry in &entries {
            let encoded_len = u64::try_from(entry.to_bytes()?.len()).map_err(|_| {
                ContextraError::Storage("MANIFEST entry length is not representable".into())
            })?;
            valid_len = valid_len
                .checked_add(encoded_len)
                .ok_or_else(|| ContextraError::Storage("MANIFEST length overflow".into()))?;
        }
        let actual_len = u64::try_from(current_bytes.len()).map_err(|_| {
            ContextraError::Storage("MANIFEST length is not representable".into())
        })?;
        if actual_len > valid_len {
            truncate_durable(Arc::clone(&file), valid_len).await?;
        }
        Ok(Self {
            path,
            io,
            file,
            operation_lock: tokio::sync::Mutex::new(()),
            state: tokio::sync::Mutex::new(state),
            _dir_lock: dir_lock,
            is_poisoned: AtomicBool::new(false),
        })
    }

    /// Checks if the manifest handle has been poisoned by a preceding fsync failure (INV-FSYNC-1).
    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.is_poisoned.load(Ordering::SeqCst)
    }

    /// Appends a new entry to the manifest and performs flush + fsync.
    pub async fn append(&self, entry: &ManifestEntry) -> Result<()> {
        self.append_batch(std::slice::from_ref(entry)).await
    }

    /// Appends multiple entries to the manifest in a single atomic batch with flush + fsync.
    pub async fn append_batch(&self, entries: &[ManifestEntry]) -> Result<()> {
        if self.is_poisoned() {
            return Err(ContextraError::Storage(format!(
                "Manifest handle for {} is poisoned due to a previous fsync failure (INV-FSYNC-1)",
                self.path.display()
            )));
        }

        if entries.is_empty() {
            return Ok(());
        }
        let mut batch_bytes = Vec::new();
        for entry in entries {
            batch_bytes.extend(entry.to_bytes()?);
        }

        let _operation = self.operation_lock.lock().await;
        let mut state = self.state.lock().await;
        let mut next_state = state.clone();
        for entry in entries {
            next_state.apply(entry)?;
        }
        append_durable(Arc::clone(&self.file), batch_bytes)
            .await
            .map_err(|error| {
                if matches!(&error, IoFault::SyncFailed(_) | IoFault::Poisoned) {
                    self.is_poisoned.store(true, Ordering::SeqCst);
                }
                ContextraError::Storage(format!(
                    "MANIFEST append failed for {}: {}",
                    self.path.display(), error
                ))
            })?;
        *state = next_state;
        drop(state);
        let should_rollover = self.file.lock().committed_len()
            >= super::DEFAULT_ROLLOVER_THRESHOLD_BYTES;
        drop(_operation);
        if should_rollover {
            if let Err(error) = self
                .maybe_rollover(&[], super::DEFAULT_ROLLOVER_THRESHOLD_BYTES)
                .await
            {
                if self.is_poisoned() {
                    return Err(error);
                }
                tracing::warn!(error = %error, "Automatic MANIFEST rollover failed; durable append remains committed");
            }
        }
        Ok(())
    }

    /// Loads all valid manifest entries from `path`.
    ///
    /// If the file does not exist, returns an empty vector.
    /// Tail-truncation (an incomplete frame at EOF caused by interrupted write / power cut) is recovered
    /// safely by returning valid entries read up to the point of truncation.
    /// Mid-file corruption or CRC mismatches on complete frames return `Err`.
    pub async fn load(path: &Path) -> Result<Vec<ManifestEntry>> {
        Self::load_with_io(path, StoreIo::default()).await
    }

    pub(crate) async fn load_with_io(path: &Path, io: StoreIo) -> Result<Vec<ManifestEntry>> {
        if !io.exists(path).await? {
            return Ok(Vec::new());
        }
        let data = io.read(path).await?;
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let mut pos = 0usize;
        if data.len() >= 5 && data.get(0..4) == Some(MANIFEST_HEADER_MAGIC.as_slice()) {
            let version = *data
                .get(4)
                .ok_or_else(|| ContextraError::Storage("MANIFEST version missing".into()))?;
            if version > CURRENT_MANIFEST_VERSION {
                return Err(ContextraError::Storage(format!(
                    "Unsupported MANIFEST format version: {}",
                    version
                )));
            }
            pos = 5;
        } else if data.len() < 5 {
            let prefix_matches = MANIFEST_HEADER_MAGIC
                .get(..data.len())
                .is_some_and(|prefix| prefix == data.as_slice());
            if prefix_matches {
                tracing::warn!("Torn MANIFEST header detected; open will initialize it");
                return Ok(Vec::new());
            }
            return Err(ContextraError::Storage("Unknown MANIFEST format".into()));
        }

        let mut entries = Vec::new();
        loop {
            if pos == data.len() {
                break;
            }
            let entry_pos = pos;
            let len_end = pos.checked_add(4).ok_or_else(|| {
                ContextraError::Storage("MANIFEST offset overflow".into())
            })?;
            if len_end > data.len() {
                tracing::warn!("MANIFEST tail truncation detected at offset {entry_pos}");
                break;
            }
            let len_bytes: [u8; 4] = data
                .get(pos..len_end)
                .and_then(|slice| slice.try_into().ok())
                .ok_or_else(|| ContextraError::Storage("MANIFEST length header invalid".into()))?;
            let len = usize::try_from(u32::from_le_bytes(len_bytes)).map_err(|_| {
                ContextraError::Storage("MANIFEST frame length is not representable".into())
            })?;
            if len < 5 || len > MAX_MANIFEST_ENTRY_SIZE as usize {
                return Err(ContextraError::Storage(format!(
                    "MANIFEST entry length ({}) invalid or exceeds max size ({}) at offset {}",
                    len, MAX_MANIFEST_ENTRY_SIZE, entry_pos
                )));
            }
            let frame_end = len_end.checked_add(len).ok_or_else(|| {
                ContextraError::Storage("MANIFEST frame end overflow".into())
            })?;
            if frame_end > data.len() {
                let partial = data.get(len_end..).ok_or_else(|| {
                    ContextraError::Storage("MANIFEST partial frame range invalid".into())
                })?;
                if is_valid_tail_truncation_candidate(len, partial) {
                    tracing::warn!("MANIFEST tail truncation detected at offset {entry_pos}");
                    break;
                }
                return Err(ContextraError::Storage(format!(
                    "MANIFEST mid-file corruption at offset {}: invalid frame length {}",
                    entry_pos, len
                )));
            }
            let entry_raw = data.get(len_end..frame_end).ok_or_else(|| {
                ContextraError::Storage("MANIFEST frame range invalid".into())
            })?;
            pos = frame_end;
            match ManifestEntry::from_bytes(entry_raw) {
                Ok(entry) => entries.push(entry),
                Err(error) if pos == data.len() => {
                    tracing::warn!(
                        "MANIFEST final-frame CRC/decoding error at EOF offset {}; discarding torn tail: {}",
                        entry_pos, error
                    );
                    break;
                }
                Err(error) => {
                    return Err(ContextraError::Storage(format!(
                        "MANIFEST entry corruption at offset {}: {}",
                        entry_pos, error
                    )))
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
                ManifestEntry::RollbackComplete { .. } | ManifestEntry::WalCheckpoint { .. } => {}
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
                | ManifestEntry::WalCheckpoint { .. } => {}
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
}
