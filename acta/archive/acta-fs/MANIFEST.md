# Extraktion fuer `acta-fs`

**Aufgabe:** Vfs-Port, DurableVfs, FaultVfs, Scrub, O_DIRECT-Ruecklesen  
**Level:** L1 · **Budget:** 2500 LOC · **Spezifikation:** docs/SPEC.md Kap. 3.7, 5.6  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 28 Dateien, 5025 Zeilen (inkl. Tests)  

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
| `durable-fs/benches/fs_bench.rs` | 46 | panic=6 |
| `durable-fs/src/error.rs` | 74 | alt_spec=2 |
| `durable-fs/src/fault_injector.rs` | 301 | alt_spec=1 |
| `durable-fs/src/health.rs` | 140 | alt_spec=3 |
| `durable-fs/src/journal.rs` | 437 | std_fs=4, alt_spec=1 |
| `durable-fs/src/lib.rs` | 456 | std_fs=12, alt_spec=5 |
| `durable-fs/src/platform.rs` | 49 | std_fs=4 |
| `durable-fs/src/scrub.rs` | 319 | float=2, zeit_zufall=1, std_fs=8 |
| `durable-fs/src/vfs.rs` | 142 | std_fs=6 |
| `durable-fs/tests/basic.rs` | 51 | std_fs=2 |
| `durable-fs/tests/crash_matrix_exhaustive.rs` | 227 | panic=24, std_fs=4 |
| `durable-fs/tests/durability_tests.rs` | 93 | std_fs=7, alt_spec=1 |
| `durable-fs/tests/fault_injector_tests.rs` | 311 | std_fs=2 |
| `durable-fs/tests/inv_crash_matrix.rs` | 75 | panic=14, std_fs=2 |
| `durable-fs/tests/journal_crash_resume.rs` | 60 | std_fs=1 |
| `durable-fs/tests/journal_property_tests.rs` | 120 | panic=23, std_fs=4 |
| `durable-fs/tests/p03_health_state_tests.rs` | 84 | alt_spec=2 |
| `durable-fs/tests/scrub_and_remove.rs` | 141 | std_fs=2 |
| `durable-fs/tests/scrub_journal_tests.rs` | 112 | std_fs=6 |
| `durable-fs/tests/scrub_refusal_matrix.rs` | 194 | std_fs=11 |
| `durable-fs/tests/vfs_conformance.rs` | 38 | panic=9 |
| `store/src/fsync.rs` | 129 | async=8, panic=7, std_fs=2 |
| `store/src/io_seam.rs` | 398 | async=20, std_fs=24 |
| `store/src/io_seam/real_fs.rs` | 122 | std_fs=15, alt_spec=1 |
| `store/src/io_seam/store_io.rs` | 154 | async=20, std_fs=4 |
| `store/src/io_seam/types.rs` | 92 | std_fs=4 |
| `testkit/src/conformance/vfs.rs` | 202 | hub=1 |
| `testkit/src/fault_vfs.rs` | 458 | hub=23 |
