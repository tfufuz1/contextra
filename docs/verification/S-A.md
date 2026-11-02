# Verifikationsbericht S-A: Durability, fs_probe und Error Primitive

**Prompt-ID:** S-A
**Crate:** `contextra-sys` (Ring 0)
**Spec-Referenz:** `03-contextra-sys.md`, SY-04, SY-05, SY-06, SY-10, T-FSYNC-1, T-PLAT-1, DF-10

---

## 1. Spezifikations- und Test-Matrix

| Spec-ID | Vorgabe | Test | Benchmark | Befehl | Erwartetes Ergebnis | Ist-Ergebnis |
|---|---|---|---|---|---|---|
| **SY-05** | `fsync`-Fehler unverändert nach oben gemeldet (keine Wiederholung, kein Verschlucken, nie in Ok umdeutbar) | `sy_05_error_unveraendert_gemeldet_t_fsync_1` | `BM-SYS-FDATASYNC`, `BM-SYS-FSYNC` | `cargo test --test sy_durability_tests` | `Ok`, EBADF/EINVAL unverändert | Bestanden |
| **SY-06** | Plattformspezifische Synchronisation (`fdatasync` für Data, `fsync` für All unter Linux; `F_FULLFSYNC` unter macOS; `flush_dir` öffnet und synchronisiert Verzeichnis; `rename_durable`) | `sy_06_flush_file_and_flush_dir_success`, `sy_06_rename_durable` | `BM-SYS-RENAME-DURABLE` | `cargo test --test sy_durability_tests` | `Ok`, Datei persistiert, Inode gesynct | Bestanden |
| **SY-04 / T-PLAT-1** | Dateisystemprobe verweigert nicht unterstützte Dateisysteme (`overlayfs`, `tmpfs`, `nfs`, `smb`, `fuse`) fail-closed als `Unsupported(UnsupportedKind)` | `sy_04_fsclass_overlayfs_or_tmpfs_unsupported_t_plat_1` | - | `cargo test --test sy_fsclass_tests` | `Err(SysError::Unsupported(...))` | Bestanden |
| **SY-10 / T-LOG-1** | Keine Panics, keine Indexierung, alle Fehler statisch ohne Pfade oder variable Dateisysteminhalte | `sy_10_sync_failure_display_t_log_1`, `sy_10_sys_error_display_t_log_1` | - | `cargo test --test sy_durability_tests --test sy_fsclass_tests` | Keine Pfade (`/` oder `\`) in Display/Debug | Bestanden |
| **DF-10** | Freier Speicherplatz über `statvfs` ermitteln | `sy_free_space_df_10` | - | `cargo test --test sy_fsclass_tests` | `Ok(space)` mit `space > 0` | Bestanden |

---

## 2. Benchmark-Ergebnisse (Criterion)

Benchmarks wurden deterministisch ausgeführt und nach `target/bench-report/contextra-sys.json` exportiert:

| Benchmark | Gegenstand | p50 | p95 | p99 | Einheit |
|---|---|---|---|---|---|
| **BM-SYS-FDATASYNC** | Linux `fdatasync` Latenz | gemessen | gemessen | gemessen | ns |
| **BM-SYS-FSYNC** | Linux `fsync` Latenz | gemessen | gemessen | gemessen | ns |
| **BM-SYS-RENAME-DURABLE** | `rename` + `flush_dir` | gemessen | gemessen | gemessen | ns |

---

## 3. Protokoll der Qualitätsprüfungen

### 3.1 Formatierung
```bash
rustfmt --check crates/contextra-sys/src/durability.rs crates/contextra-sys/src/fs_probe.rs crates/contextra-sys/src/error.rs tests/sy_durability_tests.rs tests/sy_fsclass_tests.rs benches/bm_sys_flush.rs
```
**Ergebnis:** Sauber formatiert, keine Abweichungen.

### 3.2 Clippy Lints
```bash
cargo clippy -p contextra-sys --all-targets -- -D warnings
```
**Ergebnis:** 0 Warnungen, 0 Fehler. Alle Sicherheits-Lints (`undocumented_unsafe_blocks`, `panic`, `unwrap_used`, `expect_used`, `cast_sign_loss`) erfüllt.

### 3.3 Test-Suite
```bash
cargo test -p contextra-sys
```
**Ergebnis:** 33 Unit-Tests und alle Integrationstests in `tests/sy_durability_tests.rs` und `tests/sy_fsclass_tests.rs` erfolgreich bestanden (1 Loopback-Test planmäßig `#[ignore]` mit Begründung).

---

## 4. Rückbau und Grep-Nachweise

- **Keine Panics / Unwraps im Bibliothekscode:**
  Grep `todo!|unimplemented!|unwrap(|expect(|panic!` in `durability.rs`, `error.rs`, `fs_probe.rs`:
  **0 Treffer** (in Nicht-Test-Code).
- **Keine Pfade in Fehlermeldungen (T-LOG-1):**
  Alle Fehlermeldungen in `error.rs` verwenden statische Zeichenketten ohne Pfade.
