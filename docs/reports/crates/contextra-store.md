# Audit- & Messreport: `contextra-store`

- **Crate-Pfad:** `crates/contextra-store`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 0 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ✅ Erfolgreich (1.5757s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (22):** `contextra-ids`, `contextra-env`, `contextra-types`, `contextra-ports`, `contextra-sys`, `contextra-mvcc`, `ahash`, `quick_cache`, `contextra-crypto`, `bytes`, `blake3`, `crc32fast`, `parking_lot`, `serde`, `tokio`, `memmap2`, `tracing`, `thiserror`, `rand`, `lru`, `uuid`, `tokio-util`
- **Features (10):** `default`, `workspace`, `test-hooks`, `inspect`, `memory-only-storage`, `fault-injection`, `block-cache-v2`, `sieve-cache`, `docid-128`, `legacy-wal-key`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ❌ Nicht bestanden / Fehler
- **Laufzeit:** 9.7696s
- **Tests bestanden:** 0
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ❌ FAIL | 0 | 0 | 12.5553s |

### Dokumentationstests (`--doc`)
- **Status:** ✅ Bestanden
- **Laufzeit:** 0.8565s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

| Benchmark-Target | Status | Ausführungszeit (s) |
|---|---|---|
| `sstable_bench` | ⏱️ TIMEOUT | 20.0194s |
| `memtable_bench` | ⏱️ TIMEOUT | 20.0323s |
| `wal_bench` | ⏱️ TIMEOUT | 20.0307s |
| `lsm_concurrency_bench` | ⏱️ TIMEOUT | 20.0328s |
| `block_cache_bench` | ⏱️ TIMEOUT | 20.0302s |
| `redb_comparative_bench` | ⏱️ TIMEOUT | 20.0245s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

### Compiler- & Laufzeitfehler:
```text
error[E0432]: unresolved import `crate::StorageEngine`
error[E0432]: unresolved import `contextra_env::ManualClock`
error[E0432]: unresolved import `crate::StorageEngine`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
error[E0433]: failed to resolve: use of undeclared type `ContextraError`
71 | use crate::error::{StoreError, StoreResult as Result};
warning: unused import: `crate::error::StoreResult as Result`
13 | use crate::error::StoreResult as Result;
warning: unused import: `crate::error::StoreResult as Result`
14 | use crate::error::StoreResult as Result;
warning: unused import: `crate::error::StoreError`
7 | use crate::error::StoreError;
8 | use crate::error::{StoreError, StoreResult as Result};
error: could not compile `contextra-store` (lib test) due to 20 previous errors; 21 warnings emitted
[tests] 71 | use crate::error::{StoreError, StoreResult as Result};
```
