# Extraktion fuer `acta-vec`

**Aufgabe:** Segmentierter exakter i8-Vektorindex mit Lineage  
**Level:** L3 · **Budget:** 4000 LOC · **Spezifikation:** docs/SPEC.md Kap. 9.3  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-gate, acta-vault, acta-seg  
**Umfang des Archivs:** 40 Dateien, 5645 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `vector` | Exakte i8-Flat-Suche (9.3): SQ8 auf i8 + norm2 umstellen; Distanz ganzzahlig |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 390 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 13 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 18 | tokio/async (G-NOTOKIO: Kern synchron) |
| unsafe | 10 | unsafe (G-UNSAFE: nur acta-sys) |
| panic | 158 | unwrap/expect/panic (G-LINT) |
| std_fs | 1 | std::fs direkt (G-FS: nur acta-fs) |
| hr6 | 5 | include!/#[deprecated] (HR-6) |
| hub | 15 | Hub-Import ex types/ports (HR-2/HR-3) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `ahash`, `blake3`, `parking_lot`, `tracing` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-vec/`) | LOC | Befunde |
|---|---:|---|
| `vector/src/casting.rs` | 87 | float=4 |
| `vector/src/compat/purge.rs` | 242 | float=2, hr6=2, hub=2 |
| `vector/src/compat/scanner.rs` | 341 | float=3, hr6=2, hub=2 |
| `vector/src/compute_pool.rs` | 295 | panic=5 |
| `vector/src/contract.rs` | 156 | float=4 |
| `vector/src/distance/dispatch.rs` | 116 | float=20, unsafe=1 |
| `vector/src/distance/inject.rs` | 173 | float=30, unsafe=1 |
| `vector/src/distance/kernel.rs` | 82 | float=37, unsafe=1 |
| `vector/src/distance/mod.rs` | 115 | float=2, unsafe=1, panic=4 |
| `vector/src/distance/scalar.rs` | 412 | float=81, unsafe=1 |
| `vector/src/distance/u8_ops.rs` | 49 | float=9, unsafe=1 |
| `vector/src/distance/validate.rs` | 69 | float=7, unsafe=1 |
| `vector/src/error.rs` | 95 | float=1 |
| `vector/src/fixed_hasher.rs` | 50 |  |
| `vector/src/lib.rs` | 131 |  |
| `vector/src/quantize/decode.rs` | 47 | float=4, unsafe=1 |
| `vector/src/quantize/distance.rs` | 104 | float=8, unsafe=1 |
| `vector/src/quantize/encode.rs` | 89 | float=7, unsafe=1 |
| `vector/src/quantize/mod.rs` | 117 | float=12, serde=3 |
| `vector/src/quantize/tests.rs` | 554 | float=52, serde=2, panic=26 |
| `vector/src/quantize/train.rs` | 196 | float=25, hr6=1 |
| `vector/src/types.rs` | 93 | float=5, serde=4 |
| `vector/tests/distance_determinism.rs` | 82 | float=2, panic=3, hub=1 |
| `vector/tests/distance_determinism_test.rs` | 33 | float=4 |
| `vector/tests/golden/distance_oracle.json` | 254 |  |
| `vector/tests/golden/hnsw_raw.bin` | 0 |  |
| `vector/tests/golden/hnsw_sq8.bin` | 0 |  |
| `vector/tests/golden/hnsw_tombstones.bin` | 0 |  |
| `vector/tests/golden/level_rng_golden.json` | 317 |  |
| `vector/tests/golden/ranking_golden.json` | 22 |  |
| `vector/tests/golden_distance_oracle.rs` | 181 | float=14, panic=10 |
| `vector/tests/golden_test.rs` | 420 | float=15, async=12, panic=54, std_fs=1, hub=4 |
| `vector/tests/nan_validation_policy.rs` | 199 | float=12, async=6, panic=10, hub=2 |
| `vector/tests/nan_validation_test.rs` | 31 | float=3 |
| `vector/tests/proptest_distance_quantize.rs` | 106 | float=6, panic=5, hub=1 |
| `vector/tests/s36_distance_metric_conformance.rs` | 41 | serde=2, panic=2, hub=1 |
| `vector/tests/simd_differential_test.rs` | 67 | float=3, panic=5, hub=2 |
| `vector/tests/sq8_error_bound_proptest.rs` | 39 | panic=4 |
| `vector/tests/wp06_distance_contract.rs` | 123 | float=5, panic=17 |
| `vector/tests/wp06_sq8_bounds.rs` | 117 | float=13, serde=2, panic=13 |
