# Extraktion fuer `acta-vec`

**Aufgabe:** Segmentierter exakter i8-Vektorindex mit Lineage  
**Level:** L3 · **Budget:** 4000 LOC · **Spezifikation:** docs/SPEC.md Kap. 9.3  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-gate, acta-vault, acta-seg  
**Umfang des Archivs:** 36 Dateien, 5399 Zeilen (inkl. Tests)  

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
| `ports/src/vector_index.rs` | 381 | panic=5, async=29, float=12, serde=3 |
| `vector/src/casting.rs` | 86 | float=4 |
| `vector/src/compat/purge.rs` | 241 | float=2 |
| `vector/src/compat/scanner.rs` | 340 | float=3 |
| `vector/src/compute_pool.rs` | 294 | panic=5 |
| `vector/src/contract.rs` | 155 | float=4 |
| `vector/src/distance/dispatch.rs` | 115 | float=20, unsafe=1 |
| `vector/src/distance/inject.rs` | 172 | float=30, unsafe=1 |
| `vector/src/distance/kernel.rs` | 81 | float=37, unsafe=1 |
| `vector/src/distance/mod.rs` | 114 | panic=4, float=2, unsafe=1 |
| `vector/src/distance/scalar.rs` | 411 | float=81, unsafe=1 |
| `vector/src/distance/u8_ops.rs` | 48 | float=9, unsafe=1 |
| `vector/src/distance/validate.rs` | 68 | float=7, unsafe=1 |
| `vector/src/error.rs` | 94 | float=1 |
| `vector/src/fixed_hasher.rs` | 49 |  |
| `vector/src/lib.rs` | 130 |  |
| `vector/src/quantize/decode.rs` | 46 | float=4, unsafe=1 |
| `vector/src/quantize/distance.rs` | 103 | float=8, unsafe=1 |
| `vector/src/quantize/encode.rs` | 88 | float=7, unsafe=1 |
| `vector/src/quantize/mod.rs` | 116 | float=12, serde=3 |
| `vector/src/quantize/tests.rs` | 553 | panic=26, float=52, serde=2 |
| `vector/src/quantize/train.rs` | 195 | float=25 |
| `vector/src/types.rs` | 92 | float=5, serde=4 |
| `vector/tests/distance_determinism.rs` | 81 | panic=3, float=2 |
| `vector/tests/distance_determinism_test.rs` | 32 | float=4 |
| `vector/tests/golden/distance_oracle.json` | 0 |  |
| `vector/tests/golden_distance_oracle.rs` | 180 | panic=10, float=14 |
| `vector/tests/golden_test.rs` | 419 | panic=54, async=12, float=15, std_fs=1 |
| `vector/tests/nan_validation_policy.rs` | 198 | panic=10, async=6, float=12 |
| `vector/tests/nan_validation_test.rs` | 30 | float=3 |
| `vector/tests/proptest_distance_quantize.rs` | 105 | panic=5, float=6 |
| `vector/tests/s36_distance_metric_conformance.rs` | 40 | panic=2, serde=2 |
| `vector/tests/simd_differential_test.rs` | 66 | panic=5, float=3 |
| `vector/tests/sq8_error_bound_proptest.rs` | 38 | panic=4 |
| `vector/tests/wp06_distance_contract.rs` | 122 | panic=17, float=5 |
| `vector/tests/wp06_sq8_bounds.rs` | 116 | panic=13, float=13, serde=2 |
