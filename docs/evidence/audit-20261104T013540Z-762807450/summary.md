# Contextra Voll-Audit

- Commit: `762807450` (2026-11-04 02:35:33 +0100), Branch `main`, hinter origin: 0, ungespeicherte Aenderungen: False
- Start: 2026-11-04T01:35:40+00:00  Dauer: 14.8 min
- Toolchain-Pin: `1.97.0`; Preflight: **PASS** (cargo 1.97.0 (c980f4866 2026-06-30) | Pin 1.97.0)
- Host: Linux-7.2.7-1-liquorix-amd64-x86_64-with-glibc2.41 | 8 CPUs | MemTotal:       10121012 kB
- Nutzer: freddy

## Gesamt

- **FAIL**: 34
- **INFO**: 4
- **PASS**: 29
- **PENDING**: 125

Hinweis: Bei `befund_*`-Schritten bedeutet FAIL, dass der Befund **vorliegt**. `INFO` ist eine Messung oder ein Hinweis ohne Urteil.

## Rote und uebersprungene Schritte

| Schritt | Status | Sek | Detail | Log |
|---|---|---:|---|---|
| `fmt_check` | FAIL | 2.4 |  | `logs/008_fmt_check.log` |
| `check_alle_targets` | FAIL | 16.3 | errors=37 | `logs/009_check_alle_targets.log` |
| `check_no_default_features` | FAIL | 41.2 | errors=17 | `logs/010_check_no_default_features.log` |
| `doc` | FAIL | 7.2 |  | `logs/011_doc.log` |
| `clippy_lib__contextra` | FAIL | 8.9 | errors=5 warnings=0 | `logs/012_clippy_lib__contextra.log` |
| `clippy_alle_targets__contextra` | FAIL | 8.4 | errors=5 warnings=0 | `logs/013_clippy_alle_targets__contextra.log` |
| `clippy_lib__contextra-durable-fs` | FAIL | 10.9 | errors=5 warnings=0 | `logs/018_clippy_lib__contextra-durable-fs.log` |
| `clippy_alle_targets__contextra-durable-fs` | FAIL | 14.6 | errors=5 warnings=0 | `logs/019_clippy_alle_targets__contextra-durable-fs.log` |
| `clippy_lib__contextra-engine` | FAIL | 1.3 | errors=5 warnings=0 | `logs/020_clippy_lib__contextra-engine.log` |
| `clippy_alle_targets__contextra-engine` | FAIL | 9.7 | errors=5 warnings=0 | `logs/021_clippy_alle_targets__contextra-engine.log` |
| `clippy_lib__contextra-erasure` | FAIL | 5.6 | errors=5 warnings=0 | `logs/022_clippy_lib__contextra-erasure.log` |
| `clippy_alle_targets__contextra-erasure` | FAIL | 11.9 | errors=5 warnings=0 | `logs/023_clippy_alle_targets__contextra-erasure.log` |
| `clippy_alle_targets__contextra-lab` | FAIL | 6.3 | errors=2 warnings=1 | `logs/025_clippy_alle_targets__contextra-lab.log` |
| `clippy_lib__contextra-mvcc` | FAIL | 1.2 | errors=5 warnings=0 | `logs/026_clippy_lib__contextra-mvcc.log` |
| `clippy_alle_targets__contextra-mvcc` | FAIL | 2.1 | errors=5 warnings=1 | `logs/027_clippy_alle_targets__contextra-mvcc.log` |
| `clippy_lib__contextra-ports` | FAIL | 5.6 | errors=5 warnings=0 | `logs/028_clippy_lib__contextra-ports.log` |
| `clippy_alle_targets__contextra-ports` | FAIL | 10.0 | errors=6 warnings=1 | `logs/029_clippy_alle_targets__contextra-ports.log` |
| `clippy_lib__contextra-store` | FAIL | 7.1 | errors=5 warnings=0 | `logs/034_clippy_lib__contextra-store.log` |
| `clippy_alle_targets__contextra-store` | FAIL | 6.5 | errors=5 warnings=0 | `logs/035_clippy_alle_targets__contextra-store.log` |
| `clippy_lib__contextra-testkit` | FAIL | 1.7 | errors=5 warnings=0 | `logs/038_clippy_lib__contextra-testkit.log` |
| `clippy_alle_targets__contextra-testkit` | FAIL | 9.9 | errors=5 warnings=0 | `logs/039_clippy_alle_targets__contextra-testkit.log` |
| `clippy_lib__contextra-text` | FAIL | 9.8 | errors=5 warnings=0 | `logs/042_clippy_lib__contextra-text.log` |
| `clippy_alle_targets__contextra-text` | FAIL | 6.0 | errors=5 warnings=0 | `logs/043_clippy_alle_targets__contextra-text.log` |
| `clippy_lib__contextra-vector` | FAIL | 1.4 | errors=5 warnings=0 | `logs/046_clippy_lib__contextra-vector.log` |
| `clippy_alle_targets__contextra-vector` | FAIL | 9.1 | errors=5 warnings=0 | `logs/047_clippy_alle_targets__contextra-vector.log` |
| `feature_matrix_hack` | FAIL | 210.8 |  | `logs/050_feature_matrix_hack.log` |
| `deny` | FAIL | 2.2 |  | `logs/051_deny.log` |
| `panic_flaeche_lib` | FAIL | 25.2 | errors=17 warnings=24 | `logs/053_panic_flaeche_lib.log` |
| `test_lib__contextra` | FAIL | 74.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/055_test_lib__contextra.log` |
| `test_integration__contextra` | FAIL | 22.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/056_test_integration__contextra.log` |
| `test_doc__contextra` | FAIL | 9.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/057_test_doc__contextra.log` |
| `nextest__contextra` | FAIL | 7.8 | nextest: keine Summary-Zeile gefunden | `logs/058_nextest__contextra.log` |
| `test_integration__contextra-arch-tests` | FAIL | 7.6 | passed=47 failed=2 ignored=0 | FAILED: test_t_arch_3b_matrix_edges, test_t_lint_3_obsolete_evidence_terms | `logs/060_test_integration__contextra-arch-tests.log` |
| `nextest__contextra-arch-tests` | FAIL | 2.9 | passed=47 failed=2 | FAILED: (37/49) contextra-arch-tests::arch test_t_arch_3b_matrix_edges, (47/49) contextra-arch-tests::arch test_t_lint_3_obsolete_evidence_terms | `logs/062_nextest__contextra-arch-tests.log` |

## Panic-Flaeche im Produktivcode (Clippy, lib, ohne Tests)

| Crate | unwrap | expect | panic | todo | index | Summe |
|---|---:|---:|---:|---:|---:|---:|
| `ctx-verify` | 0 | 0 | 0 | 0 | 1 | 1 |

## unsafe-Inventar gegen capabilities.toml

- `contextra-erasure`: 0 Zeilen, island=False, nicht in capabilities.toml
- `contextra-lab`: 0 Zeilen, island=False, nicht in capabilities.toml
- `contextra-simd`: 223 Zeilen, island=True, ok
- `contextra-sys`: 19 Zeilen, island=True, ok
- `contextra-testmacros`: 0 Zeilen, island=False, nicht in capabilities.toml
- `ctx-verify`: 0 Zeilen, island=False, nicht in capabilities.toml

## Reife-Behauptung vs. Beleg (capabilities.toml gegen diesen Lauf)

| Crate | behauptet | lib-Tests | Clippy lib | Urteil |
|---|---|---|---|---|
| `contextra` | stable | FAIL | FAIL | **Behauptung ohne Beleg** |
| `contextra-arch-tests` | stable | PASS | PASS | ok |
| `contextra-crypto` | beta | PASS | PASS | ok |
| `contextra-durable-fs` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-engine` | beta | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-mvcc` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-ports` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-rank` | stable | PENDING | PASS | **Behauptung ohne Beleg** |
| `contextra-simd` | beta | PENDING | PASS | **Behauptung ohne Beleg** |
| `contextra-store` | beta | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-sys` | stable | PENDING | PASS | **Behauptung ohne Beleg** |
| `contextra-testkit` | stable | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-text` | beta | PENDING | FAIL | **Behauptung ohne Beleg** |
| `contextra-types` | stable | PENDING | PASS | **Behauptung ohne Beleg** |
| `contextra-vector` | beta | PENDING | FAIL | **Behauptung ohne Beleg** |

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
| 1-statisch | `fmt_check` | FAIL | 2.4 |  |
| 1-statisch | `check_alle_targets` | FAIL | 16.3 | errors=37 |
| 1-statisch | `check_no_default_features` | FAIL | 41.2 | errors=17 |
| 1-statisch | `doc` | FAIL | 7.2 |  |
| 1-statisch | `clippy_lib__contextra` | FAIL | 8.9 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra` | FAIL | 8.4 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-arch-tests` | PASS | 10.9 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-arch-tests` | PASS | 1.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-crypto` | PASS | 26.6 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-crypto` | PASS | 29.0 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-durable-fs` | FAIL | 10.9 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-durable-fs` | FAIL | 14.6 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-engine` | FAIL | 1.3 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-engine` | FAIL | 9.7 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-erasure` | FAIL | 5.6 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-erasure` | FAIL | 11.9 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-lab` | PASS | 11.8 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-lab` | FAIL | 6.3 | errors=2 warnings=1 |
| 1-statisch | `clippy_lib__contextra-mvcc` | FAIL | 1.2 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-mvcc` | FAIL | 2.1 | errors=5 warnings=1 |
| 1-statisch | `clippy_lib__contextra-ports` | FAIL | 5.6 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-ports` | FAIL | 10.0 | errors=6 warnings=1 |
| 1-statisch | `clippy_lib__contextra-rank` | PASS | 7.6 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-rank` | PASS | 7.7 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-simd` | PASS | 5.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-simd` | PASS | 9.0 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-store` | FAIL | 7.1 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-store` | FAIL | 6.5 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-sys` | PASS | 8.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-sys` | PASS | 23.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-testkit` | FAIL | 1.7 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-testkit` | FAIL | 9.9 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-testmacros` | PASS | 2.6 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-testmacros` | PASS | 9.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-text` | FAIL | 9.8 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-text` | FAIL | 6.0 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-types` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-types` | PASS | 8.1 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-vector` | FAIL | 1.4 | errors=5 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-vector` | FAIL | 9.1 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__ctx-verify` | PASS | 2.2 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__ctx-verify` | PASS | 1.3 | errors=0 warnings=0 |
| 1-statisch | `feature_matrix_hack` | FAIL | 210.8 |  |
| 1-statisch | `deny` | FAIL | 2.2 |  |
| 1-statisch | `audit` | PASS | 3.9 |  |
| 1-statisch | `panic_flaeche_lib` | FAIL | 25.2 | errors=17 warnings=24 |
| 1-statisch | `unsafe_inventar` | PASS | 0.2 | keine Verstoesse |
| 2-tests | `test_lib__contextra` | FAIL | 74.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra` | FAIL | 22.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra` | FAIL | 9.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `nextest__contextra` | FAIL | 7.8 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-arch-tests` | PASS | 19.3 | passed=30 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-arch-tests` | FAIL | 7.6 | passed=47 failed=2 ignored=0 | FAILED: test_t_arch_3b_matrix_edges, test_t_lint_3_obsolete_evidence_terms |
| 2-tests | `test_doc__contextra-arch-tests` | PASS | 0.4 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-arch-tests` | FAIL | 2.9 | passed=47 failed=2 | FAILED: (37/49) contextra-arch-tests::arch test_t_arch_3b_matrix_edges, (47/49) contextra-arch-tests::arch test_t_lint_3_obsolete_evidence_terms |
| 2-tests | `test_lib__contextra-crypto` | PASS | 53.3 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-crypto` | PASS | 26.3 | passed=39 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-crypto` | PASS | 0.6 | passed=1 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-crypto` | PASS | 18.8 | passed=39 failed=0 |
| 2-tests | `test_lib__contextra-durable-fs` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-durable-fs` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-durable-fs` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-durable-fs` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-engine` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-engine` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-engine` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-engine` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-erasure` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-erasure` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-erasure` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-erasure` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-lab` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-lab` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-lab` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-lab` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-mvcc` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-mvcc` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-mvcc` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-mvcc` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-ports` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-ports` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-ports` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-ports` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-rank` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-rank` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-rank` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-rank` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-simd` | PENDING | 0.0 |  |
| 2-tests | `test_integration__contextra-simd` | PENDING | 0.0 |  |
| 2-tests | `test_doc__contextra-simd` | PENDING | 0.0 |  |
| 2-tests | `nextest__contextra-simd` | PENDING | 0.0 |  |
| 2-tests | `test_lib__contextra-store` | PENDING | 0.0 |  |
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
