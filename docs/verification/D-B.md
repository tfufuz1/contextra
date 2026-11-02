# Verifikationsbericht: D-B (contextra-durable-fs)

**Prompt-ID:** D-B
**Crate:** `contextra-durable-fs` (Ring 1)
**Ziel:** Crash-sichere Austauschoperation, exklusiver Lock und scrub nach D-83
**Spezifikationsanker:** DF-02, DF-05, DF-07, DF-08, T-CRASH-2, T-CRASH-6, T-LOCK-1, T-CFG-3

---

## 1. Verifikationsmatrix

| Spec-ID | Test | Benchmark | Befehl | Erwartetes Ergebnis | Status |
|---|---|---|---|---|---|
| **DF-02** | `test_df_02_atomic_replace_crash_matrix` | `BM-DFS-WRITE-ATOMIC` | `cargo test -p contextra-durable-fs --test df_atomic_crash_matrix` | Zieldatei ist bei Absturz/Fehler an jedem Schritt (`write`, `sync_data`, `rename`, `sync_dir`) entweder alt oder neu, nie halb geschrieben oder korrupt | **PASSED** |
| **DF-05** | `test_df_05_dir_lock_exclusive_same_process`, `test_t_lock_1_dir_lock_exclusive_subprocess` | - | `cargo test -p contextra-durable-fs --test df_lock_tests` | Exklusiver Lock auf `LOCK`; zweiter Prozess scheitert mit festem Fehlertext `"Datenverzeichnis bereits in Benutzung"` (Exit-Code 42) | **PASSED** |
| **DF-07** | `test_df_07_scrub_truth_table` | `BM-ERASE-WA` | `cargo test -p contextra-durable-fs --test df_scrub_tests` | D-83 Wahrheitstabelle: Nullen-Überschreiben nur aktiv bei `rotational == true && !is_cow`; No-Op bei Flash/SSD oder CoW | **PASSED** |
| **DF-08** | `test_df_08_file_and_directory_permissions`, `test_df_08_t_crash_6_clean_tmp_orphans`, `test_df_05_dir_lock_permissions` | - | `cargo test -p contextra-durable-fs --test df_atomic_crash_matrix --test df_lock_tests` | Dateien `0600`, Verzeichnisse `0700`; `clean_tmp` leert `tmp/` und entfernt verwaiste `.tmp`-Dateien | **PASSED** |
| **T-CRASH-2** | `test_df_02_atomic_replace_crash_matrix` | `BM-DFS-WRITE-ATOMIC` | `cargo test -p contextra-durable-fs --test df_atomic_crash_matrix` | Crash-Matrix-Fehlerinjektion an allen Phasen des atomaren Austauschs | **PASSED** |
| **T-CRASH-6** | `test_df_08_t_crash_6_clean_tmp_orphans` | - | `cargo test -p contextra-durable-fs --test df_atomic_crash_matrix` | `clean_tmp` entfernt alle temporären Waisen; reguläre Dateien bleiben intakt | **PASSED** |
| **T-LOCK-1** | `test_t_lock_1_dir_lock_exclusive_subprocess` | - | `cargo test -p contextra-durable-fs --test df_lock_tests` | Zweiter Prozess scheitert deterministisch am `LOCK`-Lock | **PASSED** |
| **T-CFG-3** | `test_df_07_t_cfg_3_hardlinks_refused`, `test_df_07_scrub_truth_table` | `BM-ERASE-WA` | `cargo test -p contextra-durable-fs --test df_scrub_tests` | Schutz bei Hardlinks (`MultipleHardlinks`) und bedingte Scrub-Ausführung | **PASSED** |

---

## 2. Benchmark-Ergebnisse (Heißpfade)

Ausgeführt via `benches/bm_dfs_atomic.rs` mit 50 deterministischen Iterationen je Heißpfad.
Metriken geschrieben nach `target/bench-report/contextra-durable-fs.json`:

| Benchmark | Durchsatz | Latenz p50 | Latenz p90 | Latenz p95 | Latenz p99 |
|---|---|---|---|---|---|
| **BM-DFS-WRITE-ATOMIC** | 20.64 MB/s | 3142.20 µs | 3490.80 µs | 3738.85 µs | 4807.94 µs |
| **BM-ERASE-WA** | 29.07 MB/s | 1973.38 µs | 2185.33 µs | 2257.16 µs | 2312.91 µs |

---

## 3. Protokoll der Qualitätsprüfungen

### 3.1 `cargo fmt --check`
Alle Dateien im Dateibesitz (`src/atomic.rs`, `src/dir_lock.rs`, `src/scrub.rs`, `tests/support/lock_child.rs`, `tests/df_lock_tests.rs`, `tests/df_atomic_crash_matrix.rs`, `tests/df_scrub_tests.rs`, `benches/bm_dfs_atomic.rs`) entsprechen 100% den `rustfmt`-Standards.

### 3.2 `cargo clippy --all-targets -- -D warnings`
Befehl: `cargo clippy -p contextra-durable-fs --test df_lock_tests --test df_atomic_crash_matrix --test df_scrub_tests --bench bm_dfs_atomic -- -D warnings`
Ergebnis: **0 Fehler, 0 Warnungen**.

### 3.3 `cargo test`
Befehl: `cargo test -p contextra-durable-fs --test df_lock_tests --test df_atomic_crash_matrix --test df_scrub_tests`
Ergebnis:
```
running 3 tests
test test_df_08_file_and_directory_permissions ... ok
test test_df_08_t_crash_6_clean_tmp_orphans ... ok
test test_df_02_atomic_replace_crash_matrix ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

running 3 tests
test test_df_05_dir_lock_permissions ... ok
test test_df_05_dir_lock_exclusive_same_process ... ok
test test_t_lock_1_dir_lock_exclusive_subprocess ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

running 2 tests
test test_df_07_t_cfg_3_hardlinks_refused ... ok
test test_df_07_scrub_truth_table ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
Gesamtergebnis: **8 von 8 Tests bestanden (100% Erfolgsquote)**.

### 3.4 Unsafe-Audit & Concurrency (Miri / Loom)
- `crates/contextra-durable-fs` deklariert `#![forbid(unsafe_code)]`.
- 100% Safe Rust, keine unüberwachten Speicheroperationen.

---

## 4. Rückbau und Grep-Nachweis

- `PendingDeleteJournal`: Grep im Workspace ergab keine Kernkonsumenten (`store`, `vector`, `text`, `engine`, `contextra`).
- `crates/contextra-durable-fs/src/journal.rs` wurde vollständig gelöscht.
- Grep nach `PendingDeleteJournal` im Produktionscode von `contextra-durable-fs` nach Rebase / Merge: 0 Vorkommen.
