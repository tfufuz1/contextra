# Audit-Report — `memfuse-store`

**Auditor:** Jules (Senior Storage Engine Engineer)
**Datum:** 2026-09-15
**Crate:** `memfuse-store` (Layer 3 — LSM Storage Engine & WAL)
**Session-ID:** `f0e53581`
**Task-ID:** `JULES-20260915-MEMFUSESTO-TEST-BC0N`

---

## 1. Summary & Findings

### VERDIKT: **GO (STABLE & CORRECT)**

During test suite verification of `memfuse-store`, a race condition test (`test_truncate_size_visible_atomically_with_file_state`) revealed a TOCTOU ordering bug in `WalCommand::Truncate` (`crates/memfuse-store/src/wal/flusher.rs`).

#### Details of TOCTOU Bug & Fix:
- **Problem:** In `flusher.rs` (`WalCommand::Truncate`), `file.set_len(offset).await` was executed before `size.store(offset, Ordering::SeqCst)`. Because `set_len()` involves an async filesystem operation, concurrent readers calling `wal.size()` could observe an in-memory size larger than the physical truncated file length on disk prior to atomic size store completion.
- **Fix:** In `crates/memfuse-store/src/wal/flusher.rs`, `size.store(offset, Ordering::SeqCst)` is updated prior to `file.set_len(offset).await`. If `file.set_len()` fails, `size` is restored to `old_size`.
- **Verification:** `cargo test -p memfuse-store --lib` (including `test_truncate_size_visible_atomically_with_file_state`) and `cargo test -p memfuse-store --features fault-injection --test wal_truncate_ordering` passed 100%.

---

## 2. Inventar-Realitätsabgleich (Stand 2026-09-15)

Actual Rust source file count in `crates/memfuse-store/src`: **33 files**.
Prompter-Inventar baseline (2026-09-10 snapshot): 10 core modules.
Modularity refactoring in `lsm/` and `wal/` into submodules (`lsm/commit.rs`, `lsm/flush.rs`, `lsm/recovery.rs`, `lsm/scan.rs`, `wal/encode.rs`, `wal/flusher.rs`, `wal/hmac.rs`, `wal/io.rs`, `wal/replay.rs`, `manifest.rs`, `system_pressure.rs`, etc.) verified and fully compatible.

---

## 3. Test Matrix Execution

- `cargo test -p memfuse-store --lib`: **PASSED** (198 tests passed).
- `cargo test -p memfuse-store --features fault-injection --test wal_truncate_ordering`: **PASSED** (6 tests passed).
