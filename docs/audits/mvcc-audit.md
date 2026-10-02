# Audit Report: `contextra-mvcc`

**Date**: 2026-09-28
**Crate**: `contextra-mvcc` (`src/ssi.rs`, `src/snapshot.rs`, `src/tx_buffer.rs`, `src/seq_log.rs`, `src/lib.rs`)
**Target Environment**: Rust 1.89.0, Ring 0 isolation (`#![forbid(unsafe_code)]`)
**Auditor**: Jules (Senior Software Engineer)

---

## 1. Executive Summary

A comprehensive architectural and implementation audit of `contextra-mvcc` was conducted to evaluate Serializable Snapshot Isolation (SSI), `SnapshotRegistry`, `TxBuffer`, and `SequenceLog`. Two critical bugs causing data corruption and false negative validation were identified, proven via red regression tests, and resolved:

1. **Bug 1 (`ssi.rs` / `forget_from`)**: `CommittedWrites::insert` previously destroyed historical exact key bucket entries when a key was re-committed at a higher sequence. When `forget_from(first_seq)` rolled back the uncommitted/failed transaction at `new_seq >= first_seq`, the key was wiped from `self.keys` completely, losing its valid earlier commit history at `old_seq < first_seq`. Consequently, subsequent reads at `snapshot_seq < old_seq` falsely passed validation (False Negative -> Data Corruption).
2. **Bug 2 (`ssi.rs`, `seq_log.rs`, `tx_buffer.rs` / `TOMBSTONE_BIT` Unmasked Comparisons)**: Raw sequence numbers carrying `TOMBSTONE_BIT` (0x8000_0000_0000_0000) passed into `ReadSet::record_read`, `SequenceLogSsiValidator`, `TxBuffer::register_read`, or `SeqLogEntry::is_visible` resulted in raw sequence values ~9.22 quintillion. In `validate`, `commit_seq > snapshot_seq` checks failed, missing real write conflicts (False Negative). In `TxBuffer::min_read_snapshot`, the minimum snapshot was reported as ~9.22 quintillion, causing `prune_through` to over-prune committed key records.

Both bugs were fixed without modifying public API signatures or breaking existing contracts. All 45 unit tests and 10 integration test files pass cleanly.

---

## 2. Component Audits

### 2.1 SSI Validator (`src/ssi.rs`)

- **ReadSet & Phantom Protection**: `ReadSet` tracks point read keys and range prefixes alongside their lowest snapshot sequence numbers. Range scans explicitly record prefixes via `record_prefix` to guard against phantoms.
- **Conflict Validation Semantics**: Point read keys and range prefixes are validated against `CommittedWrites` and optional `SequenceLog`. Validation is 100% deterministic and data-driven (`commit_seq > snapshot_seq`).
- **Coarsening Soundness (B-16)**: Under high commit volume (threshold >= 80% of `max_tracked_keys`), older exact buckets are coarsened into Longest Common Prefix (LCP) summaries. Coarsening guarantees 0% false negatives: every original key $k$ in a chunk starts with extracted prefix $p$.
- **Finding 1 (Fixed - `forget_from` History Restoration)**:
  - *File/Line*: `crates/contextra-mvcc/src/ssi.rs:274-285` (former), `ssi.rs:383-424` (fixed).
  - *Root Cause*: `CommittedWrites::insert` used `keys.retain(|k| k != &key)` to strip `key` from `seq_index[old_seq]`. On rollback via `forget_from(first_seq)`, `remove_from_seq` removed `key` from `self.keys` without restoring `old_seq`.
  - *Fix*: `insert` preserves `key` in `seq_index[old_seq]`. On `forget_from(first_seq)`, if `self.keys[key] >= first_seq`, `remove_from_seq` reverse-scans `seq_index` for `old_seq < first_seq` containing `key`, restoring `self.keys[key] = old_seq` if present.
- **Finding 2 (Fixed - `TOMBSTONE_BIT` Masking)**:
  - *File/Line*: `crates/contextra-mvcc/src/ssi.rs:66,88,276,384,428,541,553,652,664,825`.
  - *Root Cause*: `snapshot_seq` and `commit_seq` were processed raw without masking `TOMBSTONE_BIT`.
  - *Fix*: Explicitly masked `seq & !TOMBSTONE_BIT` across all `ReadSet` and `SequenceLogSsiValidator` entry points.

### 2.2 Snapshot Registry (`src/snapshot.rs`)

- **Concurrency & Memory Ordering**: Modifications to `active: Mutex<BTreeMap<u64, Vec<Instant>>>` are serialized. `min_active_seqno` is exposed via lock-free `AtomicU64` reads using `Ordering::Acquire` matching `Ordering::Release` writes in `update_min`.
- **Drop Semantics & Ref-Counting**: RAII `SnapshotGuard` deregisters on `drop()`. Double registrations increment ref-counts, and sequential/out-of-order drops decrement correctly without leaking or underflowing.
- **Tombstone Bit Handling**: `register_at`, `pin_at`, and `release_at` strip `TOMBSTONE_BIT` consistently.

### 2.3 TxBuffer (`src/tx_buffer.rs`)

- **Sharded Staging**: Buffer is partitioned into sub-buffers (`DEFAULT_SHARD_COUNT = 64`) indexed by `tx.inner() % shard_count`.
- **Deadlock Prevention**: Callers must never acquire multiple shard locks concurrently. Multi-shard sweeps (`reap_orphans`, `min_read_snapshot`) lock shards sequentially in ascending order (0..N-1) using `try_write()` or releasing read locks per shard.
- **Memory & Staging Caps**: Staging enforces `max_ops_per_tx` (default 10,000 ops), `max_tx_staged_bytes` (16 MiB), and `max_total_staged_bytes` (256 MiB).
- **ReadSet & Staging Lifecycle**: `drain_kv`, `discard_kv`, and `reap_orphans` atomically purge operations and associated `ReadSet` data, freeing global staged byte budgets.

### 2.4 Sequence Log (`src/seq_log.rs`)

- **Visibility Rule**: `SeqLogEntry::is_visible(as_of)` evaluates `insert_seq <= as_of && (delete_seq.is_none() || delete_seq > as_of)`.
- **Compaction & Retention**: Soft-deleted entries below `min_active_seqno` are purged via `compact`.
- **Finding 2 (Fixed - `TOMBSTONE_BIT` Masking)**:
  - *File/Line*: `crates/contextra-mvcc/src/seq_log.rs:42,135,141,214,232,245,265,295`.
  - *Fix*: Masked `TOMBSTONE_BIT` across `SeqLogEntry::is_visible` and `SequenceLog` methods (`record_insert`, `record_delete`, `is_visible`, `compact`, `changes_since`, `pin_snapshot_at`, `unpin_snapshot`).

---

## 3. Invariants Assumed by `contextra-store`

1. **Commit Serialization**: `contextra-store` callers must serialize `validate` and `record_commit_key` operations (e.g., under `commit_mutex`) to ensure deterministic `commit_seq` assignment and eliminate commit races.
2. **Pruning Watermark Computation**: `contextra-store` compaction workers compute safe SSI pruning bounds as `min(min_read_snapshot(), min_active_seqno())`.
3. **Rollback Discipline**: When a WAL write or transaction commit fails after candidate keys are registered, `contextra-store` must call `ssi_validator.forget_from(first_seq)` to roll back registered candidates while restoring earlier valid commit states.

---

## 4. Verification & Testing

- **Regression Tests (`crates/contextra-mvcc/tests/audit_regression_tests.rs`)**:
  - `test_forget_from_restores_earlier_commit_history`: Verified red run prior to fix (panicked expecting conflict), green after fix.
  - `test_tombstone_bit_masking_in_read_set_and_validator`: Verified red run prior to fix (`min_snapshot_seq` was ~9.22E18), green after fix.
  - `test_tombstone_bit_masking_in_tx_buffer`: Verified red run prior to fix (`min_read_snapshot` was ~9.22E18), green after fix.
  - `test_tombstone_bit_masking_in_seq_log`: Verified red run prior to fix (entry invisible at snapshot 15), green after fix.
- **Full Test Suite Execution**:
  - `cargo test -p contextra-mvcc`: 45 unit tests, 10 integration test files (including proptests and loom tests) passed cleanly.
  - `cargo clippy -p contextra-mvcc --all-targets --all-features -- -D warnings`: Clean, 0 warnings.
  - `cargo fmt --all -- --check`: Clean formatting.

---

## 5. Residual Risks

- **Coarsening False Positives**: Under extreme write volume where `max_tracked_keys` threshold is reached, coarsening exact keys into prefix summaries may cause false positive conflict rejections for concurrent transactions touching unrelated keys sharing the prefix. This is fail-closed by design and preserves 100% data safety.
