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
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use super::entry::ManifestEntry;
use super::recovery::is_valid_tail_truncation_candidate;
use super::MAX_MANIFEST_ENTRY_SIZE;
use crate::util::DirLock;

pub const MANIFEST_HEADER_MAGIC: &[u8; 4] = b"MFMN";
pub const CURRENT_MANIFEST_VERSION: u8 = 1;

/// Append-only Manifest file handle for recording active SSTable set transitions.
// ANCHOR[contextra-store:MAN-CORE-001]
/// Verwaltet die VersionSet-Manifestdatei zur dauerhaften Erfassung aller SSTables.
pub struct Manifest {
    pub(super) path: PathBuf,
    pub(super) file: tokio::sync::Mutex<tokio::fs::File>,
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

        Ok(Self {
            path,
            file: tokio::sync::Mutex::new(file),
            _dir_lock: dir_lock,
        })
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
                    let version = header[4];
                    if version > CURRENT_MANIFEST_VERSION {
                        return Err(ContextraError::Storage(format!(
                            "Unsupported MANIFEST format version: {}",
                            version
                        )));
                    }
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
                let avail =
                    usize::try_from(file_size.saturating_sub(entry_pos + 4)).unwrap_or(usize::MAX);
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
        let mut valid_map = std::collections::BTreeMap::new();
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
                | ManifestEntry::WalFloor { .. }
                | ManifestEntry::TextCommit { .. } => {}
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
                ManifestEntry::SstSwap { add, remove } => {
                    for p in remove {
                        let key = p
                            .file_name()
                            .map(PathBuf::from)
                            .unwrap_or_else(|| p.clone());
                        valid_map.remove(&key);
                    }
                    for p in add {
                        let key = p
                            .file_name()
                            .map(PathBuf::from)
                            .unwrap_or_else(|| p.clone());
                        valid_map.insert(key, next_rank);
                        next_rank += 1;
                    }
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
    ) -> std::collections::BTreeSet<PathBuf> {
        let valid = Self::reconstruct_valid_sstables(entries);
        let valid_set: std::collections::BTreeSet<PathBuf> =
            valid.into_iter().map(|(p, _)| p).collect();
        let mut dead_set = std::collections::BTreeSet::new();

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
                ManifestEntry::SstSwap { remove, .. } => {
                    for p in remove {
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
                | ManifestEntry::WalFloor { .. }
                | ManifestEntry::TextCommit { .. } => {}
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

use contextra_ports::keys::FileId;

/// Strukturierter WAL-Floor für Recovery und Verkettungsprüfung (F.4, ST-04).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalFloor {
    pub seq: u64,
    pub tail_mac: [u8; 32],
    pub removed_file_ids: Vec<FileId>,
}

/// Strukturierte Text-Commit-Informationen (F.4, D-121).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextCommitInfo {
    pub add: Option<FileId>,
    pub remove: Vec<FileId>,
    pub floor_seq: u64,
}

impl Manifest {
    /// Hängt einen WalFloor-Eintrag an und erzwingt Durability via flush + fsync (F.4).
    pub async fn append_wal_floor(
        &self,
        seq: u64,
        tail_mac: [u8; 32],
        removed_file_ids: Vec<FileId>,
    ) -> Result<()> {
        let entry = ManifestEntry::WalFloor {
            seq,
            tail_mac,
            removed_file_ids,
        };
        self.append(&entry).await
    }

    /// Liefert den aktuellsten (letzten) WalFloor aus den Manifest-Einträgen.
    pub fn current_wal_floor(entries: &[ManifestEntry]) -> Option<WalFloor> {
        for entry in entries.iter().rev() {
            if let ManifestEntry::WalFloor {
                seq,
                tail_mac,
                removed_file_ids,
            } = entry
            {
                return Some(WalFloor {
                    seq: *seq,
                    tail_mac: *tail_mac,
                    removed_file_ids: removed_file_ids.clone(),
                });
            }
        }
        None
    }

    /// Hängt einen TextCommit-Eintrag an und führt flush + fsync aus (F.4 / D-121).
    pub async fn append_text_commit(
        &self,
        add: Option<FileId>,
        remove: Vec<FileId>,
        floor_seq: u64,
    ) -> Result<()> {
        let entry = ManifestEntry::TextCommit {
            add,
            remove,
            floor_seq,
        };
        self.append(&entry).await
    }

    /// Liefert den aktuellsten (letzten) TextCommit aus den Manifest-Einträgen.
    pub fn current_text_commit(entries: &[ManifestEntry]) -> Option<TextCommitInfo> {
        for entry in entries.iter().rev() {
            if let ManifestEntry::TextCommit {
                add,
                remove,
                floor_seq,
            } = entry
            {
                return Some(TextCommitInfo {
                    add: *add,
                    remove: remove.clone(),
                    floor_seq: *floor_seq,
                });
            }
        }
        None
    }

    /// Extrahiert alle TextCommit-Einträge in logischer Reihenfolge.
    pub fn extract_text_commits(entries: &[ManifestEntry]) -> Vec<TextCommitInfo> {
        let mut list = Vec::new();
        for entry in entries {
            if let ManifestEntry::TextCommit {
                add,
                remove,
                floor_seq,
            } = entry
            {
                list.push(TextCommitInfo {
                    add: *add,
                    remove: remove.clone(),
                    floor_seq: *floor_seq,
                });
            }
        }
        list
    }

    /// Hängt einen atomaren SSTable-Tausch (SstSwap) an (ADR SL-1 ausstehend).
    pub async fn append_sst_swap(&self, add: &[PathBuf], remove: &[PathBuf]) -> Result<()> {
        let entry = ManifestEntry::SstSwap {
            add: add.to_vec(),
            remove: remove.to_vec(),
        };
        self.append(&entry).await
    }
}

/// Knoten eines augmentierten Intervall-Suchbaums zur O(log n + k) Kandidatenermittlung (SPEC 5.5).
#[derive(Debug, Clone)]
struct IntervalNode {
    path: PathBuf,
    lo: Vec<u8>,
    hi: Vec<u8>,
    max_hi: Vec<u8>,
    left: Option<Box<IntervalNode>>,
    right: Option<Box<IntervalNode>>,
}

impl IntervalNode {
    fn new(path: PathBuf, lo: Vec<u8>, hi: Vec<u8>) -> Self {
        let max_hi = hi.clone();
        Self {
            path,
            lo,
            hi,
            max_hi,
            left: None,
            right: None,
        }
    }

    fn insert(&mut self, path: PathBuf, lo: Vec<u8>, hi: Vec<u8>) {
        if hi > self.max_hi {
            self.max_hi = hi.clone();
        }

        if lo < self.lo {
            match self.left {
                Some(ref mut l) => l.insert(path, lo, hi),
                None => self.left = Some(Box::new(IntervalNode::new(path, lo, hi))),
            }
        } else {
            match self.right {
                Some(ref mut r) => r.insert(path, lo, hi),
                None => self.right = Some(Box::new(IntervalNode::new(path, lo, hi))),
            }
        }
    }

    fn query(&self, query_lo: &[u8], query_hi: &[u8], results: &mut Vec<PathBuf>) {
        if self.max_hi.as_slice() < query_lo {
            return;
        }

        if let Some(ref l) = self.left {
            if l.max_hi.as_slice() >= query_lo {
                l.query(query_lo, query_hi, results);
            }
        }

        if self.lo.as_slice() <= query_hi && self.hi.as_slice() >= query_lo {
            results.push(self.path.clone());
        }

        if query_hi >= self.lo.as_slice() {
            if let Some(ref r) = self.right {
                r.query(query_lo, query_hi, results);
            }
        }
    }
}

/// Schnellprüfungs-Index für Manifest-SSTable-Intervalle in O(log n + k) (SPEC 5.5).
#[derive(Debug, Clone, Default)]
pub struct ManifestIntervalIndex {
    root: Option<Box<IntervalNode>>,
}

impl ManifestIntervalIndex {
    /// Erzeugt einen neuen leeren Intervallindex.
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Fügt ein Dateipfad-Intervall [lo, hi] in den Index ein.
    pub fn insert(&mut self, path: impl Into<PathBuf>, lo: impl AsRef<[u8]>, hi: impl AsRef<[u8]>) {
        let p = path.into();
        let lo_vec = lo.as_ref().to_vec();
        let hi_vec = hi.as_ref().to_vec();
        match self.root {
            Some(ref mut r) => r.insert(p, lo_vec, hi_vec),
            None => self.root = Some(Box::new(IntervalNode::new(p, lo_vec, hi_vec))),
        }
    }

    /// Ermittelt alle Kandidatendateien, deren Intervall [lo, hi] den Bereich [query_lo, query_hi] schneidet.
    ///
    /// Komplexität: O(log n + k), wobei k die Anzahl der schneidenden Treffer ist.
    pub fn candidates_intersecting(
        &self,
        query_lo: impl AsRef<[u8]>,
        query_hi: impl AsRef<[u8]>,
    ) -> Vec<PathBuf> {
        let mut results = Vec::new();
        if let Some(ref r) = self.root {
            r.query(query_lo.as_ref(), query_hi.as_ref(), &mut results);
        }
        results
    }
}
