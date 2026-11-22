# Contextra Voll-Audit

- Commit: `ed402e21a` (2026-10-30 01:38:58 +0100), Branch `main`, hinter origin: 0, ungespeicherte Aenderungen: True
- Start: 2026-10-30T01:13:30+00:00  Dauer: 11.0 min
- Toolchain: rustc 1.89.0 (29483883e 2025-08-04)
- Host: Linux-7.2.7-1-liquorix-amd64-x86_64-with-glibc2.41 | 8 CPUs | MemTotal:       10121016 kB
- Nutzer: freddy

## Gesamt

- **FAIL**: 59
- **INFO**: 2
- **PASS**: 45
- **PENDING**: 62

Hinweis: Bei `befund_*`-Schritten bedeutet FAIL, dass der Befund **vorliegt**. `INFO` ist eine Messung ohne Bestanden/Durchgefallen.

## Rote und uebersprungene Schritte

| Schritt | Status | Sek | Detail | Log |
|---|---|---:|---|---|
| `befund_cfg_features_undefiniert` | FAIL | 0.2 | 5 Crates mit undefinierten Features | `logs/000_befund_cfg_features_undefiniert.log` |
| `befund_wal_aead_tail_widerspruch` | FAIL | 0.0 | tolerierend Z.[475, 607], ablehnend Z.[527] | `logs/001_befund_wal_aead_tail_widerspruch.log` |
| `befund_safe_fn_mit_unsafe_inhalt` | FAIL | 0.0 | 3 safe-Signaturen | `logs/002_befund_safe_fn_mit_unsafe_inhalt.log` |
| `befund_ingest_verdrahtung` | FAIL | 0.0 | kein Crate haengt von contextra-ingest ab | `logs/003_befund_ingest_verdrahtung.log` |
| `befund_default_members` | FAIL | 0.0 | members=16 default-members=13; Produkt fehlt in default-members: ['contextra-ingest', 'contextra-privacy', 'contextra-infer-ollama', 'contextra-audit-export', 'contextra-testkit', 'contextra-avv-generator']; Kern in default-members: 8 | `logs/004_befund_default_members.log` |
| `befund_adr_status` | FAIL | 0.0 | ADRs=['0001', '0002', '0003', '0004', '0005', '0006', '0007', '0008', '0077', '0098', '0099']; Proposed=['0006', '0007', '0008']; naechste freie Nummer<1000: 0100 | `logs/005_befund_adr_status.log` |
| `befund_from_entropy` | FAIL | 0.0 | 1 Dateien mit from_entropy (K14) | `logs/006_befund_from_entropy.log` |
| `fmt_check` | FAIL | 2.5 |  | `logs/015_fmt_check.log` |
| `check_alle_targets` | FAIL | 10.5 | errors=8 | `logs/016_check_alle_targets.log` |
| `check_no_default_features` | FAIL | 14.1 | errors=3 | `logs/017_check_no_default_features.log` |
| `doc` | FAIL | 5.0 |  | `logs/018_doc.log` |
| `clippy_lib__contextra` | FAIL | 4.2 | errors=67 warnings=1 | `logs/019_clippy_lib__contextra.log` |
| `clippy_alle_targets__contextra` | FAIL | 2.5 | errors=67 warnings=1 | `logs/020_clippy_alle_targets__contextra.log` |
| `clippy_lib__contextra-bench` | FAIL | 4.3 | errors=67 warnings=1 | `logs/021_clippy_lib__contextra-bench.log` |
| `clippy_alle_targets__contextra-bench` | FAIL | 5.8 | errors=67 warnings=1 | `logs/022_clippy_alle_targets__contextra-bench.log` |
| `clippy_lib__contextra-crypto` | FAIL | 2.6 | errors=3 warnings=0 | `logs/023_clippy_lib__contextra-crypto.log` |
| `clippy_alle_targets__contextra-crypto` | FAIL | 2.7 | errors=3 warnings=0 | `logs/024_clippy_alle_targets__contextra-crypto.log` |
| `clippy_lib__contextra-db` | FAIL | 1.7 | errors=67 warnings=1 | `logs/025_clippy_lib__contextra-db.log` |
| `clippy_alle_targets__contextra-db` | FAIL | 4.5 | errors=67 warnings=1 | `logs/026_clippy_alle_targets__contextra-db.log` |
| `clippy_lib__contextra-engine` | FAIL | 1.8 | errors=67 warnings=1 | `logs/027_clippy_lib__contextra-engine.log` |
| `clippy_alle_targets__contextra-engine` | FAIL | 6.4 | errors=67 warnings=1 | `logs/028_clippy_alle_targets__contextra-engine.log` |
| `clippy_alle_targets__contextra-rank` | FAIL | 4.0 | errors=5 warnings=1 | `logs/034_clippy_alle_targets__contextra-rank.log` |
| `clippy_lib__contextra-store` | FAIL | 1.6 | errors=3 warnings=0 | `logs/037_clippy_lib__contextra-store.log` |
| `clippy_alle_targets__contextra-store` | FAIL | 1.4 | errors=3 warnings=1 | `logs/038_clippy_alle_targets__contextra-store.log` |
| `clippy_lib__contextra-text` | FAIL | 4.9 | errors=64 warnings=0 | `logs/043_clippy_lib__contextra-text.log` |
| `clippy_alle_targets__contextra-text` | FAIL | 5.1 | errors=188 warnings=1 | `logs/044_clippy_alle_targets__contextra-text.log` |
| `clippy_lib__contextra-vector` | FAIL | 4.3 | errors=3 warnings=0 | `logs/047_clippy_lib__contextra-vector.log` |
| `clippy_alle_targets__contextra-vector` | FAIL | 1.6 | errors=3 warnings=0 | `logs/048_clippy_alle_targets__contextra-vector.log` |
| `feature_matrix_hack` | FAIL | 36.6 |  | `logs/051_feature_matrix_hack.log` |
| `deny` | FAIL | 1.4 |  | `logs/052_deny.log` |
| `panic_flaeche_lib` | FAIL | 14.2 | errors=3 warnings=64 | `logs/054_panic_flaeche_lib.log` |
| `unsafe_inventar` | FAIL | 0.1 |  | `logs/055_unsafe_inventar.log` |
| `test_lib__contextra` | FAIL | 2.7 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/056_test_lib__contextra.log` |
| `test_integration__contextra` | FAIL | 0.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/057_test_integration__contextra.log` |
| `test_doc__contextra` | FAIL | 0.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/058_test_doc__contextra.log` |
| `nextest__contextra` | FAIL | 1.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/059_nextest__contextra.log` |
| `test_lib__contextra-bench` | FAIL | 36.0 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/060_test_lib__contextra-bench.log` |
| `test_integration__contextra-bench` | FAIL | 0.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/061_test_integration__contextra-bench.log` |
| `test_doc__contextra-bench` | FAIL | 0.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/062_test_doc__contextra-bench.log` |
| `nextest__contextra-bench` | FAIL | 1.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/063_nextest__contextra-bench.log` |
| `test_lib__contextra-crypto` | FAIL | 23.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/064_test_lib__contextra-crypto.log` |
| `test_integration__contextra-crypto` | FAIL | 7.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/065_test_integration__contextra-crypto.log` |
| `test_doc__contextra-crypto` | FAIL | 0.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/066_test_doc__contextra-crypto.log` |
| `nextest__contextra-crypto` | FAIL | 1.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/067_nextest__contextra-crypto.log` |
| `test_lib__contextra-db` | FAIL | 20.3 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/068_test_lib__contextra-db.log` |
| `test_integration__contextra-db` | FAIL | 0.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/069_test_integration__contextra-db.log` |
| `test_doc__contextra-db` | FAIL | 0.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/070_test_doc__contextra-db.log` |
| `nextest__contextra-db` | FAIL | 1.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/071_nextest__contextra-db.log` |
| `test_lib__contextra-engine` | FAIL | 16.3 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/072_test_lib__contextra-engine.log` |
| `test_integration__contextra-engine` | FAIL | 1.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/073_test_integration__contextra-engine.log` |
| `test_doc__contextra-engine` | FAIL | 0.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/074_test_doc__contextra-engine.log` |
| `nextest__contextra-engine` | FAIL | 1.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/075_nextest__contextra-engine.log` |
| `test_integration__contextra-rank` | FAIL | 6.7 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/085_test_integration__contextra-rank.log` |
| `nextest__contextra-rank` | FAIL | 3.4 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/087_nextest__contextra-rank.log` |
| `test_lib__contextra-store` | FAIL | 8.4 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/092_test_lib__contextra-store.log` |
| `test_integration__contextra-store` | FAIL | 1.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/093_test_integration__contextra-store.log` |
| `test_doc__contextra-store` | FAIL | 0.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/094_test_doc__contextra-store.log` |
| `nextest__contextra-store` | FAIL | 1.4 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/095_nextest__contextra-store.log` |
| `test_integration__contextra-text` | FAIL | 8.7 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER | `logs/105_test_integration__contextra-text.log` |

## Alle Schritte

| Gruppe | Schritt | Status | Sek | Detail |
|---|---|---|---:|---|
| B-befunde | `befund_cfg_features_undefiniert` | FAIL | 0.2 | 5 Crates mit undefinierten Features |
| B-befunde | `befund_wal_aead_tail_widerspruch` | FAIL | 0.0 | tolerierend Z.[475, 607], ablehnend Z.[527] |
| B-befunde | `befund_safe_fn_mit_unsafe_inhalt` | FAIL | 0.0 | 3 safe-Signaturen |
| B-befunde | `befund_ingest_verdrahtung` | FAIL | 0.0 | kein Crate haengt von contextra-ingest ab |
| B-befunde | `befund_default_members` | FAIL | 0.0 | members=16 default-members=13; Produkt fehlt in default-members: ['contextra-ingest', 'contextra-privacy', 'contextra-infer-ollama', 'contextra-audit-export', 'contextra-testkit', 'contextra-avv-generator']; Kern in default-members: 8 |
| B-befunde | `befund_adr_status` | FAIL | 0.0 | ADRs=['0001', '0002', '0003', '0004', '0005', '0006', '0007', '0008', '0077', '0098', '0099']; Proposed=['0006', '0007', '0008']; naechste freie Nummer<1000: 0100 |
| B-befunde | `befund_from_entropy` | FAIL | 0.0 | 1 Dateien mit from_entropy (K14) |
| B-befunde | `befund_maturity_claims` | INFO | 0.0 | stable=15 beta=12 |
| B-befunde | `befund_windows_flush` | INFO | 0.0 | windows-Pfad vorhanden=True, compile_error!=False |
| T1-produkt | `produkt_huelle_ohne_kern` | PASS | 0.8 | 0 Produkt-Crates mit verbotener Huelle |
| T1-produkt | `produkt_sqlcipher_probe` | PASS | 2.5 | cipher=Some("4.19.0 community") fts5=true leak=false wrong_key_rejected=true rows=1 |
| T1-produkt | `produkt_crates_io_versionen` | PASS | 3.7 |  |
| 0-basis | `toolchain_versionen` | PASS | 0.8 |  |
| 0-basis | `cargo_fetch` | PASS | 0.4 |  |
| 0-basis | `cargo_tree_duplicates` | PASS | 0.3 |  |
| 1-statisch | `fmt_check` | FAIL | 2.5 |  |
| 1-statisch | `check_alle_targets` | FAIL | 10.5 | errors=8 |
| 1-statisch | `check_no_default_features` | FAIL | 14.1 | errors=3 |
| 1-statisch | `doc` | FAIL | 5.0 |  |
| 1-statisch | `clippy_lib__contextra` | FAIL | 4.2 | errors=67 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra` | FAIL | 2.5 | errors=67 warnings=1 |
| 1-statisch | `clippy_lib__contextra-bench` | FAIL | 4.3 | errors=67 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-bench` | FAIL | 5.8 | errors=67 warnings=1 |
| 1-statisch | `clippy_lib__contextra-crypto` | FAIL | 2.6 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-crypto` | FAIL | 2.7 | errors=3 warnings=0 |
| 1-statisch | `clippy_lib__contextra-db` | FAIL | 1.7 | errors=67 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-db` | FAIL | 4.5 | errors=67 warnings=1 |
| 1-statisch | `clippy_lib__contextra-engine` | FAIL | 1.8 | errors=67 warnings=1 |
| 1-statisch | `clippy_alle_targets__contextra-engine` | FAIL | 6.4 | errors=67 warnings=1 |
| 1-statisch | `clippy_lib__contextra-mvcc` | PASS | 2.4 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-mvcc` | PASS | 3.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-ports` | PASS | 4.6 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-ports` | PASS | 2.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-rank` | PASS | 2.7 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-rank` | FAIL | 4.0 | errors=5 warnings=1 |
| 1-statisch | `clippy_lib__contextra-simd` | PASS | 5.9 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-simd` | PASS | 2.0 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-store` | FAIL | 1.6 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-store` | FAIL | 1.4 | errors=3 warnings=1 |
| 1-statisch | `clippy_lib__contextra-sys` | PASS | 0.4 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-sys` | PASS | 0.5 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-testkit` | PASS | 1.3 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-testkit` | PASS | 0.7 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-text` | FAIL | 4.9 | errors=64 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-text` | FAIL | 5.1 | errors=188 warnings=1 |
| 1-statisch | `clippy_lib__contextra-types` | PASS | 1.4 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-types` | PASS | 2.2 | errors=0 warnings=0 |
| 1-statisch | `clippy_lib__contextra-vector` | FAIL | 4.3 | errors=3 warnings=0 |
| 1-statisch | `clippy_alle_targets__contextra-vector` | FAIL | 1.6 | errors=3 warnings=0 |
| 1-statisch | `clippy_lib__xtask` | PASS | 5.1 | errors=0 warnings=0 |
| 1-statisch | `clippy_alle_targets__xtask` | PASS | 6.9 | errors=0 warnings=0 |
| 1-statisch | `feature_matrix_hack` | FAIL | 36.6 |  |
| 1-statisch | `deny` | FAIL | 1.4 |  |
| 1-statisch | `audit` | PASS | 3.5 |  |
| 1-statisch | `panic_flaeche_lib` | FAIL | 14.2 | errors=3 warnings=64 |
| 1-statisch | `unsafe_inventar` | FAIL | 0.1 |  |
| 2-tests | `test_lib__contextra` | FAIL | 2.7 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra` | FAIL | 0.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra` | FAIL | 0.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `nextest__contextra` | FAIL | 1.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_lib__contextra-bench` | FAIL | 36.0 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-bench` | FAIL | 0.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-bench` | FAIL | 0.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `nextest__contextra-bench` | FAIL | 1.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_lib__contextra-crypto` | FAIL | 23.6 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-crypto` | FAIL | 7.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-crypto` | FAIL | 0.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `nextest__contextra-crypto` | FAIL | 1.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_lib__contextra-db` | FAIL | 20.3 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-db` | FAIL | 0.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-db` | FAIL | 0.8 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `nextest__contextra-db` | FAIL | 1.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_lib__contextra-engine` | FAIL | 16.3 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-engine` | FAIL | 1.1 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-engine` | FAIL | 0.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `nextest__contextra-engine` | FAIL | 1.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_lib__contextra-mvcc` | PASS | 27.7 | passed=47 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-mvcc` | PASS | 26.4 | passed=106 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-mvcc` | PASS | 0.4 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-mvcc` | PASS | 10.6 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_lib__contextra-ports` | PASS | 11.9 | passed=42 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-ports` | PASS | 3.5 | passed=44 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-ports` | PASS | 0.6 | passed=3 failed=0 ignored=0 |
| 2-tests | `nextest__contextra-ports` | PASS | 2.3 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_lib__contextra-rank` | PASS | 13.1 | passed=8 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-rank` | FAIL | 6.7 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-rank` | PASS | 2.2 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-rank` | FAIL | 3.4 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_lib__contextra-simd` | PASS | 10.6 | passed=13 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-simd` | PASS | 7.5 | passed=77 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-simd` | PASS | 2.4 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-simd` | PASS | 5.7 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_lib__contextra-store` | FAIL | 8.4 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_integration__contextra-store` | FAIL | 1.2 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_doc__contextra-store` | FAIL | 0.9 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `nextest__contextra-store` | FAIL | 1.4 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
| 2-tests | `test_lib__contextra-sys` | PASS | 3.4 | passed=30 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-sys` | PASS | 0.2 | passed=30 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-sys` | PASS | 0.6 | passed=0 failed=0 ignored=0 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-sys` | PASS | 0.6 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_lib__contextra-testkit` | PASS | 6.5 | passed=11 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-testkit` | PASS | 8.0 | passed=25 failed=0 ignored=0 |
| 2-tests | `test_doc__contextra-testkit` | PASS | 0.4 | passed=0 failed=0 ignored=1 | KEINE TESTS AUSGEFUEHRT |
| 2-tests | `nextest__contextra-testkit` | PASS | 0.6 | passed=0 failed=0 ignored=0 |
| 2-tests | `test_lib__contextra-text` | PASS | 167.3 | passed=106 failed=0 ignored=0 |
| 2-tests | `test_integration__contextra-text` | FAIL | 8.7 | passed=0 failed=0 ignored=0 | COMPILE-FEHLER |
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
| 2-tests | `test_lib__xtask` | PENDING | 0.0 |  |
| 2-tests | `test_integration__xtask` | PENDING | 0.0 |  |
| 2-tests | `test_doc__xtask` | PENDING | 0.0 |  |
| 2-tests | `nextest__xtask` | PENDING | 0.0 |  |
| 2-tests | `test_gesamt_debug` | PENDING | 0.0 |  |
| 2-tests | `test_gesamt_release` | PENDING | 0.0 |  |
| 3-gates | `xtask_gate_liste` | PENDING | 0.0 |  |
| 3-gates | `xtask_check_compile` | PENDING | 0.0 |  |
| 3-gates | `xtask_check_alle` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-mvcc__mvcc_bench` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-crypto__crypto_benchmarks` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-crypto__deletion_proof_latency_bench` | PENDING | 0.0 |  |
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
| 4-bench | `bench__contextra-bench__competitive_kv` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-bench__locomo` | PENDING | 0.0 |  |
| 4-bench | `bench__contextra-bench__long_mem_eval` | PENDING | 0.0 |  |
| 5-tief | `fuzz__text__fuzz_bm25_tokenize` | PENDING | 0.0 |  |
| 5-tief | `fuzz__text__fuzz_tokenizer_smoke` | PENDING | 0.0 |  |
| 5-tief | `fuzz__db__rrf_fusion` | PENDING | 0.0 |  |
| 5-tief | `fuzz__crypto__deletion_proof_tamper` | PENDING | 0.0 |  |
| 5-tief | `fuzz__crypto__deletion_proof_verify_external_fuzz` | PENDING | 0.0 |  |
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
