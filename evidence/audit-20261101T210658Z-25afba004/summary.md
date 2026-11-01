# Contextra Voll-Audit

- Commit: `25afba004` (2026-11-01 22:06:33 +0100), Branch `main`, hinter origin: 0, ungespeicherte Aenderungen: False
- Start: 2026-11-01T21:06:58+00:00  Dauer: 35.1 min
- Toolchain-Pin: `1.89.0`; Preflight: **PASS** (cargo 1.89.0 (c24e10642 2025-06-23) | Pin 1.89.0)
- Host: Linux-7.2.7-1-liquorix-amd64-x86_64-with-glibc2.41 | 8 CPUs | MemTotal:       10121020 kB
- Nutzer: freddy

## Gesamt

- **FAIL**: 59
- **INFO**: 6
- **PASS**: 60
- **PENDING**: 32

Hinweis: Bei `befund_*`-Schritten bedeutet FAIL, dass der Befund **vorliegt**. `INFO` ist eine Messung oder ein Hinweis ohne Urteil.

## Rote und uebersprungene Schritte

| Schritt | Status | Sek | Detail | Log |
|---|---|---:|---|---|
| `befund_cfg_features_undefiniert` | FAIL | 1.4 | 2 Crates mit undefinierten Features | `logs/000_befund_cfg_features_undefiniert.log` |
| `befund_safe_fn_mit_unsafe_inhalt` | FAIL | 0.0 | 2 bekannte K7/K8-Stellen (mmap_readonly, reopen_and_dup2) | gesamt 13 Kandidaten | `logs/002_befund_safe_fn_mit_unsafe_inhalt.log` |
| `befund_from_entropy` | FAIL | 0.1 | 1 Produktivdateien mit from_entropy (K14) | `logs/006_befund_from_entropy.log` |
| `befund_maturity_claims` | FAIL | 0.0 | stable=15 beta=12 Eintraege ohne Crate=21 | `logs/007_befund_maturity_claims.log` |
| `befund_bench_fuzz_dateien` | FAIL | 0.0 | 10 fehlende Dateien/Verzeichnisse | `logs/009_befund_bench_fuzz_dateien.log` |
| `produkt_huelle_ohne_kern` | FAIL | 6.8 | 5 Produkt-Crates mit Befund | `logs/010_produkt_huelle_ohne_kern.log` |
| `fmt_check` | FAIL | 3.4 |  | `logs/017_fmt_check.log` |
| `check_alle_targets` | FAIL | 75.1 | errors=247 | `logs/018_check_alle_targets.log` |
| `check_no_default_features` | FAIL | 7.3 | errors=34 | `logs/019_check_no_default_features.log` |
| `doc` | FAIL | 9.3 |  | `logs/020_doc.log` |
| `clippy_lib__contextra` | FAIL | 6.1 | errors=83 warnings=1 | `logs/021_clippy_lib__contextra.log` |
| `clippy_alle_targets__contextra` | FAIL | 4.1 | errors=83 warnings=1 | `logs/022_clippy_alle_targets__contextra.log` |
| `clippy_lib__contextra-crypto` | FAIL | 7.0 | errors=27 warnings=0 | `logs/025_clippy_lib__contextra-crypto.log` |
| `clippy_alle_targets__contextra-crypto` | FAIL | 7.4 | errors=27 warnings=1 | `logs/026_clippy_alle_targets__contextra-crypto.log` |
| `clippy_lib__contextra-durable-fs` | FAIL | 7.7 | errors=9 warnings=0 | `logs/027_clippy_lib__contextra-durable-fs.log` |
| `clippy_alle_targets__contextra-durable-fs` | FAIL | 0.7 | errors=10 warnings=1 | `logs/028_clippy_alle_targets__contextra-durable-fs.log` |
| `clippy_lib__contextra-engine` | FAIL | 2.0 | errors=83 warnings=1 | `logs/029_clippy_lib__contextra-engine.log` |
| `clippy_alle_targets__contextra-engine` | FAIL | 6.1 | errors=83 warnings=1 | `logs/030_clippy_alle_targets__contextra-engine.log` |
| `clippy_alle_targets__contextra-ports` | FAIL | 5.0 | errors=5 warnings=0 | `logs/034_clippy_alle_targets__contextra-ports.log` |
| `clippy_alle_targets__contextra-rank` | FAIL | 5.4 | errors=4 warnings=1 | `logs/036_clippy_alle_targets__contextra-rank.log` |
| `clippy_lib__contextra-store` | FAIL | 4.1 | errors=27 warnings=0 | `logs/039_clippy_lib__contextra-store.log` |
| `clippy_alle_targets__contextra-store` | FAIL | 4.1 | errors=27 warnings=1 | `logs/040_clippy_alle_targets__contextra-store.log` |
| `clippy_lib__contextra-text` | FAIL | 5.7 | errors=56 warnings=0 | `logs/045_clippy_lib__contextra-text.log` |
| `clippy_alle_targets__contextra-text` | FAIL | 6.0 | errors=180 warnings=1 | `logs/046_clippy_alle_targets__contextra-text.log` |
| `clippy_lib__contextra-vector` | FAIL | 1.3 | errors=27 warnings=0 | `logs/049_clippy_lib__contextra-vector.log` |
| `clippy_alle_targets__contextra-vector` | FAIL | 3.0 | errors=27 warnings=1 | `logs/050_clippy_alle_targets__contextra-vector.log` |
| `feature_matrix_hack` | FAIL | 203.2 |  | `logs/051_feature_matrix_hack.log` |
| `deny` | FAIL | 1.4 |  | `logs/052_deny.log` |
| `panic_flaeche_lib` | FAIL | 21.4 | errors=34 warnings=224 | `logs/054_panic_flaeche_lib.log` |
| `test_lib__contextra` | FAIL | 46.0 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/056_test_lib__contextra.log` |
| `test_integration__contextra` | FAIL | 6.7 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/057_test_integration__contextra.log` |
| `test_doc__contextra` | FAIL | 6.4 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/058_test_doc__contextra.log` |
| `nextest__contextra` | FAIL | 6.6 | nextest: keine Summary-Zeile gefunden | `logs/059_nextest__contextra.log` |
| `test_lib__contextra-arch-tests` | FAIL | 0.3 | passed=2 failed=1 ignored=0 | FAILED: tests::erlaubt_abwaerts_abhaengigkeit | `logs/060_test_lib__contextra-arch-tests.log` |
| `test_integration__contextra-arch-tests` | FAIL | 0.7 | passed=4 failed=1 ignored=0 | FAILED: tests::erlaubt_abwaerts_abhaengigkeit | `logs/061_test_integration__contextra-arch-tests.log` |
| `nextest__contextra-arch-tests` | FAIL | 1.0 | passed=4 failed=1 | FAILED: (4/5) contextra-arch-tests tests::erlaubt_abwaerts_abhaengigkeit | `logs/063_nextest__contextra-arch-tests.log` |
| `test_integration__contextra-crypto` | FAIL | 2.7 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/065_test_integration__contextra-crypto.log` |
| `nextest__contextra-crypto` | FAIL | 1.9 | nextest: keine Summary-Zeile gefunden | `logs/067_nextest__contextra-crypto.log` |
| `test_lib__contextra-engine` | FAIL | 30.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/072_test_lib__contextra-engine.log` |
| `test_integration__contextra-engine` | FAIL | 10.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/073_test_integration__contextra-engine.log` |
| `test_doc__contextra-engine` | FAIL | 6.0 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/074_test_doc__contextra-engine.log` |
| `nextest__contextra-engine` | FAIL | 10.6 | nextest: keine Summary-Zeile gefunden | `logs/075_nextest__contextra-engine.log` |
| `test_lib__contextra-ports` | FAIL | 8.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/080_test_lib__contextra-ports.log` |
| `test_integration__contextra-ports` | FAIL | 1.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/081_test_integration__contextra-ports.log` |
| `nextest__contextra-ports` | FAIL | 1.1 | nextest: keine Summary-Zeile gefunden | `logs/083_nextest__contextra-ports.log` |
| `test_integration__contextra-rank` | FAIL | 2.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/085_test_integration__contextra-rank.log` |
| `nextest__contextra-rank` | FAIL | 2.3 | nextest: keine Summary-Zeile gefunden | `logs/087_nextest__contextra-rank.log` |
| `test_lib__contextra-store` | FAIL | 22.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/092_test_lib__contextra-store.log` |
| `test_integration__contextra-store` | FAIL | 50.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/093_test_integration__contextra-store.log` |
| `nextest__contextra-store` | FAIL | 114.4 | nextest: keine Summary-Zeile gefunden | `logs/095_nextest__contextra-store.log` |
| `test_integration__contextra-text` | FAIL | 10.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/105_test_integration__contextra-text.log` |
| `nextest__contextra-text` | FAIL | 9.8 | nextest: keine Summary-Zeile gefunden | `logs/107_nextest__contextra-text.log` |
| `test_lib__contextra-vector` | FAIL | 15.4 | passed=65 failed=1 ignored=0 | FAILED: hnsw::tests::test_hnsw_rebuild_exponential_backoff_and_alarm | `logs/112_test_lib__contextra-vector.log` |
| `test_integration__contextra-vector` | FAIL | 9.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/113_test_integration__contextra-vector.log` |
| `nextest__contextra-vector` | FAIL | 18.4 | nextest: keine Summary-Zeile gefunden | `logs/115_nextest__contextra-vector.log` |
| `test_gesamt_debug` | FAIL | 52.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/116_test_gesamt_debug.log` |
| `test_gesamt_release` | FAIL | 46.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/117_test_gesamt_release.log` |
| `bench__contextra-crypto__crypto_benchmarks` | FAIL | 0.7 | keine Criterion-Zeilen erkannt | `logs/119_bench__contextra-crypto__crypto_benchmarks.log` |
| `bench__contextra-crypto__deletion_proof_latency_bench` | FAIL | 0.3 | keine Criterion-Zeilen erkannt | `logs/120_bench__contextra-crypto__deletion_proof_latency_bench.log` |

## Alle Schritte

| Gruppe | Schritt | Status | Sek | Detail |
|---|---|---|---:|---|
| B-befunde | `befund_cfg_features_undefiniert` | FAIL | 1.4 | 2 Crates mit undefinierten Features |
| B-befunde | `befund_wal_aead_tail_hinweis` | INFO | 0.0 | Toleranz (AEAD-Entschluesselungsfehler am Dateiende) Z.[473, 605]; Ablehnung (Parsefehler nach erfolgreicher Entschluesselung) Z.[525]. Kein Widerspruch, da verschiedene Fehlerklassen; Policy bitte bestaetigen. |
| B-befunde | `befund_safe_fn_mit_unsafe_inhalt` | FAIL | 0.0 | 2 bekannte K7/K8-Stellen (mmap_readonly, reopen_and_dup2) | gesamt 13 Kandidaten |
| B-befunde | `befund_ingest_verdrahtung` | INFO | 0.0 | contextra-ingest entfernt (Rueckbau geplant, kein Befund) |
| B-befunde | `befund_default_members` | PASS | 0.0 | members=15 default-members=15; Produkt ohne Verzeichnis (Rueckbau geplant): 5; unerwartet fehlend: keine; Produkt nicht in default-members: keine |
| B-befunde | `befund_adr_status` | PASS | 0.0 | 1 ADR-Dateien in 1 Verzeichnissen; Proposed: keine |
| B-befunde | `befund_from_entropy` | FAIL | 0.1 | 1 Produktivdateien mit from_entropy (K14) |
| B-befunde | `befund_maturity_claims` | FAIL | 0.0 | stable=15 beta=12 Eintraege ohne Crate=21 |
| B-befunde | `befund_windows_flush` | INFO | 0.0 | windows-Pfad vorhanden=True, compile_error!=False |
| B-befunde | `befund_bench_fuzz_dateien` | FAIL | 0.0 | 10 fehlende Dateien/Verzeichnisse |
| T1-produkt | `produkt_huelle_ohne_kern` | FAIL | 6.8 | 5 Produkt-Crates mit Befund |
| T1-produkt | `produkt_sqlcipher_probe` | PASS | 321.6 | rusqlite=* cipher=Some("4.19.0 community") fts5=true leak=false wrong_key_rejected=true rows=1 |
| T1-produkt | `produkt_crates_io_versionen` | INFO | 3.4 | 8 Abfragen |
| 0-basis | `toolchain_preflight` | PASS | 0.1 | cargo 1.89.0 (c24e10642 2025-06-23) | Pin 1.89.0 |
| 0-basis | `toolchain_versionen` | INFO | 0.4 |  |
| 0-basis | `cargo_fetch` | PASS | 0.6 |  |
| 0-basis | `cargo_tree_duplicates` | INFO | 0.3 |  |
| 1-statisch | `fmt_check` | FAIL | 3.4 |  |
| 1-statisch | `check_alle_targets` | FAIL | 75.1 | errors=247 |
| 1-statisch | `check_no_default_features` | FAIL | 7.3 | errors=34 |
| 1-statisch | `doc` | FAIL | 9.3 |  |
| 1-statisch | `clippy_lib__contextra` | FAIL | 6.1 | errors=83 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra` | FAIL | 4.1 | errors=83 warnings=1 |
| 1-statisch | `clippy_lib__contextra-arch-tests` | PASS | 2.7 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-arch-tests` | PASS | 0.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-crypto` | FAIL | 7.0 | errors=27 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-crypto` | FAIL | 7.4 | errors=27 warnings=1 |
| 1-statisch | `clippy_lib__contextra-durable-fs` | FAIL | 7.7 | errors=9 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-durable-fs` | FAIL | 0.7 | errors=10 warnings=1 |
| 1-statisch | `clippy_lib__contextra-engine` | FAIL | 2.0 | errors=83 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-engine` | FAIL | 6.1 | errors=83 warnings=1 |
| 1-statisch | `clippy_lib__contextra-mvcc` | PASS | 3.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-mvcc` | PASS | 20.9 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-ports` | PASS | 4.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-ports` | FAIL | 5.0 | errors=5 warnings=0 |
| 1-statisch | `clippy_lib__contextra-rank` | PASS | 4.1 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-rank` | FAIL | 5.4 | errors=4 warnings=1 |
| 1-statisch | `clippy_lib__contextra-simd` | PASS | 9.9 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-simd` | PASS | 5.4 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-store` | FAIL | 4.1 | errors=27 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-store` | FAIL | 4.1 | errors=27 warnings=1 |
| 1-statisch | `clippy_lib__contextra-sys` | PASS | 0.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-sys` | PASS | 0.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-testkit` | PASS | 1.8 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-testkit` | PASS | 2.0 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-text` | FAIL | 5.7 | errors=56 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-text` | FAIL | 6.0 | errors=180 warnings=1 |
| 1-statisch | `clippy_lib__contextra-types` | PASS | 2.1 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-types` | PASS | 3.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-vector` | FAIL | 1.3 | errors=27 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-vector` | FAIL | 3.0 | errors=27 warnings=1 |
| 1-statisch | `feature_matrix_hack` | FAIL | 203.2 |  |
| 1-statisch | `deny` | FAIL | 1.4 |  |
| 1-statisch | `audit` | PASS | 1.1 |  |
| 1-statisch | `panic_flaeche_lib` | FAIL | 21.4 | errors=34 warnings=224 |
| 1-statisch | `unsafe_inventar` | PASS | 0.1 | keine Verstoesse |
| 2-tests | `test_lib__contextra` | FAIL | 46.0 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra` | FAIL | 6.7 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra` | FAIL | 6.4 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `nextest__contextra` | FAIL | 6.6 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-arch-tests` | FAIL | 0.3 | passed=2 failed=1 ignored=0 | FAILED: tests::erlaubt_abwaerts_abhaengigkeit |
| 2-tests | `test_integration__contextra-arch-tests` | FAIL | 0.7 | passed=4 failed=1 ignored=0 | FAILED: tests::erlaubt_abwaerts_abhaengigkeit |
| 2-tests | `test_doc__contextra-arch-tests` | PASS | 0.3 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-arch-tests` | FAIL | 1.0 | passed=4 failed=1 | FAILED: (4/5) contextra-arch-tests tests::erlaubt_abwaerts_abhaengigkeit |
| 2-tests | `test_lib__contextra-crypto` | PASS | 124.2 | passed=71 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-crypto` | FAIL | 2.7 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-crypto` | PASS | 0.6 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-crypto` | FAIL | 1.9 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-durable-fs` | PASS | 3.4 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `test_integration__contextra-durable-fs` | PASS | 2.3 | passed=36 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-durable-fs` | PASS | 0.5 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-durable-fs` | PASS | 0.6 | passed=36 failed=0 |
| 2-tests | `test_lib__contextra-engine` | FAIL | 30.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-engine` | FAIL | 10.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-engine` | FAIL | 6.0 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `nextest__contextra-engine` | FAIL | 10.6 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-mvcc` | PASS | 21.8 | passed=47 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-mvcc` | PASS | 31.6 | passed=106 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-mvcc` | PASS | 0.4 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-mvcc` | PASS | 12.2 | passed=106 failed=0 |
| 2-tests | `test_lib__contextra-ports` | FAIL | 8.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-ports` | FAIL | 1.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-ports` | PASS | 0.4 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-ports` | FAIL | 1.1 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-rank` | PASS | 8.7 | passed=7 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-rank` | FAIL | 2.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-rank` | PASS | 0.5 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-rank` | FAIL | 2.3 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-simd` | PASS | 11.0 | passed=13 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-simd` | PASS | 8.6 | passed=77 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-simd` | PASS | 0.8 | passed=2 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-simd` | PASS | 3.0 | passed=77 failed=0 |
| 2-tests | `test_lib__contextra-store` | FAIL | 22.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-store` | FAIL | 50.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-store` | PASS | 1.0 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-store` | FAIL | 114.4 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-sys` | PASS | 0.9 | passed=30 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-sys` | PASS | 0.2 | passed=30 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-sys` | PASS | 0.5 | passed=3 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-sys` | PASS | 0.7 | passed=30 failed=0 |
| 2-tests | `test_lib__contextra-testkit` | PASS | 4.1 | passed=11 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-testkit` | PASS | 4.9 | passed=57 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-testkit` | PASS | 0.5 | passed=0 failed=0 ignored=1 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-testkit` | PASS | 3.4 | passed=57 failed=0 |
| 2-tests | `test_lib__contextra-text` | PASS | 245.0 | passed=106 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-text` | FAIL | 10.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-text` | PASS | 1.9 | passed=1 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-text` | FAIL | 9.8 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_lib__contextra-types` | PASS | 5.6 | passed=120 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-types` | PASS | 7.9 | passed=183 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-types` | PASS | 1.1 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-types` | PASS | 2.1 | passed=183 failed=0 |
| 2-tests | `test_lib__contextra-vector` | FAIL | 15.4 | passed=65 failed=1 ignored=0 | FAILED: hnsw::tests::test_hnsw_rebuild_exponential_backoff_and_alarm |
| 2-tests | `test_integration__contextra-vector` | FAIL | 9.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-vector` | PASS | 0.8 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-vector` | FAIL | 18.4 | nextest: keine Summary-Zeile gefunden |
| 2-tests | `test_gesamt_debug` | FAIL | 52.5 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_gesamt_release` | FAIL | 46.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 4-bench | `bench__contextra-mvcc__mvcc_bench` | PASS | 94.8 | 7 Messpunkte |
| 4-bench | `bench__contextra-crypto__crypto_benchmarks` | FAIL | 0.7 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-crypto__deletion_proof_latency_bench` | FAIL | 0.3 | keine Criterion-Zeilen erkannt |
| 4-bench | `bench__contextra-vector__distance_bench` | PASS | 81.8 | 6 Messpunkte |
| 4-bench | `bench__contextra-vector__sq8_bench` | PASS | 58.3 | 4 Messpunkte |
| 4-bench | `bench__contextra-vector__hnsw_bench` | PASS | 38.7 | 3 Messpunkte |
| 4-bench | `bench__contextra-vector__hnsw_delete_search_bench` | PASS | 28.8 | 2 Messpunkte |
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
| 5-tief | `fuzz__text__fuzz_bm25_tokenize` | PENDING | 0.0 |  |
| 5-tief | `fuzz__text__fuzz_tokenizer_smoke` | PENDING | 0.0 |  |
| 5-tief | `fuzz__crypto__wal_hmac_chain_verify_fuzz` | PENDING | 0.0 |  |
| 5-tief | `fuzz__vector__fuzz_hnsw_persistence` | PENDING | 0.0 |  |
| 5-tief | `fuzz__vector__hnsw_insert_search` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_compaction_interleave` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_manifest_load` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_memtable_concurrent` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_recovery_arbitrary_state` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_sstable_binary_search` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__fuzz_wal_replay` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__wal_mutation_chaos` | PENDING | 0.0 |  |
| 5-tief | `fuzz__store__wal_roundtrip` | PENDING | 0.0 |  |
| 5-tief | `mutation_tier0_1` | PENDING | 0.0 |  |
| 5-tief | `loom_mvcc` | PENDING | 0.0 |  |
| 5-tief | `miri_vector_scalar` | PENDING | 0.0 |  |
| 5-tief | `asan_vector` | PENDING | 0.0 |  |
| 5-tief | `coverage` | PENDING | 0.0 |  |
| S-spec | `spec_gates_statisch` | PENDING | 0.0 |  |
| 6-spec | `spec_budgets_mutation` | PENDING | 0.0 |  |
