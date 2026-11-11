# FROZEN_SURFACE — WAL (Auftrag STORE-MOD-A)

Die folgende Tabelle inventarisiert alle Symbole, Typen, Konstanten, Statics und Module aus `crate::wal::*`, die außerhalb des Eigentums von Agent A referenziert werden. Diese Außenfläche ist strikt eingefroren (Name, Modulpfad, Signatur, Sichtbarkeit, Trait-Bounds, Semantik).

| Item | Art | Export-Pfad | Referenzierende Dateien außerhalb des Eigentums |
|---|---|---|---|
| `Wal` | Struct | `crate::wal::Wal` | `scanner/wal.rs`, `lsm/engine.rs`, `lsm/recovery.rs`, `lsm/group_commit.rs`, `lsm/mod.rs`, `compat/scanner/wal.rs`, `compat/scanner/purge.rs`, diverse Integrationstests |
| `WalOp` | Enum | `crate::wal::WalOp` | `scanner/wal.rs`, `lsm/recovery.rs`, `lsm/group_commit.rs`, `lsm/mod.rs`, `lsm/observer.rs`, `compat/scanner/wal.rs`, `compat/scanner/purge.rs` |
| `WalEntry` | Struct | `crate::wal::WalEntry` | `scanner/wal.rs`, `lsm/commit.rs`, `lsm/observer.rs`, `compat/scanner/wal.rs`, `compat/scanner/purge.rs`, diverse Integrationstests |
| `WalConfig` | Struct | `crate::wal::WalConfig` | `lsm/engine.rs`, diverse Tests |
| `WalVersion` | Enum | `crate::wal::WalVersion` | `lsm/engine.rs`, diverse Tests |
| `PoisonCause` | Enum | `crate::wal::PoisonCause` | `lsm/ops/write.rs`, diverse Tests |
| `PreparedBatch` | Struct | `crate::wal::PreparedBatch` | `lsm/group_commit.rs` |
| `KeyManager` | Re-export | `crate::wal::KeyManager` | `lsm/engine.rs`, `lsm/recovery.rs`, `compaction/engine.rs`, `sstable/builder.rs`, `sstable/reader.rs` |
| `WAL_V2_HEADER` | Const | `crate::wal::WAL_V2_HEADER` | `tests/proof_format_constants.rs` |
| `WAL_V3_HEADER` | Const | `crate::wal::WAL_V3_HEADER` | `scanner/wal.rs`, `compat/scanner/wal.rs`, `compat/scanner/purge.rs`, diverse Tests |
| `MAX_WAL_ENTRY_SIZE` | Const | `crate::wal::MAX_WAL_ENTRY_SIZE` | `tests/wal_frame_limits.rs` |
| `ReplayProgressSink` | Trait | `crate::wal::ReplayProgressSink` | `tests/wal_recovery_runtime_stall.rs` |
| `WalSeq` | Type Alias | `crate::wal::WalSeq` | `tests/wal_recovery_runtime_stall.rs` |
| `FAIL_APPEND_FOR_TX` | Static | `crate::wal::FAIL_APPEND_FOR_TX` | `lsm/engine.rs`, diverse Tests |
| `DELAY_APPEND_FOR_TX`| Static | `crate::wal::DELAY_APPEND_FOR_TX`| `lsm/tests/commit_tests.rs`, diverse Tests |
| `DELAY_APPEND_MS`    | Static | `crate::wal::DELAY_APPEND_MS`    | `lsm/tests/commit_tests.rs`, diverse Tests |
| `FAIL_APPEND_PARTIAL_ONCE` | Static | `crate::wal::FAIL_APPEND_PARTIAL_ONCE` | `tests/wal_torn_write_fault_injection.rs` |
| `FAIL_APPEND_AFTER_PARTIAL_BYTES` | Static | `crate::wal::FAIL_APPEND_AFTER_PARTIAL_BYTES` | `tests/wal_torn_write_fault_injection.rs` |
| `fs` | Modul (Shim)| `crate::wal::fs` | `fsync.rs`, `compat/scanner/wal.rs` |
| `io::do_scan_entries_with_callback` | Fn | `crate::wal::io::do_scan_entries_with_callback` | `src/wal/flusher.rs` |
| `hmac::migrate_legacy_key` | Fn | `crate::wal::hmac::migrate_legacy_key` | `tests/hmac_legacy_key_migration.rs` |
| `hmac::LegacyKeyStatus` | Enum | `crate::wal::hmac::LegacyKeyStatus` | `tests/hmac_legacy_key_migration.rs` |
