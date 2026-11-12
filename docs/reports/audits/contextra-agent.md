# Audit- & Messreport: `contextra-agent`

- **Crate-Pfad:** `crates/contextra-agent`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 0 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ❌ Fehlgeschlagen (3.2948s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (12):** `contextra-types`, `contextra-ports`, `bytes`, `contextra-db`, `contextra-checkpoint`, `contextra-router`, `serde`, `serde_json`, `tracing`, `tokio`, `tokio-util`, `thiserror`
- **Features (2):** `default`, `test-utils`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ❌ Nicht bestanden / Fehler
- **Laufzeit:** 3.5103s
- **Tests bestanden:** 0
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ❌ FAIL | 0 | 0 | 3.4311s |

### Dokumentationstests (`--doc`)
- **Status:** ❌ Fehler / Keine
- **Laufzeit:** 3.5534s
- **Tests bestanden:** 0

---

## 3. Performance & Benchmark-Messungen

_Keine expliziten Criterion-Benchmarks in diesem Crate vorhanden._

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
