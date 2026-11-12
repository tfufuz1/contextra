# Audit- & Messreport: `contextra-py`

- **Crate-Pfad:** `crates/contextra-py`
- **Gesamtstatus:** **FEHLGESCHLAGEN (FAIL)**
- **Test-Statistik:** 0 bestanden, 0 fehlgeschlagen, 0 ignoriert
- **Kompilierung (cargo check):** ❌ Fehlgeschlagen (3.3987s)

---

## 1. Komponenten & Abhängigkeiten

- **Abhängigkeiten (10):** `contextra-types`, `contextra`, `contextra-wire`, `tokio`, `parking_lot`, `flatbuffers`, `serde_json`, `pyo3`, `numpy`, `pythonize`
- **Features (4):** `default`, `pyo3`, `docid-128`, `test-hooks`

---

## 2. Test-Ergebnisse im Detail

### Unit-Tests (`--lib`)
- **Status:** ❌ Nicht bestanden / Fehler
- **Laufzeit:** 0s
- **Tests bestanden:** 0
- **Tests fehlgeschlagen:** 0
- **Tests ignoriert:** 0

### Integrationstests (`--tests`)
_Keine separaten Integrationstest-Targets deklariert._

### Dokumentationstests (`--doc`)
- **Status:** ❌ Fehler / Keine
- **Laufzeit:** 0s
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
```
