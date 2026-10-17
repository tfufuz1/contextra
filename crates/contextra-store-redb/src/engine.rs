//! Implementation of `StorageEngine` trait on top of `redb`.

use std::collections::BTreeMap;
use std::ops::Bound;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use bytes::Bytes;
use parking_lot::RwLock;
use redb::{ReadableTable, TableDefinition};

use contextra_ports::{BoxFuture, StorageEngine, StorageStats};
use contextra_types::{ContextraError, Result, TxId};

use crate::capabilities::RedbDeletionCapabilities;

const DATA_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("contextra_data_v1");
const META_TABLE: TableDefinition<&str, u64> = TableDefinition::new("contextra_meta_v1");

const META_LAST_SEQ: &str = "last_seq_no";
const META_LAST_TX: &str = "last_tx_id";

#[derive(Debug, Clone)]
enum StagedOp {
    Put(Vec<u8>),
    Delete,
}

#[derive(Debug, Default)]
struct Staging {
    uncommitted: BTreeMap<TxId, BTreeMap<Vec<u8>, StagedOp>>,
}

/// Helper encoding functions for composite redb keys and values.
fn encode_key(key: &[u8], seq: u64) -> Result<Vec<u8>> {
    let key_len = u16::try_from(key.len()).map_err(|_| {
        ContextraError::InvalidInput(format!(
            "Key length {} exceeds maximum supported redb key length {}",
            key.len(),
            u16::MAX
        ))
    })?;
    let mut buf = Vec::with_capacity(2 + key.len() + 8);
    buf.extend_from_slice(&key_len.to_be_bytes());
    buf.extend_from_slice(key);
    buf.extend_from_slice(&seq.to_be_bytes());
    Ok(buf)
}

fn decode_key(encoded: &[u8]) -> Option<(Vec<u8>, u64)> {
    if encoded.len() < 2 + 8 {
        return None;
    }
    let key_len = u16::from_be_bytes([encoded[0], encoded[1]]) as usize;
    if encoded.len() < 2 + key_len + 8 {
        return None;
    }
    let user_key = encoded[2..2 + key_len].to_vec();
    let seq_bytes = <[u8; 8]>::try_from(&encoded[2 + key_len..2 + key_len + 8]).ok()?;
    let seq = u64::from_be_bytes(seq_bytes);
    Some((user_key, seq))
}

fn encode_val(op: &StagedOp, tx_id: TxId) -> Vec<u8> {
    let mut buf = Vec::new();
    match op {
        StagedOp::Put(v) => {
            buf.push(0u8);
            buf.extend_from_slice(&tx_id.0.to_be_bytes());
            buf.extend_from_slice(v);
        }
        StagedOp::Delete => {
            buf.push(1u8);
            buf.extend_from_slice(&tx_id.0.to_be_bytes());
        }
    }
    buf
}

fn decode_val(val_bytes: &[u8]) -> Option<(bool, TxId, &[u8])> {
    if val_bytes.len() < 1 + 8 {
        return None;
    }
    let is_tombstone = val_bytes[0] == 1;
    let tx_id_bytes = <[u8; 8]>::try_from(&val_bytes[1..9]).ok()?;
    let tx_id = TxId(u64::from_be_bytes(tx_id_bytes));
    let payload = &val_bytes[9..];
    Some((is_tombstone, tx_id, payload))
}

/// Storage engine adapter wrapping [`redb::Database`].
///
/// # MVCC & Transaction Isolation
/// Writes are staged in uncommitted transaction buffers in memory until [`commit`](StorageEngine::commit)
/// is called. Upon commit, writes are assigned a monotonically increasing sequence number and written
/// into redb's B-tree table in a single atomic redb transaction.
/// Point lookups and range scans evaluate historical versions at sequence numbers to guarantee
/// MVCC snapshot isolation.
///
/// # Honest Deletion Semantics & Deviations
/// Deleting a key removes it logically by inserting a tombstone or clearing its table entry in redb.
/// However, redb retains deleted bytes in internal B-tree free-list pages until overwritten by future allocations.
/// Physical disk zeroing/purge is **not** supported by redb.
/// Thus [`deletion_capabilities`](RedbStorageEngine::deletion_capabilities) reports `max_deletion_stage = S1`.
#[derive(Clone)]
pub struct RedbStorageEngine {
    db: Arc<redb::Database>,
    staging: Arc<RwLock<Staging>>,
    seq: Arc<AtomicU64>,
    last_tx: Arc<RwLock<TxId>>,
}

impl std::fmt::Debug for RedbStorageEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbStorageEngine")
            .field("last_seq_no", &self.seq.load(Ordering::SeqCst))
            .field("last_tx_id", &*self.last_tx.read())
            .finish()
    }
}

impl RedbStorageEngine {
    /// Opens or creates a redb storage engine at `path`.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = redb::Database::create(path)
            .map_err(|e| ContextraError::Storage(format!("Failed to open redb database: {e}")))?;
        Self::new(db)
    }

    /// Wraps an existing [`redb::Database`].
    pub fn new(db: redb::Database) -> Result<Self> {
        let db_arc = Arc::new(db);
        let write_txn = db_arc
            .begin_write()
            .map_err(|e| ContextraError::Storage(format!("Failed to begin redb write txn: {e}")))?;
        {
            write_txn
                .open_table(DATA_TABLE)
                .map_err(|e| ContextraError::Storage(format!("Failed to open DATA_TABLE: {e}")))?;
            write_txn
                .open_table(META_TABLE)
                .map_err(|e| ContextraError::Storage(format!("Failed to open META_TABLE: {e}")))?;
        }
        write_txn
            .commit()
            .map_err(|e| ContextraError::Storage(format!("Failed to commit init txn: {e}")))?;

        let read_txn = db_arc
            .begin_read()
            .map_err(|e| ContextraError::Storage(format!("Failed to begin redb read txn: {e}")))?;
        let meta_table = read_txn
            .open_table(META_TABLE)
            .map_err(|e| ContextraError::Storage(format!("Failed to open META_TABLE: {e}")))?;

        let initial_seq = meta_table
            .get(META_LAST_SEQ)
            .map_err(|e| ContextraError::Storage(format!("Failed to read META_LAST_SEQ: {e}")))?
            .map_or(0u64, |v| v.value());

        let initial_tx = meta_table
            .get(META_LAST_TX)
            .map_err(|e| ContextraError::Storage(format!("Failed to read META_LAST_TX: {e}")))?
            .map_or(0u64, |v| v.value());

        Ok(Self {
            db: db_arc,
            staging: Arc::new(RwLock::new(Staging::default())),
            seq: Arc::new(AtomicU64::new(initial_seq)),
            last_tx: Arc::new(RwLock::new(TxId(initial_tx))),
        })
    }

    /// Returns the deletion capabilities descriptor for redb backend.
    ///
    /// # Deletion Semantics
    /// Physical purge is **not** available on redb. Page allocations in redb are released
    /// to internal B-tree free-lists when keys are deleted, but deleted key/value bytes
    /// remain on disk in the database file until overwritten by subsequent writes.
    /// Therefore, forget receipts reach at most stage **S1** (Logical free-listing).
    pub fn deletion_capabilities(&self) -> RedbDeletionCapabilities {
        RedbDeletionCapabilities::default()
    }

    /// Collects visible key-value pairs matching conditions from redb `DATA_TABLE`.
    fn collect_visible<F>(
        &self,
        as_of_seq: u64,
        key_filter: F,
        limit: Option<usize>,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>>
    where
        F: Fn(&[u8]) -> bool,
    {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| ContextraError::Storage(format!("redb read error: {e}")))?;
        let table = read_txn
            .open_table(DATA_TABLE)
            .map_err(|e| ContextraError::Storage(format!("redb open table error: {e}")))?;

        let iter = table
            .iter()
            .map_err(|e| ContextraError::Storage(format!("redb iter error: {e}")))?;

        // Group entries by user key, taking latest version with seq <= as_of_seq
        let mut key_versions: BTreeMap<Vec<u8>, (u64, bool, Vec<u8>)> = BTreeMap::new();

        for item in iter {
            let (k, v) = item.map_err(|e| ContextraError::Storage(format!("redb iter item error: {e}")))?;
            if let Some((user_key, ver_seq)) = decode_key(k.value()) {
                if ver_seq <= as_of_seq && key_filter(&user_key) {
                    if let Some((is_tombstone, _, payload)) = decode_val(v.value()) {
                        let is_newer = key_versions
                            .get(&user_key)
                            .is_none_or(|(best_seq, _, _)| ver_seq >= *best_seq);
                        if is_newer {
                            key_versions.insert(user_key, (ver_seq, is_tombstone, payload.to_vec()));
                        }
                    }
                }
            }
        }

        let mut results = Vec::new();
        for (user_key, (_, is_tombstone, payload)) in key_versions {
            if !is_tombstone {
                results.push((user_key, payload));
                if let Some(lim) = limit {
                    if results.len() >= lim {
                        break;
                    }
                }
            }
        }

        Ok(results)
    }
}

impl StorageEngine for RedbStorageEngine {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
        self.get_at_seq(key, u64::MAX)
    }

    fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let read_txn = self
                .db
                .begin_read()
                .map_err(|e| ContextraError::Storage(format!("redb read error: {e}")))?;
            let table = read_txn
                .open_table(DATA_TABLE)
                .map_err(|e| ContextraError::Storage(format!("redb open table error: {e}")))?;

            let start = encode_key(key, 0)?;
            let end = encode_key(key, seq)?;

            let range = table
                .range(start.as_slice()..=end.as_slice())
                .map_err(|e| ContextraError::Storage(format!("redb range error: {e}")))?;

            let mut latest: Option<(u64, bool, Vec<u8>)> = None;
            for item in range {
                let (k, v) = item.map_err(|e| ContextraError::Storage(format!("redb item error: {e}")))?;
                if let Some((_, ver_seq)) = decode_key(k.value()) {
                    if ver_seq <= seq {
                        if let Some((is_tombstone, _, payload)) = decode_val(v.value()) {
                            let is_newer = latest
                                .as_ref()
                                .is_none_or(|(best_seq, _, _)| ver_seq >= *best_seq);
                            if is_newer {
                                latest = Some((ver_seq, is_tombstone, payload.to_vec()));
                            }
                        }
                    }
                }
            }

            if let Some((_, is_tombstone, payload)) = latest {
                if !is_tombstone {
                    return Ok(Some(Bytes::from(payload)));
                }
            }

            Ok(None)
        })
    }

    fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut staging = self.staging.write();
            staging
                .uncommitted
                .entry(tx_id)
                .or_default()
                .insert(key.to_vec(), StagedOp::Put(value.to_vec()));
            Ok(())
        })
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut staging = self.staging.write();
            staging
                .uncommitted
                .entry(tx_id)
                .or_default()
                .insert(key.to_vec(), StagedOp::Delete);
            Ok(())
        })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let ops = {
                let mut staging = self.staging.write();
                staging.uncommitted.remove(&tx_id)
            };

            let new_seq = self.seq.fetch_add(1, Ordering::SeqCst) + 1;

            let write_txn = self
                .db
                .begin_write()
                .map_err(|e| ContextraError::Storage(format!("redb begin write error: {e}")))?;

            {
                let mut data_table = write_txn
                    .open_table(DATA_TABLE)
                    .map_err(|e| ContextraError::Storage(format!("redb open data table error: {e}")))?;

                if let Some(ops_map) = ops {
                    for (key, op) in ops_map {
                        let enc_key = encode_key(&key, new_seq)?;
                        let enc_val = encode_val(&op, tx_id);
                        data_table
                            .insert(enc_key.as_slice(), enc_val.as_slice())
                            .map_err(|e| {
                                ContextraError::Storage(format!("redb insert data error: {e}"))
                            })?;
                    }
                }

                let mut meta_table = write_txn
                    .open_table(META_TABLE)
                    .map_err(|e| ContextraError::Storage(format!("redb open meta table error: {e}")))?;

                meta_table
                    .insert(META_LAST_SEQ, &new_seq)
                    .map_err(|e| ContextraError::Storage(format!("redb insert meta seq error: {e}")))?;
                meta_table
                    .insert(META_LAST_TX, &tx_id.0)
                    .map_err(|e| ContextraError::Storage(format!("redb insert meta tx error: {e}")))?;
            }

            write_txn
                .commit()
                .map_err(|e| ContextraError::Storage(format!("redb write commit error: {e}")))?;

            *self.last_tx.write() = tx_id;
            Ok(())
        })
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let mut staging = self.staging.write();
            staging.uncommitted.remove(&tx_id);
            Ok(())
        })
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            {
                let mut staging = self.staging.write();
                staging.uncommitted.retain(|t, _| *t <= tx_id);
            }

            let write_txn = self
                .db
                .begin_write()
                .map_err(|e| ContextraError::Storage(format!("redb begin write error: {e}")))?;

            let mut max_remaining_seq = 0u64;

            {
                let mut data_table = write_txn
                    .open_table(DATA_TABLE)
                    .map_err(|e| ContextraError::Storage(format!("redb open data table error: {e}")))?;

                let mut keys_to_remove = Vec::new();

                let iter = data_table
                    .iter()
                    .map_err(|e| ContextraError::Storage(format!("redb iter error: {e}")))?;

                for item in iter {
                    let (k, v) = item.map_err(|e| ContextraError::Storage(format!("redb item error: {e}")))?;
                    if let Some((_, ver_seq)) = decode_key(k.value()) {
                        if let Some((_, item_tx, _)) = decode_val(v.value()) {
                            if item_tx > tx_id {
                                keys_to_remove.push(k.value().to_vec());
                            } else if ver_seq > max_remaining_seq {
                                max_remaining_seq = ver_seq;
                            }
                        }
                    }
                }

                for k in keys_to_remove {
                    data_table
                        .remove(k.as_slice())
                        .map_err(|e| ContextraError::Storage(format!("redb remove key error: {e}")))?;
                }

                let mut meta_table = write_txn
                    .open_table(META_TABLE)
                    .map_err(|e| ContextraError::Storage(format!("redb open meta error: {e}")))?;

                meta_table
                    .insert(META_LAST_SEQ, &max_remaining_seq)
                    .map_err(|e| ContextraError::Storage(format!("redb update meta seq error: {e}")))?;
                meta_table
                    .insert(META_LAST_TX, &tx_id.0)
                    .map_err(|e| ContextraError::Storage(format!("redb update meta tx error: {e}")))?;
            }

            write_txn
                .commit()
                .map_err(|e| ContextraError::Storage(format!("redb commit error: {e}")))?;

            self.seq.store(max_remaining_seq, Ordering::SeqCst);
            *self.last_tx.write() = tx_id;

            Ok(())
        })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        Box::pin(async move {
            let items = self.scan_prefix(b"").await?;
            let total_bytes: u64 = items
                .iter()
                .map(|(k, v)| (k.len() + v.len()) as u64)
                .sum();
            Ok(StorageStats {
                num_segments: 1,
                total_size_bytes: total_bytes,
                memtable_size_bytes: 0,
            })
        })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move { Ok(self.seq.load(Ordering::SeqCst)) })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(*self.last_tx.read()) })
    }

    fn pin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn unpin_checkpoint<'a>(&'a self, _seq_no: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            self.collect_visible(u64::MAX, |k| k.starts_with(prefix), None)
        })
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            self.collect_visible(seq_no, |k| k.starts_with(prefix), None)
        })
    }

    fn scan<'a>(
        &'a self,
        start: Bound<&'a [u8]>,
        end: Bound<&'a [u8]>,
        limit: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let start_bound = match start {
                Bound::Included(b) => Bound::Included(b.to_vec()),
                Bound::Excluded(b) => Bound::Excluded(b.to_vec()),
                Bound::Unbounded => Bound::Unbounded,
            };
            let end_bound = match end {
                Bound::Included(b) => Bound::Included(b.to_vec()),
                Bound::Excluded(b) => Bound::Excluded(b.to_vec()),
                Bound::Unbounded => Bound::Unbounded,
            };

            self.collect_visible(
                u64::MAX,
                move |k| match (&start_bound, &end_bound) {
                    (Bound::Included(s), Bound::Included(e)) => k >= s.as_slice() && k <= e.as_slice(),
                    (Bound::Included(s), Bound::Excluded(e)) => k >= s.as_slice() && k < e.as_slice(),
                    (Bound::Included(s), Bound::Unbounded) => k >= s.as_slice(),
                    (Bound::Excluded(s), Bound::Included(e)) => k > s.as_slice() && k <= e.as_slice(),
                    (Bound::Excluded(s), Bound::Excluded(e)) => k > s.as_slice() && k < e.as_slice(),
                    (Bound::Excluded(s), Bound::Unbounded) => k > s.as_slice(),
                    (Bound::Unbounded, Bound::Included(e)) => k <= e.as_slice(),
                    (Bound::Unbounded, Bound::Excluded(e)) => k < e.as_slice(),
                    (Bound::Unbounded, Bound::Unbounded) => true,
                },
                limit,
            )
        })
    }
}
