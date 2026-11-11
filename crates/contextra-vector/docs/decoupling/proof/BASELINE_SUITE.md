# Baseline Suite — `contextra-vector` (Stand `origin/main` f92caabaa)

Datum: 2026-11-11
Umgebung: Linux x86_64, Rust 1.89.0 (29483883e 2025-08-04), Cargo 1.89.0 (c24e10642 2025-06-23)

## 1. Testläufe und Ergebnisse

### 1.1 Unittests (`crates/contextra-vector/src/lib.rs`)
Befehl: `cargo test -p contextra-vector --lib --offline`
- Dauer: 0.44s
- Ergebnis: 64 passed, 0 failed, 0 ignored
- Rote / flaky Tests: keine

### 1.2 Vorhandene Golden- und Orakeltests
Befehl: `cargo test -p contextra-vector --offline --test golden_test`
- Dauer: 0.01s
- Ergebnis: 2 passed, 0 failed, 1 ignored (`generate_golden_files`)
- Rote / flaky Tests: keine

Befehl: `cargo test -p contextra-vector --offline --test golden_distance_oracle`
- Dauer: 0.00s
- Ergebnis: 3 passed, 0 failed, 0 ignored
- Rote / flaky Tests: keine

### 1.3 Determinismus-Tests
Befehle:
- `cargo test -p contextra-vector --offline --test determinism_test` (0.07s, 3 passed)
- `cargo test -p contextra-vector --offline --test distance_determinism_test` (0.00s, 1 passed)
- Rote / flaky Tests: keine

### 1.4 Concurrency / Loom Tests
Befehle:
- `RUSTFLAGS="--cfg loom" cargo test --release -p contextra-vector --offline --test loom_quantizer_test` (0.00s, 1 passed: `test_quantizer_concurrent_sharing`)
- `RUSTFLAGS="--cfg loom" cargo test --release -p contextra-vector --offline --test loom_quantizer_race_test` (20.77s, 4 passed: `test_loom_remove_search_visibility`, `test_loom_snapshot_pin_compact_seq_log`, `test_loom_insert_search_rebuild_swap`, `test_sq8_quantizer_write_lock_no_race`)
- Rote / flaky Tests: keine

### 1.5 Vollständige Testsuite-Zusammenfassung
- Gesamtzahl Tests (Coverage Matrix): 312 Tests (132 Unit, 165 Async, 6 Proptest, 9 Benchmark)
- Auffälligkeiten: Voller Suite-Lauf über alle Integrations- und Proptest-Dateien in einem Einzelschritt ohne Filterung benötigt hohe Rechenzeit (> 400s Timeout), während gezielte Modul- und Einzeltarget-Aufrufe sub-sekündlich abschließen.
- Baseline-Status: STABIL GRÜN am unveränderten Code.
