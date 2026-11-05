# Contextra Voll-Audit

- Commit: `5e26c5e84` (2026-11-04 15:01:55 +0100), Branch `main`, hinter origin: 0, ungespeicherte Aenderungen: True
- Start: 2026-11-04T14:12:48+00:00  Dauer: 18.2 min
- Toolchain-Pin: `1.97.0`; Preflight: **PASS** (cargo 1.97.0 (c980f4866 2026-06-30) | Pin 1.97.0)
- Host: Linux-7.2.7-1-liquorix-amd64-x86_64-with-glibc2.41 | 8 CPUs | MemTotal:       10121012 kB
- Nutzer: freddy

## Gesamt

- **FAIL**: 26
- **INFO**: 4
- **PASS**: 70
- **PENDING**: 92

Hinweis: Bei `befund_*`-Schritten bedeutet FAIL, dass der Befund **vorliegt**. `INFO` ist eine Messung oder ein Hinweis ohne Urteil.

## Rote und uebersprungene Schritte

| Schritt | Status | Sek | Detail | Log |
|---|---|---:|---|---|
| `check_alle_targets` | FAIL | 28.5 | errors=19 | `logs/009_check_alle_targets.log` |
| `check_no_default_features` | FAIL | 6.6 | errors=14 | `logs/010_check_no_default_features.log` |
| `doc` | FAIL | 8.3 |  | `logs/011_doc.log` |
| `clippy_lib__contextra` | FAIL | 19.0 | errors=107 warnings=1 | `logs/012_clippy_lib__contextra.log` |
| `clippy_alle_targets__contextra` | FAIL | 15.5 | errors=107 warnings=1 | `logs/013_clippy_alle_targets__contextra.log` |
| `clippy_lib__contextra-engine` | FAIL | 7.0 | errors=107 warnings=1 | `logs/020_clippy_lib__contextra-engine.log` |
| `clippy_alle_targets__contextra-engine` | FAIL | 18.2 | errors=107 warnings=1 | `logs/021_clippy_alle_targets__contextra-engine.log` |
| `clippy_lib__contextra-store` | FAIL | 13.5 | errors=105 warnings=0 | `logs/034_clippy_lib__contextra-store.log` |
| `clippy_alle_targets__contextra-store` | FAIL | 11.9 | errors=106 warnings=0 | `logs/035_clippy_alle_targets__contextra-store.log` |
| `clippy_lib__contextra-vector` | FAIL | 5.8 | errors=2 warnings=0 | `logs/046_clippy_lib__contextra-vector.log` |
| `clippy_alle_targets__contextra-vector` | FAIL | 12.6 | errors=103 warnings=1 | `logs/047_clippy_alle_targets__contextra-vector.log` |
| `feature_matrix_hack` | FAIL | 66.8 |  | `logs/050_feature_matrix_hack.log` |
| `panic_flaeche_lib` | FAIL | 17.5 | errors=2 warnings=105 | `logs/053_panic_flaeche_lib.log` |
| `test_lib__contextra` | FAIL | 5.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/055_test_lib__contextra.log` |
| `test_integration__contextra` | FAIL | 1.3 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/056_test_integration__contextra.log` |
| `test_doc__contextra` | FAIL | 1.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/057_test_doc__contextra.log` |
| `nextest__contextra` | FAIL | 1.9 | nextest: keine Summary-Zeile gefunden | `logs/058_nextest__contextra.log` |
| `test_integration__contextra-arch-tests` | FAIL | 3.3 | passed=48 failed=4 ignored=0 | FAILED: test_t_arch_3b_matrix_edges, test_t_arch_5_crate_headers_and_descriptions, test_t_arch_6_exports_and_dev_dependencies, test_t_trace_1_test_traceability | `logs/060_test_integration__contextra-arch-tests.log` |
| `nextest__contextra-arch-tests` | FAIL | 6.8 | passed=48 failed=4 | FAILED: (38/52) contextra-arch-tests::arch test_t_arch_5_crate_headers_and_descriptions, (40/52) contextra-arch-tests::arch test_t_arch_3b_matrix_edges, (45/52) contextra-arch-tests::arch test_t_trace_1_test_traceability, (47/52) contextra-arch-tests::arch test_t_arch_6_exports_and_dev_dependencies | `logs/062_nextest__contextra-arch-tests.log` |
| `test_integration__contextra-durable-fs` | FAIL | 3.9 | passed=31 failed=3 ignored=0 | FAILED: test_df_05_dir_lock_exclusive_same_process, test_df_05_dir_lock_permissions, test_t_lock_1_dir_lock_exclusive_subprocess | `logs/068_test_integration__contextra-durable-fs.log` |
| `nextest__contextra-durable-fs` | FAIL | 0.8 | passed=31 failed=3 | FAILED: (27/34) contextra-durable-fs::df_lock_tests test_df_05_dir_lock_exclusive_same_process, (28/34) contextra-durable-fs::df_lock_tests test_t_lock_1_dir_lock_exclusive_subprocess, (33/34) contextra-durable-fs::df_lock_tests test_df_05_dir_lock_permissions | `logs/070_nextest__contextra-durable-fs.log` |
| `test_lib__contextra-engine` | FAIL | 9.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/071_test_lib__contextra-engine.log` |
| `test_integration__contextra-engine` | FAIL | 1.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/072_test_integration__contextra-engine.log` |
| `test_doc__contextra-engine` | FAIL | 1.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/073_test_doc__contextra-engine.log` |
| `nextest__contextra-engine` | FAIL | 5.6 | nextest: keine Summary-Zeile gefunden | `logs/074_nextest__contextra-engine.log` |
| `test_lib__contextra-store` | FAIL | 571.3 | passed=255 failed=2 ignored=0 | FAILED: fsync::tests::test_sync_file_and_parent_dir_unix, system_pressure::tests::test_blocking_pool_load_triggers_pressure | `logs/099_test_lib__contextra-store.log` |

## Alle Schritte

| Gruppe | Schritt | Status | Sek | Detail |
|---|---|---|---:|---|
| B-befunde | `befund_cfg_features_undefiniert` | PASS | 0.2 | keine undefinierten Features |
| B-befunde | `befund_default_members` | INFO | 0.0 | members=19 default-members=16; nicht in default-members (von `cargo test` ohne --workspace nicht erfasst): ['contextra-lab', 'contextra-testmacros', 'ctx-verify'] |
| B-befunde | `befund_maturity_claims` | INFO | 0.0 | stable=9 beta=6 Eintraege ohne Crate=0 |
| B-befunde | `befund_bench_fuzz_dateien` | PASS | 0.0 | 0 fehlende Dateien/Verzeichnisse |
| 0-basis | `toolchain_preflight` | PASS | 0.1 | cargo 1.97.0 (c980f4866 2026-06-30) | Pin 1.97.0 |
| 0-basis | `toolchain_versionen` | INFO | 0.4 |  |
| 0-basis | `cargo_fetch` | PASS | 0.6 |  |
| 0-basis | `cargo_tree_duplicates` | INFO | 0.3 |  |
| 1-statisch | `fmt_check` | PASS | 6.1 |  |
| 1-statisch | `check_alle_targets` | FAIL | 28.5 | errors=19 |
| 1-statisch | `check_no_default_features` | FAIL | 6.6 | errors=14 |
| 1-statisch | `doc` | FAIL | 8.3 |  |
| 1-statisch | `clippy_lib__contextra` | FAIL | 19.0 | errors=107 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra` | FAIL | 15.5 | errors=107 warnings=1 |
| 1-statisch | `clippy_lib__contextra-arch-tests` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-arch-tests` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-crypto` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-crypto` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-durable-fs` | PASS | 1.2 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-durable-fs` | PASS | 3.2 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-engine` | FAIL | 7.0 | errors=107 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-engine` | FAIL | 18.2 | errors=107 warnings=1 |
| 1-statisch | `clippy_lib__contextra-erasure` | PASS | 2.0 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-erasure` | PASS | 2.7 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-lab` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-lab` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-mvcc` | PASS | 3.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-mvcc` | PASS | 8.6 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-ports` | PASS | 0.9 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-ports` | PASS | 1.4 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-rank` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-rank` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-simd` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-simd` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-store` | FAIL | 13.5 | errors=105 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-store` | FAIL | 11.9 | errors=106 warnings=0 |
| 1-statisch | `clippy_lib__contextra-sys` | PASS | 0.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-sys` | PASS | 0.7 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-testkit` | PASS | 2.2 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-testkit` | PASS | 6.6 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-testmacros` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-testmacros` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-text` | PASS | 2.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-text` | PASS | 7.8 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-types` | PASS | 0.2 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-types` | PASS | 1.6 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-vector` | FAIL | 5.8 | errors=2 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-vector` | FAIL | 12.6 | errors=103 warnings=1 |
| 1-statisch | `clippy_lib__ctx-verify` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__ctx-verify` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `feature_matrix_hack` | FAIL | 66.8 |  |
| 1-statisch | `deny` | PASS | 4.4 |  |
| 1-statisch | `audit` | PASS | 1.4 |  |
| 1-statisch | `panic_flaeche_lib` | FAIL | 17.5 | errors=2 warnings=105 |
| 1-statisch | `unsafe_inventar` | PASS | 0.1 | keine Verstoesse |
| 2-tests | `test_lib__contextra` | FAIL | 5.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra` | FAIL | 1.3 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra` | FAIL | 1.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `nextest__contextra` | FAIL | 1.9 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-arch-tests` | PASS | 0.3 | passed=32 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-arch-tests` | FAIL | 3.3 | passed=48 failed=4 ignored=0 | FAILED: test_t_arch_3b_matrix_edges, test_t_arch_5_crate_headers_and_descriptions, test_t_arch_6_exports_and_dev_dependencies, test_t_trace_1_test_traceability |
| 2-tests | `test_doc__contextra-arch-tests` | PASS | 0.3 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-arch-tests` | FAIL | 6.8 | passed=48 failed=4 | FAILED: (38/52) contextra-arch-tests::arch test_t_arch_5_crate_headers_and_descriptions, (40/52) contextra-arch-tests::arch test_t_arch_3b_matrix_edges, (45/52) contextra-arch-tests::arch test_t_trace_1_test_traceability, (47/52) contextra-arch-tests::arch test_t_arch_6_exports_and_dev_dependencies |
| 2-tests | `test_lib__contextra-crypto` | PASS | 0.3 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-crypto` | PASS | 35.8 | passed=42 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-crypto` | PASS | 0.8 | passed=1 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-crypto` | PASS | 27.9 | passed=42 failed=0 |
| 2-tests | `test_lib__contextra-durable-fs` | PASS | 2.7 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-durable-fs` | FAIL | 3.9 | passed=31 failed=3 ignored=0 | FAILED: test_df_05_dir_lock_exclusive_same_process, test_df_05_dir_lock_permissions, test_t_lock_1_dir_lock_exclusive_subprocess |
| 2-tests | `test_doc__contextra-durable-fs` | PASS | 1.0 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-durable-fs` | FAIL | 0.8 | passed=31 failed=3 | FAILED: (27/34) contextra-durable-fs::df_lock_tests test_df_05_dir_lock_exclusive_same_process, (28/34) contextra-durable-fs::df_lock_tests test_t_lock_1_dir_lock_exclusive_subprocess, (33/34) contextra-durable-fs::df_lock_tests test_df_05_dir_lock_permissions |
| 2-tests | `test_lib__contextra-engine` | FAIL | 9.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-engine` | FAIL | 1.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-engine` | FAIL | 1.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `nextest__contextra-engine` | FAIL | 5.6 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-erasure` | PASS | 1.7 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-erasure` | PASS | 4.5 | passed=43 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-erasure` | PASS | 0.5 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-erasure` | PASS | 0.7 | passed=43 failed=0 |
| 2-tests | `test_lib__contextra-lab` | PASS | 0.3 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-lab` | PASS | 0.7 | passed=28 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-lab` | PASS | 2.3 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-lab` | PASS | 0.7 | passed=28 failed=0 |
| 2-tests | `test_lib__contextra-mvcc` | PASS | 3.2 | passed=52 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-mvcc` | PASS | 29.1 | passed=151 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-mvcc` | PASS | 0.4 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-mvcc` | PASS | 16.8 | passed=151 failed=0 |
| 2-tests | `test_lib__contextra-ports` | PASS | 1.5 | passed=28 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-ports` | PASS | 1.1 | passed=36 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-ports` | PASS | 0.4 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-ports` | PASS | 0.7 | passed=36 failed=0 |
| 2-tests | `test_lib__contextra-rank` | PASS | 0.3 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-rank` | PASS | 0.9 | passed=19 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-rank` | PASS | 1.1 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-rank` | PASS | 4.4 | passed=19 failed=0 |
| 2-tests | `test_lib__contextra-simd` | PASS | 0.3 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-simd` | PASS | 17.1 | passed=56 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-simd` | PASS | 1.2 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-simd` | PASS | 22.5 | passed=56 failed=0 |
| 2-tests | `test_lib__contextra-store` | FAIL | 571.3 | passed=255 failed=2 ignored=0 | FAILED: fsync::tests::test_sync_file_and_parent_dir_unix, system_pressure::tests::test_blocking_pool_load_triggers_pressure |
| 2-tests | `test_integration__contextra-store` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-store` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-store` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-sys` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-sys` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-sys` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-sys` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-testkit` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-testkit` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-testkit` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-testkit` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-testmacros` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-testmacros` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-testmacros` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-testmacros` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-text` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-text` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-text` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-text` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-types` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-types` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-types` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-types` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-vector` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-vector` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-vector` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-vector` | PENDING | 0.0 |  |
| 2-tests | `test_lib__ctx-verify` | PENDING | 0.0 |  |
| 2-tests | `test_integration__ctx-verify` | PENDING | 0.0 |  |
| 2-tests | `test_doc__ctx-verify` | PENDING | 0.0 |  |
| 2-tests | `nextest__ctx-verify` | PENDING | 0.0 |  |
| 2-tests | `test_gesamt_debug` | PENDING | 0.0 |  |
| 2-tests | `test_gesamt_release` | PENDING | 0.0 |  |
| 3-testbarkeit | `testbarkeit_crates` | PENDING | 0.0 |  |
| 3-testbarkeit | `testbarkeit_funktionen` | PENDING | 0.0 |  |
| 3-testbarkeit | `testbarkeit_verwaiste_dateien` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-arch-tests` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-crypto` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-durable-fs` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-engine` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-erasure` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-lab` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-mvcc` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-ports` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-rank` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-simd` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-store` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-sys` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-testkit` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-testmacros` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-text` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-types` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__contextra-vector` | PENDING | 0.0 |  |
| 3-testbarkeit | `liste_tests__ctx-verify` | PENDING | 0.0 |  |
| 3-testbarkeit | `coverage` | PENDING | 0.0 |  |
| 3-testbarkeit | `coverage_funktionen` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-mvcc__mvcc_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-crypto__crypto_benchmarks` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__distance_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__sq8_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__hnsw_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__hnsw_delete_search_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__audit_benchmarks` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__flush_threshold_amplification` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__partial_rebuild_recall_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__distance_nan_impact_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-vector__filtered_search_selectivity_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__sstable_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__memtable_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__wal_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__lsm_concurrency_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__block_cache_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-store__redb_comparative_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-simd__simd_bench` | PENDING | 0.0 |  |
| 5-tief | `fuzz__crypto__wal_hmac_chain_verify_fuzz` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_compaction_interleave` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_manifest_load` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_memtable_concurrent` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_recovery_arbitrary_state` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_sstable_binary_search` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_wal_replay` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__wal_mutation_chaos` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__wal_roundtrip` | PENDING | 0.0 |  |
| 5-tief | `fuzz__text__fuzz_bm25_tokenize` | PENDING | 0.0 |  |
| 5-tief | `fuzz__text__fuzz_tokenizer_smoke` | PENDING | 0.0 |  |
| 5-tief | `fuzz__vector__fuzz_hnsw_persistence` | PENDING | 0.0 |  |
| 5-tief | `fuzz__vector__hnsw_insert_search` | PENDING | 0.0 |  |
| 5-tief | `mutation_tier0_1` | PENDING | 0.0 |  |
| 5-tief | `loom_mvcc` | PENDING | 0.0 |  |
| 5-tief | `miri_vector_scalar` | PENDING | 0.0 |  |
| 5-tief | `asan_vector` | PENDING | 0.0 |  |
