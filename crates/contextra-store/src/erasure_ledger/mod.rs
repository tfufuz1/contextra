// FILE-CONTEXT
// STAND: 2026-10-16
// ZWECK: ErasureLedger Modul-Einstieg und CTRALEDG v1 Append-Only Persistenz, Decision Filter & In-Memory Index.
// INVARIANTEN: Zero-Panic, Fail-Closed, INV-DURABILITY-RING (fsync_parent_dir + sync_all).

pub mod coverage;
pub mod entry;
pub mod replay_filter;

use arc_swap::ArcSwap;
use contextra_types::{ContextraError, Result};
pub use coverage::{LedgerCoverage, LedgerGap};
pub use replay_filter::{ErasureLedgerHandle, LedgerReplayFilter};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

pub use entry::{
    compute_digest, compute_keyed_hash, parse_ctraledg_frame, parse_frame, serialize_frame,
    CtraledgFrameResult, CtraledgRecord, CtraledgSegmentHeader, FrameReadResult, LedgerEntry,
    LedgerEntryKind, LedgerKey, RecordType, CTRALEDG_MAGIC, CTRALEDG_VERSION, ERASED_LEDGER_DOMAIN,
};

pub const LEDGER_FILENAME: &str = "erasure.ledger";
pub const HEAD_FILENAME: &str = "ledger.head";
pub const INITIAL_CHAIN_MAC: [u8; 32] = [0u8; 32];
pub const MAX_SEGMENT_SIZE: u64 = 64 * 1024 * 1024; // 64 MiB segment rotation threshold

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

/// In-memory index of deleted scopes and key families for fast RestoreGuard queries.
#[derive(Debug, Clone, Default)]
pub struct LedgerIndexMap {
    pub scope_digests: HashSet<[u8; 32]>,
    pub family_digests: HashSet<[u8; 32]>,
    pub erased_tx_by_scope: HashMap<[u8; 32], u64>,
}

impl LedgerIndexMap {
    pub fn is_erased(&self, scope_digest: &[u8; 32], family_digests: &[[u8; 32]], tx: u64) -> bool {
        if let Some(&erased_tx) = self.erased_tx_by_scope.get(scope_digest) {
            if tx <= erased_tx {
                return true;
            }
        }
        for fd in family_digests {
            if self.family_digests.contains(fd) {
                return true;
            }
        }
        false
    }
}

/// In-memory representation and file handle of the ErasureLedger in CTRALEDG v1 format.
pub struct ErasureLedger {
    dir: PathBuf,
    key: LedgerKey,
    last_chain_mac: [u8; 32],
    last_seq: u64,
    entries: Vec<LedgerEntry>,
    records: Vec<CtraledgRecord>,
    seen_proof_ids: HashSet<[u8; 16]>,
    distinct_prefix_lens: HashSet<u32>,
    db_instance_id: [u8; 16],
    index: Arc<ArcSwap<LedgerIndexMap>>,
    current_segment_num: u64,
    current_segment_bytes: u64,
}

impl std::fmt::Debug for ErasureLedger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErasureLedger")
            .field("dir", &self.dir)
            .field("last_chain_mac", &self.last_chain_mac)
            .field("last_seq", &self.last_seq)
            .field("entries_count", &self.entries.len())
            .field("records_count", &self.records.len())
            .finish()
    }
}

impl ErasureLedger {
    /// Primary entry point for opening the erasure ledger in Assured mode.
    /// Returns Ok((Some(ledger), status)) or fail-closed status.
    pub async fn open_assured(
        dir: impl AsRef<Path>,
        key: LedgerKey,
        db_has_data: bool,
        expected_head_mac: Option<[u8; 32]>,
    ) -> Result<(Option<Self>, LedgerOpenStatus)> {
        Self::open_internal(dir, key, db_has_data, expected_head_mac).await
    }

    /// Alias for backwards compatibility.
    pub async fn open_or_fail_closed(
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

            // Fresh database: create new ledger
            let mut db_instance_id = [0u8; 16];
            let rng = contextra_ports::SystemRng::new();
            contextra_ports::Rng::fill_bytes(&rng, &mut db_instance_id);

            let index = Arc::new(ArcSwap::from_pointee(LedgerIndexMap::default()));

            let ledger = Self {
                dir: dir.clone(),
                key,
                last_chain_mac: INITIAL_CHAIN_MAC,
                last_seq: 0,
                entries: Vec::new(),
                records: Vec::new(),
                seen_proof_ids: HashSet::new(),
                distinct_prefix_lens: HashSet::new(),
                db_instance_id,
                index,
                current_segment_num: 0,
                current_segment_bytes: CtraledgSegmentHeader::SIZE as u64,
            };

            ledger.init_segment_0().await?;
            ledger.write_head_marker().await?;

            return Ok((Some(ledger), LedgerOpenStatus::Fresh));
        }

        // Check if legacy ELDG file exists and needs migration
        if legacy_exists && segment_files.is_empty() {
            let bytes = tokio::fs::read(&legacy_ledger_path).await.map_err(|e| {
                ContextraError::Storage(format!("Failed to read legacy ledger: {}", e))
            })?;

            let is_ctraledg = bytes.len() >= 8 && &bytes[0..8] == CTRALEDG_MAGIC;
            if !is_ctraledg {
                // Execute migration from legacy ELDG format to CTRALEDG v1 format
                tracing::info!("Migrating legacy ELDG ledger to CTRALEDG v1 format");
                if let Err(e) = Self::migrate_legacy_eldg(&dir, &key, &bytes).await {
                    return Ok((
                        None,
                        LedgerOpenStatus::Corrupt {
                            reason: format!("Migration of legacy ELDG failed: {}", e),
                        },
                    ));
                }
                segment_files = Self::list_segment_files(&dir).await?;
            }
        }

        // Read and parse CTRALEDG segment files
        let mut prev_mac = INITIAL_CHAIN_MAC;
        let mut last_seq = 0u64;
        let mut entries = Vec::new();
        let mut records = Vec::new();
        let mut seen_proof_ids = HashSet::new();
        let mut distinct_prefix_lens = HashSet::new();
        let mut index_map = LedgerIndexMap::default();
        let mut current_segment_num = 0u64;
        let mut current_segment_bytes = 0u64;

        let files_to_process = if !segment_files.is_empty() {
            segment_files
        } else if legacy_exists {
            vec![legacy_ledger_path]
        } else {
            Vec::new()
        };

        for seg_path in files_to_process {
            let file_bytes = tokio::fs::read(&seg_path).await.map_err(|e| {
                ContextraError::Storage(format!("Failed to read segment {}: {}", seg_path.display(), e))
            })?;

            if file_bytes.len() < CtraledgSegmentHeader::SIZE {
                return Ok((
                    None,
                    LedgerOpenStatus::Corrupt {
                        reason: format!("Segment header too short in {}", seg_path.display()),
                    },
                ));
            }

            let hdr = match CtraledgSegmentHeader::parse(&file_bytes[0..CtraledgSegmentHeader::SIZE]) {
                Ok(h) => h,
                Err(e) => {
                    return Ok((
                        None,
                        LedgerOpenStatus::Corrupt {
                            reason: format!("Corrupt CTRALEDG header in {}: {}", seg_path.display(), e),
                        },
                    ));
                }
            };

            current_segment_num = hdr.segment_num;
            let mut offset = CtraledgSegmentHeader::SIZE;
            let mut torn_tail = false;
            let mut verified_len = offset;

            while offset < file_bytes.len() {
                match parse_ctraledg_frame(&key, &prev_mac, &file_bytes[offset..]) {
                    CtraledgFrameResult::Ok {
                        record,
                        chain_mac,
                        frame_len,
                    } => {
                        prev_mac = chain_mac;
                        last_seq += 1;
                        offset += frame_len;
                        verified_len = offset;

                        seen_proof_ids.insert(record.purge_id);
                        index_map.scope_digests.insert(record.scope_digest);
                        index_map.erased_tx_by_scope.insert(record.scope_digest, record.tx);
                        for fd in &record.family_digests {
                            index_map.family_digests.insert(*fd);
                        }

                        if !record.payload.is_empty() {
                            if let Ok(entry) = LedgerEntry::from_payload_bytes(&record.payload) {
                                if let LedgerEntryKind::Prefix { prefix_len } = entry.kind {
                                    distinct_prefix_lens.insert(prefix_len);
                                }
                                entries.push(entry);
                            }
                        } else {
                            entries.push(LedgerEntry {
                                ledger_seq: last_seq,
                                kind: LedgerEntryKind::Key,
                                keyed_hash: record.scope_digest,
                                erased_at_tx: record.tx,
                                proof_id: record.purge_id,
                            });
                        }

                        records.push(record);
                    }
                    CtraledgFrameResult::TruncatedTail { .. } => {
                        torn_tail = true;
                        break;
                    }
                    CtraledgFrameResult::Corrupt { reason } => {
                        return Ok((
                            None,
                            LedgerOpenStatus::Corrupt {
                                reason: format!("Corruption in {}: {}", seg_path.display(), reason),
                            },
                        ));
                    }
                }
            }

            if torn_tail && verified_len < file_bytes.len() {
                tracing::warn!("Truncating torn write tail in segment {}", seg_path.display());
                let file = OpenOptions::new()
                    .write(true)
                    .open(&seg_path)
                    .await
                    .map_err(|e| ContextraError::Storage(e.to_string()))?;
                file.set_len(verified_len as u64)
                    .await
                    .map_err(|e| ContextraError::Storage(e.to_string()))?;
                file.sync_all()
                    .await
                    .map_err(|e| ContextraError::Storage(e.to_string()))?;
                crate::util::fsync_parent_dir(&seg_path).await?;
                current_segment_bytes = verified_len as u64;
            } else {
                current_segment_bytes = file_bytes.len() as u64;
            }
        }

        // Check head marker consistency
        let (stored_db_id, stored_head_mac, stored_head_seq) = if head_exists {
            Self::read_head_marker(&head_path).await?
        } else {
            ([0u8; 16], INITIAL_CHAIN_MAC, 0)
        };

        if head_exists && (stored_head_mac != prev_mac || stored_head_seq != last_seq) {
            return Ok((
                None,
                LedgerOpenStatus::RollbackSuspected {
                    reason: "Head marker MAC or sequence does not match ledger chain state".into(),
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
            last_seq,
            entries,
            records,
            seen_proof_ids,
            distinct_prefix_lens,
            db_instance_id,
            index,
            current_segment_num,
            current_segment_bytes,
        };

        if !head_exists {
            ledger.write_head_marker().await?;
        }

        Ok((Some(ledger), LedgerOpenStatus::Ok))
    }

    async fn migrate_legacy_eldg(dir: &Path, key: &LedgerKey, legacy_bytes: &[u8]) -> Result<()> {
        let mut offset = 0;
        let mut prev_mac = INITIAL_CHAIN_MAC;
        let mut records = Vec::new();

        while offset < legacy_bytes.len() {
            let res = parse_frame(key, &prev_mac, &legacy_bytes[offset..]);
            match res {
                FrameReadResult::Ok {
                    entry,
                    chain_mac,
                    frame_len,
                } => {
                    let payload = entry.to_payload_bytes()?;
                    records.push((entry, payload));
                    prev_mac = chain_mac;
                    offset += frame_len;
                }
                FrameReadResult::TruncatedTail { .. } => break,
                FrameReadResult::Corrupt { reason } => {
                    return Err(ContextraError::Storage(format!(
                        "Legacy ELDG frame corrupt during migration: {}",
                        reason
                    )));
                }
            }
        }

        let tmp_path = dir.join("migration_tmp.ctraledg");
        let target_seg_path = dir.join("segment_0000000000000000.ctraledg");

        let hdr = CtraledgSegmentHeader::new(0, [0u8; 32]);
        let mut buffer = hdr.to_bytes().to_vec();
        let mut cur_mac = INITIAL_CHAIN_MAC;

        let clock = contextra_ports::SystemClock::new();
        let now_ms = (contextra_ports::Clock::now_unix_nanos(&clock) / 1_000_000) as u64;

        for (entry, payload) in records {
            let (frame, new_mac) = CtraledgRecord::serialize_frame(
                key,
                &cur_mac,
                RecordType::Intent,
                entry.erased_at_tx,
                now_ms,
                entry.proof_id,
                entry.keyed_hash,
                Vec::new(),
                payload,
            )?;
            buffer.extend_from_slice(&frame);
            cur_mac = new_mac;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to create migration temp file: {}", e)))?;

        file.write_all(&buffer).await.map_err(|e| ContextraError::Storage(e.to_string()))?;
        file.sync_all().await.map_err(|e| ContextraError::Storage(e.to_string()))?;

        tokio::fs::rename(&tmp_path, &target_seg_path).await.map_err(|e| {
            ContextraError::Storage(format!("Failed to rename migrated ledger segment: {}", e))
        })?;

        let legacy_file = dir.join(LEDGER_FILENAME);
        if tokio::fs::try_exists(&legacy_file).await.unwrap_or(false) {
            let _ = tokio::fs::remove_file(&legacy_file).await;
        }

        crate::util::fsync_parent_dir(dir).await?;

        Ok(())
    }

    async fn list_segment_files(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut list = Vec::new();
        if !tokio::fs::try_exists(dir).await.unwrap_or(false) {
            return Ok(list);
        }

        let mut entries = tokio::fs::read_dir(dir).await.map_err(|e| {
            ContextraError::Storage(format!("Failed to read dir {}: {}", dir.display(), e))
        })?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext == "ctraledg" && !path.file_name().and_then(|s| s.to_str()).unwrap_or("").contains("tmp") {
                    list.push(path);
                }
            }
        }

        list.sort();
        Ok(list)
    }

    async fn init_segment_0(&self) -> Result<()> {
        let seg_path = self.dir.join("segment_0000000000000000.ctraledg");
        let hdr = CtraledgSegmentHeader::new(0, [0u8; 32]);
        let bytes = hdr.to_bytes();

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&seg_path)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;

        file.write_all(&bytes).await.map_err(|e| ContextraError::Storage(e.to_string()))?;
        file.sync_all().await.map_err(|e| ContextraError::Storage(e.to_string()))?;
        crate::util::fsync_parent_dir(&self.dir).await?;
        Ok(())
    }

    /// Appends an Intent record (containing plaintext prefix payload if applicable).
    pub async fn append_intent(
        &mut self,
        tx: u64,
        purge_id: [u8; 16],
        scope_digest: [u8; 32],
        family_digests: Vec<[u8; 32]>,
        payload: Vec<u8>,
    ) -> Result<()> {
        self.append_record_internal(
            RecordType::Intent,
            tx,
            purge_id,
            scope_digest,
            family_digests,
            payload,
        )
        .await
    }

    /// Appends a Purged record (digests only, zero/empty payload).
    pub async fn append_purged(
        &mut self,
        tx: u64,
        purge_id: [u8; 16],
        scope_digest: [u8; 32],
        family_digests: Vec<[u8; 32]>,
    ) -> Result<()> {
        self.append_record_internal(
            RecordType::Purged,
            tx,
            purge_id,
            scope_digest,
            family_digests,
            Vec::new(),
        )
        .await
    }

    /// Group append multiple records.
    pub async fn append_batch(&mut self, records_data: &[(RecordType, u64, [u8; 16], [u8; 32], Vec<[u8; 32]>, Vec<u8>)]) -> Result<()> {
        if records_data.is_empty() {
            return Ok(());
        }

        let clock = contextra_ports::SystemClock::new();
        let now_ms = (contextra_ports::Clock::now_unix_nanos(&clock) / 1_000_000) as u64;

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

        // Rotate segment if adding this buffer would exceed 64 MiB
        if self.current_segment_bytes > CtraledgSegmentHeader::SIZE as u64
            && self.current_segment_bytes + buffer.len() as u64 > MAX_SEGMENT_SIZE
        {
            let prev_seg_path = self.get_current_segment_path();
            let prev_bytes = tokio::fs::read(&prev_seg_path)
                .await
                .map_err(|e| ContextraError::Storage(format!("Failed to read previous segment for rotation: {}", e)))?;
            let prev_hash = blake3::hash(&prev_bytes).into();

            self.current_segment_num += 1;
            let new_seg_path = self.get_current_segment_path();
            let hdr = CtraledgSegmentHeader::new(self.current_segment_num, prev_hash);
            let hdr_bytes = hdr.to_bytes();

            let mut new_file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&new_seg_path)
                .await
                .map_err(|e| ContextraError::Storage(format!("Failed to create new segment file: {}", e)))?;

            new_file.write_all(&hdr_bytes).await.map_err(|e| ContextraError::Storage(e.to_string()))?;
            new_file.sync_all().await.map_err(|e| ContextraError::Storage(e.to_string()))?;

            let mut app_file = OpenOptions::new()
                .append(true)
                .open(&new_seg_path)
                .await
                .map_err(|e| ContextraError::Storage(format!("Failed to open new segment: {}", e)))?;

            app_file.write_all(&buffer).await.map_err(|e| ContextraError::Storage(e.to_string()))?;
            app_file.sync_all().await.map_err(|e| ContextraError::Storage(e.to_string()))?;
            crate::util::fsync_parent_dir(&self.dir).await?;

            self.current_segment_bytes = CtraledgSegmentHeader::SIZE as u64 + buffer.len() as u64;
        } else {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&active_seg_path)
                .await
                .map_err(|e| ContextraError::Storage(format!("Failed to open segment: {}", e)))?;

            file.write_all(&buffer).await.map_err(|e| ContextraError::Storage(e.to_string()))?;
            file.sync_all().await.map_err(|e| ContextraError::Storage(e.to_string()))?;
            crate::util::fsync_parent_dir(&self.dir).await?;

            self.current_segment_bytes += buffer.len() as u64;
        }

        // Update state
        self.last_chain_mac = cur_mac;
        self.last_seq += new_records.len() as u64;

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

    /// Legacy append method.
    pub async fn append(&mut self, entries: &[LedgerEntry]) -> Result<()> {
        let mut batch = Vec::new();
        for entry in entries {
            let payload = entry.to_payload_bytes()?;
            batch.push((
                RecordType::Intent,
                entry.erased_at_tx,
                entry.proof_id,
                entry.keyed_hash,
                Vec::new(),
                payload,
            ));
        }
        self.append_batch(&batch).await
    }

    async fn append_record_internal(
        &mut self,
        record_type: RecordType,
        tx: u64,
        purge_id: [u8; 16],
        scope_digest: [u8; 32],
        family_digests: Vec<[u8; 32]>,
        payload: Vec<u8>,
    ) -> Result<()> {
        self.append_batch(&[(record_type, tx, purge_id, scope_digest, family_digests, payload)]).await
    }

    fn get_current_segment_path(&self) -> PathBuf {
        let filename = format!("segment_{:016x}.ctraledg", self.current_segment_num);
        self.dir.join(filename)
    }

    fn rebuild_index(&self) {
        let mut map = LedgerIndexMap::default();
        for rec in &self.records {
            map.scope_digests.insert(rec.scope_digest);
            map.erased_tx_by_scope.insert(rec.scope_digest, rec.tx);
            for fd in &rec.family_digests {
                map.family_digests.insert(*fd);
            }
        }
        self.index.store(Arc::new(map));
    }

    /// Decides whether a key version at transaction `tx` should be kept or dropped.
    pub fn decide(&self, storage_scope: &[u8], key: &[u8], tx: u64) -> LedgerDecision {
        let key_hash = compute_keyed_hash(&self.key, storage_scope, key);

        let mut prefix_hashes = std::collections::HashMap::new();
        for &len in &self.distinct_prefix_lens {
            let usize_len = len as usize;
            if key.len() >= usize_len {
                let prefix_bytes = &key[..usize_len];
                let prefix_hash = compute_keyed_hash(&self.key, storage_scope, prefix_bytes);
                prefix_hashes.insert(len, prefix_hash);
            }
        }

        for entry in &self.entries {
            match entry.kind {
                LedgerEntryKind::Key => {
                    if entry.keyed_hash == key_hash && tx <= entry.erased_at_tx {
                        return LedgerDecision::Drop;
                    }
                }
                LedgerEntryKind::Prefix { prefix_len } => {
                    if let Some(prefix_hash) = prefix_hashes.get(&prefix_len) {
                        if entry.keyed_hash == *prefix_hash && tx <= entry.erased_at_tx {
                            return LedgerDecision::Drop;
                        }
                    }
                }
            }
        }

        LedgerDecision::Keep
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

    pub fn read_head_info(&self) -> (u64, [u8; 32]) {
        (self.last_seq, self.last_chain_mac)
    }

    pub fn index_handle(&self) -> Arc<ArcSwap<LedgerIndexMap>> {
        Arc::clone(&self.index)
    }

    /// Standalone compaction: converts finished Intent records into Purged records (clearing payload) and atomically replaces segment file.
    pub async fn compact_segments(&mut self) -> Result<usize> {
        let mut compacted_count = 0;
        let mut new_records = Vec::new();

        for rec in &self.records {
            if rec.record_type == RecordType::Intent {
                let mut purged_rec = rec.clone();
                purged_rec.record_type = RecordType::Purged;
                purged_rec.payload.clear();
                new_records.push(purged_rec);
                compacted_count += 1;
            } else {
                new_records.push(rec.clone());
            }
        }

        if compacted_count == 0 {
            return Ok(0);
        }

        let seg_path = self.get_current_segment_path();
        let tmp_path = self.dir.join("compaction_tmp.ctraledg");

        let hdr = CtraledgSegmentHeader::new(self.current_segment_num, [0u8; 32]);
        let mut buffer = hdr.to_bytes().to_vec();
        let mut cur_mac = INITIAL_CHAIN_MAC;

        for rec in &mut new_records {
            let (frame, new_mac) = CtraledgRecord::serialize_frame(
                &self.key,
                &cur_mac,
                rec.record_type,
                rec.tx,
                rec.timestamp,
                rec.purge_id,
                rec.scope_digest,
                rec.family_digests.clone(),
                rec.payload.clone(),
            )?;
            buffer.extend_from_slice(&frame);
            rec.mac = new_mac;
            cur_mac = new_mac;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)
            .await
            .map_err(|e| ContextraError::Storage(e.to_string()))?;

        file.write_all(&buffer).await.map_err(|e| ContextraError::Storage(e.to_string()))?;
        file.sync_all().await.map_err(|e| ContextraError::Storage(e.to_string()))?;

        tokio::fs::rename(&tmp_path, &seg_path).await.map_err(|e| ContextraError::Storage(e.to_string()))?;
        crate::util::fsync_parent_dir(&self.dir).await?;

        self.last_chain_mac = cur_mac;
        self.records = new_records;
        self.rebuild_index();
        self.write_head_marker().await?;

        Ok(compacted_count)
    }

    pub fn last_chain_mac(&self) -> &[u8; 32] {
        &self.last_chain_mac
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

    pub fn seen_proof_ids(&self) -> &HashSet<[u8; 16]> {
        &self.seen_proof_ids
    }

    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }

    async fn write_head_marker(&self) -> Result<()> {
        let head_path = self.dir.join(HEAD_FILENAME);
        let mut marker_bytes = Vec::with_capacity(60);
        marker_bytes.extend_from_slice(b"CTRA"); // Magic
        marker_bytes.extend_from_slice(&self.db_instance_id);
        marker_bytes.extend_from_slice(&self.last_chain_mac);
        marker_bytes.extend_from_slice(&self.last_seq.to_le_bytes());

        contextra_durable_fs::atomic_replace(&head_path, &marker_bytes)?;
        Ok(())
    }

    async fn read_head_marker(path: &Path) -> Result<([u8; 16], [u8; 32], u64)> {
        let bytes = tokio::fs::read(path)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to read head marker: {}", e)))?;

        if bytes.len() < 52 || (&bytes[0..4] != b"ELDG" && &bytes[0..4] != b"CTRA") {
            return Err(ContextraError::Storage("Corrupt head marker".into()));
        }

        let mut db_id = [0u8; 16];
        db_id.copy_from_slice(&bytes[4..20]);

        let mut head_mac = [0u8; 32];
        head_mac.copy_from_slice(&bytes[20..52]);

        let seq = if bytes.len() >= 60 {
            u64::from_le_bytes(bytes[52..60].try_into().unwrap_or([0; 8]))
        } else {
            0
        };

        Ok((db_id, head_mac, seq))
    }
}
