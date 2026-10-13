# Contextra Testkit Invariant & Test Matrix

**Stand:** 2026-10-13
**Crate:** `contextra-testkit` (Ring Tooling)
**Status:** AUDITED & VERIFIED

This document establishes the authoritative mapping between testkit invariants, system requirements, and existing verified test implementations.

---

## 1. Invariant to Test Mapping

| Invariante | Beschreibung | Testfunktion | Testdatei | Status |
| :--- | :--- | :--- | :--- | :---: |
| **INV-P28-CLOCK** | Nanosekundengenaue deterministische Uhr mit Typinvariante Monotonie (`set_nanos` nie rückwärts) | `test_manual_clock_set_nanos_monotonic` | `crates/contextra-testkit/src/manual_clock.rs` | ✅ |
| **INV-P28-CLOCK-REWIND** | Explizites Zurücksetzen der Zeitstempel für Testkaskaden via `rewind_for_test` | `test_manual_clock_rewind_for_test` | `crates/contextra-testkit/src/manual_clock.rs` | ✅ |
| **INV-FAULT-VFS-WRITE** | Deterministische I/O Write-Limits und Fehler-Injektion | `test_fault_vfs_write_limit` | `crates/contextra-testkit/src/fault_vfs.rs` | ✅ |
| **INV-FAULT-VFS-TORN** | Injektion von Torn Writes (partielle Schreibvorgänge) | `test_fault_vfs_torn_write` | `crates/contextra-testkit/src/fault_vfs.rs` | ✅ |
| **INV-FAULT-VFS-BITROT** | Injektion von Bitrot (Byte-Flip Corruptions) | `test_fault_vfs_bitrot` | `crates/contextra-testkit/src/fault_vfs.rs` | ✅ |
| **INV-FAULT-VFS-LOST-FSYNC** | Injektion von verlorenen fsyncs (Ok ohne Persistierung) | `test_fault_vfs_sync_failure` | `crates/contextra-testkit/src/fault_vfs.rs` | ✅ |
| **INV-FAULT-VFS-SEED** | Seed-Protokollierung für FaultVfs Injektions-Reproduzierbarkeit | `test_fault_vfs_seed_logging` | `crates/contextra-testkit/src/fault_vfs.rs` | ✅ |
| **INV-REF-MODEL-MVCC** | Deterministisches MVCC-Referenzmodell für Snapshot-Isolationsprüfungen | `test_reference_model_mvcc_isolation` | `crates/contextra-testkit/src/reference_model.rs` | ✅ |
| **INV-DST-DETERMINISM** | Bitidentische Ausführungsreihenfolge pro Seed im DST Framework | `test_dst_bitidentical_op_sequence` | `crates/contextra-testkit/tests/dst_scenarios.rs` | ✅ |
| **INV-DST-TORN-WRITE** | Crash-Recovery-Sicherheit bei Torn Writes im DST Runner | `test_dst_scenario_torn_write` | `crates/contextra-testkit/tests/dst_scenarios.rs` | ✅ |
| **INV-DST-FSYNC-ERROR** | Crash-Recovery-Sicherheit bei fsync-Fehlern im DST Runner | `test_dst_scenario_fsync_error` | `crates/contextra-testkit/tests/dst_scenarios.rs` | ✅ |
| **INV-DST-ENOSPC** | Crash-Recovery-Sicherheit bei ENOSPC im DST Runner | `test_dst_scenario_enospc` | `crates/contextra-testkit/tests/dst_scenarios.rs` | ✅ |
| **INV-DST-P07-CAMPAIGN** | P07 DST Campaign & Seed Multi-Execution Verification | `test_p07_naive_wal_sut_passes_campaign` | `crates/contextra-testkit/tests/p07_dst_runner_test.rs` | ✅ |
| **INV-DST-REGRESSION-SEEDS** | Deterministische Ausführung aller in `tests/regressions/seeds.txt` gelisteten Seeds | `test_p07_regression_seeds_file_parsed` | `crates/contextra-testkit/tests/p07_dst_runner_test.rs` | ✅ |
| **INV-SIM-FS-DETERMINISM** | Deterministische Crash-Resolution in SimFs pro Seed | `test_sim_fs_crash_determinism` | `crates/contextra-testkit/tests/p07_sim_fs_test.rs` | ✅ |
| **INV-CRASH-HARNESS-SEEDS** | Crash-Konsistenz-Harness mit mind. 8 fixen Seeds gegen WAL / Store API | `test_store_wal_crash_harness_fixed_seeds` | `crates/contextra-testkit/tests/store_wal_crash_harness.rs` | ✅ |
| **INV-WAL-PARTIAL-TAIL** | WAL Truncation partieller Frames beim Replay auf Open (Invariante I-3) | `test_store_wal_partial_tail_truncation_recovery` | `crates/contextra-testkit/tests/store_wal_crash_harness.rs` | 🟡 (Blockiert durch F-WAL-01) |

---

## 2. Audit-Berichtigung früherer Phantom-Einträge

Die folgenden in früheren Berichten fälschlicherweise zitierten Test-Dateien und Skripte existieren nicht im Workspace und wurden als **Lücken / Phantom-Claims** identifiziert:

| Zitiertes Phantom-Item | Status | Tatsächlicher Befund |
| :--- | :---: | :--- |
| `xtask/src/check_panic_free.rs` | 🔴 **Lücke** | Datei existiert nicht. Panic-Freiheit wird durch `cargo xtask doctrine-scan` / `check-unsafe-islands` geprüft. |
| `crates/contextra-mvcc/tests/ssi_tests.rs` | 🔴 **Lücke** | Datei existiert nicht. MVCC SSI Logik ist in `crates/contextra-mvcc/src/ssi.rs` abgedeckt. |
| `crates/contextra-store/tests/wal_legacy_key.rs` | 🔴 **Lücke** | Datei existiert nicht unter diesem genauen Namen (ersetzt durch `wal_legacy_key_tests.rs` & `wal_legacy_key_forced_migration.rs`). |

---

## 3. Verifikation der Existenten Symbole

Alle in Tabelle 1 aufgeführten `contextra_testkit::*` Symbole und Funktionen wurden automatisiert per `cargo xtask symbol-exists` und `cargo test -p contextra-testkit` verifiziert.
