# ADR-002: Fail-Closed WAL Poisoning & HMAC Rollback Discipline (INV-FSYNC-1 / WP-STO-3)

## Status
Accepted

## Context
Under POSIX fsync semantics (and modern OS buffer cache behavior, commonly known as the *fsyncgate* vulnerability), when `fsync` or `fdatasync` fails (e.g. `EIO`, disk full, write error), the kernel page cache state for the dirty pages becomes indeterminate. Retrying `fsync` on the same file descriptor can return `Ok`, masking the fact that pages were evicted without reaching stable persistent storage.

Furthermore, in Write-Ahead-Logging (WAL) with cryptographic integrity chains (HMAC-SHA256), failures during appending, file truncation (`set_len`), directory sync, or transaction cancellation rollback can lead to state divergence between in-memory HMAC anchors and on-disk logs. Silent ignore patterns (`let _ =`) on persistence and rollback returns lead to subtle data corruption, torn writes, or panic cascades.

## Decision
1. **Typed Poison Causes (`PoisonCause`)**:
   We introduce `#[non_exhaustive] pub enum PoisonCause` with variants:
   - `FsyncFailed`: An `fsync` (`sync_all` or parent directory fsync) failed.
   - `WriteFailed`: A physical write or flush to disk failed.
   - `TruncateFailed`: File truncation (`set_len`) failed.
   - `HmacChainDiverged`: In-memory HMAC rollback could not be applied synchronously (e.g., lock contention or divergence).
   - `TornWrite`: Suspected incomplete write at log tail.

2. **Zero Hot-Path Allocation & First-Write-Wins**:
   The poison state is stored in an `Arc<AtomicU8>` field (`poison_state`). The first error code wins via `compare_exchange(0, cause, SeqCst, SeqCst)`. Subsequent errors do not overwrite the primary root cause.

3. **Irreversibility Discipline (SPEC §3.5, §12.1)**:
   Poisoning caused by `FsyncFailed`, `WriteFailed`, `TruncateFailed`, or `HmacChainDiverged` is **permanent and irreversible** for that handle instance.
   `recover_from_poison()` strictly inspects `poison_cause()`:
   - If `cause != PoisonCause::TornWrite`, recovery is unconditionally rejected with an error.
   - Only `PoisonCause::TornWrite` may be healed via offline replay against the verified HMAC chain prefix, truncating orphan tail bytes.
   - Healing non-torn-write errors requires closing the handle and performing a full cold database restart/recovery replay.

4. **Drop-Guard Hardening (`SingleCommitGuard`, `LeaderCancelGuard`)**:
   `wal.rollback_hmac_sync(prev_hmac)` returns `bool` (non-blocking `try_lock`). If it returns `false`, both guards immediately poison the handle with `PoisonCause::HmacChainDiverged`. `LeaderCancelGuard` additionally fails all queued follower requests with an error. No `let _ =` is permitted on persistence or chain rollback return values.

5. **Flusher Fail-Closed Safety**:
   In `wal/flusher.rs`, errors during `write_all`, `sync_all`, `set_len`, parent directory sync, or migration rewrite immediately poison the handle with their typed causes and return `Err` to all pending acknowledgments.

6. **INV-LOG-1 Compliance**:
   Neither error strings nor log messages contain any user payload data.

## Consequences
- Guaranteed fail-closed behavior on storage hardware anomalies and OS fsync failures.
- Prevents silent data corruption from retrying failed fsync operations.
- Clean separation between recoverable partial write tails (`TornWrite`) and unrecoverable I/O / chain divergence faults.
