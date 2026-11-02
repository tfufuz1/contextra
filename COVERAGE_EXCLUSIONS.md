# Coverage Exclusions (`COVERAGE_EXCLUSIONS.md`)

## Crate: `contextra-durable-fs` (Prompt D-C)

In accordance with Rule 6 and quality requirements, this document registers and justifies all code paths and branches that cannot be exercised on a standard Linux x86_64 container build machine:

| Datei | Pfad / Block | Begründung |
|---|---|---|
| `crates/contextra-durable-fs/src/probe.rs` | `#[cfg(target_os = "macos")]` (APFS Erkennung und `df -k` Parsing) | Plattformabhängiger Codezweig: Nur auf macOS / Darwin ausführbar. Auf Linux-Buildservern per `cfg` inaktiv. |
| `crates/contextra-durable-fs/src/probe.rs` | `#[cfg(windows)]` (Windows NTFS Pfad) | Plattformabhängiger Codezweig: Windows-Support gemäß SPEC D-84 entfernt/deaktiviert. |
| `crates/contextra-durable-fs/src/probe.rs` | `#[cfg(not(target_os = "linux"))]` in `detect_rotational` | Auf Nicht-Linux-Systemen steht `/sys/block/.../queue/rotational` nicht zur Verfügung. |
| `crates/contextra-durable-fs/src/probe.rs` | Hardware-Ausfall-Zweige (`probe.rename_atomic == false`, `probe.dir_fsync_ok == false`, `probe.write_atomic_64 == false`) | Physische Dateisystem-Fehlerzustände treten auf intakter Standardhardware nicht deterministisch auf; sie werden in Mock-/Truth-Table-Tests (`test_t_plat_1_verdict_truth_table_refused_filesystems`) synthetisch abgedeckt. |
| `crates/contextra-durable-fs/src/lib.rs` | Lock poisoning fallback branches (`ErrorKind::Other("lock poisoned")`) | Tritt nur bei Panics innerhalb gehaltener Mutex-Guards auf; da Bibliotheks-Code strikt panic-frei (`#![deny(clippy::panic)]`) implementiert ist, ist dieser Zweig im Normalbetrieb unerreichbar. |
