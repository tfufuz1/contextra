# Audit Report: `contextra-checkpoint`

**Datum:** 2026-09-27
**Crate:** `crates/contextra-checkpoint`
**Ring:** Ring 1
**Safety:** `#![forbid(unsafe_code)]`

---

## (1) P1–P6 Summary Table

| Prüfpunkt | Bezeichnung | Status | Befund / Details |
|---|---|---|---|
| **P1** | `INV-CHECKPOINT-DETERMINISM-1` | ❌ **FAIL** | Direct `SystemTime::now()` calls exist in `guard.rs` (lines 52, 128, 145) and `orphan.rs` (lines 45, 84). Clock port from `contextra_ports::Clock` is NOT injected into `CheckpointGuard` or `PersistentCheckpointStore`. Commit `647276c5` referenced in audit prompt does not exist in repository git history. |
| **P2** | RAII-Guard-Vollständigkeit | ⚠️ **PARTIAL** | `CheckpointGuard` and `PinGuard` can only be explicitly consumed via `commit()` or `rollback().await` / `unpin().await`. On `Drop` without explicit commit/rollback, both guards synchronously register themselves into `InstanceOrphanRegistry`. However, `InstanceOrphanRegistry::register_*_sync` updates in-memory `Mutex` state without immediate sync disk I/O, meaning an ungraceful crash/panic prior to the next piggyback flush could lose the orphan record. |
| **P3** | Orphan-Reaper | ⚠️ **PARTIAL** | No active async background reaper task is spawned during `Drop` or runtime. Orphan recovery is executed synchronously on startup (`PersistentCheckpointStore::open` / `recover_orphaned_pins` / `recover_orphaned_checkpoints`) and during manual calls. `await_pending_rollbacks()` and `pending_rollback_count()` are deprecated no-ops. |
| **P4** | Snapshot-Pinning / Unpinning | ✅ **PASS** | `PersistentCheckpointStore::drop_checkpoint` explicitly calls `storage.unpin_checkpoint(seq_no)` after deleting the checkpoint from storage. During `create_checkpoint`, replacing an existing checkpoint unpins the old `seq_no`. |
| **P5** | Blake3-Manifest-Verifikation | ⚠️ **FAIL (BLOCKER IN LIST)** | `CheckpointManifest` calculates a BLAKE3 checksum over `(meta, components)` during `new()` and validates it via `verify()`. On tampering/corruption, `verify()` returns `ContextraError::Serialization(...)`. `get_checkpoint()` and `list_checkpoints()` call `verify()` on JSON-parsed manifests. However, `list_checkpoints()` calls `storage.scan_prefix()` which includes uncommitted LSM entries, causing uncommitted checkpoints to leak into `list_checkpoints()` and fail recovery tests (`checkpoint_systematic_crash.rs`). |
| **P6** | TxId-Bereich-Separation | ✅ **PASS** | `PersistentCheckpointStore::allocate_tx()` allocates TxIds starting from `TxId::INTERNAL_BASE` (`1 << 62`). User transaction TxIds operate below `INTERNAL_BASE`. |

---

## (2) RAII-Guard-Lifecycle-Nachweis

### `CheckpointGuard<S>` Lifecycle Analysis
1. **Creation**: Created via `CheckpointGuard::new(...)`, `for_agent_step(...)`, or `PersistentCheckpointStore::create_guard(tx)`.
2. **Commit (`commit(self)`)**:
   - Takes ownership (`self`).
   - Extracts `checkpoint.take()`, preventing `Drop` handling.
   - Triggers piggyback flush on Tokio handle if available.
   - Returns `StateCheckpoint`.
3. **Rollback (`rollback(mut self)`)**:
   - Takes ownership (`self`).
   - Extracts `checkpoint.take()`.
   - Validates serialization barrier (`last_tx <= cp.tx_id`). Returns error if newer committed transaction exists.
   - Executes `storage.rollback_to_tx(cp.tx_id)`.
4. **Drop (`Drop::drop(&mut self)`)**:
   - Executed if guard is dropped without calling `commit()` or `rollback()`.
   - If `self.checkpoint` is `Some(...)`, increments `skipped_rollbacks` counter and calls `self.orphan_registry.register_checkpoint_sync(cp)`.
   - **Silent Drop Safety**: Guard cannot be dropped silently without orphan registration as long as `checkpoint` is `Some`.

### `PinGuard<S>` Lifecycle Analysis
1. **Pin (`PinGuard::pin(...)`)**: Calls `storage.pin_checkpoint(seq_no)`.
2. **Defuse (`defuse(mut self)`)**: Clears `self.seq_no`, keeping sequence number pinned permanently.
3. **Unpin (`unpin(mut self)`)**: Calls `storage.unpin_checkpoint(seq_no)` and triggers `orphan_registry.flush_orphan_registry()`.
4. **Drop (`Drop::drop(&mut self)`)**: If `self.seq_no` is `Some`, calls `orphan_registry.register_orphan_sync(PinnedSeqNoOrphan { seq_no, timestamp_ms })`.

---

## (3) Orphan-Reaper-Implementierungsstatus

### Background Reaper vs. Startup / Piggyback Recovery
- **Background Task Spawning**: Removed per ADR-053 and Ring 0/1 async safety rules. No background thread or tokio task is spawned to monitor or reaper orphaned checkpoints/pins in the background.
- **Startup Recovery**:
  - During `PersistentCheckpointStore::open_with_orphan_registry`, all orphan sequence pins in `InstanceOrphanRegistry` are iterated and `storage.unpin_checkpoint(seq_no)` is invoked.
  - `PersistentCheckpointStore::recover_orphaned_checkpoints()` iterates orphaned checkpoints, checks serialization barriers against `storage.last_tx_id()`, and invokes `storage.rollback_to_tx(cp.tx_id)` if safe.
- **Piggyback Flushes**:
  - `InstanceOrphanRegistry` accumulates dirty orphan records in memory (`Mutex<Vec<...>>`).
  - Piggyback flushes are triggered during explicit lifecycle events: `commit()`, `rollback()`, `unpin()`, `shutdown()`, and `close()`.

---

## (4) Blake3-Verifikations-Nachweis

### Implementation
In `crates/contextra-checkpoint/src/manifest.rs`:
```rust
pub struct CheckpointManifest {
    pub meta: CheckpointMeta,
    pub components: Vec<String>,
    pub checksum: String,
}

impl CheckpointManifest {
    pub fn new(meta: CheckpointMeta, components: Vec<String>) -> Result<Self> {
        let payload = serde_json::to_vec(&(&meta, &components))
            .map_err(|e| ContextraError::Serialization(e.to_string()))?;
        let checksum = blake3::hash(&payload).to_hex().to_string();
        Ok(Self { meta, components, checksum })
    }

    pub fn verify(&self) -> Result<()> {
        let payload = serde_json::to_vec(&(&self.meta, &self.components))
            .map_err(|e| ContextraError::Serialization(e.to_string()))?;
        let expected = blake3::hash(&payload).to_hex().to_string();
        if self.checksum != expected {
            return Err(ContextraError::Serialization(format!(
                "Checkpoint manifest checksum mismatch for '{}': expected {}, got {}",
                self.meta.name, expected, self.checksum
            )));
        }
        Ok(())
    }
}
```

### Verification Flow & Corruption Handling
- `get_checkpoint_internal()` reads storage bytes and deserializes as `CheckpointManifest`. It calls `manifest.verify()`. If corrupted/tampered, `verify()` returns `Err(ContextraError::Serialization(...))`.
- `list_checkpoints()` scans storage and calls `manifest.verify()` on every manifest record.
- **Unit Test Coverage**: `checkpoint_manifest_CASE_tampered_manifest_fails_verify` in `manifest.rs` confirms that modifying `manifest.components` causes `verify()` to fail with `ContextraError::Serialization`.

---

## (5) Test & Clippy Results

### Test Log Summary
```
cargo test -p contextra-checkpoint --locked -- --nocapture
Unit tests (src/lib.rs): 54 passed; 0 failed
Integration tests (tests/cache_concurrency_pinning.rs): 3 passed; 0 failed
Integration tests (tests/checkpoint_systematic_crash.rs): 1 failed

Failure Detail:
test systematic_crash_at_every_checkpoint_io_point ... FAILED
panicked at 'BLOCKER FINDING: Uncommitted checkpoint 'chk_systematic' was incorrectly exposed in list_checkpoints at crash_point 0'
```

### Blocker Finding Analysis
Uncommitted checkpoint metadata stored during an interrupted `create_checkpoint` operation is scanned by `storage.scan_prefix()` in `list_checkpoints()`. `scan_prefix()` does not filter entries by `last_committed_tx`, leaking uncommitted checkpoints into `list_checkpoints()` output and populating the in-memory cache.

### Clippy Log Summary
`cargo clippy -p contextra-checkpoint` blocked by pre-existing clippy errors in dependency `contextra-ports` (`crates/contextra-ports/src/plugin.rs` uses `.expect()` on `Option` under `-D warnings`).

---

## (6) VERDICT & SIGN-OFF

**VERDICT: REJECTED / NEEDS REVISION**

### Key Recommendations for Remediation
1. **P1 (INV-CHECKPOINT-DETERMINISM-1)**: Inject `Arc<dyn contextra_ports::Clock>` into `PersistentCheckpointStore` and `CheckpointGuard` to eliminate direct `SystemTime::now()` calls in non-test code.
2. **Systematic Crash / Uncommitted Manifest Leak**: Update `list_checkpoints()` or `LsmStorage::scan_prefix` to filter scanned entries against `last_committed_tx` so uncommitted checkpoint manifests are not exposed.
3. **Clippy in `contextra-ports`**: Fix `expect()` usage in `contextra-ports/src/plugin.rs` so workspace-wide clippy checks pass cleanly.

**VERIFIED-BY-SESSION: PENDING (TS: 2026-09-27T20:20:00Z)**
