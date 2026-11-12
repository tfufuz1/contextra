# Audit- & Messreport: `contextra-cognition`

- **Crate-Pfad:** `crates/contextra-cognition`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 0 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ❌ Fehlgeschlagen (3.4973s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (15):** `contextra-engine`, `contextra-types`, `contextra-ports`, `contextra-graph`, `bytes`, `ahash`, `serde_json`, `serde`, `tokio`, `tracing`, `parking_lot`, `tokio-util`, `async-trait`, `thiserror`, `blake3`
- **Features (3):** `default`, `graph-connectivity-health`, `edge-reinforcement-learning`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ❌ Nicht bestanden / Fehler
- **Laufzeit:** 3.6568s
- **Tests bestanden:** 0
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
| Test-Target | Status | Bestanden | Fehlgeschlagen | Laufzeit (s) |
|---|---|---|---|---|
| `__all__` | ❌ FAIL | 0 | 0 | 3.6189s |

### Dokumentationstests (`--doc`)
- **Status:** ❌ Fehler / Keine
- **Laufzeit:** 3.5646s
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
