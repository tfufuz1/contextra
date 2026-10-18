// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: SystemUnderTest (SUT) trait and NaiveWalSut reference implementation for DST.
// INVARIANTEN: Safe Rust, zero panic, robust checksummed WAL protocol.

use crate::dst::op::DstOp;
use crate::sim_fs::{SimFs, SimOpenOptions, SimVfs};
use std::collections::BTreeMap;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

/// System Under Test interface for Deterministic Simulation Testing.
pub trait SystemUnderTest {
    /// Applies an operation to the SUT.
    fn apply(&mut self, op: &DstOp) -> io::Result<()>;
    /// Commits staged transaction and returns sequence number.
    fn commit(&mut self) -> io::Result<u64>;
    /// Recovers state after a crash on `sim_fs`.
    fn crash_and_recover(&mut self, sim_fs: &SimFs) -> io::Result<()>;
    /// Returns current visible snapshot map.
    fn snapshot(&self) -> BTreeMap<Vec<u8>, Vec<u8>>;
}

/// Decoded WAL record.
struct DecodedRecord {
    tag: u8,
    seq: u64,
    key: Vec<u8>,
    val: Vec<u8>,
    raw_len: usize,
}

/// Simple FNV-1a checksum for record integrity checks.
fn fnv1a_checksum(data: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for &b in data {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

/// Correct reference WAL implementation backed by `SimFs`.
#[derive(Debug, Clone)]
pub struct NaiveWalSut {
    sim_fs: SimFs,
    wal_path: PathBuf,
    active_map: BTreeMap<Vec<u8>, Vec<u8>>,
    staged: Vec<DstOp>,
    seq: u64,
}

impl NaiveWalSut {
    /// Creates a new `NaiveWalSut` on `SimFs`.
    pub fn new(sim_fs: SimFs, wal_path: PathBuf) -> Self {
        Self {
            sim_fs,
            wal_path,
            active_map: BTreeMap::new(),
            staged: Vec::new(),
            seq: 0,
        }
    }

    fn encode_record(tag: u8, seq: u64, key: &[u8], val: &[u8]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(1 + 8 + 4 + key.len() + 4 + val.len() + 4);
        buf.push(tag);
        buf.extend_from_slice(&seq.to_le_bytes());
        let k_len = u32::try_from(key.len()).unwrap_or(u32::MAX);
        buf.extend_from_slice(&k_len.to_le_bytes());
        buf.extend_from_slice(key);
        let v_len = u32::try_from(val.len()).unwrap_or(u32::MAX);
        buf.extend_from_slice(&v_len.to_le_bytes());
        buf.extend_from_slice(val);

        let checksum = fnv1a_checksum(&buf);
        buf.extend_from_slice(&checksum.to_le_bytes());
        buf
    }

    fn decode_record(slice: &[u8]) -> Option<DecodedRecord> {
        // Min header: 1 (tag) + 8 (seq) + 4 (key_len) + 4 (val_len) + 4 (checksum) = 21 bytes
        if slice.len() < 21 {
            return None;
        }

        let tag = slice[0];
        let seq = u64::from_le_bytes(slice[1..9].try_into().ok()?);
        let key_len = u32::from_le_bytes(slice[9..13].try_into().ok()?) as usize;

        if slice.len() < 13 + key_len + 8 {
            return None;
        }

        let key = slice[13..13 + key_len].to_vec();
        let val_offset = 13 + key_len;
        let val_len =
            u32::from_le_bytes(slice[val_offset..val_offset + 4].try_into().ok()?) as usize;

        let total_len = val_offset + 4 + val_len + 4;
        if slice.len() < total_len {
            return None;
        }

        let val = slice[val_offset + 4..val_offset + 4 + val_len].to_vec();
        let checksum_offset = val_offset + 4 + val_len;
        let expected_checksum = u32::from_le_bytes(
            slice[checksum_offset..checksum_offset + 4]
                .try_into()
                .ok()?,
        );

        let calculated_checksum = fnv1a_checksum(&slice[..checksum_offset]);
        if expected_checksum != calculated_checksum {
            return None;
        }

        Some(DecodedRecord {
            tag,
            seq,
            key,
            val,
            raw_len: total_len,
        })
    }
}

impl SystemUnderTest for NaiveWalSut {
    fn apply(&mut self, op: &DstOp) -> io::Result<()> {
        match op {
            DstOp::Put { key, value } => {
                self.staged.push(DstOp::Put {
                    key: key.clone(),
                    value: value.clone(),
                });
            }
            DstOp::Delete { key } => {
                self.staged.push(DstOp::Delete { key: key.clone() });
            }
            DstOp::Forget { key } => {
                self.staged.push(DstOp::Forget { key: key.clone() });
            }
            DstOp::Rollback => {
                self.staged.clear();
            }
            DstOp::Sync => {
                let mut opts = SimOpenOptions::new();
                opts.create(true).write(true);
                if let Ok(file) = self.sim_fs.open(&self.wal_path, &opts) {
                    let _ = file.sync_all(); // INTENTIONAL-DROP
                }
            }
            DstOp::Commit => {
                self.commit()?;
            }
        }
        Ok(())
    }

    fn commit(&mut self) -> io::Result<u64> {
        if self.staged.is_empty() {
            return Ok(self.seq);
        }

        let mut opts = SimOpenOptions::new();
        opts.create(true).write(true);

        let file = match self.sim_fs.open(&self.wal_path, &opts) {
            Ok(f) => f,
            Err(e) => {
                self.staged.clear();
                return Err(e);
            }
        };

        let offset = match file.len() {
            Ok(l) => l,
            Err(e) => {
                self.staged.clear();
                return Err(e);
            }
        };

        let next_seq = self.seq + 1;
        let mut payload_bytes = Vec::new();

        for op in &self.staged {
            let (tag, k, v) = match op {
                DstOp::Put { key, value } => (1u8, key.as_slice(), value.as_slice()),
                DstOp::Delete { key } => (2u8, key.as_slice(), &[][..]),
                DstOp::Forget { key } => (3u8, key.as_slice(), &[][..]),
                _ => continue,
            };
            let rec = Self::encode_record(tag, next_seq, k, v);
            payload_bytes.extend_from_slice(&rec);
        }

        // Commit tag record
        let commit_rec = Self::encode_record(4, next_seq, &[], &[]);
        payload_bytes.extend_from_slice(&commit_rec);

        if let Err(e) = file.write_at(&payload_bytes, offset) {
            let _ = file.set_len(offset); // INTENTIONAL-DROP
            self.staged.clear();
            return Err(e);
        }

        if let Err(e) = file.sync_data() {
            let _ = file.set_len(offset); // INTENTIONAL-DROP
            let _ = file.sync_data(); // INTENTIONAL-DROP
            self.staged.clear();
            return Err(e);
        }

        let parent = self
            .wal_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));

        if let Err(e) = self.sim_fs.sync_dir(parent) {
            let _ = file.set_len(offset); // INTENTIONAL-DROP
            let _ = file.sync_data(); // INTENTIONAL-DROP
            self.staged.clear();
            return Err(e);
        }

        // Update active in-memory state after durable sync
        for op in self.staged.drain(..) {
            match op {
                DstOp::Put { key, value } => {
                    self.active_map.insert(key, value);
                }
                DstOp::Delete { key } | DstOp::Forget { key } => {
                    self.active_map.remove(&key);
                }
                _ => {}
            }
        }

        self.seq = next_seq;
        Ok(self.seq)
    }

    fn crash_and_recover(&mut self, sim_fs: &SimFs) -> io::Result<()> {
        self.sim_fs = sim_fs.clone();
        self.active_map.clear();
        self.staged.clear();
        self.seq = 0;

        let mut opts = SimOpenOptions::new();
        opts.read(true);
        let file = match self.sim_fs.open(&self.wal_path, &opts) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e),
        };

        let file_len = usize::try_from(file.len()?).unwrap_or(usize::MAX);
        let mut buf = vec![0u8; file_len];
        let read_bytes = file.read_at(&mut buf, 0)?;
        buf.truncate(read_bytes);

        let mut offset = 0;
        let mut pending_batch: Vec<(u8, Vec<u8>, Vec<u8>)> = Vec::new();

        while offset < buf.len() {
            let rec = match Self::decode_record(&buf[offset..]) {
                Some(r) => r,
                None => break, // Cutoff at corrupt/torn write record
            };

            offset += rec.raw_len;

            match rec.tag {
                1 => pending_batch.push((1, rec.key, rec.val)),
                2 => pending_batch.push((2, rec.key, Vec::new())),
                3 => pending_batch.push((3, rec.key, Vec::new())),
                4 => {
                    // Commit acknowledged
                    for (btag, k, v) in pending_batch.drain(..) {
                        if btag == 1 {
                            self.active_map.insert(k, v);
                        } else {
                            self.active_map.remove(&k);
                        }
                    }
                    self.seq = rec.seq;
                }
                _ => break,
            }
        }

        Ok(())
    }

    fn snapshot(&self) -> BTreeMap<Vec<u8>, Vec<u8>> {
        self.active_map.clone()
    }
}
