# Audit- & Messreport: `contextra-bench`

- **Crate-Pfad:** `benchmarks/contextra-bench`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 0 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ❌ Fehlgeschlagen (4.9933s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (15):** `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-db`, `tokio`, `serde`, `serde_json`, `tempfile`, `tracing`, `tracing-subscriber`, `anyhow`, `hdrhistogram`, `rand`, `contextra-rank`, `contextra-types`
- **Features (3):** `datasets`, `default`, `external-benchmarks`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ❌ Nicht bestanden / Fehler
- **Laufzeit:** 3.6286s
- **Tests bestanden:** 0
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ❌ FAIL | 0 | 0 | 3.6175s |

### Dokumentationstests (`--doc`)
- **Status:** ❌ Fehler / Keine
- **Laufzeit:** 3.7273s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

| Benchmark-Target | Status | Ausführungszeit (s) |
|---|---|---|
| `competitive_kv` | ⏱️ TIMEOUT | 20.0205s |

---

## 4. Aufgetretene Fehler & Compiler-Befunde

### Compiler- & Laufzeitfehler:
```text
71 | use crate::error::{StoreError, StoreResult as Result};
warning: unused import: `crate::error::StoreResult as Result`
13 | use crate::error::StoreResult as Result;
warning: unused import: `crate::error::StoreResult as Result`
14 | use crate::error::StoreResult as Result;
error[E0428]: the name `types` is defined multiple times
error: could not compile `contextra-graph` (lib) due to 1 previous error; 4 warnings emitted
71 | use crate::error::{StoreError, StoreResult as Result};
warning: unused import: `crate::error::StoreResult as Result`
13 | use crate::error::StoreResult as Result;
warning: unused import: `crate::error::StoreResult as Result`
14 | use crate::error::StoreResult as Result;
error[E0428]: the name `types` is defined multiple times
error: could not compile `contextra-graph` (lib) due to 1 previous error; 4 warnings emitted
[tests] 71 | use crate::error::{StoreError, StoreResult as Result};
[tests] warning: unused import: `crate::error::StoreResult as Result`
[tests] 13 | use crate::error::StoreResult as Result;
[tests] warning: unused import: `crate::error::StoreResult as Result`
[tests] 14 | use crate::error::StoreResult as Result;
[tests] error[E0428]: the name `types` is defined multiple times
[tests] error: could not compile `contextra-graph` (lib) due to 1 previous error; 4 warnings emitted
```
