# Audit Report: MemTable and Read Path (Visibility, Scans, Lock Hierarchy)

**Date**: 2026-09-13
**Target Scope**: `contextra-store` (`memtable.rs`, `lsm/ops/read.rs`, `lsm/scan.rs`, `lsm/tests/scan_tests.rs`, `lsm/tests/mvcc_tests.rs`)
**Auditor**: Jules

---

## Executive Summary
An exhaustive audit of `MemTable` and `LsmStorage` read paths was conducted to verify visibility semantics (`seq_no`, `snapshot_tx`, `TOMBSTONE_BIT`), iteration and prefix/range scan bounds, multi-version precedence across storage layers (`MemTable` > `immutable_memtables` > `SSTables`), flush safety, and lock hierarchy deadlocks.

All findings have been documented below. Proven bugs have been fixed and verified with regression tests in `crates/contextra-store/tests/read_path_audit.rs`.

---

## Detailed Audit Findings

| ID | Severity | File:Line | Description | Proof / Evidence | Status |
|---|---|---|---|---|---|
| AUDIT-READ-01 | Low | `memtable.rs:337` | `get_at_seq` binary search did not mask `seq_no` parameter with `!TOMBSTONE_BIT`. If a caller passed a sequence number with `TOMBSTONE_BIT` set, binary search compared raw `seq_no` against masked entries. | In `tests/read_path_audit.rs::test_memtable_get_at_seq_with_tombstone_bit_in_param`, passing `15 \| TOMBSTONE_BIT` failed to find earlier valid version `10` before fix. | **FIXED** |
| AUDIT-READ-02 | Low | `memtable.rs:337` | `get_at_seq` binary search returned the first matching index when multiple versions had identical raw sequence numbers, potentially missing a later entry in the same sequence batch. | Advancing index `while last + 1 < versions.len() && (versions[last + 1].0 & !TOMBSTONE_BIT) == target_raw_seq` ensures latest version in equal-seq batch is examined first. | **FIXED** |
| AUDIT-READ-03 | Medium | `lsm/scan.rs:105` | SSTable prefix pruning check `prefix_end = prefix.to_vec(); prefix_end.last_mut().checked_add(1)` failed for prefixes ending in `0xFF` or empty prefix `b""`, causing unnecessary SSTable scans or incorrect range boundaries. | Replacing ad-hoc byte increment with `upper_bound_for_prefix(prefix)` correctly handles `0xFF` overflow and empty prefix `b""`. Verified in `tests/read_path_audit.rs::test_prefix_scan_ff_bytes_and_empty_prefix`. | **FIXED** |
| AUDIT-READ-04 | Info | `lsm/scan.rs:164` | `scan_memtable` had a redundant special-case branch for `SstableScanMode::Range(Unbounded, Unbounded)` that allocated full `mt.iter()` vector instead of using direct `scan_range_into_matching`. | Unified `SstableScanMode::Range(start, end)` branch to consistently call `mt.scan_range_into_matching`. | **FIXED** |
| AUDIT-READ-05 | Pass | `lsm/ops/read.rs:103` | Snapshot consistency across read methods: `get_at_seq` loads `last_committed_tx` exactly ONCE at method entry before reading memtable and SSTables. | Audited `get_at_seq` line 103: `let snapshot_tx = storage.last_committed_tx.load(Ordering::Acquire);` is loaded once and passed to all lower layers. | **VERIFIED** |
| AUDIT-READ-06 | Pass | `lsm/ops/compaction.rs:135` | Lock ordering during Flush transition: `commit_mutex` -> `state.write()` -> `sstables.write()`. Readers acquire `state.read()` or `sstables.read()`. | Lock order is acyclic: no lock is acquired in reverse order. `last_committed_tx` is updated via `advance_visibility()` before `sstables.push()` (ADR-043), preventing gap or duplicate views. Verified in `tests/read_path_audit.rs::test_concurrent_readers_during_repeated_force_flush`. | **VERIFIED** |
| AUDIT-READ-07 | Pass | `memtable.rs:155` | Zero panics / no unchecked indexing in non-test code: checked bounds, no `unwrap()` or `expect()` in production paths. | Executed `grep -n "unwrap\|expect\|panic"` across `memtable.rs`, `lsm/ops/read.rs`, `lsm/scan.rs`. All instances are strictly inside `#[cfg(test)]` modules. | **VERIFIED** |

---

## Lock Hierarchy & Deadlock Analysis

### Documented Lock Hierarchy
1. `commit_mutex` (`tokio::sync::Mutex<()>`)
2. `state` (`tokio::sync::RwLock<LsmState>`)
3. `sstables` (`tokio::sync::RwLock<Vec<Arc<SstableReader>>>`)
4. `wal` (`tokio::sync::RwLock<Arc<Wal>>`)
5. `MemTableShard.entries` (`parking_lot::RwLock<MemTableMap>`)

### Lock Invariants & Deadlock Safety
- Readers acquire `state.read()` and `sstables.read()` independently or sequentially (`state` before `sstables`).
- Flush acquires `commit_mutex`, then `state.write()`, swaps active `MemTable` and `Wal`, releases `commit_mutex` and `state.write()`, builds SSTable asynchronously, then re-acquires `state.write()` and `sstables.write()` to apply atomic transition.
- Readers never attempt to acquire `commit_mutex` or write guards on `state` or `sstables`.
- No circular dependencies exist across lock acquisitions.

---

## Testing & Verification Results
- `cargo test -p contextra-store --test read_path_audit`: PASS (4/4 tests passed)
- `cargo test -p contextra-store --lib`: PASS (all unit tests passed)
- `cargo clippy -p contextra-store --all-targets --all-features -- -D warnings`: PASS
- `cargo fmt --all -- --check`: PASS
