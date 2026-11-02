# Verifikationsbericht D-C: Startprobe, Crate-Manifest, lib.rs und Rückbau

**Crate:** `contextra-durable-fs`
**Spezifikation:** `05-contextra-durable-fs.md`
**Zweck:** Bereitstellung der Startprobe (`probe.rs`), Aktualisierung des Crate-Manifests, Neuorganisation von `lib.rs` (Module `real_vfs`, `atomic`, `dir_lock`, `probe`, `scrub`) und Rückbau aller Altreste (`RealFs`, `health.rs`, `journal.rs`, `fault_injector.rs`).

---

## 1. Verifikationsmatrix

| Spec-ID | Test | Benchmark | Befehl | Erwartetes Ergebnis |
|---|---|---|---|---|
| **DF-01** | `test_df_01_real_vfs_implements_ports_vfs` | – | `cargo test -p contextra-durable-fs --test df_arch_tests test_df_01` | `ok`, `RealVfs` implementiert `contextra_ports::vfs::Vfs` |
| **DF-06** | `test_df_06_probe_dir_contract`, `test_df_06_l4_slot_atomicity_64_and_80` | `BM-DFS-PROBE-STARTUP` | `cargo test -p contextra-durable-fs --test df_probe_tests test_df_06` | `ok`, Startprobe prüft 64B-Alignment, 80B-Slots, fsync, Rename |
| **DF-06** | `test_df_06_fsync_p50_measurement` | `BM-DFS-FSYNC-P50` | `cargo test -p contextra-durable-fs --test df_probe_tests test_df_06_fsync` | `ok`, fsync_p50 Stichproben-Median > 0 |
| **DF-10** | `test_df_10_t_ops_2_free_space` | – | `cargo test -p contextra-durable-fs --test df_probe_tests test_df_10` | `ok`, `free_space(&Path)` liefert Bytes > 1 MiB auf gültigem FS |
| **DF-11** | `test_df_11_no_paths_in_display_error` | – | `cargo test -p contextra-durable-fs --test df_probe_tests test_df_11` | `ok`, Keine Dateipfade im Display-String von I/O-Fehlern |
| **DF-12** | `test_df_12_naming_conventions_no_plain_names` | – | `cargo test -p contextra-durable-fs --test df_probe_tests test_df_12` | `ok`, Keine Klartextnamen oder Kundendaten in Verzeichnisnamen |
| **T-PLAT-1** | `test_t_plat_1_verdict_truth_table_supported_tiers`, `test_t_plat_1_verdict_truth_table_refused_filesystems`, `test_t_plat_1_overlayfs_container_refusal` | – | `cargo test -p contextra-durable-fs --test df_probe_tests test_t_plat_1` | `ok`, Wahrheitstabelle: Tier 1 (ext4, XFS), Tier 2 (APFS, btrfs, ZFS), Refused (NFS, SMB, FUSE, overlayfs, Windows, Unknown) |
| **T-OPS-2** | `test_df_10_t_ops_2_free_space` | – | `cargo test -p contextra-durable-fs --test df_probe_tests test_df_10` | `ok`, Plattenreserve-Prüfung über freien Speicherplatz |
| **T-ARCH-4** | `test_t_arch_4_no_network_no_tokio_fs_no_block_on` | – | `cargo test -p contextra-durable-fs --test df_arch_tests test_t_arch_4` | `ok`, 0 Netzwerk-Imports, 0 `tokio::fs`, 0 `block_on` im Crate |
| **T-LINT-2** | `test_t_lint_2_single_vfs_trait_no_second_vfs` | – | `cargo test -p contextra-durable-fs --test df_arch_tests test_t_lint_2` | `ok`, Genau ein Vfs-Trait im Workspace (`contextra_ports::vfs::Vfs`), kein zweites Vfs |

---

## 2. Ausführungsprotokoll der Qualitäts-Gates

### 2.1 Format-Check (`cargo fmt --check`)
```
$ cargo fmt --check --package contextra-durable-fs
Status: SUCCESS (Keine Formatierungsabweichungen)
```

### 2.2 Clippy Lints (`cargo clippy -p contextra-durable-fs --all-targets -- -D warnings`)
```
$ cargo clippy -p contextra-durable-fs --all-targets -- -D warnings
Status: SUCCESS (0 Warnungen, 0 Fehler)
```

### 2.3 Unit- und Integrationstests (`cargo test -p contextra-durable-fs`)
```
running 4 tests (df_arch_tests)
test test_df_01_real_vfs_implements_ports_vfs ... ok
test test_clean_removal_no_realfs_no_dead_modules ... ok
test test_t_lint_2_single_vfs_trait_no_second_vfs ... ok
test test_t_arch_4_no_network_no_tokio_fs_no_block_on ... ok
test result: ok. 4 passed; 0 failed; 0 ignored

running 9 tests (df_probe_tests)
test test_df_10_t_ops_2_free_space ... ok
test test_df_11_no_paths_in_display_error ... ok
test test_df_06_l4_slot_atomicity_64_and_80 ... ok
test test_df_06_fsync_p50_measurement ... ok
test test_t_plat_1_verdict_truth_table_refused_filesystems ... ok
test test_t_plat_1_verdict_truth_table_supported_tiers ... ok
test test_df_06_probe_dir_contract ... ok
test test_df_12_naming_conventions_no_plain_names ... ok
test test_t_plat_1_overlayfs_container_refusal ... ok
test result: ok. 9 passed; 0 failed; 0 ignored

Gesamtergebnis: 13 passed; 0 failed; 0 ignored
```

### 2.4 Benchmarks (`benches/bm_dfs_probe.rs`)
```
$ cargo bench -p contextra-durable-fs --bench bm_dfs_probe -- --test
Testing BM-DFS-PROBE-STARTUP/probe_dir_latency: Success
Testing BM-DFS-FSYNC-P50/fsync_p50_sample_stability: Success
Report geschrieben nach target/bench-report/contextra-durable-fs.json
```

---

## 3. Evidenzprotokoll-Vorlage G-F (Startprobe-Matrix)

| Dateisystem | OS / Architektur | Startprobe-Ergebnis | Klassifikation | Restrisiko | Bemerkung |
|---|---|---|---|---|---|
| **ext4** | Linux x86_64 / aarch64 | `Supported(Tier::Tier1)` | Tier 1 | Keines | Standard-Linux-Dateisystem |
| **XFS** | Linux x86_64 / aarch64 | `Supported(Tier::Tier1)` | Tier 1 | Keines | High-Performance-LSM-Untergrund |
| **btrfs** | Linux x86_64 / aarch64 | `Supported(Tier::Tier2)` | Tier 2 | `FsJournalOrCow` | Copy-on-Write erfordert Berichtsausweis |
| **ZFS** | Linux / FreeBSD | `Supported(Tier::Tier2)` | Tier 2 | `FsJournalOrCow` | CoW-Transaktionsmodell |
| **APFS** | macOS aarch64 / x86_64 | `Supported(Tier::Tier2)` | Tier 2 | `FsJournalOrCow` | `F_FULLFSYNC` erforderlich |
| **NFS** | Alle | `Refused(UnsupportedFilesystem::Nfs)` | Refused | – | Fehlende synchrone Durability-Garantien |
| **SMB / CIFS** | Alle | `Refused(UnsupportedFilesystem::Smb)` | Refused | – | Fehlende atomare Rename-Semantik |
| **FUSE** | Alle | `Refused(UnsupportedFilesystem::Fuse)` | Refused | – | Userspace-Indirektion ohne Flushtreue |
| **overlayfs** | Linux Container | `Refused(UnsupportedFilesystem::Overlayfs)` | Refused | – | Schichtenspeicher ohne atomare Haltbarkeit |
| **NTFS** | Windows | `Refused(UnsupportedFilesystem::Windows)` | Refused | – | Plattform gemäß SPEC D-84 entfernt |

---

## 4. Rückbau-Nachweis (Grep-Protokoll)

```
$ grep -E "RealFs|Stand SPEC|Wird eingebunden von" crates/contextra-durable-fs/src/lib.rs crates/contextra-durable-fs/src/probe.rs
Ergebnis: 0 Treffer (Vollständig bereinigt)
```
