# Extraktion fuer `acta-sys`

**Aufgabe:** Einziges unsafe: mlock, MADV_DONTDUMP, SecureBuf, Sandbox, SIMD-Kerne, Hostpruefungen  
**Level:** L1 · **Budget:** 1600 LOC · **Spezifikation:** docs/SPEC.md Kap. 4.6, 13.1, 13.2  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 35 Dateien, 9896 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `sys` | einzige unsafe-Insel (G-UNSAFE); acl_win32 entfernt (nur Linux) |
| `vault` | SecureBuf (mlock, feste Kapazität; 4.6) |
| `simd` | SIMD-Kerne (unsafe) gehoeren nach acta-sys; i8-Dot fuer acta-vec |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 360 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 4 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| unsafe | 316 | unsafe (G-UNSAFE: nur acta-sys) |
| zeit_zufall | 1 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 234 | unwrap/expect/panic (G-LINT) |
| std_fs | 89 | std::fs direkt (G-FS: nur acta-fs) |
| hr6 | 1 | include!/#[deprecated] (HR-6) |
| hub | 3 | Hub-Import ex types/ports (HR-2/HR-3) |
| alt_spec | 11 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |
| liste_a | 1 | verbotene Formulierung (1.6, Liste A) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `memmap2`, `tracing` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-sys/`) | LOC | Befunde |
|---|---:|---|
| `simd/benches/simd_bench.rs` | 193 | panic=11, float=7 |
| `simd/src/contract.rs` | 65 | float=7 |
| `simd/src/dispatch.rs` | 1199 | float=64, unsafe=44 |
| `simd/src/error.rs` | 42 |  |
| `simd/src/kernels/avx2.rs` | 670 | float=39, unsafe=108 |
| `simd/src/kernels/avx512.rs` | 680 | float=40, unsafe=113 |
| `simd/src/kernels/mod.rs` | 47 |  |
| `simd/src/kernels/neon.rs` | 176 | float=16, unsafe=25 |
| `simd/src/kernels/scalar.rs` | 380 | float=56 |
| `simd/src/lib.rs` | 202 | float=19, serde=1, unsafe=1 |
| `simd/src/metric.rs` | 22 | serde=3 |
| `simd/tests/building_block_conformance.rs` | 91 | float=2 |
| `simd/tests/differential_tests.rs` | 573 | panic=32, float=22 |
| `simd/tests/edge_cases_tests.rs` | 556 | panic=37, float=20 |
| `simd/tests/golden_distance_oracle.rs` | 208 | panic=14, float=14 |
| `simd/tests/kernel_reference_tests.rs` | 356 | panic=30, float=19 |
| `simd/tests/quantized_and_mixed_precision_tests.rs` | 338 | panic=11, float=26 |
| `simd/tests/simd_cross_validation.rs` | 458 | panic=27, float=9 |
| `sys/benches/sys_bench.rs` | 167 | panic=20, std_fs=1 |
| `sys/src/acl_win32.rs` | 517 | std_fs=19, unsafe=7 |
| `sys/src/contract.rs` | 43 |  |
| `sys/src/durability.rs` | 486 | std_fs=15, unsafe=4 |
| `sys/src/error.rs` | 94 |  |
| `sys/src/lib.rs` | 89 | unsafe=2 |
| `sys/src/mlock.rs` | 327 | unsafe=5 |
| `sys/src/mmap.rs` | 304 | std_fs=6, unsafe=2 |
| `sys/src/posix.rs` | 215 | std_fs=3, unsafe=4 |
| `sys/src/vault.rs` | 304 | unsafe=1 |
| `sys/tests/error_paths.rs` | 106 | panic=12, std_fs=1 |
| `sys/tests/integration_tests.rs` | 233 | panic=27, std_fs=1 |
| `vault/src/error.rs` | 20 |  |
| `vault/src/lib.rs` | 20 |  |
| `vault/src/vault.rs` | 428 |  |
| `vault/tests/vault_hardening_test.rs` | 65 |  |
| `vault/tests/vault_tests.rs` | 222 | panic=13 |
