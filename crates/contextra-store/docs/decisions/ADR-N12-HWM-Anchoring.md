# ADR-N12: High-Water-Mark Anchoring for WAL Checkpointing and Truncation Detection

## Status
Pending Implementation

## Context
When a MemTable is flushed to an SSTable, the storage engine records a WAL checkpoint in the Manifest representing the latest sequence number persisted to disk (the High-Water-Mark, HWM). If the active WAL is truncated or corrupted prior to this checkpoint, reopening the database must detect this loss of durability and fail closed rather than silently losing committed records.

## Decision
1. Manifest checkpoints will record the WAL HWM along with the corresponding sequence number and HMAC state.
2. During recovery, replay verifies that the WAL contains entries up to and including the recorded HWM. Any truncation before the HWM raises `ContextraError::WalTruncationDetected`.
3. Periodic checkpoints anchor cross-segment continuity across WAL rotations.

## Validation
Specification tests are maintained in `crates/contextra-store/tests/wal_spec_pending.rs` and can be run via:
`cargo test -p contextra-store --test wal_spec_pending -- --ignored`
