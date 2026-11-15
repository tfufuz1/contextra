# Extraktion fuer `acta-sys`

**Aufgabe:** Einziges unsafe: mlock, MADV_DONTDUMP, SecureBuf, Sandbox, SIMD-Kerne, Hostpruefungen  
**Level:** L1 · **Budget:** 1600 LOC · **Spezifikation:** docs/SPEC.md Kap. 4.6, 13.1, 13.2  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 35 Dateien, 9931 Zeilen (inkl. Tests)  

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
| `simd/benches/simd_bench.rs` | 194 | float=7, panic=11 |
| `simd/src/contract.rs` | 66 | float=7 |
| `simd/src/dispatch.rs` | 1200 | float=64, unsafe=44 |
| `simd/src/error.rs` | 43 |  |
| `simd/src/kernels/avx2.rs` | 671 | float=39, unsafe=108 |
| `simd/src/kernels/avx512.rs` | 681 | float=40, unsafe=113 |
| `simd/src/kernels/mod.rs` | 48 |  |
| `simd/src/kernels/neon.rs` | 177 | float=16, unsafe=25 |
| `simd/src/kernels/scalar.rs` | 381 | float=56 |
| `simd/src/lib.rs` | 203 | float=19, serde=1, unsafe=1 |
| `simd/src/metric.rs` | 23 | serde=3 |
| `simd/tests/building_block_conformance.rs` | 92 | float=2 |
| `simd/tests/differential_tests.rs` | 574 | float=22, panic=32 |
| `simd/tests/edge_cases_tests.rs` | 557 | float=20, panic=37 |
| `simd/tests/golden_distance_oracle.rs` | 209 | float=14, panic=14 |
| `simd/tests/kernel_reference_tests.rs` | 357 | float=19, panic=30 |
| `simd/tests/quantized_and_mixed_precision_tests.rs` | 339 | float=26, panic=11 |
| `simd/tests/simd_cross_validation.rs` | 459 | float=9, panic=27, liste_a=1 |
| `sys/benches/sys_bench.rs` | 168 | panic=20, std_fs=8 |
| `sys/src/acl_win32.rs` | 518 | unsafe=7, std_fs=19 |
| `sys/src/contract.rs` | 44 |  |
| `sys/src/durability.rs` | 487 | unsafe=4, std_fs=24 |
| `sys/src/error.rs` | 95 |  |
| `sys/src/lib.rs` | 90 | unsafe=2, alt_spec=1 |
| `sys/src/mlock.rs` | 328 | unsafe=5 |
| `sys/src/mmap.rs` | 305 | unsafe=2, std_fs=17, alt_spec=3 |
| `sys/src/posix.rs` | 216 | unsafe=4, std_fs=5 |
| `sys/src/vault.rs` | 305 | unsafe=1 |
| `sys/tests/error_paths.rs` | 107 | panic=12, std_fs=6 |
| `sys/tests/integration_tests.rs` | 234 | panic=27, std_fs=10 |
| `vault/src/error.rs` | 21 |  |
| `vault/src/lib.rs` | 21 |  |
| `vault/src/vault.rs` | 429 | zeit_zufall=1, hr6=1, alt_spec=4 |
| `vault/tests/vault_hardening_test.rs` | 66 | hub=1 |
| `vault/tests/vault_tests.rs` | 223 | panic=13, hub=2, alt_spec=3 |
