// FILE-CONTEXT
// STAND: 2026-10-16
// ZWECK: Anti-Resurrection-Ledger Modul-Einstieg (Format CTRALEDG v1; Spezifikation §6.5; Befund B-10).
// INVARIANTEN: Zero-Panic, Fail-Closed, Lock-Free Index (ArcSwap), Persistent Chaining, Scrubbed Compaction.

pub mod coverage;
pub mod entry;
pub mod replay_filter;

use arc_swap::ArcSwap;
use contextra_ports::Clock;
use contextra_types::{ContextraError, Result};
pub use coverage::{LedgerCoverage, LedgerGap};
pub use entry::{
    compute_keyed_digest, compute_keyed_hash, parse_legacy_eldg_frame, parse_record,
    CTRALRecord, CTRALSegmentHeader, FrameReadResult, LedgerKey, RecordType, CTRALEDG_MAGIC,
    CTRALEDG_VERSION,
};
pub use replay_filter::{ErasureLedgerHandle, LedgerReplayFilter};

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

pub const LEDGER_FILENAME: &str = "erasure.ledger";
pub const HEAD_FILENAME: &str = "ledger.head";
pub const INITIAL_CHAIN_MAC: [u8; 32] = [0u8; 32];
pub const MAX_SEGMENT_SIZE: u64 = 64 * 1024 * 1024; // 64 MiB rotation threshold

/// Decision returned when evaluating whether a key version at transaction `tx` should be kept or dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerDecision {
    Keep,
    Drop,
}

/// Status returned when opening the erasure ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerOpenStatus {
    Fresh,
    Ok,
    RollbackSuspected { reason: String },
    Corrupt { reason: String },
}

/// Legacy / compatibility record representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErasureRecord {
    pub storage_scope: Vec<u8>,
    pub key_prefix: Vec<u8>,
    pub erased_at_tx: u64,
}

impl ErasureRecord {
    pub fn for_prefix(storage_scope: Vec<u8>, key_prefix: Vec<u8>, erased_at_tx: u64) -> Self {
        Self {
            storage_scope,
            key_prefix,
            erased_at_tx,
        }
    }
}

/// Legacy / compatibility entry struct for existing tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    pub ledger_seq: u64,
    pub kind: LedgerEntryKind,
    pub keyed_hash: [u8; 32],
    pub erased_at_tx: u64,
    pub proof_id: [u8; 16],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerEntryKind {
    Key,
    Prefix { prefix_len: u32 },
}

impl LedgerEntry {
    pub fn to_payload_bytes(&self) -> Result<Vec<u8>> {
        let mut payload = Vec::with_capacity(70);
        payload.extend_from_slice(&self.ledger_seq.to_be_bytes());
        match self.kind {
            LedgerEntryKind::Key => {
                payload.push(1u8);
                payload.extend_from_slice(&self.keyed_hash);
            }
            LedgerEntryKind::Prefix { prefix_len } => {
                payload.push(2u8);
                payload.extend_from_slice(&self.keyed_hash);
                payload.extend_from_slice(&prefix_len.to_be_bytes());
            }
        }
        payload.extend_from_slice(&self.erased_at_tx.to_be_bytes());
        payload.extend_from_slice(&self.proof_id);
        Ok(payload)
    }
}

/// Lock-free in-memory lookup index for `RestoreGuard` and read-path queries.
#[derive(Debug, Default, Clone)]
pub struct ErasureLedgerIndex {
    /// Maps `(scope_digest, family_digest)` -> `max_erased_at_tx`
    pub scopes: HashMap<([u8; 32], [u8; 32]), u64>,
    /// Distinct prefix lengths present in the ledger
    pub distinct_prefix_lens: HashSet<u32>,
}

impl ErasureLedgerIndex {
    pub fn is_in_deleted_scope(&self, key: &LedgerKey, storage_scope: &[u8], raw_key: &[u8], tx: u64) -> bool {
        let scope_digest = compute_keyed_digest(key, storage_scope);

        for prefix_len in 0..=raw_key.len() {
            let prefix_bytes = &raw_key[..prefix_len];
            let prefix_digest = compute_keyed_digest(key, prefix_bytes);
            if let Some(&erased_at_tx) = self.scopes.get(&(scope_digest, prefix_digest)) {
                if tx <= erased_at_tx {
                    return true;
                }
            }
        }

        false
    }
}

/// In-memory representation and file handle of the Anti-Resurrection ErasureLedger (CTRALEDG v1).
pub struct ErasureLedger {
    dir: PathBuf,
    key: LedgerKey,
    last_chain_mac: [u8; 32],
    segment_seq: u64,
    prev_segment_hash: [u8; 32],
    records: Vec<CTRALRecord>,
    seen_purge_ids: HashSet<[u8; 16]>,
    db_instance_id: [u8; 16],
    index: ArcSwap<ErasureLedgerIndex>,
}

impl std::fmt::Debug for ErasureLedger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErasureLedger")
            .field("dir", &self.dir)
            .field("segment_seq", &self.segment_seq)
            .field("last_chain_mac", &self.last_chain_mac)
            .field("records_count", &self.records.len())
            .finish()
    }
}

impl ErasureLedger {
    /// Memory-only constructor for isolated testing.
    pub fn new_in_memory() -> Self {
        Self {
            dir: PathBuf::new(),
            key: LedgerKey::new([0u8; 32]),
            last_chain_mac: INITIAL_CHAIN_MAC,
            segment_seq: 0,
            prev_segment_hash: [0u8; 32],
            records: Vec::new(),
            seen_purge_ids: HashSet::new(),
            db_instance_id: [0u8; 16],
            index: ArcSwap::new(Arc::new(ErasureLedgerIndex::default())),
        }
    }

    /// Records an erasure directly in memory (for testing and in-memory test harnesses).
    pub fn record_erasure(&mut self, record: ErasureRecord) -> Result<()> {
        let scope_digest = compute_keyed_digest(&self.key, &record.storage_scope);
        let family_digest = compute_keyed_digest(&self.key, &record.key_prefix);

        let mut next_index = (**self.index.load()).clone();
        next_index.scopes.insert((scope_digest, family_digest), record.erased_at_tx);
        if !record.key_prefix.is_empty() {
            #[allow(clippy::cast_possible_truncation)]
            next_index.distinct_prefix_lens.insert(record.key_prefix.len() as u32);
        }
        self.index.store(Arc::new(next_index));

        let ctral_record = CTRALRecord {
            record_type: RecordType::Purged,
            tx: record.erased_at_tx,
            timestamp_ms: 0,
            purge_id: [0u8; 16],
            scope_digest,
            family_digests: vec![family_digest],
            payload: Vec::new(),
            mac: INITIAL_CHAIN_MAC,
        };
        self.records.push(ctral_record);
        Ok(())
    }

    /// Resolves the active segment path based on segment_seq.
    fn active_segment_path(&self) -> PathBuf {
        if self.segment_seq == 0 {
            self.dir.join(LEDGER_FILENAME)
        } else {
            self.dir.join(format!("erasure_{:08}.ledger", self.segment_seq))
        }
    }

    /// Primary Assured mode constructor (Spec §6.5).
    /// Returns `Ok((ledger, status))` on success or fail-closed `LedgerOpenStatus`.
    pub async fn open_assured(
        dir: impl AsRef<Path>,
        key: LedgerKey,
        db_has_data: bool,
        expected_head_mac: Option<[u8; 32]>,
    ) -> Result<(Option<Self>, LedgerOpenStatus)> {
        Self::open_assured(dir, key, db_has_data, expected_head_mac).await
    }

    async fn open_internal(
        dir: impl AsRef<Path>,
        key: LedgerKey,
        db_has_data: bool,
        expected_head_mac: Option<[u8; 32]>,
    ) -> Result<(Option<Self>, LedgerOpenStatus)> {
        let dir = dir.as_ref().to_path_buf();
        let legacy_ledger_path = dir.join(LEDGER_FILENAME);
        let head_path = dir.join(HEAD_FILENAME);

        let legacy_exists = tokio::fs::try_exists(&legacy_ledger_path).await.unwrap_or(false);
        let head_exists = tokio::fs::try_exists(&head_path).await.unwrap_or(false);

        let mut segment_files = Self::list_segment_files(&dir).await?;

        if !legacy_exists && segment_files.is_empty() {
            if db_has_data {
                return Ok((
                    None,
                    LedgerOpenStatus::RollbackSuspected {
                        reason: "Database contains data but erasure ledger is missing".into(),
                    },
                ));
            }

            // Fresh database: create new ledger file and head marker
            let mut db_instance_id = [0u8; 16];
            let rng = contextra_ports::SystemRng::new();
            contextra_ports::Rng::fill_bytes(&rng, &mut db_instance_id);

            let header = CTRALSegmentHeader::new(0, [0u8; 32]);
            let header_bytes = header.to_bytes();

            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&ledger_path)
                .await
                .map_err(|e| ContextraError::Storage(format!("Failed to create erasure.ledger: {}", e)))?;

            file.write_all(&header_bytes).await.map_err(|e| {
                ContextraError::Storage(format!("Failed to write header to erasure.ledger: {}", e))
            })?;
            file.sync_all().await.map_err(|e| {
                ContextraError::Storage(format!("Failed to sync erasure.ledger header: {}", e))
            })?;
            crate::util::fsync_parent_dir(&ledger_path).await?;

            let ledger = Self {
                dir: dir.clone(),
                key,
                last_chain_mac: INITIAL_CHAIN_MAC,
                segment_seq: 0,
                prev_segment_hash: [0u8; 32],
                records: Vec::new(),
                seen_purge_ids: HashSet::new(),
                db_instance_id,
                index: ArcSwap::new(Arc::new(ErasureLedgerIndex::default())),
            };

            ledger.init_segment_0().await?;
            ledger.write_head_marker().await?;

            return Ok((Some(ledger), LedgerOpenStatus::Fresh));
        }

        // Discover all segment files in order
        let mut segment_paths = Vec::new();
        segment_paths.push(ledger_path.clone());

        if let Ok(mut read_dir) = tokio::fs::read_dir(&dir).await {
            let mut rotated_segments = Vec::new();
            while let Ok(Some(entry)) = read_dir.next_entry().await {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with("erasure_") && file_name.ends_with(".ledger") {
                        let seq_str = &file_name[8..file_name.len() - 7];
                        if let Ok(seq) = seq_str.parse::<u64>() {
                            rotated_segments.push((seq, path));
                        }
                    }
                }
            }
            rotated_segments.sort_by_key(|(seq, _)| *seq);
            for (_, path) in rotated_segments {
                segment_paths.push(path);
            }
        }

        let mut prev_mac = INITIAL_CHAIN_MAC;
        let mut records = Vec::new();
        let mut seen_purge_ids = HashSet::new();
        let mut index = ErasureLedgerIndex::default();
        let mut last_segment_seq = 0u64;
        let mut last_prev_hash = [0u8; 32];

        for path in &segment_paths {
            let mut file_bytes = tokio::fs::read(path).await.map_err(|e| {
                ContextraError::Storage(format!("Failed to read ledger segment {:?}: {}", path, e))
            })?;

            // Check for Legacy ELDG format migration on primary segment
            if path == &ledger_path && file_bytes.len() >= 4 && (file_bytes.len() < 8 || &file_bytes[0..8] != CTRALEDG_MAGIC) {
                tracing::info!("Migrating legacy ELDG ledger to CTRALEDG v1 format");
                match Self::migrate_legacy_eldg(&dir, &ledger_path, &file_bytes, &key).await {
                    Ok(migrated_bytes) => {
                        file_bytes = migrated_bytes;
                    }
                    Err(e) => {
                        return Ok((
                            None,
                            LedgerOpenStatus::Corrupt {
                                reason: format!("Legacy ELDG migration failed: {}", e),
                            },
                        ));
                    }
                }
            }

            if file_bytes.len() < entry::CTRALEDG_HEADER_SIZE {
                return Ok((
                    None,
                    LedgerOpenStatus::Corrupt {
                        reason: format!("Segment {:?} shorter than CTRALEDG header size", path),
                    },
                ));
            }

            let segment_header = match CTRALSegmentHeader::parse(&file_bytes[..entry::CTRALEDG_HEADER_SIZE]) {
                Ok(h) => h,
                Err(e) => {
                    return Ok((
                        None,
                        LedgerOpenStatus::Corrupt {
                            reason: format!("Segment header parse failed for {:?}: {}", path, e),
                        },
                    ));
                }
            };

            last_segment_seq = segment_header.segment_seq;
            last_prev_hash = segment_header.prev_segment_hash;

            let mut offset = entry::CTRALEDG_HEADER_SIZE;
            let mut torn_tail_detected = false;
            let mut verified_len = entry::CTRALEDG_HEADER_SIZE;

            while offset < file_bytes.len() {
                let res = parse_record(&key, &prev_mac, &file_bytes[offset..]);
                match res {
                    FrameReadResult::Ok {
                        record,
                        mac,
                        frame_len,
                    } => {
                        if !seen_purge_ids.contains(&record.purge_id) || record.purge_id == [0u8; 16] {
                            if record.purge_id != [0u8; 16] {
                                seen_purge_ids.insert(record.purge_id);
                            }
                            for fd in &record.family_digests {
                                index.scopes.insert((record.scope_digest, *fd), record.tx);
                            }
                            records.push(record);
                        }
                        prev_mac = mac;
                        offset += frame_len;
                        verified_len = offset;
                    }
                    FrameReadResult::TruncatedTail { bytes_read: _ } => {
                        tracing::warn!(
                            "ErasureLedger tail truncation detected at offset {} in {:?}",
                            verified_len, path
                        );
                        torn_tail_detected = true;
                        break;
                    }
                    FrameReadResult::Corrupt { reason } => {
                        return Ok((
                            None,
                            LedgerOpenStatus::Corrupt {
                                reason: format!("Corruption in {:?} at offset {}: {}", path, offset, reason),
                            },
                        ));
                    }
                }
            }

            // Handle torn tail truncation at end of file
            if torn_tail_detected && verified_len < file_bytes.len() {
                let file = OpenOptions::new()
                    .write(true)
                    .open(path)
                    .await
                    .map_err(|e| ContextraError::Storage(e.to_string()))?;
                file.set_len(verified_len as u64)
                    .await
                    .map_err(|e| ContextraError::Storage(e.to_string()))?;
                file.sync_all()
                    .await
                    .map_err(|e| ContextraError::Storage(e.to_string()))?;
                crate::util::fsync_parent_dir(path).await?;
            }
        }

        // Check head marker consistency
        let (stored_db_id, stored_head_seq, stored_head_mac) = if head_exists {
            Self::read_head_marker(&head_path).await?
        } else {
            ([0u8; 16], last_segment_seq, INITIAL_CHAIN_MAC)
        };

        if head_exists && (stored_head_mac != prev_mac || stored_head_seq != last_segment_seq) {
            return Ok((
                None,
                LedgerOpenStatus::RollbackSuspected {
                    reason: "Head marker MAC or sequence does not match ledger state".into(),
                },
            ));
        }

        if let Some(exp_mac) = expected_head_mac {
            if exp_mac != prev_mac {
                return Ok((
                    None,
                    LedgerOpenStatus::RollbackSuspected {
                        reason: "Expected head MAC mismatch".into(),
                    },
                ));
            }
        }

        let db_instance_id = if head_exists {
            stored_db_id
        } else {
            let mut id = [0u8; 16];
            let rng = contextra_ports::SystemRng::new();
            contextra_ports::Rng::fill_bytes(&rng, &mut id);
            id
        };

        let index = Arc::new(ArcSwap::from_pointee(index_map));

        let ledger = Self {
            dir,
            key,
            last_chain_mac: prev_mac,
            segment_seq: last_segment_seq,
            prev_segment_hash: last_prev_hash,
            records,
            seen_purge_ids,
            db_instance_id,
            index: ArcSwap::new(Arc::new(index)),
        };

        if !head_exists {
            ledger.write_head_marker().await?;
        }

        Ok((Some(ledger), LedgerOpenStatus::Ok))
    }

    /// Convenience wrapper delegating to `open_assured` for recovery callers.
    pub async fn open_or_fail_closed(
        dir: impl AsRef<Path>,
        key: LedgerKey,
        db_has_data: bool,
        expected_head_mac: Option<[u8; 32]>,
    ) -> Result<(Option<Self>, LedgerOpenStatus)> {
        Self::open_assured(dir, key, db_has_data, expected_head_mac).await
    }

    /// Atomically migrates legacy `ELDG` format file to `CTRALEDG` v1 format.
    async fn migrate_legacy_eldg(
        _dir: &Path,
        ledger_path: &Path,
        file_bytes: &[u8],
        key: &LedgerKey,
    ) -> Result<Vec<u8>> {
        let tmp_path = ledger_path.with_extension("tmp");
        let mut offset = 0;
        let mut prev_mac = INITIAL_CHAIN_MAC;

        let header = CTRALSegmentHeader::new(0, [0u8; 32]);
        let mut new_bytes = Vec::new();
        new_bytes.extend_from_slice(&header.to_bytes());

        while offset < file_bytes.len() {
            if let Some((payload, frame_len)) = parse_legacy_eldg_frame(&file_bytes[offset..]) {
                // Convert ELDG payload into CTRALRecord Intent
                let entry_kind_byte = if payload.len() > 8 { payload[8] } else { 1 };
                let (scope, family_raw) = match entry_kind_byte {
                    1 => (b"lsm".as_slice(), payload.as_slice()),
                    _ => (b"lsm".as_slice(), payload.as_slice()),
                };

                let scope_digest = compute_keyed_digest(key, scope);
                let family_digest = compute_keyed_digest(key, family_raw);

                let rec = CTRALRecord {
                    record_type: RecordType::Intent,
                    tx: 1,
                    timestamp_ms: 0,
                    purge_id: [0u8; 16],
                    scope_digest,
                    family_digests: vec![family_digest],
                    payload: family_raw.to_vec(),
                    mac: INITIAL_CHAIN_MAC,
                };

                let (frame_bytes, new_mac) = rec.serialize(key, &prev_mac)?;
                new_bytes.extend_from_slice(&frame_bytes);
                prev_mac = new_mac;
                offset += frame_len;
            } else {
                break;
            }
        }

        tokio::fs::write(&tmp_path, &new_bytes)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed writing migrated ledger: {}", e)))?;

        let file = OpenOptions::new()
            .write(true)
            .open(&tmp_path)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        file.sync_all().await.map_err(|e| ContextraError::Storage(e.to_string()))?;

        tokio::fs::rename(&tmp_path, ledger_path)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed renaming migrated ledger: {}", e)))?;

        crate::util::fsync_parent_dir(ledger_path).await?;
        Ok(new_bytes)
    }

    /// Appends an `Intent` record (0x01) containing plaintext scope/prefixes in payload.
    pub async fn append_intent(
        &mut self,
        tx: u64,
        clock: &dyn Clock,
        purge_id: [u8; 16],
        scope: &[u8],
        families: &[&[u8]],
        payload: &[u8],
    ) -> Result<[u8; 32]> {
        let scope_digest = compute_keyed_digest(&self.key, scope);
        let family_digests = families
            .iter()
            .map(|f| compute_keyed_digest(&self.key, f))
            .collect();

        let timestamp_ms = clock.now_unix_nanos() / 1_000_000;

        let record = CTRALRecord {
            record_type: RecordType::Intent,
            tx,
            timestamp_ms,
            purge_id,
            scope_digest,
            family_digests,
            payload: payload.to_vec(),
            mac: INITIAL_CHAIN_MAC,
        };

        self.append_records(&[record]).await
    }

    /// Appends a `Purged` record (0x02) containing ONLY digests (payload MUST be empty).
    pub async fn append_purged(
        &mut self,
        tx: u64,
        clock: &dyn Clock,
        purge_id: [u8; 16],
        scope: &[u8],
        families: &[&[u8]],
    ) -> Result<[u8; 32]> {
        let scope_digest = compute_keyed_digest(&self.key, scope);
        let family_digests = families
            .iter()
            .map(|f| compute_keyed_digest(&self.key, f))
            .collect();

        let timestamp_ms = clock.now_unix_nanos() / 1_000_000;

        let record = CTRALRecord {
            record_type: RecordType::Purged,
            tx,
            timestamp_ms,
            purge_id,
            scope_digest,
            family_digests,
            payload: Vec::new(),
            mac: INITIAL_CHAIN_MAC,
        };

        self.append_records(&[record]).await
    }

    /// Appends records with segment rotation at 64 MiB and single atomic fsync.
    pub async fn append_records(&mut self, records: &[CTRALRecord]) -> Result<[u8; 32]> {
        if records.is_empty() {
            return Ok(self.last_chain_mac);
        }

        let mut active_path = self.active_segment_path();

        // Check if segment rotation at 64 MiB is needed
        if let Ok(meta) = tokio::fs::metadata(&active_path).await {
            if meta.len() >= MAX_SEGMENT_SIZE {
                self.rotate_segment().await?;
                active_path = self.active_segment_path();
            }
        }

        let active_seg_path = self.get_current_segment_path();
        let mut buffer = Vec::new();
        let mut cur_mac = self.last_chain_mac;

        let mut new_records = Vec::new();
        let mut new_entries = Vec::new();

        for (rt, tx, purge_id, scope_digest, family_digests, payload) in records_data {
            if self.seen_proof_ids.contains(purge_id) {
                continue;
            }

            let (frame_bytes, new_mac) = CtraledgRecord::serialize_frame(
                &self.key,
                &cur_mac,
                *rt,
                *tx,
                now_ms,
                *purge_id,
                *scope_digest,
                family_digests.clone(),
                payload.clone(),
            )?;

            buffer.extend_from_slice(&frame_bytes);
            cur_mac = new_mac;

            let rec = CtraledgRecord {
                record_type: *rt,
                tx: *tx,
                timestamp: now_ms,
                purge_id: *purge_id,
                scope_digest: *scope_digest,
                family_digests: family_digests.clone(),
                payload: payload.clone(),
                mac: new_mac,
                crc32c: crc32fast::hash(&frame_bytes[4..frame_bytes.len() - 4]),
            };
            new_records.push(rec);

            if !payload.is_empty() {
                if let Ok(entry) = LedgerEntry::from_payload_bytes(payload) {
                    new_entries.push(entry);
                }
            } else {
                new_entries.push(LedgerEntry {
                    ledger_seq: self.last_seq + new_records.len() as u64,
                    kind: LedgerEntryKind::Key,
                    keyed_hash: *scope_digest,
                    erased_at_tx: *tx,
                    proof_id: *purge_id,
                });
            }
        }

        if buffer.is_empty() {
            return Ok(());
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&active_seg_path)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to open segment: {}", e)))?;

        file.write_all(&buffer).await.map_err(|e| ContextraError::Storage(e.to_string()))?;
        file.sync_all().await.map_err(|e| ContextraError::Storage(e.to_string()))?;
        crate::util::fsync_parent_dir(&self.dir).await?;

        // Update state
        self.last_chain_mac = cur_mac;
        self.last_seq += new_records.len() as u64;
        self.current_segment_bytes += buffer.len() as u64;

        for rec in new_records {
            self.seen_proof_ids.insert(rec.purge_id);
            self.records.push(rec);
        }
        for entry in new_entries {
            if let LedgerEntryKind::Prefix { prefix_len } = entry.kind {
                self.distinct_prefix_lens.insert(prefix_len);
            }
            self.entries.push(entry);
        }

        self.rebuild_index();
        self.write_head_marker().await?;

        Ok(())
    }

    /// Compatibility append for legacy `LedgerEntry` callers in tests.
    pub async fn append(&mut self, entries: &[LedgerEntry]) -> Result<()> {
        let clock = contextra_ports::SystemClock::new();
        for entry in entries {
            let scope = b"lsm";
            let family_digest = entry.keyed_hash;

            let record = CTRALRecord {
                record_type: RecordType::Intent,
                tx: entry.erased_at_tx,
                timestamp_ms: clock.now_unix_nanos() / 1_000_000,
                purge_id: entry.proof_id,
                scope_digest: compute_keyed_digest(&self.key, scope),
                family_digests: vec![family_digest],
                payload: entry.to_payload_bytes()?,
                mac: INITIAL_CHAIN_MAC,
            };

            self.append_records(&[record]).await?;
        }
        Ok(())
    }

    /// Performs segment rotation when active segment reaches 64 MiB.
    async fn rotate_segment(&mut self) -> Result<()> {
        self.segment_seq += 1;
        self.prev_segment_hash = self.last_chain_mac;

        let rotated_path = self.active_segment_path();

        let header = CTRALSegmentHeader::new(self.segment_seq, self.prev_segment_hash);
        let header_bytes = header.to_bytes();

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&rotated_path)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to create rotated segment: {}", e)))?;

        file.write_all(&header_bytes).await.map_err(|e| {
            ContextraError::Storage(format!("Failed writing header to rotated segment: {}", e))
        })?;
        file.sync_all().await.map_err(|e| {
            ContextraError::Storage(format!("Failed syncing rotated segment: {}", e))
        })?;

        crate::util::fsync_parent_dir(&rotated_path).await?;
        Ok(())
    }

    /// Evaluates whether a key version at transaction `tx` should be kept or dropped.
    pub fn decide(&self, storage_scope: &[u8], key: &[u8], tx: u64) -> LedgerDecision {
        if self.index.load().is_in_deleted_scope(&self.key, storage_scope, key, tx) {
            LedgerDecision::Drop
        } else {
            LedgerDecision::Keep
        }
    }

    pub fn decide_batch(
        &self,
        storage_scope: &[u8],
        items: &[(&[u8], u64)],
    ) -> Vec<LedgerDecision> {
        items
            .iter()
            .map(|(key, tx)| self.decide(storage_scope, key, *tx))
            .collect()
    }

    /// Lock-free query check for `RestoreGuard`.
    pub fn is_in_deleted_scope(&self, storage_scope: &[u8], key: &[u8], tx: u64) -> bool {
        self.index.load().is_in_deleted_scope(&self.key, storage_scope, key, tx)
    }

    /// Compacts `Intent` records into `Purged` records, writing new file atomically and scrubbing old segment.
    pub async fn compact_intents_to_purged(&mut self) -> Result<()> {
        let active_path = self.active_segment_path();
        let tmp_path = self.dir.join("erasure_compacted.tmp");

        let header = CTRALSegmentHeader::new(self.segment_seq, self.prev_segment_hash);
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&header.to_bytes());

        let mut current_mac = INITIAL_CHAIN_MAC;
        let mut compacted_records = Vec::new();

        for rec in &self.records {
            let mut compacted = rec.clone();
            if compacted.record_type == RecordType::Intent {
                compacted.record_type = RecordType::Purged;
                compacted.payload.clear(); // Plaintext payload stripped
            }

            let (frame_bytes, new_mac) = compacted.serialize(&self.key, &current_mac)?;
            buffer.extend_from_slice(&frame_bytes);
            current_mac = new_mac;
            compacted_records.push(compacted);
        }

        tokio::fs::write(&tmp_path, &buffer).await.map_err(|e| {
            ContextraError::Storage(format!("Failed writing compacted ledger tmp file: {}", e))
        })?;

        let file = OpenOptions::new()
            .write(true)
            .open(&tmp_path)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;
        file.sync_all().await.map_err(|e| ContextraError::Storage(e.to_string()))?;

        // Scrub old segment before atomic replacement if possible
        let _ = contextra_durable_fs::scrub_if_safe(
            &active_path,
            contextra_durable_fs::ScrubPolicy::OverwriteZeros,
            false,
            false,
        );

        tokio::fs::rename(&tmp_path, &active_path).await.map_err(|e| {
            ContextraError::Storage(format!("Failed replacing ledger with compacted version: {}", e))
        })?;

        crate::util::fsync_parent_dir(&active_path).await?;

        self.last_chain_mac = current_mac;
        self.records = compacted_records;
        self.write_head_marker().await?;

        Ok(())
    }

    pub fn last_chain_mac(&self) -> &[u8; 32] {
        &self.last_chain_mac
    }

    pub fn segment_seq(&self) -> u64 {
        self.segment_seq
    }

    pub fn db_instance_id(&self) -> &[u8; 16] {
        &self.db_instance_id
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn key(&self) -> &LedgerKey {
        &self.key
    }

    pub fn seen_purge_ids(&self) -> &HashSet<[u8; 16]> {
        &self.seen_purge_ids
    }

    pub fn records(&self) -> &[CTRALRecord] {
        &self.records
    }

    pub fn entries(&self) -> Vec<LedgerEntry> {
        self.records
            .iter()
            .enumerate()
            .map(|(idx, r)| {
                let kind = if r.payload.len() >= 45 && r.payload[8] == 2 {
                    let prefix_len = u32::from_be_bytes(r.payload[41..45].try_into().unwrap_or([0; 4]));
                    LedgerEntryKind::Prefix { prefix_len }
                } else {
                    LedgerEntryKind::Key
                };
                LedgerEntry {
                    #[allow(clippy::cast_possible_truncation)]
                    ledger_seq: (idx + 1) as u64,
                    kind,
                    keyed_hash: r.family_digests.first().copied().unwrap_or([0u8; 32]),
                    erased_at_tx: r.tx,
                    proof_id: r.purge_id,
                }
            })
            .collect()
    }

    pub fn update_after_append(&mut self, new_mac: [u8; 32], to_write: Vec<CTRALRecord>) {
        let mut next_index = (**self.index.load()).clone();
        for rec in &to_write {
            if rec.purge_id != [0u8; 16] {
                self.seen_purge_ids.insert(rec.purge_id);
            }
            for fd in &rec.family_digests {
                next_index.scopes.insert((rec.scope_digest, *fd), rec.tx);
            }
            self.records.push(rec.clone());
        }
        self.last_chain_mac = new_mac;
        self.index.store(Arc::new(next_index));
    }

    async fn write_head_marker(&self) -> Result<()> {
        let head_path = self.dir.join(HEAD_FILENAME);
        let mut marker_bytes = Vec::with_capacity(64);
        marker_bytes.extend_from_slice(CTRALEDG_MAGIC); // Magic
        marker_bytes.extend_from_slice(&self.db_instance_id);
        marker_bytes.extend_from_slice(&self.segment_seq.to_be_bytes());
        marker_bytes.extend_from_slice(&self.last_chain_mac);
        marker_bytes.extend_from_slice(&self.last_seq.to_le_bytes());

        contextra_durable_fs::atomic_replace(&head_path, &marker_bytes)?;
        Ok(())
    }

    async fn read_head_marker(path: &Path) -> Result<([u8; 16], u64, [u8; 32])> {
        let bytes = tokio::fs::read(path)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to read head marker: {}", e)))?;

        if bytes.len() != 8 + 16 + 8 + 32 || &bytes[0..8] != CTRALEDG_MAGIC {
            return Err(ContextraError::Storage("Corrupt head marker".into()));
        }

        let mut db_id = [0u8; 16];
        db_id.copy_from_slice(&bytes[8..24]);

        let seq = u64::from_be_bytes(
            bytes[24..32]
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid head seq".into()))?,
        );

        let mut head_mac = [0u8; 32];
        head_mac.copy_from_slice(&bytes[32..64]);

        Ok((db_id, seq, head_mac))
    }
}
