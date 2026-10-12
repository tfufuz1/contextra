// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Durable simulation engine using FaultVfs with WAL record chaining and recovery.
// INVARIANTEN: Zero corrupted records readable after crash recovery; torn writes truncated.

use crate::fault_vfs::FaultVfs;
use std::collections::{BTreeMap, HashSet};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

/// Simple FNV-1a 32-bit checksum for deterministic record verification.
fn fnv1a_32(data: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for &byte in data {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

/// Simple 64-bit hash combiner for record chaining.
fn combine_hash(prev: u64, current_checksum: u32, seq: u64) -> u64 {
    let mut h = prev.wrapping_add(seq).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    h ^= u64::from(current_checksum).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    h
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StagedOp {
    Put(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
}

/// Durable simulation engine backed by `FaultVfs`.
#[derive(Debug, Clone)]
pub struct DstEngine {
    vfs: FaultVfs,
    path: PathBuf,
    active_data: BTreeMap<Vec<u8>, Vec<u8>>,
    shredded_keys: HashSet<Vec<u8>>,
    staged: Vec<StagedOp>,
    seq_no: u64,
    last_chain_hash: u64,
    raw_log_bytes: Vec<u8>,
}

impl DstEngine {
    /// Creates a new, empty `DstEngine` attached to a `FaultVfs` path.
    pub fn new(vfs: FaultVfs, path: PathBuf) -> Self {
        Self {
            vfs,
            path,
            active_data: BTreeMap::new(),
            shredded_keys: HashSet::new(),
            staged: Vec::new(),
            seq_no: 0,
            last_chain_hash: 0,
            raw_log_bytes: Vec::new(),
        }
    }

    /// Gets active value for `key`, returning `None` if deleted, missing, or shredded.
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        if self.shredded_keys.contains(key) {
            return None;
        }
        self.active_data.get(key).cloned()
    }

    /// Returns `true` if `key` was permanently shredded/forgotten.
    pub fn is_forgotten(&self, key: &[u8]) -> bool {
        self.shredded_keys.contains(key)
    }

    /// Stages a `Put` operation.
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.staged.push(StagedOp::Put(key, value));
    }

    /// Stages a `Delete` operation.
    pub fn delete(&mut self, key: Vec<u8>) {
        self.staged.push(StagedOp::Delete(key));
    }

    /// Permanently shreds/forgets a key and commits the shredding operation immediately.
    pub fn forget(&mut self, key: Vec<u8>) -> io::Result<()> {
        self.active_data.remove(&key);
        self.shredded_keys.insert(key.clone());

        self.seq_no += 1;
        let record = Self::encode_record(3, self.seq_no, self.last_chain_hash, &key, &[]);
        let checksum = fnv1a_32(&record[..record.len() - 4]);
        self.last_chain_hash = combine_hash(self.last_chain_hash, checksum, self.seq_no);

        let mut new_log = self.raw_log_bytes.clone();
        new_log.extend_from_slice(&record);

        self.vfs.write_file(&self.path, &new_log)?;
        self.vfs.sync()?;
        self.raw_log_bytes = new_log;
        Ok(())
    }

    /// Clears any uncommitted staged operations.
    pub fn rollback(&mut self) {
        self.staged.clear();
    }

    /// Commits all staged operations to disk via `FaultVfs` and syncs.
    pub fn commit(&mut self) -> io::Result<u64> {
        if self.staged.is_empty() {
            return Ok(self.seq_no);
        }

        let new_seq = self.seq_no + 1;
        let mut append_bytes = Vec::new();
        let mut current_hash = self.last_chain_hash;

        for op in &self.staged {
            let (tag, k, v) = match op {
                StagedOp::Put(k, v) => (1u8, k.as_slice(), v.as_slice()),
                StagedOp::Delete(k) => (2u8, k.as_slice(), &[][..]),
            };
            let record = Self::encode_record(tag, new_seq, current_hash, k, v);
            let checksum = fnv1a_32(&record[..record.len() - 4]);
            current_hash = combine_hash(current_hash, checksum, new_seq);
            append_bytes.extend_from_slice(&record);
        }

        // Commit record
        let commit_rec = Self::encode_record(4, new_seq, current_hash, &[], &[]);
        let commit_checksum = fnv1a_32(&commit_rec[..commit_rec.len() - 4]);
        current_hash = combine_hash(current_hash, commit_checksum, new_seq);
        append_bytes.extend_from_slice(&commit_rec);

        let mut new_log = self.raw_log_bytes.clone();
        new_log.extend_from_slice(&append_bytes);

        // Attempt write & sync
        self.vfs.write_file(&self.path, &new_log)?;
        self.vfs.sync()?;

        // Apply staged changes to active memory state on success
        for op in self.staged.drain(..) {
            match op {
                StagedOp::Put(k, v) => {
                    self.shredded_keys.remove(&k);
                    self.active_data.insert(k, v);
                }
                StagedOp::Delete(k) => {
                    self.active_data.remove(&k);
                }
            }
        }

        self.seq_no = new_seq;
        self.last_chain_hash = current_hash;
        self.raw_log_bytes = new_log;

        Ok(self.seq_no)
    }

    /// Explicitly syncs underlying storage.
    pub fn sync(&self) -> io::Result<()> {
        self.vfs.sync()
    }

    /// Recovers state from `FaultVfs` path, stopping at the last complete and valid record.
    pub fn recover(vfs: &FaultVfs, path: &Path) -> io::Result<Self> {
        let raw_bytes = match vfs.read_file(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e),
        };

        let mut engine = Self::new(vfs.clone(), path.to_path_buf());
        let mut offset = 0;
        let mut valid_until_offset = 0;

        let mut pending_batch: Vec<(u8, Vec<u8>, Vec<u8>)> = Vec::new();

        while offset < raw_bytes.len() {
            let record = match Self::decode_record(&raw_bytes[offset..]) {
                Some((rec, len)) => {
                    offset += len;
                    rec
                }
                None => {
                    // Torn write or corrupted record at tail -> halt recovery
                    break;
                }
            };

            // Verify chain hash
            let checksum = fnv1a_32(&raw_bytes[offset - record.raw_len..offset - 4]);
            if record.checksum != checksum || record.prev_hash != engine.last_chain_hash {
                // Hash mismatch or checksum mismatch -> corrupt cutoff
                break;
            }

            engine.last_chain_hash = combine_hash(engine.last_chain_hash, checksum, record.seq_no);

            match record.tag {
                1 => {
                    // Put
                    pending_batch.push((1, record.key, record.value));
                }
                2 => {
                    // Delete
                    pending_batch.push((2, record.key, Vec::new()));
                }
                3 => {
                    // Forget
                    engine.active_data.remove(&record.key);
                    engine.shredded_keys.insert(record.key);
                    engine.seq_no = record.seq_no;
                    valid_until_offset = offset;
                }
                4 => {
                    // Commit
                    for (tag, k, v) in pending_batch.drain(..) {
                        if tag == 1 {
                            engine.shredded_keys.remove(&k);
                            engine.active_data.insert(k, v);
                        } else if tag == 2 {
                            engine.active_data.remove(&k);
                        }
                    }
                    engine.seq_no = record.seq_no;
                    valid_until_offset = offset;
                }
                5 => {
                    // Rollback
                    pending_batch.clear();
                    engine.seq_no = record.seq_no;
                    valid_until_offset = offset;
                }
                _ => break,
            }
        }

        engine.raw_log_bytes = raw_bytes[..valid_until_offset].to_vec();
        Ok(engine)
    }

    /// Returns the sequence number of the latest committed transaction.
    pub fn last_seq(&self) -> u64 {
        self.seq_no
    }

    /// Returns the current hash chain head.
    pub fn chain_hash(&self) -> u64 {
        self.last_chain_hash
    }

    /// Returns the count of active key-value pairs.
    pub fn active_key_count(&self) -> usize {
        self.active_data.len()
    }

    fn encode_record(
        tag: u8,
        seq_no: u64,
        prev_hash: u64,
        key: &[u8],
        value: &[u8],
    ) -> Vec<u8> {
        let mut buf = Vec::with_capacity(1 + 8 + 8 + 4 + key.len() + 4 + value.len() + 4);
        buf.push(tag);
        buf.extend_from_slice(&seq_no.to_le_bytes());
        buf.extend_from_slice(&prev_hash.to_le_bytes());
        let k_len = u32::try_from(key.len()).unwrap_or(u32::MAX);
        buf.extend_from_slice(&k_len.to_le_bytes());
        buf.extend_from_slice(key);
        let v_len = u32::try_from(value.len()).unwrap_or(u32::MAX);
        buf.extend_from_slice(&v_len.to_le_bytes());
        buf.extend_from_slice(value);

        let checksum = fnv1a_32(&buf);
        buf.extend_from_slice(&checksum.to_le_bytes());
        buf
    }

    fn decode_record(slice: &[u8]) -> Option<(DecodedRecord, usize)> {
        // Min length: 1 + 8 + 8 + 4 + 0 + 4 + 0 + 4 = 29
        if slice.len() < 29 {
            return None;
        }

        let tag = slice[0];
        let seq_no = u64::from_le_bytes(slice[1..9].try_into().ok()?);
        let prev_hash = u64::from_le_bytes(slice[9..17].try_into().ok()?);
        let key_len = u32::from_le_bytes(slice[17..21].try_into().ok()?) as usize;

        if slice.len() < 21 + key_len + 4 + 4 {
            return None;
        }

        let key = slice[21..21 + key_len].to_vec();
        let val_offset = 21 + key_len;
        let val_len = u32::from_le_bytes(slice[val_offset..val_offset + 4].try_into().ok()?) as usize;

        let total_len = val_offset + 4 + val_len + 4;
        if slice.len() < total_len {
            return None;
        }

        let value = slice[val_offset + 4..val_offset + 4 + val_len].to_vec();
        let checksum_offset = val_offset + 4 + val_len;
        let checksum = u32::from_le_bytes(slice[checksum_offset..checksum_offset + 4].try_into().ok()?);

        Some((
            DecodedRecord {
                tag,
                seq_no,
                prev_hash,
                key,
                value,
                checksum,
                raw_len: total_len,
            },
            total_len,
        ))
    }
}

struct DecodedRecord {
    tag: u8,
    seq_no: u64,
    prev_hash: u64,
    key: Vec<u8>,
    value: Vec<u8>,
    checksum: u32,
    raw_len: usize,
}
