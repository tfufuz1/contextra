# Extraktion fuer `acta-fs`

**Aufgabe:** Vfs-Port, DurableVfs, FaultVfs, Scrub, O_DIRECT-Ruecklesen  
**Level:** L1 · **Budget:** 2500 LOC · **Spezifikation:** docs/SPEC.md Kap. 3.7, 5.6  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 29 Dateien, 5400 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `durable-fs` | Vfs/DurableVfs/Scrub/FaultVfs (3.7, 5.6 WP-5..7) |
| `store` | fsync-/IO-Naht; FsyncFailed-Regel WP-7 |
| `testkit` | FaultVfs (Absturzinjektion, 16.2) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 2 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| async | 48 | tokio/async (G-NOTOKIO: Kern synchron) |
| zeit_zufall | 1 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 83 | unwrap/expect/panic (G-LINT) |
| std_fs | 124 | std::fs direkt (G-FS: nur acta-fs) |
| hub | 24 | Hub-Import ex types/ports (HR-2/HR-3) |
| alt_spec | 16 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `crc32fast`, `parking_lot` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-fs/`) | LOC | Befunde |
|---|---:|---|
| `durable-fs/benches/fs_bench.rs` | 45 | panic=6 |
| `durable-fs/src/error.rs` | 73 |  |
| `durable-fs/src/fault_injector.rs` | 300 |  |
| `durable-fs/src/health.rs` | 139 |  |
| `durable-fs/src/journal.rs` | 436 | std_fs=2 |
| `durable-fs/src/lib.rs` | 455 | std_fs=5 |
| `durable-fs/src/platform.rs` | 48 | std_fs=2 |
| `durable-fs/src/scrub.rs` | 318 | float=2, std_fs=3 |
| `durable-fs/src/vfs.rs` | 141 | std_fs=3 |
| `durable-fs/tests/basic.rs` | 50 | std_fs=2 |
| `durable-fs/tests/crash_matrix_exhaustive.rs` | 226 | panic=24, std_fs=2 |
| `durable-fs/tests/durability_tests.rs` | 92 | std_fs=6 |
| `durable-fs/tests/fault_injector_tests.rs` | 310 | std_fs=1 |
| `durable-fs/tests/inv_crash_matrix.rs` | 74 | panic=14, std_fs=1 |
| `durable-fs/tests/journal_crash_resume.rs` | 59 | std_fs=1 |
| `durable-fs/tests/journal_property_tests.rs` | 119 | panic=23, std_fs=1 |
| `durable-fs/tests/p03_health_state_tests.rs` | 83 |  |
| `durable-fs/tests/scrub_and_remove.rs` | 140 | std_fs=1 |
| `durable-fs/tests/scrub_journal_tests.rs` | 111 | std_fs=6 |
| `durable-fs/tests/scrub_refusal_matrix.rs` | 193 | std_fs=2 |
| `durable-fs/tests/vfs_conformance.rs` | 37 | panic=9 |
| `ports/src/vfs.rs` | 403 | async=1 |
| `store/src/fsync.rs` | 128 | panic=7, async=8, std_fs=1 |
| `store/src/io_seam.rs` | 397 | async=20, std_fs=15 |
| `store/src/io_seam/real_fs.rs` | 121 | std_fs=9 |
| `store/src/io_seam/store_io.rs` | 153 | async=20 |
| `store/src/io_seam/types.rs` | 91 | std_fs=1 |
| `testkit/src/conformance/vfs.rs` | 201 |  |
| `testkit/src/fault_vfs.rs` | 457 |  |
