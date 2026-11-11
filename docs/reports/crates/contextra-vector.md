# Audit- & Messreport: `contextra-vector`

- **Crate-Pfad:** `crates/contextra-vector`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 0 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (2.2471s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (18):** `contextra-ids`, `contextra-env`, `contextra-types`, `contextra-ports`, `contextra-mvcc`, `contextra-simd`, `contextra-sys`, `subtle`, `ahash`, `parking_lot`, `rand`, `roaring`, `tracing`, `memmap2`, `serde`, `zeroize`, `thiserror`, `blake3`
- **Features (19):** `default`, `std`, `simd`, `serde`, `tracing`, `inspect`, `experimental`, `mmap`, `diskann`, `quantize-rabitq`, `workspace`, `ports-compat`, `docid-128`, `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search`, `graph`, `experimental-all`, `partial-index-rebuild`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ❌ Nicht bestanden / Fehler
- **Laufzeit:** 8.123s
- **Tests bestanden:** 0
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ❌ FAIL | 0 | 0 | 4.1888s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.5993s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

| Benchmark-Target | Status | Ausführungszeit (s) |
|---|---|---|
| `distance_bench` | ⏱️ TIMEOUT | 20.0191s |
| `sq8_bench` | ⏱️ TIMEOUT | 20.0287s |
| `hnsw_bench` | ⏱️ TIMEOUT | 20.0332s |
| `hnsw_delete_search_bench` | ⏱️ TIMEOUT | 20.0294s |
| `audit_benchmarks` | ⏱️ TIMEOUT | 20.011s |
| `flush_threshold_amplification` | ⏱️ TIMEOUT | 20.0289s |
| `partial_rebuild_recall_bench` | ❌ FAIL | 0.3641s |
| `distance_nan_impact_bench` | ⏱️ TIMEOUT | 20.0286s |
| `filtered_search_selectivity_bench` | ❌ FAIL | 0.359s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

### Compiler- & Laufzeitfehler:
```text
error[E0599]: no method named `insert` found for struct `GenericHnswIndex` in the current scope
error[E0599]: no method named `commit` found for struct `GenericHnswIndex` in the current scope
error[E0599]: no method named `insert` found for struct `GenericHnswIndex` in the current scope
error[E0599]: no method named `commit` found for struct `GenericHnswIndex` in the current scope
error[E0599]: no method named `search` found for struct `GenericHnswIndex` in the current scope
error[E0599]: no method named `insert` found for struct `GenericHnswIndex` in the current scope
error[E0599]: no method named `commit` found for struct `GenericHnswIndex` in the current scope
error: could not compile `contextra-vector` (lib test) due to 7 previous errors; 15 warnings emitted
[tests] error[E0599]: no method named `insert` found for struct `GenericHnswIndex` in the current scope
[tests] error[E0599]: no method named `commit` found for struct `GenericHnswIndex` in the current scope
[tests] error[E0599]: no method named `insert` found for struct `GenericHnswIndex` in the current scope
[tests] error[E0599]: no method named `commit` found for struct `GenericHnswIndex` in the current scope
[tests] error[E0599]: no method named `search` found for struct `GenericHnswIndex` in the current scope
[tests] error[E0599]: no method named `insert` found for struct `GenericHnswIndex` in the current scope
[tests] error[E0599]: no method named `commit` found for struct `GenericHnswIndex` in the current scope
[tests] error: could not compile `contextra-vector` (lib test) due to 7 previous errors; 15 warnings emitted
```
