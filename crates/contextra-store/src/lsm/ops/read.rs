use super::super::engine::LsmStorage;
use super::super::scan::{check_in_range, SstableScanMode};
use super::super::validate_key;
use bytes::Bytes;
use contextra_core::{Result, StorageEngine, TOMBSTONE_BIT};
use std::sync::atomic::Ordering;

pub(crate) async fn get_tracked(
    storage: &LsmStorage,
    tx_id: contextra_core::TxId,
    key: &[u8],
) -> Result<Option<Bytes>> {
    validate_key(key)?;
    let snapshot_seq = storage.last_applied_seq.load(Ordering::Acquire);
    let _lease = storage.snapshot_registry.acquire(snapshot_seq);
    get_at_seq_tracked(storage, tx_id, key, snapshot_seq).await
}

/// Retrieves a value by key at `snapshot_seq`, registering the read in the transaction's ReadSet.
///
/// # Invarianten & SSI ReadSet Clamping
/// `INV-MVCC-SSI-1`: A caller passing `snapshot_seq = u64::MAX` intends to read the latest state.
/// To accurately detect concurrent write conflicts during transaction commit, the registered
/// sequence number must reflect the actual sequence number observed at the time of the read,
/// rather than `u64::MAX`. Therefore, the registered sequence number is clamped to
/// `min(snapshot_seq, storage.last_applied_seq.load(Ordering::Acquire))`, captured *before* reading.
/// Registrieren eines Werts, der nicht größer ist als der tatsächlich gelesene, ist konservativ (im Zweifel ein Konflikt, nie ein übersehener).
/// Reading semantics remain unchanged and continue to query `storage.get_at_seq(key, snapshot_seq)`.
///
/// # Hinweis zu Phantomschutz
/// Note: `scan_prefix_tracked` currently does not offer phantom protection; phantom protection is
/// handled in a separate task.
pub(crate) async fn get_at_seq_tracked(
    storage: &LsmStorage,
    tx_id: contextra_core::TxId,
    key: &[u8],
    snapshot_seq: u64,
) -> Result<Option<Bytes>> {
    validate_key(key)?;
    let current_seq = storage.last_applied_seq.load(Ordering::Acquire);
    let registered_seq = snapshot_seq.min(current_seq);
    let _lease = storage.snapshot_registry.acquire(registered_seq);
    storage
        .tx_buffer
        .register_read(tx_id, key.to_vec(), registered_seq);
    storage.get_at_seq(key, snapshot_seq).await
}

pub(crate) async fn scan_prefix_tracked(
    storage: &LsmStorage,
    tx_id: contextra_core::TxId,
    prefix: &[u8],
) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let snapshot_seq = storage.last_applied_seq.load(Ordering::Acquire);
    let _lease = storage.snapshot_registry.acquire(snapshot_seq);
    storage
        .tx_buffer
        .register_prefix_read(tx_id, prefix.to_vec(), snapshot_seq);
    let results = storage.scan_prefix_at(prefix, snapshot_seq).await?;
    for (k, _) in &results {
        storage
            .tx_buffer
            .register_read(tx_id, k.clone(), snapshot_seq);
    }
    Ok(results)
}

pub(super) async fn get(storage: &LsmStorage, key: &[u8]) -> Result<Option<Bytes>> {
    validate_key(key)?;
    let snapshot_seq = storage.last_applied_seq.load(Ordering::Acquire);
    let _lease = storage.snapshot_registry.acquire(snapshot_seq);
    let res = storage.get_at_seq(key, snapshot_seq).await?;
    tracing::debug!(
        "LsmStorage::get key={:?} seq={} found={}",
        String::from_utf8_lossy(key),
        snapshot_seq,
        res.is_some()
    );
    Ok(res)
}

pub(super) async fn get_at_seq(
    storage: &LsmStorage,
    key: &[u8],
    seq_no: u64,
) -> Result<Option<Bytes>> {
    validate_key(key)?;
    let _lease = storage.snapshot_registry.acquire(seq_no);
    storage.compaction_engine.record_read_op();
    // Genau EINMAL laden — Snapshot-Konsistenz über die gesamte Methode (INVARIANT-2)
    let snapshot_tx = storage.last_committed_tx.load(Ordering::Acquire);
    let (memtable, immutable_memtables) = {
        let state = storage.state.read().await;
        (
            std::sync::Arc::clone(&state.memtable),
            state.immutable_memtables.clone(),
        )
    };
    tracing::debug!(
        "LsmStorage::get_at_seq key={:?} seq={} snapshot_tx={}",
        String::from_utf8_lossy(key),
        seq_no,
        snapshot_tx
    );

    // 1. MemTable (only if seq_no in entry <= target seq_no AND tx_id <= snapshot_tx)
    if let Some((val, seq, _tx)) = memtable.get_at_seq(key, seq_no, snapshot_tx) {
        if (seq & TOMBSTONE_BIT) != 0 {
            return Ok(None);
        }
        return Ok(Some(val));
    }

    // 2. Immutable MemTables (newest first)
    for mt in immutable_memtables.iter().rev() {
        if let Some((val, seq, _tx)) = mt.get_at_seq(key, seq_no, snapshot_tx) {
            if (seq & TOMBSTONE_BIT) != 0 {
                return Ok(None);
            }
            return Ok(Some(val));
        }
    }

    // 3. SSTables (newest first, filtered by seq_no and snapshot_tx)
    let sstables = storage.sstables.read().await.clone();
    for sst in sstables.iter().rev() {
        if (sst.metadata().min_tx_id < contextra_core::TxId::INTERNAL_BASE
            && snapshot_tx < sst.metadata().min_tx_id)
            || seq_no < sst.metadata().min_seq
        {
            continue;
        }
        if let Some((val, seq, tx)) = sst.get_at(key, seq_no, snapshot_tx).await? {
            tracing::debug!(
                "LsmStorage::get_at_seq SSTable check: seq={} target_seq={} tx={} snapshot_tx={}",
                seq & !TOMBSTONE_BIT,
                seq_no,
                tx,
                snapshot_tx
            );
            if (seq & TOMBSTONE_BIT) != 0 {
                tracing::debug!("LsmStorage::get_at_seq FOUND TOMBSTONE");
                return Ok(None);
            }
            tracing::debug!("LsmStorage::get_at_seq MATCH found in SSTable");
            return Ok(Some(val));
        }
    }

    Ok(None)
}

pub(super) async fn scan_prefix(
    storage: &LsmStorage,
    prefix: &[u8],
) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let snapshot_seq = storage.last_applied_seq.load(Ordering::Acquire);
    let _lease = storage.snapshot_registry.acquire(snapshot_seq);
    storage.scan_prefix_at(prefix, snapshot_seq).await
}

pub(super) async fn scan_prefix_bounded(
    storage: &LsmStorage,
    prefix: &[u8],
    limit: usize,
    cursor: Option<&[u8]>,
) -> Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)> {
    let snapshot_seq = storage.last_applied_seq.load(Ordering::Acquire);
    let _lease = storage.snapshot_registry.acquire(snapshot_seq);
    let cur_bytes = cursor.map(Bytes::copy_from_slice);

    let map = storage
        .collect_visible_entries(
            SstableScanMode::Prefix(prefix),
            |k, _raw_seq, _tx| {
                if let Some(cb) = cursor {
                    if k <= cb {
                        return false;
                    }
                }
                k.starts_with(prefix)
            },
            true,
            None,
            "scan_prefix_bounded()",
        )
        .await?;

    // Cursor-Bound ableiten für den Paginierungs-Iterator
    let range_bound = match &cur_bytes {
        Some(cb) => std::ops::Bound::Excluded(cb.clone()),
        None => std::ops::Bound::Unbounded,
    };

    let mut results = Vec::new();
    let mut iter = map.range((range_bound, std::ops::Bound::Unbounded));

    for (k, (v, seq)) in iter.by_ref() {
        if (seq & TOMBSTONE_BIT) == 0 {
            results.push((k.to_vec(), v.to_vec()));
            if results.len() == limit {
                break;
            }
        }
    }

    let next_cursor = if results.len() == limit {
        let mut has_more = false;
        for (_k, (_v, seq)) in iter {
            if (seq & TOMBSTONE_BIT) == 0 {
                has_more = true;
                break;
            }
        }
        if has_more {
            results.last().map(|(k, _)| k.clone())
        } else {
            None
        }
    } else {
        None
    };

    Ok((results, next_cursor))
}

pub(super) async fn scan_prefix_at(
    storage: &LsmStorage,
    prefix: &[u8],
    seq_no: u64,
) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let _lease = storage.snapshot_registry.acquire(seq_no);
    let map = storage
        .collect_visible_entries(
            SstableScanMode::Prefix(prefix),
            |k, raw_seq, _tx| raw_seq <= seq_no && k.starts_with(prefix),
            false,
            None,
            "scan_prefix_at()",
        )
        .await?;

    let mut results = Vec::with_capacity(map.len());
    for (k, (v, seq)) in map {
        if (seq & TOMBSTONE_BIT) == 0 {
            results.push((k.to_vec(), v.to_vec()));
        }
    }

    Ok(results)
}

pub(super) async fn scan_bounded(
    storage: &LsmStorage,
    start: std::ops::Bound<&[u8]>,
    end: std::ops::Bound<&[u8]>,
    limit: usize,
    cursor: Option<&[u8]>,
) -> Result<(Vec<(Vec<u8>, Vec<u8>)>, Option<Vec<u8>>)> {
    use std::ops::Bound;

    let snapshot_seq = storage.last_applied_seq.load(Ordering::Acquire);
    let _lease = storage.snapshot_registry.acquire(snapshot_seq);

    let effective_start = match cursor {
        Some(c) => Bound::Excluded(c),
        None => start,
    };

    let map = storage
        .collect_visible_entries(
            SstableScanMode::Range(effective_start, end),
            |k, _raw_seq, _tx| check_in_range(k, effective_start, end),
            true,
            None,
            "scan_bounded()",
        )
        .await?;

    // 4. Apply limit (cursor is already handled via effective_start)
    let mut results = Vec::new();
    let mut iter = map.into_iter();

    for (k, (v, seq)) in iter.by_ref() {
        if (seq & TOMBSTONE_BIT) == 0 {
            results.push((k.to_vec(), v.to_vec()));
            if results.len() == limit {
                break;
            }
        }
    }

    let next_cursor = if results.len() == limit {
        let mut has_more = false;
        for (_k, (_v, seq)) in iter {
            if (seq & TOMBSTONE_BIT) == 0 {
                has_more = true;
                break;
            }
        }
        if has_more {
            results.last().map(|(k, _)| k.clone())
        } else {
            None
        }
    } else {
        None
    };

    Ok((results, next_cursor))
}

pub(super) async fn scan(
    storage: &LsmStorage,
    start: std::ops::Bound<&[u8]>,
    end: std::ops::Bound<&[u8]>,
    limit: Option<usize>,
) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let snapshot_seq = storage.last_applied_seq.load(Ordering::Acquire);
    let _lease = storage.snapshot_registry.acquire(snapshot_seq);
    let map = storage
        .collect_visible_entries(
            SstableScanMode::Range(start, end),
            |k, _raw_seq, _tx| check_in_range(k, start, end),
            false,
            limit,
            "scan()",
        )
        .await?;

    // 4. Filter tombstones and apply limit
    let mut results = Vec::new();
    for (k, (v, seq)) in map {
        if (seq & TOMBSTONE_BIT) == 0 {
            results.push((k.to_vec(), v.to_vec()));
            if let Some(lim) = limit {
                if results.len() == lim {
                    break;
                }
            }
        }
    }

    Ok(results)
}
