# Contextra Voll-Audit

- Commit: `44d7748c1` (2026-11-04 03:23:02 +0100), Branch `main`, hinter origin: 0, ungespeicherte Aenderungen: False
- Start: 2026-11-04T02:23:18+00:00  Dauer: 19.7 min
- Toolchain-Pin: `1.97.0`; Preflight: **PASS** (cargo 1.97.0 (c980f4866 2026-06-30) | Pin 1.97.0)
- Host: Linux-7.2.7-1-liquorix-amd64-x86_64-with-glibc2.41 | 8 CPUs | MemTotal:       10121012 kB
- Nutzer: freddy

## Gesamt

- **FAIL**: 87
- **INFO**: 7
- **PASS**: 55
- **PENDING**: 43

Hinweis: Bei `befund_*`-Schritten bedeutet FAIL, dass der Befund **vorliegt**. `INFO` ist eine Messung oder ein Hinweis ohne Urteil.

## Rote und uebersprungene Schritte

| Schritt | Status | Sek | Detail | Log |
|---|---|---:|---|---|
| `fmt_check` | FAIL | 2.9 |  | `logs/008_fmt_check.log` |
| `check_alle_targets` | FAIL | 55.4 | errors=39 | `logs/009_check_alle_targets.log` |
| `check_no_default_features` | FAIL | 12.4 | errors=14 | `logs/010_check_no_default_features.log` |
| `doc` | FAIL | 8.0 |  | `logs/011_doc.log` |
| `clippy_lib__contextra` | FAIL | 7.9 | errors=5 warnings=0 | `logs/012_clippy_lib__contextra.log` |
| `clippy_alle_targets__contextra` | FAIL | 9.4 | errors=5 warnings=0 | `logs/013_clippy_alle_targets__contextra.log` |
| `clippy_lib__contextra-durable-fs` | FAIL | 2.0 | errors=5 warnings=0 | `logs/018_clippy_lib__contextra-durable-fs.log` |
| `clippy_alle_targets__contextra-durable-fs` | FAIL | 1.7 | errors=5 warnings=0 | `logs/019_clippy_alle_targets__contextra-durable-fs.log` |
| `clippy_lib__contextra-engine` | FAIL | 2.8 | errors=5 warnings=0 | `logs/020_clippy_lib__contextra-engine.log` |
| `clippy_alle_targets__contextra-engine` | FAIL | 3.1 | errors=5 warnings=0 | `logs/021_clippy_alle_targets__contextra-engine.log` |
| `clippy_lib__contextra-erasure` | FAIL | 2.3 | errors=5 warnings=0 | `logs/022_clippy_lib__contextra-erasure.log` |
| `clippy_alle_targets__contextra-erasure` | FAIL | 5.4 | errors=5 warnings=0 | `logs/023_clippy_alle_targets__contextra-erasure.log` |
| `clippy_alle_targets__contextra-lab` | FAIL | 1.5 | errors=2 warnings=0 | `logs/025_clippy_alle_targets__contextra-lab.log` |
| `clippy_lib__contextra-mvcc` | FAIL | 1.4 | errors=5 warnings=0 | `logs/026_clippy_lib__contextra-mvcc.log` |
| `clippy_alle_targets__contextra-mvcc` | FAIL | 1.8 | errors=5 warnings=0 | `logs/027_clippy_alle_targets__contextra-mvcc.log` |
| `clippy_lib__contextra-ports` | FAIL | 4.9 | errors=5 warnings=0 | `logs/028_clippy_lib__contextra-ports.log` |
| `clippy_alle_targets__contextra-ports` | FAIL | 3.3 | errors=6 warnings=1 | `logs/029_clippy_alle_targets__contextra-ports.log` |
| `clippy_lib__contextra-store` | FAIL | 1.5 | errors=5 warnings=0 | `logs/034_clippy_lib__contextra-store.log` |
| `clippy_alle_targets__contextra-store` | FAIL | 2.8 | errors=5 warnings=0 | `logs/035_clippy_alle_targets__contextra-store.log` |
| `clippy_lib__contextra-testkit` | FAIL | 1.8 | errors=5 warnings=0 | `logs/038_clippy_lib__contextra-testkit.log` |
| `clippy_alle_targets__contextra-testkit` | FAIL | 2.4 | errors=5 warnings=0 | `logs/039_clippy_alle_targets__contextra-testkit.log` |
| `clippy_lib__contextra-text` | FAIL | 1.4 | errors=5 warnings=0 | `logs/042_clippy_lib__contextra-text.log` |
| `clippy_alle_targets__contextra-text` | FAIL | 1.5 | errors=5 warnings=0 | `logs/043_clippy_alle_targets__contextra-text.log` |
| `clippy_lib__contextra-vector` | FAIL | 6.8 | errors=5 warnings=0 | `logs/046_clippy_lib__contextra-vector.log` |
| `clippy_alle_targets__contextra-vector` | FAIL | 1.7 | errors=5 warnings=0 | `logs/047_clippy_alle_targets__contextra-vector.log` |
| `feature_matrix_hack` | FAIL | 184.3 |  | `logs/050_feature_matrix_hack.log` |
| `deny` | FAIL | 2.1 |  | `logs/051_deny.log` |
| `panic_flaeche_lib` | FAIL | 24.7 | errors=14 warnings=126 | `logs/053_panic_flaeche_lib.log` |
| `test_lib__contextra` | FAIL | 58.7 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/055_test_lib__contextra.log` |
| `test_integration__contextra` | FAIL | 13.0 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/056_test_integration__contextra.log` |
| `test_doc__contextra` | FAIL | 5.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/057_test_doc__contextra.log` |
| `nextest__contextra` | FAIL | 3.5 | nextest: keine Summary-Zeile gefunden | `logs/058_nextest__contextra.log` |
| `test_integration__contextra-arch-tests` | FAIL | 26.0 | passed=46 failed=3 ignored=0 | FAILED: test_t_arch_3b_matrix_edges, test_t_gov_1_ratchet_hygiene, test_t_lint_3_obsolete_evidence_terms | `logs/060_test_integration__contextra-arch-tests.log` |
| `nextest__contextra-arch-tests` | FAIL | 24.3 | passed=46 failed=3 | FAILED: (33/49) contextra-arch-tests::arch test_t_arch_3b_matrix_edges, (38/49) contextra-arch-tests::arch test_t_gov_1_ratchet_hygiene, (47/49) contextra-arch-tests::arch test_t_lint_3_obsolete_evidence_terms | `logs/062_nextest__contextra-arch-tests.log` |
| `test_integration__contextra-crypto` | FAIL | 3.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/064_test_integration__contextra-crypto.log` |
| `nextest__contextra-crypto` | FAIL | 1.2 | nextest: keine Summary-Zeile gefunden | `logs/066_nextest__contextra-crypto.log` |
| `test_integration__contextra-durable-fs` | FAIL | 3.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/068_test_integration__contextra-durable-fs.log` |
| `nextest__contextra-durable-fs` | FAIL | 4.9 | nextest: keine Summary-Zeile gefunden | `logs/070_nextest__contextra-durable-fs.log` |
| `test_lib__contextra-engine` | FAIL | 9.7 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/071_test_lib__contextra-engine.log` |
| `test_integration__contextra-engine` | FAIL | 6.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/072_test_integration__contextra-engine.log` |
| `test_doc__contextra-engine` | FAIL | 5.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/073_test_doc__contextra-engine.log` |
| `nextest__contextra-engine` | FAIL | 6.0 | nextest: keine Summary-Zeile gefunden | `logs/074_nextest__contextra-engine.log` |
| `test_lib__contextra-mvcc` | FAIL | 1.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/083_test_lib__contextra-mvcc.log` |
| `test_integration__contextra-mvcc` | FAIL | 5.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/084_test_integration__contextra-mvcc.log` |
| `nextest__contextra-mvcc` | FAIL | 7.2 | nextest: keine Summary-Zeile gefunden | `logs/086_nextest__contextra-mvcc.log` |
| `test_lib__contextra-ports` | FAIL | 12.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/087_test_lib__contextra-ports.log` |
| `test_integration__contextra-ports` | FAIL | 7.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/088_test_integration__contextra-ports.log` |
| `nextest__contextra-ports` | FAIL | 5.9 | nextest: keine Summary-Zeile gefunden | `logs/090_nextest__contextra-ports.log` |
| `test_integration__contextra-rank` | FAIL | 1.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/092_test_integration__contextra-rank.log` |
| `nextest__contextra-rank` | FAIL | 0.8 | nextest: keine Summary-Zeile gefunden | `logs/094_nextest__contextra-rank.log` |
| `test_integration__contextra-simd` | FAIL | 3.3 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/096_test_integration__contextra-simd.log` |
| `nextest__contextra-simd` | FAIL | 1.6 | nextest: keine Summary-Zeile gefunden | `logs/098_nextest__contextra-simd.log` |
| `test_lib__contextra-store` | FAIL | 22.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/099_test_lib__contextra-store.log` |
| `test_integration__contextra-store` | FAIL | 35.4 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/100_test_integration__contextra-store.log` |
| `nextest__contextra-store` | FAIL | 11.5 | nextest: keine Summary-Zeile gefunden | `logs/102_nextest__contextra-store.log` |
| `test_integration__contextra-sys` | FAIL | 2.0 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/104_test_integration__contextra-sys.log` |
| `nextest__contextra-sys` | FAIL | 0.8 | nextest: keine Summary-Zeile gefunden | `logs/106_nextest__contextra-sys.log` |
| `test_lib__contextra-testkit` | FAIL | 4.0 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/107_test_lib__contextra-testkit.log` |
| `test_integration__contextra-testkit` | FAIL | 13.4 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/108_test_integration__contextra-testkit.log` |
| `nextest__contextra-testkit` | FAIL | 8.0 | nextest: keine Summary-Zeile gefunden | `logs/110_nextest__contextra-testkit.log` |
| `test_integration__contextra-testmacros` | FAIL | 0.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/112_test_integration__contextra-testmacros.log` |
| `nextest__contextra-testmacros` | FAIL | 0.6 | nextest: keine Summary-Zeile gefunden | `logs/114_nextest__contextra-testmacros.log` |
| `test_lib__contextra-text` | FAIL | 13.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/115_test_lib__contextra-text.log` |
| `test_integration__contextra-text` | FAIL | 11.4 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/116_test_integration__contextra-text.log` |
| `nextest__contextra-text` | FAIL | 13.5 | nextest: keine Summary-Zeile gefunden | `logs/118_nextest__contextra-text.log` |
| `test_lib__contextra-types` | FAIL | 7.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/119_test_lib__contextra-types.log` |
| `test_integration__contextra-types` | FAIL | 12.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/120_test_integration__contextra-types.log` |
| `nextest__contextra-types` | FAIL | 16.2 | nextest: keine Summary-Zeile gefunden | `logs/122_nextest__contextra-types.log` |
| `test_lib__contextra-vector` | FAIL | 6.3 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/123_test_lib__contextra-vector.log` |
| `test_integration__contextra-vector` | FAIL | 9.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/124_test_integration__contextra-vector.log` |
| `nextest__contextra-vector` | FAIL | 5.8 | nextest: keine Summary-Zeile gefunden | `logs/126_nextest__contextra-vector.log` |
| `test_integration__ctx-verify` | FAIL | 2.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/128_test_integration__ctx-verify.log` |
| `nextest__ctx-verify` | FAIL | 1.9 | nextest: keine Summary-Zeile gefunden | `logs/130_nextest__ctx-verify.log` |
| `test_gesamt_debug` | FAIL | 13.3 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/131_test_gesamt_debug.log` |
| `test_gesamt_release` | FAIL | 178.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/132_test_gesamt_release.log` |
| `testbarkeit_funktionen` | FAIL | 0.0 | 3055 Funktionen, 145 mit direktem Umweltzugriff; 5 komplexe pub-Funktionen ohne Testbezug | `logs/134_testbarkeit_funktionen.log` |
| `testbarkeit_verwaiste_dateien` | FAIL | 1.5 | 17 verwaiste Dateien (39 Tests laufen nie), 0 fehlende deklarierte Dateien | `logs/135_testbarkeit_verwaiste_dateien.log` |
| `liste_tests__contextra` | FAIL | 8.2 | 0 Tests gelistet | `logs/136_liste_tests__contextra.log` |
| `liste_tests__contextra-crypto` | FAIL | 4.1 | 0 Tests gelistet | `logs/138_liste_tests__contextra-crypto.log` |
| `liste_tests__contextra-durable-fs` | FAIL | 3.3 | 0 Tests gelistet | `logs/139_liste_tests__contextra-durable-fs.log` |
| `liste_tests__contextra-engine` | FAIL | 9.3 | 0 Tests gelistet | `logs/140_liste_tests__contextra-engine.log` |
| `liste_tests__contextra-mvcc` | FAIL | 6.1 | 0 Tests gelistet | `logs/143_liste_tests__contextra-mvcc.log` |
| `liste_tests__contextra-ports` | FAIL | 2.0 | 0 Tests gelistet | `logs/144_liste_tests__contextra-ports.log` |
| `liste_tests__contextra-rank` | FAIL | 0.4 | 0 Tests gelistet | `logs/145_liste_tests__contextra-rank.log` |
| `liste_tests__contextra-simd` | FAIL | 0.4 | 0 Tests gelistet | `logs/146_liste_tests__contextra-simd.log` |
| `liste_tests__contextra-store` | FAIL | 14.1 | 0 Tests gelistet | `logs/147_liste_tests__contextra-store.log` |
| `liste_tests__contextra-sys` | FAIL | 0.3 | 0 Tests gelistet | `logs/148_liste_tests__contextra-sys.log` |

## Alle Schritte

| Gruppe | Schritt | Status | Sek | Detail |
|---|---|---|---:|---|
| B-befunde | `befund_cfg_features_undefiniert` | PASS | 0.1 | keine undefinierten Features |
| B-befunde | `befund_default_members` | INFO | 0.0 | members=19 default-members=16; nicht in default-members (von `cargo test` ohne --workspace nicht erfasst): ['contextra-lab', 'contextra-testmacros', 'ctx-verify'] |
| B-befunde | `befund_maturity_claims` | INFO | 0.0 | stable=9 beta=6 Eintraege ohne Crate=0 |
| B-befunde | `befund_bench_fuzz_dateien` | PASS | 0.0 | 0 fehlende Dateien/Verzeichnisse |
| 0-basis | `toolchain_preflight` | PASS | 0.1 | cargo 1.97.0 (c980f4866 2026-06-30) | Pin 1.97.0 |
| 0-basis | `toolchain_versionen` | INFO | 0.4 |  |
| 0-basis | `cargo_fetch` | PASS | 0.5 |  |
| 0-basis | `cargo_tree_duplicates` | INFO | 0.3 |  |
| 1-statisch | `fmt_check` | FAIL | 2.9 |  |
| 1-statisch | `check_alle_targets` | FAIL | 55.4 | errors=39 |
| 1-statisch | `check_no_default_features` | FAIL | 12.4 | errors=14 |
| 1-statisch | `doc` | FAIL | 8.0 |  |
| 1-statisch | `clippy_lib__contextra` | FAIL | 7.9 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra` | FAIL | 9.4 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-arch-tests` | PASS | 1.2 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-arch-tests` | PASS | 0.4 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-crypto` | PASS | 1.1 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-crypto` | PASS | 0.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-durable-fs` | FAIL | 2.0 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-durable-fs` | FAIL | 1.7 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-engine` | FAIL | 2.8 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-engine` | FAIL | 3.1 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-erasure` | FAIL | 2.3 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-erasure` | FAIL | 5.4 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-lab` | PASS | 1.0 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-lab` | FAIL | 1.5 | errors=2 warnings=0 |
| 1-statisch | `clippy_lib__contextra-mvcc` | FAIL | 1.4 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-mvcc` | FAIL | 1.8 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-ports` | FAIL | 4.9 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-ports` | FAIL | 3.3 | errors=6 warnings=1 |
| 1-statisch | `clippy_lib__contextra-rank` | PASS | 1.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-rank` | PASS | 2.2 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-simd` | PASS | 1.8 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-simd` | PASS | 1.4 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-store` | FAIL | 1.5 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-store` | FAIL | 2.8 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-sys` | PASS | 1.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-sys` | PASS | 0.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-testkit` | FAIL | 1.8 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-testkit` | FAIL | 2.4 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-testmacros` | PASS | 1.1 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-testmacros` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-text` | FAIL | 1.4 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-text` | FAIL | 1.5 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-types` | PASS | 1.7 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-types` | PASS | 0.6 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-vector` | FAIL | 6.8 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-vector` | FAIL | 1.7 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__ctx-verify` | PASS | 0.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__ctx-verify` | PASS | 4.1 | errors=0 warnings=0 |
| 1-statisch | `feature_matrix_hack` | FAIL | 184.3 |  |
| 1-statisch | `deny` | FAIL | 2.1 |  |
| 1-statisch | `audit` | PASS | 1.7 |  |
| 1-statisch | `panic_flaeche_lib` | FAIL | 24.7 | errors=14 warnings=126 |
| 1-statisch | `unsafe_inventar` | PASS | 0.2 | keine Verstoesse |
| 2-tests | `test_lib__contextra` | FAIL | 58.7 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra` | FAIL | 13.0 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra` | FAIL | 5.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `nextest__contextra` | FAIL | 3.5 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-arch-tests` | PASS | 2.9 | passed=30 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-arch-tests` | FAIL | 26.0 | passed=46 failed=3 ignored=0 | FAILED: test_t_arch_3b_matrix_edges, test_t_gov_1_ratchet_hygiene, test_t_lint_3_obsolete_evidence_terms |
| 2-tests | `test_doc__contextra-arch-tests` | PASS | 0.4 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-arch-tests` | FAIL | 24.3 | passed=46 failed=3 | FAILED: (33/49) contextra-arch-tests::arch test_t_arch_3b_matrix_edges, (38/49) contextra-arch-tests::arch test_t_gov_1_ratchet_hygiene, (47/49) contextra-arch-tests::arch test_t_lint_3_obsolete_evidence_terms |
| 2-tests | `test_lib__contextra-crypto` | PASS | 11.0 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-crypto` | FAIL | 3.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-crypto` | PASS | 1.0 | passed=1 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-crypto` | FAIL | 1.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-durable-fs` | PASS | 9.4 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-durable-fs` | FAIL | 3.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-durable-fs` | PASS | 0.4 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-durable-fs` | FAIL | 4.9 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-engine` | FAIL | 9.7 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-engine` | FAIL | 6.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-engine` | FAIL | 5.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `nextest__contextra-engine` | FAIL | 6.0 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-erasure` | PASS | 13.1 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-erasure` | PASS | 9.1 | passed=18 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-erasure` | PASS | 0.6 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-erasure` | PASS | 0.9 | passed=18 failed=0 |
| 2-tests | `test_lib__contextra-lab` | PASS | 10.1 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-lab` | PASS | 3.9 | passed=25 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-lab` | PASS | 0.7 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-lab` | PASS | 1.2 | passed=25 failed=0 |
| 2-tests | `test_lib__contextra-mvcc` | FAIL | 1.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-mvcc` | FAIL | 5.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-mvcc` | PASS | 2.7 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-mvcc` | FAIL | 7.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-ports` | FAIL | 12.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-ports` | FAIL | 7.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-ports` | PASS | 2.5 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-ports` | FAIL | 5.9 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-rank` | PASS | 8.1 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-rank` | FAIL | 1.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-rank` | PASS | 0.5 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-rank` | FAIL | 0.8 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-simd` | PASS | 10.9 | passed=13 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-simd` | FAIL | 3.3 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-simd` | PASS | 1.0 | passed=2 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-simd` | FAIL | 1.6 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-store` | FAIL | 22.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-store` | FAIL | 35.4 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-store` | PASS | 13.2 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-store` | FAIL | 11.5 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-sys` | PASS | 11.8 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-sys` | FAIL | 2.0 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-sys` | PASS | 0.3 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-sys` | FAIL | 0.8 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-testkit` | FAIL | 4.0 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-testkit` | FAIL | 13.4 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-testkit` | PASS | 0.4 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-testkit` | FAIL | 8.0 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-testmacros` | PASS | 0.3 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-testmacros` | FAIL | 0.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-testmacros` | PASS | 0.2 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-testmacros` | FAIL | 0.6 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-text` | FAIL | 13.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-text` | FAIL | 11.4 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-text` | PASS | 0.5 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-text` | FAIL | 13.5 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-types` | FAIL | 7.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-types` | FAIL | 12.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-types` | PASS | 0.9 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-types` | FAIL | 16.2 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-vector` | FAIL | 6.3 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-vector` | FAIL | 9.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-vector` | PASS | 0.4 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-vector` | FAIL | 5.8 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__ctx-verify` | PASS | 0.8 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__ctx-verify` | FAIL | 2.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__ctx-verify` | PASS | 2.0 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__ctx-verify` | FAIL | 1.9 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_gesamt_debug` | FAIL | 13.3 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_gesamt_release` | FAIL | 178.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 3-testbarkeit | `testbarkeit_crates` | PASS | 5.9 | 19 Crates, 2098 Tests (statisch); ohne Tests: keine; schwach: ['contextra-erasure', 'contextra-ports', 'contextra-testkit'] |
| 3-testbarkeit | `testbarkeit_funktionen` | FAIL | 0.0 | 3055 Funktionen, 145 mit direktem Umweltzugriff; 5 komplexe pub-Funktionen ohne Testbezug |
| 3-testbarkeit | `testbarkeit_verwaiste_dateien` | FAIL | 1.5 | 17 verwaiste Dateien (39 Tests laufen nie), 0 fehlende deklarierte Dateien |
| 3-testbarkeit | `liste_tests__contextra` | FAIL | 8.2 | 0 Tests gelistet |
| 3-testbarkeit | `liste_tests__contextra-arch-tests` | INFO | 0.3 | 0 Tests gelistet |
| 3-testbarkeit | `liste_tests__contextra-crypto` | FAIL | 4.1 | 0 Tests gelistet |
| 3-testbarkeit | `liste_tests__contextra-durable-fs` | FAIL | 3.3 | 0 Tests gelistet |
| 3-testbarkeit | `liste_tests__contextra-engine` | FAIL | 9.3 | 0 Tests gelistet |
| 3-testbarkeit | `liste_tests__contextra-erasure` | INFO | 6.2 | 0 Tests gelistet |
| 3-testbarkeit | `liste_tests__contextra-lab` | INFO | 3.6 | 0 Tests gelistet |
| 3-testbarkeit | `liste_tests__contextra-mvcc` | FAIL | 6.1 | 0 Tests gelistet |
| 3-testbarkeit | `liste_tests__contextra-ports` | FAIL | 2.0 | 0 Tests gelistet |
| 3-testbarkeit | `liste_tests__contextra-rank` | FAIL | 0.4 | 0 Tests gelistet |
| 3-testbarkeit | `liste_tests__contextra-simd` | FAIL | 0.4 | 0 Tests gelistet |
| 3-testbarkeit | `liste_tests__contextra-store` | FAIL | 14.1 | 0 Tests gelistet |
| 3-testbarkeit | `liste_tests__contextra-sys` | FAIL | 0.3 | 0 Tests gelistet |
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
