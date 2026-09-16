# ADR-0XX: P0-A Lock-Handoff Invariant for Group-Commit WAL Serialization

## Status
Accepted

## Context
In `memfuse-store`, group commit batching allows multiple concurrent transaction commit requests to be aggregated under a batch leader. Sequence allocation and HMAC state calculation occur sequentially during batch preparation under `commit_mutex`.

However, during physical disk write (`append_batch_locked`), holding `commit_mutex` across disk I/O would stall incoming transactions from preparing their batches. Releasing `commit_mutex` before acquiring the WAL file lock (`truncate_lock`) introduced a race condition: concurrent batch leaders could acquire `commit_mutex`, prepare subsequent batches, and acquire `truncate_lock` out of sequence relative to their HMAC chain derivation.

## Decision
We enforce a strict **Lock-Handoff Invariant** in group commit (`lsm/mod.rs:788–792` and `tests/loom_group_commit.rs`):

```rust
// LOCK-HANDOFF: Acquire truncate_lock BEFORE releasing commit_mutex.
let truncate_guard = wal.truncate_lock.lock().await;
drop(_commit_lock);

let append_res = wal.append_batch_locked(all_wal_entries, &truncate_guard).await;
drop(truncate_guard);
```

By acquiring `truncate_lock` while still holding `commit_mutex`, the physical write order to disk is strictly total-ordered to match the exact sequence and HMAC chain order derived during preparation.

## Consequences
- Prevents HMAC chain corruption and out-of-order physical writes under high parallelism.
- Allows pipeline parallelism where `commit_mutex` is dropped immediately after acquiring `truncate_guard`, unblocking subsequent transactions for in-memory preparation while disk I/O completes.
